// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::f64::consts::{FRAC_PI_2, TAU};

use glam::{DMat3, DVec3};

use crate::rotations::Rotation;

pub fn icrf_to_iau(
    (right_ascension, declination, rotation_angle): (f64, f64, f64),
    (right_ascension_rate, declination_rate, rotation_rate): (f64, f64, f64),
) -> Rotation {
    let m1 = DMat3::from_rotation_z(-(right_ascension + FRAC_PI_2));
    let m2 = DMat3::from_rotation_x(-(FRAC_PI_2 - declination));
    let m3 = DMat3::from_rotation_z(-(rotation_angle % TAU));
    let m = m3 * m2 * m1;
    let v = DVec3::new(right_ascension_rate, -declination_rate, rotation_rate);
    Rotation::new(m).with_angular_velocity(v)
}
