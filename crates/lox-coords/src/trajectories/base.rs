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
use lox_time::base_time::BaseTime;
use lox_time::deltas::TimeDelta;
use lox_utils::interpolation::cubic_spline::CubicSpline;

use crate::base::{BaseCartesian, BaseState};
use crate::trajectories::LoxTrajectoryError;

pub trait BaseTrajectory {
    fn state_at_time(&self, time: BaseTime) -> Result<BaseState, LoxTrajectoryError>;
    fn state_after_delta(&self, delta: TimeDelta) -> Result<BaseState, LoxTrajectoryError>;
    fn set_field(&mut self, field: &str, data: &[f64]) -> Result<(), LoxTrajectoryError>;
    fn field_at_time(&self, field: &str, time: BaseTime) -> Result<f64, LoxTrajectoryError>;
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
pub struct BaseCubicSplineTrajectory {
    t0: BaseTime,
    t1: BaseTime,
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

impl BaseCubicSplineTrajectory {
    pub fn new(times: &[BaseTime], states: &[BaseCartesian]) -> Result<Self, LoxTrajectoryError> {
        let n = times.len();
        if n != states.len() {
            return Err(LoxTrajectoryError::DimensionMismatch(n, states.len()));
        }
        if n < 4 {
            return Err(LoxTrajectoryError::TooFewStates(states.len()));
        }
        let t0 = *times.first().unwrap();
        let t1 = *times.last().unwrap();
        let dt_max = t1 - t0;
        let t: Vec<f64> = times
            .iter()
            .map(|&t| (t - t0).to_decimal_seconds())
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

    fn time_delta(&self, epoch: BaseTime) -> Option<TimeDelta> {
        let delta = epoch - self.t0;
        if delta.is_negative() {
            return None;
        }
        if delta > self.dt_max {
            return None;
        }
        Some(delta)
    }

    pub fn time(&self, delta: TimeDelta) -> BaseTime {
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

impl BaseTrajectory for BaseCubicSplineTrajectory {
    fn state_at_time(&self, time: BaseTime) -> Result<BaseState, LoxTrajectoryError> {
        let dt = self
            .time_delta(time)
            .ok_or(LoxTrajectoryError::EpochOutOfRange(
                self.t0.to_string(),
                self.t1.to_string(),
                time.to_string(),
            ))?;
        self.state_after_delta(dt)
    }

    fn state_after_delta(&self, delta: TimeDelta) -> Result<BaseState, LoxTrajectoryError> {
        if delta.is_negative() || delta > self.dt_max {
            return Err(LoxTrajectoryError::DeltaOutOfRange(
                self.dt_max.to_decimal_seconds().to_string(),
                delta.to_decimal_seconds().to_string(),
            ));
        }
        let time = self.time(delta);
        let position = self.position(delta);
        let velocity = self.velocity(delta);
        Ok((time, BaseCartesian::new(position, velocity)))
    }

    fn set_field(&mut self, field: &str, values: &[f64]) -> Result<(), LoxTrajectoryError> {
        let t = self.t.clone();
        let spline = CubicSpline::new(t, values.to_vec())?;
        self.fields.insert(field.to_string(), spline);
        Ok(())
    }

    fn field_at_time(&self, field: &str, time: BaseTime) -> Result<f64, LoxTrajectoryError> {
        let dt = self
            .time_delta(time)
            .ok_or(LoxTrajectoryError::EpochOutOfRange(
                self.t0.to_string(),
                self.t1.to_string(),
                time.to_string(),
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
