use std::f64::consts::{PI, TAU};

use crate::types::{Arcsec, Radians};

/// Module math provides common mathematical functions shared by many parts of the library.

/// Normalizes an angle `a` to the range [center-π, center+π).
pub(crate) fn normalize_two_pi(a: Radians, center: Radians) -> Radians {
    a - 2.0 * PI * ((a + PI - center) / (2.0 * PI)).floor()
}

pub(crate) const ARCSECONDS_IN_CIRCLE: f64 = 360.0 * 60.0 * 60.0;

pub(crate) const RADIANS_IN_ARCSECOND: Radians = TAU / ARCSECONDS_IN_CIRCLE;

pub(crate) const RADIANS_IN_MILLIARCSECOND: Radians = RADIANS_IN_ARCSECOND / 1000.0;

/// Converts arcseconds to radians, modulo 2π.
#[inline]
pub(crate) fn arcsec_to_rad_two_pi(arcsec: Arcsec) -> Radians {
    arcsec_to_rad(arcsec % ARCSECONDS_IN_CIRCLE)
}

#[inline]
pub(crate) fn arcsec_to_rad(arcsec: Arcsec) -> Radians {
    arcsec * RADIANS_IN_ARCSECOND
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use super::*;

    const TOLERANCE: f64 = f64::EPSILON;

    #[test]
    fn test_normalize_two_pi() {
        // Center 0.0 – expected range [-π, π).
        //
        // abs is preferred to rel for floating-point comparisons with 0.0. See
        // https://randomascii.wordpress.com/2012/02/25/comparing-floating-point-numbers-2012-edition/#inferna
        assert_float_eq!(normalize_two_pi(0.0, 0.0), 0.0, abs <= TOLERANCE);
        assert_float_eq!(normalize_two_pi(PI, 0.0), -PI, rel <= TOLERANCE);
        assert_float_eq!(normalize_two_pi(-PI, 0.0), -PI, rel <= TOLERANCE);
        assert_float_eq!(normalize_two_pi(TAU, 0.0), 0.0, abs <= TOLERANCE);
        assert_float_eq!(normalize_two_pi(PI / 2.0, 0.0), PI / 2.0, rel <= TOLERANCE);
        assert_float_eq!(
            normalize_two_pi(-PI / 2.0, 0.0),
            -PI / 2.0,
            rel <= TOLERANCE,
        );

        // Center π – expected range [0, 2π).
        assert_float_eq!(normalize_two_pi(0.0, PI), 0.0, abs <= TOLERANCE);
        assert_float_eq!(normalize_two_pi(PI, PI), PI, rel <= TOLERANCE);
        assert_float_eq!(normalize_two_pi(-PI, PI), PI, rel <= TOLERANCE);
        assert_float_eq!(normalize_two_pi(TAU, PI), 0.0, abs <= TOLERANCE);
        assert_float_eq!(normalize_two_pi(PI / 2.0, PI), PI / 2.0, rel <= TOLERANCE);
        assert_float_eq!(
            normalize_two_pi(-PI / 2.0, PI),
            3.0 * PI / 2.0,
            rel <= TOLERANCE,
        );

        // Center -π – expected range [-2π, 0).
        assert_float_eq!(normalize_two_pi(0.0, -PI), -TAU, rel <= TOLERANCE);
        assert_float_eq!(normalize_two_pi(PI, -PI), -PI, rel <= TOLERANCE);
        assert_float_eq!(normalize_two_pi(-PI, -PI), -PI, rel <= TOLERANCE);
        assert_float_eq!(normalize_two_pi(TAU, -PI), -TAU, rel <= TOLERANCE);
        assert_float_eq!(
            normalize_two_pi(PI / 2.0, -PI),
            -3.0 * PI / 2.0,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            normalize_two_pi(-PI / 2.0, -PI),
            -PI / 2.0,
            rel <= TOLERANCE,
        );
    }
}
