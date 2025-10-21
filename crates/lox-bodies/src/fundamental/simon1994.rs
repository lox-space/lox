//! Functions for calculating fundamental astronomical parameters as proposed by Simon et al.
//! (1994).

use lox_units::{Angle, types::units::JulianCenturies};

use crate::{Moon, Sun};

pub fn mean_moon_sun_elongation_simon1994(centuries_since_j2000_tdb: JulianCenturies) -> Angle {
    Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[1072260.70369, 1602961601.2090],
    ))
}

impl Sun {
    pub fn mean_anomaly_simon1994(&self, centuries_since_j2000_tdb: JulianCenturies) -> Angle {
        Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
            centuries_since_j2000_tdb,
            &[1287104.79305, 129596581.0481],
        ))
    }
}

impl Moon {
    pub fn mean_anomaly_simon1994(&self, centuries_since_j2000_tdb: JulianCenturies) -> Angle {
        Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
            centuries_since_j2000_tdb,
            &[485868.249036, 1717915923.2178],
        ))
    }

    pub fn mean_argument_of_latitude_simon1994(
        &self,
        centuries_since_j2000_tdb: JulianCenturies,
    ) -> Angle {
        Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
            centuries_since_j2000_tdb,
            &[335779.526232, 1739527262.8478],
        ))
    }

    pub fn ascending_node_mean_longitude_simon1994(
        &self,
        centuries_since_j2000_tdb: JulianCenturies,
    ) -> Angle {
        Angle::arcseconds_normalized_signed(fast_polynomial::poly_array(
            centuries_since_j2000_tdb,
            &[450160.398036, -6962890.5431],
        ))
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;
    use lox_units::AngleUnits;

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
    fn test_mean_moon_sun_elongation_simon1994() {
        assert_approx_eq!(
            mean_moon_sun_elongation_simon1994(T_ZERO),
            5.198466588650503.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            mean_moon_sun_elongation_simon1994(T_POSITIVE),
            5.067187555274916.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            mean_moon_sun_elongation_simon1994(T_NEGATIVE),
            -0.953439685154148.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_sun_mean_anomaly_simon1994() {
        assert_approx_eq!(
            Sun.mean_anomaly_simon1994(T_ZERO),
            6.24006012692298.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Sun.mean_anomaly_simon1994(T_POSITIVE),
            2.806501115480207.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Sun.mean_anomaly_simon1994(T_NEGATIVE),
            -2.892751475993361.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_moon_mean_anomaly_simon1994() {
        assert_approx_eq!(
            Moon.mean_anomaly_simon1994(T_ZERO),
            2.355555743493879.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Moon.mean_anomaly_simon1994(T_POSITIVE),
            5.399393108792649.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Moon.mean_anomaly_simon1994(T_NEGATIVE),
            -0.688281621805333.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_moon_mean_argument_of_latitude_simon1994() {
        assert_approx_eq!(
            Moon.mean_argument_of_latitude_simon1994(T_ZERO),
            1.627905081537519.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Moon.mean_argument_of_latitude_simon1994(T_POSITIVE),
            2.076369815616488.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Moon.mean_argument_of_latitude_simon1994(T_NEGATIVE),
            -5.103744959722151.rad(),
            rtol <= TOLERANCE
        );
    }

    #[test]
    fn test_moon_ascending_node_mean_longitude_simon1994() {
        assert_approx_eq!(
            Moon.ascending_node_mean_longitude_simon1994(T_ZERO),
            2.182439196615671.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Moon.ascending_node_mean_longitude_simon1994(T_POSITIVE),
            -1.793813955913912.rad(),
            rtol <= TOLERANCE
        );
        assert_approx_eq!(
            Moon.ascending_node_mean_longitude_simon1994(T_NEGATIVE),
            6.158692349145257.rad(),
            rtol <= TOLERANCE
        );
    }
}
