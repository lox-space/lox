/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub use glam::DVec3;

use crate::bodies::PointMass;
use crate::frames::{InertialFrame, ReferenceFrame};
use crate::time::epochs::Epoch;
use crate::two_body::elements::{cartesian_to_keplerian, keplerian_to_cartesian};

pub mod elements;

#[derive(Debug, Clone, PartialEq)]
pub struct CartesianState {
    position: DVec3,
    velocity: DVec3,
}

impl CartesianState {
    pub fn new(position: DVec3, velocity: DVec3) -> Self {
        Self { position, velocity }
    }

    pub fn from_coords(x: f64, y: f64, z: f64, vx: f64, vy: f64, vz: f64) -> Self {
        let position = DVec3::new(x, y, z);
        let velocity = DVec3::new(vx, vy, vz);
        Self::new(position, velocity)
    }

    pub fn position(&self) -> DVec3 {
        self.position
    }

    pub fn velocity(&self) -> DVec3 {
        self.velocity
    }

    pub fn to_keplerian_state(&self, grav_param: f64) -> KeplerianState {
        let (semi_major, eccentricity, inclination, ascending_node, periapsis_arg, true_anomaly) =
            cartesian_to_keplerian(grav_param, self.position, self.velocity);
        KeplerianState {
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        }
    }

    pub fn semi_major(&self, grav_param: f64) -> f64 {
        self.to_keplerian_state(grav_param).semi_major
    }

    pub fn eccentricity(&self, grav_param: f64) -> f64 {
        self.to_keplerian_state(grav_param).eccentricity
    }

    pub fn inclination(&self, grav_param: f64) -> f64 {
        self.to_keplerian_state(grav_param).inclination
    }

    pub fn ascending_node(&self, grav_param: f64) -> f64 {
        self.to_keplerian_state(grav_param).ascending_node
    }

    pub fn periapsis_arg(&self, grav_param: f64) -> f64 {
        self.to_keplerian_state(grav_param).periapsis_arg
    }

    pub fn true_anomaly(&self, grav_param: f64) -> f64 {
        self.to_keplerian_state(grav_param).true_anomaly
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeplerianState {
    semi_major: f64,
    eccentricity: f64,
    inclination: f64,
    ascending_node: f64,
    periapsis_arg: f64,
    true_anomaly: f64,
}

impl KeplerianState {
    pub fn new(
        semi_major: f64,
        eccentricity: f64,
        inclination: f64,
        ascending_node: f64,
        periapsis_arg: f64,
        true_anomaly: f64,
    ) -> Self {
        Self {
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        }
    }

    pub fn position(&self, grav_param: f64) -> DVec3 {
        self.to_cartesian_state(grav_param).position
    }

    pub fn velocity(&self, grav_param: f64) -> DVec3 {
        self.to_cartesian_state(grav_param).velocity
    }

    pub fn to_cartesian_state(&self, grav_param: f64) -> CartesianState {
        let (position, velocity) = keplerian_to_cartesian(
            grav_param,
            self.semi_major,
            self.eccentricity,
            self.inclination,
            self.ascending_node,
            self.periapsis_arg,
            self.true_anomaly,
        );
        CartesianState::new(position, velocity)
    }

    pub fn semi_major(&self) -> f64 {
        self.semi_major
    }

    pub fn eccentricity(&self) -> f64 {
        self.eccentricity
    }

    pub fn inclination(&self) -> f64 {
        self.inclination
    }

    pub fn ascending_node(&self) -> f64 {
        self.ascending_node
    }

    pub fn periapsis_arg(&self) -> f64 {
        self.periapsis_arg
    }

    pub fn true_anomaly(&self) -> f64 {
        self.true_anomaly
    }
}

pub type Elements = (f64, f64, f64, f64, f64, f64);

impl From<Elements> for KeplerianState {
    fn from(elements: Elements) -> Self {
        let (semi_major, eccentricity, inclination, ascending_node, periapsis_arg, true_anomaly) =
            elements;
        Self {
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        }
    }
}

pub trait CoordinateSystem {
    type Origin: PointMass;
    type Frame: ReferenceFrame;

    fn origin(&self) -> Self::Origin;
    fn reference_frame(&self) -> Self::Frame;
}

pub trait TwoBody {
    fn time(&self) -> Epoch;
    fn to_cartesian_state(&self) -> CartesianState;
    fn to_keplerian_state(&self) -> KeplerianState;
    fn position(&self) -> DVec3;
    fn velocity(&self) -> DVec3;
    fn semi_major(&self) -> f64;
    fn eccentricity(&self) -> f64;
    fn inclination(&self) -> f64;
    fn ascending_node(&self) -> f64;
    fn periapsis_arg(&self) -> f64;
    fn true_anomaly(&self) -> f64;
}

#[derive(Debug, Clone)]
pub struct Cartesian<T, S>
where
    T: PointMass + Copy,
    S: ReferenceFrame + Copy,
{
    time: Epoch,
    origin: T,
    frame: S,
    state: CartesianState,
}

impl<T, S> Cartesian<T, S>
where
    T: PointMass + Copy,
    S: ReferenceFrame + Copy,
{
    pub fn new(time: Epoch, origin: T, frame: S, position: DVec3, velocity: DVec3) -> Self {
        Self {
            time,
            origin,
            frame,
            state: CartesianState { position, velocity },
        }
    }
}

impl<T, S> CoordinateSystem for Cartesian<T, S>
where
    T: PointMass + Copy,
    S: ReferenceFrame + Copy,
{
    type Origin = T;
    type Frame = S;

    fn origin(&self) -> Self::Origin {
        self.origin
    }

    fn reference_frame(&self) -> Self::Frame {
        self.frame
    }
}

impl<T, S> TwoBody for Cartesian<T, S>
where
    T: PointMass + Copy,
    S: ReferenceFrame + Copy,
{
    fn time(&self) -> Epoch {
        self.time
    }

    fn to_cartesian_state(&self) -> CartesianState {
        self.state.clone()
    }

    fn to_keplerian_state(&self) -> KeplerianState {
        let mu = self.origin.gravitational_parameter();
        self.state.to_keplerian_state(mu)
    }

    fn position(&self) -> DVec3 {
        self.state.position
    }

    fn velocity(&self) -> DVec3 {
        self.state.velocity
    }

    fn semi_major(&self) -> f64 {
        self.to_keplerian_state().semi_major
    }

    fn eccentricity(&self) -> f64 {
        self.to_keplerian_state().eccentricity
    }

    fn inclination(&self) -> f64 {
        self.to_keplerian_state().inclination
    }

    fn ascending_node(&self) -> f64 {
        self.to_keplerian_state().ascending_node
    }

    fn periapsis_arg(&self) -> f64 {
        self.to_keplerian_state().periapsis_arg
    }

    fn true_anomaly(&self) -> f64 {
        self.to_keplerian_state().true_anomaly
    }
}

impl<T, S> From<Keplerian<T, S>> for Cartesian<T, S>
where
    T: PointMass + Copy,
    S: InertialFrame + Copy,
{
    fn from(keplerian: Keplerian<T, S>) -> Self {
        let time = keplerian.time;
        let origin = keplerian.origin;
        let frame = keplerian.frame;
        let state = keplerian.to_cartesian_state();
        Cartesian {
            time,
            origin,
            frame,
            state,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Keplerian<T, S>
where
    T: PointMass + Copy,
    S: InertialFrame + Copy,
{
    time: Epoch,
    origin: T,
    frame: S,
    state: KeplerianState,
}

impl<T, S> Keplerian<T, S>
where
    T: PointMass + Copy,
    S: InertialFrame + Copy,
{
    pub fn new(time: Epoch, origin: T, frame: S, elements: Elements) -> Self {
        Self {
            time,
            origin,
            frame,
            state: elements.into(),
        }
    }
}

impl<T, S> CoordinateSystem for Keplerian<T, S>
where
    T: PointMass + Copy,
    S: InertialFrame + Copy,
{
    type Origin = T;
    type Frame = S;

    fn origin(&self) -> Self::Origin {
        self.origin
    }

    fn reference_frame(&self) -> Self::Frame {
        self.frame
    }
}

impl<T, S> TwoBody for Keplerian<T, S>
where
    T: PointMass + Copy,
    S: InertialFrame + Copy,
{
    fn time(&self) -> Epoch {
        self.time
    }

    fn to_cartesian_state(&self) -> CartesianState {
        let mu = self.origin.gravitational_parameter();
        let (position, velocity) = keplerian_to_cartesian(
            mu,
            self.state.semi_major,
            self.state.eccentricity,
            self.state.inclination,
            self.state.ascending_node,
            self.state.periapsis_arg,
            self.state.true_anomaly,
        );
        CartesianState { position, velocity }
    }

    fn to_keplerian_state(&self) -> KeplerianState {
        self.state.clone()
    }

    fn position(&self) -> DVec3 {
        self.to_cartesian_state().position
    }

    fn velocity(&self) -> DVec3 {
        self.to_cartesian_state().velocity
    }

    fn semi_major(&self) -> f64 {
        self.state.semi_major
    }

    fn eccentricity(&self) -> f64 {
        self.state.eccentricity
    }

    fn inclination(&self) -> f64 {
        self.state.inclination
    }

    fn ascending_node(&self) -> f64 {
        self.state.ascending_node
    }

    fn periapsis_arg(&self) -> f64 {
        self.state.periapsis_arg
    }

    fn true_anomaly(&self) -> f64 {
        self.state.true_anomaly
    }
}

impl<T, S> From<Cartesian<T, S>> for Keplerian<T, S>
where
    T: PointMass + Copy,
    S: InertialFrame + Copy,
{
    fn from(cartesian: Cartesian<T, S>) -> Self {
        let time = cartesian.time;
        let origin = cartesian.origin;
        let frame = cartesian.frame;
        let state = cartesian.to_keplerian_state();
        Self {
            time,
            origin,
            frame,
            state,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Mul;

    use float_eq::assert_float_eq;

    use crate::bodies::Earth;
    use crate::frames::Icrf;
    use crate::time::dates::{Date, Time};
    use crate::time::epochs::TimeScale;

    use super::*;

    #[test]
    fn test_two_body() {
        let date = Date::new(2023, 3, 25).expect("Date should be valid");
        let time = Time::new(21, 8, 0).expect("Time should be valid");
        let epoch = Epoch::from_date_and_time(TimeScale::TDB, date, time);
        let semi_major = 24464560.0e-3;
        let eccentricity = 0.7311;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.44369564302687126;
        let pos = DVec3::new(
            -0.107622532467967e7,
            -0.676589636432773e7,
            -0.332308783350379e6,
        )
        .mul(1e-3);
        let vel = DVec3::new(
            0.935685775154103e4,
            -0.331234775037644e4,
            -0.118801577532701e4,
        )
        .mul(1e-3);

        let cartesian = Cartesian::new(epoch, Earth, Icrf, pos, vel);

        assert_eq!(
            cartesian.to_cartesian_state(),
            CartesianState {
                position: cartesian.position(),
                velocity: cartesian.velocity(),
            }
        );
        assert_eq!(cartesian.time(), epoch);
        assert_eq!(cartesian.origin(), Earth);
        assert_eq!(cartesian.position(), pos);
        assert_eq!(cartesian.velocity(), vel);
        assert_float_eq!(cartesian.semi_major(), semi_major, rel <= 1e-6);
        assert_float_eq!(cartesian.eccentricity(), eccentricity, rel <= 1e-6);
        assert_float_eq!(cartesian.inclination(), inclination, rel <= 1e-6);
        assert_float_eq!(cartesian.ascending_node(), ascending_node, rel <= 1e-6);
        assert_float_eq!(cartesian.periapsis_arg(), periapsis_arg, rel <= 1e-6);
        assert_float_eq!(cartesian.true_anomaly(), true_anomaly, rel <= 1e-6);

        let keplerian = Keplerian::new(
            epoch,
            Earth,
            Icrf,
            (
                semi_major,
                eccentricity,
                inclination,
                ascending_node,
                periapsis_arg,
                true_anomaly,
            ),
        );

        assert_eq!(
            keplerian.to_keplerian_state(),
            KeplerianState {
                semi_major,
                eccentricity,
                inclination,
                ascending_node,
                periapsis_arg,
                true_anomaly
            }
        );
        assert_eq!(keplerian.time(), epoch);
        assert_eq!(keplerian.origin(), Earth);
        assert_float_eq!(keplerian.position().x, pos.x, rel <= 1e-8);
        assert_float_eq!(keplerian.position().y, pos.y, rel <= 1e-8);
        assert_float_eq!(keplerian.position().z, pos.z, rel <= 1e-8);
        assert_float_eq!(keplerian.velocity().x, vel.x, rel <= 1e-6);
        assert_float_eq!(keplerian.velocity().y, vel.y, rel <= 1e-6);
        assert_float_eq!(keplerian.velocity().z, vel.z, rel <= 1e-6);
        assert_float_eq!(keplerian.semi_major(), semi_major, rel <= 1e-6);
        assert_float_eq!(keplerian.eccentricity(), eccentricity, rel <= 1e-6);
        assert_float_eq!(keplerian.inclination(), inclination, rel <= 1e-6);
        assert_float_eq!(keplerian.ascending_node(), ascending_node, rel <= 1e-6);
        assert_float_eq!(keplerian.periapsis_arg(), periapsis_arg, rel <= 1e-6);
        assert_float_eq!(keplerian.true_anomaly(), true_anomaly, rel <= 1e-6);

        let cartesian1 = Cartesian::from(keplerian.clone());
        let keplerian1 = Keplerian::from(cartesian.clone());

        assert_float_eq!(
            cartesian.state.position.x,
            cartesian1.state.position.x,
            rel <= 1e-8
        );
        assert_float_eq!(
            cartesian.state.position.y,
            cartesian1.state.position.y,
            rel <= 1e-8
        );
        assert_float_eq!(
            cartesian.state.position.z,
            cartesian1.state.position.z,
            rel <= 1e-8
        );
        assert_float_eq!(
            cartesian.state.velocity.x,
            cartesian1.state.velocity.x,
            rel <= 1e-6
        );
        assert_float_eq!(
            cartesian.state.velocity.y,
            cartesian1.state.velocity.y,
            rel <= 1e-6
        );
        assert_float_eq!(
            cartesian.state.velocity.z,
            cartesian1.state.velocity.z,
            rel <= 1e-6
        );

        assert_float_eq!(
            keplerian.state.semi_major,
            keplerian1.state.semi_major,
            rel <= 1e-2
        );
        assert_float_eq!(
            keplerian.state.eccentricity,
            keplerian1.state.eccentricity,
            abs <= 1e-6
        );
        assert_float_eq!(
            keplerian.state.inclination,
            keplerian1.state.inclination,
            rel <= 1e-6
        );
        assert_float_eq!(
            keplerian.state.ascending_node,
            keplerian1.state.ascending_node,
            rel <= 1e-6
        );
        assert_float_eq!(
            keplerian.state.periapsis_arg,
            keplerian1.state.periapsis_arg,
            rel <= 1e-6
        );
        assert_float_eq!(
            keplerian.state.true_anomaly,
            keplerian1.state.true_anomaly,
            rel <= 1e-6
        );
    }
}
