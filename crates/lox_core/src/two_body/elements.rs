/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use float_eq::float_eq;
use glam::{DMat3, DVec3};
use std::f64::consts::PI;

pub fn keplerian_to_perifocal(
    semi_latus: f64,
    eccentricity: f64,
    true_anomaly: f64,
    grav_param: f64,
) -> (DVec3, DVec3) {
    let (sin_nu, cos_nu) = true_anomaly.sin_cos();
    let sqrt_mu_p = (grav_param / semi_latus).sqrt();

    let pos = DVec3::new(cos_nu, sin_nu, 0.0) * (semi_latus / (1.0 + eccentricity * cos_nu));
    let vel = DVec3::new(-sin_nu, eccentricity + cos_nu, 0.0) * sqrt_mu_p;

    (pos, vel)
}

pub fn keplerian_to_cartesian(
    grav_param: f64,
    semi_major: f64,
    eccentricity: f64,
    inclination: f64,
    ascending_node: f64,
    periapsis_arg: f64,
    true_anomaly: f64,
) -> (DVec3, DVec3) {
    let semi_latus = semi_latus_rectum(semi_major, eccentricity);
    let (pos, vel) = keplerian_to_perifocal(semi_latus, eccentricity, true_anomaly, grav_param);
    let rot = DMat3::from_rotation_z(ascending_node)
        * DMat3::from_rotation_x(inclination)
        * DMat3::from_rotation_z(periapsis_arg);
    (rot * pos, rot * vel)
}

pub fn semi_latus_rectum(semi_major: f64, eccentricity: f64) -> f64 {
    if float_eq!(eccentricity, 0.0, abs <= 1e-3) {
        semi_major
    } else {
        semi_major * (1.0 - eccentricity.powi(2))
    }
}

type Keplerian = (f64, f64, f64, f64, f64, f64);

pub fn cartesian_to_keplerian(grav_param: f64, pos: DVec3, vel: DVec3) -> Keplerian {
    let r = pos.length();
    let v = vel.length();
    let h = pos.cross(vel);
    let hm = h.length();
    let node = DVec3::Z.cross(h);
    let e = eccentricity_vector(grav_param, pos, vel);
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
        true_anomaly = (h.dot(e.cross(pos)) / hm).atan2(pos.dot(e));
    } else if !equatorial && circular {
        ascending_node = azimuth(node);
        periapsis_arg = 0.0;
        true_anomaly = (pos.dot(h.cross(node)) / hm).atan2(pos.dot(node));
    } else if equatorial && circular {
        ascending_node = 0.0;
        periapsis_arg = 0.0;
        true_anomaly = azimuth(pos);
    } else {
        if semi_major > 0.0 {
            let e_se = pos.dot(vel) / (grav_param * semi_major).sqrt();
            let e_ce = (r * v.powi(2)) / grav_param - 1.0;
            true_anomaly = eccentric_to_true(e_se.atan2(e_ce), eccentricity);
        } else {
            let e_sh = pos.dot(vel) / (-grav_param * semi_major).sqrt();
            let e_ch = (r * v.powi(2)) / grav_param - 1.0;
            true_anomaly =
                hyperbolic_to_true(((e_ch + e_sh) / (e_ch - e_sh)).ln() / 2.0, eccentricity);
        }
        ascending_node = azimuth(node);
        let px = pos.dot(node);
        let py = pos.dot(h.cross(node)) / hm;
        periapsis_arg = py.atan2(px) - true_anomaly;
    }

    (
        semi_major,
        eccentricity,
        inclination,
        mod_two_pi(ascending_node),
        mod_two_pi(periapsis_arg),
        normalize_two_pi(true_anomaly, 0.0),
    )
}

fn normalize_two_pi(a: f64, center: f64) -> f64 {
    a - 2.0 * PI * ((a + PI - center) / (2.0 * PI)).floor()
}

fn mod_two_pi(a: f64) -> f64 {
    let w = a % (2.0 * PI);
    if w < 0.0 {
        w + 2.0 * PI
    } else {
        w
    }
}

fn hyperbolic_to_true(e: f64, ecc: f64) -> f64 {
    2.0 * (((1.0 + ecc) / (ecc - 1.0)).sqrt() * (e / 2.0).tanh()).atan()
}

fn eccentric_to_true(e: f64, ecc: f64) -> f64 {
    2.0 * (((1.0 + ecc) / (1.0 - ecc)).sqrt() * (e / 2.0).tan()).atan()
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
    use super::*;
    use float_eq::assert_float_eq;
    use glam::DVec3;

    #[test]
    fn test_perifocal() {
        let semi_latus = 1.13880762905224e7;
        let eccentricity = 0.7311;
        let true_anomaly = 0.44369564302687126;
        let grav_param = 3.9860047e14;

        let pos = DVec3::new(6194863.12535486, 2944437.90016286, 0.0);
        let vel = DVec3::new(-2539.71254827, 9668.69568539, 0.0);
        let (pos1, vel1) =
            keplerian_to_perifocal(semi_latus, eccentricity, true_anomaly, grav_param);

        assert_float_eq!(pos.x, pos1.x, rel <= 1e-8);
        assert_float_eq!(pos.y, pos1.y, rel <= 1e-8);
        assert_float_eq!(pos.z, pos1.z, rel <= 1e-8);
        assert_float_eq!(vel.x, vel1.x, rel <= 1e-8);
        assert_float_eq!(vel.y, vel1.y, rel <= 1e-8);
        assert_float_eq!(vel.z, vel1.z, rel <= 1e-8);
    }

    #[test]
    fn test_elliptic() {
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

        let (
            semi_major1,
            eccentricity1,
            inclination1,
            ascending_node1,
            periapsis_arg1,
            true_anomaly1,
        ) = cartesian_to_keplerian(grav_param, pos, vel);
        let (pos1, vel1) = keplerian_to_cartesian(
            grav_param,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );

        assert_float_eq!(pos.x, pos1.x, rel <= 1e-8);
        assert_float_eq!(pos.y, pos1.y, rel <= 1e-8);
        assert_float_eq!(pos.z, pos1.z, rel <= 1e-8);
        assert_float_eq!(vel.x, vel1.x, rel <= 1e-8);
        assert_float_eq!(vel.y, vel1.y, rel <= 1e-8);
        assert_float_eq!(vel.z, vel1.z, rel <= 1e-8);

        assert_float_eq!(semi_major, semi_major1, rel <= 1e-8);
        assert_float_eq!(eccentricity, eccentricity1, rel <= 1e-8);
        assert_float_eq!(inclination, inclination1, rel <= 1e-8);
        assert_float_eq!(ascending_node, ascending_node1, rel <= 1e-8);
        assert_float_eq!(periapsis_arg, periapsis_arg1, rel <= 1e-8);
        assert_float_eq!(true_anomaly, true_anomaly1, rel <= 1e-8);
    }

    #[test]
    fn test_circular() {
        let grav_param = 3.986004418e14;
        let semi_major = 6778136.6;
        let eccentricity = 0.0;
        let inclination = 15f64.to_radians();
        let ascending_node = 20f64.to_radians();
        let periapsis_arg = 0.0;
        let true_anomaly = 30f64.to_radians();
        let pos = DVec3::new(4396398.60746266, 5083838.45333733, 877155.42119322);
        let vel = DVec3::new(-5797.06004014, 4716.60916063, 1718.86034246);

        let (
            semi_major1,
            eccentricity1,
            inclination1,
            ascending_node1,
            periapsis_arg1,
            true_anomaly1,
        ) = cartesian_to_keplerian(grav_param, pos, vel);
        let (pos1, vel1) = keplerian_to_cartesian(
            grav_param,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );

        assert_float_eq!(pos.x, pos1.x, rel <= 1e-8);
        assert_float_eq!(pos.y, pos1.y, rel <= 1e-8);
        assert_float_eq!(pos.z, pos1.z, rel <= 1e-8);
        assert_float_eq!(vel.x, vel1.x, rel <= 1e-4);
        assert_float_eq!(vel.y, vel1.y, rel <= 1e-4);
        assert_float_eq!(vel.z, vel1.z, rel <= 1e-4);

        assert_float_eq!(semi_major, semi_major1, rel <= 1e-8);
        assert_float_eq!(eccentricity, eccentricity1, abs <= 1e-8);
        assert_float_eq!(inclination, inclination1, rel <= 1e-8);
        assert_float_eq!(ascending_node, ascending_node1, rel <= 1e-8);
        assert_float_eq!(periapsis_arg, periapsis_arg1, rel <= 1e-8);
        assert_float_eq!(true_anomaly, true_anomaly1, rel <= 1e-8);
    }

    #[test]
    fn test_circular_orekit() {
        let grav_param = 3.9860047e14;
        let semi_major = 24464560.0;
        let eccentricity = 0.0;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 0.0;
        let true_anomaly = 0.048363;

        let (pos, vel) = keplerian_to_cartesian(
            grav_param,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );

        let (
            semi_major1,
            eccentricity1,
            inclination1,
            ascending_node1,
            periapsis_arg1,
            true_anomaly1,
        ) = cartesian_to_keplerian(grav_param, pos, vel);

        assert_float_eq!(semi_major, semi_major1, rel <= 1e-8);
        assert_float_eq!(eccentricity, eccentricity1, abs <= 1e-8);
        assert_float_eq!(inclination, inclination1, rel <= 1e-8);
        assert_float_eq!(ascending_node, ascending_node1, rel <= 1e-8);
        assert_float_eq!(periapsis_arg, periapsis_arg1, rel <= 1e-8);
        assert_float_eq!(true_anomaly, true_anomaly1, rel <= 1e-8);
    }

    #[test]
    fn test_hyperbolic_orekit() {
        let grav_param = 3.9860047e14;
        let semi_major = -24464560.0;
        let eccentricity = 1.7311;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.12741601769795755;

        let (pos, vel) = keplerian_to_cartesian(
            grav_param,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );

        let (
            semi_major1,
            eccentricity1,
            inclination1,
            ascending_node1,
            periapsis_arg1,
            true_anomaly1,
        ) = cartesian_to_keplerian(grav_param, pos, vel);

        assert_float_eq!(semi_major, semi_major1, rel <= 1e-8);
        assert_float_eq!(eccentricity, eccentricity1, abs <= 1e-8);
        assert_float_eq!(inclination, inclination1, rel <= 1e-8);
        assert_float_eq!(ascending_node, ascending_node1, rel <= 1e-8);
        assert_float_eq!(periapsis_arg, periapsis_arg1, rel <= 1e-8);
        assert_float_eq!(true_anomaly, true_anomaly1, rel <= 1e-8);
    }

    #[test]
    fn test_equatorial() {
        let grav_param = 3.9860047e14;
        let semi_major = 24464560.0;
        let eccentricity = 0.7311;
        let inclination = 0.0;
        let ascending_node = 0.0;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.44369564302687126;

        let (pos, vel) = keplerian_to_cartesian(
            grav_param,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );

        let (
            semi_major1,
            eccentricity1,
            inclination1,
            ascending_node1,
            periapsis_arg1,
            true_anomaly1,
        ) = cartesian_to_keplerian(grav_param, pos, vel);

        assert_float_eq!(semi_major, semi_major1, rel <= 1e-8);
        assert_float_eq!(eccentricity, eccentricity1, abs <= 1e-8);
        assert_float_eq!(inclination, inclination1, rel <= 1e-8);
        assert_float_eq!(ascending_node, ascending_node1, rel <= 1e-8);
        assert_float_eq!(periapsis_arg, periapsis_arg1, rel <= 1e-8);
        assert_float_eq!(true_anomaly, true_anomaly1, rel <= 1e-8);
    }

    #[test]
    fn test_circular_equatorial() {
        let grav_param = 3.9860047e14;
        let semi_major = 24464560.0;
        let eccentricity = 0.0;
        let inclination = 0.0;
        let ascending_node = 0.0;
        let periapsis_arg = 0.0;
        let true_anomaly = 0.44369564302687126;

        let (pos, vel) = keplerian_to_cartesian(
            grav_param,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );

        let (
            semi_major1,
            eccentricity1,
            inclination1,
            ascending_node1,
            periapsis_arg1,
            true_anomaly1,
        ) = cartesian_to_keplerian(grav_param, pos, vel);

        assert_float_eq!(semi_major, semi_major1, rel <= 1e-8);
        assert_float_eq!(eccentricity, eccentricity1, abs <= 1e-8);
        assert_float_eq!(inclination, inclination1, rel <= 1e-8);
        assert_float_eq!(ascending_node, ascending_node1, rel <= 1e-8);
        assert_float_eq!(periapsis_arg, periapsis_arg1, rel <= 1e-8);
        assert_float_eq!(true_anomaly, true_anomaly1, rel <= 1e-8);
    }
}
