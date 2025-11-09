// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Module s06 exposes a function for calculating the Celestial Intermediate Origin (CIO) locator,
//! s, using IAU 2006 precession and IAU 2000A nutation.

use fast_polynomial::poly_array;
use lox_core::types::units::JulianCenturies;
use lox_core::units::{Angle, AngleUnits};
use lox_time::Time;
use lox_time::julian_dates::JulianDate;
use lox_time::time_scales::Tdb;

use crate::iers::cio::CioLocator;
use crate::iers::cip::CipCoords;
use crate::iers::fundamental::iers03::{
    d_iers03, earth_l_iers03, f_iers03, l_iers03, lp_iers03, omega_iers03, pa_iers03,
    venus_l_iers03,
};

mod terms;

impl CioLocator {
    pub fn iau2006(time: Time<Tdb>, CipCoords { x, y }: CipCoords) -> Self {
        let t = time.centuries_since_j2000();
        let fundamental_args = fundamental_args(t);
        let evaluated_terms = evaluate_terms(&fundamental_args);
        let s = poly_array(t, &evaluated_terms).arcsec();
        CioLocator(s - y.to_radians() / 2.0 * x)
    }
}

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
    let s = poly_array(centuries_since_j2000_tdb, &evaluated_terms).arcsec();
    Angle::radians(s.to_radians() - x.to_radians() * y.to_radians() / 2.0)
}

fn fundamental_args(centuries_since_j2000_tdb: JulianCenturies) -> FundamentalArgs {
    // The output of the CIO calculation is dependent on the ordering of these arguments. DO NOT
    // EDIT.
    [
        l_iers03(centuries_since_j2000_tdb),
        lp_iers03(centuries_since_j2000_tdb),
        f_iers03(centuries_since_j2000_tdb),
        d_iers03(centuries_since_j2000_tdb),
        omega_iers03(centuries_since_j2000_tdb),
        venus_l_iers03(centuries_since_j2000_tdb),
        earth_l_iers03(centuries_since_j2000_tdb),
        pa_iers03(centuries_since_j2000_tdb),
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
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_cio_locator_iau2006() {
        let cip_coords = CipCoords {
            x: 5.791_308_486_706_011e-4.rad(),
            y: 4.020_579_816_732_961e-5.rad(),
        };
        let time = Time::from_two_part_julian_date(Tdb, 2400000.5, 53736.0);
        let exp = CioLocator(-1.220_032_213_076_463e-8.rad());
        let act = CioLocator::iau2006(time, cip_coords);
        assert_approx_eq!(act, exp, atol <= 1e-18);
    }

    #[test]
    fn test_fundamental_args_ordering() {
        let j2000: JulianCenturies = 0.0;
        let act = fundamental_args(j2000);
        let exp = [
            l_iers03(j2000),
            lp_iers03(j2000),
            f_iers03(j2000),
            d_iers03(j2000),
            omega_iers03(j2000),
            venus_l_iers03(j2000),
            earth_l_iers03(j2000),
            pa_iers03(j2000),
        ];

        assert_approx_eq!(act, exp, rtol <= 1e-12)
    }
}
