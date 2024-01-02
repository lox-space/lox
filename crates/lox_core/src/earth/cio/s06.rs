/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module s06 exposes a function for calculating the Celestial Intermediate Origin (CIO) locator,
//! s, using IAU 2006 precession and IAU 2000A nutation.

mod terms;

use crate::bodies::fundamental::iers03::{
    general_accum_precession_in_longitude_iers03, mean_moon_sun_elongation_iers03,
};
use crate::bodies::{Earth, Moon, Sun, Venus};
use crate::earth::cip::xy06::XY;
use crate::math::arcsec_to_rad;
use crate::time::intervals::TDBJulianCenturiesSinceJ2000;
use crate::types::Radians;
use glam::DVec2;

/// l, l', F, D, Ω, LVe, LE and pA.
type FundamentalArgs = [Radians; 8];

/// Computes the Celestial Intermediate Origin (CIO) locator s, in radians, given the (X, Y)
/// coordinates of the Celestial Intermediate Pole (CIP).
pub fn s(t: TDBJulianCenturiesSinceJ2000, xy: DVec2) -> Radians {
    let fundamental_args = fundamental_args(t);
    let evaluated_terms = evaluate_terms(&fundamental_args);
    let arcsec = fast_polynomial::poly_array(t, &evaluated_terms);
    let radians = arcsec_to_rad(arcsec);
    radians - xy[0] * xy[1] / 2.0
}

fn fundamental_args(t: TDBJulianCenturiesSinceJ2000) -> FundamentalArgs {
    // The output of the CIO calculation is dependent on the ordering of these arguments. DO NOT
    // EDIT.
    [
        Moon.mean_anomaly_iers03(t),
        Sun.mean_anomaly_iers03(t),
        Moon.mean_longitude_minus_ascending_node_mean_longitude_iers03(t),
        mean_moon_sun_elongation_iers03(t),
        Moon.ascending_node_mean_longitude_iers03(t),
        Venus.mean_longitude_iers03(t),
        Earth.mean_longitude_iers03(t),
        general_accum_precession_in_longitude_iers03(t),
    ]
}

fn evaluate_terms(args: &FundamentalArgs) -> [f64; 6] {
    [
        evaluate_single_order_terms(args, terms::COEFFICIENTS[0], &terms::ZERO_ORDER),
        evaluate_single_order_terms(args, terms::COEFFICIENTS[1], &terms::FIRST_ORDER),
        evaluate_single_order_terms(args, terms::COEFFICIENTS[2], &terms::SECOND_ORDER),
        evaluate_single_order_terms(args, terms::COEFFICIENTS[3], &terms::THIRD_ORDER),
        evaluate_single_order_terms(args, terms::COEFFICIENTS[4], &terms::FOURTH_ORDER),
        terms::COEFFICIENTS[5],
    ]
}

fn evaluate_single_order_terms(
    args: &FundamentalArgs,
    term_coefficient: f64,
    terms: &[terms::Term],
) -> f64 {
    terms.iter().rev().fold(term_coefficient, |acc, term| {
        let a = term
            .fundamental_arg_coeffs
            .iter()
            .zip(args)
            .fold(0.0, |acc, (coeff, arg)| acc + coeff * arg);

        acc + term.sin_coeff * a.sin() + term.cos_coeff * a.cos()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::earth::cip::xy06::xy;
    use float_eq::assert_float_eq;

    const TOLERANCE: f64 = 1e-11;

    #[test]
    fn test_s_jd0() {
        let jd0: TDBJulianCenturiesSinceJ2000 = -67.11964407939767;
        let xy = xy(jd0);
        assert_float_eq!(s(jd0, xy), -0.0723985415686306, rel <= TOLERANCE);
    }

    #[test]
    fn test_s_j2000() {
        let j2000: TDBJulianCenturiesSinceJ2000 = 0.0;
        let xy = xy(j2000);
        assert_float_eq!(s(j2000, xy), -0.00000001013396519178, rel <= TOLERANCE);
    }

    #[test]
    fn test_s_j2100() {
        let j2100: TDBJulianCenturiesSinceJ2000 = 1.0;
        let xy = xy(j2100);
        assert_float_eq!(s(j2100, xy), -0.00000000480511934533, rel <= TOLERANCE);
    }

    #[test]
    fn test_fundamental_args_ordering() {
        let j2000: TDBJulianCenturiesSinceJ2000 = 0.0;
        let actual = fundamental_args(j2000);
        let expected = [
            Moon.mean_anomaly_iers03(j2000),
            Sun.mean_anomaly_iers03(j2000),
            Moon.mean_longitude_minus_ascending_node_mean_longitude_iers03(j2000),
            mean_moon_sun_elongation_iers03(j2000),
            Moon.ascending_node_mean_longitude_iers03(j2000),
            Venus.mean_longitude_iers03(j2000),
            Earth.mean_longitude_iers03(j2000),
            general_accum_precession_in_longitude_iers03(j2000),
        ];

        expected.iter().enumerate().for_each(|(i, expected)| {
            assert_float_eq!(*expected, actual[i], rel <= TOLERANCE);
        });
    }
}
