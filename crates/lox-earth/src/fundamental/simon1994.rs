// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Fundamental astronomical arguments as proposed by Simon et al. (1994).
//!
//! References:
//! - Simon, J. L., Bretagnon, P., Chapront, J., Chapront-Touzé, M., Francou, G., &
//!   Laskar, J. (1994). Numerical expressions for precession formulae and mean elements
//!   for the Moon and the planets. Astronomy and Astrophysics, 282(2), 663-683.

use lox_core::types::units::JulianCenturies;
use lox_core::units::Angle;

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
pub fn d_simon1994(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[1072260.70369, 1602961601.2090],
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
pub fn lp_simon1994(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[1287104.79305, 129596581.0481],
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
pub fn l_simon1994(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[485868.249036, 1717915923.2178],
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
pub fn f_simon1994(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[335779.526232, 1739527262.8478],
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
pub fn omega_simon1994(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[450160.398036, -6962890.5431],
    ))
}

#[cfg(test)]
mod tests {
    use lox_core::units::AngleUnits;
    use lox_test_utils::assert_approx_eq;

    use super::*;

    // Note that all expected values are outputs from the equivalent ERFA functions.

    // Relative error tolerance for float_eq assertions.
    // This is somewhat loose, being based on observations of how closely our implementations
    // match ERFA outputs rather than any target tolerance.
    // See https://github.com/lox-space/lox/pull/23#discussion_r1398485509
    const TOLERANCE: f64 = 1e-10;

    // Test cases for t.
    const T_ZERO: JulianCenturies = 0.0;
    const T_POSITIVE: JulianCenturies = 1.23456789;
    const T_NEGATIVE: JulianCenturies = -1.23456789;

    #[test]
    fn test_d() {
        assert_approx_eq!(
            d_simon1994(T_ZERO),
            5.198466588650503.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            d_simon1994(T_POSITIVE),
            5.067187555274916.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            d_simon1994(T_NEGATIVE),
            -0.953439685154148.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_lp() {
        assert_approx_eq!(
            lp_simon1994(T_ZERO),
            6.24006012692298.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            lp_simon1994(T_POSITIVE),
            2.806501115480207.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            lp_simon1994(T_NEGATIVE),
            -2.892751475993361.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_l() {
        assert_approx_eq!(
            l_simon1994(T_ZERO),
            2.355555743493879.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            l_simon1994(T_POSITIVE),
            5.399393108792649.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            l_simon1994(T_NEGATIVE),
            -0.688281621805333.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_f() {
        assert_approx_eq!(
            f_simon1994(T_ZERO),
            1.627905081537519.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            f_simon1994(T_POSITIVE),
            2.076369815616488.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            f_simon1994(T_NEGATIVE),
            -5.103744959722151.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_omega() {
        assert_approx_eq!(
            omega_simon1994(T_ZERO),
            2.182439196615671.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            omega_simon1994(T_POSITIVE),
            -1.793813955913912.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            omega_simon1994(T_NEGATIVE),
            6.158692349145257.rad(),
            rtol <= TOLERANCE
        );
    }
}
