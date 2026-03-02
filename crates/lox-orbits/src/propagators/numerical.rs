// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

use differential_equations::{
    interpolate::Interpolation,
    ode::{ODE, ODEProblem, OrdinaryNumericalMethod},
    prelude::ExplicitRungeKutta,
    traits::State,
};
use glam::DVec3;
use lox_bodies::{
    DynOrigin, J2, MeanRadius, Origin, PointMass, TryJ2, TryMeanRadius, TryPointMass,
    UndefinedOriginPropertyError,
};
use lox_core::coords::Cartesian;
use lox_frames::{DynFrame, ReferenceFrame};
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::{DynTimeScale, TimeScale};
use thiserror::Error;

use crate::orbits::{CartesianOrbit, TrajectorError, Trajectory};
use crate::propagators::Propagator;

/// Number of maximum-size integration steps per characteristic orbital timescale (r/v).
/// Since r/v ≈ T/(2π) for a circular orbit, this yields ~50 steps per orbit.
const H_MAX_STEPS_PER_TIMESCALE: f64 = 8.0;

#[derive(Debug, Error)]
pub enum J2Error {
    #[error("ODE solver failed: {0}")]
    Solver(String),
    #[error("ODE solver returned no solution")]
    EmptySolution,
    #[error("at least two time steps are needed")]
    InvalidTimeSteps,
    #[error(transparent)]
    Trajectory(#[from] TrajectorError),
}

#[derive(Debug, Clone, Copy)]
pub struct J2Propagator<T: TimeScale, O: TryJ2 + TryPointMass + TryMeanRadius, R: ReferenceFrame> {
    initial_state: CartesianOrbit<T, O, R>,
    rtol: f64,
    atol: f64,
    h_max: f64,
    h_min: f64,
    max_steps: usize,
}

pub type DynJ2Propagator = J2Propagator<DynTimeScale, DynOrigin, DynFrame>;

fn default_h_max(position: DVec3, velocity: DVec3) -> f64 {
    position.length() / velocity.length() / H_MAX_STEPS_PER_TIMESCALE
}

// Infallible — static bounds
impl<T, O, R> J2Propagator<T, O, R>
where
    T: TimeScale,
    O: J2 + PointMass + MeanRadius + Copy,
    R: ReferenceFrame,
{
    pub fn new(initial_state: CartesianOrbit<T, O, R>) -> Self {
        let h_max = default_h_max(initial_state.position(), initial_state.velocity());
        Self {
            initial_state,
            rtol: 1e-8,
            atol: 1e-6,
            h_max,
            h_min: 1e-6,
            max_steps: 100_000,
        }
    }
}

// Fallible — Try* bounds (covers DynOrigin)
impl<T, O, R> J2Propagator<T, O, R>
where
    T: TimeScale,
    O: TryJ2 + TryPointMass + TryMeanRadius + Copy,
    R: ReferenceFrame,
{
    pub fn try_new(
        initial_state: CartesianOrbit<T, O, R>,
    ) -> Result<Self, UndefinedOriginPropertyError> {
        initial_state.origin().try_gravitational_parameter()?;
        initial_state.origin().try_j2()?;
        initial_state.origin().try_mean_radius()?;

        let h_max = default_h_max(initial_state.position(), initial_state.velocity());
        Ok(Self {
            initial_state,
            rtol: 1e-8,
            atol: 1e-6,
            h_max,
            h_min: 1e-6,
            max_steps: 100_000,
        })
    }
}

impl<T, O, R> J2Propagator<T, O, R>
where
    T: TimeScale,
    O: TryJ2 + TryPointMass + TryMeanRadius + Copy,
    R: ReferenceFrame,
{
    pub fn with_rtol(mut self, rtol: f64) -> Self {
        self.rtol = rtol;
        self
    }

    pub fn with_atol(mut self, atol: f64) -> Self {
        self.atol = atol;
        self
    }

    pub fn with_h_max(mut self, h_max: f64) -> Self {
        self.h_max = h_max;
        self
    }

    pub fn with_h_min(mut self, h_min: f64) -> Self {
        self.h_min = h_min;
        self
    }

    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    pub fn initial_state(&self) -> &CartesianOrbit<T, O, R> {
        &self.initial_state
    }

    pub fn origin(&self) -> O {
        self.initial_state.origin()
    }

    pub fn reference_frame(&self) -> R
    where
        R: Copy,
    {
        self.initial_state.reference_frame()
    }

    fn gravitational_parameter(&self) -> f64 {
        self.initial_state
            .origin()
            .try_gravitational_parameter()
            .expect("gravitational parameter should be available")
            .as_f64()
    }

    fn j2(&self) -> f64 {
        self.initial_state
            .origin()
            .try_j2()
            .expect("J2 should be available")
    }

    fn mean_radius(&self) -> f64 {
        self.initial_state
            .origin()
            .try_mean_radius()
            .expect("mean radius should be available")
            .as_f64()
    }

    fn solver(
        &self,
    ) -> impl OrdinaryNumericalMethod<f64, CartesianState> + Interpolation<f64, CartesianState>
    {
        ExplicitRungeKutta::dop853()
            .rtol(self.rtol)
            .atol(self.atol)
            .h_max(self.h_max)
            .h_min(self.h_min)
            .max_steps(self.max_steps)
    }
}

impl<T, O, R> ODE<f64, CartesianState> for J2Propagator<T, O, R>
where
    T: TimeScale,
    O: TryJ2 + TryPointMass + TryMeanRadius + Copy,
    R: ReferenceFrame,
{
    fn diff(&self, _t: f64, s: &CartesianState, dydt: &mut CartesianState) {
        let mu = self.gravitational_parameter();
        let j2 = self.j2();
        let rm = self.mean_radius();

        let p = s.position();
        let pm = p.length();
        let pj = -3.0 / 2.0 * mu * j2 * rm.powi(2) / pm.powi(5);

        let acc = -mu * p / pm.powi(3)
            + pj * p * (DVec3::new(1.0, 1.0, 3.0) - 5.0 * p.z.powi(2) / pm.powi(2));

        dydt.0.set_position(s.velocity());
        dydt.0.set_velocity(acc);
    }
}

impl<T, O, R> Propagator<T, O> for J2Propagator<T, O, R>
where
    T: TimeScale + Copy + PartialOrd,
    O: TryJ2 + TryPointMass + TryMeanRadius + Origin + Copy,
    R: ReferenceFrame + Copy,
{
    type Frame = R;
    type Error = J2Error;

    fn state_at(&self, time: Time<T>) -> Result<CartesianOrbit<T, O, R>, J2Error> {
        let epoch = self.initial_state.time();
        let t0 = 0.0_f64;
        let t1 = (time - epoch).to_seconds().to_f64();
        let s0 = CartesianState::from(*self.initial_state());

        let mut solver = self.solver();

        let problem = ODEProblem::new(self, t0, t1, s0);
        let solution = problem
            .solve(&mut solver)
            .map_err(|e| J2Error::Solver(format!("{:?}", e)))?;

        let (_, final_state) = solution.iter().next_back().ok_or(J2Error::EmptySolution)?;

        let origin = self.initial_state.origin();
        let frame = self.initial_state.reference_frame();
        Ok(CartesianOrbit::new(final_state.0, time, origin, frame))
    }

    fn propagate(&self, interval: TimeInterval<T>) -> Result<Trajectory<T, O, R>, J2Error> {
        let start = interval.start();

        // Propagate to start of interval
        let s0: CartesianState = if start != self.initial_state.time() {
            self.state_at(start)?
        } else {
            *self.initial_state()
        }
        .into();

        let t1 = (interval.end() - start).to_seconds().to_f64();

        let mut solver = self.solver();

        let problem = ODEProblem::new(self, 0.0, t1, s0);
        let solution = problem
            .solve(&mut solver)
            .map_err(|e| J2Error::Solver(format!("{:?}", e)))?;

        let origin = self.initial_state.origin();
        let frame = self.initial_state.reference_frame();

        Ok(solution
            .iter()
            .map(|(t, s)| {
                CartesianOrbit::new(s.0, start + TimeDelta::from_seconds_f64(*t), origin, frame)
            })
            .collect())
    }

    fn propagate_to(
        &self,
        times: impl IntoIterator<Item = Time<T>>,
    ) -> Result<Trajectory<T, O, Self::Frame>, Self::Error> {
        let times: Vec<Time<T>> = times.into_iter().collect();
        if times.len() < 2 {
            return Err(J2Error::InvalidTimeSteps);
        }

        let t0 = times[0];
        let steps: Vec<f64> = times
            .iter()
            .map(|t| (*t - t0).to_seconds().to_f64())
            .collect();
        let t1 = *steps.last().unwrap();

        // Propagate to first time step
        let s0: CartesianState = if t0 != self.initial_state.time() {
            self.state_at(t0)?
        } else {
            *self.initial_state()
        }
        .into();

        let mut solver = self.solver();

        let problem = ODEProblem::new(self, 0.0, t1, s0);
        let solution = problem
            .t_eval(steps)
            .solve(&mut solver)
            .map_err(|e| J2Error::Solver(format!("{:?}", e)))?;

        let origin = self.initial_state.origin();
        let frame = self.initial_state.reference_frame();

        Ok(solution
            .iter()
            .map(|(t, s)| {
                CartesianOrbit::new(s.0, t0 + TimeDelta::from_seconds_f64(*t), origin, frame)
            })
            .collect())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CartesianState(Cartesian);

impl CartesianState {
    fn position(&self) -> DVec3 {
        self.0.position()
    }

    fn velocity(&self) -> DVec3 {
        self.0.velocity()
    }
}

impl State<f64> for CartesianState {
    fn len(&self) -> usize {
        6
    }

    fn get(&self, i: usize) -> f64 {
        match i {
            0 => self.0.position().x,
            1 => self.0.position().y,
            2 => self.0.position().z,
            3 => self.0.velocity().x,
            4 => self.0.velocity().y,
            5 => self.0.velocity().z,
            _ => unreachable!("index out of bounds"),
        }
    }

    fn set(&mut self, i: usize, value: f64) {
        match i {
            0 => {
                self.0.set::<0>(value);
            }
            1 => {
                self.0.set::<1>(value);
            }
            2 => {
                self.0.set::<2>(value);
            }
            3 => {
                self.0.set::<3>(value);
            }
            4 => {
                self.0.set::<4>(value);
            }
            5 => {
                self.0.set::<5>(value);
            }
            _ => unreachable!("index out of bounds"),
        };
    }

    fn zeros() -> Self {
        Self::default()
    }
}

impl Add for CartesianState {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        CartesianState(self.0 + rhs.0)
    }
}

impl AddAssign for CartesianState {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for CartesianState {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        CartesianState(self.0 - rhs.0)
    }
}

impl Mul<f64> for CartesianState {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        CartesianState(self.0 * rhs)
    }
}

impl Div<f64> for CartesianState {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        CartesianState(self.0 / rhs)
    }
}

impl Neg for CartesianState {
    type Output = Self;

    fn neg(self) -> Self::Output {
        CartesianState(-self.0)
    }
}

impl<T, O, R> From<CartesianOrbit<T, O, R>> for CartesianState
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    fn from(orbit: CartesianOrbit<T, O, R>) -> Self {
        Self(orbit.state())
    }
}

#[cfg(test)]
mod tests {
    use lox_bodies::Earth;
    use lox_frames::Icrf;
    use lox_test_utils::assert_approx_eq;
    use lox_time::Time;
    use lox_time::intervals::Interval;
    use lox_time::time;
    use lox_time::time_scales::Tdb;
    use lox_units::{DistanceUnits, VelocityUnits};

    use super::*;

    fn initial_state() -> CartesianOrbit<Tdb, Earth, Icrf> {
        let time = time!(Tdb, 2023, 1, 1).unwrap();
        CartesianOrbit::new(
            Cartesian::new(
                1131.340.km(),
                -2282.343.km(),
                6672.423.km(),
                -5.64305.kps(),
                4.30333.kps(),
                2.42879.kps(),
            ),
            time,
            Earth,
            Icrf,
        )
    }

    #[test]
    fn test_j2_ode() {
        let s0_orbit = initial_state();
        let j2 = J2Propagator::new(s0_orbit);

        let s0 = CartesianState(Cartesian::new(
            1131.340.km(),
            -2282.343.km(),
            6672.423.km(),
            -5.64305.kps(),
            4.30333.kps(),
            2.42879.kps(),
        ));
        let mut dsdt = CartesianState::default();
        j2.diff(0.0, &s0, &mut dsdt);

        let acc_exp = DVec3::new(-1.2324031762444367, 2.4862258582559233, -7.287340551142344);

        assert_eq!(dsdt.position(), s0.velocity());
        assert_approx_eq!(dsdt.velocity(), acc_exp, rtol <= 1e-8);
    }

    #[test]
    fn test_j2_propagator() {
        let s0_orbit = initial_state();
        let time = s0_orbit.time();
        let j2 = J2Propagator::new(s0_orbit);
        let dt = TimeDelta::from_minutes(40);
        let interval = Interval::new(time, time + dt);
        let traj = j2.propagate(interval).unwrap();
        let s1 = traj.states().into_iter().last().unwrap();
        let p_act = s1.position();
        let v_act = s1.velocity();
        let p_exp = DVec3::new(
            -4255.223590627231e3,
            4384.471704756651e3,
            -3.936_135_007_962_321e6,
        );
        let v_exp = DVec3::new(
            3.6559899898490054e3,
            -1.884445831960271e3,
            -6.123308149589636e3,
        );
        assert_approx_eq!(p_act, p_exp, rtol <= 1e-1);
        assert_approx_eq!(v_act, v_exp, rtol <= 1e-1);
    }

    /// Propagating [epoch, epoch+40m] and [epoch+20m, epoch+40m] should
    /// produce the same final state.
    #[test]
    fn test_propagate_with_offset_interval() {
        let s0_orbit = initial_state();
        let epoch = s0_orbit.time();
        let j2 = J2Propagator::new(s0_orbit);

        let dt = TimeDelta::from_minutes(40);
        let offset = TimeDelta::from_minutes(20);

        // Full interval from epoch
        let full = Interval::new(epoch, epoch + dt);
        let traj_full = j2.propagate(full).unwrap();
        let s_full = traj_full.states().into_iter().last().unwrap();

        // Offset interval starting 20 minutes after epoch
        let offset_interval = Interval::new(epoch + offset, epoch + dt);
        let traj_offset = j2.propagate(offset_interval).unwrap();
        let s_offset = traj_offset.states().into_iter().last().unwrap();

        // Final states should match
        assert_approx_eq!(s_full.position(), s_offset.position(), rtol <= 1e-6);
        assert_approx_eq!(s_full.velocity(), s_offset.velocity(), rtol <= 1e-6);

        // Trajectory timestamps should be consistent with the interval
        assert_eq!(traj_offset.start_time(), epoch + offset);
    }

    /// `state_at` and `propagate` should agree on the final state.
    #[test]
    fn test_state_at_matches_propagate() {
        let s0_orbit = initial_state();
        let epoch = s0_orbit.time();
        let j2 = J2Propagator::new(s0_orbit);

        let target = epoch + TimeDelta::from_minutes(40);
        let state = j2.state_at(target).unwrap();

        let interval = Interval::new(epoch, target);
        let traj = j2.propagate(interval).unwrap();
        let last = traj.states().into_iter().last().unwrap();

        assert_approx_eq!(state.position(), last.position(), rtol <= 1e-6);
        assert_approx_eq!(state.velocity(), last.velocity(), rtol <= 1e-6);
    }

    #[test]
    fn test_propagate_to() {
        let s0_orbit = initial_state();
        let epoch = s0_orbit.time();
        let j2 = J2Propagator::new(s0_orbit);

        let dt = TimeDelta::from_minutes(40);
        let interval = Interval::new(epoch, epoch + dt);
        let times: Vec<_> = interval.step_by(TimeDelta::from_minutes(10)).collect();

        let traj = j2.propagate_to(times.clone()).unwrap();
        let states = traj.states();

        // Should have exactly as many states as requested times
        assert_eq!(states.len(), times.len());

        // First state should match the initial state
        assert_approx_eq!(states[0].position(), s0_orbit.position(), rtol <= 1e-10);

        // Last state should match state_at for the same time
        let last_time = *times.last().unwrap();
        let expected = j2.state_at(last_time).unwrap();
        assert_approx_eq!(
            states.last().unwrap().position(),
            expected.position(),
            rtol <= 1e-6
        );
    }

    /// `propagate_to` with times starting after epoch should produce the
    /// same final state as propagating from epoch.
    #[test]
    fn test_propagate_to_with_offset_times() {
        let s0_orbit = initial_state();
        let epoch = s0_orbit.time();
        let j2 = J2Propagator::new(s0_orbit);

        let start = epoch + TimeDelta::from_minutes(20);
        let end = start + TimeDelta::from_minutes(20);
        let interval = Interval::new(start, end);
        let times: Vec<_> = interval.step_by(TimeDelta::from_minutes(5)).collect();

        let traj = j2.propagate_to(times.clone()).unwrap();
        let states = traj.states();

        assert_eq!(states.len(), times.len());

        // First state should match state_at for the offset time
        let expected_first = j2.state_at(times[0]).unwrap();
        assert_approx_eq!(
            states[0].position(),
            expected_first.position(),
            rtol <= 1e-6
        );

        // Last state should match state_at
        let expected_last = j2.state_at(*times.last().unwrap()).unwrap();
        assert_approx_eq!(
            states.last().unwrap().position(),
            expected_last.position(),
            rtol <= 1e-6
        );
    }

    #[test]
    fn test_propagate_to_too_few_times() {
        let s0_orbit = initial_state();
        let j2 = J2Propagator::new(s0_orbit);

        // Empty
        let result = j2.propagate_to(vec![]);
        assert!(result.is_err());

        // Single element
        let result = j2.propagate_to(vec![s0_orbit.time()]);
        assert!(result.is_err());
    }
}
