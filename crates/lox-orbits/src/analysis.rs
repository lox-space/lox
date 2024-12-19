use std::f64::consts::PI;

/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */
use lox_bodies::{Origin, RotationalElements, Spheroid, TrySpheroid};
use lox_math::roots::Brent;
use lox_math::series::{Series, SeriesError};
use lox_math::types::units::Radians;
use lox_time::deltas::TimeDelta;
use lox_time::time_scales::Tdb;
use lox_time::transformations::TryToScale;
use lox_time::TimeLike;
use thiserror::Error;

use crate::events::{find_windows, Window};
use crate::frames::{
    BodyFixed, DynFrame, FrameTransformationProvider, Icrf, TryRotateTo, TryToFrame,
};
use crate::ground::{DynGroundLocation, GroundLocation};
use crate::states::State;
use crate::trajectories::{DynTrajectory, Trajectory};

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

pub fn elevation_dyn<T: TimeLike + TryToScale<Tdb, P> + Clone, P: FrameTransformationProvider>(
    time: T,
    gs: &DynGroundLocation,
    mask: &ElevationMask,
    sc: &DynTrajectory<T>,
    provider: &P,
) -> Radians {
    let body_fixed = DynFrame::BodyFixed(gs.origin());
    let sc = sc.interpolate_at(time.clone());
    let rot = DynFrame::Icrf.try_rotation(&body_fixed, time, provider);
    let (r1, v1) = rot.unwrap().rotate_state(sc.position(), sc.velocity());
    let sc = State::new(sc.time(), r1, v1, sc.origin(), body_fixed);
    let obs = gs.observables_dyn(sc);
    obs.elevation() - mask.min_elevation(obs.azimuth())
}

pub fn visibility_dyn<T: TimeLike + TryToScale<Tdb, P> + Clone, P: FrameTransformationProvider>(
    times: &[T],
    gs: &DynGroundLocation,
    mask: &ElevationMask,
    sc: &DynTrajectory<T>,
    provider: &P,
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
            elevation_dyn(
                start.clone() + TimeDelta::from_decimal_seconds(t).unwrap(),
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

pub fn elevation<
    T: TimeLike + TryToScale<Tdb, P> + Clone,
    O: Origin + TrySpheroid + RotationalElements + Clone,
    P: FrameTransformationProvider,
>(
    time: T,
    gs: &GroundLocation<O>,
    mask: &ElevationMask,
    sc: &Trajectory<T, O, Icrf>,
    provider: &P,
) -> Radians {
    let body_fixed = BodyFixed(gs.origin());
    let sc = sc.interpolate_at(time.clone());
    let sc = sc.try_to_frame(body_fixed, provider).unwrap();
    let obs = gs.observables(sc);
    obs.elevation() - mask.min_elevation(obs.azimuth())
}

pub fn visibility<
    T: TimeLike + TryToScale<Tdb, P> + Clone,
    O: Origin + Spheroid + RotationalElements + Clone,
    P: FrameTransformationProvider,
>(
    times: &[T],
    gs: &GroundLocation<O>,
    mask: &ElevationMask,
    sc: &Trajectory<T, O, Icrf>,
    provider: &P,
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
                start.clone() + TimeDelta::from_decimal_seconds(t).unwrap(),
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
    use lox_math::assert_close;
    use lox_math::is_close::IsClose;
    use lox_time::time_scales::Tai;
    use lox_time::transformations::ToTai;
    use lox_time::utc::Utc;
    use lox_time::Time;
    use std::iter::zip;

    use crate::frames::NoOpFrameTransformationProvider;

    use super::*;

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
            .map(|t| {
                elevation(
                    *t,
                    &location(),
                    &mask,
                    &sc,
                    &NoOpFrameTransformationProvider,
                )
            })
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
        let actual = visibility(&times, &gs, &mask, &sc, &NoOpFrameTransformationProvider);
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in zip(actual, expected) {
            assert_close!(actual.start(), expected.start(), 0.0, 1e-4);
            assert_close!(actual.end(), expected.end(), 0.0, 1e-4);
        }
    }

    fn ground_station_trajectory() -> Trajectory<Time<Tai>, Earth, Icrf> {
        Trajectory::from_csv(
            include_str!("../../../data/trajectory_cebr.csv"),
            Earth,
            Icrf,
        )
        .unwrap()
    }

    fn spacecraft_trajectory() -> Trajectory<Time<Tai>, Earth, Icrf> {
        Trajectory::from_csv(
            include_str!("../../../data/trajectory_lunar.csv"),
            Earth,
            Icrf,
        )
        .unwrap()
    }

    fn location() -> GroundLocation<Earth> {
        let longitude = -4.3676f64.to_radians();
        let latitude = 40.4527f64.to_radians();
        GroundLocation::new(longitude, latitude, 0.0, Earth)
    }

    fn contacts() -> Vec<Window<Time<Tai>>> {
        let mut windows = vec![];
        let mut reader =
            csv::Reader::from_reader(include_str!("../../../data/contacts.csv").as_bytes());
        for result in reader.records() {
            let record = result.unwrap();
            let start = record[0].parse::<Utc>().unwrap().to_tai();
            let end = record[1].parse::<Utc>().unwrap().to_tai();
            windows.push(Window::new(start, end));
        }
        windows
    }
}
