/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::f64::consts::TAU;

use glam::DVec3;

use lox_bodies::PointMass;
use lox_time::time_scales::TimeScale;
use lox_time::Time;

use crate::base::{BaseCartesian, BaseKeplerian, BaseState, BaseTwoBody};
use crate::frames::{InertialFrame, ReferenceFrame};
use crate::CoordinateSystem;

pub trait TwoBody<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: InertialFrame + Copy,
{
    fn time(&self) -> Time<T>;

    fn to_cartesian(&self) -> Cartesian<T, O, F>;

    fn to_keplerian(&self) -> Keplerian<T, O, F>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cartesian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: ReferenceFrame + Copy,
{
    time: Time<T>,
    state: BaseCartesian,
    origin: O,
    frame: F,
}

impl<T, O, F> Cartesian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: ReferenceFrame + Copy,
{
    pub fn new(time: Time<T>, origin: O, frame: F, position: DVec3, velocity: DVec3) -> Self {
        let state = BaseCartesian::new(position, velocity);
        Self {
            time,
            state,
            origin,
            frame,
        }
    }

    pub fn from_base(scale: T, origin: O, frame: F, (time, state): BaseState) -> Self {
        let time = Time::from_base_time(scale, time);
        Self {
            time,
            state,
            origin,
            frame,
        }
    }

    pub fn time(&self) -> Time<T> {
        self.time
    }

    pub fn position(&self) -> DVec3 {
        self.state.position()
    }

    pub fn velocity(&self) -> DVec3 {
        self.state.velocity()
    }

    pub fn base(&self) -> BaseState {
        (self.time.base_time(), self.state)
    }
}

impl<T, O, F> TwoBody<T, O, F> for Cartesian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: InertialFrame + Copy,
{
    fn time(&self) -> Time<T> {
        self.time
    }

    fn to_cartesian(&self) -> Cartesian<T, O, F> {
        *self
    }

    fn to_keplerian(&self) -> Keplerian<T, O, F> {
        Keplerian::from(*self)
    }
}

impl<T, O, F> CoordinateSystem for Cartesian<T, O, F>
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

impl<T, O, F> From<Keplerian<T, O, F>> for Cartesian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: InertialFrame + Copy,
{
    fn from(keplerian: Keplerian<T, O, F>) -> Self {
        let grav_param = keplerian.origin.gravitational_parameter();
        let state = keplerian.state.to_cartesian_state(grav_param);
        Cartesian {
            state,
            time: keplerian.time(),
            origin: keplerian.origin,
            frame: keplerian.frame,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keplerian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: InertialFrame + Copy,
{
    time: Time<T>,
    state: BaseKeplerian,
    origin: O,
    frame: F,
}

impl<T, O, F> Keplerian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: InertialFrame + Copy,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        time: Time<T>,
        origin: O,
        frame: F,
        semi_major: f64,
        eccentricity: f64,
        inclination: f64,
        ascending_node: f64,
        periapsis_arg: f64,
        true_anomaly: f64,
    ) -> Self {
        let state = BaseKeplerian::new(
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );
        Self {
            time,
            state,
            origin,
            frame,
        }
    }

    pub fn time(&self) -> Time<T> {
        self.time
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

    pub fn orbital_period(&self) -> f64 {
        let mu = self.origin().gravitational_parameter();
        let a = self.semi_major_axis();
        TAU * (a.powi(3) / mu).sqrt()
    }
}

impl<T, O, F> TwoBody<T, O, F> for Keplerian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: InertialFrame + Copy,
{
    fn time(&self) -> Time<T> {
        self.time
    }

    fn to_cartesian(&self) -> Cartesian<T, O, F> {
        Cartesian::from(*self)
    }

    fn to_keplerian(&self) -> Keplerian<T, O, F> {
        *self
    }
}

impl<T, O, F> CoordinateSystem for Keplerian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: InertialFrame + Copy,
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

impl<T, O, F> From<Cartesian<T, O, F>> for Keplerian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: InertialFrame + Copy,
{
    fn from(cartesian: Cartesian<T, O, F>) -> Self {
        let grav_param = cartesian.origin.gravitational_parameter();
        let state = cartesian.state.to_keplerian_state(grav_param);
        Self {
            state,
            time: cartesian.time,
            origin: cartesian.origin,
            frame: cartesian.frame,
        }
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use lox_bodies::Earth;
    use lox_time::calendar_dates::Date;
    use lox_time::subsecond::Subsecond;
    use lox_time::time_scales::Tdb;
    use lox_time::utc::Utc;
    use lox_time::Time;

    use crate::frames::Icrf;

    use super::*;

    #[test]
    fn test_cartesian() {
        let date = Date::new(2023, 3, 25).expect("Date should be valid");
        let utc = Utc::new(21, 8, 0, Subsecond::default()).expect("Time should be valid");
        let time = Time::from_date_and_utc_timestamp(Tdb, date, utc);
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

        let cartesian = Cartesian::new(time, Earth, Icrf, pos, vel);
        assert_eq!(cartesian.to_cartesian(), cartesian);

        let cartesian1 = cartesian.to_keplerian().to_cartesian();

        assert_eq!(cartesian1.time(), time);
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
    fn test_cartesian_two_body_time() {
        let date = Date::new(2023, 3, 25).expect("Date should be valid");
        let utc = Utc::new(21, 8, 0, Subsecond::default()).expect("Time should be valid");
        let time = Time::from_date_and_utc_timestamp(Tdb, date, utc);
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

        let cartesian = Cartesian::new(time, Earth, Icrf, pos, vel);
        assert_eq!(TwoBody::time(&cartesian), time);
    }

    #[test]
    fn test_keplerian() {
        let date = Date::new(2023, 3, 25).expect("Date should be valid");
        let utc = Utc::new(21, 8, 0, Subsecond::default()).expect("Time should be valid");
        let time = Time::from_date_and_utc_timestamp(Tdb, date, utc);
        let semi_major = 24464560.0e-3;
        let eccentricity = 0.7311;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.44369564302687126;

        let keplerian = Keplerian::new(
            time,
            Earth,
            Icrf,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );
        assert_eq!(keplerian.to_keplerian(), keplerian);

        let keplerian1 = keplerian.to_cartesian().to_keplerian();

        assert_eq!(keplerian1.time(), time);
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

    #[test]
    fn test_keplerian_two_body_time() {
        let date = Date::new(2023, 3, 25).expect("Date should be valid");
        let utc = Utc::new(21, 8, 0, Subsecond::default()).expect("Time should be valid");
        let time = Time::from_date_and_utc_timestamp(Tdb, date, utc);
        let semi_major = 24464560.0e-3;
        let eccentricity = 0.7311;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.44369564302687126;

        let keplerian = Keplerian::new(
            time,
            Earth,
            Icrf,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );
        assert_eq!(TwoBody::time(&keplerian), time);
    }
}
