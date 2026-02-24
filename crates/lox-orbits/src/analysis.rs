// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use glam::DVec3;
use lox_bodies::{
    DynOrigin, Origin, RotationalElements, Spheroid, TryMeanRadius, TrySpheroid,
    UndefinedOriginPropertyError,
};
use lox_core::types::units::Radians;
use lox_ephem::Ephemeris;
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::TryRotation;
use lox_math::roots::{Brent, RootFinderError};
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

use crate::events::{find_windows, intersect_windows};
use crate::ground::{DynGroundLocation, GroundLocation, Observables};
use crate::orbits::{DynTrajectory, Trajectory};
use lox_frames::{DynFrame, Iau, Icrf};

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

#[derive(Debug, Error)]
pub enum VisibilityError {
    #[error(transparent)]
    RootFinder(#[from] RootFinderError),
    #[error(transparent)]
    Series(#[from] SeriesError),
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

        let mut current_time = interval.start();
        while current_time <= interval.end() {
            let body_fixed = DynFrame::Iau(gs.origin());
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

            current_time = current_time + time_resolution;
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

pub fn elevation_dyn(
    time: DynTime,
    gs: &DynGroundLocation,
    mask: &ElevationMask,
    sc: &DynTrajectory,
) -> Radians {
    let body_fixed = DynFrame::Iau(gs.origin());
    let sc = sc.interpolate_at(time);
    let sc = sc
        .try_to_frame(body_fixed, &DefaultRotationProvider)
        .unwrap();
    let obs = gs.observables_dyn(sc);
    obs.elevation() - mask.min_elevation(obs.azimuth())
}

pub fn visibility_dyn(
    times: &[DynTime],
    gs: &DynGroundLocation,
    mask: &ElevationMask,
    sc: &DynTrajectory,
) -> Result<Vec<TimeInterval<DynTimeScale>>, VisibilityError> {
    if times.len() < 2 {
        return Ok(vec![]);
    }
    let start = *times.first().unwrap();
    let end = *times.last().unwrap();
    let times: Vec<f64> = times
        .iter()
        .map(|t| (*t - start).to_seconds().to_f64())
        .collect();
    let root_finder = Brent::default();
    find_windows(
        |t: f64| {
            Ok(elevation_dyn(
                start + TimeDelta::from_seconds_f64(t),
                gs,
                mask,
                sc,
            ))
        },
        start,
        end,
        &times,
        root_finder,
    )
    .map_err(VisibilityError::RootFinder)
}

pub fn visibility_los(
    times: &[DynTime],
    gs: &DynGroundLocation,
    body: DynOrigin,
    sc: &DynTrajectory,
    ephem: &impl Ephemeris,
) -> Result<Vec<TimeInterval<DynTimeScale>>, RootFinderError> {
    if times.len() < 2 {
        return Ok(vec![]);
    }
    let start = *times.first().unwrap();
    let end = *times.last().unwrap();
    let times: Vec<f64> = times
        .iter()
        .map(|t| (*t - start).to_seconds().to_f64())
        .collect();
    let root_finder = Brent::default();
    find_windows(
        |t| {
            let time = start + TimeDelta::from_seconds_f64(t);
            let tdb = time.try_to_scale(Tdb, &DefaultOffsetProvider).unwrap();
            let r_body = ephem.position(tdb, sc.origin(), body).unwrap();
            let r_sc = sc.interpolate_at(time).position() - r_body;
            // Compute ground station ICRF position by rotating body-fixed position
            let body_fixed_frame = DynFrame::Iau(gs.origin());
            let rot = DefaultRotationProvider
                .try_rotation(body_fixed_frame, DynFrame::Icrf, time)
                .unwrap();
            let (r_gs_icrf, _) = rot.rotate_state(gs.body_fixed_position(), DVec3::ZERO);
            let r_gs = r_gs_icrf - r_body;
            Ok(body.line_of_sight(r_gs, r_sc).unwrap())
        },
        start,
        end,
        &times,
        root_finder,
    )
}

pub fn visibility_combined<E: Ephemeris + Send + Sync>(
    times: &[DynTime],
    gs: &DynGroundLocation,
    mask: &ElevationMask,
    bodies: &[DynOrigin],
    sc: &DynTrajectory,
    ephem: &E,
) -> Result<Vec<DynPass>, VisibilityError> {
    let w1 = visibility_dyn(times, gs, mask, sc)?;
    let wb = bodies
        .par_iter()
        .map(|&body| visibility_los(times, gs, body, sc, ephem))
        .collect::<Result<Vec<Vec<TimeInterval<DynTimeScale>>>, RootFinderError>>()?;
    let mut windows = w1;

    for w2 in wb {
        windows = intersect_windows(&windows, &w2);
    }

    // Convert windows to passes
    let mut passes = Vec::new();

    let time_resolution = if times.len() >= 2 {
        times[1] - times[0]
    } else {
        TimeDelta::from_seconds_f64(60.0)
    };

    for window in windows {
        if let Some(pass) = DynPass::from_interval(window, time_resolution, gs, mask, sc) {
            passes.push(pass);
        }
    }

    Ok(passes)
}

pub fn elevation<T, O, P>(
    time: Time<T>,
    gs: &GroundLocation<O>,
    mask: &ElevationMask,
    sc: &Trajectory<T, O, Icrf>,
    provider: &P,
) -> Radians
where
    T: TimeScale + Copy,
    O: Origin + TrySpheroid + RotationalElements + Copy,
    P: TryRotation<Icrf, Iau<O>, T>,
{
    let body_fixed = Iau::new(gs.origin());
    let sc = sc.interpolate_at(time);
    let sc = sc.try_to_frame(body_fixed, provider).unwrap();
    let obs = gs.observables(sc);
    obs.elevation() - mask.min_elevation(obs.azimuth())
}

pub fn visibility<T, O, P>(
    times: &[Time<T>],
    gs: &GroundLocation<O>,
    mask: &ElevationMask,
    sc: &Trajectory<T, O, Icrf>,
    provider: &P,
) -> Result<Vec<TimeInterval<T>>, RootFinderError>
where
    T: TimeScale + Copy,
    O: Origin + Spheroid + RotationalElements + Copy,
    P: TryRotation<Icrf, Iau<O>, T>,
{
    if times.len() < 2 {
        return Ok(vec![]);
    }
    let start = *times.first().unwrap();
    let end = *times.last().unwrap();
    let times: Vec<f64> = times
        .iter()
        .map(|t| (*t - start).to_seconds().to_f64())
        .collect();
    let root_finder = Brent::default();
    find_windows(
        |t: f64| {
            Ok(elevation(
                start + TimeDelta::from_seconds_f64(t),
                gs,
                mask,
                sc,
                provider,
            ))
        },
        start,
        end,
        &times,
        root_finder,
    )
}

#[cfg(test)]
mod tests {
    use lox_bodies::Earth;
    use lox_core::coords::LonLatAlt;
    use lox_ephem::spk::parser::Spk;
    use lox_test_utils::{assert_approx_eq, data_dir, data_file, read_data_file};
    use lox_time::Time;
    use lox_time::time_scales::Tai;
    use lox_time::utc::Utc;
    use std::iter::zip;
    use std::sync::OnceLock;

    use super::*;

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
        let gs = ground_station_trajectory();
        let sc = spacecraft_trajectory();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let expected: Vec<Radians> = read_data_file("elevation.csv")
            .lines()
            .map(|line| line.parse::<f64>().unwrap().to_radians())
            .collect();
        let actual: Vec<Radians> = gs
            .times()
            .iter()
            .map(|t| elevation(*t, &location(), &mask, &sc, &DefaultRotationProvider))
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
        let gs = location();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let sc = spacecraft_trajectory();
        let times: Vec<Time<Tai>> = sc.states().iter().map(|s| s.time()).collect();
        let expected = contacts();
        let actual =
            visibility(&times, &gs, &mask, &sc, &DefaultRotationProvider).expect("visibility");
        assert_eq!(actual.len(), expected.len());
        assert_approx_eq!(expected, actual, rtol <= 1e-4);
    }

    #[test]
    fn test_visibility_combined() {
        let gs = location_dyn();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let sc = spacecraft_trajectory_dyn();
        let times: Vec<DynTime> = sc.states().iter().map(|s| s.time()).collect();
        let expected = contacts_combined();
        let actual =
            visibility_combined(&times, &gs, &mask, &[DynOrigin::Moon], &sc, ephemeris()).unwrap();
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in zip(actual, expected) {
            assert_approx_eq!(actual.interval().start(), expected.start(), rtol <= 1e-4);
            assert_approx_eq!(actual.interval().end(), expected.end(), rtol <= 1e-4);
        }
    }

    #[test]
    fn test_pass_observables_above_mask() {
        let gs = location_dyn();
        let mask = ElevationMask::with_fixed_elevation(10.0_f64.to_radians());
        let sc = spacecraft_trajectory_dyn();
        let times: Vec<DynTime> = sc.states().iter().map(|s| s.time()).collect();

        let passes = visibility_combined(&times, &gs, &mask, &[], &sc, ephemeris()).unwrap();

        for pass in passes {
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

    fn spacecraft_trajectory() -> Trajectory<Tai, Earth, Icrf> {
        Trajectory::from_csv(&read_data_file("trajectory_lunar.csv"), Earth, Icrf).unwrap()
    }

    fn spacecraft_trajectory_dyn() -> DynTrajectory {
        DynTrajectory::from_csv_dyn(
            &read_data_file("trajectory_lunar.csv"),
            DynOrigin::Earth,
            DynFrame::Icrf,
        )
        .unwrap()
    }

    fn location() -> GroundLocation<Earth> {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        GroundLocation::new(coords, Earth)
    }

    fn location_dyn() -> GroundLocation<DynOrigin> {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        GroundLocation::try_new(coords, DynOrigin::Earth).unwrap()
    }

    fn contacts() -> Vec<TimeInterval<Tai>> {
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

    fn ephemeris() -> &'static Spk {
        static EPHEMERIS: OnceLock<Spk> = OnceLock::new();
        EPHEMERIS.get_or_init(|| Spk::from_file(data_dir().join("spice/de440s.bsp")).unwrap())
    }
}
