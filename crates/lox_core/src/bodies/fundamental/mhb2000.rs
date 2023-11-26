/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Functions for calculating fundamental astronomical parameters using the Mathews-Herring-Buffett
//! (MHB2000) nutation series. Note that these typically differ from their IERS03 equivalents by
//! less than 0.1 microarcseconds, but are retained as a faithful reproduction of the original
//! model.

use crate::bodies::{Moon, Neptune, Sun};
use crate::math::arcsec_to_rad_two_pi;
use crate::time::intervals::TDBJulianCenturiesSinceJ2000;
use crate::types::{Arcsec, Radians};
use std::f64::consts::TAU;

pub fn mean_moon_sun_elongation_mhb2000_luni_solar(t: TDBJulianCenturiesSinceJ2000) -> Radians {
    let arcsec: Arcsec = fast_polynomial::poly_array(
        t,
        &[
            1072260.70369,
            1602961601.2090,
            -6.3706,
            0.006593,
            -0.00003169,
        ],
    );
    arcsec_to_rad_two_pi(arcsec)
}

pub fn mean_moon_sun_elongation_mhb2000_planetary(t: TDBJulianCenturiesSinceJ2000) -> Radians {
    fast_polynomial::poly_array(t, &[5.198466741, 7771.3771468121]) % TAU
}

impl Sun {
    pub fn mean_anomaly_mhb2000(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        let arcsec: Arcsec = fast_polynomial::poly_array(
            t,
            &[
                1287104.79305,
                129596581.0481,
                -0.5532,
                0.000136,
                -0.00001149,
            ],
        );
        arcsec_to_rad_two_pi(arcsec)
    }
}

impl Moon {
    pub fn mean_anomaly_mhb2000(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        fast_polynomial::poly_array(t, &[2.35555598, 8328.6914269554]) % TAU
    }

    pub fn mean_longitude_minus_ascending_node_mean_longitude_mhb2000(
        &self,
        t: TDBJulianCenturiesSinceJ2000,
    ) -> Radians {
        fast_polynomial::poly_array(t, &[1.627905234, 8433.466158131]) % TAU
    }

    pub fn ascending_node_mean_longitude_mhb2000(
        &self,
        t: TDBJulianCenturiesSinceJ2000,
    ) -> Radians {
        fast_polynomial::poly_array(t, &[2.18243920, -33.757045]) % TAU
    }
}

impl Neptune {
    pub fn mean_longitude_mhb2000(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        fast_polynomial::poly_array(t, &[5.3211590, 3.81277740]) % TAU
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use super::*;

    // Note that all expected values are outputs from the equivalent ERFA functions.

    // Relative error tolerance for float_eq assertions.
    // This is somewhat loose, being based on observations of how closely our implementations
    // match ERFA outputs rather than any target tolerance.
    // See https://github.com/lox-space/lox/pull/23#discussion_r1398485509
    const TOLERANCE: f64 = 1e-11;

    // Test cases for t.
    const T_ZERO: TDBJulianCenturiesSinceJ2000 = 0.0;
    const T_POSITIVE: TDBJulianCenturiesSinceJ2000 = 1.23456789;
    const T_NEGATIVE: TDBJulianCenturiesSinceJ2000 = -1.23456789;

    #[test]
    fn test_mean_moon_sun_elongation_mhb2000_luni_solar() {
        assert_float_eq!(
            mean_moon_sun_elongation_mhb2000_luni_solar(T_ZERO),
            5.198466588650503,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            mean_moon_sun_elongation_mhb2000_luni_solar(T_POSITIVE),
            5.067140540624282,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            mean_moon_sun_elongation_mhb2000_luni_solar(T_NEGATIVE),
            -0.953486820095515,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_mean_moon_sun_elongation_mhb2000_planetary() {
        assert_float_eq!(
            mean_moon_sun_elongation_mhb2000_planetary(T_ZERO),
            5.1984667410,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            mean_moon_sun_elongation_mhb2000_planetary(T_POSITIVE),
            5.06718921180569,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            mean_moon_sun_elongation_mhb2000_planetary(T_NEGATIVE),
            -0.953441036985836,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_sun_mean_anomaly_mhb2000() {
        assert_float_eq!(
            Sun.mean_anomaly_mhb2000(T_ZERO),
            6.24006012692298,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Sun.mean_anomaly_mhb2000(T_POSITIVE),
            2.806497028816457,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Sun.mean_anomaly_mhb2000(T_NEGATIVE),
            -2.892755565138653,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_moon_mean_anomaly_mhb2000() {
        assert_float_eq!(
            Moon.mean_anomaly_mhb2000(T_ZERO),
            2.35555598,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.mean_anomaly_mhb2000(T_POSITIVE),
            5.399394871613055,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.mean_anomaly_mhb2000(T_NEGATIVE),
            -0.688282911613584,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_moon_mean_longitude_minus_ascending_node_mean_longitude_mhb2000() {
        assert_float_eq!(
            Moon.mean_longitude_minus_ascending_node_mean_longitude_mhb2000(T_ZERO),
            1.627905234,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.mean_longitude_minus_ascending_node_mean_longitude_mhb2000(T_POSITIVE),
            2.07637146761946,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.mean_longitude_minus_ascending_node_mean_longitude_mhb2000(T_NEGATIVE),
            -5.103746306797973,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_moon_ascending_node_mean_longitude_mhb2000() {
        assert_float_eq!(
            Moon.ascending_node_mean_longitude_mhb2000(T_ZERO),
            2.18243920,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.ascending_node_mean_longitude_mhb2000(T_POSITIVE),
            -1.793812775207527,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.ascending_node_mean_longitude_mhb2000(T_NEGATIVE),
            6.15869117520753,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_neptune_mean_longitude_mhb2000() {
        assert_float_eq!(
            Neptune.mean_longitude_mhb2000(T_ZERO),
            5.3211590,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Neptune.mean_longitude_mhb2000(T_POSITIVE),
            3.7451062425781,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Neptune.mean_longitude_mhb2000(T_NEGATIVE),
            0.614026450242314,
            rel <= TOLERANCE
        );
    }
}
