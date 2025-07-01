/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use glam::DVec3;
use itertools::Itertools;
use lox_bodies::{
    DynOrigin, Origin, RotationalElements, Spheroid, TryMeanRadius, TrySpheroid,
    UndefinedOriginPropertyError,
};
use lox_ephem::{Ephemeris, path_from_ids};
use lox_math::roots::Brent;
use lox_math::series::{Series, SeriesError};
use lox_math::types::units::Radians;
use lox_time::deltas::TimeDelta;
use lox_time::julian_dates::JulianDate;
use lox_time::time_scales::{DynTimeScale, TryToScale};
use lox_time::time_scales::{Tdb, TimeScale};
use lox_time::ut1::DeltaUt1TaiProvider;
use lox_time::{DynTime, Time};
use rayon::prelude::*;
use std::f64::consts::PI;
use thiserror::Error;

use crate::events::{Window, find_windows, intersect_windows};
use crate::frames::{DynFrame, Iau, Icrf, TryRotateTo};
use crate::ground::{DynGroundLocation, DynGroundPropagator, GroundLocation, Observables};
use crate::states::{DynState, State};
use crate::trajectories::{DynTrajectory, Trajectory};

// Salvatore Alfano, David Negron, Jr., and Jennifer L. Moore
// Rapid Determination of Satellite Visibility Periods
// The Journal of the Astronautical Sciences. Vol. 40, No. 2, April-June 1992, pp. 281-296
pub fn line_of_sight(radius: f64, r1: DVec3, r2: DVec3) -> f64 {
    let r1n = r1.length();
    let r2n = r2.length();
    let theta1 = radius / r1n;
    let theta2 = radius / r2n;
    let theta = r1.dot(r2) / r1n / r2n;
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
        let mean_radius = self.try_mean_radius()?;
        if let (Ok(r_eq), Ok(r_p)) = (self.try_equatorial_radius(), self.try_polar_radius()) {
            return Ok(line_of_sight_spheroid(mean_radius, r_eq, r_p, r1, r2));
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
pub enum ElevationMask {
    Fixed(f64),
    Variable(Series<Vec<f64>, Vec<f64>>),
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
        Ok(Self::Variable(Series::new(azimuth, elevation)?))
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

#[derive(Debug, Clone)]
pub struct Pass<T: TimeScale> {
    window: Window<T>,
    times: Vec<Time<T>>,
    observables: Vec<Observables>,
    // Series for interpolation
    azimuth_series: Series<Vec<f64>, Vec<f64>>,
    elevation_series: Series<Vec<f64>, Vec<f64>>,
    range_series: Series<Vec<f64>, Vec<f64>>,
    range_rate_series: Series<Vec<f64>, Vec<f64>>,
}

pub type DynPass = Pass<DynTimeScale>;

impl DynPass {
    /// Create a Pass from a window, calculating observables for times when the satellite is above the elevation mask
    pub fn from_window<P: DeltaUt1TaiProvider>(
        window: Window<DynTimeScale>,
        time_resolution: TimeDelta,
        gs: &DynGroundLocation,
        mask: &ElevationMask,
        sc: &DynTrajectory,
        provider: Option<&P>,
    ) -> Result<DynPass, SeriesError> {
        let mut pass_times = Vec::new();
        let mut pass_observables = Vec::new();

        // Generate times at the specified resolution within the window
        let mut current_time = window.start();

        while current_time <= window.end() {
            let state = sc.interpolate_at(current_time);

            // Transform to body-fixed frame like elevation_dyn does
            let body_fixed = DynFrame::Iau(gs.origin());
            let rot = DynFrame::Icrf.try_rotation(body_fixed, current_time, provider);
            let (r1, v1) = rot
                .unwrap()
                .rotate_state(state.position(), state.velocity());
            let state_bf = DynState::new(state.time(), r1, v1, state.origin(), body_fixed);

            let obs = gs.observables_dyn(state_bf);

            // Check if satellite is above the elevation mask
            let min_elev = mask.min_elevation(obs.azimuth());
            if obs.elevation() >= min_elev {
                pass_times.push(current_time);
                pass_observables.push(obs);
            }

            current_time = current_time + time_resolution;
        }

        // Only create a pass if we have valid observations
        if pass_times.is_empty() {
            return Err(SeriesError::InsufficientPoints(0, 2));
        }

        Pass::new(window, pass_times, pass_observables)
    }
}

impl<T: TimeScale> Pass<T> {
    pub fn new(
        window: Window<T>,
        times: Vec<Time<T>>,
        observables: Vec<Observables>,
    ) -> Result<Self, SeriesError>
    where
        T: Clone,
    {
        // If we have less than 2 points, we can't create a proper Series
        // We'll create dummy series with duplicate points for Series compatibility
        let (time_seconds, azimuths, elevations, ranges, range_rates) = if times.len() < 2 {
            if times.is_empty() {
                // Create default points at window start and end
                let start_seconds = 0.0;
                let end_seconds = window.duration().to_decimal_seconds();
                let default_obs = observables
                    .first()
                    .cloned()
                    .unwrap_or_else(|| Observables::new(0.0, 0.0, 0.0, 0.0));
                (
                    vec![start_seconds, end_seconds],
                    vec![default_obs.azimuth(), default_obs.azimuth()],
                    vec![default_obs.elevation(), default_obs.elevation()],
                    vec![default_obs.range(), default_obs.range()],
                    vec![default_obs.range_rate(), default_obs.range_rate()],
                )
            } else {
                // Duplicate the single point
                let time_sec = (times[0].clone() - window.start()).to_decimal_seconds();
                let obs = &observables[0];
                (
                    vec![time_sec, time_sec + 1.0], // Add 1 second offset for second point
                    vec![obs.azimuth(), obs.azimuth()],
                    vec![obs.elevation(), obs.elevation()],
                    vec![obs.range(), obs.range()],
                    vec![obs.range_rate(), obs.range_rate()],
                )
            }
        } else {
            // Normal case with multiple points
            let time_seconds: Vec<f64> = times
                .iter()
                .map(|t| (t.clone() - window.start()).to_decimal_seconds())
                .collect();

            // Extract observable arrays
            let azimuths: Vec<f64> = observables.iter().map(|o| o.azimuth()).collect();
            let elevations: Vec<f64> = observables.iter().map(|o| o.elevation()).collect();
            let ranges: Vec<f64> = observables.iter().map(|o| o.range()).collect();
            let range_rates: Vec<f64> = observables.iter().map(|o| o.range_rate()).collect();

            (time_seconds, azimuths, elevations, ranges, range_rates)
        };

        // Create series for each observable
        let azimuth_series = Series::new(time_seconds.clone(), azimuths)?;
        let elevation_series = Series::new(time_seconds.clone(), elevations)?;
        let range_series = Series::new(time_seconds.clone(), ranges)?;
        let range_rate_series = Series::new(time_seconds, range_rates)?;

        Ok(Pass {
            window,
            times,
            observables,
            azimuth_series,
            elevation_series,
            range_series,
            range_rate_series,
        })
    }

    pub fn window(&self) -> &Window<T> {
        &self.window
    }

    pub fn times(&self) -> &[Time<T>] {
        &self.times
    }

    pub fn observables(&self) -> &[Observables] {
        &self.observables
    }

    pub fn interpolate(&self, time: Time<T>) -> Option<Observables>
    where
        T: Clone + PartialOrd,
    {
        // Check if the time is within the window
        if time < self.window.start() || time > self.window.end() {
            return None;
        }

        // If we have no data points, return None
        if self.times.is_empty() {
            return None;
        }

        // Convert time to seconds since window start for interpolation
        let target_seconds = (time - self.window.start()).to_decimal_seconds();

        // Use Series interpolation for each observable
        let azimuth = self.azimuth_series.interpolate(target_seconds);
        let elevation = self.elevation_series.interpolate(target_seconds);
        let range = self.range_series.interpolate(target_seconds);
        let range_rate = self.range_rate_series.interpolate(target_seconds);

        Some(Observables::new(azimuth, elevation, range, range_rate))
    }
}

pub fn elevation_dyn<P: DeltaUt1TaiProvider>(
    time: DynTime,
    gs: &DynGroundLocation,
    mask: &ElevationMask,
    sc: &DynTrajectory,
    provider: Option<&P>,
) -> Radians {
    let body_fixed = DynFrame::Iau(gs.origin());
    let sc = sc.interpolate_at(time);
    let rot = DynFrame::Icrf.try_rotation(body_fixed, time, provider);
    let (r1, v1) = rot.unwrap().rotate_state(sc.position(), sc.velocity());
    let sc = State::new(sc.time(), r1, v1, sc.origin(), body_fixed);
    let obs = gs.observables_dyn(sc);
    obs.elevation() - mask.min_elevation(obs.azimuth())
}

pub fn visibility_dyn<P: DeltaUt1TaiProvider>(
    times: &[DynTime],
    gs: &DynGroundLocation,
    mask: &ElevationMask,
    sc: &DynTrajectory,
    provider: Option<&P>,
) -> Vec<Window<DynTimeScale>> {
    if times.len() < 2 {
        return vec![];
    }
    let start = *times.first().unwrap();
    let end = *times.last().unwrap();
    let times: Vec<f64> = times
        .iter()
        .map(|t| (*t - start).to_decimal_seconds())
        .collect();
    let root_finder = Brent::default();
    find_windows(
        |t| {
            elevation_dyn(
                start + TimeDelta::try_from_decimal_seconds(t).unwrap(),
                gs,
                mask,
                sc,
                provider,
            )
        },
        start,
        end,
        &times,
        root_finder,
    )
}

pub fn visibility_los<P: DeltaUt1TaiProvider + Clone>(
    times: &[DynTime],
    gs: &DynGroundLocation,
    body: DynOrigin,
    sc: &DynTrajectory,
    ephem: &impl Ephemeris,
    provider: Option<&P>,
) -> Vec<Window<DynTimeScale>> {
    if times.len() < 2 {
        return vec![];
    }
    let start = *times.first().unwrap();
    let end = *times.last().unwrap();
    let times: Vec<f64> = times
        .iter()
        .map(|t| (*t - start).to_decimal_seconds())
        .collect();
    let root_finder = Brent::default();
    find_windows(
        |t| {
            let time = start + TimeDelta::from_decimal_seconds(t);
            let epoch = time
                .try_to_scale(Tdb, provider)
                .unwrap()
                .seconds_since_j2000();
            let origin_id = sc.origin().id();
            let target_id = body.id();
            let path = path_from_ids(origin_id.0, target_id.0);
            let mut r_body = DVec3::ZERO;
            for (origin, target) in path.into_iter().tuple_windows() {
                let p: DVec3 = ephem.position(epoch, origin, target).unwrap().into();
                r_body += p;
            }
            let r_sc = sc.interpolate_at(time).position() - r_body;
            let r_gs = DynGroundPropagator::with_dynamic(gs.clone(), provider.cloned())
                .propagate_dyn(time)
                .unwrap()
                .position()
                - r_body;
            body.line_of_sight(r_gs, r_sc).unwrap()
        },
        start,
        end,
        &times,
        root_finder,
    )
}

pub fn visibility_combined<
    P: DeltaUt1TaiProvider + Clone + Send + Sync,
    E: Ephemeris + Send + Sync,
>(
    times: &[DynTime],
    gs: &DynGroundLocation,
    mask: &ElevationMask,
    bodies: &[DynOrigin],
    sc: &DynTrajectory,
    ephem: &E,
    provider: Option<&P>,
) -> Result<Vec<DynPass>, SeriesError> {
    let w1 = visibility_dyn(times, gs, mask, sc, provider);
    let wb: Vec<Vec<Window<DynTimeScale>>> = bodies
        .par_iter()
        .map(|&body| visibility_los(times, gs, body, sc, ephem, provider))
        .collect();
    let mut windows = w1;
    for w2 in wb {
        windows = intersect_windows(&windows, &w2);
    }

    // Convert windows to passes
    let mut passes = Vec::new();

    // Calculate time resolution from the provided times array
    let time_resolution = if times.len() >= 2 {
        times[1] - times[0]
    } else {
        // Default to 60 seconds if we don't have enough times
        TimeDelta::try_from_decimal_seconds(60.0).expect("60 seconds should be valid")
    };

    for window in windows {
        // Use the new from_window constructor to properly calculate observables
        match DynPass::from_window(window, time_resolution, gs, mask, sc, provider) {
            Ok(pass) => passes.push(pass),
            Err(SeriesError::InsufficientPoints(_, _)) => {
                // Skip windows where the satellite never rises above the elevation mask
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(passes)
}

pub fn elevation<
    T: TimeScale + TryToScale<Tdb, P> + Clone,
    O: Origin + TrySpheroid + RotationalElements + Clone,
    P,
>(
    time: Time<T>,
    gs: &GroundLocation<O>,
    mask: &ElevationMask,
    sc: &Trajectory<T, O, Icrf>,
    provider: Option<&P>,
) -> Radians {
    let body_fixed = Iau(gs.origin());
    let sc = sc.interpolate_at(time.clone());
    let sc = sc.try_to_frame(body_fixed, provider).unwrap();
    let obs = gs.observables(sc);
    obs.elevation() - mask.min_elevation(obs.azimuth())
}

pub fn visibility<
    T: TimeScale + TryToScale<Tdb, P> + Clone,
    O: Origin + Spheroid + RotationalElements + Clone,
    P,
>(
    times: &[Time<T>],
    gs: &GroundLocation<O>,
    mask: &ElevationMask,
    sc: &Trajectory<T, O, Icrf>,
    provider: Option<&P>,
) -> Vec<Window<T>> {
    if times.len() < 2 {
        return vec![];
    }
    let start = times.first().unwrap().clone();
    let end = times.last().unwrap().clone();
    let times: Vec<f64> = times
        .iter()
        .map(|t| (t.clone() - start.clone()).to_decimal_seconds())
        .collect();
    let root_finder = Brent::default();
    find_windows(
        |t| {
            elevation(
                start.clone() + TimeDelta::try_from_decimal_seconds(t).unwrap(),
                gs,
                mask,
                sc,
                provider,
            )
        },
        start.clone(),
        end.clone(),
        &times,
        root_finder,
    )
}

#[cfg(test)]
mod tests {
    use lox_bodies::Earth;
    use lox_ephem::spk::parser::{Spk, parse_daf_spk};
    use lox_math::assert_close;
    use lox_math::is_close::IsClose;
    use lox_time::Time;
    use lox_time::time_scales::Tai;
    use lox_time::ut1::DeltaUt1Tai;
    use lox_time::utc::Utc;
    use std::iter::zip;
    use std::path::PathBuf;
    use std::sync::OnceLock;

    use super::*;

    #[test]
    fn test_line_of_sight() {
        let r1 = DVec3::new(0.0, -4464.696, -5102.509);
        let r2 = DVec3::new(0.0, 5740.323, 3189.068);
        let r_sun = DVec3::new(122233179.0, -76150708.0, 33016374.0);
        let r = Earth.equatorial_radius();

        let los = line_of_sight(r, r1, r2);
        let los_sun = line_of_sight(r, r1, r_sun);

        assert!(los < 0.0);
        assert!(los_sun >= 0.0);
    }

    #[test]
    fn test_line_of_sight_trait() {
        let r1 = DVec3::new(0.0, -4464.696, -5102.509);
        let r2 = DVec3::new(0.0, 5740.323, 3189.068);
        let r_sun = DVec3::new(122233179.0, -76150708.0, 33016374.0);

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
        let expected: Vec<Radians> = include_str!("../../../data/elevation.csv")
            .lines()
            .map(|line| line.parse::<f64>().unwrap().to_radians())
            .collect();
        let actual: Vec<Radians> = gs
            .times()
            .iter()
            .map(|t| elevation(*t, &location(), &mask, &sc, None::<&()>))
            .collect();
        for (actual, expected) in actual.iter().zip(expected.iter()) {
            assert_close!(actual, expected, 1e-1);
        }
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
        let actual = visibility(&times, &gs, &mask, &sc, None::<&()>);
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in zip(actual, expected) {
            assert_close!(actual.start(), expected.start(), 0.0, 1e-4);
            assert_close!(actual.end(), expected.end(), 0.0, 1e-4);
        }
    }

    #[test]
    fn test_visibility_combined() {
        let gs = location_dyn();
        let mask = ElevationMask::with_fixed_elevation(0.0);
        let sc = spacecraft_trajectory_dyn();
        let times: Vec<DynTime> = sc.states().iter().map(|s| s.time()).collect();
        let expected = contacts_combined();
        let actual = visibility_combined(
            &times,
            &gs,
            &mask,
            &[DynOrigin::Moon],
            &sc,
            ephemeris(),
            None::<&DeltaUt1Tai>,
        )
        .unwrap();
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in zip(actual, expected) {
            assert_close!(actual.window().start(), expected.start(), 0.0, 1e-4);
            assert_close!(actual.window().end(), expected.end(), 0.0, 1e-4);
        }
    }

    #[test]
    fn test_pass_observables_above_mask() {
        // Test that all observables in a Pass are above the elevation mask
        let gs = location_dyn();
        let mask = ElevationMask::with_fixed_elevation(10.0_f64.to_radians()); // 10 degree mask
        let sc = spacecraft_trajectory_dyn();
        let times: Vec<DynTime> = sc.states().iter().map(|s| s.time()).collect();

        let passes = visibility_combined(
            &times,
            &gs,
            &mask,
            &[], // No occluding bodies for this test
            &sc,
            ephemeris(),
            None::<&DeltaUt1Tai>,
        )
        .unwrap();

        // For each pass, verify all observables have elevation >= mask minimum
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
        Trajectory::from_csv(
            include_str!("../../../data/trajectory_cebr.csv"),
            Earth,
            Icrf,
        )
        .unwrap()
    }

    fn spacecraft_trajectory() -> Trajectory<Tai, Earth, Icrf> {
        Trajectory::from_csv(
            include_str!("../../../data/trajectory_lunar.csv"),
            Earth,
            Icrf,
        )
        .unwrap()
    }

    fn spacecraft_trajectory_dyn() -> DynTrajectory {
        Trajectory::from_csv_dyn(
            include_str!("../../../data/trajectory_lunar.csv"),
            DynOrigin::Earth,
            DynFrame::Icrf,
        )
        .unwrap()
    }

    fn location() -> GroundLocation<Earth> {
        let longitude = -4.3676f64.to_radians();
        let latitude = 40.4527f64.to_radians();
        GroundLocation::new(longitude, latitude, 0.0, Earth)
    }

    fn location_dyn() -> GroundLocation<DynOrigin> {
        let longitude = -4.3676f64.to_radians();
        let latitude = 40.4527f64.to_radians();
        GroundLocation::with_dynamic(longitude, latitude, 0.0, DynOrigin::Earth).unwrap()
    }

    fn contacts() -> Vec<Window<Tai>> {
        let mut windows = vec![];
        let mut reader =
            csv::Reader::from_reader(include_str!("../../../data/contacts.csv").as_bytes());
        for result in reader.records() {
            let record = result.unwrap();
            let start = record[0].parse::<Utc>().unwrap().to_time();
            let end = record[1].parse::<Utc>().unwrap().to_time();
            windows.push(Window::new(start, end));
        }
        windows
    }

    fn contacts_combined() -> Vec<Window<DynTimeScale>> {
        let mut windows = vec![];
        let mut reader = csv::Reader::from_reader(
            include_str!("../../../data/contacts_combined.csv").as_bytes(),
        );
        for result in reader.records() {
            let record = result.unwrap();
            let start = record[0].parse::<Utc>().unwrap().to_dyn_time();
            let end = record[1].parse::<Utc>().unwrap().to_dyn_time();
            windows.push(Window::new(start, end));
        }
        windows
    }

    fn ephemeris() -> &'static Spk {
        let contents = std::fs::read(data_dir().join("de440s.bsp")).unwrap();
        static EPHEMERIS: OnceLock<Spk> = OnceLock::new();
        EPHEMERIS.get_or_init(|| parse_daf_spk(&contents).unwrap())
    }

    pub fn data_dir() -> PathBuf {
        PathBuf::from(format!("{}/../../data", env!("CARGO_MANIFEST_DIR")))
    }
}
