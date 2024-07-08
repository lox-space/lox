/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::f64::consts::TAU;

use float_eq::float_eq;
use glam::{DMat3, DVec3};

use lox_bodies::PointMass;
use lox_time::deltas::TimeDelta;
use lox_time::TimeLike;

use crate::frames::{CoordinateSystem, Icrf};
use crate::origins::CoordinateOrigin;
use crate::states::{State, ToCartesian};

pub trait ToKeplerian<T: TimeLike, O: PointMass> {
    fn to_keplerian(&self) -> Keplerian<T, O>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Keplerian<T: TimeLike, O: PointMass> {
    time: T,
    origin: O,
    semi_major_axis: f64,
    eccentricity: f64,
    inclination: f64,
    longitude_of_ascending_node: f64,
    argument_of_periapsis: f64,
    true_anomaly: f64,
}

impl<T, O> Keplerian<T, O>
where
    T: TimeLike,
    O: PointMass,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        time: T,
        origin: O,
        semi_major_axis: f64,
        eccentricity: f64,
        inclination: f64,
        longitude_of_ascending_node: f64,
        argument_of_periapsis: f64,
        true_anomaly: f64,
    ) -> Self {
        Self {
            time,
            origin,
            semi_major_axis,
            eccentricity,
            inclination,
            longitude_of_ascending_node,
            argument_of_periapsis,
            true_anomaly,
        }
    }

    pub fn time(&self) -> T
    where
        T: Clone,
    {
        self.time.clone()
    }

    pub fn semi_major_axis(&self) -> f64 {
        self.semi_major_axis
    }

    pub fn eccentricity(&self) -> f64 {
        self.eccentricity
    }

    pub fn inclination(&self) -> f64 {
        self.inclination
    }

    pub fn longitude_of_ascending_node(&self) -> f64 {
        self.longitude_of_ascending_node
    }

    pub fn argument_of_periapsis(&self) -> f64 {
        self.argument_of_periapsis
    }

    pub fn true_anomaly(&self) -> f64 {
        self.true_anomaly
    }

    pub fn semiparameter(&self) -> f64 {
        if is_circular(self.eccentricity) {
            self.semi_major_axis
        } else {
            self.semi_major_axis * (1.0 - self.eccentricity.powi(2))
        }
    }

    pub fn to_perifocal(&self) -> (DVec3, DVec3) {
        let grav_param = self.origin.gravitational_parameter();
        let semiparameter = self.semiparameter();
        let (sin_nu, cos_nu) = self.true_anomaly.sin_cos();
        let sqrt_mu_p = (grav_param / semiparameter).sqrt();

        let pos =
            DVec3::new(cos_nu, sin_nu, 0.0) * (semiparameter / (1.0 + self.eccentricity * cos_nu));
        let vel = DVec3::new(-sin_nu, self.eccentricity + cos_nu, 0.0) * sqrt_mu_p;

        (pos, vel)
    }

    pub fn orbital_period(&self) -> TimeDelta {
        let mu = self.origin.gravitational_parameter();
        let a = self.semi_major_axis();
        TimeDelta::from_decimal_seconds(TAU * (a.powi(3) / mu).sqrt()).unwrap()
    }
}

impl<T: TimeLike, O: PointMass + Clone> CoordinateOrigin<O> for Keplerian<T, O> {
    fn origin(&self) -> O {
        self.origin.clone()
    }
}

impl<T: TimeLike, O: PointMass> CoordinateSystem<Icrf> for Keplerian<T, O> {
    fn reference_frame(&self) -> Icrf {
        Icrf
    }
}

impl<T, O> ToCartesian<T, O, Icrf> for Keplerian<T, O>
where
    T: TimeLike + Clone,
    O: PointMass + Clone,
{
    fn to_cartesian(&self) -> State<T, O, Icrf> {
        let (pos, vel) = self.to_perifocal();
        let rot = DMat3::from_rotation_z(self.longitude_of_ascending_node)
            * DMat3::from_rotation_x(self.inclination)
            * DMat3::from_rotation_z(self.argument_of_periapsis);
        State::new(self.time(), rot * pos, rot * vel, self.origin(), Icrf)
    }
}

pub fn is_equatorial(inclination: f64) -> bool {
    float_eq!(inclination.abs(), 0.0, abs <= 1e-8)
}

pub fn is_circular(eccentricity: f64) -> bool {
    float_eq!(eccentricity, 0.0, abs <= 1e-8)
}

#[cfg(test)]
mod tests {
    use super::*;

    use float_eq::assert_float_eq;
    use lox_bodies::Earth;
    use lox_time::time_scales::Tdb;
    use lox_time::{time, Time};

    #[test]
    fn test_keplerian() {
        let time = time!(Tdb, 2023, 3, 25, 21, 8, 0.0).expect("time should be valid");
        let semi_major = 24464.560;
        let eccentricity = 0.7311;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.44369564302687126;

        let keplerian = Keplerian::new(
            time,
            Earth,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );
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
            keplerian.longitude_of_ascending_node(),
            keplerian1.longitude_of_ascending_node(),
            rel <= 1e-6
        );
        assert_float_eq!(
            keplerian.argument_of_periapsis(),
            keplerian1.argument_of_periapsis(),
            rel <= 1e-6
        );
        assert_float_eq!(
            keplerian.true_anomaly(),
            keplerian1.true_anomaly(),
            rel <= 1e-6
        );
    }
}
