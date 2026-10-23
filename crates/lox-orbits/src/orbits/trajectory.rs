// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::num::ParseFloatError;

use lox_bodies::{CoordinateOrigin, Origin};
use lox_core::coords::Cartesian;
use lox_core::glam::DVec3;
use lox_core::trajectory::{HermiteTrajectory, TimeAxisError};
use lox_ephem::Ephemeris;
use lox_frames::{Frame, Icrf, ReferenceFrame, rotations::TryRotation, traits::frame_key};
use lox_time::{Time, deltas::TimeDelta, time_scales::TimeScale, utc::Utc};
use thiserror::Error;

use lox_time::intervals::TimeInterval;

use crate::propagators::Propagator;

use super::{CartesianOrbit, Orbit};

/// A time-ordered sequence of Cartesian orbital states with Hermite cubic interpolation.
///
/// Position is interpolated using Hermite cubic splines that incorporate both
/// position and velocity at each knot point. Velocity is derived as the
/// analytical derivative of the position spline.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trajectory<O: CoordinateOrigin = Origin, R: ReferenceFrame = Frame> {
    epoch: Time,
    origin: O,
    frame: R,
    data: HermiteTrajectory,
}

impl<O, R> Trajectory<O, R>
where
    O: CoordinateOrigin + Copy,
    R: ReferenceFrame + Copy,
{
    /// Constructs a trajectory from Cartesian orbital states.
    ///
    /// The origin and reference frame are taken from the first state.
    ///
    /// # Errors
    ///
    /// Returns an error if fewer than 2 states are provided or the states are
    /// not in strictly increasing time order.
    pub fn try_new(
        states: impl IntoIterator<Item = CartesianOrbit<O, R>>,
    ) -> Result<Self, TrajectoryError> {
        let mut states = states.into_iter().peekable();
        let first = states.peek().ok_or(TimeAxisError::TooFewKnots(0))?;
        let epoch = first.time();
        let origin = first.origin();
        let frame = first.reference_frame();
        let data = HermiteTrajectory::try_new(states.map(|orb| (orb.time() - epoch, orb.state())))?;
        Ok(Self::from_parts(epoch, origin, frame, data))
    }

    /// Constructs a trajectory from its constituent parts.
    ///
    /// `epoch` is the absolute time of the first knot of `data`.
    pub fn from_parts(epoch: Time, origin: O, frame: R, data: HermiteTrajectory) -> Self {
        Self {
            epoch,
            origin,
            frame,
            data,
        }
    }

    /// Decomposes this trajectory into its constituent parts.
    pub fn into_parts(self) -> (Time, O, R, HermiteTrajectory) {
        (self.epoch, self.origin, self.frame, self.data)
    }

    /// Interpolates the trajectory at the given absolute time.
    ///
    /// # Errors
    ///
    /// Returns [`TrajectoryError::OutOfRange`] if `time` lies outside the
    /// trajectory.
    pub fn at(&self, time: Time) -> Result<CartesianOrbit<O, R>, TrajectoryError> {
        self.at_delta(time - self.epoch)
    }

    /// Interpolates the trajectory at the given time delta from the trajectory start.
    ///
    /// # Errors
    ///
    /// Returns [`TrajectoryError::OutOfRange`] if `dt` lies outside the
    /// trajectory.
    pub fn at_delta(&self, dt: TimeDelta) -> Result<CartesianOrbit<O, R>, TrajectoryError> {
        let state = self
            .data
            .at(dt.to_seconds().to_f64())
            .ok_or_else(|| self.out_of_range(self.epoch + dt))?;
        Ok(Orbit::from_state(
            state,
            self.epoch + dt,
            self.origin,
            self.frame,
        ))
    }

    fn out_of_range(&self, time: Time) -> TrajectoryError {
        TrajectoryError::OutOfRange {
            time,
            start: self.start_time(),
            end: self.end_time(),
        }
    }

    /// Transforms the entire trajectory into a different reference frame.
    pub fn into_frame<R1, P>(self, frame: R1, provider: &P) -> Result<Trajectory<O, R1>, P::Error>
    where
        R1: ReferenceFrame + Copy,
        P: TryRotation<R, R1, TimeScale>,
    {
        if frame_key(&self.frame) == frame_key(&frame) {
            return Ok(Trajectory::from_parts(
                self.epoch,
                self.origin,
                frame,
                self.data,
            ));
        }

        let data = self.data.try_map_states(|dt, state| {
            provider
                .try_rotation(self.frame, frame, self.epoch + dt)
                .map(|rot| rot * state)
        })?;

        Ok(Trajectory::from_parts(self.epoch, self.origin, frame, data))
    }

    /// Returns the central body origin.
    pub fn origin(&self) -> O {
        self.origin
    }

    /// Returns the reference frame.
    pub fn reference_frame(&self) -> R {
        self.frame
    }

    /// Returns the time of the first state.
    pub fn start_time(&self) -> Time {
        self.epoch
    }

    /// Returns the time of the last state.
    pub fn end_time(&self) -> Time {
        self.epoch + self.data.axis().span()
    }

    /// Returns the interval from the first to the last state.
    pub fn interval(&self) -> TimeInterval {
        TimeInterval::from_duration(self.epoch, self.data.axis().span())
    }

    /// Returns the times of all states.
    pub fn times(&self) -> Vec<Time> {
        self.data
            .axis()
            .knots()
            .iter()
            .map(|&dt| self.epoch + dt)
            .collect()
    }

    /// Returns all states.
    pub fn states(&self) -> Vec<CartesianOrbit<O, R>> {
        self.data
            .iter()
            .map(|(dt, state)| Orbit::from_state(state, self.epoch + dt, self.origin, self.frame))
            .collect()
    }

    /// Returns the trajectory as `[t, x, y, z, vx, vy, vz]` rows, with `t` in
    /// seconds since the first state.
    pub fn to_vec(&self) -> Vec<Vec<f64>> {
        self.data
            .axis()
            .seconds()
            .iter()
            .zip(self.data.states())
            .map(|(&t, s)| {
                let (p, v) = (s.position(), s.velocity());
                vec![t, p.x, p.y, p.z, v.x, v.y, v.z]
            })
            .collect()
    }
}

impl<O, R> Trajectory<O, R>
where
    O: CoordinateOrigin + Copy + Into<Origin>,
    R: ReferenceFrame + Copy + Into<Frame>,
{
    /// Converts this trajectory into a dynamically-typed trajectory.
    pub fn into_dynamic(self) -> Trajectory {
        Trajectory::from_parts(
            self.epoch.into_dynamic(),
            self.origin.into(),
            self.frame.into(),
            self.data,
        )
    }
}

impl<O, R> Propagator<O> for Trajectory<O, R>
where
    O: CoordinateOrigin + Copy,
    R: ReferenceFrame + Copy,
{
    type Frame = R;
    type Error = TrajectoryError;

    fn state_at(&self, time: Time) -> Result<CartesianOrbit<O, R>, TrajectoryError> {
        self.at(time)
    }

    fn propagate(&self, interval: TimeInterval) -> Result<Trajectory<O, R>, Self::Error> {
        let mut states = Vec::new();
        states.push(self.at(interval.start())?);
        for s in self.states() {
            if s.time() > interval.start() && s.time() < interval.end() {
                states.push(s);
            }
        }
        states.push(self.at(interval.end())?);
        Trajectory::try_new(states)
    }
}

/// Errors that can occur when constructing, evaluating or parsing a trajectory.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum TrajectoryError {
    /// The states do not form a valid time axis.
    #[error(transparent)]
    InvalidTimeAxis(#[from] TimeAxisError),
    /// The requested time lies outside the trajectory.
    #[error("{time} is outside the trajectory from {start} to {end}")]
    OutOfRange {
        /// The requested time.
        time: Time,
        /// The time of the first state.
        start: Time,
        /// The time of the last state.
        end: Time,
    },
    /// An error occurred while parsing CSV data.
    #[error("CSV error: {0}")]
    CsvError(String),
}

impl From<csv::Error> for TrajectoryError {
    fn from(err: csv::Error) -> Self {
        TrajectoryError::CsvError(err.to_string())
    }
}

/// Errors that can occur when transforming a trajectory to a different origin.
#[derive(Debug, Error)]
pub enum TrajectoryTransformationError {
    /// The underlying trajectory construction failed.
    #[error(transparent)]
    TrajectoryError(#[from] TrajectoryError),
    /// A state transformation (e.g. origin change) failed.
    #[error("state transformation failed: {0}")]
    StateTransformationError(String),
}

impl<O> Trajectory<O, Icrf>
where
    O: CoordinateOrigin + Copy,
{
    /// Transforms the entire trajectory to a different central body origin using an ephemeris.
    pub fn to_origin<O1: CoordinateOrigin + Copy, E: Ephemeris>(
        &self,
        target: O1,
        ephemeris: &E,
    ) -> Result<Trajectory<O1, Icrf>, TrajectoryTransformationError> {
        if self.origin().id() == target.id() {
            return Ok(Trajectory::from_parts(
                self.epoch,
                target,
                Icrf,
                self.data.clone(),
            ));
        }
        let states: Result<Vec<_>, _> = self
            .states()
            .into_iter()
            .map(|state| {
                state.to_origin(target, ephemeris).map_err(|e| {
                    TrajectoryTransformationError::StateTransformationError(e.to_string())
                })
            })
            .collect();
        Ok(Trajectory::try_new(states?)?)
    }
}

impl<O, R> Trajectory<O, R>
where
    O: CoordinateOrigin + Copy,
    R: ReferenceFrame + Copy,
{
    /// Parses a trajectory from CSV data with columns `[time, x, y, z, vx, vy, vz]` in km and km/s.
    pub fn from_csv(csv: &str, origin: O, frame: R) -> Result<Self, TrajectoryError> {
        Trajectory::try_new(parse_csv_states(csv, origin, frame)?)
    }
}

impl Trajectory {
    /// Parses a dynamically-typed trajectory from CSV data.
    pub fn from_csv_dynamic(
        csv: &str,
        origin: Origin,
        frame: Frame,
    ) -> Result<Trajectory, TrajectoryError> {
        Ok(Trajectory::from_csv(csv, origin, frame)?.into_dynamic())
    }
}

fn parse_csv_f64(record: &csv::StringRecord, idx: usize) -> Result<f64, TrajectoryError> {
    record
        .get(idx)
        .unwrap()
        .parse()
        .map_err(|e: ParseFloatError| TrajectoryError::CsvError(format!("invalid value: {e}")))
}

fn parse_csv_vec3(
    record: &csv::StringRecord,
    i0: usize,
    i1: usize,
    i2: usize,
) -> Result<DVec3, TrajectoryError> {
    Ok(DVec3::new(
        parse_csv_f64(record, i0)?,
        parse_csv_f64(record, i1)?,
        parse_csv_f64(record, i2)?,
    ))
}

fn parse_csv_states<O: CoordinateOrigin + Copy, R: ReferenceFrame + Copy>(
    csv: &str,
    origin: O,
    frame: R,
) -> Result<Vec<CartesianOrbit<O, R>>, TrajectoryError> {
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    let mut states = Vec::new();
    for result in reader.records() {
        let record = result?;
        if record.len() != 7 {
            return Err(TrajectoryError::CsvError(
                "invalid record length".to_string(),
            ));
        }
        let time = Utc::from_iso(record.get(0).unwrap())
            .map_err(|e| TrajectoryError::CsvError(e.to_string()))?
            .to_dynamic_time();
        // CSV data is in km and km/s, convert to m and m/s
        let position = parse_csv_vec3(&record, 1, 2, 3)? * 1e3;
        let velocity = parse_csv_vec3(&record, 4, 5, 6)? * 1e3;
        let state = Cartesian::from_vecs(position, velocity);
        states.push(CartesianOrbit::new(state, time, origin, frame));
    }
    Ok(states)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_bodies::{Earth, Origin};
    use lox_core::glam::DVec3;
    use lox_frames::{Frame, Icrf};
    use lox_time::time_scales::TimeScale;
    use lox_time::{time, time_scales::Tdb};

    fn sample_trajectory() -> Trajectory<Earth, Icrf> {
        let t0 = time!(Tdb, 2023, 1, 1, 12).unwrap();
        let t1 = t0 + lox_time::deltas::TimeDelta::from_seconds(60);
        let t2 = t0 + lox_time::deltas::TimeDelta::from_seconds(120);
        let s0 = CartesianOrbit::new(
            Cartesian::from_vecs(DVec3::new(7000e3, 0.0, 0.0), DVec3::new(0.0, 7500.0, 0.0)),
            t0.into_dynamic(),
            Earth,
            Icrf,
        );
        let s1 = CartesianOrbit::new(
            Cartesian::from_vecs(
                DVec3::new(6999e3, 100e3, 0.0),
                DVec3::new(-10.0, 7499.0, 0.0),
            ),
            t1.into_dynamic(),
            Earth,
            Icrf,
        );
        let s2 = CartesianOrbit::new(
            Cartesian::from_vecs(
                DVec3::new(6996e3, 200e3, 0.0),
                DVec3::new(-20.0, 7498.0, 0.0),
            ),
            t2.into_dynamic(),
            Earth,
            Icrf,
        );
        Trajectory::try_new(vec![s0, s1, s2]).unwrap()
    }

    #[test]
    fn test_trajectory_into_dynamic() {
        let traj = sample_trajectory();
        let first_pos = traj.states().first().unwrap().position();
        let dynamic_traj = traj.into_dynamic();

        assert_eq!(dynamic_traj.origin(), Origin::Earth);
        assert_eq!(dynamic_traj.reference_frame(), Frame::Icrf);
        assert_eq!(
            dynamic_traj.states().first().unwrap().time().scale(),
            TimeScale::Tdb
        );
        assert_eq!(dynamic_traj.states().first().unwrap().position(), first_pos);
    }

    #[test]
    fn test_trajectory_interval() {
        let traj = sample_trajectory();

        let interval = traj.interval();

        assert_eq!(interval.start(), traj.start_time());
        assert_eq!(interval.end(), traj.end_time());
        assert_eq!(interval.duration(), TimeDelta::from_seconds(120));
    }

    #[test]
    fn test_trajectory_into_parts() {
        let traj = sample_trajectory();
        let epoch_before = traj.start_time();
        let (epoch, origin, frame, _data) = traj.into_parts();

        assert_eq!(origin, Earth);
        assert_eq!(frame, Icrf);
        assert_eq!(epoch, epoch_before);
    }
}
