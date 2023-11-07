/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at http://mozilla.org/MPL/2.0/.
 */

// Auto-generated by `lox_gen`. Do not edit!

use super::{
    BodyRotationalElements, Ellipsoid, NaifId, PointMass, PolynomialCoefficient, TriAxial,
};
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Gaspra;
impl NaifId for Gaspra {
    fn id() -> i32 {
        9511010i32
    }
}
impl Ellipsoid for Gaspra {
    fn polar_radius() -> f64 {
        4.4f64
    }
    fn mean_radius() -> f64 {
        6.233333333333334f64
    }
}
impl TriAxial for Gaspra {
    fn subplanetary_radius() -> f64 {
        9.1f64
    }
    fn along_orbit_radius() -> f64 {
        5.2f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Gaspra {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [9.47f64, 0f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [26.7f64, 0f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [83.67f64, 1226.911485f64, 0f64];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Ida;
impl NaifId for Ida {
    fn id() -> i32 {
        2431010i32
    }
}
impl Ellipsoid for Ida {
    fn polar_radius() -> f64 {
        7.6f64
    }
    fn mean_radius() -> f64 {
        15.466666666666667f64
    }
}
impl TriAxial for Ida {
    fn subplanetary_radius() -> f64 {
        26.8f64
    }
    fn along_orbit_radius() -> f64 {
        12f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Ida {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [168.76f64, 0f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [-87.12f64, 0f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [274.05f64, 1864.628007f64, 0f64];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Dactyl;
impl NaifId for Dactyl {
    fn id() -> i32 {
        2431011i32
    }
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Ceres;
impl NaifId for Ceres {
    fn id() -> i32 {
        2000001i32
    }
}
impl PointMass for Ceres {
    fn gravitational_parameter() -> f64 {
        62.62888864440993f64
    }
}
impl Ellipsoid for Ceres {
    fn polar_radius() -> f64 {
        446f64
    }
    fn mean_radius() -> f64 {
        473.5333333333333f64
    }
}
impl TriAxial for Ceres {
    fn subplanetary_radius() -> f64 {
        487.3f64
    }
    fn along_orbit_radius() -> f64 {
        487.3f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Ceres {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [291.418f64, 0f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [66.764f64, 0f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] = [170.65f64, 952.1532f64, 0f64];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Pallas;
impl NaifId for Pallas {
    fn id() -> i32 {
        2000002i32
    }
}
impl PointMass for Pallas {
    fn gravitational_parameter() -> f64 {
        13.665878145967422f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Pallas {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [33f64, 0f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [-3f64, 0f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] = [38f64, 1105.8036f64, 0f64];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Vesta;
impl NaifId for Vesta {
    fn id() -> i32 {
        2000004i32
    }
}
impl PointMass for Vesta {
    fn gravitational_parameter() -> f64 {
        17.288232879171513f64
    }
}
impl Ellipsoid for Vesta {
    fn polar_radius() -> f64 {
        229f64
    }
    fn mean_radius() -> f64 {
        266f64
    }
}
impl TriAxial for Vesta {
    fn subplanetary_radius() -> f64 {
        289f64
    }
    fn along_orbit_radius() -> f64 {
        280f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Vesta {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [309.031f64, 0f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [42.235f64, 0f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [285.39f64, 1617.3329428f64, 0f64];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Psyche;
impl NaifId for Psyche {
    fn id() -> i32 {
        2000016i32
    }
}
impl PointMass for Psyche {
    fn gravitational_parameter() -> f64 {
        1.5896582441709424f64
    }
}
impl Ellipsoid for Psyche {
    fn polar_radius() -> f64 {
        94.5f64
    }
    fn mean_radius() -> f64 {
        116.66666666666667f64
    }
}
impl TriAxial for Psyche {
    fn subplanetary_radius() -> f64 {
        139.5f64
    }
    fn along_orbit_radius() -> f64 {
        116f64
    }
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Lutetia;
impl NaifId for Lutetia {
    fn id() -> i32 {
        2000021i32
    }
}
impl Ellipsoid for Lutetia {
    fn polar_radius() -> f64 {
        46.5f64
    }
    fn mean_radius() -> f64 {
        53f64
    }
}
impl TriAxial for Lutetia {
    fn subplanetary_radius() -> f64 {
        62f64
    }
    fn along_orbit_radius() -> f64 {
        50.5f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Lutetia {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [52f64, 0f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [12f64, 0f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] = [94f64, 1057.7515f64, 0f64];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Kleopatra;
impl NaifId for Kleopatra {
    fn id() -> i32 {
        2000216i32
    }
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Eros;
impl NaifId for Eros {
    fn id() -> i32 {
        2000433i32
    }
}
impl PointMass for Eros {
    fn gravitational_parameter() -> f64 {
        0.0004463f64
    }
}
impl Ellipsoid for Eros {
    fn polar_radius() -> f64 {
        5.5f64
    }
    fn mean_radius() -> f64 {
        9.333333333333334f64
    }
}
impl TriAxial for Eros {
    fn subplanetary_radius() -> f64 {
        17f64
    }
    fn along_orbit_radius() -> f64 {
        5.5f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Eros {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [11.35f64, 0f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [17.22f64, 0f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [326.07f64, 1639.38864745f64, 0f64];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Davida;
impl NaifId for Davida {
    fn id() -> i32 {
        2000511i32
    }
}
impl PointMass for Davida {
    fn gravitational_parameter() -> f64 {
        3.8944831481705644f64
    }
}
impl Ellipsoid for Davida {
    fn polar_radius() -> f64 {
        127f64
    }
    fn mean_radius() -> f64 {
        151.33333333333334f64
    }
}
impl TriAxial for Davida {
    fn subplanetary_radius() -> f64 {
        180f64
    }
    fn along_orbit_radius() -> f64 {
        147f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Davida {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [297f64, 0f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [5f64, 0f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [268.1f64, 1684.4193549f64, 0f64];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Mathilde;
impl NaifId for Mathilde {
    fn id() -> i32 {
        2000253i32
    }
}
impl Ellipsoid for Mathilde {
    fn polar_radius() -> f64 {
        23f64
    }
    fn mean_radius() -> f64 {
        26.666666666666668f64
    }
}
impl TriAxial for Mathilde {
    fn subplanetary_radius() -> f64 {
        33f64
    }
    fn along_orbit_radius() -> f64 {
        24f64
    }
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Steins;
impl NaifId for Steins {
    fn id() -> i32 {
        2002867i32
    }
}
impl Ellipsoid for Steins {
    fn polar_radius() -> f64 {
        2.04f64
    }
    fn mean_radius() -> f64 {
        2.6700000000000004f64
    }
}
impl TriAxial for Steins {
    fn subplanetary_radius() -> f64 {
        3.24f64
    }
    fn along_orbit_radius() -> f64 {
        2.73f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Steins {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [91f64, 0f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [-62f64, 0f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] =
        [321.76f64, 1428.09917f64, 0f64];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Braille;
impl NaifId for Braille {
    fn id() -> i32 {
        2009969i32
    }
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct WilsonHarrington;
impl NaifId for WilsonHarrington {
    fn id() -> i32 {
        2004015i32
    }
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Toutatis;
impl NaifId for Toutatis {
    fn id() -> i32 {
        2004179i32
    }
}
impl Ellipsoid for Toutatis {
    fn polar_radius() -> f64 {
        0.85f64
    }
    fn mean_radius() -> f64 {
        1.3316666666666666f64
    }
}
impl TriAxial for Toutatis {
    fn subplanetary_radius() -> f64 {
        2.13f64
    }
    fn along_orbit_radius() -> f64 {
        1.015f64
    }
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Itokawa;
impl NaifId for Itokawa {
    fn id() -> i32 {
        2025143i32
    }
}
impl Ellipsoid for Itokawa {
    fn polar_radius() -> f64 {
        0.104f64
    }
    fn mean_radius() -> f64 {
        0.17300000000000001f64
    }
}
impl TriAxial for Itokawa {
    fn subplanetary_radius() -> f64 {
        0.268f64
    }
    fn along_orbit_radius() -> f64 {
        0.147f64
    }
}
#[allow(clippy::approx_constant)]
impl BodyRotationalElements for Itokawa {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = [90.53f64, 0f64, 0f64];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = [-66.3f64, 0f64, 0f64];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] = [0f64, 712.143f64, 0f64];
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Bennu;
impl NaifId for Bennu {
    fn id() -> i32 {
        2101955i32
    }
}
#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;
    #[test]
    fn test_naif_id_9511010() {
        assert_eq!(Gaspra::id(), 9511010i32)
    }
    #[test]
    fn test_tri_axial_9511010() {
        assert_eq!(Gaspra::polar_radius(), 4.4f64);
        assert_eq!(Gaspra::mean_radius(), 6.233333333333334f64);
        assert_eq!(Gaspra::subplanetary_radius(), 9.1f64);
        assert_eq!(Gaspra::along_orbit_radius(), 5.2f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_9511010() {
        assert_eq!([9.47f64, 0f64, 0f64], Gaspra::RIGHT_ASCENSION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_9511010() {
        assert_eq!([26.7f64, 0f64, 0f64], Gaspra::DECLINATION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_9511010() {
        assert_eq!(
            [83.67f64, 1226.911485f64, 0f64],
            Gaspra::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_2431010() {
        assert_eq!(Ida::id(), 2431010i32)
    }
    #[test]
    fn test_tri_axial_2431010() {
        assert_eq!(Ida::polar_radius(), 7.6f64);
        assert_eq!(Ida::mean_radius(), 15.466666666666667f64);
        assert_eq!(Ida::subplanetary_radius(), 26.8f64);
        assert_eq!(Ida::along_orbit_radius(), 12f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_2431010() {
        assert_eq!([168.76f64, 0f64, 0f64], Ida::RIGHT_ASCENSION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_2431010() {
        assert_eq!([-87.12f64, 0f64, 0f64], Ida::DECLINATION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_2431010() {
        assert_eq!(
            [274.05f64, 1864.628007f64, 0f64],
            Ida::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_2431011() {
        assert_eq!(Dactyl::id(), 2431011i32)
    }
    #[test]
    fn test_naif_id_2000001() {
        assert_eq!(Ceres::id(), 2000001i32)
    }
    #[test]
    fn test_point_mass_2000001() {
        assert_eq!(Ceres::gravitational_parameter(), 62.62888864440993f64);
    }
    #[test]
    fn test_tri_axial_2000001() {
        assert_eq!(Ceres::polar_radius(), 446f64);
        assert_eq!(Ceres::mean_radius(), 473.5333333333333f64);
        assert_eq!(Ceres::subplanetary_radius(), 487.3f64);
        assert_eq!(Ceres::along_orbit_radius(), 487.3f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_2000001() {
        assert_eq!(
            [291.418f64, 0f64, 0f64],
            Ceres::RIGHT_ASCENSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_2000001() {
        assert_eq!([66.764f64, 0f64, 0f64], Ceres::DECLINATION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_2000001() {
        assert_eq!(
            [170.65f64, 952.1532f64, 0f64],
            Ceres::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_2000002() {
        assert_eq!(Pallas::id(), 2000002i32)
    }
    #[test]
    fn test_point_mass_2000002() {
        assert_eq!(Pallas::gravitational_parameter(), 13.665878145967422f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_2000002() {
        assert_eq!([33f64, 0f64, 0f64], Pallas::RIGHT_ASCENSION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_2000002() {
        assert_eq!([-3f64, 0f64, 0f64], Pallas::DECLINATION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_2000002() {
        assert_eq!(
            [38f64, 1105.8036f64, 0f64],
            Pallas::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_2000004() {
        assert_eq!(Vesta::id(), 2000004i32)
    }
    #[test]
    fn test_point_mass_2000004() {
        assert_eq!(Vesta::gravitational_parameter(), 17.288232879171513f64);
    }
    #[test]
    fn test_tri_axial_2000004() {
        assert_eq!(Vesta::polar_radius(), 229f64);
        assert_eq!(Vesta::mean_radius(), 266f64);
        assert_eq!(Vesta::subplanetary_radius(), 289f64);
        assert_eq!(Vesta::along_orbit_radius(), 280f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_2000004() {
        assert_eq!(
            [309.031f64, 0f64, 0f64],
            Vesta::RIGHT_ASCENSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_2000004() {
        assert_eq!([42.235f64, 0f64, 0f64], Vesta::DECLINATION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_2000004() {
        assert_eq!(
            [285.39f64, 1617.3329428f64, 0f64],
            Vesta::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_2000016() {
        assert_eq!(Psyche::id(), 2000016i32)
    }
    #[test]
    fn test_point_mass_2000016() {
        assert_eq!(Psyche::gravitational_parameter(), 1.5896582441709424f64);
    }
    #[test]
    fn test_tri_axial_2000016() {
        assert_eq!(Psyche::polar_radius(), 94.5f64);
        assert_eq!(Psyche::mean_radius(), 116.66666666666667f64);
        assert_eq!(Psyche::subplanetary_radius(), 139.5f64);
        assert_eq!(Psyche::along_orbit_radius(), 116f64);
    }
    #[test]
    fn test_naif_id_2000021() {
        assert_eq!(Lutetia::id(), 2000021i32)
    }
    #[test]
    fn test_tri_axial_2000021() {
        assert_eq!(Lutetia::polar_radius(), 46.5f64);
        assert_eq!(Lutetia::mean_radius(), 53f64);
        assert_eq!(Lutetia::subplanetary_radius(), 62f64);
        assert_eq!(Lutetia::along_orbit_radius(), 50.5f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_2000021() {
        assert_eq!([52f64, 0f64, 0f64], Lutetia::RIGHT_ASCENSION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_2000021() {
        assert_eq!([12f64, 0f64, 0f64], Lutetia::DECLINATION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_2000021() {
        assert_eq!(
            [94f64, 1057.7515f64, 0f64],
            Lutetia::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_2000216() {
        assert_eq!(Kleopatra::id(), 2000216i32)
    }
    #[test]
    fn test_naif_id_2000433() {
        assert_eq!(Eros::id(), 2000433i32)
    }
    #[test]
    fn test_point_mass_2000433() {
        assert_eq!(Eros::gravitational_parameter(), 0.0004463f64);
    }
    #[test]
    fn test_tri_axial_2000433() {
        assert_eq!(Eros::polar_radius(), 5.5f64);
        assert_eq!(Eros::mean_radius(), 9.333333333333334f64);
        assert_eq!(Eros::subplanetary_radius(), 17f64);
        assert_eq!(Eros::along_orbit_radius(), 5.5f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_2000433() {
        assert_eq!([11.35f64, 0f64, 0f64], Eros::RIGHT_ASCENSION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_2000433() {
        assert_eq!([17.22f64, 0f64, 0f64], Eros::DECLINATION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_2000433() {
        assert_eq!(
            [326.07f64, 1639.38864745f64, 0f64],
            Eros::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_2000511() {
        assert_eq!(Davida::id(), 2000511i32)
    }
    #[test]
    fn test_point_mass_2000511() {
        assert_eq!(Davida::gravitational_parameter(), 3.8944831481705644f64);
    }
    #[test]
    fn test_tri_axial_2000511() {
        assert_eq!(Davida::polar_radius(), 127f64);
        assert_eq!(Davida::mean_radius(), 151.33333333333334f64);
        assert_eq!(Davida::subplanetary_radius(), 180f64);
        assert_eq!(Davida::along_orbit_radius(), 147f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_2000511() {
        assert_eq!([297f64, 0f64, 0f64], Davida::RIGHT_ASCENSION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_2000511() {
        assert_eq!([5f64, 0f64, 0f64], Davida::DECLINATION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_2000511() {
        assert_eq!(
            [268.1f64, 1684.4193549f64, 0f64],
            Davida::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_2000253() {
        assert_eq!(Mathilde::id(), 2000253i32)
    }
    #[test]
    fn test_tri_axial_2000253() {
        assert_eq!(Mathilde::polar_radius(), 23f64);
        assert_eq!(Mathilde::mean_radius(), 26.666666666666668f64);
        assert_eq!(Mathilde::subplanetary_radius(), 33f64);
        assert_eq!(Mathilde::along_orbit_radius(), 24f64);
    }
    #[test]
    fn test_naif_id_2002867() {
        assert_eq!(Steins::id(), 2002867i32)
    }
    #[test]
    fn test_tri_axial_2002867() {
        assert_eq!(Steins::polar_radius(), 2.04f64);
        assert_eq!(Steins::mean_radius(), 2.6700000000000004f64);
        assert_eq!(Steins::subplanetary_radius(), 3.24f64);
        assert_eq!(Steins::along_orbit_radius(), 2.73f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_2002867() {
        assert_eq!([91f64, 0f64, 0f64], Steins::RIGHT_ASCENSION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_2002867() {
        assert_eq!([-62f64, 0f64, 0f64], Steins::DECLINATION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_2002867() {
        assert_eq!(
            [321.76f64, 1428.09917f64, 0f64],
            Steins::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_2009969() {
        assert_eq!(Braille::id(), 2009969i32)
    }
    #[test]
    fn test_naif_id_2004015() {
        assert_eq!(WilsonHarrington::id(), 2004015i32)
    }
    #[test]
    fn test_naif_id_2004179() {
        assert_eq!(Toutatis::id(), 2004179i32)
    }
    #[test]
    fn test_tri_axial_2004179() {
        assert_eq!(Toutatis::polar_radius(), 0.85f64);
        assert_eq!(Toutatis::mean_radius(), 1.3316666666666666f64);
        assert_eq!(Toutatis::subplanetary_radius(), 2.13f64);
        assert_eq!(Toutatis::along_orbit_radius(), 1.015f64);
    }
    #[test]
    fn test_naif_id_2025143() {
        assert_eq!(Itokawa::id(), 2025143i32)
    }
    #[test]
    fn test_tri_axial_2025143() {
        assert_eq!(Itokawa::polar_radius(), 0.104f64);
        assert_eq!(Itokawa::mean_radius(), 0.17300000000000001f64);
        assert_eq!(Itokawa::subplanetary_radius(), 0.268f64);
        assert_eq!(Itokawa::along_orbit_radius(), 0.147f64);
    }
    #[test]
    fn test_rotational_elements_right_ascension_coefficients_2025143() {
        assert_eq!(
            [90.53f64, 0f64, 0f64],
            Itokawa::RIGHT_ASCENSION_COEFFICIENTS
        )
    }
    #[test]
    fn test_rotational_elements_declination_coefficients_2025143() {
        assert_eq!([-66.3f64, 0f64, 0f64], Itokawa::DECLINATION_COEFFICIENTS)
    }
    #[test]
    fn test_rotational_elements_prime_meridian_coefficients_2025143() {
        assert_eq!(
            [0f64, 712.143f64, 0f64],
            Itokawa::PRIME_MERIDIAN_COEFFICIENTS
        )
    }
    #[test]
    fn test_naif_id_2101955() {
        assert_eq!(Bennu::id(), 2101955i32)
    }
}
