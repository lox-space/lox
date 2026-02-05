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
    J2, MeanRadius, PointMass, TryJ2, TryMeanRadius, TryPointMass, UndefinedOriginPropertyError,
};
use lox_core::{
    coords::{Cartesian, CartesianTrajectory, TimeStampedCartesian},
    elements::GravitationalParameter,
};
use lox_time::deltas::TimeDelta;
use lox_units::DistanceUnits;

pub struct J2Propagator<O: TryJ2 + TryPointMass + TryMeanRadius>(O);

impl<O> J2Propagator<O>
where
    O: J2 + PointMass + MeanRadius,
{
    pub fn new(origin: O) -> Self {
        Self(origin)
    }
}

impl<O> J2Propagator<O>
where
    O: TryJ2 + TryPointMass + TryMeanRadius,
{
    pub fn try_new(origin: O) -> Result<Self, UndefinedOriginPropertyError> {
        origin.try_gravitational_parameter()?;
        origin.try_j2()?;
        origin.try_mean_radius()?;

        Ok(Self(origin))
    }

    pub fn propagate(&self, initial: TimeStampedCartesian, dt: TimeDelta) -> CartesianTrajectory {
        let s0 = CartesianState(initial.state);
        let t1 = dt.to_seconds().to_f64();
        let mut solver = ExplicitRungeKutta::dop853()
            .rtol(1e-8)
            .atol(1e-6)
            .h_max(100.0);

        let problem = ODEProblem::new(self, 0.0, t1, s0);
        let solution = problem.solve(&mut solver).unwrap();
        solution
            .iter()
            .map(|(t, s)| TimeStampedCartesian {
                time: initial.time + TimeDelta::from_seconds_f64(*t),
                state: s.0,
            })
            .collect()
    }

    fn gravitational_parameter(&self) -> f64 {
        GravitationalParameter::km3_per_s2(self.0.try_gravitational_parameter().unwrap()).as_f64()
    }

    fn j2(&self) -> f64 {
        self.0.try_j2().unwrap()
    }

    fn mean_radius(&self) -> f64 {
        self.0.try_mean_radius().unwrap().km().as_f64()
    }
}

impl<O> ODE<f64, CartesianState> for J2Propagator<O>
where
    O: TryJ2 + TryPointMass + TryMeanRadius,
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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
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
    use lox_test_utils::assert_approx_eq;
    use lox_units::{DistanceUnits, VelocityUnits};

    use super::*;

    #[test]
    fn test_j2_ode() {
        let j2 = J2Propagator::new(Earth);
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
        let j2 = J2Propagator::new(Earth);
        let s0 = TimeStampedCartesian {
            state: Cartesian::new(
                1131.340.km(),
                -2282.343.km(),
                6672.423.km(),
                -5.64305.kps(),
                4.30333.kps(),
                2.42879.kps(),
            ),
            time: TimeDelta::default(),
        };
        let dt = TimeDelta::from_minutes(40.0);
        let tra = j2.propagate(s0, dt);
        let s1 = tra.into_iter().last().unwrap();
        let p_act = s1.state.position();
        let v_act = s1.state.velocity();
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
