/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;
use std::rc::Rc;

use glam::DVec3;
use thiserror::Error;

use lox_bodies::PointMass;
use lox_time::deltas::TimeDelta;
use lox_time::time_scales::TimeScale;
use lox_time::Time;
use lox_utils::interpolation::cubic_spline::{CubicSpline, LoxCubicSplineError};

use crate::frames::ReferenceFrame;
use crate::two_body::Cartesian;
use crate::CoordinateSystem;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum LoxTrajectoryError {
    #[error("epoch must be between {0} and {1} but was {2}")]
    EpochOutOfRange(String, String, String),
    #[error("time delta must be between 0.0 seconds and {0} seconds but was {1} seconds")]
    DeltaOutOfRange(String, String),
    #[error("`states` must have at least four element but had {0}")]
    TooFewStates(usize),
    #[error("unknown field `{0}`")]
    FieldNotFound(String),
    #[error(transparent)]
    CubicSplineError(#[from] LoxCubicSplineError),
}

pub trait Trajectory {
    type Time: TimeScale + Copy;
    type Origin: PointMass + Copy;
    type Frame: ReferenceFrame + Copy;

    fn state_at_epoch(
        &self,
        epoch: Time<Self::Time>,
    ) -> Result<Cartesian<Self::Time, Self::Origin, Self::Frame>, LoxTrajectoryError>;
    fn state_after_delta(
        &self,
        delta: TimeDelta,
    ) -> Result<Cartesian<Self::Time, Self::Origin, Self::Frame>, LoxTrajectoryError>;
    fn field_at_epoch(
        &self,
        field: &str,
        epoch: Time<Self::Time>,
    ) -> Result<f64, LoxTrajectoryError>;
    fn field_after_delta(&self, field: &str, delta: TimeDelta) -> Result<f64, LoxTrajectoryError>;
}

#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
struct RcVecF64(Rc<Vec<f64>>);

impl AsRef<[f64]> for RcVecF64 {
    fn as_ref(&self) -> &[f64] {
        self.0.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CubicSplineTrajectory<T: TimeScale + Copy, O: PointMass + Copy, F: ReferenceFrame + Copy>
{
    origin: O,
    frame: F,
    t0: Time<T>,
    t1: Time<T>,
    dt_max: TimeDelta,
    t: RcVecF64,
    x: CubicSpline<RcVecF64, Vec<f64>>,
    y: CubicSpline<RcVecF64, Vec<f64>>,
    z: CubicSpline<RcVecF64, Vec<f64>>,
    vy: CubicSpline<RcVecF64, Vec<f64>>,
    vx: CubicSpline<RcVecF64, Vec<f64>>,
    vz: CubicSpline<RcVecF64, Vec<f64>>,
    fields: HashMap<String, CubicSpline<RcVecF64, Vec<f64>>>,
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
        let t0 = s0.time();
        let t1 = states.last().unwrap().time();
        let dt_max = t1 - t0;
        let t: Vec<f64> = states
            .iter()
            .map(|s| (s.time() - t0).to_decimal_seconds())
            .collect();
        let t = RcVecF64(Rc::new(t));
        let x: Vec<f64> = states.iter().map(|s| s.position().x).collect();
        let y: Vec<f64> = states.iter().map(|s| s.position().y).collect();
        let z: Vec<f64> = states.iter().map(|s| s.position().z).collect();
        let vx: Vec<f64> = states.iter().map(|s| s.velocity().x).collect();
        let vy: Vec<f64> = states.iter().map(|s| s.velocity().y).collect();
        let vz: Vec<f64> = states.iter().map(|s| s.velocity().z).collect();
        let x = CubicSpline::new(t.clone(), x)?;
        let y = CubicSpline::new(t.clone(), y)?;
        let z = CubicSpline::new(t.clone(), z)?;
        let vx = CubicSpline::new(t.clone(), vx)?;
        let vy = CubicSpline::new(t.clone(), vy)?;
        let vz = CubicSpline::new(t.clone(), vz)?;
        Ok(Self {
            origin: s0.origin(),
            frame: s0.reference_frame(),
            t0,
            t1,
            dt_max,
            t,
            x,
            y,
            z,
            vx,
            vy,
            vz,
            fields: HashMap::default(),
        })
    }

    pub fn add_field(&mut self, field: &str, values: &[f64]) -> Result<(), LoxTrajectoryError> {
        let t = self.t.clone();
        let spline = CubicSpline::new(t, values.to_vec())?;
        self.fields.insert(field.to_string(), spline);
        Ok(())
    }

    fn time_delta(&self, epoch: Time<T>) -> Option<TimeDelta> {
        let delta = epoch - self.t0;
        if delta.is_negative() {
            return None;
        }
        if delta > self.dt_max {
            return None;
        }
        Some(delta)
    }

    pub fn time(&self, delta: TimeDelta) -> Time<T> {
        self.t0 + delta
    }

    pub fn position(&self, delta: TimeDelta) -> DVec3 {
        let t = delta.to_decimal_seconds();
        DVec3::new(
            self.x.interpolate(t),
            self.y.interpolate(t),
            self.z.interpolate(t),
        )
    }

    pub fn velocity(&self, delta: TimeDelta) -> DVec3 {
        let t = delta.to_decimal_seconds();
        DVec3::new(
            self.vx.interpolate(t),
            self.vy.interpolate(t),
            self.vz.interpolate(t),
        )
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

impl<T, O, F> Trajectory for CubicSplineTrajectory<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: ReferenceFrame + Copy,
{
    type Time = T;

    type Origin = O;

    type Frame = F;

    fn state_at_epoch(
        &self,
        epoch: Time<Self::Time>,
    ) -> Result<Cartesian<Self::Time, Self::Origin, Self::Frame>, LoxTrajectoryError> {
        let dt = self
            .time_delta(epoch)
            .ok_or(LoxTrajectoryError::EpochOutOfRange(
                self.t0.to_string(),
                self.t1.to_string(),
                epoch.to_string(),
            ))?;
        self.state_after_delta(dt)
    }

    fn state_after_delta(
        &self,
        delta: TimeDelta,
    ) -> Result<Cartesian<Self::Time, Self::Origin, Self::Frame>, LoxTrajectoryError> {
        if delta.is_negative() || delta > self.dt_max {
            return Err(LoxTrajectoryError::DeltaOutOfRange(
                self.dt_max.to_decimal_seconds().to_string(),
                delta.to_decimal_seconds().to_string(),
            ));
        }
        let time = self.time(delta);
        let position = self.position(delta);
        let velocity = self.velocity(delta);
        Ok(Cartesian::new(
            time,
            self.origin(),
            self.reference_frame(),
            position,
            velocity,
        ))
    }

    fn field_at_epoch(
        &self,
        field: &str,
        epoch: Time<Self::Time>,
    ) -> Result<f64, LoxTrajectoryError> {
        let dt = self
            .time_delta(epoch)
            .ok_or(LoxTrajectoryError::EpochOutOfRange(
                self.t0.to_string(),
                self.t1.to_string(),
                epoch.to_string(),
            ))?;
        self.field_after_delta(field, dt)
    }

    fn field_after_delta(&self, field: &str, delta: TimeDelta) -> Result<f64, LoxTrajectoryError> {
        if delta.is_negative() || delta > self.dt_max {
            return Err(LoxTrajectoryError::DeltaOutOfRange(
                self.dt_max.to_decimal_seconds().to_string(),
                delta.to_decimal_seconds().to_string(),
            ));
        }
        let spl = self
            .fields
            .get(field)
            .ok_or(LoxTrajectoryError::FieldNotFound(field.to_owned()))?;
        Ok(spl.interpolate(delta.to_decimal_seconds()))
    }
}

#[cfg(test)]
mod tests {

    use crate::frames::Icrf;
    use crate::two_body::Cartesian;
    use lox_bodies::Earth;
    use lox_time::time_scales::TAI;

    use super::*;

    #[test]
    fn test_trajectory() {
        let states: Vec<Cartesian<TAI, Earth, Icrf>> = vec![1.0, 2.0, 3.0, 4.0]
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
        let actual = trajectory.state_at_epoch(t).expect("should be valid");

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
        let states: Vec<Cartesian<TAI, Earth, Icrf>> = vec![1.0, 2.0, 3.0, 4.0]
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
            .add_field("field", &field)
            .expect("should be valid");
        let t = Time::default() + TimeDelta::from_decimal_seconds(2.5).expect("should be valid");
        let actual = trajectory
            .field_at_epoch("field", t)
            .expect("should be valid");
        assert_eq!(actual, 2.5);
    }

    #[test]
    fn test_trajectory_too_few_states() {
        let states: Vec<Cartesian<TAI, Earth, Icrf>> = vec![1.0, 2.0, 3.0]
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
        let states: Vec<Cartesian<TAI, Earth, Icrf>> = vec![1.0, 2.0, 3.0, 4.0]
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
        let field = trajectory.field_at_epoch("field", t);
        assert_eq!(
            field,
            Err(LoxTrajectoryError::FieldNotFound("field".to_owned()))
        );
    }

    #[test]
    fn test_trajectory_delta_out_of_range() {
        let states: Vec<Cartesian<TAI, Earth, Icrf>> = vec![1.0, 2.0, 3.0, 4.0]
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
            .add_field("field", &field)
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
        let states: Vec<Cartesian<TAI, Earth, Icrf>> = vec![1.0, 2.0, 3.0, 4.0]
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
            .add_field("field", &field)
            .expect("should be valid");

        let dt = TimeDelta::from_decimal_seconds(-5.0).expect("should be valid");
        let t = Time::default() + dt;
        assert_eq!(
            trajectory.state_at_epoch(t),
            Err(LoxTrajectoryError::EpochOutOfRange(
                "12:00:01.000 TAI".to_owned(),
                "12:00:04.000 TAI".to_owned(),
                "11:59:55.000 TAI".to_owned()
            ))
        );
        assert_eq!(
            trajectory.field_at_epoch("field", t),
            Err(LoxTrajectoryError::EpochOutOfRange(
                "12:00:01.000 TAI".to_owned(),
                "12:00:04.000 TAI".to_owned(),
                "11:59:55.000 TAI".to_owned()
            ))
        );

        let dt = TimeDelta::from_decimal_seconds(5.0).expect("should be valid");
        let t = Time::default() + dt;
        assert_eq!(
            trajectory.state_at_epoch(t),
            Err(LoxTrajectoryError::EpochOutOfRange(
                "12:00:01.000 TAI".to_owned(),
                "12:00:04.000 TAI".to_owned(),
                "12:00:05.000 TAI".to_owned()
            ))
        );
        assert_eq!(
            trajectory.field_at_epoch("field", t),
            Err(LoxTrajectoryError::EpochOutOfRange(
                "12:00:01.000 TAI".to_owned(),
                "12:00:04.000 TAI".to_owned(),
                "12:00:05.000 TAI".to_owned()
            ))
        );
    }
}
