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

    #[test]
    fn test_rotation_angle_00() {
        struct TestCase {
            t: UT1DaysSinceJ2000,
            expected: Radians,
        }

        [
            TestCase {
                t: -123.45,
                expected: 6.227104062035152,
            },
            TestCase {
                t: 0.0,
                expected: 4.894961212823756,
            },
            TestCase {
                t: 123.45,
                expected: 3.562818363612361,
            },
        ]
        .iter()
        .for_each(|tc| {
            let actual = Earth::rotation_angle_00(tc.t);
            assert_float_eq!(tc.expected, actual, rel <= 1e-9);
        });
    }
}
