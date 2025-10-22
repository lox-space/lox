// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Functions for calculating fundamental astronomical parameters as specified by IERS Conventions
//! (2003).

use crate::{Earth, Jupiter, Mars, Mercury, Moon, Neptune, Saturn, Sun, Uranus, Venus};

use lox_units::{Angle, AngleUnits, types::units::JulianCenturies};

/// General accumulated precession in longitude.
pub fn general_accum_precession_in_longitude_iers03(
    centuries_since_j2000_tdb: JulianCenturies,
) -> Angle {
    fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[0.0, 0.024381750, 0.00000538691],
    )
    .rad()
}

/// Mean elongation of the Moon from the Sun.
pub fn mean_moon_sun_elongation_iers03(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
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

impl Sun {
    pub fn mean_anomaly_iers03(&self, centuries_since_j2000_tdb: JulianCenturies) -> Angle {
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
}

impl Moon {
    pub fn mean_anomaly_iers03(&self, centuries_since_j2000_tdb: JulianCenturies) -> Angle {
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

    pub fn mean_longitude_minus_ascending_node_mean_longitude_iers03(
        &self,
        centuries_since_j2000_tdb: JulianCenturies,
    ) -> Angle {
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

    pub fn ascending_node_mean_longitude_iers03(
        &self,
        centuries_since_j2000_tdb: JulianCenturies,
    ) -> Angle {
        Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
            centuries_since_j2000_tdb,
            &[450160.398036, -6962890.5431, 7.4722, 0.007702, -0.00005939],
        ))
    }
}

impl Mercury {
    #[inline]
    pub fn mean_longitude_iers03(&self, centuries_since_j2000_tdb: JulianCenturies) -> Angle {
        (4.402608842 + 2608.7903141574 * centuries_since_j2000_tdb)
            .rad()
            .mod_two_pi_signed()
    }
}

impl Venus {
    #[inline]
    pub fn mean_longitude_iers03(&self, centuries_since_j2000_tdb: JulianCenturies) -> Angle {
        (3.176146697 + 1021.3285546211 * centuries_since_j2000_tdb)
            .rad()
            .mod_two_pi_signed()
    }
}

impl Earth {
    #[inline]
    pub fn mean_longitude_iers03(&self, centuries_since_j2000_tdb: JulianCenturies) -> Angle {
        (1.753470314 + 628.3075849991 * centuries_since_j2000_tdb)
            .rad()
            .mod_two_pi_signed()
    }
}

impl Mars {
    #[inline]
    pub fn mean_longitude_iers03(&self, centuries_since_j2000_tdb: JulianCenturies) -> Angle {
        (6.203480913 + 334.0612426700 * centuries_since_j2000_tdb)
            .rad()
            .mod_two_pi_signed()
    }
}

impl Jupiter {
    #[inline]
    pub fn mean_longitude_iers03(&self, centuries_since_j2000_tdb: JulianCenturies) -> Angle {
        (0.599546497 + 52.9690962641 * centuries_since_j2000_tdb)
            .rad()
            .mod_two_pi_signed()
    }
}

impl Saturn {
    #[inline]
    pub fn mean_longitude_iers03(&self, centuries_since_j2000_tdb: JulianCenturies) -> Angle {
        (0.874016757 + 21.3299104960 * centuries_since_j2000_tdb)
            .rad()
            .mod_two_pi_signed()
    }
}

impl Neptune {
    #[inline]
    pub fn mean_longitude_iers03(&self, centuries_since_j2000_tdb: JulianCenturies) -> Angle {
        (5.311886287 + 3.8133035638 * centuries_since_j2000_tdb)
            .rad()
            .mod_two_pi_signed()
    }
}

impl Uranus {
    #[inline]
    pub fn mean_longitude_iers03(&self, centuries_since_j2000_tdb: JulianCenturies) -> Angle {
        (5.481293872 + 7.4781598567 * centuries_since_j2000_tdb)
            .rad()
            .mod_two_pi_signed()
    }
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use lox_test_utils::assert_approx_eq;

    use super::*;

    // Note that all expected values are outputs from the equivalent ERFA functions.

    // rtolative error tolerance for float_eq assertions.
    // This is somewhat loose, being based on observations of how closely our implementations
    // match ERFA outputs rather than any target tolerance.
    // See https://github.com/lox-space/lox/pull/23#discussion_r1398485509
    const TOLERANCE: f64 = 1e-11;

    // Test cases for t.
    const T_ZERO: JulianCenturies = 0.0;
    const T_POSITIVE: JulianCenturies = 1.23456789;
    const T_NEGATIVE: JulianCenturies = -1.23456789;

    #[test]
    fn test_general_accum_precession_in_longitude() {
        assert_approx_eq!(
            general_accum_precession_in_longitude_iers03(T_ZERO),
            0.0.rad(),
            atol <= TOLERANCE
        );
        assert_approx_eq!(
            general_accum_precession_in_longitude_iers03(T_POSITIVE),
            0.030109136153306.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            general_accum_precession_in_longitude_iers03(T_NEGATIVE),
            -0.030092715150709.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_moon_sun_elongation() {
        assert_approx_eq!(
            mean_moon_sun_elongation_iers03(T_ZERO),
            5.198466588660199.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            mean_moon_sun_elongation_iers03(T_POSITIVE),
            5.067140540634685.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            mean_moon_sun_elongation_iers03(T_NEGATIVE),
            -0.953486820085112.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_anomaly_sun() {
        assert_approx_eq!(
            Sun.mean_anomaly_iers03(T_ZERO),
            6.240060126913284.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Sun.mean_anomaly_iers03(T_POSITIVE),
            2.806497028806777.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Sun.mean_anomaly_iers03(T_NEGATIVE),
            -2.892755565148333.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_anomaly_moon() {
        assert_approx_eq!(
            Moon.mean_anomaly_iers03(T_ZERO),
            2.355555743493879.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Moon.mean_anomaly_iers03(T_POSITIVE),
            5.399629142881749.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Moon.mean_anomaly_iers03(T_NEGATIVE),
            -0.688046529809469.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_moon_mean_long_minus_ascending_node_mean_long() {
        assert_approx_eq!(
            Moon.mean_longitude_minus_ascending_node_mean_longitude_iers03(T_ZERO),
            1.627905081537519.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Moon.mean_longitude_minus_ascending_node_mean_longitude_iers03(T_POSITIVE),
            2.076275583431815.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Moon.mean_longitude_minus_ascending_node_mean_longitude_iers03(T_NEGATIVE),
            -5.103839172987284.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_moon_ascending_node_mean_longitude() {
        assert_approx_eq!(
            Moon.ascending_node_mean_longitude_iers03(T_ZERO),
            2.182439196615671.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Moon.ascending_node_mean_longitude_iers03(T_POSITIVE),
            -1.793758671799353.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Moon.ascending_node_mean_longitude_iers03(T_NEGATIVE),
            6.158747492734907.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_mercury() {
        assert_approx_eq!(
            Mercury.mean_longitude_iers03(T_ZERO),
            4.402608842.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Mercury.mean_longitude_iers03(T_POSITIVE),
            1.857299860610716.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Mercury.mean_longitude_iers03(T_NEGATIVE),
            -5.618452790969762.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_venus() {
        assert_approx_eq!(
            Venus.mean_longitude_iers03(T_ZERO),
            3.176146697.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Venus.mean_longitude_iers03(T_POSITIVE),
            1.155338629224197.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Venus.mean_longitude_iers03(T_NEGATIVE),
            -1.086230542403939.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_earth() {
        assert_approx_eq!(
            Earth.mean_longitude_iers03(T_ZERO),
            1.753470314.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Earth.mean_longitude_iers03(T_POSITIVE),
            4.610047014245303.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Earth.mean_longitude_iers03(T_NEGATIVE),
            -1.103106386245365.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_mars() {
        assert_approx_eq!(
            Mars.mean_longitude_iers03(T_ZERO),
            6.203480913.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Mars.mean_longitude_iers03(T_POSITIVE),
            3.934534133027128.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Mars.mean_longitude_iers03(T_NEGATIVE),
            -4.093942921386315.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_jupiter() {
        assert_approx_eq!(
            Jupiter.mean_longitude_iers03(T_ZERO),
            0.599546497.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Jupiter.mean_longitude_iers03(T_POSITIVE),
            3.161638835180952.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Jupiter.mean_longitude_iers03(T_NEGATIVE),
            -1.962545841180955.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_saturn() {
        assert_approx_eq!(
            Saturn.mean_longitude_iers03(T_ZERO),
            0.874016757.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Saturn.mean_longitude_iers03(T_POSITIVE),
            2.074498123217225.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Saturn.mean_longitude_iers03(T_NEGATIVE),
            -0.326464609217226.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_uranus() {
        assert_approx_eq!(
            Uranus.mean_longitude_iers03(T_ZERO),
            5.481293872.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Uranus.mean_longitude_iers03(T_POSITIVE),
            2.147219293009648.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Uranus.mean_longitude_iers03(T_NEGATIVE),
            -3.75100216336882.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_neptune() {
        assert_approx_eq!(
            Neptune.mean_longitude_iers03(T_ZERO),
            5.311886287.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Neptune.mean_longitude_iers03(T_POSITIVE),
            3.73648311451046.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Neptune.mean_longitude_iers03(T_NEGATIVE),
            0.604104152309954.rad(),
            rtol <= TOLERANCE
        );
    }
}
