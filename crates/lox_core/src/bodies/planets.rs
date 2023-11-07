/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at http://mozilla.org/MPL/2.0/.
 */

// Auto-generated by `lox_gen`. Do not edit!

use super::{
    BodyRotationalElements, BodyTrigRotationalElements, Ellipsoid, NaifId, PointMass,
    PolynomialCoefficient, Spheroid,
};
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Mercury;
impl NaifId for Mercury {
    fn id() -> i32 {
        199i32
    }
}
impl PointMass for Mercury {
    fn gravitational_parameter() -> f64 {
        22031.868551400003f64
    }
}
impl Ellipsoid for Mercury {
    fn polar_radius() -> f64 {
        2438.26f64
    }
    fn mean_radius() -> f64 {
        2439.7733333333335f64
    }
}
impl Spheroid for Mercury {
    fn equatorial_radius() -> f64 {
        2440.53f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Mercury {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [281.0103f64, -0.0328f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [61.4155f64, -0.0049f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [329.5988f64, 6.1385108f64, 0f64];
}
#[allow(clippy::approx_constant)]
impl BodyTrigRotationalElements for Mercury {
    const NUT_PREC_RIGHT_ASCENSION_COEFFICIENTS: &'static [PolynomialCoefficient] =
        &[0f64, 0f64, 0f64, 0f64, 0f64];
    const NUT_PREC_DECLINATION_COEFFICIENTS: &'static [PolynomialCoefficient] =
        &[0f64, 0f64, 0f64, 0f64, 0f64];
    const NUT_PREC_PRIME_MERIDIAN_COEFFICIENTS: &'static [PolynomialCoefficient] = &[
        0.01067257f64,
        -0.00112309f64,
        -0.0001104f64,
        -0.00002539f64,
        -0.00000571f64,
    ];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Venus;
impl NaifId for Venus {
    fn id() -> i32 {
        299i32
    }
}
impl PointMass for Venus {
    fn gravitational_parameter() -> f64 {
        324858.592f64
    }
}
impl Ellipsoid for Venus {
    fn polar_radius() -> f64 {
        6051.8f64
    }
    fn mean_radius() -> f64 {
        6051.8f64
    }
}
impl Spheroid for Venus {
    fn equatorial_radius() -> f64 {
        6051.8f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Venus {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [272.76f64, 0f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [67.16f64, 0f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] = [160.2f64, -1.4813688f64, 0f64];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Earth;
impl NaifId for Earth {
    fn id() -> i32 {
        399i32
    }
}
impl PointMass for Earth {
    fn gravitational_parameter() -> f64 {
        398600.43550702266f64
    }
}
impl Ellipsoid for Earth {
    fn polar_radius() -> f64 {
        6356.7519f64
    }
    fn mean_radius() -> f64 {
        6371.008366666666f64
    }
}
impl Spheroid for Earth {
    fn equatorial_radius() -> f64 {
        6378.1366f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Earth {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [0f64, -0.641f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [90f64, -0.557f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [190.147f64, 360.9856235f64, 0f64];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Mars;
impl NaifId for Mars {
    fn id() -> i32 {
        499i32
    }
}
impl PointMass for Mars {
    fn gravitational_parameter() -> f64 {
        42828.37362069909f64
    }
}
impl Ellipsoid for Mars {
    fn polar_radius() -> f64 {
        3376.2f64
    }
    fn mean_radius() -> f64 {
        3389.5266666666666f64
    }
}
impl Spheroid for Mars {
    fn equatorial_radius() -> f64 {
        3396.19f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Mars {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [317.269202f64, -0.10927547f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [54.432516f64, -0.05827105f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [176.049863f64, 350.891982443297f64, 0f64];
}
#[allow(clippy::approx_constant)]
impl BodyTrigRotationalElements for Mars {
    const NUT_PREC_RIGHT_ASCENSION_COEFFICIENTS: &'static [PolynomialCoefficient] = &[
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0.000068f64,
        0.000238f64,
        0.000052f64,
        0.000009f64,
        0.419057f64,
    ];
    const NUT_PREC_DECLINATION_COEFFICIENTS: &'static [PolynomialCoefficient] = &[
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0.000051f64,
        0.000141f64,
        0.000031f64,
        0.000005f64,
        1.591274f64,
    ];
    const NUT_PREC_PRIME_MERIDIAN_COEFFICIENTS: &'static [PolynomialCoefficient] = &[
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0.000145f64,
        0.000157f64,
        0.00004f64,
        0.000001f64,
        0.000001f64,
        0.584542f64,
    ];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Jupiter;
impl NaifId for Jupiter {
    fn id() -> i32 {
        599i32
    }
}
impl PointMass for Jupiter {
    fn gravitational_parameter() -> f64 {
        126686531.9003704f64
    }
}
impl Ellipsoid for Jupiter {
    fn polar_radius() -> f64 {
        66854f64
    }
    fn mean_radius() -> f64 {
        69946f64
    }
}
impl Spheroid for Jupiter {
    fn equatorial_radius() -> f64 {
        71492f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Jupiter {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [268.056595f64, -0.006499f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [64.495303f64, 0.002413f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] = [284.95f64, 870.536f64, 0f64];
}
#[allow(clippy::approx_constant)]
impl BodyTrigRotationalElements for Jupiter {
    const NUT_PREC_RIGHT_ASCENSION_COEFFICIENTS: &'static [PolynomialCoefficient] = &[
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0.000117f64,
        0.000938f64,
        0.001432f64,
        0.00003f64,
        0.00215f64,
    ];
    const NUT_PREC_DECLINATION_COEFFICIENTS: &'static [PolynomialCoefficient] = &[
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0f64,
        0.00005f64,
        0.000404f64,
        0.000617f64,
        -0.000013f64,
        0.000926f64,
    ];
    const NUT_PREC_PRIME_MERIDIAN_COEFFICIENTS: &'static [PolynomialCoefficient] = &[
        0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64,
    ];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Saturn;
impl NaifId for Saturn {
    fn id() -> i32 {
        699i32
    }
}
impl PointMass for Saturn {
    fn gravitational_parameter() -> f64 {
        37931206.23436167f64
    }
}
impl Ellipsoid for Saturn {
    fn polar_radius() -> f64 {
        54364f64
    }
    fn mean_radius() -> f64 {
        58300f64
    }
}
impl Spheroid for Saturn {
    fn equatorial_radius() -> f64 {
        60268f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Saturn {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [40.589f64, -0.036f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [83.537f64, -0.004f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] = [38.9f64, 810.7939024f64, 0f64];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Uranus;
impl NaifId for Uranus {
    fn id() -> i32 {
        799i32
    }
}
impl PointMass for Uranus {
    fn gravitational_parameter() -> f64 {
        5793951.256527211f64
    }
}
impl Ellipsoid for Uranus {
    fn polar_radius() -> f64 {
        24973f64
    }
    fn mean_radius() -> f64 {
        25363.666666666668f64
    }
}
impl Spheroid for Uranus {
    fn equatorial_radius() -> f64 {
        25559f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Uranus {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [257.311f64, 0f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [-15.175f64, 0f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [203.81f64, -501.1600928f64, 0f64];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Neptune;
impl NaifId for Neptune {
    fn id() -> i32 {
        899i32
    }
}
impl PointMass for Neptune {
    fn gravitational_parameter() -> f64 {
        6835103.145462294f64
    }
}
impl Ellipsoid for Neptune {
    fn polar_radius() -> f64 {
        24341f64
    }
    fn mean_radius() -> f64 {
        24623f64
    }
}
impl Spheroid for Neptune {
    fn equatorial_radius() -> f64 {
        24764f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Neptune {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [299.36f64, 0f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [43.46f64, 0f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [249.978f64, 541.1397757f64, 0f64];
}
#[allow(clippy::approx_constant)]
impl BodyTrigRotationalElements for Neptune {
    const NUT_PREC_RIGHT_ASCENSION_COEFFICIENTS: &'static [PolynomialCoefficient] =
        &[0.7f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64];
    const NUT_PREC_DECLINATION_COEFFICIENTS: &'static [PolynomialCoefficient] =
        &[-0.51f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64];
    const NUT_PREC_PRIME_MERIDIAN_COEFFICIENTS: &'static [PolynomialCoefficient] =
        &[-0.48f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Pluto;
impl NaifId for Pluto {
    fn id() -> i32 {
        999i32
    }
}
impl PointMass for Pluto {
    fn gravitational_parameter() -> f64 {
        869.6138177608748f64
    }
}
impl Ellipsoid for Pluto {
    fn polar_radius() -> f64 {
        1188.3f64
    }
    fn mean_radius() -> f64 {
        1188.3f64
    }
}
impl Spheroid for Pluto {
    fn equatorial_radius() -> f64 {
        1188.3f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Pluto {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [132.993f64, 0f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [-6.163f64, 0f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [302.695f64, 56.3625225f64, 0f64];
}
#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;
    #[test]
    fn test_naif_id_199() {
        assert_eq!(Mercury::id(), 199i32)
    }
    #[test]
    fn test_point_mass_199() {
        assert_eq!(Mercury::gravitational_parameter(), 22031.868551400003f64);
    }
    #[test]
    fn test_spheroid_199() {
        assert_eq!(Mercury::polar_radius(), 2438.26f64);
        assert_eq!(Mercury::mean_radius(), 2439.7733333333335f64);
        assert_eq!(Mercury::equatorial_radius(), 2440.53f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_199() {
        assert_eq!(
            [281.0103f64, -0.0328f64, 0f64],
            Mercury::RIGHT_ASCENSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_199() {
        assert_eq!(
            [61.4155f64, -0.0049f64, 0f64],
            Mercury::DECLINATION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_199() {
        assert_eq!(
            [329.5988f64, 6.1385108f64, 0f64],
            Mercury::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_trig_rotational_elements_nut_prec_right_ascension_coefficients199() {
        assert_eq!(
            &[0f64, 0f64, 0f64, 0f64, 0f64],
            Mercury::NUT_PREC_RIGHT_ASCENSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_trig_rotational_elements_nut_prec_declination_coefficients199() {
        assert_eq!(
            &[0f64, 0f64, 0f64, 0f64, 0f64],
            Mercury::NUT_PREC_DECLINATION_COEFFICIENTS
        )
    }
    #[test]
    fn test_trig_rotational_elements_nut_prec_prime_meridian_coefficients199() {
        assert_eq!(
            &[
                0.01067257f64,
                -0.00112309f64,
                -0.0001104f64,
                -0.00002539f64,
                -0.00000571f64
            ],
            Mercury::NUT_PREC_PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_299() {
        assert_eq!(Venus::id(), 299i32)
    }
    #[test]
    fn test_point_mass_299() {
        assert_eq!(Venus::gravitational_parameter(), 324858.592f64);
    }
    #[test]
    fn test_spheroid_299() {
        assert_eq!(Venus::polar_radius(), 6051.8f64);
        assert_eq!(Venus::mean_radius(), 6051.8f64);
        assert_eq!(Venus::equatorial_radius(), 6051.8f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_299() {
        assert_eq!([272.76f64, 0f64, 0f64], Venus::RIGHT_ASCENSION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_299() {
        assert_eq!([67.16f64, 0f64, 0f64], Venus::DECLINATION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_299() {
        assert_eq!(
            [160.2f64, -1.4813688f64, 0f64],
            Venus::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_399() {
        assert_eq!(Earth::id(), 399i32)
    }
    #[test]
    fn test_point_mass_399() {
        assert_eq!(Earth::gravitational_parameter(), 398600.43550702266f64);
    }
    #[test]
    fn test_spheroid_399() {
        assert_eq!(Earth::polar_radius(), 6356.7519f64);
        assert_eq!(Earth::mean_radius(), 6371.008366666666f64);
        assert_eq!(Earth::equatorial_radius(), 6378.1366f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_399() {
        assert_eq!([0f64, -0.641f64, 0f64], Earth::RIGHT_ASCENSION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_399() {
        assert_eq!([90f64, -0.557f64, 0f64], Earth::DECLINATION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_399() {
        assert_eq!(
            [190.147f64, 360.9856235f64, 0f64],
            Earth::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_499() {
        assert_eq!(Mars::id(), 499i32)
    }
    #[test]
    fn test_point_mass_499() {
        assert_eq!(Mars::gravitational_parameter(), 42828.37362069909f64);
    }
    #[test]
    fn test_spheroid_499() {
        assert_eq!(Mars::polar_radius(), 3376.2f64);
        assert_eq!(Mars::mean_radius(), 3389.5266666666666f64);
        assert_eq!(Mars::equatorial_radius(), 3396.19f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_499() {
        assert_eq!(
            [317.269202f64, -0.10927547f64, 0f64],
            Mars::RIGHT_ASCENSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_499() {
        assert_eq!(
            [54.432516f64, -0.05827105f64, 0f64],
            Mars::DECLINATION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_499() {
        assert_eq!(
            [176.049863f64, 350.891982443297f64, 0f64],
            Mars::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_trig_rotational_elements_nut_prec_right_ascension_coefficients499() {
        assert_eq!(
            &[
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0.000068f64,
                0.000238f64,
                0.000052f64,
                0.000009f64,
                0.419057f64
            ],
            Mars::NUT_PREC_RIGHT_ASCENSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_trig_rotational_elements_nut_prec_declination_coefficients499() {
        assert_eq!(
            &[
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0.000051f64,
                0.000141f64,
                0.000031f64,
                0.000005f64,
                1.591274f64
            ],
            Mars::NUT_PREC_DECLINATION_COEFFICIENTS
        )
    }
    #[test]
    fn test_trig_rotational_elements_nut_prec_prime_meridian_coefficients499() {
        assert_eq!(
            &[
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0.000145f64,
                0.000157f64,
                0.00004f64,
                0.000001f64,
                0.000001f64,
                0.584542f64
            ],
            Mars::NUT_PREC_PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_599() {
        assert_eq!(Jupiter::id(), 599i32)
    }
    #[test]
    fn test_point_mass_599() {
        assert_eq!(Jupiter::gravitational_parameter(), 126686531.9003704f64);
    }
    #[test]
    fn test_spheroid_599() {
        assert_eq!(Jupiter::polar_radius(), 66854f64);
        assert_eq!(Jupiter::mean_radius(), 69946f64);
        assert_eq!(Jupiter::equatorial_radius(), 71492f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_599() {
        assert_eq!(
            [268.056595f64, -0.006499f64, 0f64],
            Jupiter::RIGHT_ASCENSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_599() {
        assert_eq!(
            [64.495303f64, 0.002413f64, 0f64],
            Jupiter::DECLINATION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_599() {
        assert_eq!(
            [284.95f64, 870.536f64, 0f64],
            Jupiter::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_trig_rotational_elements_nut_prec_right_ascension_coefficients599() {
        assert_eq!(
            &[
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0.000117f64,
                0.000938f64,
                0.001432f64,
                0.00003f64,
                0.00215f64
            ],
            Jupiter::NUT_PREC_RIGHT_ASCENSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_trig_rotational_elements_nut_prec_declination_coefficients599() {
        assert_eq!(
            &[
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0.00005f64,
                0.000404f64,
                0.000617f64,
                -0.000013f64,
                0.000926f64
            ],
            Jupiter::NUT_PREC_DECLINATION_COEFFICIENTS
        )
    }
    #[test]
    fn test_trig_rotational_elements_nut_prec_prime_meridian_coefficients599() {
        assert_eq!(
            &[
                0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64,
                0f64
            ],
            Jupiter::NUT_PREC_PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_699() {
        assert_eq!(Saturn::id(), 699i32)
    }
    #[test]
    fn test_point_mass_699() {
        assert_eq!(Saturn::gravitational_parameter(), 37931206.23436167f64);
    }
    #[test]
    fn test_spheroid_699() {
        assert_eq!(Saturn::polar_radius(), 54364f64);
        assert_eq!(Saturn::mean_radius(), 58300f64);
        assert_eq!(Saturn::equatorial_radius(), 60268f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_699() {
        assert_eq!(
            [40.589f64, -0.036f64, 0f64],
            Saturn::RIGHT_ASCENSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_699() {
        assert_eq!(
            [83.537f64, -0.004f64, 0f64],
            Saturn::DECLINATION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_699() {
        assert_eq!(
            [38.9f64, 810.7939024f64, 0f64],
            Saturn::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_799() {
        assert_eq!(Uranus::id(), 799i32)
    }
    #[test]
    fn test_point_mass_799() {
        assert_eq!(Uranus::gravitational_parameter(), 5793951.256527211f64);
    }
    #[test]
    fn test_spheroid_799() {
        assert_eq!(Uranus::polar_radius(), 24973f64);
        assert_eq!(Uranus::mean_radius(), 25363.666666666668f64);
        assert_eq!(Uranus::equatorial_radius(), 25559f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_799() {
        assert_eq!(
            [257.311f64, 0f64, 0f64],
            Uranus::RIGHT_ASCENSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_799() {
        assert_eq!([-15.175f64, 0f64, 0f64], Uranus::DECLINATION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_799() {
        assert_eq!(
            [203.81f64, -501.1600928f64, 0f64],
            Uranus::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_899() {
        assert_eq!(Neptune::id(), 899i32)
    }
    #[test]
    fn test_point_mass_899() {
        assert_eq!(Neptune::gravitational_parameter(), 6835103.145462294f64);
    }
    #[test]
    fn test_spheroid_899() {
        assert_eq!(Neptune::polar_radius(), 24341f64);
        assert_eq!(Neptune::mean_radius(), 24623f64);
        assert_eq!(Neptune::equatorial_radius(), 24764f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_899() {
        assert_eq!(
            [299.36f64, 0f64, 0f64],
            Neptune::RIGHT_ASCENSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_899() {
        assert_eq!([43.46f64, 0f64, 0f64], Neptune::DECLINATION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_899() {
        assert_eq!(
            [249.978f64, 541.1397757f64, 0f64],
            Neptune::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_trig_rotational_elements_nut_prec_right_ascension_coefficients899() {
        assert_eq!(
            &[0.7f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64],
            Neptune::NUT_PREC_RIGHT_ASCENSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_trig_rotational_elements_nut_prec_declination_coefficients899() {
        assert_eq!(
            &[-0.51f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64],
            Neptune::NUT_PREC_DECLINATION_COEFFICIENTS
        )
    }
    #[test]
    fn test_trig_rotational_elements_nut_prec_prime_meridian_coefficients899() {
        assert_eq!(
            &[-0.48f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64],
            Neptune::NUT_PREC_PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_999() {
        assert_eq!(Pluto::id(), 999i32)
    }
    #[test]
    fn test_point_mass_999() {
        assert_eq!(Pluto::gravitational_parameter(), 869.6138177608748f64);
    }
    #[test]
    fn test_spheroid_999() {
        assert_eq!(Pluto::polar_radius(), 1188.3f64);
        assert_eq!(Pluto::mean_radius(), 1188.3f64);
        assert_eq!(Pluto::equatorial_radius(), 1188.3f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_999() {
        assert_eq!(
            [132.993f64, 0f64, 0f64],
            Pluto::RIGHT_ASCENSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_999() {
        assert_eq!([-6.163f64, 0f64, 0f64], Pluto::DECLINATION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_999() {
        assert_eq!(
            [302.695f64, 56.3625225f64, 0f64],
            Pluto::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
}
