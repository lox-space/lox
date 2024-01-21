/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use float_eq::float_eq;
use glam::{DMat3, DVec3};

use crate::math::{mod_two_pi, normalize_two_pi};
use crate::time::epochs::Epoch;

pub trait TwoBodyState {
    fn time(&self) -> Epoch;
    fn to_cartesian_state(&self, grav_param: f64) -> CartesianState;
    fn to_keplerian_state(&self, grav_param: f64) -> KeplerianState;
}

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
}

impl TwoBodyState for CartesianState {
    fn time(&self) -> Epoch {
        self.time
    }

    fn to_cartesian_state(&self, _grav_param: f64) -> CartesianState {
        *self
    }

    fn to_keplerian_state(&self, grav_param: f64) -> KeplerianState {
        let r = self.position.length();
        let v = self.velocity.length();
        let h = self.position.cross(self.velocity);
        let hm = h.length();
        let node = DVec3::Z.cross(h);
        let e = eccentricity_vector(grav_param, self.position, self.velocity);
        let eccentricity = e.length();
        let inclination = h.angle_between(DVec3::Z);

        let equatorial = is_equatorial(inclination);
        let circular = is_circular(eccentricity);

        let semi_major = if circular {
            hm.powi(2) / grav_param
        } else {
            -grav_param / (2.0 * (v.powi(2) / 2.0 - grav_param / r))
        };

        let ascending_node;
        let periapsis_arg;
        let true_anomaly;
        if equatorial && !circular {
            ascending_node = 0.0;
            periapsis_arg = azimuth(e);
            true_anomaly = (h.dot(e.cross(self.position)) / hm).atan2(self.position.dot(e));
        } else if !equatorial && circular {
            ascending_node = azimuth(node);
            periapsis_arg = 0.0;
            true_anomaly = (self.position.dot(h.cross(node)) / hm).atan2(self.position.dot(node));
        } else if equatorial && circular {
            ascending_node = 0.0;
            periapsis_arg = 0.0;
            true_anomaly = azimuth(self.position);
        } else {
            if semi_major > 0.0 {
                let e_se = self.position.dot(self.velocity) / (grav_param * semi_major).sqrt();
                let e_ce = (r * v.powi(2)) / grav_param - 1.0;
                true_anomaly =
                    crate::coords::anomalies::eccentric_to_true(e_se.atan2(e_ce), eccentricity);
            } else {
                let e_sh = self.position.dot(self.velocity) / (-grav_param * semi_major).sqrt();
                let e_ch = (r * v.powi(2)) / grav_param - 1.0;
                true_anomaly = crate::coords::anomalies::hyperbolic_to_true(
                    ((e_ch + e_sh) / (e_ch - e_sh)).ln() / 2.0,
                    eccentricity,
                );
            }
            ascending_node = azimuth(node);
            let px = self.position.dot(node);
            let py = self.position.dot(h.cross(node)) / hm;
            periapsis_arg = py.atan2(px) - true_anomaly;
        }

        KeplerianState::new(
            self.time,
            semi_major,
            eccentricity,
            inclination,
            mod_two_pi(ascending_node),
            mod_two_pi(periapsis_arg),
            normalize_two_pi(true_anomaly, 0.0),
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct KeplerianState {
    time: Epoch,
    semi_major: f64,
    eccentricity: f64,
    inclination: f64,
    ascending_node: f64,
    periapsis_argument: f64,
    true_anomaly: f64,
}

impl KeplerianState {
    pub fn new(
        time: Epoch,
        semi_major: f64,
        eccentricity: f64,
        inclination: f64,
        ascending_node: f64,
        periapsis_argument: f64,
        true_anomaly: f64,
    ) -> Self {
        Self {
            time,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_argument,
            true_anomaly,
        }
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

    pub fn semi_latus_rectum(&self) -> f64 {
        if float_eq!(self.eccentricity, 0.0, abs <= 1e-3) {
            self.semi_major
        } else {
            self.semi_major * (1.0 - self.eccentricity.powi(2))
        }
    }

    pub fn to_perifocal(&self, grav_param: f64) -> (DVec3, DVec3) {
        let semi_latus = self.semi_latus_rectum();
        let (sin_nu, cos_nu) = self.true_anomaly.sin_cos();
        let sqrt_mu_p = (grav_param / semi_latus).sqrt();

        let pos =
            DVec3::new(cos_nu, sin_nu, 0.0) * (semi_latus / (1.0 + self.eccentricity * cos_nu));
        let vel = DVec3::new(-sin_nu, self.eccentricity + cos_nu, 0.0) * sqrt_mu_p;

        (pos, vel)
    }
}

impl TwoBodyState for KeplerianState {
    fn time(&self) -> Epoch {
        self.time
    }

    fn to_cartesian_state(&self, grav_param: f64) -> CartesianState {
        let (pos, vel) = self.to_perifocal(grav_param);
        let rot = DMat3::from_rotation_z(self.ascending_node)
            * DMat3::from_rotation_x(self.inclination)
            * DMat3::from_rotation_z(self.periapsis_argument);
        CartesianState::new(self.time, rot * pos, rot * vel)
    }

    fn to_keplerian_state(&self, _grav_param: f64) -> KeplerianState {
        *self
    }
}

fn azimuth(v: DVec3) -> f64 {
    v.y.atan2(v.x)
}

fn eccentricity_vector(grav_param: f64, pos: DVec3, vel: DVec3) -> DVec3 {
    (pos * (vel.dot(vel) - grav_param / pos.length()) - vel * pos.dot(vel)) / grav_param
}

fn is_equatorial(inclination: f64) -> bool {
    float_eq!(inclination.abs(), 0.0, abs <= 1e-3)
}

fn is_circular(eccentricity: f64) -> bool {
    float_eq!(eccentricity, 0.0, abs <= 1e-3)
}

#[cfg(test)]
mod tests {
    use crate::time::epochs::TimeScale;
    use float_eq::assert_float_eq;
    use glam::DVec3;

    use super::*;

    #[test]
    fn test_elliptic() {
        let time = Epoch::j2000(TimeScale::TDB);
        let grav_param = 3.9860047e14;
        let semi_major = 24464560.0;
        let eccentricity = 0.7311;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.44369564302687126;
        let pos = DVec3::new(
            -0.107622532467967e7,
            -0.676589636432773e7,
            -0.332308783350379e6,
        );
        let vel = DVec3::new(
            0.935685775154103e4,
            -0.331234775037644e4,
            -0.118801577532701e4,
        );
        let cartesian = CartesianState::new(time, pos, vel);
        let keplerian = KeplerianState::new(
            time,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );

        let cartesian1 = keplerian.to_cartesian_state(grav_param);
        let keplerian1 = cartesian.to_keplerian_state(grav_param);

        assert_float_eq!(pos.x, cartesian1.position.x, rel <= 1e-8);
        assert_float_eq!(pos.y, cartesian1.position.y, rel <= 1e-8);
        assert_float_eq!(pos.z, cartesian1.position.z, rel <= 1e-8);
        assert_float_eq!(vel.x, cartesian1.velocity.x, rel <= 1e-8);
        assert_float_eq!(vel.y, cartesian1.velocity.y, rel <= 1e-8);
        assert_float_eq!(vel.z, cartesian1.velocity.z, rel <= 1e-8);

        assert_float_eq!(semi_major, keplerian1.semi_major, rel <= 1e-8);
        assert_float_eq!(eccentricity, keplerian1.eccentricity, rel <= 1e-8);
        assert_float_eq!(inclination, keplerian1.inclination, rel <= 1e-8);
        assert_float_eq!(ascending_node, keplerian1.ascending_node, rel <= 1e-8);
        assert_float_eq!(periapsis_arg, keplerian1.periapsis_argument, rel <= 1e-8);
        assert_float_eq!(true_anomaly, keplerian1.true_anomaly, rel <= 1e-8);
    }

    #[test]
    fn test_circular() {
        let time = Epoch::j2000(TimeScale::TDB);
        let grav_param = 3.986004418e14;
        let semi_major = 6778136.6;
        let eccentricity = 0.0;
        let inclination = 15f64.to_radians();
        let ascending_node = 20f64.to_radians();
        let periapsis_arg = 0.0;
        let true_anomaly = 30f64.to_radians();
        let pos = DVec3::new(4396398.60746266, 5083838.45333733, 877155.42119322);
        let vel = DVec3::new(-5797.06004014, 4716.60916063, 1718.86034246);
        let cartesian = CartesianState::new(time, pos, vel);
        let keplerian = KeplerianState::new(
            time,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );

        let cartesian1 = keplerian.to_cartesian_state(grav_param);
        let keplerian1 = cartesian.to_keplerian_state(grav_param);

        assert_float_eq!(pos.x, cartesian1.position.x, rel <= 1e-8);
        assert_float_eq!(pos.y, cartesian1.position.y, rel <= 1e-8);
        assert_float_eq!(pos.z, cartesian1.position.z, rel <= 1e-8);
        assert_float_eq!(vel.x, cartesian1.velocity.x, rel <= 1e-8);
        assert_float_eq!(vel.y, cartesian1.velocity.y, rel <= 1e-8);
        assert_float_eq!(vel.z, cartesian1.velocity.z, rel <= 1e-8);

        assert_float_eq!(semi_major, keplerian1.semi_major, rel <= 1e-8);
        assert_float_eq!(eccentricity, keplerian1.eccentricity, abs <= 1e-8);
        assert_float_eq!(inclination, keplerian1.inclination, rel <= 1e-8);
        assert_float_eq!(ascending_node, keplerian1.ascending_node, rel <= 1e-8);
        assert_float_eq!(periapsis_arg, keplerian1.periapsis_argument, rel <= 1e-8);
        assert_float_eq!(true_anomaly, keplerian1.true_anomaly, rel <= 1e-8);
    }

    #[test]
    fn test_circular_orekit() {
        let time = Epoch::j2000(TimeScale::TDB);
        let grav_param = 3.9860047e14;
        let semi_major = 24464560.0;
        let eccentricity = 0.0;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 0.0;
        let true_anomaly = 0.048363;
        let keplerian = KeplerianState::new(
            time,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );

        let keplerian1 = keplerian
            .to_cartesian_state(grav_param)
            .to_keplerian_state(grav_param);

        assert_float_eq!(semi_major, keplerian1.semi_major, rel <= 1e-8);
        assert_float_eq!(eccentricity, keplerian1.eccentricity, abs <= 1e-8);
        assert_float_eq!(inclination, keplerian1.inclination, rel <= 1e-8);
        assert_float_eq!(ascending_node, keplerian1.ascending_node, rel <= 1e-8);
        assert_float_eq!(periapsis_arg, keplerian1.periapsis_argument, rel <= 1e-8);
        assert_float_eq!(true_anomaly, keplerian1.true_anomaly, rel <= 1e-8);
    }

    #[test]
    fn test_hyperbolic_orekit() {
        let time = Epoch::j2000(TimeScale::TDB);
        let grav_param = 3.9860047e14;
        let semi_major = -24464560.0;
        let eccentricity = 1.7311;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.12741601769795755;
        let keplerian = KeplerianState::new(
            time,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );

        let keplerian1 = keplerian
            .to_cartesian_state(grav_param)
            .to_keplerian_state(grav_param);

        assert_float_eq!(semi_major, keplerian1.semi_major, rel <= 1e-8);
        assert_float_eq!(eccentricity, keplerian1.eccentricity, rel <= 1e-8);
        assert_float_eq!(inclination, keplerian1.inclination, rel <= 1e-8);
        assert_float_eq!(ascending_node, keplerian1.ascending_node, rel <= 1e-8);
        assert_float_eq!(periapsis_arg, keplerian1.periapsis_argument, rel <= 1e-8);
        assert_float_eq!(true_anomaly, keplerian1.true_anomaly, rel <= 1e-8);
    }

    #[test]
    fn test_equatorial() {
        let time = Epoch::j2000(TimeScale::TDB);
        let grav_param = 3.9860047e14;
        let semi_major = 24464560.0;
        let eccentricity = 0.7311;
        let inclination = 0.0;
        let ascending_node = 0.0;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.44369564302687126;
        let keplerian = KeplerianState::new(
            time,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );

        let keplerian1 = keplerian
            .to_cartesian_state(grav_param)
            .to_keplerian_state(grav_param);

        assert_float_eq!(semi_major, keplerian1.semi_major, rel <= 1e-8);
        assert_float_eq!(eccentricity, keplerian1.eccentricity, rel <= 1e-8);
        assert_float_eq!(inclination, keplerian1.inclination, rel <= 1e-8);
        assert_float_eq!(ascending_node, keplerian1.ascending_node, rel <= 1e-8);
        assert_float_eq!(periapsis_arg, keplerian1.periapsis_argument, rel <= 1e-8);
        assert_float_eq!(true_anomaly, keplerian1.true_anomaly, rel <= 1e-8);
    }

    #[test]
    fn test_circular_equatorial() {
        let time = Epoch::j2000(TimeScale::TDB);
        let grav_param = 3.9860047e14;
        let semi_major = 24464560.0;
        let eccentricity = 0.0;
        let inclination = 0.0;
        let ascending_node = 0.0;
        let periapsis_arg = 0.0;
        let true_anomaly = 0.44369564302687126;
        let keplerian = KeplerianState::new(
            time,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );

        let keplerian1 = keplerian
            .to_cartesian_state(grav_param)
            .to_keplerian_state(grav_param);

        assert_float_eq!(semi_major, keplerian1.semi_major, rel <= 1e-8);
        assert_float_eq!(eccentricity, keplerian1.eccentricity, abs <= 1e-8);
        assert_float_eq!(inclination, keplerian1.inclination, rel <= 1e-8);
        assert_float_eq!(ascending_node, keplerian1.ascending_node, rel <= 1e-8);
        assert_float_eq!(periapsis_arg, keplerian1.periapsis_argument, rel <= 1e-8);
        assert_float_eq!(true_anomaly, keplerian1.true_anomaly, rel <= 1e-8);
    }
}
