/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub use crate::dynamic::DynOrigin;
pub use generated::*;
use lox_math::constants::f64::time::{SECONDS_PER_DAY, SECONDS_PER_JULIAN_CENTURY};
use std::fmt::{Display, Formatter};
use thiserror::Error;

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

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("undefined property '{prop}' for origin '{origin}'")]
pub struct UndefinedOriginPropertyError {
    origin: String,
    prop: String,
}

pub type Radii = (f64, f64, f64);

pub trait TryTriaxialEllipsoid: Origin {
    fn try_radii(&self) -> Result<Radii, UndefinedOriginPropertyError>;
}

pub trait TriaxialEllipsoid: Origin {
    fn radii(&self) -> Radii;
}

impl<T: TriaxialEllipsoid> TryTriaxialEllipsoid for T {
    fn try_radii(&self) -> Result<Radii, UndefinedOriginPropertyError> {
        Ok(self.radii())
    }
}

fn flattening(equatorial_radius: f64, polar_radius: f64) -> f64 {
    (equatorial_radius - polar_radius) / equatorial_radius
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

pub trait TrySpheroid: TryTriaxialEllipsoid {
    fn try_equatorial_radius(&self) -> Result<f64, UndefinedOriginPropertyError> {
        self.try_radii().map(|radii| radii.0)
    }

    fn try_polar_radius(&self) -> Result<f64, UndefinedOriginPropertyError> {
        self.try_radii().map(|radii| radii.2)
    }

    fn try_flattening(&self) -> Result<f64, UndefinedOriginPropertyError> {
        self.try_radii().map(|radii| flattening(radii.0, radii.2))
    }
}

impl<T: Spheroid> TrySpheroid for T {
    fn try_equatorial_radius(&self) -> Result<f64, UndefinedOriginPropertyError> {
        Ok(self.equatorial_radius())
    }

    fn try_polar_radius(&self) -> Result<f64, UndefinedOriginPropertyError> {
        Ok(self.polar_radius())
    }

    fn try_flattening(&self) -> Result<f64, UndefinedOriginPropertyError> {
        Ok(self.flattening())
    }
}

pub trait TryMeanRadius: Origin {
    fn try_mean_radius(&self) -> Result<f64, UndefinedOriginPropertyError>;
}

pub trait MeanRadius: Origin {
    fn mean_radius(&self) -> f64;
}

impl<T: MeanRadius> TryMeanRadius for T {
    fn try_mean_radius(&self) -> Result<f64, UndefinedOriginPropertyError> {
        Ok(self.mean_radius())
    }
}

pub trait PointMass: Origin {
    fn gravitational_parameter(&self) -> f64;
}

pub trait TryPointMass: Origin {
    fn try_gravitational_parameter(&self) -> Result<f64, UndefinedOriginPropertyError>;
}

impl<T: PointMass> TryPointMass for T {
    fn try_gravitational_parameter(&self) -> Result<f64, UndefinedOriginPropertyError> {
        Ok(self.gravitational_parameter())
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum RotationalElementType {
    RightAscension,
    Declination,
    Rotation,
}

impl RotationalElementType {
    fn sincos(&self, val: f64) -> f64 {
        match self {
            RotationalElementType::Declination => val.cos(),
            _ => val.sin(),
        }
    }

    fn sincos_dot(&self, val: f64) -> f64 {
        match self {
            RotationalElementType::Declination => val.sin(),
            _ => val.cos(),
        }
    }

    fn sign(&self) -> f64 {
        match self {
            RotationalElementType::Declination => -1.0,
            _ => 1.0,
        }
    }

    fn dt(&self) -> f64 {
        match self {
            RotationalElementType::Rotation => SECONDS_PER_DAY,
            _ => SECONDS_PER_JULIAN_CENTURY,
        }
    }
}

struct NutationPrecessionCoefficients<const N: usize> {
    theta0: [f64; N],
    theta1: [f64; N],
}

pub(crate) struct RotationalElement<const N: usize> {
    typ: RotationalElementType,
    c0: f64,
    c1: f64,
    c2: f64,
    c: Option<[f64; N]>,
}

impl<const N: usize> RotationalElement<N> {
    fn trig_term<const M: usize>(
        &self,
        nut_prec: Option<&NutationPrecessionCoefficients<M>>,
        t: f64,
    ) -> f64 {
        let Some(nut_prec) = nut_prec else { return 0.0 };
        self.c
            .map(|c| {
                c.iter()
                    .zip(nut_prec.theta0.iter())
                    .zip(nut_prec.theta1.iter())
                    .map(|((&c, &theta0), &theta1)| {
                        c * self
                            .typ
                            .sincos(theta0 + theta1 * t / SECONDS_PER_JULIAN_CENTURY)
                    })
                    .sum()
            })
            .unwrap_or_default()
    }

    fn trig_term_dot<const M: usize>(
        &self,
        nut_prec: Option<&NutationPrecessionCoefficients<M>>,
        t: f64,
    ) -> f64 {
        let Some(nut_prec) = nut_prec else { return 0.0 };
        self.c
            .map(|c| {
                c.iter()
                    .zip(nut_prec.theta0.iter())
                    .zip(nut_prec.theta1.iter())
                    .map(|((&c, &theta0), &theta1)| {
                        c * theta1 / SECONDS_PER_JULIAN_CENTURY
                            * self
                                .typ
                                .sincos_dot(theta0 + theta1 * t / SECONDS_PER_JULIAN_CENTURY)
                    })
                    .sum()
            })
            .unwrap_or_default()
    }

    fn angle<const M: usize>(
        &self,
        nut_prec: Option<&NutationPrecessionCoefficients<M>>,
        t: f64,
    ) -> f64 {
        self.c0
            + self.c1 * t / self.typ.dt()
            + self.c2 * t.powi(2) / self.typ.dt().powi(2)
            + self.trig_term(nut_prec, t)
    }

    fn angle_dot<const M: usize>(
        &self,
        nut_prec: Option<&NutationPrecessionCoefficients<M>>,
        t: f64,
    ) -> f64 {
        self.c1 / self.typ.dt()
            + 2.0 * self.c2 * t / self.typ.dt().powi(2)
            + self.typ.sign() * self.trig_term_dot(nut_prec, t)
    }
}

pub type Elements = (f64, f64, f64);

pub trait RotationalElements: Origin {
    fn rotational_elements(&self, t: f64) -> Elements;

    fn rotational_element_rates(&self, t: f64) -> Elements;

    fn right_ascension(&self, t: f64) -> f64 {
        self.rotational_elements(t).0
    }

    fn right_ascension_rate(&self, t: f64) -> f64 {
        self.rotational_element_rates(t).0
    }

    fn declination(&self, t: f64) -> f64 {
        self.rotational_elements(t).1
    }

    fn declination_rate(&self, t: f64) -> f64 {
        self.rotational_element_rates(t).1
    }

    fn rotation_angle(&self, t: f64) -> f64 {
        self.rotational_elements(t).2
    }

    fn rotation_rate(&self, t: f64) -> f64 {
        self.rotational_element_rates(t).2
    }
}

pub trait TryRotationalElements: Origin {
    fn try_rotational_elements(&self, t: f64) -> Result<Elements, UndefinedOriginPropertyError>;

    fn try_rotational_element_rates(
        &self,
        t: f64,
    ) -> Result<Elements, UndefinedOriginPropertyError>;

    fn try_right_ascension(&self, t: f64) -> Result<f64, UndefinedOriginPropertyError> {
        self.try_rotational_elements(t).map(|r| r.0)
    }

    fn try_right_ascension_rate(&self, t: f64) -> Result<f64, UndefinedOriginPropertyError> {
        self.try_rotational_element_rates(t).map(|r| r.0)
    }

    fn try_declination(&self, t: f64) -> Result<f64, UndefinedOriginPropertyError> {
        self.try_rotational_elements(t).map(|r| r.1)
    }

    fn try_declination_rate(&self, t: f64) -> Result<f64, UndefinedOriginPropertyError> {
        self.try_rotational_element_rates(t).map(|r| r.1)
    }

    fn try_rotation_angle(&self, t: f64) -> Result<f64, UndefinedOriginPropertyError> {
        self.try_rotational_elements(t).map(|r| r.2)
    }

    fn try_rotation_rate(&self, t: f64) -> Result<f64, UndefinedOriginPropertyError> {
        self.try_rotational_element_rates(t).map(|r| r.2)
    }
}

impl<T: RotationalElements> TryRotationalElements for T {
    fn try_rotational_elements(&self, t: f64) -> Result<Elements, UndefinedOriginPropertyError> {
        Ok(self.rotational_elements(t))
    }

    fn try_rotational_element_rates(
        &self,
        t: f64,
    ) -> Result<Elements, UndefinedOriginPropertyError> {
        Ok(self.rotational_element_rates(t))
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

    const NUTATION_PRECESSION_JUPITER: NutationPrecessionCoefficients<15> =
        NutationPrecessionCoefficients {
            theta0: [
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
            theta1: [
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
        };

    const RIGHT_ASCENSION_JUPITER: RotationalElement<15> = RotationalElement {
        typ: RotationalElementType::RightAscension,
        c0: 4.6784701644349695f64,
        c1: -0.00011342894808711148f64,
        c2: 0f64,
        c: Some([
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
    };

    const DECLINATION_JUPITER: RotationalElement<15> = RotationalElement {
        typ: RotationalElementType::Declination,
        c0: 1.1256553894213766f64,
        c1: 0.00004211479485062318f64,
        c2: 0f64,
        c: Some([
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
    };

    const ROTATION_JUPITER: RotationalElement<15> = RotationalElement {
        typ: RotationalElementType::Rotation,
        c0: 4.973315703557842f64,
        c1: 15.193719457141356f64,
        c2: 0f64,
        c: None,
    };

    impl RotationalElements for Jupiter {
        fn rotational_elements(&self, t: f64) -> Elements {
            (
                RIGHT_ASCENSION_JUPITER.angle(Some(&NUTATION_PRECESSION_JUPITER), t),
                DECLINATION_JUPITER.angle(Some(&NUTATION_PRECESSION_JUPITER), t),
                ROTATION_JUPITER.angle(Some(&NUTATION_PRECESSION_JUPITER), t),
            )
        }

        fn rotational_element_rates(&self, t: f64) -> Elements {
            (
                RIGHT_ASCENSION_JUPITER.angle_dot(Some(&NUTATION_PRECESSION_JUPITER), t),
                DECLINATION_JUPITER.angle_dot(Some(&NUTATION_PRECESSION_JUPITER), t),
                ROTATION_JUPITER.angle_dot(Some(&NUTATION_PRECESSION_JUPITER), t),
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
            Jupiter.right_ascension_rate(0.0),
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
            Jupiter.declination_rate(0.0),
            3.004482367136341e-15,
            rel <= 1e-8
        );
    }

    #[test]
    fn test_rotational_elements_prime_meridian() {
        assert_float_eq!(Jupiter.rotation_angle(0.0), 4.973315703557842, rel <= 1e-8);
    }

    #[test]
    fn test_rotational_elements_prime_meridian_dot() {
        assert_float_eq!(
            Jupiter.rotation_rate(0.0),
            0.00017585323445765458,
            rel <= 1e-8
        );
    }
}
