/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at http://mozilla.org/MPL/2.0/.
 */

//! Functions for calculating fundamental astronomical parameters as specified by IERS Conventions
//! (2003).

use std::f64::consts::TAU;

use crate::math::arcsec_to_rad_two_pi;
use crate::time::intervals::TDBJulianCenturiesSinceJ2000;
use crate::types::Radians;

use super::{Earth, Jupiter, Mars, Mercury, Moon, Neptune, Saturn, Sun, Uranus, Venus};

/// General accumulated precession in longitude.
pub fn general_accum_precession_in_longitude(t: TDBJulianCenturiesSinceJ2000) -> Radians {
    fast_polynomial::poly_array(t, &[0.0, 0.024381750, 0.00000538691])
}

/// Mean elongation of the Moon from the Sun.
pub fn mean_moon_sun_elongation(t: TDBJulianCenturiesSinceJ2000) -> Radians {
    let arcsec: f64 = fast_polynomial::poly_array(
        t,
        &[
            1072260.703692,
            1602961601.2090,
            -6.3706,
            0.006593,
            -0.00003169,
        ],
    );
    arcsec_to_rad_two_pi(arcsec)
}

pub trait MeanAnomaly {
    fn mean_anomaly(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians;
}

impl MeanAnomaly for Sun {
    fn mean_anomaly(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        let arcsec: f64 = fast_polynomial::poly_array(
            t,
            &[
                1287104.793048,
                129596581.0481,
                -0.5532,
                0.000136,
                -0.00001149,
            ],
        );
        arcsec_to_rad_two_pi(arcsec)
    }
}

impl MeanAnomaly for Moon {
    fn mean_anomaly(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        let arcsec: f64 = fast_polynomial::poly_array(
            t,
            &[
                485868.249036,
                1717915923.2178,
                31.8792,
                0.051635,
                -0.00024470,
            ],
        );
        arcsec_to_rad_two_pi(arcsec)
    }
}

impl Moon {
    /// Mean longitude of the Moon minus the mean longitude of the Moon's ascending node.
    pub fn mean_longitude_minus_ascending_node_mean_longitude(
        &self,
        t: TDBJulianCenturiesSinceJ2000,
    ) -> Radians {
        let arcsec = fast_polynomial::poly_array(
            t,
            &[
                335779.526232,
                1739527262.8478,
                -12.7512,
                -0.001037,
                0.00000417,
            ],
        );
        arcsec_to_rad_two_pi(arcsec)
    }

    pub fn ascending_node_mean_longitude(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        let arcsec = fast_polynomial::poly_array(
            t,
            &[450160.398036, -6962890.5431, 7.4722, 0.007702, -0.00005939],
        );
        arcsec_to_rad_two_pi(arcsec)
    }
}

pub trait MeanLongitude {
    fn mean_longitude(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians;
}

impl MeanLongitude for Mercury {
    #[inline]
    fn mean_longitude(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        (4.402608842 + 2608.7903141574 * t) % TAU
    }
}

impl MeanLongitude for Venus {
    #[inline]
    fn mean_longitude(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        (3.176146697 + 1021.3285546211 * t) % TAU
    }
}

impl MeanLongitude for Earth {
    #[inline]
    fn mean_longitude(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        (1.753470314 + 628.3075849991 * t) % TAU
    }
}

impl MeanLongitude for Mars {
    #[inline]
    fn mean_longitude(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        (6.203480913 + 334.0612426700 * t) % TAU
    }
}

impl MeanLongitude for Jupiter {
    #[inline]
    fn mean_longitude(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        (0.599546497 + 52.9690962641 * t) % TAU
    }
}

impl MeanLongitude for Saturn {
    #[inline]
    fn mean_longitude(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        (0.874016757 + 21.3299104960 * t) % TAU
    }
}

impl MeanLongitude for Neptune {
    #[inline]
    fn mean_longitude(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        (5.311886287 + 3.8133035638 * t) % TAU
    }
}

impl MeanLongitude for Uranus {
    #[inline]
    fn mean_longitude(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        (5.481293872 + 7.4781598567 * t) % TAU
    }
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use float_eq::assert_float_eq;

    use super::*;

    // Note that all expected values are outputs from the equivalent ERFA functions.

    // Relative error TOLERANCE for float_eq assertions.
    // This is somewhat loose, being based on observations of how closely our implementations
    // match ERFA outputs rather than any target TOLERANCE.
    const TOLERANCE: f64 = 1e-11;

    // Test cases for t.
    const T_ZERO: TDBJulianCenturiesSinceJ2000 = 0.0;
    const T_POSITIVE: TDBJulianCenturiesSinceJ2000 = 1.23456789;
    const T_NEGATIVE: TDBJulianCenturiesSinceJ2000 = -1.23456789;

    #[test]
    fn test_general_accum_precession_in_longitude() {
        assert_float_eq!(
            general_accum_precession_in_longitude(T_ZERO),
            0.0,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            general_accum_precession_in_longitude(T_POSITIVE),
            0.030109136153306,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            general_accum_precession_in_longitude(T_NEGATIVE),
            -0.030092715150709,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_moon_sun_elongation() {
        assert_float_eq!(
            mean_moon_sun_elongation(T_ZERO),
            5.198466588660199,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            mean_moon_sun_elongation(T_POSITIVE),
            5.067140540634685,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            mean_moon_sun_elongation(T_NEGATIVE),
            -0.953486820085112,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_anomaly_sun() {
        assert_float_eq!(
            Sun.mean_anomaly(T_ZERO),
            6.240060126913284,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Sun.mean_anomaly(T_POSITIVE),
            2.806497028806777,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Sun.mean_anomaly(T_NEGATIVE),
            -2.892755565148333,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_anomaly_moon() {
        assert_float_eq!(
            Moon.mean_anomaly(T_ZERO),
            2.355555743493879,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.mean_anomaly(T_POSITIVE),
            5.399629142881749,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.mean_anomaly(T_NEGATIVE),
            -0.688046529809469,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_moon_mean_long_minus_ascending_node_mean_long() {
        assert_float_eq!(
            Moon.mean_longitude_minus_ascending_node_mean_longitude(T_ZERO),
            1.627905081537519,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.mean_longitude_minus_ascending_node_mean_longitude(T_POSITIVE),
            2.076275583431815,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.mean_longitude_minus_ascending_node_mean_longitude(T_NEGATIVE),
            -5.103839172987284,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_moon_ascending_node_mean_longitude() {
        assert_float_eq!(
            Moon.ascending_node_mean_longitude(T_ZERO),
            2.182439196615671,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.ascending_node_mean_longitude(T_POSITIVE),
            -1.793758671799353,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.ascending_node_mean_longitude(T_NEGATIVE),
            6.158747492734907,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_mercury() {
        assert_float_eq!(
            Mercury.mean_longitude(T_ZERO),
            4.402608842,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Mercury.mean_longitude(T_POSITIVE),
            1.857299860610716,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Mercury.mean_longitude(T_NEGATIVE),
            -5.618452790969762,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_venus() {
        assert_float_eq!(Venus.mean_longitude(T_ZERO), 3.176146697, rel <= TOLERANCE);
        assert_float_eq!(
            Venus.mean_longitude(T_POSITIVE),
            1.155338629224197,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Venus.mean_longitude(T_NEGATIVE),
            -1.086230542403939,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_earth() {
        assert_float_eq!(Earth.mean_longitude(T_ZERO), 1.753470314, rel <= TOLERANCE);
        assert_float_eq!(
            Earth.mean_longitude(T_POSITIVE),
            4.610047014245303,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Earth.mean_longitude(T_NEGATIVE),
            -1.103106386245365,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_mars() {
        assert_float_eq!(Mars.mean_longitude(T_ZERO), 6.203480913, rel <= TOLERANCE);
        assert_float_eq!(
            Mars.mean_longitude(T_POSITIVE),
            3.934534133027128,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Mars.mean_longitude(T_NEGATIVE),
            -4.093942921386315,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_jupiter() {
        assert_float_eq!(
            Jupiter.mean_longitude(T_ZERO),
            0.599546497,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Jupiter.mean_longitude(T_POSITIVE),
            3.161638835180952,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Jupiter.mean_longitude(T_NEGATIVE),
            -1.962545841180955,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_saturn() {
        assert_float_eq!(Saturn.mean_longitude(T_ZERO), 0.874016757, rel <= TOLERANCE);
        assert_float_eq!(
            Saturn.mean_longitude(T_POSITIVE),
            2.074498123217225,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Saturn.mean_longitude(T_NEGATIVE),
            -0.326464609217226,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_uranus() {
        assert_float_eq!(Uranus.mean_longitude(T_ZERO), 5.481293872, rel <= TOLERANCE);
        assert_float_eq!(
            Uranus.mean_longitude(T_POSITIVE),
            2.147219293009648,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Uranus.mean_longitude(T_NEGATIVE),
            -3.75100216336882,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_longitude_neptune() {
        assert_float_eq!(
            Neptune.mean_longitude(T_ZERO),
            5.311886287,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Neptune.mean_longitude(T_POSITIVE),
            3.73648311451046,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Neptune.mean_longitude(T_NEGATIVE),
            0.604104152309954,
            rel <= TOLERANCE
        );
    }
}
