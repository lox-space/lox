// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Module s06 exposes a function for calculating the Celestial Intermediate Origin (CIO) locator,
//! s, using IAU 2006 precession and IAU 2000A nutation.

use lox_bodies::fundamental::iers03::{
    general_accum_precession_in_longitude_iers03, mean_moon_sun_elongation_iers03,
};
use lox_bodies::{Earth, Moon, Sun, Venus};
use lox_units::types::units::JulianCenturies;
use lox_units::{Angle, AngleUnits};

use crate::cip::xy06::CipCoords;

mod terms;

/// l, l', F, D, Ω, LVe, LE and pA.
type FundamentalArgs = [Angle; 8];

/// Computes the Celestial Intermediate Origin (CIO) locator s, in radians, given the (X, Y)
/// coordinates of the Celestial Intermediate Pole (CIP).
pub fn cio_locator(
    centuries_since_j2000_tdb: JulianCenturies,
    CipCoords { x, y }: CipCoords,
) -> Angle {
    let fundamental_args = fundamental_args(centuries_since_j2000_tdb);
    let evaluated_terms = evaluate_terms(&fundamental_args);
    let s = fast_polynomial::poly_array(centuries_since_j2000_tdb, &evaluated_terms).arcsec();
    Angle::radians(s.to_radians() - x.to_radians() * y.to_radians() / 2.0)
}

fn fundamental_args(centuries_since_j2000_tdb: JulianCenturies) -> FundamentalArgs {
    // The output of the CIO calculation is dependent on the ordering of these arguments. DO NOT
    // EDIT.
    [
        Moon.mean_anomaly_iers03(centuries_since_j2000_tdb),
        Sun.mean_anomaly_iers03(centuries_since_j2000_tdb),
        Moon.mean_longitude_minus_ascending_node_mean_longitude_iers03(centuries_since_j2000_tdb),
        mean_moon_sun_elongation_iers03(centuries_since_j2000_tdb),
        Moon.ascending_node_mean_longitude_iers03(centuries_since_j2000_tdb),
        Venus.mean_longitude_iers03(centuries_since_j2000_tdb),
        Earth.mean_longitude_iers03(centuries_since_j2000_tdb),
        general_accum_precession_in_longitude_iers03(centuries_since_j2000_tdb),
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
            .fold(0.0.rad(), |acc, (&coeff, &arg)| acc + coeff * arg);

        acc + term.sin_coeff * a.sin() + term.cos_coeff * a.cos()
    })
}

#[cfg(test)]
mod tests {
    use std::iter::zip;

    use lox_test_utils::assert_approx_eq;

    use super::*;

    const TOLERANCE: f64 = 1e-11;

    #[test]
    fn test_s_jd0() {
        let jd0: JulianCenturies = -67.11964407939767;
        let xy = CipCoords::new(jd0);
        assert_approx_eq!(
            cio_locator(jd0, xy),
            -0.0723985415686306.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_s_j2000() {
        let j2000: JulianCenturies = 0.0;
        let xy = CipCoords::new(j2000);
        assert_approx_eq!(
            cio_locator(j2000, xy),
            -0.00000001013396519178.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_s_j2100() {
        let j2100: JulianCenturies = 1.0;
        let xy = CipCoords::new(j2100);
        assert_approx_eq!(
            cio_locator(j2100, xy),
            -0.00000000480511934533.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_fundamental_args_ordering() {
        let j2000: JulianCenturies = 0.0;
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

        for (act, exp) in zip(actual, expected) {
            assert_approx_eq!(act, exp, rtol <= TOLERANCE)
        }
    }
}
