/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::time::constants::f64::{SECONDS_PER_DAY, SECONDS_PER_JULIAN_CENTURY};

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

pub type PolynomialCoefficients = (f64, f64, f64, &'static [f64]);

pub type NutationPrecessionCoefficients = (&'static [f64], &'static [f64]);

pub trait RotationalElements {
    fn nutation_precession_coefficients() -> NutationPrecessionCoefficients;

    fn right_ascension_coefficients() -> PolynomialCoefficients;

    fn declination_coefficients() -> PolynomialCoefficients;

    fn prime_meridian_coefficients() -> PolynomialCoefficients;

    fn theta(t: f64) -> Vec<f64> {
        let t = t / SECONDS_PER_JULIAN_CENTURY;
        let (theta0, theta1) = Self::nutation_precession_coefficients();
        let mut theta = vec![0.0; theta0.len()];
        if theta0.is_empty() {
            return theta;
        }

        for i in 0..theta.len() {
            theta[i] = theta0[i] + theta1[i] * t;
        }
        theta
    }

    fn right_ascension(t: f64) -> f64 {
        let dt = SECONDS_PER_JULIAN_CENTURY;
        let (c0, c1, c2, c) = Self::right_ascension_coefficients();
        let theta = Self::theta(t);
        let mut c_trig = vec![0.0; c.len()];
        if !c.is_empty() {
            for i in 0..c.len() {
                c_trig[i] = c[i] * theta[i].sin();
            }
        }
        let c_trig: f64 = c_trig.iter().sum();
        c0 + c1 * t / dt + c2 * t.powi(2) / dt.powi(2) + c_trig
    }

    fn right_ascension_dot(t: f64) -> f64 {
        let dt = SECONDS_PER_JULIAN_CENTURY;
        let (_, c1, c2, c) = Self::right_ascension_coefficients();
        let (_, theta1) = Self::nutation_precession_coefficients();
        let theta = Self::theta(t);
        let mut c_trig = vec![0.0; c.len()];
        if !c.is_empty() {
            for i in 0..c.len() {
                c_trig[i] = c[i] * theta1[i] / SECONDS_PER_JULIAN_CENTURY * theta[i].cos()
            }
        }
        let c_trig: f64 = c_trig.iter().sum();
        c1 / dt + 2.0 * c2 * t / dt.powi(2) + c_trig
    }

    fn declination(t: f64) -> f64 {
        let dt = SECONDS_PER_JULIAN_CENTURY;
        let (c0, c1, c2, c) = Self::declination_coefficients();
        let theta = Self::theta(t);
        let mut c_trig = vec![0.0; c.len()];
        if !c.is_empty() {
            for i in 0..c.len() {
                c_trig[i] = c[i] * theta[i].cos();
            }
        }
        let c_trig: f64 = c_trig.iter().sum();
        c0 + c1 * t / dt + c2 * t.powi(2) / dt.powi(2) + c_trig
    }

    fn declination_dot(t: f64) -> f64 {
        let dt = SECONDS_PER_JULIAN_CENTURY;
        let (_, c1, c2, c) = Self::declination_coefficients();
        let (_, theta1) = Self::nutation_precession_coefficients();
        let theta = Self::theta(t);
        let mut c_trig = vec![0.0; c.len()];
        if !c.is_empty() {
            for i in 0..c.len() {
                c_trig[i] = c[i] * theta1[i] / SECONDS_PER_JULIAN_CENTURY * theta[i].sin()
            }
        }
        let c_trig: f64 = c_trig.iter().sum();
        c1 / dt + 2.0 * c2 * t / dt.powi(2) - c_trig
    }

    fn prime_meridian(t: f64) -> f64 {
        let dt = SECONDS_PER_DAY;
        let (c0, c1, c2, c) = Self::prime_meridian_coefficients();
        let theta = Self::theta(t);
        let mut c_trig = vec![0.0; c.len()];
        if !c.is_empty() {
            for i in 0..c.len() {
                c_trig[i] = c[i] * theta[i].sin();
            }
        }
        let c_trig: f64 = c_trig.iter().sum();
        c0 + c1 * t / dt + c2 * t.powi(2) / dt.powi(2) + c_trig
    }

    fn prime_meridian_dot(t: f64) -> f64 {
        let dt = SECONDS_PER_DAY;
        let (_, c1, c2, c) = Self::prime_meridian_coefficients();
        let (_, theta1) = Self::nutation_precession_coefficients();
        let theta = Self::theta(t);
        let mut c_trig = vec![0.0; c.len()];
        if !c.is_empty() {
            for i in 0..c.len() {
                c_trig[i] = c[i] * theta1[i] / SECONDS_PER_JULIAN_CENTURY * theta[i].cos()
            }
        }
        let c_trig: f64 = c_trig.iter().sum();
        c1 / dt + 2.0 * c2 * t / dt.powi(2) + c_trig
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

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

    // Jupiter is manually defined with known data here to avoid depending on the correctness of the
    // PCK parser to test RotationalElements.
    struct Jupiter;

    impl RotationalElements for Jupiter {
        fn nutation_precession_coefficients() -> NutationPrecessionCoefficients {
            (
                &[
                    1.2796754075622423f64,
                    0.42970006184100396f64,
                    4.9549897464119015f64,
                    6.2098814785958245f64,
                    2.092649773141201f64,
                    4.010766621082969f64,
                    6.147922290150026f64,
                    1.9783307071355725f64,
                    2.5593508151244846f64,
                    0.8594001236820079f64,
                    1.734171606432425f64,
                    3.0699533280603655f64,
                    5.241627996900319f64,
                    1.9898901100379935f64,
                    0.864134346731335f64,
                ],
                &[
                    1596.503281347521f64,
                    787.7927551311844f64,
                    84.66068602648895f64,
                    20.792107379008446f64,
                    4.574507969477138f64,
                    1.1222467090323538f64,
                    41.58421475801689f64,
                    105.9414855960558f64,
                    3193.006562695042f64,
                    1575.5855102623689f64,
                    84.65553032387855f64,
                    20.80363527871787f64,
                    4.582318317879813f64,
                    105.94580703128374f64,
                    1.1222467090323538f64,
                ],
            )
        }

        fn right_ascension_coefficients() -> PolynomialCoefficients {
            (
                4.6784701644349695f64,
                -0.00011342894808711148f64,
                0f64,
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
                    0.0000020420352248333656f64,
                    0.000016371188383706813f64,
                    0.000024993114888558796f64,
                    0.0000005235987755982989f64,
                    0.00003752457891787809f64,
                ],
            )
        }

        fn declination_coefficients() -> PolynomialCoefficients {
            (
                1.1256553894213766f64,
                0.00004211479485062318f64,
                0f64,
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
                    0.0000008726646259971648f64,
                    0.000007051130178057092f64,
                    0.000010768681484805013f64,
                    -0.00000022689280275926283f64,
                    0.00001616174887346749f64,
                ],
            )
        }

        fn prime_meridian_coefficients() -> PolynomialCoefficients {
            (
                4.973315703557842f64,
                15.193719457141356f64,
                0f64,
                &[
                    0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64,
                    0f64, 0f64,
                ],
            )
        }
    }

    #[test]
    fn test_rotational_elements_right_ascension() {
        assert_float_eq!(
            Jupiter::right_ascension(0.0),
            4.678480799964803,
            rel <= 1e-8
        );
    }

    #[test]
    fn test_rotational_elements_right_ascension_dot() {
        assert_float_eq!(
            Jupiter::right_ascension_dot(0.0),
            -1.3266588500099516e-13,
            rel <= 1e-8
        );
    }

    #[test]
    fn test_rotational_elements_declination() {
        assert_float_eq!(Jupiter::declination(0.0), 1.1256642372977634, rel <= 1e-8);
    }

    #[test]
    fn test_rotational_elements_declination_dot() {
        assert_float_eq!(
            Jupiter::declination_dot(0.0),
            3.004482367136341e-15,
            rel <= 1e-8
        );
    }

    #[test]
    fn test_rotational_elements_prime_meridian() {
        assert_float_eq!(Jupiter::prime_meridian(0.0), 4.973315703557842, rel <= 1e-8);
    }

    #[test]
    fn test_rotational_elements_prime_meridian_dot() {
        assert_float_eq!(
            Jupiter::prime_meridian_dot(0.0),
            0.00017585323445765458,
            rel <= 1e-8
        );
    }
}
