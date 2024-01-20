/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use glam::DVec3;

use crate::coords::elements::{cartesian_to_keplerian, keplerian_to_cartesian};
use crate::time::epochs::Epoch;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StateVector(
    pub Epoch,
    pub f64,
    pub f64,
    pub f64,
    pub f64,
    pub f64,
    pub f64,
);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CartesianState {
    time: Epoch,
    position: DVec3,
    velocity: DVec3,
}

impl CartesianState {
    pub fn new(time: Epoch, position: DVec3, velocity: DVec3) -> Self {
        Self {
            time,
            position,
            velocity,
        }
    }

    pub fn time(&self) -> Epoch {
        self.time
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
        KeplerianState::new(
            self.time,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        )
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

impl From<StateVector> for CartesianState {
    fn from(state: StateVector) -> Self {
        let StateVector(time, x, y, z, vx, vy, vz) = state;
        let position = DVec3::new(x, y, z);
        let velocity = DVec3::new(vx, vy, vz);
        Self::new(time, position, velocity)
    }
}

impl From<CartesianState> for [f64; 6] {
    fn from(cartesian: CartesianState) -> Self {
        [
            cartesian.position.x,
            cartesian.position.y,
            cartesian.position.z,
            cartesian.velocity.x,
            cartesian.velocity.y,
            cartesian.velocity.z,
        ]
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct KeplerianState {
    time: Epoch,
    semi_major: f64,
    eccentricity: f64,
    inclination: f64,
    ascending_node: f64,
    periapsis_arg: f64,
    true_anomaly: f64,
}

impl KeplerianState {
    pub fn new(
        time: Epoch,
        semi_major: f64,
        eccentricity: f64,
        inclination: f64,
        ascending_node: f64,
        periapsis_arg: f64,
        true_anomaly: f64,
    ) -> Self {
        Self {
            time,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        }
    }

    pub fn time(&self) -> Epoch {
        self.time
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
        CartesianState::new(self.time, position, velocity)
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

impl From<StateVector> for KeplerianState {
    fn from(state: StateVector) -> Self {
        let StateVector(
            time,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        ) = state;
        Self {
            time,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        }
    }
}

impl From<KeplerianState> for [f64; 6] {
    fn from(keplerian: KeplerianState) -> Self {
        [
            keplerian.semi_major,
            keplerian.eccentricity,
            keplerian.inclination,
            keplerian.ascending_node,
            keplerian.periapsis_arg,
            keplerian.true_anomaly,
        ]
    }
}
