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
use crate::coords::states::TwoBodyState;
use crate::frames::{InertialFrame, ReferenceFrame};
use crate::time::epochs::Epoch;

pub mod anomalies;
pub mod states;

pub trait CoordinateSystem {
    type Origin: PointMass;
    type Frame: ReferenceFrame;

    fn origin(&self) -> Self::Origin;
    fn reference_frame(&self) -> Self::Frame;
}

pub trait TwoBody<T, S>
where
    T: PointMass + Copy,
    S: InertialFrame + Copy,
{
    fn to_cartesian(&self) -> Cartesian<T, S>;

    fn to_keplerian(&self) -> Keplerian<T, S>;
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

    pub fn time(&self) -> Epoch {
        self.state.time()
    }

    pub fn position(&self) -> DVec3 {
        self.state.position()
    }

    pub fn velocity(&self) -> DVec3 {
        self.state.position()
    }
}

impl<T, S> TwoBody<T, S> for Cartesian<T, S>
where
    T: PointMass + Copy,
    S: InertialFrame + Copy,
{
    fn to_cartesian(&self) -> Cartesian<T, S> {
        *self
    }

    fn to_keplerian(&self) -> Keplerian<T, S> {
        Keplerian::from(*self)
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

impl<T, S> From<Keplerian<T, S>> for Cartesian<T, S>
where
    T: PointMass + Copy,
    S: InertialFrame + Copy,
{
    fn from(keplerian: Keplerian<T, S>) -> Self {
        let grav_param = keplerian.origin.gravitational_parameter();
        let state = keplerian.state.to_cartesian_state(grav_param);
        Cartesian {
            state,
            origin: keplerian.origin,
            frame: keplerian.frame,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
        let state = KeplerianState::new(
            time,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );
        Self {
            state,
            origin,
            frame,
        }
    }

    pub fn time(&self) -> Epoch {
        self.state.time()
    }

    pub fn semi_major_axis(&self) -> f64 {
        self.state.semi_major_axis()
    }

    pub fn eccentricity(&self) -> f64 {
        self.state.eccentricity()
    }

    pub fn inclination(&self) -> f64 {
        self.state.inclination()
    }

    pub fn ascending_node(&self) -> f64 {
        self.state.ascending_node()
    }

    pub fn periapsis_argument(&self) -> f64 {
        self.state.periapsis_argument()
    }

    pub fn true_anomaly(&self) -> f64 {
        self.state.true_anomaly()
    }
}

impl<T, S> TwoBody<T, S> for Keplerian<T, S>
where
    T: PointMass + Copy,
    S: InertialFrame + Copy,
{
    fn to_cartesian(&self) -> Cartesian<T, S> {
        Cartesian::from(*self)
    }

    fn to_keplerian(&self) -> Keplerian<T, S> {
        *self
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

impl<T, S> From<Cartesian<T, S>> for Keplerian<T, S>
where
    T: PointMass + Copy,
    S: InertialFrame + Copy,
{
    fn from(cartesian: Cartesian<T, S>) -> Self {
        let grav_param = cartesian.origin.gravitational_parameter();
        let state = cartesian.state.to_keplerian_state(grav_param);
        Self {
            state,
            origin: cartesian.origin,
            frame: cartesian.frame,
        }
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use crate::bodies::Earth;
    use crate::frames::Icrf;
    use crate::time::dates::{Date, Time};
    use crate::time::epochs::TimeScale;

    use super::*;

    #[test]
    fn test_cartesian() {
        let date = Date::new(2023, 3, 25).expect("Date should be valid");
        let time = Time::new(21, 8, 0).expect("Time should be valid");
        let epoch = Epoch::from_date_and_time(TimeScale::TDB, date, time);
        let pos = DVec3::new(
            -0.107622532467967e7,
            -0.676589636432773e7,
            -0.332308783350379e6,
        ) * 1e-3;
        let vel = DVec3::new(
            0.935685775154103e4,
            -0.331234775037644e4,
            -0.118801577532701e4,
        ) * 1e-3;

        let cartesian = Cartesian::new(epoch, Earth, Icrf, pos, vel);
        let cartesian1 = cartesian.to_keplerian().to_cartesian();

        assert_eq!(cartesian1.time(), epoch);
        assert_eq!(cartesian1.origin(), Earth);
        assert_eq!(cartesian1.reference_frame(), Icrf);

        assert_float_eq!(cartesian.position().x, cartesian1.position().x, rel <= 1e-8);
        assert_float_eq!(cartesian.position().y, cartesian1.position().y, rel <= 1e-8);
        assert_float_eq!(cartesian.position().z, cartesian1.position().z, rel <= 1e-8);
        assert_float_eq!(cartesian.velocity().x, cartesian1.velocity().x, rel <= 1e-6);
        assert_float_eq!(cartesian.velocity().y, cartesian1.velocity().y, rel <= 1e-6);
        assert_float_eq!(cartesian.velocity().z, cartesian1.velocity().z, rel <= 1e-6);
    }

    #[test]
    fn test_keplerian() {
        let date = Date::new(2023, 3, 25).expect("Date should be valid");
        let time = Time::new(21, 8, 0).expect("Time should be valid");
        let epoch = Epoch::from_date_and_time(TimeScale::TDB, date, time);
        let semi_major = 24464560.0e-3;
        let eccentricity = 0.7311;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.44369564302687126;

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
        let keplerian1 = keplerian.to_cartesian().to_keplerian();

        assert_eq!(keplerian1.time(), epoch);
        assert_eq!(keplerian1.origin(), Earth);
        assert_eq!(keplerian1.reference_frame(), Icrf);

        assert_float_eq!(
            keplerian.semi_major_axis(),
            keplerian1.semi_major_axis(),
            rel <= 1e-6
        );
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
            keplerian.periapsis_argument(),
            keplerian1.periapsis_argument(),
            rel <= 1e-6
        );
        assert_float_eq!(
            keplerian.true_anomaly(),
            keplerian1.true_anomaly(),
            rel <= 1e-6
        );
    }
}
