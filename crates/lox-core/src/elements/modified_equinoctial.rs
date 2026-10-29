// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2026 Marijan Smetko <msmetko@msmetko.xyz>
//
// SPDX-License-Identifier: MPL-2.0

//! Modified Equinoctial Elements (MEE), non-singular for circular, equatorial, and parabolic
//! orbits.
//!
//! | Element | Definition |
//! |---------|------------|
//! | `p`     | semi-parameter |
//! | `f`     | e·cos(ω + I·Ω) |
//! | `g`     | e·sin(ω + I·Ω) |
//! | `h`     | tanᴵ(i/2)·cos(Ω) |
//! | `k`     | tanᴵ(i/2)·sin(Ω) |
//! | `L`     | ν + ω + I·Ω (true longitude) |
//!
//! The [`RetrogradeFactor`] I = ±1 avoids the singularity of tan(i/2) at i = π, see the
//! [`equinoctial`](super::equinoctial) module.
//!
//! # References
//!
//! - Walker, M. J. H., Ireland, B., & Owens, J. (1985). *A set of modified equinoctial orbit
//!   elements*. Celestial Mechanics, 36, 409-419. <https://doi.org/10.1007/BF01227493>
//!   Erratum: <https://doi.org/10.1007/BF01238929>

use glam::DVec3;
use thiserror::Error;

use crate::coords::Cartesian;
use crate::elements::equinoctial::{
    RetrogradeFactor, equinoctial_basis, inclination_from_node_components, node_components,
    node_components_from_angular_momentum,
};
use crate::elements::keplerian::{
    Eccentricity, GravitationalParameter, Inclination, Keplerian, KeplerianError,
};
use crate::math::float::{atan2, cos, sin, sqrt};
use crate::units::{Angle, AngleUnits, Distance};

/// Modified Equinoctial Elements (MEE).
///
/// Non-singular for circular (e = 0), equatorial (i = 0 or i = π), and parabolic (e = 1) orbits.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "ModifiedEquinoctialRepr", into = "ModifiedEquinoctialRepr")
)]
pub struct ModifiedEquinoctial {
    p: Distance,
    f: f64,
    g: f64,
    h: f64,
    k: f64,
    l: Angle,
    retrograde_factor: RetrogradeFactor,
}

/// Serde wire format for [`ModifiedEquinoctial`]: forces deserialization through the validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
struct ModifiedEquinoctialRepr {
    p: Distance,
    f: f64,
    g: f64,
    h: f64,
    k: f64,
    l: Angle,
    retrograde_factor: RetrogradeFactor,
}

#[cfg(feature = "serde")]
impl From<ModifiedEquinoctial> for ModifiedEquinoctialRepr {
    fn from(elements: ModifiedEquinoctial) -> Self {
        Self {
            p: elements.p,
            f: elements.f,
            g: elements.g,
            h: elements.h,
            k: elements.k,
            l: elements.l,
            retrograde_factor: elements.retrograde_factor,
        }
    }
}

#[cfg(feature = "serde")]
impl TryFrom<ModifiedEquinoctialRepr> for ModifiedEquinoctial {
    type Error = ModifiedEquinoctialError;

    fn try_from(repr: ModifiedEquinoctialRepr) -> Result<Self, Self::Error> {
        ModifiedEquinoctial::try_new(
            repr.p,
            repr.f,
            repr.g,
            repr.h,
            repr.k,
            repr.l,
            repr.retrograde_factor,
        )
    }
}

/// Error returned when constructing or converting to modified equinoctial elements.
#[derive(Debug, Clone, Error)]
pub enum ModifiedEquinoctialError {
    /// The semi-parameter is not finite and positive.
    #[error("semi-parameter must be finite and positive but was {0}")]
    InvalidSemiParameter(Distance),
    /// An element is not finite.
    #[error("modified equinoctial elements must be finite")]
    NonFiniteElements,
    /// The true longitude lies on or beyond the asymptotes of an open orbit.
    #[error("true longitude {0} lies outside the asymptotes of the open orbit")]
    InvalidTrueLongitude(Angle),
    /// The Cartesian state is not finite.
    #[error("Cartesian state must be finite")]
    NonFiniteState,
    /// The Cartesian state has zero angular momentum, e.g. a rectilinear orbit.
    #[error("modified equinoctial elements are undefined for states with zero angular momentum")]
    ZeroAngularMomentum,
}

impl ModifiedEquinoctial {
    /// Creates modified equinoctial elements from their components.
    ///
    /// `p` is the semi-parameter, `f` = e·cos(ω + I·Ω), `g` = e·sin(ω + I·Ω),
    /// `h` = tanᴵ(i/2)·cos(Ω), `k` = tanᴵ(i/2)·sin(Ω), and `l` = ν + ω + I·Ω.
    ///
    /// # Errors
    ///
    /// Returns a [`ModifiedEquinoctialError`] if the semi-parameter is not finite and positive,
    /// if any element is not finite, or if the true longitude lies outside the asymptotes of an
    /// open orbit.
    pub fn try_new(
        p: Distance,
        f: f64,
        g: f64,
        h: f64,
        k: f64,
        l: Angle,
        retrograde_factor: RetrogradeFactor,
    ) -> Result<Self, ModifiedEquinoctialError> {
        if !p.as_f64().is_finite() || p.as_f64() <= 0.0 {
            return Err(ModifiedEquinoctialError::InvalidSemiParameter(p));
        }
        if ![f, g, h, k, l.as_f64()].iter().all(|x| x.is_finite()) {
            return Err(ModifiedEquinoctialError::NonFiniteElements);
        }
        let (sin_l, cos_l) = l.sin_cos();
        if 1.0 + f * cos_l + g * sin_l <= 0.0 {
            return Err(ModifiedEquinoctialError::InvalidTrueLongitude(l));
        }
        Ok(Self {
            p,
            f,
            g,
            h,
            k,
            l,
            retrograde_factor,
        })
    }

    /// Returns the semi-parameter (semi-latus rectum).
    pub fn p(&self) -> Distance {
        self.p
    }

    /// Returns f = e·cos(ω + I·Ω).
    pub fn f(&self) -> f64 {
        self.f
    }

    /// Returns g = e·sin(ω + I·Ω).
    pub fn g(&self) -> f64 {
        self.g
    }

    /// Returns h = tanᴵ(i/2)·cos(Ω).
    pub fn h(&self) -> f64 {
        self.h
    }

    /// Returns k = tanᴵ(i/2)·sin(Ω).
    pub fn k(&self) -> f64 {
        self.k
    }

    /// Returns the true longitude L = ν + ω + I·Ω.
    pub fn l(&self) -> Angle {
        self.l
    }

    /// Returns the retrograde factor I.
    pub fn retrograde_factor(&self) -> RetrogradeFactor {
        self.retrograde_factor
    }

    /// Returns the eccentricity e = √(f² + g²).
    pub fn eccentricity(&self) -> Eccentricity {
        Eccentricity::try_new(sqrt(self.f * self.f + self.g * self.g))
            .expect("finite elements have a valid eccentricity")
    }

    /// Returns the inclination.
    pub fn inclination(&self) -> Inclination {
        inclination_from_node_components(self.k, self.h, self.retrograde_factor)
    }

    /// Converts the elements to a Cartesian state.
    pub fn to_cartesian(&self, grav_param: GravitationalParameter) -> Cartesian {
        let p = self.p.as_f64();
        let (f_hat, g_hat) = equinoctial_basis(self.k, self.h, self.retrograde_factor);
        let (sin_l, cos_l) = self.l.sin_cos();
        let r = p / (1.0 + self.f * cos_l + self.g * sin_l);
        let sqrt_mu_p = sqrt(grav_param.as_f64() / p);

        let pos = r * cos_l * f_hat + r * sin_l * g_hat;
        let vel = sqrt_mu_p * (-(sin_l + self.g) * f_hat + (cos_l + self.f) * g_hat);
        Cartesian::from_vecs(pos, vel)
    }

    /// Converts the elements to Keplerian elements.
    ///
    /// # Errors
    ///
    /// Returns a [`KeplerianError`] if the elements of an open orbit lie so close to its
    /// asymptotes that they are rejected by the validation of [`Keplerian`].
    pub fn to_keplerian(&self) -> Result<Keplerian, KeplerianError> {
        let sign = self.retrograde_factor.as_f64();
        let longitude_of_periapsis = atan2(self.g, self.f);
        let longitude_of_ascending_node = atan2(self.k, self.h);

        Keplerian::builder()
            .with_semi_parameter(self.p, self.eccentricity().as_f64())
            .with_inclination(self.inclination().as_angle())
            .with_longitude_of_ascending_node(longitude_of_ascending_node.rad())
            .with_argument_of_periapsis(
                (longitude_of_periapsis - sign * longitude_of_ascending_node).rad(),
            )
            .with_true_anomaly((self.l.as_f64() - longitude_of_periapsis).rad())
            .build()
    }
}

impl Cartesian {
    /// Converts the Cartesian state to modified equinoctial elements.
    ///
    /// The [`RetrogradeFactor`] is chosen from the inclination.
    ///
    /// # Errors
    ///
    /// Returns a [`ModifiedEquinoctialError`] if the state is not finite, its angular momentum
    /// is zero, or the resulting elements are invalid, e.g. for a non-positive gravitational
    /// parameter.
    pub fn to_modified_equinoctial(
        &self,
        grav_param: GravitationalParameter,
    ) -> Result<ModifiedEquinoctial, ModifiedEquinoctialError> {
        let r = self.position();
        let v = self.velocity();
        let mu = grav_param.as_f64();

        if !r.is_finite() || !v.is_finite() {
            return Err(ModifiedEquinoctialError::NonFiniteState);
        }
        let angular_momentum = r.cross(v);
        let hm = angular_momentum.length();
        if hm == 0.0 {
            return Err(ModifiedEquinoctialError::ZeroAngularMomentum);
        }

        // Same inclination as `Cartesian::to_keplerian`, so that both paths agree on the
        // retrograde factor.
        let inclination = Inclination::try_new(angular_momentum.angle_between(DVec3::Z).rad())
            .expect("the angle to the z-axis lies in [0, π]");
        let retrograde_factor = RetrogradeFactor::from_inclination(inclination);
        let (k, h) =
            node_components_from_angular_momentum(angular_momentum / hm, retrograde_factor);
        let (f_hat, g_hat) = equinoctial_basis(k, h, retrograde_factor);
        let e: DVec3 = v.cross(angular_momentum) / mu - r / r.length();

        ModifiedEquinoctial::try_new(
            Distance::new(hm * hm / mu),
            e.dot(f_hat),
            e.dot(g_hat),
            h,
            k,
            Angle::from_atan2(r.dot(g_hat), r.dot(f_hat)),
            retrograde_factor,
        )
    }
}

impl Keplerian {
    /// Converts the Keplerian elements to modified equinoctial elements.
    ///
    /// The [`RetrogradeFactor`] is chosen from the inclination.
    pub fn to_modified_equinoctial(&self) -> ModifiedEquinoctial {
        let e = self.eccentricity().as_f64();
        let retrograde_factor = RetrogradeFactor::from_inclination(self.inclination());
        let longitude_of_ascending_node = self.longitude_of_ascending_node().as_f64();
        let longitude_of_periapsis = self.argument_of_periapsis().as_f64()
            + retrograde_factor.as_f64() * longitude_of_ascending_node;
        let (k, h) = node_components(
            self.inclination(),
            longitude_of_ascending_node,
            retrograde_factor,
        );

        ModifiedEquinoctial {
            p: self.semi_parameter(),
            f: e * cos(longitude_of_periapsis),
            g: e * sin(longitude_of_periapsis),
            h,
            k,
            l: Angle::new(longitude_of_periapsis + self.true_anomaly().as_f64()),
            retrograde_factor,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::f64::consts::PI;

    use lox_approx::assert_approx_eq;
    use rstest::rstest;

    use crate::units::{DistanceUnits, VelocityUnits};

    use super::*;

    fn mu() -> GravitationalParameter {
        GravitationalParameter::km3_per_s2(398600.43550702266)
    }

    fn keplerian(e: f64, i: f64, node: f64, periapsis: f64, nu: f64) -> Keplerian {
        Keplerian::builder()
            .with_semi_parameter(12000.0.km(), e)
            .with_inclination(i.rad())
            .with_longitude_of_ascending_node(node.rad())
            .with_argument_of_periapsis(periapsis.rad())
            .with_true_anomaly(nu.rad())
            .build()
            .unwrap()
    }

    #[rstest]
    #[case::general(0.7311, 0.122138, 1.00681, 3.10686, 0.4437)]
    #[case::circular(0.0, 1.7, 1.2, 0.0, 0.8)]
    #[case::equatorial(0.001, 0.0, 0.0, 0.8, 0.5)]
    #[case::circular_equatorial(0.0, 0.0, 0.0, 0.0, PI / 2.0)]
    #[case::polar(0.2, PI / 2.0, 4.0, 2.0, -1.0)]
    #[case::retrograde(0.3, 2.9, 5.0, 1.0, 2.0)]
    #[case::retrograde_equatorial(0.3, PI, 0.0, 1.0, 2.0)]
    #[case::retrograde_circular_equatorial(0.0, PI, 0.0, 0.0, 2.0)]
    #[case::parabolic(1.0, 0.5, 1.0, 2.0, 1.5)]
    #[case::hyperbolic(2.5, 2.0, 3.0, 4.0, -1.0)]
    fn test_conversions_preserve_state(
        #[case] e: f64,
        #[case] i: f64,
        #[case] node: f64,
        #[case] periapsis: f64,
        #[case] nu: f64,
    ) {
        let kep = keplerian(e, i, node, periapsis, nu);
        let expected = kep.to_cartesian(mu());

        let from_keplerian = kep.to_modified_equinoctial();
        let from_cartesian = expected.to_modified_equinoctial(mu()).unwrap();
        assert_eq!(
            from_keplerian.retrograde_factor(),
            from_cartesian.retrograde_factor()
        );
        assert_approx_eq!(from_keplerian.eccentricity().as_f64(), e, atol <= 1e-12);
        assert_approx_eq!(from_cartesian.inclination().as_f64(), i, atol <= 1e-12);

        for mee in [from_keplerian, from_cartesian] {
            let actual = mee.to_cartesian(mu());
            assert_approx_eq!(
                actual.position(),
                expected.position(),
                atol <= 1e-6,
                rtol <= 1e-10
            );
            assert_approx_eq!(
                actual.velocity(),
                expected.velocity(),
                atol <= 1e-9,
                rtol <= 1e-10
            );

            let actual = mee.to_keplerian().unwrap().to_cartesian(mu());
            assert_approx_eq!(
                actual.position(),
                expected.position(),
                atol <= 1e-6,
                rtol <= 1e-10
            );
            assert_approx_eq!(
                actual.velocity(),
                expected.velocity(),
                atol <= 1e-9,
                rtol <= 1e-10
            );
        }
    }

    #[test]
    fn test_retrograde_factor_follows_inclination() {
        let prograde = keplerian(0.1, 1.0, 0.0, 0.0, 0.0).to_modified_equinoctial();
        let retrograde = keplerian(0.1, 2.0, 0.0, 0.0, 0.0).to_modified_equinoctial();
        assert_eq!(prograde.retrograde_factor(), RetrogradeFactor::Prograde);
        assert_eq!(retrograde.retrograde_factor(), RetrogradeFactor::Retrograde);
        // tanᴵ(i/2) stays bounded by one.
        assert!(retrograde.h().abs() <= 1.0);
    }

    #[test]
    fn test_retrograde_equatorial_node_components_vanish() {
        let cart = keplerian(0.1, PI, 0.0, 1.0, 0.5).to_cartesian(mu());
        let mee = cart.to_modified_equinoctial(mu()).unwrap();
        assert_approx_eq!(mee.h(), 0.0, atol <= 1e-15);
        assert_approx_eq!(mee.k(), 0.0, atol <= 1e-15);
    }

    #[test]
    fn test_try_new_validates() {
        let p = 7000.0.km();
        let prograde = RetrogradeFactor::Prograde;
        assert!(ModifiedEquinoctial::try_new(p, 0.1, 0.2, 0.3, 0.4, 0.5.rad(), prograde).is_ok());
        assert!(matches!(
            ModifiedEquinoctial::try_new(-p, 0.0, 0.0, 0.0, 0.0, 0.0.rad(), prograde),
            Err(ModifiedEquinoctialError::InvalidSemiParameter(_))
        ));
        assert!(matches!(
            ModifiedEquinoctial::try_new(p, f64::NAN, 0.0, 0.0, 0.0, 0.0.rad(), prograde),
            Err(ModifiedEquinoctialError::NonFiniteElements)
        ));
        // e = 2 at L = π lies beyond the asymptotes.
        assert!(matches!(
            ModifiedEquinoctial::try_new(p, 2.0, 0.0, 0.0, 0.0, PI.rad(), prograde),
            Err(ModifiedEquinoctialError::InvalidTrueLongitude(_))
        ));
    }

    #[test]
    fn test_cartesian_errors() {
        let rectilinear = Cartesian::builder()
            .position(7000.0.km(), 0.0.km(), 0.0.km())
            .velocity(1.0.kps(), 0.0.kps(), 0.0.kps())
            .build();
        assert!(matches!(
            rectilinear.to_modified_equinoctial(mu()),
            Err(ModifiedEquinoctialError::ZeroAngularMomentum)
        ));
        let non_finite = Cartesian::from_vecs(DVec3::NAN, DVec3::X);
        assert!(matches!(
            non_finite.to_modified_equinoctial(mu()),
            Err(ModifiedEquinoctialError::NonFiniteState)
        ));
        let state = Cartesian::builder()
            .position(7000.0.km(), 0.0.km(), 0.0.km())
            .velocity(0.0.kps(), 7.5.kps(), 0.0.kps())
            .build();
        assert!(
            state
                .to_modified_equinoctial(GravitationalParameter::km3_per_s2(0.0))
                .is_err()
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialization_validates() {
        let mee = keplerian(0.1, 0.5, 1.0, 2.0, 0.3).to_modified_equinoctial();
        let json = serde_json::to_value(mee).unwrap();
        let back: ModifiedEquinoctial = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(back, mee);

        // e = 2 at L = π lies beyond the asymptotes.
        let mut beyond_asymptotes = json;
        beyond_asymptotes["f"] = 2.0.into();
        beyond_asymptotes["g"] = 0.0.into();
        beyond_asymptotes["l"] = serde_json::to_value(PI.rad()).unwrap();
        assert!(serde_json::from_value::<ModifiedEquinoctial>(beyond_asymptotes).is_err());
    }
}
