// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

use differential_equations::{ode::ODE, traits::State};
use glam::DVec3;
use lox_bodies::{
    J2, MeanRadius, PointMass, TryJ2, TryMeanRadius, TryPointMass, UndefinedOriginPropertyError,
};
use lox_core::coords::Cartesian;

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

    fn gravitational_parameter(&self) -> f64 {
        self.0.try_gravitational_parameter().unwrap()
    }

    fn j2(&self) -> f64 {
        self.0.try_j2().unwrap()
    }

    fn mean_radius(&self) -> f64 {
        self.0.try_mean_radius().unwrap()
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

        let acc =
            -mu * p / rm.powi(3) + pj * p * (DVec3::new(1.0, 1.0, 3.0) - 5.0 * p.z / rm.powi(2));

        dydt.0.set_position(s.velocity());
        dydt.0.set_velocity(acc);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct CartesianState(Cartesian);

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
