// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Fundamental astronomical arguments using the Mathews-Herring-Buffett (MHB2000) nutation series.
//!
//! Note that these typically differ from their IERS03 equivalents by less than 0.1 microarcseconds,
//! but are retained as a faithful reproduction of the original model.
//!
//! References:
//! - Mathews, P. M., Herring, T. A., & Buffett, B. A. (2002). Modeling of nutation and precession:
//!   New nutation series for nonrigid Earth and insights into the Earth's interior.
//!   Journal of Geophysical Research, 107(B4).

use lox_core::types::units::JulianCenturies;
use lox_core::units::{Angle, AngleUnits};

/// Mean elongation of the Moon from the Sun (D) for luni-solar nutation.
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// The mean elongation D in radians, normalized to [-π, π]
#[inline]
pub fn d_mhb2000_luni_solar(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[
            1072260.70369,
            1602961601.2090,
            -6.3706,
            0.006593,
            -0.00003169,
        ],
    ))
}

/// Mean elongation of the Moon from the Sun (D) for planetary nutation.
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// The mean elongation D in radians, normalized to [-π, π]
#[inline]
pub fn d_mhb2000_planetary(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    fast_polynomial::poly_array(centuries_since_j2000_tdb, &[5.198466741, 7771.3771468121])
        .rad()
        .mod_two_pi_signed()
}

/// Sun's mean anomaly (l').
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// The Sun's mean anomaly l' in radians, normalized to [-π, π]
#[inline]
pub fn lp_mhb2000(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[
            1287104.79305,
            129596581.0481,
            -0.5532,
            0.000136,
            -0.00001149,
        ],
    ))
}

/// Moon's mean anomaly (l).
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// The Moon's mean anomaly l in radians, normalized to [-π, π]
#[inline]
pub fn l_mhb2000(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    fast_polynomial::poly_array(centuries_since_j2000_tdb, &[2.35555598, 8328.6914269554])
        .rad()
        .mod_two_pi_signed()
}

/// Moon's mean argument of latitude (F).
///
/// This is the mean longitude of the Moon minus the mean longitude of the
/// Moon's ascending node.
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// The Moon's mean argument of latitude F in radians, normalized to [-π, π]
#[inline]
pub fn f_mhb2000(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    fast_polynomial::poly_array(centuries_since_j2000_tdb, &[1.627905234, 8433.466158131])
        .rad()
        .mod_two_pi_signed()
}

/// Mean longitude of the Moon's ascending node (Ω).
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// The mean longitude of the Moon's ascending node Ω in radians, normalized to [-π, π]
#[inline]
pub fn omega_mhb2000(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    fast_polynomial::poly_array(centuries_since_j2000_tdb, &[2.18243920, -33.757045])
        .rad()
        .mod_two_pi_signed()
}

/// Neptune's mean longitude (L_N).
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// Neptune's mean longitude in radians, normalized to [-π, π]
#[inline]
pub fn neptune_l_mhb2000(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    fast_polynomial::poly_array(centuries_since_j2000_tdb, &[5.3211590, 3.81277740])
        .rad()
        .mod_two_pi_signed()
}

#[cfg(test)]
mod tests {

    use lox_core::types::units::JulianCenturies;
    use lox_test_utils::assert_approx_eq;

    use super::*;

    // Note that all expected values are outputs from the equivalent ERFA functions.

    // Relative error tolerance for float_eq assertions.
    // This is somewhat loose, being based on observations of how closely our implementations
    // match ERFA outputs rather than any target tolerance.
    // See https://github.com/lox-space/lox/pull/23#discussion_r1398485509
    const TOLERANCE: f64 = 1e-11;

    // Test cases for t.
    const T_ZERO: JulianCenturies = 0.0;
    const T_POSITIVE: JulianCenturies = 1.23456789;
    const T_NEGATIVE: JulianCenturies = -1.23456789;

    #[test]
    fn test_d_luni_solar() {
        assert_approx_eq!(
            d_mhb2000_luni_solar(T_ZERO),
            5.198466588650503.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            d_mhb2000_luni_solar(T_POSITIVE),
            5.067140540624282.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            d_mhb2000_luni_solar(T_NEGATIVE),
            -0.953486820095515.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_d_planetary() {
        assert_approx_eq!(
            d_mhb2000_planetary(T_ZERO),
            5.1984667410.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            d_mhb2000_planetary(T_POSITIVE),
            5.06718921180569.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            d_mhb2000_planetary(T_NEGATIVE),
            -0.953441036985836.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_lp() {
        assert_approx_eq!(
            lp_mhb2000(T_ZERO),
            6.24006012692298.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            lp_mhb2000(T_POSITIVE),
            2.806497028816457.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            lp_mhb2000(T_NEGATIVE),
            -2.892755565138653.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_l() {
        assert_approx_eq!(l_mhb2000(T_ZERO), 2.35555598.rad(), rtol <= TOLERANCE);
        assert_approx_eq!(
            l_mhb2000(T_POSITIVE),
            5.399394871613055.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            l_mhb2000(T_NEGATIVE),
            -0.688282911613584.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_f() {
        assert_approx_eq!(f_mhb2000(T_ZERO), 1.627905234.rad(), rtol <= TOLERANCE);
        assert_approx_eq!(
            f_mhb2000(T_POSITIVE),
            2.07637146761946.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            f_mhb2000(T_NEGATIVE),
            -5.103746306797973.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_omega() {
        assert_approx_eq!(omega_mhb2000(T_ZERO), 2.18243920.rad(), rtol <= TOLERANCE);
        assert_approx_eq!(
            omega_mhb2000(T_POSITIVE),
            -1.793812775207527.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            omega_mhb2000(T_NEGATIVE),
            6.15869117520753.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_neptune_l() {
        assert_approx_eq!(
            neptune_l_mhb2000(T_ZERO),
            5.3211590.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            neptune_l_mhb2000(T_POSITIVE),
            3.7451062425781.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            neptune_l_mhb2000(T_NEGATIVE),
            0.614026450242314.rad(),
            rtol <= TOLERANCE
        );
    }
}
