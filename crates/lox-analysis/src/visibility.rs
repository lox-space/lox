// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::ops::Deref;

use lox_bodies::{
    CoordinateOrigin, Origin, TryMeanRadius, TrySpheroid, UndefinedOriginPropertyError,
};
use lox_core::glam::DVec3;
use lox_core::math::series::{InterpolationType, Series, SeriesError};
use lox_ephem::Ephemeris;
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::{RotationError, TryRotation};
use lox_frames::{Frame, ReferenceFrame};
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::series::TimeSeries;
use lox_time::time_scales::{Tdb, TimeScale};
use std::f64::consts::PI;
use thiserror::Error;

use lox_core::units::{AngularRate, Distance};

use lox_core::error::LoxError;

use crate::events::{DetectFn, Differentiable, RateBounded};
use lox_orbits::ground::{GroundLocation, Observables};
use lox_orbits::orbits::Trajectory;

// ---------------------------------------------------------------------------
// Line-of-sight geometry
// ---------------------------------------------------------------------------

// Salvatore Alfano, David Negron, Jr., and Jennifer L. Moore
// Rapid Determination of Satellite Visibility Periods
// The Journal of the Astronautical Sciences. Vol. 40, No. 2, April-June 1992, pp. 281-296

/// Computes the line-of-sight angle for a spherical body with the given `radius`.
///
/// Returns a positive value when the two position vectors `r1` and `r2` have
/// mutual line of sight, and a negative value when they are occluded.
pub fn line_of_sight(radius: f64, r1: DVec3, r2: DVec3) -> f64 {
    let r1n = r1.length();
    let r2n = r2.length();
    let theta1 = radius / r1n;
    let theta2 = radius / r2n;
    // Clamp to the domain of `acos` to avoid floating point errors when `r1 == r2`.
    let theta = (r1.dot(r2) / r1n / r2n).clamp(-1.0, 1.0);
    theta1.acos() + theta2.acos() - theta.acos()
}

/// Computes the line-of-sight angle for a spheroid body, scaling the z-axis
/// to account for oblateness before delegating to [`line_of_sight`].
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

/// Extension trait for computing line-of-sight between two position vectors
/// around a body that implements [`TrySpheroid`] and [`TryMeanRadius`].
pub trait LineOfSight: TrySpheroid + TryMeanRadius {
    /// Computes the line-of-sight angle, using a spheroid model when available.
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

/// Errors from constructing an [`ElevationMask`].
#[derive(Debug, Clone, Error, PartialEq)]
pub enum ElevationMaskError {
    /// The azimuth range does not span \[-π, π\].
    #[error("invalid azimuth range: {}..{}", .0.to_degrees(), .1.to_degrees())]
    InvalidAzimuthRange(f64, f64),
    /// Failed to construct the interpolation series.
    #[error("series error")]
    SeriesError(#[from] SeriesError),
}

/// Minimum elevation angle as a function of azimuth.
///
/// Can be either a constant angle ([`Fixed`](Self::Fixed)) or an
/// azimuth-dependent piecewise-linear profile ([`Variable`](Self::Variable)).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ElevationMask {
    /// Constant minimum elevation angle (radians).
    Fixed(f64),
    /// Azimuth-dependent minimum elevation angle (interpolated series).
    Variable(Series),
}

impl ElevationMask {
    /// Creates a variable elevation mask from paired azimuth/elevation vectors (radians).
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

    /// Creates a fixed elevation mask with a constant minimum elevation (radians).
    pub fn with_fixed_elevation(elevation: f64) -> Self {
        Self::Fixed(elevation)
    }

    /// Returns the minimum elevation angle (radians) at the given azimuth.
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

// ---------------------------------------------------------------------------
// Pass
// ---------------------------------------------------------------------------

/// A visibility pass between a ground station and spacecraft.
///
/// Stores the time interval, sampled times, observables, and [`TimeSeries`] for
/// each observable channel to support interpolation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pass {
    interval: TimeInterval,
    times: Vec<Time>,
    observables: Vec<Observables>,
    azimuth_series: TimeSeries,
    elevation_series: TimeSeries,
    range_series: TimeSeries,
    range_rate_series: TimeSeries,
}

impl Pass {
    /// Create a Pass from an interval, calculating observables for times when
    /// the satellite is above the elevation mask.
    ///
    /// Returns `None` if the satellite is never above the mask within the interval.
    pub fn from_interval(
        interval: TimeInterval<TimeScale>,
        time_resolution: TimeDelta,
        gs: &GroundLocation,
        mask: &ElevationMask,
        sc: &lox_orbits::orbits::Trajectory,
        body_fixed_frame: Frame,
    ) -> Option<Pass> {
        let mut pass_times = Vec::new();
        let mut pass_observables = Vec::new();

        for current_time in interval.step_by(time_resolution) {
            let state = sc.at(current_time);
            let state_bf = state
                .try_to_frame(body_fixed_frame, &DefaultRotationProvider)
                .unwrap();
            let obs = gs.observables_dynamic(state_bf);

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

impl Pass {
    /// Create a new Pass with Series-based interpolation.
    ///
    /// Requires at least 2 data points so that the observables can be
    /// interpolated. Returns `Err(SeriesError::InsufficientPoints)` otherwise.
    pub fn try_new(
        interval: TimeInterval,
        times: Vec<Time>,
        observables: Vec<Observables>,
    ) -> Result<Self, SeriesError>
where {
        if times.len() < 2 {
            return Err(SeriesError::InsufficientPoints(times.len()));
        }

        let epoch = interval.start();
        let time_seconds: Vec<f64> = times
            .iter()
            .map(|t| (*t - epoch).to_seconds().to_f64())
            .collect();
        let azimuths: Vec<f64> = observables.iter().map(|o| o.azimuth()).collect();
        let elevations: Vec<f64> = observables.iter().map(|o| o.elevation()).collect();
        let ranges: Vec<f64> = observables.iter().map(|o| o.range()).collect();
        let range_rates: Vec<f64> = observables.iter().map(|o| o.range_rate()).collect();

        let azimuth_series = TimeSeries::try_new(
            epoch,
            time_seconds.clone(),
            azimuths,
            InterpolationType::Linear,
        )?;
        let elevation_series = TimeSeries::try_new(
            epoch,
            time_seconds.clone(),
            elevations,
            InterpolationType::Linear,
        )?;
        let range_series = TimeSeries::try_new(
            epoch,
            time_seconds.clone(),
            ranges,
            InterpolationType::Linear,
        )?;
        let range_rate_series =
            TimeSeries::try_new(epoch, time_seconds, range_rates, InterpolationType::Linear)?;

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

    /// Returns the time interval of this pass.
    pub fn interval(&self) -> &TimeInterval {
        &self.interval
    }

    /// Returns the sampled time points within the pass.
    pub fn times(&self) -> &[Time] {
        &self.times
    }

    /// Returns the sampled observables at each time point.
    pub fn observables(&self) -> &[Observables] {
        &self.observables
    }

    /// Interpolates observables at the given time, or `None` if outside the pass interval.
    pub fn interpolate(&self, time: Time) -> Option<Observables>
where {
        if time < self.interval.start() || time > self.interval.end() {
            return None;
        }

        if self.times.is_empty() {
            return None;
        }

        let azimuth = self.azimuth_series.interpolate(time);
        let elevation = self.elevation_series.interpolate(time);
        let range = self.range_series.interpolate(time);
        let range_rate = self.range_rate_series.interpolate(time);

        Some(Observables::new(azimuth, elevation, range, range_rate))
    }
}

// ---------------------------------------------------------------------------
// DetectFn error type
// ---------------------------------------------------------------------------

/// Errors from detect function evaluation.
#[derive(Debug, Error)]
pub enum EvalError {
    /// Frame rotation failed.
    #[error("rotation error: {0}")]
    Rotation(Box<dyn std::error::Error + Send + Sync>),
    /// A required origin property (e.g. mean radius) is undefined.
    #[error(transparent)]
    UndefinedProperty(#[from] UndefinedOriginPropertyError),
    /// Ephemeris lookup failed.
    #[error("ephemeris error: {0}")]
    Ephemeris(Box<dyn std::error::Error + Send + Sync>),
}

// `events::DetectFn` requires `Error: Into<LoxError>` so the lazy machinery can
// erase detector-specific errors.
impl From<EvalError> for LoxError {
    fn from(e: EvalError) -> Self {
        LoxError::new(e)
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
/// The detect function:
/// 1. Interpolates the spacecraft trajectory at the given time
/// 2. Rotates the state into the body-fixed frame via `TryRotation<R, Frame, TimeScale>`
/// 3. Computes observables (azimuth, elevation, range, range rate)
/// 4. Returns elevation minus minimum elevation from the mask
///
/// `T` is a *trajectory handle* rather than a plain reference: `&Trajectory` on
/// the borrowing paths, `Arc<Trajectory>` where the detector has to outlive the
/// scope that built it, which is what the streaming engine's `'static` iterators
/// require. Every detector in this module is generic the same way, so one
/// implementation serves both.
///
/// The bound is [`Deref`] rather than [`Borrow`](std::borrow::Borrow) on
/// purpose: `Deref::Target` is unique, so `O` and `R` are inferred from the
/// handle, whereas `&T` implements both `Borrow<T>` and `Borrow<&T>` and would
/// leave the ephemeris handle ambiguous.
pub(crate) struct ElevationDetectFn<T> {
    pub(crate) gs: GroundLocation,
    pub(crate) mask: ElevationMask,
    pub(crate) sc: T,
    pub(crate) body_fixed_frame: Frame,
}

impl<O, R, T> DetectFn for ElevationDetectFn<T>
where
    O: TrySpheroid + Copy,
    R: ReferenceFrame + Copy,
    T: Deref<Target = Trajectory<O, R>>,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    type Error = EvalError;

    fn eval(&self, time: Time) -> Result<f64, Self::Error> {
        let sc = self.sc.at(time.into_dynamic());
        let sc = sc
            .try_to_frame(self.body_fixed_frame, &DefaultRotationProvider)
            .map_err(|e| EvalError::Rotation(Box::new(e)))?;
        let obs = self.gs.compute_observables(sc.position(), sc.velocity());
        Ok(obs.elevation() - self.mask.min_elevation(obs.azimuth()))
    }
}

impl<O, R, T> RateBounded for ElevationDetectFn<T>
where
    O: TrySpheroid + Copy,
    R: ReferenceFrame + Copy,
    T: Deref<Target = Trajectory<O, R>>,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    fn eval_bounded(&self, time: Time) -> Result<(f64, f64), Self::Error> {
        let sc = self.sc.at(time);
        let sc = sc
            .try_to_frame(self.body_fixed_frame, &DefaultRotationProvider)
            .map_err(|e| EvalError::Rotation(Box::new(e)))?;
        let obs = self.gs.compute_observables(sc.position(), sc.velocity());
        let value = obs.elevation() - self.mask.min_elevation(obs.azimuth());

        // The topocentric elevation-angle rate is bounded by the transverse
        // angular rate of the line-of-sight vector, |v_bf| / range (the
        // body-fixed velocity is the analytic derivative of the interpolated
        // position). A fixed mask contributes no azimuth-dependent term, so
        // this bounds the whole crossing function. Azimuth-varying masks would
        // additionally need the mask slope; until that is modelled they report
        // an unbounded rate, which degrades adaptive stepping to the fixed step.
        let bound = match &self.mask {
            ElevationMask::Fixed(_) => {
                let range = obs.range();
                if range > 0.0 {
                    sc.velocity().length() / range
                } else {
                    f64::INFINITY
                }
            }
            ElevationMask::Variable(_) => f64::INFINITY,
        };
        Ok((value, bound))
    }
}

/// Slant-range threshold detector for a ground station / spacecraft pair.
///
/// Costs the same body-fixed rotation as [`ElevationDetectFn`], so it earns
/// nothing by running first; it is staged *inside* the elevation windows
/// instead.
///
/// Only the pipeline range gate uses it — the eager path has no ground-space
/// range knob — so it is dead until the hard cut (plan step 7).
#[allow(dead_code)]
pub(crate) struct GroundSpaceRangeDetectFn<T> {
    pub(crate) gs: GroundLocation,
    pub(crate) sc: T,
    pub(crate) body_fixed_frame: Frame,
    pub(crate) threshold: Distance,
    pub(crate) direction: RangeDirection,
}

impl<O, R, T> DetectFn for GroundSpaceRangeDetectFn<T>
where
    O: TrySpheroid + Copy,
    R: ReferenceFrame + Copy,
    T: Deref<Target = Trajectory<O, R>>,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    type Error = EvalError;

    fn eval(&self, time: Time) -> Result<f64, Self::Error> {
        let sc = self.sc.at(time.into_dynamic());
        let sc = sc
            .try_to_frame(self.body_fixed_frame, &DefaultRotationProvider)
            .map_err(|e| EvalError::Rotation(Box::new(e)))?;
        let obs = self.gs.compute_observables(sc.position(), sc.velocity());
        Ok(self
            .direction
            .residual(obs.range(), self.threshold.to_meters()))
    }
}

impl<O, R, T> RateBounded for GroundSpaceRangeDetectFn<T>
where
    O: TrySpheroid + Copy,
    R: ReferenceFrame + Copy,
    T: Deref<Target = Trajectory<O, R>>,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    fn eval_bounded(&self, time: Time) -> Result<(f64, f64), Self::Error> {
        let sc = self.sc.at(time);
        let sc = sc
            .try_to_frame(self.body_fixed_frame, &DefaultRotationProvider)
            .map_err(|e| EvalError::Rotation(Box::new(e)))?;
        let obs = self.gs.compute_observables(sc.position(), sc.velocity());
        let value = self
            .direction
            .residual(obs.range(), self.threshold.to_meters());
        // d/dt |r| = (r·v)/|r|, so |d/dt| <= |v| by Cauchy-Schwarz. The
        // body-fixed velocity already accounts for the station's co-rotation.
        Ok((value, sc.velocity().length()))
    }
}

/// Line-of-sight between a ground station and spacecraft, relative to an
/// occulting body.
pub(crate) struct LineOfSightDetectFn<T, E> {
    pub(crate) gs: GroundLocation,
    pub(crate) sc: T,
    pub(crate) body: Origin,
    pub(crate) ephemeris: E,
    pub(crate) body_fixed_frame: Frame,
}

impl<O, R, T, E> DetectFn for LineOfSightDetectFn<T, E>
where
    O: TrySpheroid + Copy,
    R: ReferenceFrame + Copy,
    T: Deref<Target = Trajectory<O, R>>,
    E: Deref,
    E::Target: Ephemeris,
    <E::Target as Ephemeris>::Error: 'static,
    DefaultRotationProvider: TryRotation<Frame, R, TimeScale>,
    <DefaultRotationProvider as TryRotation<Frame, R, TimeScale>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    type Error = EvalError;

    fn eval(&self, time: Time) -> Result<f64, Self::Error> {
        // Convert to TDB for ephemeris lookup (infallible via DefaultOffsetProvider).
        let sc = &*self.sc;
        let tdb = time.to_scale(Tdb);
        let r_body = self
            .ephemeris
            .position(tdb, sc.origin(), self.body)
            .map_err(|e| EvalError::Ephemeris(Box::new(e)))?;
        let r_sc = sc.at(time.into_dynamic()).position() - r_body;
        // Compute ground station position in the scenario frame R by rotating
        // from body-fixed → R.
        let rot = DefaultRotationProvider
            .try_rotation(self.body_fixed_frame, sc.reference_frame(), time)
            .map_err(|e| EvalError::Rotation(Box::new(e)))?;
        let (r_gs_frame, _) = rot.rotate_state(self.gs.body_fixed_position(), DVec3::ZERO);
        let r_gs = r_gs_frame - r_body;
        Ok(self.body.line_of_sight(r_gs, r_sc)?)
    }
}

/// Line-of-sight between two spacecraft, relative to a non-central occulting body. Uses the
/// ephemeris to compute the body position.
pub(crate) struct InterSatLosOccluderDetectFn<T, E> {
    pub(crate) sc1: T,
    pub(crate) sc2: T,
    pub(crate) body: Origin,
    pub(crate) ephemeris: E,
}

impl<O, R, T, E> DetectFn for InterSatLosOccluderDetectFn<T, E>
where
    O: CoordinateOrigin + Copy,
    R: ReferenceFrame + Copy,
    T: Deref<Target = Trajectory<O, R>>,
    E: Deref,
    E::Target: Ephemeris,
    <E::Target as Ephemeris>::Error: 'static,
{
    type Error = EvalError;

    fn eval(&self, time: Time) -> Result<f64, Self::Error> {
        let (sc1, sc2) = (&*self.sc1, &*self.sc2);
        let tdb = time.to_scale(Tdb);
        let r_body = self
            .ephemeris
            .position(tdb, sc1.origin(), self.body)
            .map_err(|e| EvalError::Ephemeris(Box::new(e)))?;
        let r_sc1 = sc1.at(time.into_dynamic()).position() - r_body;
        let r_sc2 = sc2.at(time.into_dynamic()).position() - r_body;
        Ok(self.body.line_of_sight(r_sc1, r_sc2)?)
    }
}

/// Line-of-sight between two spacecraft when the occluding body is the
/// trajectories' origin. `r_body == 0` by construction, so no ephemeris
/// lookup is required.
pub(crate) struct InterSatLosCentralBodyDetectFn<T> {
    pub(crate) sc1: T,
    pub(crate) sc2: T,
    pub(crate) body: Origin,
}

impl<O, R, T> DetectFn for InterSatLosCentralBodyDetectFn<T>
where
    O: CoordinateOrigin + Copy,
    R: ReferenceFrame + Copy,
    T: Deref<Target = Trajectory<O, R>>,
{
    type Error = EvalError;

    fn eval(&self, time: Time) -> Result<f64, Self::Error> {
        let r_sc1 = self.sc1.at(time.into_dynamic()).position();
        let r_sc2 = self.sc2.at(time.into_dynamic()).position();
        Ok(self.body.line_of_sight(r_sc1, r_sc2)?)
    }
}

/// Direction for a range threshold comparison.
#[derive(Clone, Copy)]
pub(crate) enum RangeDirection {
    /// Positive when range < threshold (i.e. `threshold - range`).
    Max,
    /// Positive when range > threshold (i.e. `range - threshold`).
    Min,
}

impl RangeDirection {
    /// Signs a range against `threshold` so the result is positive inside the
    /// admissible region.
    fn residual(self, range: f64, threshold: f64) -> f64 {
        match self {
            RangeDirection::Max => threshold - range,
            RangeDirection::Min => range - threshold,
        }
    }
}

/// Range threshold detector for inter-satellite pairs.
pub(crate) struct InterSatelliteRangeDetectFn<T> {
    pub(crate) sc1: T,
    pub(crate) sc2: T,
    pub(crate) threshold: Distance,
    pub(crate) direction: RangeDirection,
}

impl<O, R, T> DetectFn for InterSatelliteRangeDetectFn<T>
where
    O: CoordinateOrigin + Copy,
    R: ReferenceFrame + Copy,
    T: Deref<Target = Trajectory<O, R>>,
{
    type Error = EvalError;

    fn eval(&self, time: Time) -> Result<f64, Self::Error> {
        let r1 = self.sc1.at(time.into_dynamic()).position();
        let r2 = self.sc2.at(time.into_dynamic()).position();
        let range = (r1 - r2).length();
        Ok(self.direction.residual(range, self.threshold.to_meters()))
    }
}

impl<O, R, T> RateBounded for InterSatelliteRangeDetectFn<T>
where
    O: CoordinateOrigin + Copy,
    R: ReferenceFrame + Copy,
    T: Deref<Target = Trajectory<O, R>>,
{
    fn eval_bounded(&self, time: Time) -> Result<(f64, f64), Self::Error> {
        let s1 = self.sc1.at(time);
        let s2 = self.sc2.at(time);
        let range = (s1.position() - s2.position()).length();
        let value = self.direction.residual(range, self.threshold.to_meters());
        // d/dt (±range) = ±(r·v)/|r|, so |d/dt| ≤ |v_rel| by Cauchy–Schwarz.
        // Velocity is the analytic derivative of the position interpolant, so
        // this bound comes free from the same `at` lookups.
        let bound = (s1.velocity() - s2.velocity()).length();
        Ok((value, bound))
    }
}

impl<O, R, T> Differentiable for InterSatelliteRangeDetectFn<T>
where
    O: CoordinateOrigin + Copy,
    R: ReferenceFrame + Copy,
    T: Deref<Target = Trajectory<O, R>>,
{
    fn eval_derivative(&self, time: Time) -> Result<(f64, f64), Self::Error> {
        let s1 = self.sc1.at(time);
        let s2 = self.sc2.at(time);
        let r = s1.position() - s2.position();
        let v = s1.velocity() - s2.velocity();
        let range = r.length();
        // d/dt |r| = (r·v)/|r|; zero relative separation has no well-defined
        // rate, so report a flat derivative there.
        let d_range = if range > 0.0 { r.dot(v) / range } else { 0.0 };
        let value = self.direction.residual(range, self.threshold.to_meters());
        let derivative = match self.direction {
            RangeDirection::Max => -d_range,
            RangeDirection::Min => d_range,
        };
        Ok((value, derivative))
    }
}

/// Slew rate (angular rate) threshold detector for inter-satellite pairs.
///
/// The angular rate ω = |r × v| / |r|² is symmetric between the two
/// spacecraft.  The detector returns `threshold - ω`, positive when the
/// angular rate is within the limit.
pub(crate) struct InterSatelliteSlewRateDetectFn<T> {
    pub(crate) sc1: T,
    pub(crate) sc2: T,
    pub(crate) threshold: AngularRate,
}

impl<O, R, T> DetectFn for InterSatelliteSlewRateDetectFn<T>
where
    O: CoordinateOrigin + Copy,
    R: ReferenceFrame + Copy,
    T: Deref<Target = Trajectory<O, R>>,
{
    type Error = EvalError;

    fn eval(&self, time: Time) -> Result<f64, Self::Error> {
        let s1 = self.sc1.at(time.into_dynamic());
        let s2 = self.sc2.at(time.into_dynamic());
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

// The eager `VisibilityAnalysis` / `VisibilityResults` implementation moved to
// `crate::legacy` for one commit while the Python bindings are ported; these are
// the pipeline-backed replacements at the canonical paths.
pub use crate::pipeline::analyses::{InterSatelliteAnalysis, VisibilityAnalysis};

#[cfg(test)]
mod tests {
    // Explicit imports shadow the glob below, so these tests still exercise the
    // eager implementation they were written for. They are ported to the
    // pipeline API in the commit that deletes `legacy`.
    use crate::assets::{AssetId, GroundStation, Scenario, Spacecraft};
    use crate::legacy::{PairType, PassError, VisibilityAnalysis};
    use lox_orbits::orbits::Ensemble;
    use std::collections::HashMap;

    use lox_approx::assert_approx_eq;
    use lox_bodies::{Earth, Spheroid};
    use lox_core::coords::LonLatAlt;
    use lox_core::units::Distance;
    use lox_ephem::spk::parser::Spk;
    use lox_orbits::propagators::OrbitSource;
    use lox_test_utils::{data_file, read_data_file};
    use lox_time::time_scales::{Tai, TimeScale};
    use lox_time::utc::Utc;
    use std::iter::zip;
    use std::sync::OnceLock;

    use super::*;
    use lox_frames::Icrf;
    use lox_orbits::ground::GroundLocation;
    use lox_orbits::orbits::Trajectory;

    /// Build a Scenario + Ensemble from ground/space assets and a TimeScale interval.
    fn make_scenario_and_ensemble(
        ground_assets: &[GroundStation],
        space_assets: &[Spacecraft],
        interval: TimeInterval<TimeScale>,
    ) -> (Scenario<Origin, Frame>, Ensemble<AssetId, Origin, Frame>) {
        let scenario_interval = TimeInterval::new(interval.start(), interval.end());
        let scenario = Scenario::with_interval(scenario_interval, Origin::Earth, Frame::Icrf)
            .with_ground_stations(ground_assets)
            .with_spacecraft(space_assets);
        // Build ensemble from OrbitSource::Trajectory entries
        let mut map = HashMap::new();
        for sc in space_assets {
            if let OrbitSource::Trajectory(traj) = sc.orbit() {
                // Re-tag Trajectory as Ensemble<Origin, Frame>
                let (epoch, origin, frame, data) = traj.clone().into_parts();
                let typed =
                    Trajectory::from_parts(epoch.with_scale(TimeScale::Tai), origin, frame, data);
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
        let sc = spacecraft_trajectory_dynamic();
        let gs_traj = ground_station_trajectory();
        let gs = location_dynamic();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let expected: Vec<f64> = read_data_file("elevation.csv")
            .lines()
            .map(|line| line.parse::<f64>().unwrap().to_radians())
            .collect();
        // Build a typed trajectory for the ElevationDetectFn
        let (epoch, o, f, data) = sc.clone().into_parts();
        let typed_sc = Trajectory::from_parts(epoch.with_scale(TimeScale::Tai), o, f, data);
        let elev_fn = ElevationDetectFn {
            gs,
            mask,
            sc: &typed_sc,
            body_fixed_frame: Frame::Iau(Origin::Earth),
        };
        // Use the ground station trajectory times
        let actual: Vec<f64> = gs_traj
            .times()
            .iter()
            .map(|t| {
                let tai_time = t.to_scale(Tai);
                elev_fn.eval(tai_time.into_dynamic()).unwrap()
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
        let gs_loc = location_dynamic();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let sc_traj = spacecraft_trajectory_dynamic();
        let gs = GroundStation::new("cebreros", gs_loc, mask);
        let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(sc_traj.clone()));
        let spk = ephemeris();
        let ground_assets = [gs.clone()];
        let space_assets = [sc.clone()];
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let (scenario, ensemble) =
            make_scenario_and_ensemble(&ground_assets, &space_assets, interval);
        let analysis =
            VisibilityAnalysis::new(&scenario, &ensemble).with_occulting_bodies(spk, vec![]);
        let results = analysis.compute().expect("visibility");
        let intervals = results
            .intervals_for(gs.id(), sc.id())
            .expect("pair not found");
        let expected = contacts_tai();
        assert_eq!(intervals.len(), expected.len());
        assert_approx_eq!(expected, intervals.to_vec(), rtol <= 1e-4);
    }

    #[test]
    fn test_visibility_no_ephemeris() {
        let gs_loc = location_dynamic();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let sc_traj = spacecraft_trajectory_dynamic();
        let gs = GroundStation::new("cebreros", gs_loc, mask);
        let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(sc_traj.clone()));
        let ground_assets = [gs.clone()];
        let space_assets = [sc.clone()];
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let (scenario, ensemble) =
            make_scenario_and_ensemble(&ground_assets, &space_assets, interval);

        // No ephemeris provided — must compile and produce the same intervals
        // as test_visibility (which used the ephemeris but had no occulters).
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble);
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
        let gs_loc = location_dynamic();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let sc_traj = spacecraft_trajectory_dynamic();
        let gs = GroundStation::new("cebreros", gs_loc, mask);
        let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(sc_traj.clone()));
        let spk = ephemeris();
        let ground_assets = [gs.clone()];
        let space_assets = [sc.clone()];
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let (scenario, ensemble) =
            make_scenario_and_ensemble(&ground_assets, &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble)
            .with_occulting_bodies(spk, vec![Origin::Moon]);
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
        let gs_loc = location_dynamic();
        let mask = ElevationMask::with_fixed_elevation(10.0_f64.to_radians());
        let sc_traj = spacecraft_trajectory_dynamic();
        let gs = GroundStation::new("cebreros", gs_loc, mask);
        let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(sc_traj.clone()));
        let ground_assets = [gs.clone()];
        let space_assets = [sc.clone()];
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let (scenario, ensemble) =
            make_scenario_and_ensemble(&ground_assets, &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble);
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

    fn ground_station_trajectory() -> Trajectory<Earth, Icrf> {
        Trajectory::from_csv(&read_data_file("trajectory_cebr.csv"), Earth, Icrf).unwrap()
    }

    fn spacecraft_trajectory_dynamic() -> Trajectory {
        Trajectory::from_csv_dynamic(
            &read_data_file("trajectory_lunar.csv"),
            Origin::Earth,
            Frame::Icrf,
        )
        .unwrap()
    }

    fn location_dynamic() -> GroundLocation<Origin> {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        GroundLocation::try_new(coords, Origin::Earth).unwrap()
    }

    fn contacts_tai() -> Vec<TimeInterval> {
        let mut intervals = vec![];
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_path(data_file("contacts.csv"))
            .unwrap();
        for result in reader.records() {
            let record = result.unwrap();
            let start = record[0].parse::<Utc>().unwrap().to_dynamic_time();
            let end = record[1].parse::<Utc>().unwrap().to_dynamic_time();
            intervals.push(TimeInterval::new(start, end));
        }
        intervals
    }

    fn contacts_combined() -> Vec<TimeInterval<TimeScale>> {
        let mut intervals = vec![];
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_path(data_file("contacts_combined.csv"))
            .unwrap();
        for result in reader.records() {
            let record = result.unwrap();
            let start = record[0].parse::<Utc>().unwrap().to_dynamic_time();
            let end = record[1].parse::<Utc>().unwrap().to_dynamic_time();
            intervals.push(TimeInterval::new(start, end));
        }
        intervals
    }

    #[test]
    fn test_visibility_adaptive_matches_uniform() {
        let gs_loc = location_dynamic();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let sc_traj = spacecraft_trajectory_dynamic();
        let gs = GroundStation::new("cebreros", gs_loc, mask);
        let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(sc_traj.clone()));
        let ground_assets = [gs.clone()];
        let space_assets = [sc.clone()];
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let (scenario, ensemble) =
            make_scenario_and_ensemble(&ground_assets, &space_assets, interval);

        let uniform = VisibilityAnalysis::new(&scenario, &ensemble)
            .compute()
            .expect("uniform visibility");
        let adaptive = VisibilityAnalysis::new(&scenario, &ensemble)
            .with_adaptive_detection()
            .compute()
            .expect("adaptive visibility");

        // Adaptive strides by the detect function's rate bound, so it samples
        // far fewer points; it must still bracket the same crossings.
        let uniform = uniform.intervals_for(gs.id(), sc.id()).expect("pair");
        let adaptive = adaptive.intervals_for(gs.id(), sc.id()).expect("pair");
        assert_eq!(uniform.len(), adaptive.len());
        assert_approx_eq!(uniform.to_vec(), adaptive.to_vec(), rtol <= 1e-6);
    }

    #[test]
    fn test_inter_satellite_visibility() {
        let sc_traj = spacecraft_trajectory_dynamic();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let sc1 = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj.clone()));
        let sc2 = Spacecraft::new("sc2", OrbitSource::Trajectory(sc_traj));
        let space_assets = [sc1.clone(), sc2.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble).with_inter_satellite();
        let results = analysis.compute().unwrap();
        let intervals = results
            .intervals_for(sc1.id(), sc2.id())
            .expect("pair not found");
        // Colocated spacecraft are always visible to each other.
        assert_eq!(intervals.len(), 1);
        let scenario_interval = TimeInterval::new(interval.start(), interval.end());
        assert_approx_eq!(
            intervals[0].start(),
            scenario_interval.start(),
            rtol <= 1e-10
        );
        assert_approx_eq!(intervals[0].end(), scenario_interval.end(), rtol <= 1e-10);
    }

    #[test]
    fn test_inter_satellite_visibility_with_range_filter() {
        let sc_traj = spacecraft_trajectory_dynamic();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let sc1 = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj.clone()));
        let sc2 = Spacecraft::new("sc2", OrbitSource::Trajectory(sc_traj));
        let space_assets = [sc1.clone(), sc2.clone()];

        // Colocated spacecraft have range = 0. A max_range filter with a large
        // threshold should still return the full interval.
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble)
            .with_inter_satellite()
            .with_max_range(Distance::kilometers(1000.0));
        let results = analysis.compute().unwrap();
        let intervals = results
            .intervals_for(sc1.id(), sc2.id())
            .expect("pair not found");
        let scenario_interval = TimeInterval::new(interval.start(), interval.end());
        assert_eq!(intervals.len(), 1);
        assert_approx_eq!(
            intervals[0].start(),
            scenario_interval.start(),
            rtol <= 1e-10
        );
        assert_approx_eq!(intervals[0].end(), scenario_interval.end(), rtol <= 1e-10);

        // A min_range filter with a positive threshold should exclude colocated
        // spacecraft entirely (range = 0 < threshold at all times).
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble)
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
        let sc_traj = spacecraft_trajectory_dynamic();
        let (epoch, origin, frame, data) = sc_traj.into_parts();
        let typed = Trajectory::from_parts(epoch.with_scale(TimeScale::Tai), origin, frame, data);
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
    fn test_inter_sat_los_central_body_detect_fn() {
        let sc_traj = spacecraft_trajectory_dynamic();
        let (epoch, origin, frame, data) = sc_traj.clone().into_parts();
        let typed = Trajectory::from_parts(epoch.with_scale(TimeScale::Tai), origin, frame, data);
        let detect = InterSatLosCentralBodyDetectFn {
            sc1: &typed,
            sc2: &typed,
            body: Origin::Earth,
        };
        let time = typed.start_time();
        let val = detect.eval(time).unwrap();
        // Colocated spacecraft -> dot(r1, r2) = |r|^2 -> theta = 0,
        // theta1 == theta2 == acos(R/|r|) -> result = 2*acos(R/|r|) > 0.
        assert!(val > 0.0);
    }

    #[test]
    fn test_inter_satellite_visibility_with_slew_rate() {
        let sc_traj = spacecraft_trajectory_dynamic();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());

        // Colocated spacecraft have ω = 0. A generous slew rate limit should
        // keep the full interval.
        let sc1 = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj.clone()))
            .with_max_slew_rate(AngularRate::degrees_per_second(10.0));
        let sc2 = Spacecraft::new("sc2", OrbitSource::Trajectory(sc_traj))
            .with_max_slew_rate(AngularRate::degrees_per_second(5.0));
        let space_assets = [sc1.clone(), sc2.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble).with_inter_satellite();
        let results = analysis.compute().unwrap();
        let intervals = results
            .intervals_for(sc1.id(), sc2.id())
            .expect("pair not found");
        let scenario_interval = TimeInterval::new(interval.start(), interval.end());
        // ω = 0 everywhere, so full interval should be returned.
        assert_eq!(intervals.len(), 1);
        assert_approx_eq!(
            intervals[0].start(),
            scenario_interval.start(),
            rtol <= 1e-10
        );
        assert_approx_eq!(intervals[0].end(), scenario_interval.end(), rtol <= 1e-10);
    }

    #[test]
    fn test_space_asset_max_slew_rate() {
        let sc_traj = spacecraft_trajectory_dynamic();
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

    fn oneweb_trajectories() -> (Trajectory, Trajectory) {
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
            .propagate(interval.into_dynamic())
            .unwrap()
            .into_dynamic();
        let traj2 = sgp4_2
            .with_step(TimeDelta::from_seconds(10))
            .propagate(interval.into_dynamic())
            .unwrap()
            .into_dynamic();

        (traj1, traj2)
    }

    #[test]
    fn test_slew_rate_trims_windows_for_crossing_orbits() {
        let (traj1, traj2) = oneweb_trajectories();
        let interval = TimeInterval::new(traj1.start_time(), traj1.end_time());

        // Without slew rate constraint: should have visibility (central body
        // LOS is always applied but these LEO sats have mutual visibility).
        let sc1_no_limit = Spacecraft::new("ow12", OrbitSource::Trajectory(traj1.clone()));
        let sc2_no_limit = Spacecraft::new("ow17", OrbitSource::Trajectory(traj2.clone()));
        let space_assets = [sc1_no_limit.clone(), sc2_no_limit.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble).with_inter_satellite();
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
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble).with_inter_satellite();
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
        let sc_traj = spacecraft_trajectory_dynamic();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());

        // Only sc1 has a slew rate limit — exercises the (Some(a), None) branch.
        let sc1 = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj.clone()))
            .with_max_slew_rate(AngularRate::degrees_per_second(10.0));
        let sc2 = Spacecraft::new("sc2", OrbitSource::Trajectory(sc_traj));
        let space_assets = [sc1.clone(), sc2.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble).with_inter_satellite();
        let results = analysis.compute().unwrap();
        let intervals = results
            .intervals_for(sc1.id(), sc2.id())
            .expect("pair not found");
        // Colocated → ω = 0, full interval returned.
        assert_eq!(intervals.len(), 1);
    }

    #[test]
    fn test_inter_satellite_asymmetric_slew_rate_sc2_only() {
        let sc_traj = spacecraft_trajectory_dynamic();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());

        // Only sc2 has a slew rate limit — exercises the (None, Some(b)) branch.
        let sc1 = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj.clone()));
        let sc2 = Spacecraft::new("sc2", OrbitSource::Trajectory(sc_traj))
            .with_max_slew_rate(AngularRate::degrees_per_second(10.0));
        let space_assets = [sc1.clone(), sc2.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble).with_inter_satellite();
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
        let space_assets = [sc1.clone(), sc2.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);
        // Set both min and max range to exercise the intersection branch.
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble)
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

    #[test]
    fn test_ground_space_filter() {
        let gs_loc = location_dynamic();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let gs1 = GroundStation::new("cebreros", gs_loc.clone(), mask.clone());
        let gs2 = GroundStation::new("malargue", gs_loc, mask);
        let sc_traj = spacecraft_trajectory_dynamic();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let sc1 = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj.clone()));
        let sc2 = Spacecraft::new("sc2", OrbitSource::Trajectory(sc_traj));
        let ground_assets = [gs1.clone(), gs2.clone()];
        let space_assets = [sc1.clone(), sc2.clone()];
        let (scenario, ensemble) =
            make_scenario_and_ensemble(&ground_assets, &space_assets, interval);

        // Only keep pairs involving cebreros.
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble)
            .with_ground_space_filter(|gs, _sc| gs.id().as_str() == "cebreros");
        let results = analysis.compute().unwrap();

        assert_eq!(results.num_pairs(), 2); // cebreros-sc1, cebreros-sc2
        assert!(results.intervals_for(gs1.id(), sc1.id()).is_some());
        assert!(results.intervals_for(gs1.id(), sc2.id()).is_some());
        assert!(results.intervals_for(gs2.id(), sc1.id()).is_none());
        assert!(results.intervals_for(gs2.id(), sc2.id()).is_none());
    }

    #[test]
    fn test_inter_satellite_filter() {
        let sc_traj = spacecraft_trajectory_dynamic();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let sc1 = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj.clone()));
        let sc2 = Spacecraft::new("sc2", OrbitSource::Trajectory(sc_traj.clone()));
        let sc3 = Spacecraft::new("sc3", OrbitSource::Trajectory(sc_traj));
        let space_assets = [sc1.clone(), sc2.clone(), sc3.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);

        let analysis = VisibilityAnalysis::new(&scenario, &ensemble).with_inter_satellite_filter(
            |sc_a, sc_b| {
                let ids = [sc_a.id().as_str(), sc_b.id().as_str()];
                ids.contains(&"sc1") && ids.contains(&"sc3")
            },
        );
        let results = analysis.compute().unwrap();

        assert_eq!(results.num_pairs(), 1);
        assert!(results.intervals_for(sc1.id(), sc3.id()).is_some());
        assert!(results.intervals_for(sc1.id(), sc2.id()).is_none());
        assert!(results.intervals_for(sc2.id(), sc3.id()).is_none());
    }

    #[test]
    fn test_both_filters_combined_with_ground_space() {
        let gs_loc = location_dynamic();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let gs1 = GroundStation::new("cebreros", gs_loc.clone(), mask.clone());
        let gs2 = GroundStation::new("malargue", gs_loc, mask);
        let sc_traj = spacecraft_trajectory_dynamic();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let sc1 = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj.clone()));
        let sc2 = Spacecraft::new("sc2", OrbitSource::Trajectory(sc_traj.clone()));
        let sc3 = Spacecraft::new("sc3", OrbitSource::Trajectory(sc_traj));
        let ground_assets = [gs1.clone(), gs2.clone()];
        let space_assets = [sc1.clone(), sc2.clone(), sc3.clone()];
        let (scenario, ensemble) =
            make_scenario_and_ensemble(&ground_assets, &space_assets, interval);

        let analysis = VisibilityAnalysis::new(&scenario, &ensemble)
            .with_ground_space_filter(|gs, _sc| gs.id().as_str() == "cebreros")
            .with_inter_satellite_filter(|sc_a, sc_b| {
                let ids = [sc_a.id().as_str(), sc_b.id().as_str()];
                ids.contains(&"sc1") && ids.contains(&"sc2")
            });
        let results = analysis.compute().unwrap();

        // 3 ground-space (cebreros × 3 spacecraft) + 1 inter-satellite (sc1-sc2) = 4
        assert_eq!(results.num_pairs(), 4);
        assert!(results.intervals_for(gs1.id(), sc1.id()).is_some());
        assert!(results.intervals_for(gs1.id(), sc2.id()).is_some());
        assert!(results.intervals_for(gs1.id(), sc3.id()).is_some());
        assert!(results.intervals_for(gs2.id(), sc1.id()).is_none());
        assert!(results.intervals_for(sc1.id(), sc2.id()).is_some());
        assert!(results.intervals_for(sc1.id(), sc3.id()).is_none());
    }

    #[test]
    fn test_min_pass_duration_filters_short_passes() {
        let gs_loc = location_dynamic();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let gs = GroundStation::new("cebreros", gs_loc, mask);
        let sc_traj = spacecraft_trajectory_dynamic();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let sc = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj));
        let ground_assets = [gs];
        let space_assets = [sc];
        let (scenario, ensemble) =
            make_scenario_and_ensemble(&ground_assets, &space_assets, interval);

        // Without min_pass_duration.
        let results_all = VisibilityAnalysis::new(&scenario, &ensemble)
            .compute()
            .unwrap();
        let all_count = results_all
            .intervals_for(ground_assets[0].id(), space_assets[0].id())
            .map_or(0, |v| v.len());

        // With a large min_pass_duration (should filter short passes).
        let results_filtered = VisibilityAnalysis::new(&scenario, &ensemble)
            .with_min_pass_duration(TimeDelta::from_hours(2))
            .compute()
            .unwrap();
        let filtered_count = results_filtered
            .intervals_for(ground_assets[0].id(), space_assets[0].id())
            .map_or(0, |v| v.len());
        assert!(filtered_count <= all_count);

        // With a very small min_pass_duration (coarse step <= step, so no effect).
        let results_small = VisibilityAnalysis::new(&scenario, &ensemble)
            .with_min_pass_duration(TimeDelta::from_seconds(1))
            .compute()
            .unwrap();
        let small_count = results_small
            .intervals_for(ground_assets[0].id(), space_assets[0].id())
            .map_or(0, |v| v.len());
        assert_eq!(small_count, all_count);
    }

    #[test]
    fn test_to_passes_rejects_inter_satellite_pair() {
        let sc_traj = spacecraft_trajectory_dynamic();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let sc1 = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj.clone()));
        let sc2 = Spacecraft::new("sc2", OrbitSource::Trajectory(sc_traj));
        let space_assets = [sc1.clone(), sc2.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, interval);

        let results = VisibilityAnalysis::new(&scenario, &ensemble)
            .with_inter_satellite()
            .compute()
            .unwrap();

        let gs_loc = location_dynamic();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let dummy_traj = Trajectory::from_csv_dynamic(
            &read_data_file("trajectory_lunar.csv"),
            Origin::Earth,
            Frame::Icrf,
        )
        .unwrap();

        let err = results
            .to_passes(
                sc1.id(),
                sc2.id(),
                &gs_loc,
                &mask,
                &dummy_traj,
                TimeDelta::from_seconds(60),
                Frame::Iau(Origin::Earth),
            )
            .unwrap_err();
        assert!(matches!(err, PassError::InterSatellitePair(_, _)));
    }

    #[test]
    fn test_to_passes_unknown_pair_returns_empty() {
        let gs_loc = location_dynamic();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let gs = GroundStation::new("cebreros", gs_loc.clone(), mask.clone());
        let sc_traj = spacecraft_trajectory_dynamic();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let sc = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj));
        let (scenario, ensemble) = make_scenario_and_ensemble(&[gs], &[sc], interval);

        let results = VisibilityAnalysis::new(&scenario, &ensemble)
            .compute()
            .unwrap();

        let dummy_traj = Trajectory::from_csv_dynamic(
            &read_data_file("trajectory_lunar.csv"),
            Origin::Earth,
            Frame::Icrf,
        )
        .unwrap();

        let unknown_id = AssetId::new("nonexistent");
        let passes = results
            .to_passes(
                &unknown_id,
                &unknown_id,
                &gs_loc,
                &mask,
                &dummy_traj,
                TimeDelta::from_seconds(60),
                Frame::Iau(Origin::Earth),
            )
            .unwrap();
        assert!(passes.is_empty());
    }

    #[test]
    fn test_combined_ground_and_inter_satellite() {
        let gs_loc = location_dynamic();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let gs = GroundStation::new("cebreros", gs_loc, mask);
        let sc_traj = spacecraft_trajectory_dynamic();
        let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
        let sc1 = Spacecraft::new("sc1", OrbitSource::Trajectory(sc_traj.clone()));
        let sc2 = Spacecraft::new("sc2", OrbitSource::Trajectory(sc_traj));
        let ground_assets = [gs.clone()];
        let space_assets = [sc1.clone(), sc2.clone()];
        let (scenario, ensemble) =
            make_scenario_and_ensemble(&ground_assets, &space_assets, interval);

        let results = VisibilityAnalysis::new(&scenario, &ensemble)
            .with_inter_satellite()
            .compute()
            .unwrap();

        // 2 ground-space + 1 inter-satellite = 3
        assert_eq!(results.num_pairs(), 3);
        assert!(results.intervals_for(gs.id(), sc1.id()).is_some());
        assert!(results.intervals_for(gs.id(), sc2.id()).is_some());
        assert!(results.intervals_for(sc1.id(), sc2.id()).is_some());

        // Pair types should be correct.
        assert_eq!(
            results.pair_type(gs.id(), sc1.id()),
            Some(PairType::GroundSpace)
        );
        assert_eq!(
            results.pair_type(sc1.id(), sc2.id()),
            Some(PairType::InterSatellite)
        );
    }

    /// ISS (LEO, ~408 km) vs a lunar-transfer spacecraft — widely separated
    /// orbits where Earth occultation is physically meaningful.  Adding the
    /// Moon as an additional occulting body should not *increase* the total
    /// visible duration.
    #[test]
    fn test_inter_satellite_with_occulting_body() {
        use lox_orbits::propagators::Propagator;
        use lox_orbits::propagators::sgp4::{Elements, Sgp4};
        use lox_time::intervals::Interval;

        // ISS TLE near the lunar trajectory epoch (2022-02-01).
        let iss_tle = Elements::from_tle(
            Some("ISS".to_string()),
            b"1 25544U 98067A   22032.58348611  .00006730  00000+0  12674-3 0  9993",
            b"2 25544  51.6444 273.4162 0006808 335.0825 135.5682 15.49587047324581",
        )
        .unwrap();
        let sgp4 = Sgp4::new(iss_tle).unwrap();

        let lunar_traj = spacecraft_trajectory_dynamic();

        // Overlap the ISS propagation with the lunar trajectory's time range.
        let t0 = lunar_traj.start_time().max(sgp4.time().into_dynamic());
        let t1 = t0 + TimeDelta::from_hours(24);
        let scenario_interval = Interval::new(t0.to_scale(Tai), t1.to_scale(Tai));
        let iss_traj = sgp4
            .with_step(TimeDelta::from_seconds(30))
            .propagate(scenario_interval.into_dynamic())
            .unwrap()
            .into_dynamic();

        let inter_interval = TimeInterval::new(t0, t1);
        let sc_iss = Spacecraft::new("iss", OrbitSource::Trajectory(iss_traj));
        let sc_lunar = Spacecraft::new("lunar", OrbitSource::Trajectory(lunar_traj));
        let spk = ephemeris();
        let space_assets = [sc_iss.clone(), sc_lunar.clone()];
        let (scenario, ensemble) = make_scenario_and_ensemble(&[], &space_assets, inter_interval);

        // Without additional occulting bodies (central body Earth is still checked).
        let results_basic = VisibilityAnalysis::new(&scenario, &ensemble)
            .with_inter_satellite()
            .compute()
            .unwrap();

        // With the Moon as an additional occulting body.
        let results_moon = VisibilityAnalysis::new(&scenario, &ensemble)
            .with_inter_satellite()
            .with_occulting_bodies(spk, vec![Origin::Moon])
            .compute()
            .unwrap();

        let basic = results_basic
            .intervals_for(sc_iss.id(), sc_lunar.id())
            .expect("pair not found");
        let with_moon = results_moon
            .intervals_for(sc_iss.id(), sc_lunar.id())
            .expect("pair not found");

        // Both should have intervals (ISS and a lunar probe do see each other).
        assert!(!basic.is_empty(), "ISS-lunar pair should have visibility");
        assert!(!with_moon.is_empty());

        // An additional occluder can only remove visibility, never add it.
        let dur_basic: f64 = basic
            .iter()
            .map(|iv| iv.duration().to_seconds().to_f64())
            .sum();
        let dur_moon: f64 = with_moon
            .iter()
            .map(|iv| iv.duration().to_seconds().to_f64())
            .sum();
        assert!(dur_moon <= dur_basic + 1e-6);
    }

    fn ephemeris() -> &'static Spk {
        static EPHEMERIS: OnceLock<Spk> = OnceLock::new();
        EPHEMERIS.get_or_init(|| Spk::from_file(data_file("spice/de440s.bsp")).unwrap())
    }
}
