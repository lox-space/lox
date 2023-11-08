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

    use crate::bodies::planets::Jupiter;

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

    struct SomeBody;

    impl RotationalElements for SomeBody {
        fn nutation_precession_coefficients() -> NutationPrecessionCoefficients {
            (
                &[
                    1.2796754075622423,
                    0.42970006184100396,
                    4.9549897464119015,
                    6.2098814785958245,
                    2.092649773141201,
                    4.010766621082969,
                    6.147922290150026,
                    1.9783307071355725,
                    2.5593508151244846,
                    0.8594001236820079,
                    1.734171606432425,
                    3.0699533280603655,
                    5.241627996900319,
                    1.9898901100379935,
                    0.864134346731335,
                    0.0,
                    0.0,
                    0.0,
                ],
                &[
                    1596.503281347521,
                    787.7927551311844,
                    84.66068602648895,
                    20.792107379008446,
                    4.574507969477138,
                    1.1222467090323538,
                    41.58421475801689,
                    105.9414855960558,
                    3193.006562695042,
                    1575.5855102623689,
                    84.65553032387855,
                    20.80363527871787,
                    4.582318317879813,
                    105.94580703128374,
                    1.1222467090323538,
                    0.0,
                    0.0,
                    0.0,
                ],
            )
        }

        fn right_ascension_coefficients() -> PolynomialCoefficients {
            (
                4.6784701644349695,
                -0.00011342894808711148,
                0.0,
                &[
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    2.0420352248333656e-6,
                    1.6371188383706813e-5,
                    2.4993114888558796e-5,
                    5.235987755982989e-7,
                    3.752457891787809e-5,
                    0.0,
                    0.0,
                    0.0,
                ],
            )
        }

        fn declination_coefficients() -> PolynomialCoefficients {
            (
                1.1256553894213766,
                4.211479485062318e-5,
                0.0,
                &[
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    8.726646259971648e-7,
                    7.051130178057092e-6,
                    1.0768681484805013e-5,
                    -2.2689280275926283e-7,
                    1.616174887346749e-5,
                    0.0,
                    0.0,
                    0.0,
                ],
            )
        }

        fn prime_meridian_coefficients() -> PolynomialCoefficients {
            (4.973315703557842, 15.193719457141356, 0.0, &[0.0; 3])
        }
    }

    #[test]
    fn test_rotational_elements_right_ascension() {
        assert_float_eq!(
            SomeBody::right_ascension(0.0),
            4.678480799964803,
            rel <= 1e-8
        );
    }

    #[test]
    fn test_rotational_elements_right_ascension_dot() {
        assert_float_eq!(
            SomeBody::right_ascension_dot(0.0),
            -1.1342894808711148e-7,
            rel <= 1e-8
        );
    }

    #[test]
    fn test_rotational_elements_declination() {
        assert_float_eq!(SomeBody::declination(0.0), 1.1256642372977634, rel <= 1e-8);
    }

    #[test]
    fn test_rotational_elements_declination_dot() {
        assert_float_eq!(
            SomeBody::declination_dot(0.0),
            4.211479485062318e-5,
            rel <= 1e-8
        );
    }

    #[test]
    fn test_rotational_elements_prime_meridian() {
        assert_float_eq!(
            SomeBody::prime_meridian(0.0),
            4.973315703557842,
            rel <= 1e-8
        );
    }

    #[test]
    fn test_rotational_elements_prime_meridian_dot() {
        assert_float_eq!(
            SomeBody::prime_meridian_dot(0.0),
            15.193719457141356,
            rel <= 1e-8
        );
    }
}
