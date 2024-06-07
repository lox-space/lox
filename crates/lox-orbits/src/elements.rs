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
use lox_time::Datetime;

use crate::frames::Icrf;
use crate::origins::CoordinateOrigin;
use crate::states::{State, ToCartesian};

pub trait ToKeplerian<T: Datetime, O: PointMass> {
    fn to_keplerian(&self) -> Keplerian<T, O>;
}

pub struct Keplerian<T: Datetime, O: PointMass> {
    time: T,
    origin: O,
    semi_major: f64,
    eccentricity: f64,
    inclination: f64,
    ascending_node: f64,
    periapsis_argument: f64,
    true_anomaly: f64,
}

impl<T, O> Keplerian<T, O>
where
    T: Datetime,
    O: PointMass,
{
    pub fn new(
        time: T,
        origin: O,
        semi_major: f64,
        eccentricity: f64,
        inclination: f64,
        ascending_node: f64,
        periapsis_argument: f64,
        true_anomaly: f64,
    ) -> Self {
        Self {
            time,
            origin,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_argument,
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

    pub fn periapsis_argument(&self) -> f64 {
        self.periapsis_argument
    }

    pub fn true_anomaly(&self) -> f64 {
        self.true_anomaly
    }

    pub fn semiparameter(&self) -> f64 {
        if is_circular(self.eccentricity) {
            self.semi_major
        } else {
            self.semi_major * (1.0 - self.eccentricity.powi(2))
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

impl<T: Datetime, O: PointMass + Clone> CoordinateOrigin<O> for Keplerian<T, O> {
    fn origin(&self) -> O {
        self.origin.clone()
    }
}

impl<T, O> ToCartesian<T, O, Icrf> for Keplerian<T, O>
where
    T: Datetime + Clone,
    O: PointMass + Clone,
{
    fn to_cartesian(&self) -> State<T, O, Icrf> {
        let (pos, vel) = self.to_perifocal();
        let rot = DMat3::from_rotation_z(self.ascending_node)
            * DMat3::from_rotation_x(self.inclination)
            * DMat3::from_rotation_z(self.periapsis_argument);
        State::new(self.time(), self.origin(), Icrf, rot * pos, rot * vel)
    }
}

pub fn azimuth(v: DVec3) -> f64 {
    v.y.atan2(v.x)
}

pub fn eccentricity_vector(grav_param: f64, pos: DVec3, vel: DVec3) -> DVec3 {
    (pos * (vel.dot(vel) - grav_param / pos.length()) - vel * pos.dot(vel)) / grav_param
}

pub fn is_equatorial(inclination: f64) -> bool {
    float_eq!(inclination.abs(), 0.0, abs <= 1e-8)
}

pub fn is_circular(eccentricity: f64) -> bool {
    float_eq!(eccentricity, 0.0, abs <= 1e-8)
}
