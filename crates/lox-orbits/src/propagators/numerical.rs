// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

use differential_equations::{
    ode::{ODE, ODEProblem},
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

#[derive(Debug, Error)]
pub enum J2Error {
    #[error("ODE solver failed")]
    Solver,
    #[error(transparent)]
    Trajectory(#[from] TrajectorError),
}

#[derive(Debug, Clone, Copy)]
pub struct J2Propagator<T: TimeScale, O: TryJ2 + TryPointMass + TryMeanRadius, R: ReferenceFrame> {
    initial_state: CartesianOrbit<T, O, R>,
    rtol: f64,
    atol: f64,
    h_max: f64,
}

pub type DynJ2Propagator = J2Propagator<DynTimeScale, DynOrigin, DynFrame>;

// Infallible — static bounds
impl<T, O, R> J2Propagator<T, O, R>
where
    T: TimeScale,
    O: J2 + PointMass + MeanRadius + Copy,
    R: ReferenceFrame,
{
    pub fn new(initial_state: CartesianOrbit<T, O, R>) -> Self {
        Self {
            initial_state,
            rtol: 1e-8,
            atol: 1e-6,
            h_max: 100.0,
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

        Ok(Self {
            initial_state,
            rtol: 1e-8,
            atol: 1e-6,
            h_max: 100.0,
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

// Single impl covers both typed and DynJ2Propagator
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
        let s0 = CartesianState(Cartesian::from_vecs(
            self.initial_state.position(),
            self.initial_state.velocity(),
        ));

        let mut solver = ExplicitRungeKutta::dop853()
            .rtol(self.rtol)
            .atol(self.atol)
            .h_max(self.h_max);

        let problem = ODEProblem::new(self, t0, t1, s0);
        let solution = problem.solve(&mut solver).map_err(|_| J2Error::Solver)?;

        let (_, final_state) = solution.iter().last().ok_or(J2Error::Solver)?;

        let origin = self.initial_state.origin();
        let frame = self.initial_state.reference_frame();
        Ok(CartesianOrbit::new(final_state.0, time, origin, frame))
    }

    fn propagate(&self, interval: TimeInterval<T>) -> Result<Trajectory<T, O, R>, J2Error> {
        let epoch = self.initial_state.time();
        let t0 = 0.0_f64;
        let t1 = (interval.end() - epoch).to_seconds().to_f64();
        let s0 = CartesianState(Cartesian::from_vecs(
            self.initial_state.position(),
            self.initial_state.velocity(),
        ));

        let mut solver = ExplicitRungeKutta::dop853()
            .rtol(self.rtol)
            .atol(self.atol)
            .h_max(self.h_max);

        let problem = ODEProblem::new(self, t0, t1, s0);
        let solution = problem.solve(&mut solver).map_err(|_| J2Error::Solver)?;

        let origin = self.initial_state.origin();
        let frame = self.initial_state.reference_frame();
        let interval_start_offset = (interval.start() - epoch).to_seconds().to_f64();

        let states: Vec<_> = solution
            .iter()
            .filter(|(t, _)| **t >= interval_start_offset)
            .map(|(t, s)| {
                CartesianOrbit::new(s.0, epoch + TimeDelta::from_seconds_f64(*t), origin, frame)
            })
            .collect();

        Trajectory::try_new(states).map_err(Into::into)
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
        let dt = TimeDelta::from_minutes(40.0);
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
}
