/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

pub mod barycenters;
pub mod minor;
pub mod planets;
pub mod satellites;
pub mod sun;

pub trait NaifId: Copy {
    fn id() -> i32;
}

pub fn naif_id<T: NaifId>(_: T) -> i32 {
    <T as NaifId>::id()
}

pub trait Ellipsoid: Copy {
    fn polar_radius() -> f64;
    fn mean_radius() -> f64;
}

pub fn polar_radius<T: Ellipsoid>(_: T) -> f64 {
    <T as Ellipsoid>::polar_radius()
}

pub fn mean_radius<T: Ellipsoid>(_: T) -> f64 {
    <T as Ellipsoid>::mean_radius()
}

pub trait Spheroid: Ellipsoid {
    fn equatorial_radius() -> f64;
}

pub fn equatorial_radius<T: Spheroid>(_: T) -> f64 {
    <T as Spheroid>::equatorial_radius()
}

pub trait TriAxial: Ellipsoid {
    fn subplanetary_radius() -> f64;
    fn along_orbit_radius() -> f64;
}

pub fn subplanetary_radius<T: TriAxial>(_: T) -> f64 {
    <T as TriAxial>::subplanetary_radius()
}

pub fn along_orbit_radius<T: TriAxial>(_: T) -> f64 {
    <T as TriAxial>::along_orbit_radius()
}

pub trait PointMass: Copy {
    fn gravitational_parameter() -> f64;
}

pub fn gravitational_parameter<T: PointMass>(_: T) -> f64 {
    <T as PointMass>::gravitational_parameter()
}

/// Semantic type alias for f64 in the context of polynomial types and operations.
pub type PolynomialCoefficient = f64;

pub trait RotationalElements: Copy {
    const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3];
    const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3];
    const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3];
}

pub trait TrigonometricRotationalElements: RotationalElements {
    const NUT_PREC_RIGHT_ASCENSION_COEFFICIENTS: &'static [PolynomialCoefficient];
    const NUT_PREC_DECLINATION_COEFFICIENTS: &'static [PolynomialCoefficient];
    const NUT_PREC_PRIME_MERIDIAN_COEFFICIENTS: &'static [PolynomialCoefficient];
}

#[cfg(test)]
mod tests {
    use super::planets::Earth;
    use super::satellites::Moon;
    use super::*;

    #[test]
    fn test_naif_id() {
        assert_eq!(naif_id(Earth), Earth::id());
    }

    #[test]
    fn test_grav_param() {
        assert_eq!(
            gravitational_parameter(Earth),
            Earth::gravitational_parameter()
        );
    }

    #[test]
    fn test_mean_radius() {
        assert_eq!(mean_radius(Earth), Earth::mean_radius());
    }

    #[test]
    fn test_polar_radius() {
        assert_eq!(polar_radius(Earth), Earth::polar_radius());
    }

    #[test]
    fn test_equatorial_radius() {
        assert_eq!(equatorial_radius(Earth), Earth::equatorial_radius());
    }

    #[test]
    fn test_subplanetary_radius() {
        assert_eq!(subplanetary_radius(Moon), Moon::subplanetary_radius());
    }

    #[test]
    fn test_along_orbit_radius() {
        assert_eq!(along_orbit_radius(Moon), Moon::along_orbit_radius());
    }
}
