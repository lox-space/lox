/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::types::Radians;
use glam::DMat3;

// TODO: Decide on correct home for this type once module structure discussed with
// Helge. (An array is preferable to a tuple because the CIP calculation requires the dynamic
// access an array provides, and we want to avoid excess allocations moving between steps of the
// pipeline.)
type XY = [f64; 2];

/// The spherical angles E and d.
struct SphericalAngles {
    e: Radians,
    d: Radians,
}

impl SphericalAngles {
    fn new(cip: XY) -> Self {
        let r2 = cip[0] * cip[0] + cip[1] * cip[1];
        let e = cip[1].atan2(cip[0]);
        let d = (r2 / (1.0 - r2)).sqrt().atan();
        Self { e, d }
    }
}

/// Compute the celestial to intermediate-frame-of-date matrix given the CIP (X, Y) coordinates
/// and the CIO locator, s. This matrix is the first step in transforming CRS to
/// TRS coordinates.
///
/// Note that the signs of all angles are reversed relative to ERFA, which uses left-handed
/// coordinates, whereas glam is right-handed.
#[allow(dead_code)] // TODO: Remove this once all module components are actively used.
pub fn celestial_to_intermediate_frame_of_date_matrix(cip: XY, s: Radians) -> DMat3 {
    let spherical_angles = SphericalAngles::new(cip);
    let mut result = DMat3::default();
    result = DMat3::from_rotation_z(-spherical_angles.e) * result;
    result = DMat3::from_rotation_y(-spherical_angles.d) * result;
    DMat3::from_rotation_z(spherical_angles.e + s) * result
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    // TODO: Is this sufficient?
    const TOLERANCE: f64 = 1e-9;

    #[test]
    fn test_celestial_to_intermediate_frame_of_date_matrix_jd0() {
        let cip = [-0.4088355637476968, -0.38359667445777073];
        let s = -0.0723985415686306;
        let expected = [
            0.899981235912944,
            -0.151285348992267,
            -0.408835563747697,
            -0.019051024078611,
            0.923304202214251,
            -0.383596674457771,
            0.435512150790498,
            0.353018545339750,
            0.8280743162060046,
        ];
        let actual = celestial_to_intermediate_frame_of_date_matrix(cip, s).to_cols_array();
        assert_mat3_eq(&expected, &actual)
    }

    #[test]
    fn test_celestial_to_intermediate_frame_of_date_matrix_j2000() {
        let cip = [-0.0000269463795685740, -0.00002800472282281282];
        let s = -0.00000001013396519178;
        let expected = [
            0.999999999636946,
            -0.00000001051127817488,
            -0.000026946379569,
            0.00000000975665225778,
            0.999999999607868,
            -0.000028004722823,
            0.000026946379852,
            0.000028004722550,
            0.999999999244814,
        ];
        let actual = celestial_to_intermediate_frame_of_date_matrix(cip, s).to_cols_array();
        assert_mat3_eq(&expected, &actual)
    }

    #[test]
    fn test_celestial_to_intermediate_frame_of_date_matrix_j2100() {
        let cip = [0.00972070446172924, -0.0000673058699616719];
        let s = -0.00000000480511934533;
        let expected = [
            0.999952752836184,
            0.00000032233307144280,
            0.009720704461729,
            0.00000033194308309287,
            0.999999997734904,
            -0.00006730586996167191,
            -0.009720704461405,
            0.00006730591667081695,
            0.999952750571089,
        ];
        let actual = celestial_to_intermediate_frame_of_date_matrix(cip, s).to_cols_array();
        assert_mat3_eq(&expected, &actual)
    }

    fn assert_mat3_eq(expected: &[f64; 9], actual: &[f64; 9]) {
        for i in 0..9 {
            assert_float_eq!(
                expected[i],
                actual[i],
                abs <= TOLERANCE,
                "actual matrix differed from expected matrix at col {}, row {}:\n\t\
                    expected: {},\n\tactual: {}",
                i / 3,
                i % 3,
                DMat3::from_cols_array(expected),
                DMat3::from_cols_array(actual),
            );
        }
    }
}