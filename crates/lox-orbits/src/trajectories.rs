/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use thiserror::Error;

use lox_bodies::PointMass;
use lox_time::time_scales::TimeScale;
use lox_time::Time;
use lox_time::{deltas::TimeDelta, TimeSystem};
use lox_utils::interpolation::cubic_spline::LoxCubicSplineError;

use crate::base::BaseState;
use crate::frames::ReferenceFrame;
use crate::two_body::Cartesian;
use crate::CoordinateSystem;

use self::base::{BaseCubicSplineTrajectory, BaseTrajectory};

pub mod base;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum LoxTrajectoryError {
    #[error("time delta must be between 0.0 seconds and {0} seconds but was {1} seconds")]
    DeltaOutOfRange(String, String),
    #[error("epoch must be between {0} and {1} but was {2}")]
    EpochOutOfRange(String, String, String),
    #[error("unknown field `{0}`")]
    FieldNotFound(String),
    #[error("`states` must have at least four element but had {0}")]
    TooFewStates(usize),
    #[error(transparent)]
    CubicSplineError(#[from] LoxCubicSplineError),
}

pub trait Trajectory<T: TimeScale + Copy, O: PointMass + Copy, F: ReferenceFrame + Copy> {
    fn state_at_time(&self, time: Time<T>) -> Result<Cartesian<T, O, F>, LoxTrajectoryError>;
    fn state_after_delta(&self, delta: TimeDelta)
        -> Result<Cartesian<T, O, F>, LoxTrajectoryError>;
    fn set_field(&mut self, field: &str, data: &[f64]) -> Result<(), LoxTrajectoryError>;
    fn field_at_time(&self, field: &str, time: Time<T>) -> Result<f64, LoxTrajectoryError>;
    fn field_after_delta(&self, field: &str, delta: TimeDelta) -> Result<f64, LoxTrajectoryError>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct CubicSplineTrajectory<T: TimeScale + Copy, O: PointMass + Copy, F: ReferenceFrame + Copy>
{
    scale: T,
    origin: O,
    frame: F,
    base: BaseCubicSplineTrajectory,
}

impl<T, O, F> CubicSplineTrajectory<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: ReferenceFrame + Copy,
{
    pub fn new(states: &[Cartesian<T, O, F>]) -> Result<Self, LoxTrajectoryError> {
        if states.len() < 4 {
            return Err(LoxTrajectoryError::TooFewStates(states.len()));
        }
        let s0 = states.first().unwrap();
        let states: Vec<BaseState> = states.iter().map(|&s| s.base()).collect();
        let base = BaseCubicSplineTrajectory::new(&states)?;
        Ok(Self {
            scale: s0.time().scale(),
            origin: s0.origin(),
            frame: s0.reference_frame(),
            base,
        })
    }

    pub fn from_base(scale: T, origin: O, frame: F, base: BaseCubicSplineTrajectory) -> Self {
        Self {
            scale,
            origin,
            frame,
            base,
        }
    }

    pub fn scale(&self) -> T {
        self.scale
    }
}

impl<T, O, F> TimeSystem for CubicSplineTrajectory<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: ReferenceFrame + Copy,
{
    type Scale = T;

    fn scale(&self) -> Self::Scale {
        self.scale
    }
}

impl<T, O, F> CoordinateSystem for CubicSplineTrajectory<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: ReferenceFrame + Copy,
{
    type Origin = O;
    type Frame = F;

    fn origin(&self) -> Self::Origin {
        self.origin
    }

    fn reference_frame(&self) -> Self::Frame {
        self.frame
    }
}

impl<T, O, F> Trajectory<T, O, F> for CubicSplineTrajectory<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: ReferenceFrame + Copy,
{
    fn state_at_time(&self, time: Time<T>) -> Result<Cartesian<T, O, F>, LoxTrajectoryError> {
        let (_, base) = self.base.state_at_time(time.base_time())?;
        Ok(Cartesian::new(
            time,
            self.origin(),
            self.reference_frame(),
            base.position(),
            base.velocity(),
        ))
    }

    fn state_after_delta(
        &self,
        delta: TimeDelta,
    ) -> Result<Cartesian<T, O, F>, LoxTrajectoryError> {
        let (base_time, base) = self.base.state_after_delta(delta)?;
        let time = Time::from_base_time(self.scale(), base_time);
        Ok(Cartesian::new(
            time,
            self.origin(),
            self.reference_frame(),
            base.position(),
            base.velocity(),
        ))
    }

    fn set_field(&mut self, field: &str, data: &[f64]) -> Result<(), LoxTrajectoryError> {
        self.base.set_field(field, data)
    }

    fn field_at_time(&self, field: &str, time: Time<T>) -> Result<f64, LoxTrajectoryError> {
        self.base.field_at_time(field, time.base_time())
    }

    fn field_after_delta(&self, field: &str, delta: TimeDelta) -> Result<f64, LoxTrajectoryError> {
        self.base.field_after_delta(field, delta)
    }
}

#[cfg(test)]
mod tests {
    use glam::DVec3;
    use lox_bodies::Earth;
    use lox_time::time_scales::Tai;

    use crate::frames::Icrf;
    use crate::two_body::Cartesian;

    use super::*;

    #[test]
    fn test_trajectory() {
        let states: Vec<Cartesian<Tai, Earth, Icrf>> = [1.0, 2.0, 3.0, 4.0]
            .iter()
            .map(|&i| {
                let position = DVec3::new(i, i, i) * 1.0e3;
                let velocity = DVec3::new(i, i, i);
                let time =
                    Time::default() + TimeDelta::from_decimal_seconds(i).expect("should be valid");
                Cartesian::new(time, Earth, Icrf, position, velocity)
            })
            .collect();
        let trajectory = CubicSplineTrajectory::new(&states).expect("should be valid");
        let t = Time::default() + TimeDelta::from_decimal_seconds(2.5).expect("should be valid");
        let actual = trajectory.state_at_time(t).expect("should be valid");

        assert_eq!(actual.position().x, 2500.0);
        assert_eq!(actual.position().y, 2500.0);
        assert_eq!(actual.position().z, 2500.0);
        assert_eq!(actual.velocity().x, 2.5);
        assert_eq!(actual.velocity().y, 2.5);
        assert_eq!(actual.velocity().z, 2.5);
        assert_eq!(actual.time(), t);
        assert_eq!(actual.origin(), Earth);
        assert_eq!(actual.reference_frame(), Icrf);
    }

    #[test]
    fn test_trajectory_field() {
        let states: Vec<Cartesian<Tai, Earth, Icrf>> = [1.0, 2.0, 3.0, 4.0]
            .iter()
            .map(|&i| {
                let position = DVec3::new(i, i, i) * 1.0e3;
                let velocity = DVec3::new(i, i, i);
                let time =
                    Time::default() + TimeDelta::from_decimal_seconds(i).expect("should be valid");
                Cartesian::new(time, Earth, Icrf, position, velocity)
            })
            .collect();
        let field = vec![1.0, 2.0, 3.0, 4.0];
        let mut trajectory = CubicSplineTrajectory::new(&states).expect("should be valid");
        trajectory
            .set_field("field", &field)
            .expect("should be valid");
        let t = Time::default() + TimeDelta::from_decimal_seconds(2.5).expect("should be valid");
        let actual = trajectory
            .field_at_time("field", t)
            .expect("should be valid");
        assert_eq!(actual, 2.5);
    }

    #[test]
    fn test_trajectory_too_few_states() {
        let states: Vec<Cartesian<Tai, Earth, Icrf>> = [1.0, 2.0, 3.0]
            .iter()
            .map(|&i| {
                let position = DVec3::new(i, i, i) * 1.0e3;
                let velocity = DVec3::new(i, i, i);
                let time =
                    Time::default() + TimeDelta::from_decimal_seconds(i).expect("should be valid");
                Cartesian::new(time, Earth, Icrf, position, velocity)
            })
            .collect();
        let trajectory = CubicSplineTrajectory::new(&states);
        assert_eq!(
            trajectory,
            Err(LoxTrajectoryError::TooFewStates(states.len()))
        );
    }

    #[test]
    fn test_trajectory_unknown_field() {
        let states: Vec<Cartesian<Tai, Earth, Icrf>> = [1.0, 2.0, 3.0, 4.0]
            .iter()
            .map(|&i| {
                let position = DVec3::new(i, i, i) * 1.0e3;
                let velocity = DVec3::new(i, i, i);
                let time =
                    Time::default() + TimeDelta::from_decimal_seconds(i).expect("should be valid");
                Cartesian::new(time, Earth, Icrf, position, velocity)
            })
            .collect();
        let trajectory = CubicSplineTrajectory::new(&states).expect("should be valid");
        let t = Time::default() + TimeDelta::from_decimal_seconds(2.5).expect("should be valid");
        let field = trajectory.field_at_time("field", t);
        assert_eq!(
            field,
            Err(LoxTrajectoryError::FieldNotFound("field".to_owned()))
        );
    }

    #[test]
    fn test_trajectory_delta_out_of_range() {
        let states: Vec<Cartesian<Tai, Earth, Icrf>> = [1.0, 2.0, 3.0, 4.0]
            .iter()
            .map(|&i| {
                let position = DVec3::new(i, i, i) * 1.0e3;
                let velocity = DVec3::new(i, i, i);
                let time =
                    Time::default() + TimeDelta::from_decimal_seconds(i).expect("should be valid");
                Cartesian::new(time, Earth, Icrf, position, velocity)
            })
            .collect();
        let field = vec![1.0, 2.0, 3.0, 4.0];
        let mut trajectory = CubicSplineTrajectory::new(&states).expect("should be valid");
        trajectory
            .set_field("field", &field)
            .expect("should be valid");

        let dt = TimeDelta::from_decimal_seconds(-5.0).expect("should be valid");
        assert_eq!(
            trajectory.state_after_delta(dt),
            Err(LoxTrajectoryError::DeltaOutOfRange(
                "3".to_owned(),
                "-5".to_owned()
            ))
        );
        assert_eq!(
            trajectory.field_after_delta("field", dt),
            Err(LoxTrajectoryError::DeltaOutOfRange(
                "3".to_owned(),
                "-5".to_owned()
            ))
        );

        let dt = TimeDelta::from_decimal_seconds(5.0).expect("should be valid");
        assert_eq!(
            trajectory.state_after_delta(dt),
            Err(LoxTrajectoryError::DeltaOutOfRange(
                "3".to_owned(),
                "5".to_owned()
            ))
        );
        assert_eq!(
            trajectory.field_after_delta("field", dt),
            Err(LoxTrajectoryError::DeltaOutOfRange(
                "3".to_owned(),
                "5".to_owned()
            ))
        );
    }

    #[test]
    fn test_trajectory_epoch_out_of_range() {
        let states: Vec<Cartesian<Tai, Earth, Icrf>> = [1.0, 2.0, 3.0, 4.0]
            .iter()
            .map(|&i| {
                let position = DVec3::new(i, i, i) * 1.0e3;
                let velocity = DVec3::new(i, i, i);
                let time =
                    Time::default() + TimeDelta::from_decimal_seconds(i).expect("should be valid");
                Cartesian::new(time, Earth, Icrf, position, velocity)
            })
            .collect();
        let field = vec![1.0, 2.0, 3.0, 4.0];
        let mut trajectory = CubicSplineTrajectory::new(&states).expect("should be valid");
        trajectory
            .set_field("field", &field)
            .expect("should be valid");

        let dt = TimeDelta::from_decimal_seconds(-5.0).expect("should be valid");
        let t = Time::default() + dt;
        assert_eq!(
            trajectory.state_at_time(t),
            Err(LoxTrajectoryError::EpochOutOfRange(
                "12:00:01.000".to_owned(),
                "12:00:04.000".to_owned(),
                "11:59:55.000".to_owned()
            ))
        );
        assert_eq!(
            trajectory.field_at_time("field", t),
            Err(LoxTrajectoryError::EpochOutOfRange(
                "12:00:01.000".to_owned(),
                "12:00:04.000".to_owned(),
                "11:59:55.000".to_owned()
            ))
        );

        let dt = TimeDelta::from_decimal_seconds(5.0).expect("should be valid");
        let t = Time::default() + dt;
        assert_eq!(
            trajectory.state_at_time(t),
            Err(LoxTrajectoryError::EpochOutOfRange(
                "12:00:01.000".to_owned(),
                "12:00:04.000".to_owned(),
                "12:00:05.000".to_owned()
            ))
        );
        assert_eq!(
            trajectory.field_at_time("field", t),
            Err(LoxTrajectoryError::EpochOutOfRange(
                "12:00:01.000".to_owned(),
                "12:00:04.000".to_owned(),
                "12:00:05.000".to_owned()
            ))
        );
    }
}
