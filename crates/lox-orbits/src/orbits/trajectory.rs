// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::num::ParseFloatError;

use glam::DVec3;
use lox_bodies::{DynOrigin, Origin};
use lox_core::coords::{Cartesian, CartesianTrajectory, TimeStampedCartesian};
use lox_ephem::Ephemeris;
use lox_frames::{DynFrame, Icrf, ReferenceFrame, rotations::TryRotation, traits::frame_id};
use lox_time::{
    Time,
    deltas::TimeDelta,
    offsets::{DefaultOffsetProvider, Offset},
    time_scales::{DynTimeScale, Tai, Tdb, TimeScale},
    utc::Utc,
};
use thiserror::Error;

use lox_time::intervals::TimeInterval;

use crate::propagators::Propagator;

use super::{CartesianOrbit, Orbit, TrajectorError};

#[derive(Debug, Clone)]
pub struct Trajectory<T: TimeScale, O: Origin, R: ReferenceFrame> {
    epoch: Time<T>,
    origin: O,
    frame: R,
    data: CartesianTrajectory,
}

impl<T, O, R> Trajectory<T, O, R>
where
    T: TimeScale + Copy,
    O: Origin + Copy,
    R: ReferenceFrame + Copy,
{
    pub fn new(states: impl IntoIterator<Item = CartesianOrbit<T, O, R>>) -> Self {
        let mut states = states.into_iter().peekable();
        let first = states.peek().unwrap();
        let epoch = first.time();
        let origin = first.origin();
        let frame = first.reference_frame();
        let data = states
            .map(|orb| {
                let time = orb.time() - epoch;
                TimeStampedCartesian {
                    time,
                    state: orb.state(),
                }
            })
            .collect();
        Self::from_parts(epoch, origin, frame, data)
    }

    pub fn from_parts(epoch: Time<T>, origin: O, frame: R, data: CartesianTrajectory) -> Self {
        Self {
            epoch,
            origin,
            frame,
            data,
        }
    }

    pub fn into_parts(self) -> (Time<T>, O, R, CartesianTrajectory) {
        (self.epoch, self.origin, self.frame, self.data)
    }

    pub fn try_new(
        states: impl IntoIterator<Item = CartesianOrbit<T, O, R>>,
    ) -> Result<Self, TrajectorError> {
        let mut states = states.into_iter().peekable();
        for i in 0..2 {
            states.peek().ok_or(TrajectorError::InsufficientStates(i))?;
        }
        Ok(Self::new(states))
    }

    pub fn at(&self, time: Time<T>) -> CartesianOrbit<T, O, R> {
        let t = (time - self.epoch).to_seconds().to_f64();
        let state = self.data.at(t);
        Orbit::from_state(state, time, self.origin, self.frame)
    }

    pub fn into_frame<R1, P>(
        self,
        frame: R1,
        provider: &P,
    ) -> Result<Trajectory<T, O, R1>, Box<dyn std::error::Error>>
    where
        R1: ReferenceFrame + Copy,
        P: TryRotation<R, R1, T>,
    {
        if frame_id(&self.frame) == frame_id(&frame) {
            return Ok(Trajectory::from_parts(
                self.epoch,
                self.origin,
                frame,
                self.data,
            ));
        }

        let data: Result<CartesianTrajectory, P::Error> = self
            .data
            .into_iter()
            .map(|TimeStampedCartesian { time, state }| {
                let t = self.epoch + time;
                provider
                    .try_rotation(self.frame, frame, t)
                    .map(|rot| TimeStampedCartesian {
                        time,
                        state: rot * state,
                    })
            })
            .collect();

        Ok(Trajectory::from_parts(
            self.epoch,
            self.origin,
            frame,
            data?,
        ))
    }

    pub fn epoch(&self) -> Time<T> {
        self.epoch
    }

    pub fn origin(&self) -> O {
        self.origin
    }

    pub fn reference_frame(&self) -> R {
        self.frame
    }

    pub fn start_time(&self) -> Time<T> {
        self.epoch
    }

    pub fn end_time(&self) -> Time<T> {
        let time_steps = self.data.time_steps();
        let last = time_steps.last().copied().unwrap_or(0.0);
        self.epoch + TimeDelta::from_seconds_f64(last)
    }

    pub fn times(&self) -> Vec<Time<T>> {
        let time_steps = self.data.time_steps();
        time_steps
            .iter()
            .map(|&t| self.epoch + TimeDelta::from_seconds_f64(t))
            .collect()
    }

    pub fn states(&self) -> Vec<CartesianOrbit<T, O, R>> {
        let time_steps = self.data.time_steps();
        time_steps
            .iter()
            .map(|&t| {
                let state = self.data.at(t);
                let time = self.epoch + TimeDelta::from_seconds_f64(t);
                Orbit::from_state(state, time, self.origin, self.frame)
            })
            .collect()
    }

    pub fn to_vec(&self) -> Vec<Vec<f64>> {
        let time_steps = self.data.time_steps();
        time_steps
            .iter()
            .map(|&t| {
                let state = self.data.at(t);
                vec![
                    t,
                    state.position().x,
                    state.position().y,
                    state.position().z,
                    state.velocity().x,
                    state.velocity().y,
                    state.velocity().z,
                ]
            })
            .collect()
    }

    pub fn interpolate(&self, dt: TimeDelta) -> CartesianOrbit<T, O, R> {
        let t = dt.to_seconds().to_f64();
        let state = self.data.at(t);
        Orbit::from_state(state, self.epoch + dt, self.origin, self.frame)
    }

    pub fn interpolate_at(&self, time: Time<T>) -> CartesianOrbit<T, O, R> {
        self.interpolate(time - self.epoch)
    }

    pub fn position(&self, t: f64) -> DVec3 {
        self.data.position(t)
    }

    pub fn velocity(&self, t: f64) -> DVec3 {
        self.data.velocity(t)
    }

    /// Find zero-crossing events of `func` evaluated along this trajectory.
    ///
    /// The closure receives the interpolated [`CartesianOrbit`] at each sample
    /// time and must return a scalar whose sign changes define events.
    pub fn find_events<F>(
        &self,
        func: F,
        step: TimeDelta,
    ) -> Result<Vec<crate::events::Event<T>>, crate::events::DetectError>
    where
        F: Fn(CartesianOrbit<T, O, R>) -> f64,
    {
        let interval = TimeInterval::new(self.start_time(), self.end_time());
        crate::events::find_events(|t| func(self.interpolate_at(t)), interval, step)
    }

    /// Find zero-crossing events of a fallible `func` evaluated along this
    /// trajectory.
    pub fn try_find_events<F, E>(
        &self,
        func: F,
        step: TimeDelta,
    ) -> Result<Vec<crate::events::Event<T>>, crate::events::DetectError>
    where
        F: Fn(CartesianOrbit<T, O, R>) -> Result<f64, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let interval = TimeInterval::new(self.start_time(), self.end_time());
        crate::events::try_find_events(|t| func(self.interpolate_at(t)), interval, step)
    }

    /// Find time intervals where `func` is positive, evaluated along this
    /// trajectory.
    pub fn find_windows<F>(
        &self,
        func: F,
        step: TimeDelta,
    ) -> Result<Vec<TimeInterval<T>>, crate::events::DetectError>
    where
        F: Fn(CartesianOrbit<T, O, R>) -> f64,
    {
        let interval = TimeInterval::new(self.start_time(), self.end_time());
        crate::events::find_windows(|t| func(self.interpolate_at(t)), interval, step)
    }

    /// Find time intervals where a fallible `func` is positive, evaluated
    /// along this trajectory.
    pub fn try_find_windows<F, E>(
        &self,
        func: F,
        step: TimeDelta,
    ) -> Result<Vec<TimeInterval<T>>, crate::events::DetectError>
    where
        F: Fn(CartesianOrbit<T, O, R>) -> Result<f64, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let interval = TimeInterval::new(self.start_time(), self.end_time());
        crate::events::try_find_windows(|t| func(self.interpolate_at(t)), interval, step)
    }
}

impl<T, O, R> Trajectory<T, O, R>
where
    T: TimeScale + Copy + Into<DynTimeScale>,
    O: Origin + Copy + Into<DynOrigin>,
    R: ReferenceFrame + Copy + Into<DynFrame>,
{
    pub fn into_dyn(self) -> DynTrajectory {
        Trajectory::from_parts(
            self.epoch.into_dyn(),
            self.origin.into(),
            self.frame.into(),
            self.data,
        )
    }
}

impl<T, O, R> Propagator<T, O> for Trajectory<T, O, R>
where
    T: TimeScale + Copy + PartialOrd,
    O: Origin + Copy,
    R: ReferenceFrame + Copy,
{
    type Frame = R;
    type Error = TrajectorError;

    fn state_at(&self, time: Time<T>) -> Result<CartesianOrbit<T, O, R>, TrajectorError> {
        Ok(self.at(time))
    }

    fn propagate(&self, interval: TimeInterval<T>) -> Result<Trajectory<T, O, R>, Self::Error> {
        let mut states = Vec::new();
        states.push(self.at(interval.start()));
        for s in self.states() {
            if s.time() > interval.start() && s.time() < interval.end() {
                states.push(s);
            }
        }
        states.push(self.at(interval.end()));
        Trajectory::try_new(states)
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum TrajectoryError {
    #[error("`states` must have at least 2 elements but had {0}")]
    InsufficientStates(usize),
    #[error("CSV error: {0}")]
    CsvError(String),
}

impl From<csv::Error> for TrajectoryError {
    fn from(err: csv::Error) -> Self {
        TrajectoryError::CsvError(err.to_string())
    }
}

#[derive(Debug, Error)]
pub enum TrajectoryTransformationError {
    #[error(transparent)]
    TrajectoryError(#[from] TrajectorError),
    #[error("state transformation failed: {0}")]
    StateTransformationError(String),
}

impl<T, O> Trajectory<T, O, Icrf>
where
    T: TimeScale + Copy,
    O: Origin + Copy,
    DefaultOffsetProvider: Offset<T, Tdb>,
{
    pub fn to_origin<O1: Origin + Copy, E: Ephemeris>(
        &self,
        target: O1,
        ephemeris: &E,
    ) -> Result<Trajectory<T, O1, Icrf>, TrajectoryTransformationError> {
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
        Ok(Trajectory::new(states?))
    }
}

impl<O, R> Trajectory<Tai, O, R>
where
    O: Origin + Copy,
    R: ReferenceFrame + Copy,
{
    pub fn from_csv(csv: &str, origin: O, frame: R) -> Result<Self, TrajectoryError> {
        let states = parse_csv_states(csv, origin, frame)?;
        if states.len() < 2 {
            return Err(TrajectoryError::InsufficientStates(states.len()));
        }
        Ok(Trajectory::new(states))
    }
}

pub type DynTrajectory = Trajectory<DynTimeScale, DynOrigin, DynFrame>;

impl DynTrajectory {
    pub fn from_csv_dyn(
        csv: &str,
        origin: DynOrigin,
        frame: DynFrame,
    ) -> Result<DynTrajectory, TrajectoryError> {
        Ok(Trajectory::from_csv(csv, origin, frame)?.into_dyn())
    }
}

impl<T, O, R> FromIterator<CartesianOrbit<T, O, R>> for Trajectory<T, O, R>
where
    T: TimeScale + Copy,
    O: Origin + Copy,
    R: ReferenceFrame + Copy,
{
    fn from_iter<U: IntoIterator<Item = CartesianOrbit<T, O, R>>>(iter: U) -> Self {
        Self::new(iter)
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

fn parse_csv_states<O: Origin + Copy, R: ReferenceFrame + Copy>(
    csv: &str,
    origin: O,
    frame: R,
) -> Result<Vec<CartesianOrbit<Tai, O, R>>, TrajectoryError> {
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    let mut states = Vec::new();
    for result in reader.records() {
        let record = result?;
        if record.len() != 7 {
            return Err(TrajectoryError::CsvError(
                "invalid record length".to_string(),
            ));
        }
        let time: Time<Tai> = Utc::from_iso(record.get(0).unwrap())
            .map_err(|e| TrajectoryError::CsvError(e.to_string()))?
            .to_time();
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
    use glam::DVec3;
    use lox_bodies::{DynOrigin, Earth};
    use lox_frames::{DynFrame, Icrf};
    use lox_time::time_scales::DynTimeScale;
    use lox_time::{time, time_scales::Tdb};

    fn sample_trajectory() -> Trajectory<Tdb, Earth, Icrf> {
        let t0 = time!(Tdb, 2023, 1, 1, 12).unwrap();
        let t1 = t0 + lox_time::deltas::TimeDelta::from_seconds(60);
        let t2 = t0 + lox_time::deltas::TimeDelta::from_seconds(120);
        let s0 = CartesianOrbit::new(
            Cartesian::from_vecs(DVec3::new(7000e3, 0.0, 0.0), DVec3::new(0.0, 7500.0, 0.0)),
            t0,
            Earth,
            Icrf,
        );
        let s1 = CartesianOrbit::new(
            Cartesian::from_vecs(
                DVec3::new(6999e3, 100e3, 0.0),
                DVec3::new(-10.0, 7499.0, 0.0),
            ),
            t1,
            Earth,
            Icrf,
        );
        let s2 = CartesianOrbit::new(
            Cartesian::from_vecs(
                DVec3::new(6996e3, 200e3, 0.0),
                DVec3::new(-20.0, 7498.0, 0.0),
            ),
            t2,
            Earth,
            Icrf,
        );
        Trajectory::new(vec![s0, s1, s2])
    }

    #[test]
    fn test_trajectory_into_dyn() {
        let traj = sample_trajectory();
        let first_pos = traj.states().first().unwrap().position();
        let dyn_traj = traj.into_dyn();

        assert_eq!(dyn_traj.origin(), DynOrigin::Earth);
        assert_eq!(dyn_traj.reference_frame(), DynFrame::Icrf);
        assert_eq!(
            dyn_traj.states().first().unwrap().time().scale(),
            DynTimeScale::Tdb
        );
        assert_eq!(dyn_traj.states().first().unwrap().position(), first_pos);
    }

    #[test]
    fn test_trajectory_into_parts() {
        let traj = sample_trajectory();
        let epoch_before = traj.epoch();
        let (epoch, origin, frame, _data) = traj.into_parts();

        assert_eq!(origin, Earth);
        assert_eq!(frame, Icrf);
        assert_eq!(epoch, epoch_before);
    }
}
