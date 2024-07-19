/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module rotation_angle exposes functions for calculating the Earth Rotation Angle (ERA).

use std::f64::consts::TAU;

use lox_bodies::Earth;
use lox_math::types::units::{Days, Radians};

pub trait RotationAngle {
    /// Computes the Earth Rotation Angle (ERA) in radians using the IAU 2000 model.
    fn rotation_angle_00(days_since_j2000_ut1: Days) -> Radians;
}

impl RotationAngle for Earth {
    fn rotation_angle_00(days_since_j2000_ut1: Days) -> Radians {
        let f = days_since_j2000_ut1.rem_euclid(1.0); // fractional part of t
        TAU * (f + 0.7790572732640 + 0.00273781191135448 * days_since_j2000_ut1) % TAU
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::before_j2000(-123.45, 6.227104062035152)]
    #[case::j2000(0.0, 4.894961212823756)]
    #[case::after_j2000(123.45, 3.562818363612361)]
    fn test_rotation_angle_00(#[case] days_since_j2000_ut1: Days, #[case] expected: Radians) {
        let actual = Earth::rotation_angle_00(days_since_j2000_ut1);
        assert_float_eq!(expected, actual, rel <= 1e-9);
    }
}
