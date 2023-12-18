/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::types::Radians;
use glam::DMat3;

// TODO: Decide on correct type once package structure discussed with Helge.
type XY = (f64, f64);

// The spherical angles E and d.
struct SphericalAngles {
    e: Radians,
    d: Radians,
}

impl SphericalAngles {
    fn new(cip: XY) -> Self {
        let r2 = cip.0 * cip.0 + cip.1 * cip.1;
        let e = cip.0.atan2(cip.1);
        let d = (r2 / (1.0 - r2)).sqrt().atan();
        Self { e, d }
    }
}

/// Compute the celestial to intermediate-frame-of-date matrix given the CIP (X, Y) coordinates
/// and the CIO locator, s. This matrix is the first step in transforming CRS to
/// TRS coordinates.
pub fn celestial_to_intermediate_frame_of_date_matrix(cip: XY, s: Radians) -> DMat3 {
    let spherical_angles = SphericalAngles::new(cip);
    let mut result = DMat3::default();
    result = DMat3::from_rotation_z(spherical_angles.e) * result;
    result = DMat3::from_rotation_y(-spherical_angles.d) * result;
    DMat3::from_rotation_z(-(spherical_angles.e + s)) * result
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    const TOLERANCE: f64 = 1e-11;

    #[test]
    fn test_celestial_to_intermediate_frame_of_date_matrix_jd0() {
        let mut cip = (-0.4088355637476968, -0.38359667445777073);
        let mut s = -0.0723985415686306;
        let mut expected = &[
            0.899981235912944,
            -0.151285348992267,
            -0.408835563747697,
            -0.019051024078611,
            0.923304202214251,
            -0.383596674457771,
            0.435512150790498,
            0.353018545339750,
            0.828074316206005,
        ];
        let mut actual = celestial_to_intermediate_frame_of_date_matrix(cip, s).to_cols_array();
        for i in 0..9 {
            assert_float_eq!(
                expected[i],
                actual[i],
                rel <= TOLERANCE,
                "\nexpected:\t{}\nactual:\t{}",
                DMat3::from_cols_array(&actual),
                DMat3::from_cols_array(&expected),
            );
        }
    }
}
