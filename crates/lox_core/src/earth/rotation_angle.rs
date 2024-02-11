/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module rotation_angle exposes functions for calculating the Earth Rotation Angle (ERA).

use crate::bodies::Earth;
use crate::time::intervals::UT1DaysSinceJ2000;
use crate::types::Radians;
use std::f64::consts::TAU;

impl Earth {
    /// Computes the Earth Rotation Angle (ERA) in radians using the IAU 2000 model.
    pub fn rotation_angle_00(t: UT1DaysSinceJ2000) -> Radians {
        let f = t.rem_euclid(1.0); // fractional part of t
        TAU * (f + 0.7790572732640 + 0.00273781191135448 * t) % TAU
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;
    use rstest::rstest;

    #[rstest]
    #[case("before J2000", -123.45, 6.227104062035152)]
    #[case("J2000", 0.0, 4.894961212823756)]
    #[case("after J2000", 123.45, 3.562818363612361)]
    fn test_rotation_angle_00(
        #[case] desc: &str,
        #[case] t: UT1DaysSinceJ2000,
        #[case] expected: Radians,
    ) {
        let actual = Earth::rotation_angle_00(t);
        assert_float_eq!(
            expected,
            actual,
            rel <= 1e-9,
            "{}: expected {}, got {}",
            desc,
            expected,
            actual
        );
    }
}
