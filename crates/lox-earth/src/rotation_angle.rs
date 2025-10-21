/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module rotation_angle exposes functions for calculating the Earth Rotation Angle (ERA).

use std::f64::consts::TAU;

use lox_units::{Angle, AngleUnits, types::units::Days};

pub fn earth_rotation_angle_00(days_since_j2000_ut1: Days) -> Angle {
    let f = days_since_j2000_ut1.rem_euclid(1.0); // fractional part of t
    let era = (TAU * (f + 0.7790572732640 + 0.00273781191135448 * days_since_j2000_ut1)).rad();
    era.mod_two_pi()
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::before_j2000(-123.45, 6.227104062035152.rad())]
    #[case::j2000(0.0, 4.894961212823756.rad())]
    #[case::after_j2000(123.45, 3.562818363612361.rad())]
    fn test_rotation_angle_00(#[case] days_since_j2000_ut1: Days, #[case] expected: Angle) {
        let actual = earth_rotation_angle_00(days_since_j2000_ut1);
        assert_approx_eq!(expected, actual, rtol <= 1e-9);
    }
}
