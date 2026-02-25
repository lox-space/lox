// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use glam::DVec3;
use lox_bodies::{DynOrigin, TryMeanRadius, TrySpheroid, UndefinedOriginPropertyError};
use lox_ephem::Ephemeris;
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::{DynRotationError, TryRotation};
use lox_math::series::{InterpolationType, Series, SeriesError};
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::offsets::DefaultOffsetProvider;
use lox_time::time_scales::DynTimeScale;
use lox_time::time_scales::{Tdb, TimeScale};
use lox_time::{DynTime, Time};
use rayon::prelude::*;
use std::f64::consts::PI;
use thiserror::Error;

use crate::assets::{AssetId, GroundAsset, SpaceAsset};
use crate::events::{
    DetectError, DetectFn, EventsToIntervals, IntervalDetector, RootFindingDetector,
};
use crate::ground::{DynGroundLocation, Observables};
use crate::orbits::DynTrajectory;
use lox_frames::DynFrame;

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
        sc: &DynTrajectory,
    ) -> Option<DynPass> {
        let mut pass_times = Vec::new();
        let mut pass_observables = Vec::new();
        let body_fixed = DynFrame::Iau(gs.origin());

        for current_time in interval.step_by(time_resolution) {
            let state = sc.interpolate_at(current_time);
            let state_bf = state
                .try_to_frame(body_fixed, &DefaultRotationProvider)
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
    #[error(transparent)]
    Rotation(#[from] DynRotationError),
    #[error(transparent)]
    UndefinedProperty(#[from] UndefinedOriginPropertyError),
    #[error("ephemeris error: {0}")]
    Ephemeris(Box<dyn std::error::Error + Send + Sync>),
}

// ---------------------------------------------------------------------------
// DetectFn implementations
// ---------------------------------------------------------------------------

/// Elevation above mask for a ground station / spacecraft pair.
struct ElevationDetectFn<'a> {
    gs: &'a DynGroundLocation,
    mask: &'a ElevationMask,
    sc: &'a DynTrajectory,
}

impl DetectFn<DynTimeScale> for ElevationDetectFn<'_> {
    // Infallible: ICRF→IAU rotations always succeed for known origins.
    // Using Infallible lets the compiler eliminate the error path, which
    // is critical for performance on this hot path (~1400 evals/pair).
    type Error = std::convert::Infallible;

    fn eval(&self, time: DynTime) -> Result<f64, Self::Error> {
        let body_fixed = DynFrame::Iau(self.gs.origin());
        let sc = self.sc.interpolate_at(time);
        let sc = sc
            .try_to_frame(body_fixed, &DefaultRotationProvider)
            .unwrap();
        let obs = self.gs.observables_dyn(sc);
        Ok(obs.elevation() - self.mask.min_elevation(obs.azimuth()))
    }
}

/// Line-of-sight between a ground station and spacecraft, relative to an
/// occulting body.
struct LineOfSightDetectFn<'a, E> {
    gs: &'a DynGroundLocation,
    sc: &'a DynTrajectory,
    body: DynOrigin,
    ephemeris: &'a E,
}

impl<E: Ephemeris> DetectFn<DynTimeScale> for LineOfSightDetectFn<'_, E>
where
    E::Error: 'static,
{
    type Error = EvalError;

    fn eval(&self, time: DynTime) -> Result<f64, Self::Error> {
        // DefaultOffsetProvider returns Infallible for DynTimeScale → Tdb.
        let tdb = time.try_to_scale(Tdb, &DefaultOffsetProvider).unwrap();
        let r_body = self
            .ephemeris
            .position(tdb, self.sc.origin(), self.body)
            .map_err(|e| EvalError::Ephemeris(Box::new(e)))?;
        let r_sc = self.sc.interpolate_at(time).position() - r_body;
        // Compute ground station ICRF position by rotating body-fixed position
        let body_fixed_frame = DynFrame::Iau(self.gs.origin());
        let rot = DefaultRotationProvider.try_rotation(body_fixed_frame, DynFrame::Icrf, time)?;
        let (r_gs_icrf, _) = rot.rotate_state(self.gs.body_fixed_position(), DVec3::ZERO);
        let r_gs = r_gs_icrf - r_body;
        Ok(self.body.line_of_sight(r_gs, r_sc)?)
    }
}

/// Line-of-sight between two spacecraft, relative to an occulting body.
struct InterSatelliteLosDetectFn<'a, E> {
    sc1: &'a DynTrajectory,
    sc2: &'a DynTrajectory,
    body: DynOrigin,
    ephemeris: &'a E,
}

impl<E: Ephemeris> DetectFn<DynTimeScale> for InterSatelliteLosDetectFn<'_, E>
where
    E::Error: 'static,
{
    type Error = EvalError;

    fn eval(&self, time: DynTime) -> Result<f64, Self::Error> {
        // DefaultOffsetProvider returns Infallible for DynTimeScale → Tdb.
        let tdb = time.try_to_scale(Tdb, &DefaultOffsetProvider).unwrap();
        let r_body = self
            .ephemeris
            .position(tdb, self.sc1.origin(), self.body)
            .map_err(|e| EvalError::Ephemeris(Box::new(e)))?;
        let r_sc1 = self.sc1.interpolate_at(time).position() - r_body;
        let r_sc2 = self.sc2.interpolate_at(time).position() - r_body;
        Ok(self.body.line_of_sight(r_sc1, r_sc2)?)
    }
}

// ---------------------------------------------------------------------------
// VisibilityResults
// ---------------------------------------------------------------------------

/// Stores raw visibility intervals per asset pair.
///
/// This is the primary result type for visibility analysis. Intervals are
/// cheap to compute; conversion to [`DynPass`] (with observables) happens
/// separately and on demand.
pub struct VisibilityResults {
    intervals: HashMap<(AssetId, AssetId), Vec<TimeInterval<DynTimeScale>>>,
}

impl VisibilityResults {
    /// Return all intervals for a specific (ground, space) pair.
    pub fn intervals_for(
        &self,
        ground_id: &AssetId,
        space_id: &AssetId,
    ) -> Option<&[TimeInterval<DynTimeScale>]> {
        let key = (ground_id.clone(), space_id.clone());
        self.intervals.get(&key).map(|v| v.as_slice())
    }

    /// Return all intervals keyed by (ground_id, space_id).
    pub fn all_intervals(&self) -> &HashMap<(AssetId, AssetId), Vec<TimeInterval<DynTimeScale>>> {
        &self.intervals
    }

    /// Iterate over all (ground_id, space_id) pair keys.
    pub fn pair_ids(&self) -> impl Iterator<Item = &(AssetId, AssetId)> {
        self.intervals.keys()
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

    /// Convert intervals for a specific pair to visibility passes.
    ///
    /// Each interval is populated with observables by sampling the spacecraft
    /// trajectory. Returns an empty vec if the pair is not found.
    pub fn to_passes(
        &self,
        ground_id: &AssetId,
        space_id: &AssetId,
        gs: &DynGroundLocation,
        mask: &ElevationMask,
        sc: &DynTrajectory,
        time_resolution: TimeDelta,
    ) -> Vec<DynPass> {
        let key = (ground_id.clone(), space_id.clone());
        self.intervals
            .get(&key)
            .map(|intervals| {
                intervals
                    .iter()
                    .filter_map(|interval| {
                        DynPass::from_interval(*interval, time_resolution, gs, mask, sc)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// VisibilityAnalysis
// ---------------------------------------------------------------------------

/// Computes ground-station-to-spacecraft visibility.
///
/// Uses the new `DetectFn` / `RootFindingDetector` / `EventsToIntervals`
/// pipeline for event detection.
pub struct VisibilityAnalysis<'a, E> {
    ground_assets: &'a [GroundAsset],
    space_assets: &'a [SpaceAsset],
    ephemeris: &'a E,
    occulting_bodies: Vec<DynOrigin>,
    step: TimeDelta,
    min_pass_duration: Option<TimeDelta>,
}

impl<'a, E> VisibilityAnalysis<'a, E>
where
    E: Ephemeris + Send + Sync,
    E::Error: 'static,
{
    pub fn new(
        ground_assets: &'a [GroundAsset],
        space_assets: &'a [SpaceAsset],
        ephemeris: &'a E,
    ) -> Self {
        Self {
            ground_assets,
            space_assets,
            ephemeris,
            occulting_bodies: Vec::new(),
            step: TimeDelta::from_seconds(60),
            min_pass_duration: None,
        }
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
    pub fn compute_pair(
        &self,
        gs: &GroundAsset,
        sc: &SpaceAsset,
        interval: TimeInterval<DynTimeScale>,
    ) -> Result<Vec<TimeInterval<DynTimeScale>>, VisibilityError> {
        let make_elev = || {
            let det = RootFindingDetector::new(
                ElevationDetectFn {
                    gs: gs.location(),
                    mask: gs.mask(),
                    sc: sc.trajectory(),
                },
                self.step,
            );
            EventsToIntervals::new(self.apply_coarse_step(det))
        };

        if self.occulting_bodies.is_empty() {
            return Ok(make_elev().detect(interval)?);
        }

        // Compute elevation windows once and reuse across occulting bodies.
        let elev_windows = make_elev().detect(interval)?;

        if elev_windows.is_empty() {
            return Ok(vec![]);
        }

        // For each occulting body, run LOS detection within the cached
        // elevation windows instead of recomputing elevation each time.
        let body_windows: Vec<Vec<TimeInterval<DynTimeScale>>> = self
            .occulting_bodies
            .par_iter()
            .map(|&body| {
                let los = EventsToIntervals::new(self.apply_coarse_step(RootFindingDetector::new(
                    LineOfSightDetectFn {
                        gs: gs.location(),
                        sc: sc.trajectory(),
                        body,
                        ephemeris: self.ephemeris,
                    },
                    self.step,
                )));
                let mut windows = Vec::new();
                for sub in &elev_windows {
                    windows.extend(los.detect(*sub)?);
                }
                Ok(windows)
            })
            .collect::<Result<Vec<_>, DetectError>>()?;

        let mut windows = body_windows[0].clone();
        for bw in &body_windows[1..] {
            windows = lox_time::intervals::intersect_intervals(&windows, bw);
        }

        Ok(windows)
    }

    /// Compute visibility intervals for all (ground, space) pairs.
    pub fn compute(
        &self,
        interval: TimeInterval<DynTimeScale>,
    ) -> Result<VisibilityResults, VisibilityError> {
        let pairs: Vec<_> = self
            .ground_assets
            .iter()
            .flat_map(|gs| self.space_assets.iter().map(move |sc| (gs, sc)))
            .collect();

        // Parallelise across pairs when the rayon overhead is worthwhile.
        // Each pair already parallelises internally (LOS per occulting body),
        // so outer-level parallelism only helps with many pairs.
        const PARALLEL_THRESHOLD: usize = 100;
        let use_parallel = pairs.len() > PARALLEL_THRESHOLD;

        let results: Result<Vec<_>, VisibilityError> = if use_parallel {
            pairs
                .par_iter()
                .map(|(gs, sc)| {
                    let key = (gs.id().clone(), sc.id().clone());
                    let windows = self.compute_pair(gs, sc, interval)?;
                    Ok((key, windows))
                })
                .collect()
        } else {
            pairs
                .iter()
                .map(|(gs, sc)| {
                    let key = (gs.id().clone(), sc.id().clone());
                    let windows = self.compute_pair(gs, sc, interval)?;
                    Ok((key, windows))
                })
                .collect()
        };

        Ok(VisibilityResults {
            intervals: results?.into_iter().collect(),
        })
    }

    /// Convert all intervals in a [`VisibilityResults`] to passes.
    pub fn to_passes(
        &self,
        results: &VisibilityResults,
    ) -> HashMap<(AssetId, AssetId), Vec<DynPass>> {
        let gs_map: HashMap<&AssetId, &GroundAsset> =
            self.ground_assets.iter().map(|g| (g.id(), g)).collect();
        let sc_map: HashMap<&AssetId, &SpaceAsset> =
            self.space_assets.iter().map(|s| (s.id(), s)).collect();

        results
            .all_intervals()
            .iter()
            .map(|((gs_id, sc_id), intervals)| {
                let gs = gs_map[gs_id];
                let sc = sc_map[sc_id];
                let passes: Vec<DynPass> = intervals
                    .iter()
                    .filter_map(|interval| {
                        DynPass::from_interval(
                            *interval,
                            self.step,
                            gs.location(),
                            gs.mask(),
                            sc.trajectory(),
                        )
                    })
                    .collect();
                ((gs_id.clone(), sc_id.clone()), passes)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// InterSatelliteVisibility
// ---------------------------------------------------------------------------

/// Computes line-of-sight visibility between spacecraft pairs.
pub struct InterSatelliteVisibility<'a, E> {
    space_assets: &'a [SpaceAsset],
    ephemeris: &'a E,
    occulting_bodies: Vec<DynOrigin>,
    step: TimeDelta,
    min_pass_duration: Option<TimeDelta>,
}

impl<'a, E> InterSatelliteVisibility<'a, E>
where
    E: Ephemeris + Send + Sync,
    E::Error: 'static,
{
    pub fn new(space_assets: &'a [SpaceAsset], ephemeris: &'a E) -> Self {
        Self {
            space_assets,
            ephemeris,
            occulting_bodies: Vec::new(),
            step: TimeDelta::from_seconds(60),
            min_pass_duration: None,
        }
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

    /// Compute LOS intervals for all unique ordered pairs `(i, j)` where `i < j`.
    pub fn compute(
        &self,
        interval: TimeInterval<DynTimeScale>,
    ) -> Result<VisibilityResults, VisibilityError> {
        let n = self.space_assets.len();
        let mut pairs: Vec<(usize, usize)> = Vec::with_capacity(n * (n - 1) / 2);
        for i in 0..n {
            for j in (i + 1)..n {
                pairs.push((i, j));
            }
        }

        let results: Result<Vec<_>, VisibilityError> = pairs
            .par_iter()
            .map(|&(i, j)| {
                let sc1 = &self.space_assets[i];
                let sc2 = &self.space_assets[j];
                let key = (sc1.id().clone(), sc2.id().clone());

                let mut windows = vec![interval];

                for &body in &self.occulting_bodies {
                    let los_fn = InterSatelliteLosDetectFn {
                        sc1: sc1.trajectory(),
                        sc2: sc2.trajectory(),
                        body,
                        ephemeris: self.ephemeris,
                    };
                    let los_detector =
                        self.apply_coarse_step(RootFindingDetector::new(los_fn, self.step));
                    let los_intervals = EventsToIntervals::new(los_detector);
                    let body_windows = los_intervals.detect(interval)?;
                    windows = lox_time::intervals::intersect_intervals(&windows, &body_windows);
                }

                Ok((key, windows))
            })
            .collect();

        Ok(VisibilityResults {
            intervals: results?.into_iter().collect(),
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use lox_bodies::{Earth, Spheroid};
    use lox_core::coords::LonLatAlt;
    use lox_ephem::spk::parser::Spk;
    use lox_test_utils::{assert_approx_eq, data_dir, data_file, read_data_file};
    use lox_time::time_scales::Tai;
    use lox_time::utc::Utc;
    use std::iter::zip;
    use std::sync::OnceLock;

    use super::*;
    use crate::ground::GroundLocation;
    use crate::orbits::Trajectory;
    use lox_frames::Icrf;

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
        let elev_fn = ElevationDetectFn {
            gs: &gs,
            mask: &mask,
            sc: &sc,
        };
        // Use the ground station trajectory times converted to dyn
        let actual: Vec<f64> = gs_traj
            .times()
            .iter()
            .map(|t| {
                let dyn_time = t.to_scale(DynTimeScale::Tai);
                elev_fn.eval(dyn_time).unwrap()
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
        let gs = GroundAsset::new("cebreros", gs_loc, mask);
        let sc = SpaceAsset::new("lunar", sc_traj.clone());
        let spk = ephemeris();
        let ground_assets = [gs.clone()];
        let space_assets = [sc.clone()];
        let analysis = VisibilityAnalysis::new(&ground_assets, &space_assets, spk);
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let results = analysis.compute(interval).expect("visibility");
        let intervals = results
            .intervals_for(gs.id(), sc.id())
            .expect("pair not found");
        let expected = contacts_dyn();
        assert_eq!(intervals.len(), expected.len());
        assert_approx_eq!(expected, intervals.to_vec(), rtol <= 1e-4);
    }

    #[test]
    fn test_visibility_combined() {
        let gs_loc = location_dyn();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let sc_traj = spacecraft_trajectory_dyn();
        let gs = GroundAsset::new("cebreros", gs_loc, mask);
        let sc = SpaceAsset::new("lunar", sc_traj.clone());
        let spk = ephemeris();
        let ground_assets = [gs.clone()];
        let space_assets = [sc.clone()];
        let analysis = VisibilityAnalysis::new(&ground_assets, &space_assets, spk)
            .with_occulting_bodies(vec![DynOrigin::Moon]);
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let results = analysis.compute(interval).unwrap();
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
        let gs = GroundAsset::new("cebreros", gs_loc, mask);
        let sc = SpaceAsset::new("lunar", sc_traj.clone());
        let spk = ephemeris();
        let ground_assets = [gs.clone()];
        let space_assets = [sc.clone()];
        let analysis = VisibilityAnalysis::new(&ground_assets, &space_assets, spk);
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let results = analysis.compute(interval).unwrap();
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

    fn contacts_dyn() -> Vec<TimeInterval<DynTimeScale>> {
        let mut intervals = vec![];
        let mut reader = csv::Reader::from_path(data_file("contacts.csv")).unwrap();
        for result in reader.records() {
            let record = result.unwrap();
            let start = record[0].parse::<Utc>().unwrap().to_dyn_time();
            let end = record[1].parse::<Utc>().unwrap().to_dyn_time();
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
        let sc1 = SpaceAsset::new("sc1", sc_traj.clone());
        let sc2 = SpaceAsset::new("sc2", sc_traj);
        let spk = ephemeris();
        let space_assets = [sc1.clone(), sc2.clone()];
        let isv = InterSatelliteVisibility::new(&space_assets, spk)
            .with_occulting_bodies(vec![DynOrigin::Earth]);
        let results = isv.compute(interval).unwrap();
        let intervals = results
            .intervals_for(sc1.id(), sc2.id())
            .expect("pair not found");
        // Colocated spacecraft are always visible to each other.
        assert_eq!(intervals.len(), 1);
        assert_approx_eq!(intervals[0].start(), interval.start(), rtol <= 1e-10);
        assert_approx_eq!(intervals[0].end(), interval.end(), rtol <= 1e-10);
    }

    fn ephemeris() -> &'static Spk {
        static EPHEMERIS: OnceLock<Spk> = OnceLock::new();
        EPHEMERIS.get_or_init(|| Spk::from_file(data_dir().join("spice/de440s.bsp")).unwrap())
    }
}
