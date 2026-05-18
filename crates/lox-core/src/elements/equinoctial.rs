// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Type I equinoctial orbital elements, non-singular for circular and
//! equatorial orbits.
//!
//! The six equinoctial elements are:
//!
//! | Element | Definition | Singularity-free at |
//! |---------|-----------|---------------------|
//! | `a`     | semi-major axis | — |
//! | `k`     | e · cos(ω + Ω) | e = 0 |
//! | `h`     | e · sin(ω + Ω) | e = 0 |
//! | `p`     | tan(i/2) · cos(Ω) | i = 0 |
//! | `q`     | tan(i/2) · sin(Ω) | i = 0 |
//! | `λ`     | M + ω + Ω (mean longitude) | e = 0, i = 0 |
//!
//! These elements are singular at i = π (retrograde orbits) where tan(i/2)
//! diverges. For retrograde orbits, Type II equinoctial elements using
//! cot(i/2) should be used instead (not yet implemented).

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use num_traits::Float;
use thiserror::Error;

use crate::anomalies::AnomalyError;
use crate::coords::Cartesian;
use crate::elements::keplerian::{GravitationalParameter, Keplerian, KeplerianError};
use crate::units::{AngleUnits, Distance};

/// Type I equinoctial orbital elements.
///
/// Non-singular for circular (e = 0) and equatorial (i = 0) orbits.
/// Singular at i = π (retrograde).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Equinoctial {
    /// Semi-major axis \[m\].
    a: Distance,
    /// e · cos(ω + Ω).
    k: f64,
    /// e · sin(ω + Ω).
    h: f64,
    /// tan(i/2) · cos(Ω).
    p: f64,
    /// tan(i/2) · sin(Ω).
    q: f64,
    /// Mean longitude M + ω + Ω \[rad\].
    lambda: f64,
}

/// Error returned when constructing or converting equinoctial elements.
#[derive(Debug, Clone, Error)]
pub enum EquinoctialError {
    /// The anomaly conversion failed.
    #[error(transparent)]
    Anomaly(#[from] AnomalyError),
    /// The Keplerian conversion failed.
    #[error(transparent)]
    Keplerian(#[from] KeplerianError),
}

impl Equinoctial {
    /// Creates equinoctial elements from raw component values.
    ///
    /// `a` is the semi-major axis in meters, `k` = e·cos(ω+Ω),
    /// `h` = e·sin(ω+Ω), `p` = tan(i/2)·cos(Ω), `q` = tan(i/2)·sin(Ω),
    /// `lambda` = M + ω + Ω in radians.
    pub fn new(a: Distance, k: f64, h: f64, p: f64, q: f64, lambda: f64) -> Self {
        Self {
            a,
            k,
            h,
            p,
            q,
            lambda,
        }
    }

    /// Converts Keplerian elements to equinoctial.
    ///
    /// Requires the true anomaly → mean anomaly conversion, which can fail
    /// for hyperbolic orbits beyond the asymptote angle.
    pub fn from_keplerian(kep: &Keplerian) -> Result<Self, AnomalyError> {
        let m = kep.true_anomaly().to_mean(kep.eccentricity())?.as_f64();
        let e = kep.eccentricity().as_f64();
        let i = kep.inclination().as_f64();
        let omega = kep.argument_of_periapsis().as_f64();
        let big_omega = kep.longitude_of_ascending_node().as_f64();

        let pomega = omega + big_omega; // longitude of periapsis
        let k = e * pomega.cos();
        let h = e * pomega.sin();
        let half_i_tan = (i / 2.0).tan();
        let p = half_i_tan * big_omega.cos();
        let q = half_i_tan * big_omega.sin();
        let lambda = m + pomega;

        Ok(Self {
            a: kep.semi_major_axis(),
            k,
            h,
            p,
            q,
            lambda,
        })
    }

    /// Converts equinoctial elements back to Keplerian.
    pub fn to_keplerian(&self) -> Result<Keplerian, EquinoctialError> {
        let e = self.eccentricity();
        let pomega = self.h.atan2(self.k); // ω + Ω
        let half_i_tan = (self.p * self.p + self.q * self.q).sqrt();
        let i = 2.0 * half_i_tan.atan();
        let big_omega = self.q.atan2(self.p);
        let omega = pomega - big_omega;
        let m = self.lambda - pomega;

        let kep = Keplerian::builder()
            .with_semi_major_axis(self.a, e)
            .with_inclination(i.rad())
            .with_longitude_of_ascending_node(big_omega.rad().mod_two_pi())
            .with_argument_of_periapsis(omega.rad().mod_two_pi())
            .with_mean_anomaly(m.rad())
            .build()?;

        Ok(kep)
    }

    /// Converts a Cartesian state to equinoctial elements (via Keplerian).
    pub fn from_cartesian(
        cart: &Cartesian,
        mu: GravitationalParameter,
    ) -> Result<Self, AnomalyError> {
        Self::from_keplerian(&cart.to_keplerian(mu))
    }

    /// Converts equinoctial elements to a Cartesian state (via Keplerian).
    pub fn to_cartesian(&self, mu: GravitationalParameter) -> Result<Cartesian, EquinoctialError> {
        Ok(self.to_keplerian()?.to_cartesian(mu))
    }

    /// Returns the semi-major axis.
    pub fn a(&self) -> Distance {
        self.a
    }

    /// Returns k = e·cos(ω + Ω).
    pub fn k(&self) -> f64 {
        self.k
    }

    /// Returns h = e·sin(ω + Ω).
    pub fn h(&self) -> f64 {
        self.h
    }

    /// Returns p = tan(i/2)·cos(Ω).
    pub fn p(&self) -> f64 {
        self.p
    }

    /// Returns q = tan(i/2)·sin(Ω).
    pub fn q(&self) -> f64 {
        self.q
    }

    /// Returns the mean longitude λ = M + ω + Ω in radians.
    pub fn lambda(&self) -> f64 {
        self.lambda
    }

    /// Returns the eccentricity e = √(k² + h²).
    pub fn eccentricity(&self) -> f64 {
        (self.k * self.k + self.h * self.h).sqrt()
    }

    /// Returns the inclination i = 2·atan(√(p² + q²)) in radians.
    pub fn inclination(&self) -> f64 {
        2.0 * (self.p * self.p + self.q * self.q).sqrt().atan()
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;

    use crate::elements::keplerian::GravitationalParameter;
    use crate::units::{AngleUnits, DistanceUnits, VelocityUnits};

    use super::*;

    fn general_orbit_keplerian() -> Keplerian {
        Keplerian::builder()
            .with_semi_major_axis(24464.560.km(), 0.7311)
            .with_inclination(0.122138.rad())
            .with_longitude_of_ascending_node(1.00681.rad())
            .with_argument_of_periapsis(3.10686.rad())
            .with_true_anomaly(0.44369564302687126.rad())
            .build()
            .unwrap()
    }

    fn general_orbit_mu() -> GravitationalParameter {
        GravitationalParameter::km3_per_s2(398600.43550702266)
    }

    #[test]
    fn test_keplerian_roundtrip() {
        let kep = general_orbit_keplerian();
        let eq = Equinoctial::from_keplerian(&kep).unwrap();
        let kep2 = eq.to_keplerian().unwrap();

        assert_approx_eq!(
            kep.semi_major_axis().as_f64(),
            kep2.semi_major_axis().as_f64(),
            rtol <= 1e-12
        );
        assert_approx_eq!(
            kep.eccentricity().as_f64(),
            kep2.eccentricity().as_f64(),
            atol <= 1e-12
        );
        assert_approx_eq!(
            kep.inclination().as_f64(),
            kep2.inclination().as_f64(),
            atol <= 1e-12
        );

        // Compare Cartesian positions to avoid angle wrapping issues
        let mu = general_orbit_mu();
        let c1 = kep.to_cartesian(mu);
        let c2 = kep2.to_cartesian(mu);
        assert_approx_eq!(c1.position(), c2.position(), rtol <= 1e-10);
        assert_approx_eq!(c1.velocity(), c2.velocity(), rtol <= 1e-10);
    }

    #[test]
    fn test_cartesian_roundtrip() {
        let mu = general_orbit_mu();
        let cart = Cartesian::builder()
            .position(
                -0.107622532467967e7.m(),
                -0.676589636432773e7.m(),
                -0.332308783350379e6.m(),
            )
            .velocity(
                0.935685775154103e4.mps(),
                -0.331234775037644e4.mps(),
                -0.118801577532701e4.mps(),
            )
            .build();

        let eq = Equinoctial::from_cartesian(&cart, mu).unwrap();
        let cart2 = eq.to_cartesian(mu).unwrap();

        assert_approx_eq!(cart.position(), cart2.position(), rtol <= 1e-8);
        assert_approx_eq!(cart.velocity(), cart2.velocity(), rtol <= 1e-6);
    }

    #[test]
    fn test_circular_orbit() {
        // e = 0: k and h should be zero, lambda should be well-defined
        let kep = Keplerian::builder()
            .with_semi_major_axis(6878.137.km(), 0.0)
            .with_inclination(97.42_f64.to_radians().rad())
            .with_longitude_of_ascending_node(69.3_f64.to_radians().rad())
            .with_argument_of_periapsis(0.0.rad())
            .with_true_anomaly(45.0_f64.to_radians().rad())
            .build()
            .unwrap();

        let eq = Equinoctial::from_keplerian(&kep).unwrap();

        assert_approx_eq!(eq.k(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.h(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.eccentricity(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.inclination(), 97.42_f64.to_radians(), atol <= 1e-12);
        // p and q should be nonzero (i ≠ 0)
        assert!(eq.p().abs() > 0.1);
        assert!(eq.q().abs() > 0.1);

        // Roundtrip back to Cartesian should preserve state
        let mu = GravitationalParameter::km3_per_s2(398600.4418);
        let c1 = kep.to_cartesian(mu);
        let c2 = eq.to_cartesian(mu).unwrap();
        assert_approx_eq!(c1.position(), c2.position(), rtol <= 1e-8);
        assert_approx_eq!(c1.velocity(), c2.velocity(), rtol <= 1e-6);
    }

    #[test]
    fn test_equatorial_orbit() {
        // i = 0: p and q should be zero
        let kep = Keplerian::builder()
            .with_semi_major_axis(42164.0.km(), 0.001)
            .with_inclination(0.0.rad())
            .with_longitude_of_ascending_node(0.0.rad())
            .with_argument_of_periapsis(45.0_f64.to_radians().rad())
            .with_true_anomaly(30.0_f64.to_radians().rad())
            .build()
            .unwrap();

        let eq = Equinoctial::from_keplerian(&kep).unwrap();

        assert_approx_eq!(eq.p(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.q(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.inclination(), 0.0, atol <= 1e-15);
        assert!(eq.eccentricity() > 0.0);

        // Roundtrip
        let mu = GravitationalParameter::km3_per_s2(398600.4418);
        let c1 = kep.to_cartesian(mu);
        let c2 = eq.to_cartesian(mu).unwrap();
        assert_approx_eq!(c1.position(), c2.position(), rtol <= 1e-8);
        assert_approx_eq!(c1.velocity(), c2.velocity(), rtol <= 1e-6);
    }

    #[test]
    fn test_circular_equatorial_orbit() {
        // Both e = 0 and i = 0: k, h, p, q all zero
        let kep = Keplerian::builder()
            .with_semi_major_axis(42164.0.km(), 0.0)
            .with_inclination(0.0.rad())
            .with_longitude_of_ascending_node(0.0.rad())
            .with_argument_of_periapsis(0.0.rad())
            .with_true_anomaly(90.0_f64.to_radians().rad())
            .build()
            .unwrap();

        let eq = Equinoctial::from_keplerian(&kep).unwrap();

        assert_approx_eq!(eq.k(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.h(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.p(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.q(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.eccentricity(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.inclination(), 0.0, atol <= 1e-15);

        // lambda should equal the true anomaly (since M = nu at e=0, and ω = Ω = 0)
        assert_approx_eq!(eq.lambda(), 90.0_f64.to_radians(), atol <= 1e-12);

        // Roundtrip
        let mu = GravitationalParameter::km3_per_s2(398600.4418);
        let c1 = kep.to_cartesian(mu);
        let c2 = eq.to_cartesian(mu).unwrap();
        assert_approx_eq!(c1.position(), c2.position(), rtol <= 1e-8);
        assert_approx_eq!(c1.velocity(), c2.velocity(), rtol <= 1e-6);
    }
}
