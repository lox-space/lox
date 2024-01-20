/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub use glam::DVec3;

use states::{CartesianState, KeplerianState};

use crate::bodies::PointMass;
use crate::coords::states::StateVector;
use crate::frames::{InertialFrame, ReferenceFrame};
use crate::time::epochs::Epoch;

pub mod elements;
pub mod states;

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
    state: CartesianState,
    origin: T,
    frame: S,
}

impl<T, S> Cartesian<T, S>
where
    T: PointMass + Copy,
    S: ReferenceFrame + Copy,
{
    pub fn new(time: Epoch, origin: T, frame: S, position: DVec3, velocity: DVec3) -> Self {
        let state = CartesianState::new(time, position, velocity);
        Self {
            state,
            origin,
            frame,
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
        self.state.time()
    }

    fn to_cartesian_state(&self) -> CartesianState {
        self.state
    }

    fn to_keplerian_state(&self) -> KeplerianState {
        let mu = self.origin.gravitational_parameter();
        self.state.to_keplerian_state(mu)
    }

    fn position(&self) -> DVec3 {
        self.state.position()
    }

    fn velocity(&self) -> DVec3 {
        self.state.velocity()
    }

    fn semi_major(&self) -> f64 {
        self.to_keplerian_state().semi_major()
    }

    fn eccentricity(&self) -> f64 {
        self.to_keplerian_state().eccentricity()
    }

    fn inclination(&self) -> f64 {
        self.to_keplerian_state().inclination()
    }

    fn ascending_node(&self) -> f64 {
        self.to_keplerian_state().ascending_node()
    }

    fn periapsis_arg(&self) -> f64 {
        self.to_keplerian_state().periapsis_arg()
    }

    fn true_anomaly(&self) -> f64 {
        self.to_keplerian_state().true_anomaly()
    }
}

impl<T, S> From<Keplerian<T, S>> for Cartesian<T, S>
where
    T: PointMass + Copy,
    S: InertialFrame + Copy,
{
    fn from(keplerian: Keplerian<T, S>) -> Self {
        let origin = keplerian.origin;
        let frame = keplerian.frame;
        let state = keplerian.to_cartesian_state();
        Cartesian {
            state,
            origin,
            frame,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Keplerian<T, S>
where
    T: PointMass + Copy,
    S: InertialFrame + Copy,
{
    state: KeplerianState,
    origin: T,
    frame: S,
}

impl<T, S> Keplerian<T, S>
where
    T: PointMass + Copy,
    S: InertialFrame + Copy,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        time: Epoch,
        origin: T,
        frame: S,
        semi_major: f64,
        eccentricity: f64,
        inclination: f64,
        ascending_node: f64,
        periapsis_arg: f64,
        true_anomaly: f64,
    ) -> Self {
        let state: KeplerianState = StateVector(
            time,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        )
        .into();
        Self {
            state,
            origin,
            frame,
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
        self.state.time()
    }

    fn to_cartesian_state(&self) -> CartesianState {
        let mu = self.origin.gravitational_parameter();
        self.state.to_cartesian_state(mu)
    }

    fn to_keplerian_state(&self) -> KeplerianState {
        self.state
    }

    fn position(&self) -> DVec3 {
        self.to_cartesian_state().position()
    }

    fn velocity(&self) -> DVec3 {
        self.to_cartesian_state().velocity()
    }

    fn semi_major(&self) -> f64 {
        self.state.semi_major()
    }

    fn eccentricity(&self) -> f64 {
        self.state.eccentricity()
    }

    fn inclination(&self) -> f64 {
        self.state.inclination()
    }

    fn ascending_node(&self) -> f64 {
        self.state.ascending_node()
    }

    fn periapsis_arg(&self) -> f64 {
        self.state.periapsis_arg()
    }

    fn true_anomaly(&self) -> f64 {
        self.state.true_anomaly()
    }
}

impl<T, S> From<Cartesian<T, S>> for Keplerian<T, S>
where
    T: PointMass + Copy,
    S: InertialFrame + Copy,
{
    fn from(cartesian: Cartesian<T, S>) -> Self {
        let origin = cartesian.origin;
        let frame = cartesian.frame;
        let state = cartesian.to_keplerian_state();
        Self {
            state,
            origin,
            frame,
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
            CartesianState::new(cartesian.time(), cartesian.position(), cartesian.velocity())
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
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );

        assert_eq!(
            keplerian.to_keplerian_state(),
            KeplerianState::new(
                keplerian.time(),
                keplerian.semi_major(),
                keplerian.eccentricity(),
                keplerian.inclination(),
                keplerian.ascending_node(),
                keplerian.periapsis_arg(),
                keplerian.true_anomaly(),
            )
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

        assert_float_eq!(cartesian.position().x, cartesian1.position().x, rel <= 1e-8);
        assert_float_eq!(cartesian.position().y, cartesian1.position().y, rel <= 1e-8);
        assert_float_eq!(cartesian.position().z, cartesian1.position().z, rel <= 1e-8);
        assert_float_eq!(cartesian.velocity().x, cartesian1.velocity().x, rel <= 1e-6);
        assert_float_eq!(cartesian.velocity().y, cartesian1.velocity().y, rel <= 1e-6);
        assert_float_eq!(cartesian.velocity().z, cartesian1.velocity().z, rel <= 1e-6);

        assert_float_eq!(keplerian.semi_major(), keplerian1.semi_major(), rel <= 1e-2);
        assert_float_eq!(
            keplerian.eccentricity(),
            keplerian1.eccentricity(),
            abs <= 1e-6
        );
        assert_float_eq!(
            keplerian.inclination(),
            keplerian1.inclination(),
            rel <= 1e-6
        );
        assert_float_eq!(
            keplerian.ascending_node(),
            keplerian1.ascending_node(),
            rel <= 1e-6
        );
        assert_float_eq!(
            keplerian.periapsis_arg(),
            keplerian1.periapsis_arg(),
            rel <= 1e-6
        );
        assert_float_eq!(
            keplerian.true_anomaly(),
            keplerian1.true_anomaly(),
            rel <= 1e-6
        );
    }
}
