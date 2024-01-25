/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

// Auto-generated by `lox_gen`. Do not edit!

use crate::bodies::{
    Ellipsoid, NutationPrecessionCoefficients, PointMass, PolynomialCoefficients,
    RotationalElements, Spheroid, Sun,
};
impl PointMass for Sun {
    fn gravitational_parameter(&self) -> f64 {
        132712440041.27942f64
    }
}
impl Ellipsoid for Sun {
    fn polar_radius(&self) -> f64 {
        695700f64
    }
    fn mean_radius(&self) -> f64 {
        695700f64
    }
}
impl Spheroid for Sun {
    fn equatorial_radius(&self) -> f64 {
        695700f64
    }
}
#[allow(clippy::approx_constant)]
impl RotationalElements for Sun {
    const NUTATION_PRECESSION_COEFFICIENTS: NutationPrecessionCoefficients =
        (&[] as &[f64], &[] as &[f64]);
    const RIGHT_ASCENSION_COEFFICIENTS: PolynomialCoefficients =
        (4.993910588731375f64, 0f64, 0f64, &[] as &[f64]);
    const DECLINATION_COEFFICIENTS: PolynomialCoefficients =
        (1.1147417932487782f64, 0f64, 0f64, &[] as &[f64]);
    const PRIME_MERIDIAN_COEFFICIENTS: PolynomialCoefficients = (
        1.4691483511587469f64,
        0.24756448241988369f64,
        0f64,
        &[] as &[f64],
    );
}
#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use crate::bodies::*;
    #[test]
    fn test_point_mass_10() {
        assert_eq!(Sun.gravitational_parameter(), 132712440041.27942f64);
    }
    #[test]
    fn test_spheroid_10() {
        assert_eq!(Sun.polar_radius(), 695700f64);
        assert_eq!(Sun.mean_radius(), 695700f64);
        assert_eq!(Sun.equatorial_radius(), 695700f64);
    }
    #[test]
    fn test_rotational_elements_nutation_precession_coefficients_10() {
        assert_eq!(
            (&[] as &[f64], &[] as &[f64]),
            Sun::NUTATION_PRECESSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_10() {
        assert_eq!(
            (4.993910588731375f64, 0f64, 0f64, &[] as &[f64]),
            Sun::RIGHT_ASCENSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_10() {
        assert_eq!(
            (1.1147417932487782f64, 0f64, 0f64, &[] as &[f64]),
            Sun::DECLINATION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_10() {
        assert_eq!(
            (
                1.4691483511587469f64,
                0.24756448241988369f64,
                0f64,
                &[] as &[f64]
            ),
            Sun::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
}
