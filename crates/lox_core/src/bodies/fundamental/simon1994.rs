//! Functions for calculating fundamental astronomical parameters as proposed by Simon et al.
//! (1994).

use crate::bodies::{Moon, Sun};
use crate::math::arcsec_to_rad_two_pi;
use crate::time::intervals::TDBJulianCenturiesSinceJ2000;
use crate::types::{Arcsec, Radians};

pub fn mean_moon_sun_elongation_simon1994(t: TDBJulianCenturiesSinceJ2000) -> Radians {
    let arcsec: Arcsec = fast_polynomial::poly_array(t, &[1072260.70369, 1602961601.2090]);
    arcsec_to_rad_two_pi(arcsec)
}

impl Sun {
    pub fn mean_anomaly_simon1994(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        let arcsec: Arcsec = fast_polynomial::poly_array(t, &[1287104.79305, 129596581.0481]);
        arcsec_to_rad_two_pi(arcsec)
    }
}

impl Moon {
    pub fn mean_anomaly_simon1994(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        let arcsec: Arcsec = fast_polynomial::poly_array(t, &[485868.249036, 1717915923.2178]);
        arcsec_to_rad_two_pi(arcsec)
    }

    pub fn mean_argument_of_latitude_simon1994(&self, t: TDBJulianCenturiesSinceJ2000) -> Radians {
        let arcsec: Arcsec = fast_polynomial::poly_array(t, &[335779.526232, 1739527262.8478]);
        arcsec_to_rad_two_pi(arcsec)
    }

    pub fn ascending_node_mean_longitude_simon1994(
        &self,
        t: TDBJulianCenturiesSinceJ2000,
    ) -> Radians {
        let arcsec: Arcsec = fast_polynomial::poly_array(t, &[450160.398036, -6962890.5431]);
        arcsec_to_rad_two_pi(arcsec)
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
    const TOLERANCE: f64 = 1e-12;

    // Test cases for t.
    const T_ZERO: TDBJulianCenturiesSinceJ2000 = 0.0;
    const T_POSITIVE: TDBJulianCenturiesSinceJ2000 = 1.23456789;
    const T_NEGATIVE: TDBJulianCenturiesSinceJ2000 = -1.23456789;

    #[test]
    fn test_mean_moon_sun_elongation_simon1994() {
        assert_float_eq!(
            mean_moon_sun_elongation_simon1994(T_ZERO),
            5.198466588650503,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            mean_moon_sun_elongation_simon1994(T_POSITIVE),
            5.067187555274916,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            mean_moon_sun_elongation_simon1994(T_NEGATIVE),
            -0.953439685154148,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_sun_mean_anomaly_simon1994() {
        assert_float_eq!(
            Sun.mean_anomaly_simon1994(T_ZERO),
            6.24006012692298,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Sun.mean_anomaly_simon1994(T_POSITIVE),
            2.806501115480207,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Sun.mean_anomaly_simon1994(T_NEGATIVE),
            -2.892751475993361,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_moon_mean_anomaly_simon1994() {
        assert_float_eq!(
            Moon.mean_anomaly_simon1994(T_ZERO),
            2.355555743493879,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.mean_anomaly_simon1994(T_POSITIVE),
            5.399393108792649,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.mean_anomaly_simon1994(T_NEGATIVE),
            -0.688281621805333,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_moon_mean_argument_of_latitude_simon1994() {
        assert_float_eq!(
            Moon.mean_argument_of_latitude_simon1994(T_ZERO),
            1.627905081537519,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.mean_argument_of_latitude_simon1994(T_POSITIVE),
            2.076369815616488,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.mean_argument_of_latitude_simon1994(T_NEGATIVE),
            -5.103744959722151,
            rel <= TOLERANCE
        );
    }

    #[test]
    fn test_moon_ascending_node_mean_longitude_simon1994() {
        assert_float_eq!(
            Moon.ascending_node_mean_longitude_simon1994(T_ZERO),
            2.182439196615671,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.ascending_node_mean_longitude_simon1994(T_POSITIVE),
            -1.793813955913912,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            Moon.ascending_node_mean_longitude_simon1994(T_NEGATIVE),
            6.158692349145257,
            rel <= TOLERANCE
        );
    }
}