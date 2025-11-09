// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Fundamental astronomical arguments following IERS Conventions (2003).
//!
//! References:
//! - IERS Conventions (2003), IERS Technical Note No. 32

use lox_core::types::units::JulianCenturies;
use lox_core::units::{Angle, AngleUnits};

/// General accumulated precession in longitude (p_A).
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// The general accumulated precession in longitude in radians
#[inline]
pub fn pa_iers03(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[0.0, 0.024381750, 0.00000538691],
    )
    .rad()
}

/// Mean elongation of the Moon from the Sun (D).
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// The mean elongation D in radians, normalized to [-π, π]
#[inline]
pub fn d_iers03(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[
            1072260.703692,
            1602961601.2090,
            -6.3706,
            0.006593,
            -0.00003169,
        ],
    ))
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
pub fn lp_iers03(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[
            1287104.793048,
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
pub fn l_iers03(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[
            485868.249036,
            1717915923.2178,
            31.8792,
            0.051635,
            -0.00024470,
        ],
    ))
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
pub fn f_iers03(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[
            335779.526232,
            1739527262.8478,
            -12.7512,
            -0.001037,
            0.00000417,
        ],
    ))
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
pub fn omega_iers03(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[450160.398036, -6962890.5431, 7.4722, 0.007702, -0.00005939],
    ))
}

/// Mercury's mean longitude (L_Me).
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// Mercury's mean longitude in radians, normalized to [-π, π]
#[inline]
pub fn mercury_l_iers03(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    (4.402608842 + 2608.7903141574 * centuries_since_j2000_tdb)
        .rad()
        .mod_two_pi_signed()
}

/// Venus's mean longitude (L_V).
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// Venus's mean longitude in radians, normalized to [-π, π]
#[inline]
pub fn venus_l_iers03(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    (3.176146697 + 1021.3285546211 * centuries_since_j2000_tdb)
        .rad()
        .mod_two_pi_signed()
}

/// Earth's mean longitude (L_E).
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// Earth's mean longitude in radians, normalized to [-π, π]
#[inline]
pub fn earth_l_iers03(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    (1.753470314 + 628.3075849991 * centuries_since_j2000_tdb)
        .rad()
        .mod_two_pi_signed()
}

/// Mars's mean longitude (L_Ma).
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// Mars's mean longitude in radians, normalized to [-π, π]
#[inline]
pub fn mars_l_iers03(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    (6.203480913 + 334.0612426700 * centuries_since_j2000_tdb)
        .rad()
        .mod_two_pi_signed()
}

/// Jupiter's mean longitude (L_J).
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// Jupiter's mean longitude in radians, normalized to [-π, π]
#[inline]
pub fn jupiter_l_iers03(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    (0.599546497 + 52.9690962641 * centuries_since_j2000_tdb)
        .rad()
        .mod_two_pi_signed()
}

/// Saturn's mean longitude (L_S).
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// Saturn's mean longitude in radians, normalized to [-π, π]
#[inline]
pub fn saturn_l_iers03(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    (0.874016757 + 21.3299104960 * centuries_since_j2000_tdb)
        .rad()
        .mod_two_pi_signed()
}

/// Uranus's mean longitude (L_U).
///
/// # Arguments
///
/// * `centuries_since_j2000_tdb` - Time in Julian centuries since J2000.0 TDB
///
/// # Returns
///
/// Uranus's mean longitude in radians, normalized to [-π, π]
#[inline]
pub fn uranus_l_iers03(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    (5.481293872 + 7.4781598567 * centuries_since_j2000_tdb)
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
pub fn neptune_l_iers03(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    (5.311886287 + 3.8133035638 * centuries_since_j2000_tdb)
        .rad()
        .mod_two_pi_signed()
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
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
    fn test_pa() {
        assert_approx_eq!(pa_iers03(T_ZERO), 0.0.rad(), atol <= TOLERANCE);
        assert_approx_eq!(
            pa_iers03(T_POSITIVE),
            0.030109136153306.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            pa_iers03(T_NEGATIVE),
            -0.030092715150709.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_d() {
        assert_approx_eq!(d_iers03(T_ZERO), 5.198466588660199.rad(), rtol <= TOLERANCE);
        assert_approx_eq!(
            d_iers03(T_POSITIVE),
            5.067140540634685.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            d_iers03(T_NEGATIVE),
            -0.953486820085112.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_lp() {
        assert_approx_eq!(
            lp_iers03(T_ZERO),
            6.240060126913284.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            lp_iers03(T_POSITIVE),
            2.806497028806777.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            lp_iers03(T_NEGATIVE),
            -2.892755565148333.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_l() {
        assert_approx_eq!(l_iers03(T_ZERO), 2.355555743493879.rad(), rtol <= TOLERANCE);
        assert_approx_eq!(
            l_iers03(T_POSITIVE),
            5.399629142881749.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            l_iers03(T_NEGATIVE),
            -0.688046529809469.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_f() {
        assert_approx_eq!(f_iers03(T_ZERO), 1.627905081537519.rad(), rtol <= TOLERANCE);
        assert_approx_eq!(
            f_iers03(T_POSITIVE),
            2.076275583431815.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            f_iers03(T_NEGATIVE),
            -5.103839172987284.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_omega() {
        assert_approx_eq!(
            omega_iers03(T_ZERO),
            2.182439196615671.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            omega_iers03(T_POSITIVE),
            -1.793758671799353.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            omega_iers03(T_NEGATIVE),
            6.158747492734907.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_mercury_l() {
        assert_approx_eq!(
            mercury_l_iers03(T_ZERO),
            4.402608842.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            mercury_l_iers03(T_POSITIVE),
            1.857299860610716.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            mercury_l_iers03(T_NEGATIVE),
            -5.618452790969762.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_venus_l() {
        assert_approx_eq!(venus_l_iers03(T_ZERO), 3.176146697.rad(), rtol <= TOLERANCE);
        assert_approx_eq!(
            venus_l_iers03(T_POSITIVE),
            1.155338629224197.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            venus_l_iers03(T_NEGATIVE),
            -1.086230542403939.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_earth_l() {
        assert_approx_eq!(earth_l_iers03(T_ZERO), 1.753470314.rad(), rtol <= TOLERANCE);
        assert_approx_eq!(
            earth_l_iers03(T_POSITIVE),
            4.610047014245303.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            earth_l_iers03(T_NEGATIVE),
            -1.103106386245365.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_mars_l() {
        assert_approx_eq!(mars_l_iers03(T_ZERO), 6.203480913.rad(), rtol <= TOLERANCE);
        assert_approx_eq!(
            mars_l_iers03(T_POSITIVE),
            3.934534133027128.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            mars_l_iers03(T_NEGATIVE),
            -4.093942921386315.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_jupiter_l() {
        assert_approx_eq!(
            jupiter_l_iers03(T_ZERO),
            0.599546497.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            jupiter_l_iers03(T_POSITIVE),
            3.161638835180952.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            jupiter_l_iers03(T_NEGATIVE),
            -1.962545841180955.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_saturn_l() {
        assert_approx_eq!(
            saturn_l_iers03(T_ZERO),
            0.874016757.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            saturn_l_iers03(T_POSITIVE),
            2.074498123217225.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            saturn_l_iers03(T_NEGATIVE),
            -0.326464609217226.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_uranus_l() {
        assert_approx_eq!(
            uranus_l_iers03(T_ZERO),
            5.481293872.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            uranus_l_iers03(T_POSITIVE),
            2.147219293009648.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            uranus_l_iers03(T_NEGATIVE),
            -3.75100216336882.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_neptune_l() {
        assert_approx_eq!(
            neptune_l_iers03(T_ZERO),
            5.311886287.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            neptune_l_iers03(T_POSITIVE),
            3.73648311451046.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            neptune_l_iers03(T_NEGATIVE),
            0.604104152309954.rad(),
            rtol <= TOLERANCE
        );
    }
}
