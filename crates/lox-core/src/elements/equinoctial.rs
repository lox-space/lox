// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Equinoctial orbital elements, non-singular for circular and equatorial orbits.
//!
//! | Element | Definition |
//! |---------|------------|
//! | `a`     | semi-major axis |
//! | `h`     | e·sin(ω + I·Ω) |
//! | `k`     | e·cos(ω + I·Ω) |
//! | `p`     | tanᴵ(i/2)·sin(Ω) |
//! | `q`     | tanᴵ(i/2)·cos(Ω) |
//! | `λ`     | M + ω + I·Ω (mean longitude) |
//!
//! The retrograde factor I = ±1 selects between the direct (I = +1) and the retrograde (I = -1)
//! set of elements. Each set is singular at one inclination only, i = π for the direct and i = 0
//! for the retrograde set, so conversions from Cartesian or Keplerian states choose the set whose
//! singularity is further away. See [`RetrogradeFactor`].
//!
//! The longitudes of the retrograde set contain -Ω instead of +Ω, so they differ by 2Ω from
//! [`Keplerian::mean_longitude`](super::Keplerian::mean_longitude) and
//! [`Keplerian::true_longitude`](super::Keplerian::true_longitude), which always add Ω.
//!
//! Equinoctial elements are defined for closed orbits only. Use
//! [`ModifiedEquinoctial`] elements for open orbits.
//!
//! # References
//!
//! - Broucke, R. A., & Cefola, P. J. (1972). *On the equinoctial orbit elements*. Celestial
//!   Mechanics, 5, 303-310. <https://doi.org/10.1007/BF01228432>

use core::f64::consts::PI;

use glam::DVec3;
use thiserror::Error;

use crate::anomalies::AnomalyError;
use crate::coords::Cartesian;
use crate::elements::keplerian::{
    Eccentricity, EccentricityError, GravitationalParameter, Inclination, Keplerian,
};
use crate::elements::longitudes::{
    mean_to_eccentric_longitude, mean_to_true_longitude, true_to_mean_longitude,
};
use crate::elements::modified_equinoctial::{ModifiedEquinoctial, ModifiedEquinoctialError};
use crate::math::float::{abs, atan, cos, hypot, sin, tan};
use crate::units::{Angle, Distance};

/// The retrograde factor I of equinoctial elements.
///
/// The node components of the elements are scaled by tanᴵ(i/2), which diverges at i = π for the
/// [`Prograde`](Self::Prograde) set and at i = 0 for the [`Retrograde`](Self::Retrograde) set.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RetrogradeFactor {
    /// I = +1, singular at i = π.
    #[default]
    Prograde,
    /// I = -1, singular at i = 0.
    Retrograde,
}

impl RetrogradeFactor {
    /// Returns the retrograde factor whose singularity is further away from the inclination,
    /// i.e. [`Retrograde`](Self::Retrograde) for inclinations above π/2.
    pub fn from_inclination(inclination: Inclination) -> Self {
        if inclination.as_f64() > PI / 2.0 {
            Self::Retrograde
        } else {
            Self::Prograde
        }
    }

    /// Returns the retrograde factor as ±1.
    pub const fn as_f64(self) -> f64 {
        match self {
            Self::Prograde => 1.0,
            Self::Retrograde => -1.0,
        }
    }
}

/// Returns the unit vectors f̂ and ĝ of the equinoctial frame, which span the orbital plane, for
/// the node components p = tanᴵ(i/2)·sin(Ω) and q = tanᴵ(i/2)·cos(Ω).
pub(crate) fn equinoctial_basis(
    p: f64,
    q: f64,
    retrograde_factor: RetrogradeFactor,
) -> (DVec3, DVec3) {
    let sign = retrograde_factor.as_f64();
    // Divide the numerators and the denominator by m² so that p² + q² cannot overflow. For
    // |p|, |q| <= 1, which covers every inclination of the matching set, m = 1 and the scaling
    // is exact.
    let m = abs(p).max(abs(q)).max(1.0);
    let (p, q, u) = (p / m, q / m, 1.0 / m);
    let u2 = u * u;
    let s = u2 + p * p + q * q;
    let f_hat = DVec3::new(u2 - p * p + q * q, 2.0 * p * q, -2.0 * sign * p * u) / s;
    let g_hat = DVec3::new(2.0 * sign * p * q, sign * (u2 + p * p - q * q), 2.0 * q * u) / s;
    (f_hat, g_hat)
}

/// Returns the node components (p, q) = tanᴵ(i/2)·(sin(Ω), cos(Ω)).
pub(crate) fn node_components(
    inclination: Inclination,
    longitude_of_ascending_node: f64,
    retrograde_factor: RetrogradeFactor,
) -> (f64, f64) {
    let i = inclination.as_f64();
    let t = match retrograde_factor {
        RetrogradeFactor::Prograde => tan(i / 2.0),
        RetrogradeFactor::Retrograde => tan((PI - i) / 2.0),
    };
    (
        t * sin(longitude_of_ascending_node),
        t * cos(longitude_of_ascending_node),
    )
}

/// Returns the node components (p, q) for the unit vector `w` along the angular momentum.
pub(crate) fn node_components_from_angular_momentum(
    w: DVec3,
    retrograde_factor: RetrogradeFactor,
) -> (f64, f64) {
    let d = 1.0 + retrograde_factor.as_f64() * w.z;
    (w.x / d, -w.y / d)
}

/// Returns the inclination for the node components (p, q).
pub(crate) fn inclination_from_node_components(
    p: f64,
    q: f64,
    retrograde_factor: RetrogradeFactor,
) -> Inclination {
    let half = 2.0 * atan(hypot(p, q));
    let i = match retrograde_factor {
        RetrogradeFactor::Prograde => half,
        RetrogradeFactor::Retrograde => PI - half,
    };
    Inclination::try_new(Angle::new(i)).expect("finite node components have a valid inclination")
}

/// Equinoctial orbital elements.
///
/// Non-singular for circular (e = 0) and equatorial (i = 0 or i = π) orbits. Only closed orbits
/// can be represented.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "EquinoctialRepr", into = "EquinoctialRepr")
)]
pub struct Equinoctial {
    a: Distance,
    h: f64,
    k: f64,
    p: f64,
    q: f64,
    lambda: Angle,
    retrograde_factor: RetrogradeFactor,
}

/// Serde wire format for [`Equinoctial`]: forces deserialization through the validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
struct EquinoctialRepr {
    a: Distance,
    h: f64,
    k: f64,
    p: f64,
    q: f64,
    lambda: Angle,
    retrograde_factor: RetrogradeFactor,
}

#[cfg(feature = "serde")]
impl From<Equinoctial> for EquinoctialRepr {
    fn from(elements: Equinoctial) -> Self {
        Self {
            a: elements.a,
            h: elements.h,
            k: elements.k,
            p: elements.p,
            q: elements.q,
            lambda: elements.lambda,
            retrograde_factor: elements.retrograde_factor,
        }
    }
}

#[cfg(feature = "serde")]
impl TryFrom<EquinoctialRepr> for Equinoctial {
    type Error = EquinoctialError;

    fn try_from(repr: EquinoctialRepr) -> Result<Self, Self::Error> {
        Equinoctial::try_new(
            repr.a,
            repr.h,
            repr.k,
            repr.p,
            repr.q,
            repr.lambda,
            repr.retrograde_factor,
        )
    }
}

/// Error returned when constructing or converting to equinoctial elements.
#[derive(Debug, Clone, Error)]
pub enum EquinoctialError {
    /// The semi-major axis is not finite and positive.
    #[error("semi-major axis must be finite and positive but was {0}")]
    InvalidSemiMajorAxis(Distance),
    /// The semi-parameter a·(1 - e²) is not positive.
    #[error("semi-parameter must be positive but was {0}")]
    InvalidSemiParameter(Distance),
    /// An element is not finite.
    #[error("equinoctial elements must be finite")]
    NonFiniteElements,
    /// The eccentricity √(h² + k²) is not finite.
    #[error(transparent)]
    InvalidEccentricity(#[from] EccentricityError),
    /// The orbit is not closed.
    #[error(
        "equinoctial elements require a circular or elliptic orbit but the eccentricity was {0}"
    )]
    NotElliptic(Eccentricity),
    /// The Cartesian state could not be converted.
    #[error(transparent)]
    ModifiedEquinoctial(#[from] ModifiedEquinoctialError),
    /// Kepler's equation could not be solved for the eccentric longitude.
    #[error("failed to solve Kepler's equation: {0}")]
    KeplerEquation(AnomalyError),
}

impl Equinoctial {
    /// Creates equinoctial elements from their components.
    ///
    /// `a` is the semi-major axis, `h` = e·sin(ω + I·Ω), `k` = e·cos(ω + I·Ω),
    /// `p` = tanᴵ(i/2)·sin(Ω), `q` = tanᴵ(i/2)·cos(Ω), and `lambda` = M + ω + I·Ω.
    ///
    /// # Errors
    ///
    /// Returns an [`EquinoctialError`] if the semi-major axis is not finite and positive, if
    /// any element or the eccentricity is not finite, if the eccentricity does not describe a
    /// closed orbit, or if the semi-parameter is not positive.
    pub fn try_new(
        a: Distance,
        h: f64,
        k: f64,
        p: f64,
        q: f64,
        lambda: Angle,
        retrograde_factor: RetrogradeFactor,
    ) -> Result<Self, EquinoctialError> {
        if !a.as_f64().is_finite() || a.as_f64() <= 0.0 {
            return Err(EquinoctialError::InvalidSemiMajorAxis(a));
        }
        if ![h, k, p, q, lambda.as_f64()].iter().all(|x| x.is_finite()) {
            return Err(EquinoctialError::NonFiniteElements);
        }
        check_closed(Eccentricity::try_new(hypot(h, k))?)?;
        let elements = Self {
            a,
            h,
            k,
            p,
            q,
            lambda,
            retrograde_factor,
        };
        let semi_parameter = elements.semi_parameter();
        if semi_parameter.as_f64() <= 0.0 {
            return Err(EquinoctialError::InvalidSemiParameter(semi_parameter));
        }
        Ok(elements)
    }

    /// Returns the semi-major axis.
    pub fn a(&self) -> Distance {
        self.a
    }

    /// Returns h = e·sin(ω + I·Ω).
    pub fn h(&self) -> f64 {
        self.h
    }

    /// Returns k = e·cos(ω + I·Ω).
    pub fn k(&self) -> f64 {
        self.k
    }

    /// Returns p = tanᴵ(i/2)·sin(Ω).
    pub fn p(&self) -> f64 {
        self.p
    }

    /// Returns q = tanᴵ(i/2)·cos(Ω).
    pub fn q(&self) -> f64 {
        self.q
    }

    /// Returns the mean longitude λ = M + ω + I·Ω.
    ///
    /// For the retrograde set, this differs by 2Ω from [`Keplerian::mean_longitude`].
    pub fn lambda(&self) -> Angle {
        self.lambda
    }

    /// Returns the retrograde factor I.
    pub fn retrograde_factor(&self) -> RetrogradeFactor {
        self.retrograde_factor
    }

    /// Returns the eccentricity e = √(h² + k²).
    pub fn eccentricity(&self) -> Eccentricity {
        Eccentricity::try_new(hypot(self.h, self.k))
            .expect("finite elements have a valid eccentricity")
    }

    /// Returns the inclination.
    pub fn inclination(&self) -> Inclination {
        inclination_from_node_components(self.p, self.q, self.retrograde_factor)
    }

    /// Returns the semi-parameter p = a·(1 - e²).
    pub fn semi_parameter(&self) -> Distance {
        let e = self.eccentricity().as_f64();
        Distance::new(self.a.as_f64() * (1.0 - e) * (1.0 + e))
    }

    /// Returns the true longitude L = ν + ω + I·Ω, in the same revolution as λ.
    ///
    /// For the retrograde set, this differs by 2Ω from [`Keplerian::true_longitude`].
    ///
    /// # Errors
    ///
    /// Returns [`EquinoctialError::KeplerEquation`] if Kepler's equation cannot be solved.
    pub fn true_longitude(&self) -> Result<Angle, EquinoctialError> {
        mean_to_true_longitude(self.lambda, self.h, self.k)
            .map_err(EquinoctialError::KeplerEquation)
    }

    /// Returns the eccentric longitude F = E + ω + I·Ω, in the same revolution as λ.
    ///
    /// # Errors
    ///
    /// Returns [`EquinoctialError::KeplerEquation`] if Kepler's equation cannot be solved.
    pub fn eccentric_longitude(&self) -> Result<Angle, EquinoctialError> {
        mean_to_eccentric_longitude(self.lambda, self.h, self.k)
            .map_err(EquinoctialError::KeplerEquation)
    }

    /// Converts the elements to modified equinoctial elements.
    ///
    /// # Errors
    ///
    /// Returns [`EquinoctialError::KeplerEquation`] if Kepler's equation cannot be solved.
    pub fn to_modified_equinoctial(&self) -> Result<ModifiedEquinoctial, EquinoctialError> {
        // Valid equinoctial elements describe a closed orbit with finite elements and a positive
        // semi-parameter, for which every true longitude is valid, so the modified equinoctial
        // elements need not be validated again.
        Ok(ModifiedEquinoctial::from_valid_parts(
            self.semi_parameter(),
            self.k,
            self.h,
            self.q,
            self.p,
            self.true_longitude()?,
            self.retrograde_factor,
        ))
    }

    /// Converts the elements to a Cartesian state.
    ///
    /// # Errors
    ///
    /// Returns [`EquinoctialError::KeplerEquation`] if Kepler's equation cannot be solved.
    pub fn to_cartesian(
        &self,
        grav_param: GravitationalParameter,
    ) -> Result<Cartesian, EquinoctialError> {
        Ok(self.to_modified_equinoctial()?.to_cartesian(grav_param))
    }

    /// Converts the elements to Keplerian elements.
    ///
    /// # Errors
    ///
    /// Returns [`EquinoctialError::KeplerEquation`] if Kepler's equation cannot be solved.
    pub fn to_keplerian(&self) -> Result<Keplerian, EquinoctialError> {
        Ok(self
            .to_modified_equinoctial()?
            .to_keplerian()
            .expect("closed orbits have valid Keplerian elements"))
    }
}

fn check_closed(eccentricity: Eccentricity) -> Result<(), EquinoctialError> {
    if eccentricity.is_circular_or_elliptic() {
        Ok(())
    } else {
        Err(EquinoctialError::NotElliptic(eccentricity))
    }
}

impl ModifiedEquinoctial {
    /// Converts the elements to equinoctial elements.
    ///
    /// # Errors
    ///
    /// Returns an [`EquinoctialError`] if the orbit is not closed or its semi-major axis
    /// overflows.
    pub fn to_equinoctial(&self) -> Result<Equinoctial, EquinoctialError> {
        let eccentricity = self.eccentricity();
        check_closed(eccentricity)?;
        let e = eccentricity.as_f64();
        let (h, k) = (self.g(), self.f());
        Equinoctial::try_new(
            Distance::new(self.p().as_f64() / ((1.0 - e) * (1.0 + e))),
            h,
            k,
            self.k(),
            self.h(),
            true_to_mean_longitude(self.l(), h, k),
            self.retrograde_factor(),
        )
    }
}

impl Keplerian {
    /// Converts the Keplerian elements to equinoctial elements.
    ///
    /// The [`RetrogradeFactor`] is chosen from the inclination.
    ///
    /// # Errors
    ///
    /// Returns an [`EquinoctialError`] if the orbit is not closed.
    pub fn to_equinoctial(&self) -> Result<Equinoctial, EquinoctialError> {
        check_closed(self.eccentricity())?;
        self.to_modified_equinoctial()?.to_equinoctial()
    }
}

impl Cartesian {
    /// Converts the Cartesian state to equinoctial elements.
    ///
    /// The [`RetrogradeFactor`] is chosen from the inclination.
    ///
    /// # Errors
    ///
    /// Returns an [`EquinoctialError`] if the state is not finite, its angular momentum is zero,
    /// or the orbit is not closed.
    pub fn to_equinoctial(
        &self,
        grav_param: GravitationalParameter,
    ) -> Result<Equinoctial, EquinoctialError> {
        self.to_modified_equinoctial(grav_param)?.to_equinoctial()
    }
}

#[cfg(test)]
mod tests {
    use core::f64::consts::TAU;

    use lox_approx::assert_approx_eq;
    use proptest::prelude::*;
    use rstest::rstest;

    use crate::anomalies::TrueAnomaly;
    use crate::math::float::acos;
    use crate::units::{AngleUnits, DistanceUnits};

    use super::*;

    fn mu() -> GravitationalParameter {
        GravitationalParameter::km3_per_s2(398600.43550702266)
    }

    proptest! {
        // `to_modified_equinoctial` skips the validation of the modified equinoctial elements,
        // which must therefore always pass.
        #[test]
        fn prop_modified_equinoctial_elements_are_valid(
            a in 1e3..1e9f64,
            // Eccentricities within 1e-8 of 1 count as parabolic and are rejected.
            e in prop_oneof![Just(0.0), 0.0..0.9, 0.9..1.0 - 2e-8],
            pomega in 0.0..TAU,
            p in -10.0..10.0f64,
            q in -10.0..10.0f64,
            lambda in -50.0..50.0f64,
            retrograde in any::<bool>(),
        ) {
            let retrograde_factor = if retrograde {
                RetrogradeFactor::Retrograde
            } else {
                RetrogradeFactor::Prograde
            };
            let eq = Equinoctial::try_new(
                Distance::new(a),
                e * sin(pomega),
                e * cos(pomega),
                p,
                q,
                Angle::new(lambda),
                retrograde_factor,
            )
            .unwrap();
            let mee = eq.to_modified_equinoctial().unwrap();
            prop_assert!(
                ModifiedEquinoctial::try_new(
                    mee.p(),
                    mee.f(),
                    mee.g(),
                    mee.h(),
                    mee.k(),
                    mee.l(),
                    mee.retrograde_factor(),
                )
                .is_ok()
            );
        }
    }

    fn keplerian(e: f64, i: f64, node: f64, periapsis: f64, nu: f64) -> Keplerian {
        Keplerian::builder()
            .with_semi_major_axis(24464.560.km(), e)
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
    #[case::highly_eccentric(0.99, 1.0, 2.0, 3.0, 3.0)]
    fn test_conversions_preserve_state(
        #[case] e: f64,
        #[case] i: f64,
        #[case] node: f64,
        #[case] periapsis: f64,
        #[case] nu: f64,
    ) {
        let kep = keplerian(e, i, node, periapsis, nu);
        let expected = kep.to_cartesian(mu());

        let from_keplerian = kep.to_equinoctial().unwrap();
        let from_cartesian = expected.to_equinoctial(mu()).unwrap();
        let from_mee = kep
            .to_modified_equinoctial()
            .unwrap()
            .to_equinoctial()
            .unwrap();
        assert_approx_eq!(from_keplerian.eccentricity().as_f64(), e, atol <= 1e-12);
        assert_approx_eq!(from_cartesian.inclination().as_f64(), i, atol <= 1e-12);
        assert_approx_eq!(
            from_keplerian.a().as_f64(),
            kep.semi_major_axis().unwrap().as_f64(),
            rtol <= 1e-12
        );

        for eq in [from_keplerian, from_cartesian, from_mee] {
            assert_eq!(
                eq.retrograde_factor(),
                RetrogradeFactor::from_inclination(kep.inclination())
            );
            for actual in [
                eq.to_cartesian(mu()).unwrap(),
                eq.to_keplerian().unwrap().to_cartesian(mu()),
                eq.to_modified_equinoctial().unwrap().to_cartesian(mu()),
            ] {
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
    }

    #[test]
    fn test_broucke_cefola_definitions() {
        let (e, i, node, periapsis, nu) = (0.1, 0.5, 1.0, 2.0, 0.3);
        let kep = keplerian(e, i, node, periapsis, nu);
        let eq = kep.to_equinoctial().unwrap();
        let mean_anomaly = TrueAnomaly::new(nu.rad())
            .to_mean(kep.eccentricity())
            .unwrap()
            .as_f64();

        assert_approx_eq!(eq.h(), e * sin(periapsis + node), atol <= 1e-15);
        assert_approx_eq!(eq.k(), e * cos(periapsis + node), atol <= 1e-15);
        assert_approx_eq!(eq.p(), tan(i / 2.0) * sin(node), atol <= 1e-15);
        assert_approx_eq!(eq.q(), tan(i / 2.0) * cos(node), atol <= 1e-15);
        assert_approx_eq!(
            eq.lambda().mod_two_pi(),
            Angle::new(mean_anomaly + periapsis + node).mod_two_pi(),
            atol <= 1e-12
        );
        assert_approx_eq!(
            eq.true_longitude().unwrap().as_f64(),
            nu + periapsis + node,
            atol <= 1e-12
        );
    }

    #[test]
    fn test_retrograde_definitions() {
        let (e, i, node, periapsis) = (0.1, 2.5, 1.0, 2.0);
        let eq = keplerian(e, i, node, periapsis, 0.0)
            .to_equinoctial()
            .unwrap();

        assert_eq!(eq.retrograde_factor(), RetrogradeFactor::Retrograde);
        assert_approx_eq!(eq.k(), e * cos(periapsis - node), atol <= 1e-15);
        assert_approx_eq!(eq.p(), sin(node) / tan(i / 2.0), atol <= 1e-15);
        assert_approx_eq!(eq.q(), cos(node) / tan(i / 2.0), atol <= 1e-15);
    }

    #[test]
    fn test_circular_equatorial_lambda_is_true_longitude() {
        let eq = keplerian(0.0, 0.0, 0.0, 0.0, PI / 2.0)
            .to_equinoctial()
            .unwrap();
        assert_eq!(eq.h(), 0.0);
        assert_eq!(eq.k(), 0.0);
        assert_eq!(eq.p(), 0.0);
        assert_eq!(eq.q(), 0.0);
        assert_approx_eq!(eq.lambda().as_f64(), PI / 2.0, atol <= 1e-12);
    }

    #[test]
    fn test_longitudes_stay_in_revolution() {
        let lambda = Angle::new(1.0 + 3.0 * TAU);
        let eq = Equinoctial::try_new(
            7000.0.km(),
            0.1,
            0.2,
            0.0,
            0.0,
            lambda,
            RetrogradeFactor::Prograde,
        )
        .unwrap();
        assert!((eq.true_longitude().unwrap() - lambda).as_f64().abs() < PI);
        assert!((eq.eccentric_longitude().unwrap() - lambda).as_f64().abs() < PI);
        assert_approx_eq!(
            eq.to_modified_equinoctial()
                .unwrap()
                .to_equinoctial()
                .unwrap()
                .lambda(),
            lambda,
            atol <= 1e-12
        );
    }

    #[test]
    fn test_open_orbits_are_rejected() {
        let parabolic = Keplerian::builder()
            .with_semi_parameter(10000.0.km(), 1.0)
            .build()
            .unwrap();
        assert!(matches!(
            parabolic.to_equinoctial(),
            Err(EquinoctialError::NotElliptic(_))
        ));
        assert!(matches!(
            parabolic
                .to_modified_equinoctial()
                .unwrap()
                .to_equinoctial(),
            Err(EquinoctialError::NotElliptic(_))
        ));
        assert!(matches!(
            Equinoctial::try_new(
                7000.0.km(),
                0.0,
                1.5,
                0.0,
                0.0,
                0.0.rad(),
                RetrogradeFactor::Prograde
            ),
            Err(EquinoctialError::NotElliptic(_))
        ));
    }

    #[test]
    fn test_open_orbits_near_asymptotes_are_rejected_as_open() {
        // Some of these orbits have no valid modified equinoctial elements, but the conversion
        // still reports the open orbit.
        let e = 1.5;
        let mut invalid_modified_equinoctial = 0;
        for step in 0..200 {
            let mut nu = acos(-1.0 / e);
            for _ in 0..60 {
                nu = nu.next_down();
                let Ok(kep) = Keplerian::builder()
                    .with_semi_parameter(12000.0.km(), e)
                    .with_argument_of_periapsis((f64::from(step) * 0.0314159).rad())
                    .with_true_anomaly(nu.rad())
                    .build()
                else {
                    continue;
                };
                if kep.to_modified_equinoctial().is_err() {
                    invalid_modified_equinoctial += 1;
                }
                assert!(matches!(
                    kep.to_equinoctial(),
                    Err(EquinoctialError::NotElliptic(_))
                ));
            }
        }
        assert!(invalid_modified_equinoctial > 0);
    }

    #[test]
    fn test_near_parabolic_orbits_are_rejected() {
        // Eccentricities within 1e-8 of one are parabolic.
        let near_parabolic = Keplerian::builder()
            .with_semi_parameter(10000.0.km(), 1.0 - 5e-9)
            .build()
            .unwrap();
        assert!(matches!(
            near_parabolic.to_equinoctial(),
            Err(EquinoctialError::NotElliptic(_))
        ));
    }

    #[test]
    fn test_try_new_validates() {
        let prograde = RetrogradeFactor::Prograde;
        assert!(matches!(
            Equinoctial::try_new((-1.0).km(), 0.0, 0.0, 0.0, 0.0, 0.0.rad(), prograde),
            Err(EquinoctialError::InvalidSemiMajorAxis(_))
        ));
        assert!(matches!(
            Equinoctial::try_new(1.0.km(), 0.0, 0.0, f64::INFINITY, 0.0, 0.0.rad(), prograde),
            Err(EquinoctialError::NonFiniteElements)
        ));
    }

    #[test]
    fn test_eccentric_longitude_solves_keplers_equation() {
        let eq = keplerian(0.99, 1.0, 2.0, 3.0, 3.0)
            .to_equinoctial()
            .unwrap();
        let f = eq.eccentric_longitude().unwrap().as_f64();
        let residual = f - eq.k() * sin(f) + eq.h() * cos(f) - eq.lambda().as_f64();
        assert!(residual.abs() <= 4.0 * f64::EPSILON, "residual {residual}");
    }

    #[test]
    fn test_retrograde_factor_agrees_between_paths_near_polar_orbits() {
        let mut node = 0.0;
        while node < TAU {
            let cart = keplerian(0.1, PI / 2.0, node, 1.0, 0.5).to_cartesian(mu());
            let via_keplerian = cart.to_keplerian(mu()).unwrap().to_equinoctial().unwrap();
            let direct = cart.to_equinoctial(mu()).unwrap();
            assert_eq!(
                direct.retrograde_factor(),
                via_keplerian.retrograde_factor(),
                "node {node}"
            );
            node += 0.01;
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialization_validates() {
        let eq = keplerian(0.1, 0.5, 1.0, 2.0, 0.3).to_equinoctial().unwrap();
        let json = serde_json::to_value(eq).unwrap();
        let back: Equinoctial = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(back, eq);

        let mut hyperbolic = json;
        hyperbolic["h"] = 0.8.into();
        hyperbolic["k"] = 0.8.into();
        assert!(serde_json::from_value::<Equinoctial>(hyperbolic).is_err());
    }

    #[test]
    fn test_large_components_do_not_panic() {
        let prograde = RetrogradeFactor::Prograde;
        assert!(matches!(
            Equinoctial::try_new(7000.0.km(), 1e200, 1e200, 0.0, 0.0, 0.0.rad(), prograde),
            Err(EquinoctialError::NotElliptic(_))
        ));
        assert!(matches!(
            Equinoctial::try_new(7000.0.km(), 1.5e308, 1.5e308, 0.0, 0.0, 0.0.rad(), prograde),
            Err(EquinoctialError::InvalidEccentricity(_))
        ));
        let eq =
            Equinoctial::try_new(7000.0.km(), 0.0, 0.0, 1e200, 1e200, 0.0.rad(), prograde).unwrap();
        assert_approx_eq!(eq.inclination().as_f64(), PI, atol <= 1e-15);
        // The node components describe an orbit with i = π and Ω = 45°, the equivalent retrograde
        // elements therefore have a mean longitude shifted by -2Ω.
        let mu = GravitationalParameter::km3_per_s2(398600.4418);
        let retrograde = Equinoctial::try_new(
            7000.0.km(),
            0.0,
            0.0,
            0.0,
            0.0,
            (-90.0).deg(),
            RetrogradeFactor::Retrograde,
        )
        .unwrap();
        let actual = eq.to_cartesian(mu).unwrap();
        let expected = retrograde.to_cartesian(mu).unwrap();
        assert_approx_eq!(actual.position(), expected.position(), atol <= 1e-6);
        assert_approx_eq!(actual.velocity(), expected.velocity(), atol <= 1e-9);
    }

    #[test]
    fn test_vanishing_semi_parameter_is_rejected() {
        assert!(matches!(
            Equinoctial::try_new(
                Distance::new(5e-324),
                0.9,
                0.0,
                0.0,
                0.0,
                0.0.rad(),
                RetrogradeFactor::Prograde
            ),
            Err(EquinoctialError::InvalidSemiParameter(_))
        ));
    }

    #[test]
    fn test_overflowing_semi_major_axis_is_rejected() {
        let e = 1.0 - 1e-7;
        let mee = ModifiedEquinoctial::try_new(
            Distance::new(f64::MAX / 2.0),
            e,
            0.0,
            0.0,
            0.0,
            0.0.rad(),
            RetrogradeFactor::Prograde,
        )
        .unwrap();
        assert!(matches!(
            mee.to_equinoctial(),
            Err(EquinoctialError::InvalidSemiMajorAxis(_))
        ));
    }
}
