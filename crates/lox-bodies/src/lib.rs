/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::f64::consts::PI;
use std::fmt::{Display, Formatter};

pub use crate::dynamic::DynOrigin;
pub use generated::*;
use lox_math::constants::f64::time::{SECONDS_PER_DAY, SECONDS_PER_JULIAN_CENTURY};

pub mod dynamic;
pub mod fundamental;
#[allow(clippy::approx_constant)]
mod generated;
#[cfg(feature = "python")]
pub mod python;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct NaifId(pub i32);

impl Display for NaifId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// `Origin` is implemented for all bodies and barycenters.
pub trait Origin {
    fn id(&self) -> NaifId;
    fn name(&self) -> &'static str;
}

pub type Radii = (f64, f64, f64);

pub trait MaybeTriaxialEllipsoid: Origin {
    fn maybe_radii(&self) -> Option<Radii>;
}

pub trait TriaxialEllipsoid: Origin {
    fn radii(&self) -> Radii;
}

impl<T: TriaxialEllipsoid> MaybeTriaxialEllipsoid for T {
    fn maybe_radii(&self) -> Option<Radii> {
        Some(self.radii())
    }
}

fn flattening(equatorial_radius: f64, polar_radius: f64) -> f64 {
    (equatorial_radius - polar_radius) / equatorial_radius
}

pub trait MaybeSpheroid: MaybeTriaxialEllipsoid {
    fn maybe_polar_radius(&self) -> Option<f64> {
        self.maybe_radii().map(|radii| radii.0)
    }

    fn maybe_equatorial_radius(&self) -> Option<f64> {
        self.maybe_radii().map(|radii| radii.2)
    }

    fn maybe_flattening(&self) -> Option<f64> {
        self.maybe_radii().map(|radii| flattening(radii.0, radii.2))
    }
}

pub trait Spheroid: TriaxialEllipsoid {
    fn equatorial_radius(&self) -> f64 {
        self.radii().0
    }

    fn polar_radius(&self) -> f64 {
        self.radii().2
    }

    fn flattening(&self) -> f64 {
        flattening(self.equatorial_radius(), self.polar_radius())
    }
}

impl<T: Spheroid> MaybeSpheroid for T {
    fn maybe_equatorial_radius(&self) -> Option<f64> {
        Some(self.equatorial_radius())
    }

    fn maybe_polar_radius(&self) -> Option<f64> {
        Some(self.polar_radius())
    }

    fn maybe_flattening(&self) -> Option<f64> {
        Some(self.flattening())
    }
}

pub trait MaybeMeanRadius: Origin {
    fn maybe_mean_radius(&self) -> Option<f64>;
}

pub trait MeanRadius: Origin {
    fn mean_radius(&self) -> f64;
}

impl<T: MeanRadius> MaybeMeanRadius for T {
    fn maybe_mean_radius(&self) -> Option<f64> {
        Some(self.mean_radius())
    }
}

pub trait PointMass: Origin {
    fn gravitational_parameter(&self) -> f64;
}

pub trait MaybePointMass: Origin {
    fn maybe_gravitational_parameter(&self) -> Option<f64>;
}

impl<T: PointMass> MaybePointMass for T {
    fn maybe_gravitational_parameter(&self) -> Option<f64> {
        Some(self.gravitational_parameter())
    }
}

pub type PolynomialCoefficients = (f64, f64, f64, Option<&'static [f64]>);

pub type NutationPrecessionCoefficients = (&'static [f64], &'static [f64]);

type Elements = (f64, f64, f64);

fn theta(nut_prec: &NutationPrecessionCoefficients, t: f64) -> Vec<f64> {
    let t = t / SECONDS_PER_JULIAN_CENTURY;
    let (theta0, theta1) = nut_prec;
    let mut theta = vec![0.0; theta0.len()];
    if theta0.is_empty() {
        return theta;
    }

    for i in 0..theta.len() {
        theta[i] = theta0[i] + theta1[i] * t;
    }
    theta
}

fn trig_term<F: Fn(f64) -> f64>(
    coeffs: &[f64],
    nut_prec: &NutationPrecessionCoefficients,
    t: f64,
    sincos: F,
) -> f64 {
    let theta = theta(nut_prec, t);
    coeffs
        .iter()
        .enumerate()
        .map(|(i, coeff)| coeff * sincos(theta[i]))
        .sum()
}

fn trig_term_dot<F: Fn(f64) -> f64>(
    coeffs: &[f64],
    nut_prec: &NutationPrecessionCoefficients,
    t: f64,
    sincos: F,
) -> f64 {
    let (_, theta1) = nut_prec;
    let theta = theta(nut_prec, t);
    coeffs
        .iter()
        .enumerate()
        .map(|(i, coeff)| coeff * theta1[i] / SECONDS_PER_JULIAN_CENTURY * sincos(theta[i]))
        .sum()
}

fn right_ascension(
    poly: &PolynomialCoefficients,
    nut_prec: &NutationPrecessionCoefficients,
    t: f64,
) -> f64 {
    let dt = SECONDS_PER_JULIAN_CENTURY;
    let (c0, c1, c2, c) = poly;
    let c_trig = c
        .map(|c| trig_term(c, nut_prec, t, f64::sin))
        .unwrap_or_default();
    c0 + c1 * t / dt + c2 * t.powi(2) / dt.powi(2) + c_trig
}

fn right_ascension_dot(
    poly: &PolynomialCoefficients,
    nut_prec: &NutationPrecessionCoefficients,
    t: f64,
) -> f64 {
    let dt = SECONDS_PER_JULIAN_CENTURY;
    let (_, c1, c2, c) = poly;
    let c_trig = c
        .map(|c| trig_term_dot(c, nut_prec, t, f64::cos))
        .unwrap_or_default();
    c1 / dt + 2.0 * c2 * t / dt.powi(2) + c_trig
}

fn declination(
    poly: &PolynomialCoefficients,
    nut_prec: &NutationPrecessionCoefficients,
    t: f64,
) -> f64 {
    let dt = SECONDS_PER_JULIAN_CENTURY;
    let (c0, c1, c2, c) = poly;
    let c_trig = c
        .map(|c| trig_term(c, nut_prec, t, f64::cos))
        .unwrap_or_default();
    c0 + c1 * t / dt + c2 * t.powi(2) / dt.powi(2) + c_trig
}

fn declination_dot(
    poly: &PolynomialCoefficients,
    nut_prec: &NutationPrecessionCoefficients,
    t: f64,
) -> f64 {
    let dt = SECONDS_PER_JULIAN_CENTURY;
    let (_, c1, c2, c) = poly;
    let c_trig = c
        .map(|c| trig_term_dot(c, nut_prec, t, f64::sin))
        .unwrap_or_default();
    c1 / dt + 2.0 * c2 * t / dt.powi(2) - c_trig
}

fn prime_meridian(
    poly: &PolynomialCoefficients,
    nut_prec: &NutationPrecessionCoefficients,
    t: f64,
) -> f64 {
    let dt = SECONDS_PER_DAY;
    let (c0, c1, c2, c) = poly;
    let c_trig = c
        .map(|c| trig_term(c, nut_prec, t, f64::sin))
        .unwrap_or_default();
    c0 + c1 * t / dt + c2 * t.powi(2) / dt.powi(2) + c_trig
}

fn prime_meridian_dot(
    poly: &PolynomialCoefficients,
    nut_prec: &NutationPrecessionCoefficients,
    t: f64,
) -> f64 {
    let dt = SECONDS_PER_DAY;
    let (_, c1, c2, c) = poly;
    let c_trig = c
        .map(|c| trig_term_dot(c, nut_prec, t, f64::cos))
        .unwrap_or_default();
    c1 / dt + 2.0 * c2 * t / dt.powi(2) + c_trig
}

fn rotational_elements(right_ascension: f64, declination: f64, prime_meridian: f64) -> Elements {
    (
        right_ascension + PI / 2.0,
        PI / 2.0 - declination,
        prime_meridian % (2.0 * PI),
    )
}

fn rotational_elements_rates(
    right_ascension_dot: f64,
    declination_dot: f64,
    prime_meridian_dot: f64,
) -> Elements {
    (right_ascension_dot, -declination_dot, prime_meridian_dot)
}

pub trait MaybeRotationalElements: Origin {
    fn maybe_nutation_precession_coefficients(&self) -> Option<NutationPrecessionCoefficients>;
    fn maybe_right_ascension_coefficients(&self) -> Option<PolynomialCoefficients>;
    fn maybe_declination_coefficients(&self) -> Option<PolynomialCoefficients>;
    fn maybe_prime_meridian_coefficients(&self) -> Option<PolynomialCoefficients>;

    fn maybe_right_ascension(&self, t: f64) -> Option<f64> {
        let (Some(poly), Some(nut_prec)) = (
            self.maybe_right_ascension_coefficients(),
            self.maybe_nutation_precession_coefficients(),
        ) else {
            return None;
        };
        Some(right_ascension(&poly, &nut_prec, t))
    }

    fn maybe_right_ascension_dot(&self, t: f64) -> Option<f64> {
        let (Some(poly), Some(nut_prec)) = (
            self.maybe_right_ascension_coefficients(),
            self.maybe_nutation_precession_coefficients(),
        ) else {
            return None;
        };
        Some(right_ascension_dot(&poly, &nut_prec, t))
    }

    fn maybe_declination(&self, t: f64) -> Option<f64> {
        let (Some(poly), Some(nut_prec)) = (
            self.maybe_declination_coefficients(),
            self.maybe_nutation_precession_coefficients(),
        ) else {
            return None;
        };
        Some(declination(&poly, &nut_prec, t))
    }

    fn maybe_declination_dot(&self, t: f64) -> Option<f64> {
        let (Some(poly), Some(nut_prec)) = (
            self.maybe_declination_coefficients(),
            self.maybe_nutation_precession_coefficients(),
        ) else {
            return None;
        };
        Some(declination_dot(&poly, &nut_prec, t))
    }

    fn maybe_prime_meridian(&self, t: f64) -> Option<f64> {
        let (Some(poly), Some(nut_prec)) = (
            self.maybe_prime_meridian_coefficients(),
            self.maybe_nutation_precession_coefficients(),
        ) else {
            return None;
        };
        Some(prime_meridian(&poly, &nut_prec, t))
    }

    fn maybe_prime_meridian_dot(&self, t: f64) -> Option<f64> {
        let (Some(poly), Some(nut_prec)) = (
            self.maybe_prime_meridian_coefficients(),
            self.maybe_nutation_precession_coefficients(),
        ) else {
            return None;
        };
        Some(prime_meridian_dot(&poly, &nut_prec, t))
    }

    fn maybe_rotational_elements(&self, t: f64) -> Option<Elements> {
        let (Some(right_ascension), Some(declination), Some(prime_meridian)) = (
            self.maybe_right_ascension(t),
            self.maybe_declination(t),
            self.maybe_prime_meridian(t),
        ) else {
            return None;
        };
        Some(rotational_elements(
            right_ascension,
            declination,
            prime_meridian,
        ))
    }

    fn maybe_rotational_element_rates(&self, t: f64) -> Option<Elements> {
        let (Some(right_ascension_dot), Some(declination_dot), Some(prime_meridian_dot)) = (
            self.maybe_right_ascension_dot(t),
            self.maybe_declination_dot(t),
            self.maybe_prime_meridian_dot(t),
        ) else {
            return None;
        };
        Some(rotational_elements_rates(
            right_ascension_dot,
            declination_dot,
            prime_meridian_dot,
        ))
    }
}

pub trait RotationalElements: Origin {
    fn nutation_precession_coefficients(&self) -> NutationPrecessionCoefficients;
    fn right_ascension_coefficients(&self) -> PolynomialCoefficients;
    fn declination_coefficients(&self) -> PolynomialCoefficients;
    fn prime_meridian_coefficients(&self) -> PolynomialCoefficients;

    fn right_ascension(&self, t: f64) -> f64 {
        right_ascension(
            &self.right_ascension_coefficients(),
            &self.nutation_precession_coefficients(),
            t,
        )
    }

    fn right_ascension_dot(&self, t: f64) -> f64 {
        right_ascension_dot(
            &self.right_ascension_coefficients(),
            &self.nutation_precession_coefficients(),
            t,
        )
    }

    fn declination(&self, t: f64) -> f64 {
        declination(
            &self.declination_coefficients(),
            &self.nutation_precession_coefficients(),
            t,
        )
    }

    fn declination_dot(&self, t: f64) -> f64 {
        declination_dot(
            &self.declination_coefficients(),
            &self.nutation_precession_coefficients(),
            t,
        )
    }

    fn prime_meridian(&self, t: f64) -> f64 {
        prime_meridian(
            &self.prime_meridian_coefficients(),
            &self.nutation_precession_coefficients(),
            t,
        )
    }

    fn prime_meridian_dot(&self, t: f64) -> f64 {
        prime_meridian_dot(
            &self.prime_meridian_coefficients(),
            &self.nutation_precession_coefficients(),
            t,
        )
    }

    fn rotational_elements(&self, t: f64) -> Elements {
        rotational_elements(
            self.right_ascension(t),
            self.declination(t),
            self.prime_meridian(t),
        )
    }

    fn rotational_element_rates(&self, t: f64) -> Elements {
        rotational_elements_rates(
            self.right_ascension_dot(t),
            self.declination_dot(t),
            self.prime_meridian_dot(t),
        )
    }
}

impl<T: RotationalElements> MaybeRotationalElements for T {
    fn maybe_nutation_precession_coefficients(&self) -> Option<NutationPrecessionCoefficients> {
        Some(self.nutation_precession_coefficients())
    }

    fn maybe_right_ascension_coefficients(&self) -> Option<PolynomialCoefficients> {
        Some(self.right_ascension_coefficients())
    }

    fn maybe_declination_coefficients(&self) -> Option<PolynomialCoefficients> {
        Some(self.declination_coefficients())
    }

    fn maybe_prime_meridian_coefficients(&self) -> Option<PolynomialCoefficients> {
        Some(self.prime_meridian_coefficients())
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use super::*;

    // Jupiter is manually redefined here using known data. This avoids a dependency on the
    // correctness of the PCK parser to test RotationalElements, and prevents compiler errors
    // when generated files are malformed or deleted in preparation for regeneration.
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct Jupiter;

    impl Origin for Jupiter {
        fn id(&self) -> NaifId {
            NaifId(599)
        }

        fn name(&self) -> &'static str {
            "Jupiter"
        }
    }
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct Rupert;

    impl Origin for Rupert {
        fn id(&self) -> NaifId {
            NaifId(1099)
        }

        fn name(&self) -> &'static str {
            "Persephone/Rupert"
        }
    }

    #[test]
    fn test_body() {
        let body = Jupiter;
        let id = body.id().0;
        let name = body.name();
        assert_eq!(id, 599);
        assert_eq!(name, "Jupiter");

        let body = Rupert;
        let id = body.id().0;
        let name = body.name();
        assert_eq!(id, 1099);
        assert_eq!(name, "Persephone/Rupert");
    }

    impl RotationalElements for Jupiter {
        fn nutation_precession_coefficients(&self) -> NutationPrecessionCoefficients {
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

        fn right_ascension_coefficients(&self) -> PolynomialCoefficients {
            (
                4.6784701644349695f64,
                -0.00011342894808711148f64,
                0f64,
                Some(&[
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
                ]),
            )
        }

        fn declination_coefficients(&self) -> PolynomialCoefficients {
            (
                1.1256553894213766f64,
                0.00004211479485062318f64,
                0f64,
                Some(&[
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
                ]),
            )
        }

        fn prime_meridian_coefficients(&self) -> PolynomialCoefficients {
            (
                4.973315703557842f64,
                15.193719457141356f64,
                0f64,
                Some(&[
                    0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64,
                    0f64, 0f64,
                ]),
            )
        }
    }

    #[test]
    fn test_rotational_elements_right_ascension() {
        assert_float_eq!(Jupiter.right_ascension(0.0), 4.678480799964803, rel <= 1e-8);
    }

    #[test]
    fn test_rotational_elements_right_ascension_dot() {
        assert_float_eq!(
            Jupiter.right_ascension_dot(0.0),
            -1.3266588500099516e-13,
            rel <= 1e-8
        );
    }

    #[test]
    fn test_rotational_elements_declination() {
        assert_float_eq!(Jupiter.declination(0.0), 1.1256642372977634, rel <= 1e-8);
    }

    #[test]
    fn test_rotational_elements_declination_dot() {
        assert_float_eq!(
            Jupiter.declination_dot(0.0),
            3.004482367136341e-15,
            rel <= 1e-8
        );
    }

    #[test]
    fn test_rotational_elements_prime_meridian() {
        assert_float_eq!(Jupiter.prime_meridian(0.0), 4.973315703557842, rel <= 1e-8);
    }

    #[test]
    fn test_rotational_elements_prime_meridian_dot() {
        assert_float_eq!(
            Jupiter.prime_meridian_dot(0.0),
            0.00017585323445765458,
            rel <= 1e-8
        );
    }

    #[test]
    fn test_rotational_elements_rotational_elements() {
        let (ra, dec, pm) = Jupiter.rotational_elements(0.0);
        let (expected_ra, expected_dec, expected_pm) =
            (6.249277121030398, 0.44513208936761073, 4.973315703557842);

        assert_float_eq!(
            ra,
            expected_ra,
            rel <= 1e-8,
            "Expected right ascension {}, got {}",
            expected_ra,
            ra
        );
        assert_float_eq!(
            dec,
            expected_dec,
            rel <= 1e-8,
            "Expected declination {}, got {}",
            expected_dec,
            dec
        );
        assert_float_eq!(
            pm,
            expected_pm,
            rel <= 1e-8,
            "Expected prime meridian {}, got {}",
            expected_pm,
            pm
        );
    }

    #[test]
    fn test_rotational_elements_rotational_element_rates() {
        let (ra_dot, dec_dot, pm_dot) = Jupiter.rotational_element_rates(0.0);
        let (expected_ra_dot, expected_dec_dot, expected_pm_dot) = (
            -1.3266588500099516e-13,
            -3.004482367136341e-15,
            0.00017585323445765458,
        );

        assert_float_eq!(
            ra_dot,
            expected_ra_dot,
            rel <= 1e-8,
            "Expected right ascension rate {}, got {}",
            expected_ra_dot,
            ra_dot
        );
        assert_float_eq!(
            dec_dot,
            expected_dec_dot,
            rel <= 1e-8,
            "Expected declination rate {}, got {}",
            expected_dec_dot,
            dec_dot
        );
        assert_float_eq!(
            pm_dot,
            expected_pm_dot,
            rel <= 1e-8,
            "Expected prime meridian rate {}, got {}",
            expected_pm_dot,
            pm_dot
        );
    }
}
