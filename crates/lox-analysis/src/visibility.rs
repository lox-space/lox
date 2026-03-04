// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use glam::DVec3;
use lox_bodies::{DynOrigin, Origin, TryMeanRadius, TrySpheroid, UndefinedOriginPropertyError};
use lox_ephem::Ephemeris;
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::{DynRotationError, RotationError, TryRotation};
use lox_frames::{DynFrame, ReferenceFrame};
use lox_math::series::{InterpolationType, Series, SeriesError};
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::{DynTimeScale, Tai, Tdb, TimeScale};
use rayon::prelude::*;
use std::f64::consts::PI;
use thiserror::Error;

use lox_core::units::{AngularRate, Distance};

use crate::assets::{AssetId, GroundStation, Scenario, Spacecraft};
use lox_orbits::events::{
    DetectError, DetectFn, EventsToIntervals, IntervalDetector, IntervalDetectorExt,
    RootFindingDetector,
};
use lox_orbits::ground::{DynGroundLocation, Observables};
use lox_orbits::orbits::{Ensemble, Trajectory};

// ---------------------------------------------------------------------------
// Line-of-sight geometry
// ---------------------------------------------------------------------------

// Salvatore Alfano, David Negron, Jr., and Jennifer L. Moore
// Rapid Determination of Satellite Visibility Periods
// The Journal of the Astronautical Sciences. Vol. 40, No. 2, April-June 1992, pp. 281-296
pub fn line_of_sight(radius: f64, r1: DVec3, r2: DVec3) -> f64 {
    let r1n = r1.length();
    let r2n = r2.length();
    let theta1 = radius / r1n;
    let theta2 = radius / r2n;
    // Clamp to the domain of `acos` to avoid floating point errors when `r1 == r2`.
    let theta = (r1.dot(r2) / r1n / r2n).clamp(-1.0, 1.0);
    theta1.acos() + theta2.acos() - theta.acos()
}

pub fn line_of_sight_spheroid(
    mean_radius: f64,
    radius_eq: f64,
    radius_p: f64,
    r1: DVec3,
    r2: DVec3,
) -> f64 {
    let eps = (1.0 - radius_p.powi(2) / radius_eq.powi(2)).sqrt();
    let scale = (1.0 - eps.powi(2)).sqrt();
    let r1 = DVec3::new(r1.x, r1.y, r1.z / scale);
    let r2 = DVec3::new(r2.x, r2.y, r2.z / scale);
    line_of_sight(mean_radius, r1, r2)
}

pub trait LineOfSight: TrySpheroid + TryMeanRadius {
    fn line_of_sight(&self, r1: DVec3, r2: DVec3) -> Result<f64, UndefinedOriginPropertyError> {
        let mean_radius = self.try_mean_radius()?.to_meters();
        if let (Ok(r_eq), Ok(r_p)) = (self.try_equatorial_radius(), self.try_polar_radius()) {
            return Ok(line_of_sight_spheroid(
                mean_radius,
                r_eq.to_meters(),
                r_p.to_meters(),
                r1,
                r2,
            ));
        }
        Ok(line_of_sight(mean_radius, r1, r2))
    }
}

impl<T: TrySpheroid + TryMeanRadius> LineOfSight for T {}

// ---------------------------------------------------------------------------
// Elevation mask
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Error, PartialEq)]
pub enum ElevationMaskError {
    #[error("invalid azimuth range: {}..{}", .0.to_degrees(), .1.to_degrees())]
    InvalidAzimuthRange(f64, f64),
    #[error("series error")]
    SeriesError(#[from] SeriesError),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ElevationMask {
    Fixed(f64),
    Variable(Series),
}

impl ElevationMask {
    pub fn new(azimuth: Vec<f64>, elevation: Vec<f64>) -> Result<Self, ElevationMaskError> {
        if !azimuth.is_empty() {
            let az_min = *azimuth.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
            let az_max = *azimuth.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
            if az_min != -PI || az_max != PI {
                return Err(ElevationMaskError::InvalidAzimuthRange(az_min, az_max));
            }
        }
        Ok(Self::Variable(Series::try_new(
            azimuth,
            elevation,
            InterpolationType::Linear,
        )?))
    }

    pub fn with_fixed_elevation(elevation: f64) -> Self {
        Self::Fixed(elevation)
    }

    pub fn min_elevation(&self, azimuth: f64) -> f64 {
        match self {
            ElevationMask::Fixed(min_elevation) => *min_elevation,
            ElevationMask::Variable(series) => series.interpolate(azimuth),
        }
    }
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum VisibilityError {
    #[error(transparent)]
    Detect(#[from] DetectError),
    #[error(transparent)]
    Series(#[from] SeriesError),
}

/// Error returned when computing passes for an invalid pair type.
#[derive(Debug, Error)]
pub enum PassError {
    #[error(
        "passes are not supported for inter-satellite pair ({0}, {1}): use intervals() instead"
    )]
    InterSatellitePair(String, String),
}

// ---------------------------------------------------------------------------
// Pass
// ---------------------------------------------------------------------------

/// A visibility pass between a ground station and spacecraft.
///
/// Stores the time interval, sampled times, observables, and `Series` for
/// each observable channel to support interpolation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pass<T: TimeScale> {
    interval: TimeInterval<T>,
    times: Vec<Time<T>>,
    observables: Vec<Observables>,
    azimuth_series: Series,
    elevation_series: Series,
    range_series: Series,
    range_rate_series: Series,
}

pub type DynPass = Pass<DynTimeScale>;

impl DynPass {
    /// Create a Pass from an interval, calculating observables for times when
    /// the satellite is above the elevation mask.
    ///
    /// Returns `None` if the satellite is never above the mask within the interval.
    pub fn from_interval(
        interval: TimeInterval<DynTimeScale>,
        time_resolution: TimeDelta,
        gs: &DynGroundLocation,
        mask: &ElevationMask,
        sc: &lox_orbits::orbits::DynTrajectory,
        body_fixed_frame: DynFrame,
    ) -> Option<DynPass> {
        let mut pass_times = Vec::new();
        let mut pass_observables = Vec::new();

        for current_time in interval.step_by(time_resolution) {
            let state = sc.interpolate_at(current_time);
            let state_bf = state
                .try_to_frame(body_fixed_frame, &DefaultRotationProvider)
                .unwrap();
            let obs = gs.observables_dyn(state_bf);

            let min_elev = mask.min_elevation(obs.azimuth());
            if obs.elevation() >= min_elev {
                pass_times.push(current_time);
                pass_observables.push(obs);
            }
        }

        if pass_times.is_empty() {
            return None;
        }

        Pass::try_new(interval, pass_times, pass_observables).ok()
    }
}

impl<T: TimeScale> Pass<T> {
    /// Create a new Pass with Series-based interpolation.
    ///
    /// Requires at least 2 data points so that the observables can be
    /// interpolated. Returns `Err(SeriesError::InsufficientPoints)` otherwise.
    pub fn try_new(
        interval: TimeInterval<T>,
        times: Vec<Time<T>>,
        observables: Vec<Observables>,
    ) -> Result<Self, SeriesError>
    where
        T: Copy,
    {
        if times.len() < 2 {
            return Err(SeriesError::InsufficientPoints(times.len()));
        }

        let time_seconds: Vec<f64> = times
            .iter()
            .map(|t| (*t - interval.start()).to_seconds().to_f64())
            .collect();
        let azimuths: Vec<f64> = observables.iter().map(|o| o.azimuth()).collect();
        let elevations: Vec<f64> = observables.iter().map(|o| o.elevation()).collect();
        let ranges: Vec<f64> = observables.iter().map(|o| o.range()).collect();
        let range_rates: Vec<f64> = observables.iter().map(|o| o.range_rate()).collect();

        let azimuth_series =
            Series::try_new(time_seconds.clone(), azimuths, InterpolationType::Linear)?;
        let elevation_series =
            Series::try_new(time_seconds.clone(), elevations, InterpolationType::Linear)?;
        let range_series =
            Series::try_new(time_seconds.clone(), ranges, InterpolationType::Linear)?;
        let range_rate_series =
            Series::try_new(time_seconds, range_rates, InterpolationType::Linear)?;

        Ok(Pass {
            interval,
            times,
            observables,
            azimuth_series,
            elevation_series,
            range_series,
            range_rate_series,
        })
    }

    pub fn interval(&self) -> &TimeInterval<T> {
        &self.interval
    }

    pub fn times(&self) -> &[Time<T>] {
        &self.times
    }

    pub fn observables(&self) -> &[Observables] {
        &self.observables
    }

    pub fn interpolate(&self, time: Time<T>) -> Option<Observables>
    where
        T: Copy + PartialOrd,
    {
        if time < self.interval.start() || time > self.interval.end() {
            return None;
        }

        if self.times.is_empty() {
            return None;
        }

        let target_seconds = (time - self.interval.start()).to_seconds().to_f64();

        let azimuth = self.azimuth_series.interpolate(target_seconds);
        let elevation = self.elevation_series.interpolate(target_seconds);
        let range = self.range_series.interpolate(target_seconds);
        let range_rate = self.range_rate_series.interpolate(target_seconds);

        Some(Observables::new(azimuth, elevation, range, range_rate))
    }
}

// ---------------------------------------------------------------------------
// DetectFn error type
// ---------------------------------------------------------------------------

/// Errors from detect function evaluation.
#[derive(Debug, Error)]
pub enum EvalError {
    #[error("rotation error: {0}")]
    Rotation(Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    UndefinedProperty(#[from] UndefinedOriginPropertyError),
    #[error("ephemeris error: {0}")]
    Ephemeris(Box<dyn std::error::Error + Send + Sync>),
}

impl From<DynRotationError> for EvalError {
    fn from(e: DynRotationError) -> Self {
        EvalError::Rotation(Box::new(e))
    }
}

impl From<RotationError> for EvalError {
    fn from(e: RotationError) -> Self {
        EvalError::Rotation(Box::new(e))
    }
}

// ---------------------------------------------------------------------------
// DetectFn implementations
// ---------------------------------------------------------------------------

/// Elevation above mask for a ground station / spacecraft pair.
///
/// Generic over origin `O` and frame `R`. The detect function:
/// 1. Interpolates the spacecraft trajectory at the given time
/// 2. Rotates the state into the body-fixed frame via `TryRotation<R, DynFrame, Tai>`
/// 3. Computes observables (azimuth, elevation, range, range rate)
/// 4. Returns elevation minus minimum elevation from the mask
struct ElevationDetectFn<'a, O: Origin, R: ReferenceFrame> {
    gs: &'a DynGroundLocation,
    mask: &'a ElevationMask,
    sc: &'a Trajectory<Tai, O, R>,
    body_fixed_frame: DynFrame,
}

impl<O, R> DetectFn<Tai> for ElevationDetectFn<'_, O, R>
where
    O: TrySpheroid + Copy,
    R: ReferenceFrame + Copy,
    DefaultRotationProvider: TryRotation<R, DynFrame, Tai>,
    <DefaultRotationProvider as TryRotation<R, DynFrame, Tai>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    type Error = EvalError;

    fn eval(&self, time: Time<Tai>) -> Result<f64, Self::Error> {
        let sc = self.sc.interpolate_at(time);
        let sc = sc
            .try_to_frame(self.body_fixed_frame, &DefaultRotationProvider)
            .map_err(|e| EvalError::Rotation(Box::new(e)))?;
        let obs = self.gs.compute_observables(sc.position(), sc.velocity());
        Ok(obs.elevation() - self.mask.min_elevation(obs.azimuth()))
    }
}

/// Line-of-sight between a ground station and spacecraft, relative to an
/// occulting body.
struct LineOfSightDetectFn<'a, O: Origin, R: ReferenceFrame, E> {
    gs: &'a DynGroundLocation,
    sc: &'a Trajectory<Tai, O, R>,
    body: DynOrigin,
    ephemeris: &'a E,
    body_fixed_frame: DynFrame,
}

impl<O, R, E: Ephemeris> DetectFn<Tai> for LineOfSightDetectFn<'_, O, R, E>
where
    O: TrySpheroid + Copy,
    R: ReferenceFrame + Copy,
    E::Error: 'static,
    DefaultRotationProvider: TryRotation<DynFrame, R, Tai>,
    <DefaultRotationProvider as TryRotation<DynFrame, R, Tai>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    type Error = EvalError;

    fn eval(&self, time: Time<Tai>) -> Result<f64, Self::Error> {
        // Convert Tai → Tdb for ephemeris lookup (infallible via DefaultOffsetProvider).
        let tdb = time.to_scale(Tdb);
        let r_body = self
            .ephemeris
            .position(tdb, self.sc.origin(), self.body)
            .map_err(|e| EvalError::Ephemeris(Box::new(e)))?;
        let r_sc = self.sc.interpolate_at(time).position() - r_body;
        // Compute ground station position in the scenario frame R by rotating
        // from body-fixed → R.
        let rot = DefaultRotationProvider
            .try_rotation(self.body_fixed_frame, self.sc.reference_frame(), time)
            .map_err(|e| EvalError::Rotation(Box::new(e)))?;
        let (r_gs_frame, _) = rot.rotate_state(self.gs.body_fixed_position(), DVec3::ZERO);
        let r_gs = r_gs_frame - r_body;
        Ok(self.body.line_of_sight(r_gs, r_sc)?)
    }
}

/// Line-of-sight between two spacecraft, relative to an occulting body.
struct InterSatelliteLosDetectFn<'a, O: Origin, R: ReferenceFrame, E> {
    sc1: &'a Trajectory<Tai, O, R>,
    sc2: &'a Trajectory<Tai, O, R>,
    body: DynOrigin,
    ephemeris: &'a E,
}

impl<O, R, E: Ephemeris> DetectFn<Tai> for InterSatelliteLosDetectFn<'_, O, R, E>
where
    O: Origin + Copy,
    R: ReferenceFrame + Copy,
    E::Error: 'static,
{
    type Error = EvalError;

    fn eval(&self, time: Time<Tai>) -> Result<f64, Self::Error> {
        let tdb = time.to_scale(Tdb);
        let r_body = self
            .ephemeris
            .position(tdb, self.sc1.origin(), self.body)
            .map_err(|e| EvalError::Ephemeris(Box::new(e)))?;
        let r_sc1 = self.sc1.interpolate_at(time).position() - r_body;
        let r_sc2 = self.sc2.interpolate_at(time).position() - r_body;
        Ok(self.body.line_of_sight(r_sc1, r_sc2)?)
    }
}

/// Direction for inter-satellite range threshold comparison.
enum RangeDirection {
    /// Positive when range < threshold (i.e. `threshold - range`).
    Max,
    /// Positive when range > threshold (i.e. `range - threshold`).
    Min,
}

/// Range threshold detector for inter-satellite pairs.
struct InterSatelliteRangeDetectFn<'a, O: Origin, R: ReferenceFrame> {
    sc1: &'a Trajectory<Tai, O, R>,
    sc2: &'a Trajectory<Tai, O, R>,
    threshold: Distance,
    direction: RangeDirection,
}

impl<O, R> DetectFn<Tai> for InterSatelliteRangeDetectFn<'_, O, R>
where
    O: Origin + Copy,
    R: ReferenceFrame + Copy,
{
    type Error = EvalError;

    fn eval(&self, time: Time<Tai>) -> Result<f64, Self::Error> {
        let r1 = self.sc1.interpolate_at(time).position();
        let r2 = self.sc2.interpolate_at(time).position();
        let range = (r1 - r2).length();
        let threshold = self.threshold.to_meters();
        Ok(match self.direction {
            RangeDirection::Max => threshold - range,
            RangeDirection::Min => range - threshold,
        })
    }
}

/// Slew rate (angular rate) threshold detector for inter-satellite pairs.
///
/// The angular rate ω = |r × v| / |r|² is symmetric between the two
/// spacecraft.  The detector returns `threshold - ω`, positive when the
/// angular rate is within the limit.
struct InterSatelliteSlewRateDetectFn<'a, O: Origin, R: ReferenceFrame> {
    sc1: &'a Trajectory<Tai, O, R>,
    sc2: &'a Trajectory<Tai, O, R>,
    threshold: AngularRate,
}

impl<O, R> DetectFn<Tai> for InterSatelliteSlewRateDetectFn<'_, O, R>
where
    O: Origin + Copy,
    R: ReferenceFrame + Copy,
{
    type Error = EvalError;

    fn eval(&self, time: Time<Tai>) -> Result<f64, Self::Error> {
        let s1 = self.sc1.interpolate_at(time);
        let s2 = self.sc2.interpolate_at(time);
        let r = s2.position() - s1.position();
        let v = s2.velocity() - s1.velocity();
        let r_len_sq = r.length_squared();
        let omega = if r_len_sq > 0.0 {
            r.cross(v).length() / r_len_sq
        } else {
            0.0
        };
        Ok(self.threshold.to_radians_per_second() - omega)
    }
}

// ---------------------------------------------------------------------------
// VisibilityResults
// ---------------------------------------------------------------------------

/// Distinguishes ground-to-space from inter-satellite visibility pairs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PairType {
    GroundSpace,
    InterSatellite,
}

type IntervalMap = HashMap<(AssetId, AssetId), Vec<TimeInterval<Tai>>>;
type PairTypeMap = HashMap<(AssetId, AssetId), PairType>;

/// Stores raw visibility intervals per asset pair.
///
/// This is the primary result type for visibility analysis. Intervals are
/// cheap to compute; conversion to [`Pass`] (with observables) happens
/// separately and on demand.
pub struct VisibilityResults {
    intervals: IntervalMap,
    pair_types: PairTypeMap,
}

impl VisibilityResults {
    /// Return all intervals for a specific pair.
    pub fn intervals_for(&self, id1: &AssetId, id2: &AssetId) -> Option<&[TimeInterval<Tai>]> {
        let key = (id1.clone(), id2.clone());
        self.intervals.get(&key).map(|v| v.as_slice())
    }

    /// Return all intervals keyed by pair ids.
    pub fn all_intervals(&self) -> &IntervalMap {
        &self.intervals
    }

    /// Iterate over all pair keys.
    pub fn pair_ids(&self) -> impl Iterator<Item = &(AssetId, AssetId)> {
        self.intervals.keys()
    }

    /// Return the [`PairType`] for a given pair, if present.
    pub fn pair_type(&self, id1: &AssetId, id2: &AssetId) -> Option<PairType> {
        self.pair_types.get(&(id1.clone(), id2.clone())).copied()
    }

    /// Return pair ids for ground-to-space pairs only.
    pub fn ground_space_pair_ids(&self) -> Vec<&(AssetId, AssetId)> {
        self.pair_types
            .iter()
            .filter(|&(_, &pt)| pt == PairType::GroundSpace)
            .map(|(k, _)| k)
            .collect()
    }

    /// Return pair ids for inter-satellite pairs only.
    pub fn inter_satellite_pair_ids(&self) -> Vec<&(AssetId, AssetId)> {
        self.pair_types
            .iter()
            .filter(|&(_, &pt)| pt == PairType::InterSatellite)
            .map(|(k, _)| k)
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn num_pairs(&self) -> usize {
        self.intervals.len()
    }

    /// Total number of visibility intervals across all pairs.
    pub fn total_intervals(&self) -> usize {
        self.intervals.values().map(|v| v.len()).sum()
    }

    /// Consume self and return the inner intervals and pair types maps.
    pub fn into_parts(self) -> (IntervalMap, PairTypeMap) {
        (self.intervals, self.pair_types)
    }

    /// Convert intervals for a specific ground-space pair to visibility passes.
    ///
    /// Returns an error if the pair is an inter-satellite pair, since passes
    /// with ground-station observables are not meaningful for such pairs.
    /// Returns an empty vec if the pair is not found.
    #[allow(clippy::too_many_arguments)]
    pub fn to_passes(
        &self,
        ground_id: &AssetId,
        space_id: &AssetId,
        gs: &DynGroundLocation,
        mask: &ElevationMask,
        sc: &lox_orbits::orbits::DynTrajectory,
        time_resolution: TimeDelta,
        body_fixed_frame: DynFrame,
    ) -> Result<Vec<DynPass>, PassError> {
        let key = (ground_id.clone(), space_id.clone());
        if self.pair_types.get(&key) == Some(&PairType::InterSatellite) {
            return Err(PassError::InterSatellitePair(
                ground_id.as_str().to_string(),
                space_id.as_str().to_string(),
            ));
        }
        Ok(self
            .intervals
            .get(&key)
            .map(|intervals| {
                intervals
                    .iter()
                    .filter_map(|interval| {
                        let dyn_interval = TimeInterval::new(
                            interval.start().into_dyn(),
                            interval.end().into_dyn(),
                        );
                        DynPass::from_interval(
                            dyn_interval,
                            time_resolution,
                            gs,
                            mask,
                            sc,
                            body_fixed_frame,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default())
    }
}

// ---------------------------------------------------------------------------
// VisibilityAnalysis
// ---------------------------------------------------------------------------

/// Computes ground-station-to-spacecraft and inter-satellite visibility.
///
/// Generic over origin `O`, reference frame `R`, and ephemeris `E`.
/// Ground-to-space pairs are always computed when ground assets are present.
/// Inter-satellite pairs are additionally computed when enabled via
/// [`with_inter_satellite`](Self::with_inter_satellite).
///
/// Trajectories are looked up from a pre-computed [`Ensemble`] by asset id.
pub struct VisibilityAnalysis<'a, O: Origin, R: ReferenceFrame, E> {
    scenario: &'a Scenario<O, R>,
    ensemble: &'a Ensemble<AssetId, Tai, O, R>,
    ephemeris: &'a E,
    occulting_bodies: Vec<DynOrigin>,
    step: TimeDelta,
    min_pass_duration: Option<TimeDelta>,
    inter_satellite: bool,
    min_range: Option<Distance>,
    max_range: Option<Distance>,
}

impl<'a, O, R, E> VisibilityAnalysis<'a, O, R, E>
where
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync + Into<DynOrigin>,
    R: ReferenceFrame + Copy + Send + Sync + Into<DynFrame>,
    E: Ephemeris + Send + Sync,
    E::Error: 'static,
    DefaultRotationProvider: TryRotation<R, DynFrame, Tai> + TryRotation<DynFrame, R, Tai>,
    <DefaultRotationProvider as TryRotation<R, DynFrame, Tai>>::Error:
        std::error::Error + Send + Sync + 'static,
    <DefaultRotationProvider as TryRotation<DynFrame, R, Tai>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    pub fn new(
        scenario: &'a Scenario<O, R>,
        ensemble: &'a Ensemble<AssetId, Tai, O, R>,
        ephemeris: &'a E,
    ) -> Self {
        Self {
            scenario,
            ensemble,
            ephemeris,
            occulting_bodies: Vec::new(),
            step: TimeDelta::from_seconds(60),
            min_pass_duration: None,
            inter_satellite: false,
            min_range: None,
            max_range: None,
        }
    }

    pub fn with_inter_satellite(mut self) -> Self {
        self.inter_satellite = true;
        self
    }

    pub fn with_occulting_bodies(mut self, bodies: Vec<DynOrigin>) -> Self {
        self.occulting_bodies = bodies;
        self
    }

    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = step;
        self
    }

    pub fn with_min_pass_duration(mut self, min_pass_duration: TimeDelta) -> Self {
        self.min_pass_duration = Some(min_pass_duration);
        self
    }

    pub fn with_min_range(mut self, min_range: Distance) -> Self {
        self.min_range = Some(min_range);
        self
    }

    pub fn with_max_range(mut self, max_range: Distance) -> Self {
        self.max_range = Some(max_range);
        self
    }

    pub fn step(&self) -> TimeDelta {
        self.step
    }

    /// Apply `min_pass_duration` → `coarse_step` conversion to a detector.
    fn apply_coarse_step<F>(&self, det: RootFindingDetector<F>) -> RootFindingDetector<F> {
        match self.min_pass_duration {
            Some(d) => {
                let coarse = TimeDelta::from_seconds_f64(d.to_seconds().to_f64() / 2.0);
                if coarse > self.step {
                    det.with_coarse_step(coarse)
                } else {
                    det
                }
            }
            None => det,
        }
    }

    /// Compute visibility intervals for a single (ground, space) pair.
    fn compute_pair(
        &self,
        gs: &GroundStation,
        sc_traj: &Trajectory<Tai, O, R>,
        interval: TimeInterval<Tai>,
    ) -> Result<Vec<TimeInterval<Tai>>, VisibilityError> {
        let body_fixed_frame = gs.body_fixed_frame();

        let make_elev = || {
            let det = RootFindingDetector::new(
                ElevationDetectFn {
                    gs: gs.location(),
                    mask: gs.mask(),
                    sc: sc_traj,
                    body_fixed_frame,
                },
                self.step,
            );
            EventsToIntervals::new(self.apply_coarse_step(det))
        };

        if self.occulting_bodies.is_empty() {
            return Ok(make_elev().detect(interval)?);
        }

        let make_los = |body: DynOrigin| {
            EventsToIntervals::new(self.apply_coarse_step(RootFindingDetector::new(
                LineOfSightDetectFn {
                    gs: gs.location(),
                    sc: sc_traj,
                    body,
                    ephemeris: self.ephemeris,
                    body_fixed_frame,
                },
                self.step,
            )))
        };

        let mut los: Box<dyn IntervalDetector<Tai> + '_> =
            Box::new(make_los(self.occulting_bodies[0]));
        for &body in &self.occulting_bodies[1..] {
            los = Box::new(los.intersect(make_los(body)));
        }

        Ok(make_elev().chain(los).detect(interval)?)
    }

    /// Compute LOS intervals for a single inter-satellite pair,
    /// optionally filtered by min/max range constraints.
    fn compute_inter_satellite_pair(
        &self,
        sc1: &Spacecraft,
        sc2: &Spacecraft,
        traj1: &Trajectory<Tai, O, R>,
        traj2: &Trajectory<Tai, O, R>,
        interval: TimeInterval<Tai>,
    ) -> Result<Vec<TimeInterval<Tai>>, VisibilityError> {
        let has_range = self.min_range.is_some() || self.max_range.is_some();
        let has_los = !self.occulting_bodies.is_empty();

        // Resolve per-pair slew rate limit: min of both assets' limits.
        let effective_slew_rate = match (sc1.max_slew_rate(), sc2.max_slew_rate()) {
            (Some(a), Some(b)) => Some(if a.to_radians_per_second() < b.to_radians_per_second() {
                a
            } else {
                b
            }),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        let has_slew_rate = effective_slew_rate.is_some();

        if !has_range && !has_slew_rate && !has_los {
            return Ok(vec![interval]);
        }

        let make_range = |threshold: Distance, direction: RangeDirection| {
            EventsToIntervals::new(self.apply_coarse_step(RootFindingDetector::new(
                InterSatelliteRangeDetectFn {
                    sc1: traj1,
                    sc2: traj2,
                    threshold,
                    direction,
                },
                self.step,
            )))
        };

        let make_los = |body: DynOrigin| {
            EventsToIntervals::new(self.apply_coarse_step(RootFindingDetector::new(
                InterSatelliteLosDetectFn {
                    sc1: traj1,
                    sc2: traj2,
                    body,
                    ephemeris: self.ephemeris,
                },
                self.step,
            )))
        };

        // Start with range constraints (cheapest: position-only).
        let mut detector: Option<Box<dyn IntervalDetector<Tai> + '_>> = None;

        if let Some(max) = self.max_range {
            detector = Some(Box::new(make_range(max, RangeDirection::Max)));
        }
        if let Some(min) = self.min_range {
            let min_det = make_range(min, RangeDirection::Min);
            detector = Some(match detector {
                Some(d) => Box::new(d.intersect(min_det)),
                None => Box::new(min_det),
            });
        }

        // Slew rate constraint (medium cost: position + velocity).
        if let Some(threshold) = effective_slew_rate {
            let slew = EventsToIntervals::new(self.apply_coarse_step(RootFindingDetector::new(
                InterSatelliteSlewRateDetectFn {
                    sc1: traj1,
                    sc2: traj2,
                    threshold,
                },
                self.step,
            )));
            detector = Some(match detector {
                Some(d) => Box::new(d.chain(slew)),
                None => Box::new(slew),
            });
        }

        // Chain LOS detectors onto previous windows (most expensive: requires ephemeris).
        for &body in &self.occulting_bodies {
            let los = make_los(body);
            detector = Some(match detector {
                Some(d) => Box::new(d.chain(los)),
                None => Box::new(los),
            });
        }

        Ok(detector.unwrap().detect(interval)?)
    }

    /// Compute visibility intervals for all pairs.
    pub fn compute(&self) -> Result<VisibilityResults, VisibilityError> {
        let interval = *self.scenario.interval();

        let mut intervals = HashMap::new();
        let mut pair_types = HashMap::new();

        if !self.scenario.ground_stations().is_empty() {
            let gs_results = self.compute_ground_space(interval)?;
            let (gs_intervals, gs_pair_types) = gs_results.into_parts();
            intervals.extend(gs_intervals);
            pair_types.extend(gs_pair_types);
        }

        if self.inter_satellite {
            let is_results = self.compute_inter_satellite(interval)?;
            let (is_intervals, is_pair_types) = is_results.into_parts();
            intervals.extend(is_intervals);
            pair_types.extend(is_pair_types);
        }

        Ok(VisibilityResults {
            intervals,
            pair_types,
        })
    }

    /// Compute ground-to-space visibility for all (ground, space) pairs.
    fn compute_ground_space(
        &self,
        interval: TimeInterval<Tai>,
    ) -> Result<VisibilityResults, VisibilityError> {
        let ground_stations = self.scenario.ground_stations();
        let spacecraft = self.scenario.spacecraft();

        let pairs: Vec<_> = ground_stations
            .iter()
            .flat_map(|gs| spacecraft.iter().map(move |sc| (gs, sc)))
            .collect();

        const PARALLEL_THRESHOLD: usize = 100;
        let use_parallel = pairs.len() > PARALLEL_THRESHOLD;

        let compute_one = |(gs, sc): &(&GroundStation, &Spacecraft)| {
            let key = (gs.id().clone(), sc.id().clone());
            let sc_traj = self.ensemble.get(sc.id()).expect(
                "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
            );
            let windows = self.compute_pair(gs, sc_traj, interval)?;
            Ok((key, windows))
        };

        let results: Result<Vec<_>, VisibilityError> = if use_parallel {
            pairs.par_iter().map(compute_one).collect()
        } else {
            pairs.iter().map(compute_one).collect()
        };

        let intervals: HashMap<_, _> = results?.into_iter().collect();
        let pair_types = intervals
            .keys()
            .map(|k| (k.clone(), PairType::GroundSpace))
            .collect();
        Ok(VisibilityResults {
            intervals,
            pair_types,
        })
    }

    /// Compute LOS visibility for all unique spacecraft pairs (i, j) where i < j.
    fn compute_inter_satellite(
        &self,
        interval: TimeInterval<Tai>,
    ) -> Result<VisibilityResults, VisibilityError> {
        let spacecraft = self.scenario.spacecraft();
        let n = spacecraft.len();
        let mut pairs: Vec<(usize, usize)> = Vec::with_capacity(n * (n - 1) / 2);
        for i in 0..n {
            for j in (i + 1)..n {
                pairs.push((i, j));
            }
        }

        let results: Result<Vec<_>, VisibilityError> = pairs
            .par_iter()
            .map(|&(i, j)| {
                let sc1 = &spacecraft[i];
                let sc2 = &spacecraft[j];
                let key = (sc1.id().clone(), sc2.id().clone());
                let traj1 = self
                    .ensemble
                    .get(sc1.id())
                    .expect("trajectory not found in ensemble");
                let traj2 = self
                    .ensemble
                    .get(sc2.id())
                    .expect("trajectory not found in ensemble");
                let windows =
                    self.compute_inter_satellite_pair(sc1, sc2, traj1, traj2, interval)?;
                Ok((key, windows))
            })
            .collect();

        let intervals: HashMap<_, _> = results?.into_iter().collect();
        let pair_types = intervals
            .keys()
            .map(|k| (k.clone(), PairType::InterSatellite))
            .collect();
        Ok(VisibilityResults {
            intervals,
            pair_types,
        })
    }

    /// Convert all ground-space intervals in a [`VisibilityResults`] to passes.
    ///
    /// Inter-satellite pairs are skipped since passes with ground-station
    /// observables are not meaningful for them.
    pub fn to_passes(
        &self,
        results: &VisibilityResults,
    ) -> HashMap<(AssetId, AssetId), Vec<DynPass>> {
        let gs_map: HashMap<&AssetId, &GroundStation> = self
            .scenario
            .ground_stations()
            .iter()
            .map(|g| (g.id(), g))
            .collect();

        results
            .ground_space_pair_ids()
            .into_iter()
            .filter_map(|(gs_id, sc_id)| {
                let gs = gs_map.get(gs_id)?;
                let sc_traj = self.ensemble.get(sc_id)?;
                let intervals = results.intervals_for(gs_id, sc_id)?;
                let passes: Vec<DynPass> = intervals
                    .iter()
                    .filter_map(|interval| {
                        // Convert Tai interval to DynTimeScale for DynPass::from_interval
                        let dyn_interval = TimeInterval::new(
                            interval.start().into_dyn(),
                            interval.end().into_dyn(),
                        );
                        // Convert typed trajectory to DynTrajectory for pass computation
                        let dyn_traj = sc_traj.clone().into_dyn();
                        DynPass::from_interval(
                            dyn_interval,
                            self.step,
                            gs.location(),
                            gs.mask(),
                            &dyn_traj,
                            gs.body_fixed_frame(),
                        )
                    })
                    .collect();
                Some(((gs_id.clone(), sc_id.clone()), passes))
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use lox_bodies::{Earth, Spheroid};
    use lox_core::coords::LonLatAlt;
    use lox_core::units::Distance;
    use lox_ephem::spk::parser::Spk;
    use lox_orbits::propagators::OrbitSource;
    use lox_test_utils::{assert_approx_eq, data_dir, data_file, read_data_file};
    use lox_time::time_scales::{DynTimeScale, Tai};
    use lox_time::utc::Utc;
    use std::iter::zip;
    use std::sync::OnceLock;

    use super::*;
    use lox_frames::Icrf;
    use lox_orbits::ground::GroundLocation;
    use lox_orbits::orbits::{DynEnsemble, DynTrajectory, Trajectory};

    /// Build a DynScenario + DynEnsemble from ground/space assets and a DynTimeScale interval.
    fn make_scenario_and_ensemble(
        ground_assets: &[GroundStation],
        space_assets: &[Spacecraft],
        interval: TimeInterval<DynTimeScale>,
    ) -> (
        Scenario<DynOrigin, DynFrame>,
        Ensemble<AssetId, Tai, DynOrigin, DynFrame>,
    ) {
        let tai_interval =
            TimeInterval::new(interval.start().to_scale(Tai), interval.end().to_scale(Tai));
        let scenario = Scenario::with_interval(tai_interval, DynOrigin::Earth, DynFrame::Icrf)
            .with_ground_stations(ground_assets)
            .with_spacecraft(space_assets);
        // Build ensemble from OrbitSource::Trajectory entries
        let mut map = HashMap::new();
        for sc in space_assets {
            if let OrbitSource::Trajectory(traj) = sc.orbit() {
                // Re-tag DynTrajectory as Ensemble<Tai, DynOrigin, DynFrame>
                let (epoch, origin, frame, data) = traj.clone().into_parts();
                let typed = Trajectory::from_parts(epoch.with_scale(Tai), origin, frame, data);
                map.insert(sc.id().clone(), typed);
            }
        }
        let ensemble = Ensemble::new(map);
        (scenario, ensemble)
    }

    #[test]
    fn test_line_of_sight() {
        let r1 = DVec3::new(0.0, -4464.696, -5102.509);
        let r2 = DVec3::new(0.0, 5740.323, 3189.068);
        let r_sun = DVec3::new(122233179.0, -76150708.0, 33016374.0);
        let r = Earth.equatorial_radius().to_kilometers();

        let los = line_of_sight(r, r1, r2);
        let los_sun = line_of_sight(r, r1, r_sun);

        assert!(los < 0.0);
        assert!(los_sun >= 0.0);
    }

    #[test]
    fn test_line_of_sight_identical() {
        let r1 = DVec3::new(0.0, -4464.696, -5102.509);
        let r2 = DVec3::new(0.0, -4464.696, -5102.509);
        let r = Earth.equatorial_radius().to_kilometers();

        let los = line_of_sight(r, r1, r2);

        assert!(los >= 0.0);
    }

    #[test]
    fn test_line_of_sight_trait() {
        let r1 = DVec3::new(0.0, -4464696.0, -5102509.0);
        let r2 = DVec3::new(0.0, 5740323.0, 3189068.0);
        let r_sun = DVec3::new(122233179e3, -76150708e3, 33016374e3);

        let los = Earth.line_of_sight(r1, r2).unwrap();
        let los_sun = Earth.line_of_sight(r1, r_sun).unwrap();

        assert!(los < 0.0);
        assert!(los_sun >= 0.0);
    }

    #[test]
    fn test_elevation() {
        let sc = spacecraft_trajectory_dyn();
        let gs_traj = ground_station_trajectory();
        let gs = location_dyn();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let expected: Vec<f64> = read_data_file("elevation.csv")
            .lines()
            .map(|line| line.parse::<f64>().unwrap().to_radians())
            .collect();
        // Build a typed trajectory for the ElevationDetectFn
        let (epoch, o, f, data) = sc.clone().into_parts();
        let typed_sc = Trajectory::from_parts(epoch.with_scale(Tai), o, f, data);
        let elev_fn = ElevationDetectFn {
            gs: &gs,
            mask: &mask,
            sc: &typed_sc,
            body_fixed_frame: DynFrame::Iau(DynOrigin::Earth),
        };
        // Use the ground station trajectory times converted to Tai
        let actual: Vec<f64> = gs_traj
            .times()
            .iter()
            .map(|t| {
                let tai_time = t.to_scale(Tai);
                elev_fn.eval(tai_time).unwrap()
            })
            .collect();
        assert_approx_eq!(actual, expected, atol <= 1e-1);
    }

    #[test]
    fn test_elevation_mask() {
        let azimuth = vec![-PI, 0.0, PI];
        let elevation = vec![-2.0, 0.0, 2.0];
        let mask = ElevationMask::new(azimuth, elevation).unwrap();
        assert_eq!(mask.min_elevation(0.0), 0.0);
    }

    #[test]
    fn test_elevation_mask_invalid_mask() {
        let azimuth = vec![-PI, 0.0, PI / 2.0];
        let elevation = vec![-2.0, 0.0, 2.0];
        let mask = ElevationMask::new(azimuth, elevation);
        assert_eq!(
            mask,
            Err(ElevationMaskError::InvalidAzimuthRange(-PI, PI / 2.0))
        )
    }

    #[test]
    fn test_visibility() {
        let gs_loc = location_dyn();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let sc_traj = spacecraft_trajectory_dyn();
        let gs = GroundStation::new("cebreros", gs_loc, mask);
        let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(sc_traj.clone()));
        let spk = ephemeris();
        let ground_assets = [gs.clone()];
        let space_assets = [sc.clone()];
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let (scenario, ensemble) =
            make_scenario_and_ensemble(&ground_assets, &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk);
        let results = analysis.compute().expect("visibility");
        let intervals = results
            .intervals_for(gs.id(), sc.id())
            .expect("pair not found");
        let expected = contacts_tai();
        assert_eq!(intervals.len(), expected.len());
        assert_approx_eq!(expected, intervals.to_vec(), rtol <= 1e-4);
    }

    #[test]
    fn test_visibility_combined() {
        let gs_loc = location_dyn();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let sc_traj = spacecraft_trajectory_dyn();
        let gs = GroundStation::new("cebreros", gs_loc, mask);
        let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(sc_traj.clone()));
        let spk = ephemeris();
        let ground_assets = [gs.clone()];
        let space_assets = [sc.clone()];
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let (scenario, ensemble) =
            make_scenario_and_ensemble(&ground_assets, &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk)
            .with_occulting_bodies(vec![DynOrigin::Moon]);
        let results = analysis.compute().unwrap();
        let passes = analysis.to_passes(&results);
        let key = (gs.id().clone(), sc.id().clone());
        let pair_passes = &passes[&key];
        let expected = contacts_combined();
        assert_eq!(pair_passes.len(), expected.len());
        for (actual, expected) in zip(pair_passes, expected) {
            assert_approx_eq!(actual.interval().start(), expected.start(), rtol <= 1e-4);
            assert_approx_eq!(actual.interval().end(), expected.end(), rtol <= 1e-4);
        }
    }

    #[test]
    fn test_pass_observables_above_mask() {
        let gs_loc = location_dyn();
        let mask = ElevationMask::with_fixed_elevation(10.0_f64.to_radians());
        let sc_traj = spacecraft_trajectory_dyn();
        let gs = GroundStation::new("cebreros", gs_loc, mask);
        let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(sc_traj.clone()));
        let spk = ephemeris();
        let ground_assets = [gs.clone()];
        let space_assets = [sc.clone()];
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let (scenario, ensemble) =
            make_scenario_and_ensemble(&ground_assets, &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk);
        let results = analysis.compute().unwrap();
        let passes = analysis.to_passes(&results);
        let key = (gs.id().clone(), sc.id().clone());
        let pair_passes = &passes[&key];
        let mask = gs.mask();

        for pass in pair_passes {
            for obs in pass.observables() {
                let min_elevation = mask.min_elevation(obs.azimuth());
                assert!(
                    obs.elevation() >= min_elevation,
                    "Observable elevation {:.2}° is below mask minimum {:.2}° at azimuth {:.2}°",
                    obs.elevation().to_degrees(),
                    min_elevation.to_degrees(),
                    obs.azimuth().to_degrees()
                );
            }
        }
    }

    fn ground_station_trajectory() -> Trajectory<Tai, Earth, Icrf> {
        Trajectory::from_csv(&read_data_file("trajectory_cebr.csv"), Earth, Icrf).unwrap()
    }

    fn spacecraft_trajectory_dyn() -> DynTrajectory {
        DynTrajectory::from_csv_dyn(
            &read_data_file("trajectory_lunar.csv"),
            DynOrigin::Earth,
            DynFrame::Icrf,
        )
        .unwrap()
    }

    fn location_dyn() -> GroundLocation<DynOrigin> {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        GroundLocation::try_new(coords, DynOrigin::Earth).unwrap()
    }

    fn contacts_tai() -> Vec<TimeInterval<Tai>> {
        let mut intervals = vec![];
        let mut reader = csv::Reader::from_path(data_file("contacts.csv")).unwrap();
        for result in reader.records() {
            let record = result.unwrap();
            let start = record[0].parse::<Utc>().unwrap().to_time();
            let end = record[1].parse::<Utc>().unwrap().to_time();
            intervals.push(TimeInterval::new(start, end));
        }
        intervals
    }

    fn contacts_combined() -> Vec<TimeInterval<DynTimeScale>> {
        let mut intervals = vec![];
        let mut reader = csv::Reader::from_path(data_file("contacts_combined.csv")).unwrap();
        for result in reader.records() {
            let record = result.unwrap();
            let start = record[0].parse::<Utc>().unwrap().to_dyn_time();
            let end = record[1].parse::<Utc>().unwrap().to_dyn_time();
            intervals.push(TimeInterval::new(start, end));
        }
        intervals
    }

    #[test]
    fn test_inter_satellite_visibility() {
        let sc_traj = spacecraft_trajectory_dyn();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let sc1 = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj.clone()));
        let sc2 = Spacecraft::new("sc2", OrbitSource::Trajectory(sc_traj));
        let spk = ephemeris();
        let space_assets = [sc1.clone(), sc2.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk)
            .with_inter_satellite()
            .with_occulting_bodies(vec![DynOrigin::Earth]);
        let results = analysis.compute().unwrap();
        let intervals = results
            .intervals_for(sc1.id(), sc2.id())
            .expect("pair not found");
        // Colocated spacecraft are always visible to each other.
        assert_eq!(intervals.len(), 1);
        let tai_interval =
            TimeInterval::new(interval.start().to_scale(Tai), interval.end().to_scale(Tai));
        assert_approx_eq!(intervals[0].start(), tai_interval.start(), rtol <= 1e-10);
        assert_approx_eq!(intervals[0].end(), tai_interval.end(), rtol <= 1e-10);
    }

    #[test]
    fn test_inter_satellite_visibility_with_range_filter() {
        let sc_traj = spacecraft_trajectory_dyn();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let sc1 = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj.clone()));
        let sc2 = Spacecraft::new("sc2", OrbitSource::Trajectory(sc_traj));
        let spk = ephemeris();
        let space_assets = [sc1.clone(), sc2.clone()];

        // Colocated spacecraft have range = 0. A max_range filter with a large
        // threshold should still return the full interval.
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk)
            .with_inter_satellite()
            .with_max_range(Distance::kilometers(1000.0));
        let results = analysis.compute().unwrap();
        let intervals = results
            .intervals_for(sc1.id(), sc2.id())
            .expect("pair not found");
        let tai_interval =
            TimeInterval::new(interval.start().to_scale(Tai), interval.end().to_scale(Tai));
        assert_eq!(intervals.len(), 1);
        assert_approx_eq!(intervals[0].start(), tai_interval.start(), rtol <= 1e-10);
        assert_approx_eq!(intervals[0].end(), tai_interval.end(), rtol <= 1e-10);

        // A min_range filter with a positive threshold should exclude colocated
        // spacecraft entirely (range = 0 < threshold at all times).
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk)
            .with_inter_satellite()
            .with_min_range(Distance::kilometers(100.0));
        let results = analysis.compute().unwrap();
        let intervals = results
            .intervals_for(sc1.id(), sc2.id())
            .expect("pair not found");
        assert!(
            intervals.is_empty(),
            "expected no intervals for colocated spacecraft with min_range, got {}",
            intervals.len()
        );
    }

    #[test]
    fn test_slew_rate_detect_fn() {
        // Two colocated trajectories have zero angular rate → always within limit.
        let sc_traj = spacecraft_trajectory_dyn();
        let (epoch, origin, frame, data) = sc_traj.into_parts();
        let typed = Trajectory::from_parts(epoch.with_scale(Tai), origin, frame, data);
        let threshold = AngularRate::degrees_per_second(1.0);
        let detect = InterSatelliteSlewRateDetectFn {
            sc1: &typed,
            sc2: &typed,
            threshold,
        };
        let time = typed.start_time();
        let val = detect.eval(time).unwrap();
        // ω = 0 for colocated → threshold - 0 = threshold
        assert_approx_eq!(val, threshold.to_radians_per_second(), rtol <= 1e-10);
    }

    #[test]
    fn test_inter_satellite_visibility_with_slew_rate() {
        let sc_traj = spacecraft_trajectory_dyn();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());

        // Colocated spacecraft have ω = 0. A generous slew rate limit should
        // keep the full interval.
        let sc1 = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj.clone()))
            .with_max_slew_rate(AngularRate::degrees_per_second(10.0));
        let sc2 = Spacecraft::new("sc2", OrbitSource::Trajectory(sc_traj))
            .with_max_slew_rate(AngularRate::degrees_per_second(5.0));
        let spk = ephemeris();
        let space_assets = [sc1.clone(), sc2.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk).with_inter_satellite();
        let results = analysis.compute().unwrap();
        let intervals = results
            .intervals_for(sc1.id(), sc2.id())
            .expect("pair not found");
        let tai_interval =
            TimeInterval::new(interval.start().to_scale(Tai), interval.end().to_scale(Tai));
        // ω = 0 everywhere, so full interval should be returned.
        assert_eq!(intervals.len(), 1);
        assert_approx_eq!(intervals[0].start(), tai_interval.start(), rtol <= 1e-10);
        assert_approx_eq!(intervals[0].end(), tai_interval.end(), rtol <= 1e-10);
    }

    #[test]
    fn test_space_asset_max_slew_rate() {
        let sc_traj = spacecraft_trajectory_dyn();
        let sc = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj));
        assert!(sc.max_slew_rate().is_none());

        let rate = AngularRate::degrees_per_second(2.5);
        let sc = sc.with_max_slew_rate(rate);
        assert_approx_eq!(
            sc.max_slew_rate().unwrap().to_degrees_per_second(),
            2.5,
            rtol <= 1e-10
        );
    }

    // Two OneWeb satellites in different orbital planes (~192° RAAN separation).
    // ONEWEB-0012: RAAN 343.68°, ONEWEB-0017: RAAN 151.03°
    // Their crossing orbits produce high angular rates during close approaches.

    fn oneweb_trajectories() -> (DynTrajectory, DynTrajectory) {
        use lox_orbits::propagators::Propagator;
        use lox_orbits::propagators::sgp4::{Elements, Sgp4};
        use lox_time::intervals::Interval;

        let tle1 = Elements::from_tle(
            Some("ONEWEB-0012".to_string()),
            b"1 44057U 19010A   24322.58825131  .00000088  00000+0  19693-3 0  9993",
            b"2 44057  87.9092 343.6767 0002420  76.7970 283.3431 13.16592150275693",
        )
        .unwrap();
        let tle2 = Elements::from_tle(
            Some("ONEWEB-0017".to_string()),
            b"1 45132U 20008B   24322.88240834 -.00000016  00000+0 -81930-4 0  9998",
            b"2 45132  87.8896 151.0343 0001369  78.1189 282.0092 13.10376984232476",
        )
        .unwrap();

        let sgp4_1 = Sgp4::new(tle1).unwrap();
        let sgp4_2 = Sgp4::new(tle2).unwrap();

        // Use the later epoch as start so both TLEs are valid.
        let t0 = sgp4_1.time().max(sgp4_2.time());
        let t1 = t0 + TimeDelta::from_hours(2);
        let interval = Interval::new(t0, t1);

        let traj1 = sgp4_1
            .with_step(TimeDelta::from_seconds(10))
            .propagate(interval)
            .unwrap()
            .into_dyn();
        let traj2 = sgp4_2
            .with_step(TimeDelta::from_seconds(10))
            .propagate(interval)
            .unwrap()
            .into_dyn();

        (traj1, traj2)
    }

    #[test]
    fn test_slew_rate_trims_windows_for_crossing_orbits() {
        let (traj1, traj2) = oneweb_trajectories();
        let interval = TimeInterval::new(traj1.start_time(), traj1.end_time());

        let spk = ephemeris();

        // Without slew rate constraint: should have visibility (no other
        // constraints → full interval returned).
        let sc1_no_limit = Spacecraft::new("ow12", OrbitSource::Trajectory(traj1.clone()));
        let sc2_no_limit = Spacecraft::new("ow17", OrbitSource::Trajectory(traj2.clone()));
        let space_assets = [sc1_no_limit.clone(), sc2_no_limit.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk).with_inter_satellite();
        let results_no_limit = analysis.compute().unwrap();
        let intervals_no_limit = results_no_limit
            .intervals_for(sc1_no_limit.id(), sc2_no_limit.id())
            .expect("pair not found");

        // With a tight slew rate constraint (0.01 deg/s): should trim windows
        // compared to the unconstrained case.
        let sc1_limited = Spacecraft::new("ow12", OrbitSource::Trajectory(traj1))
            .with_max_slew_rate(AngularRate::degrees_per_second(0.01));
        let sc2_limited = Spacecraft::new("ow17", OrbitSource::Trajectory(traj2))
            .with_max_slew_rate(AngularRate::degrees_per_second(0.01));
        let space_assets = [sc1_limited.clone(), sc2_limited.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk).with_inter_satellite();
        let results_limited = analysis.compute().unwrap();
        let intervals_limited = results_limited
            .intervals_for(sc1_limited.id(), sc2_limited.id())
            .expect("pair not found");

        // The constrained intervals should be strictly shorter in total duration.
        let total_no_limit: f64 = intervals_no_limit
            .iter()
            .map(|i| (i.end() - i.start()).to_seconds().to_f64())
            .sum();
        let total_limited: f64 = intervals_limited
            .iter()
            .map(|i| (i.end() - i.start()).to_seconds().to_f64())
            .sum();
        assert!(
            total_limited < total_no_limit,
            "slew rate constraint should reduce total visibility (got {total_limited:.0}s vs {total_no_limit:.0}s)"
        );
    }

    #[test]
    fn test_inter_satellite_asymmetric_slew_rate_sc1_only() {
        let sc_traj = spacecraft_trajectory_dyn();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());

        // Only sc1 has a slew rate limit — exercises the (Some(a), None) branch.
        let sc1 = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj.clone()))
            .with_max_slew_rate(AngularRate::degrees_per_second(10.0));
        let sc2 = Spacecraft::new("sc2", OrbitSource::Trajectory(sc_traj));
        let spk = ephemeris();
        let space_assets = [sc1.clone(), sc2.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk).with_inter_satellite();
        let results = analysis.compute().unwrap();
        let intervals = results
            .intervals_for(sc1.id(), sc2.id())
            .expect("pair not found");
        // Colocated → ω = 0, full interval returned.
        assert_eq!(intervals.len(), 1);
    }

    #[test]
    fn test_inter_satellite_asymmetric_slew_rate_sc2_only() {
        let sc_traj = spacecraft_trajectory_dyn();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());

        // Only sc2 has a slew rate limit — exercises the (None, Some(b)) branch.
        let sc1 = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj.clone()));
        let sc2 = Spacecraft::new("sc2", OrbitSource::Trajectory(sc_traj))
            .with_max_slew_rate(AngularRate::degrees_per_second(10.0));
        let spk = ephemeris();
        let space_assets = [sc1.clone(), sc2.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk).with_inter_satellite();
        let results = analysis.compute().unwrap();
        let intervals = results
            .intervals_for(sc1.id(), sc2.id())
            .expect("pair not found");
        assert_eq!(intervals.len(), 1);
    }

    #[test]
    fn test_inter_satellite_both_min_and_max_range() {
        let (traj1, traj2) = oneweb_trajectories();
        let interval = TimeInterval::new(traj1.start_time(), traj1.end_time());
        let sc1 = Spacecraft::new("ow12", OrbitSource::Trajectory(traj1));
        let sc2 = Spacecraft::new("ow17", OrbitSource::Trajectory(traj2));
        let spk = ephemeris();
        let space_assets = [sc1.clone(), sc2.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        // Set both min and max range to exercise the intersection branch (line 835).
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk)
            .with_inter_satellite()
            .with_min_range(Distance::kilometers(100.0))
            .with_max_range(Distance::kilometers(5000.0));
        let results = analysis.compute().unwrap();
        let intervals = results
            .intervals_for(sc1.id(), sc2.id())
            .expect("pair not found");
        // Should have some visibility windows within the range band.
        assert!(!intervals.is_empty());
    }

    fn ephemeris() -> &'static Spk {
        static EPHEMERIS: OnceLock<Spk> = OnceLock::new();
        EPHEMERIS.get_or_init(|| Spk::from_file(data_file("spice/de440s.bsp")).unwrap())
    }
}
