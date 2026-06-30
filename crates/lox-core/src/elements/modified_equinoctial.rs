// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2026 Marijan Smetko <msmetko@msmetko.xyz>
//
// SPDX-License-Identifier: MPL-2.0

//! Modified Equinoctial Elements (MEE), non-singular for circular,
//! equatorial, and parabolic orbits.
//!
//! Based on:
//! Walker, M. J. H., Ireland, B., & Owens, J. (1985). *A set of modified equinoctial orbit elements*.
//! Celestial Mechanics, 36, 409-419. <https://doi.org/10.1007/BF01227493>
//!
//! Erratum: <https://doi.org/10.1007/BF01238929>

use crate::anomalies::AnomalyError;
use crate::coords::Cartesian;
use crate::elements::keplerian::{GravitationalParameter, Keplerian, KeplerianError};
use crate::math::float::{atan, atan2, cos, sin, sqrt, tan};
use crate::units::{Angle, AngleUnits, Distance};
use glam::DVec3;
use thiserror::Error;

/// Modified Equinoctial Elements (MEE).
///
/// Non-singular for circular (e = 0) and equatorial (i = 0) orbits.
/// Also fully supports parabolic (e = 1) orbits, unlike Type I equinoctial elements
/// which use the semi-major axis. Singular at i = π (retrograde) where tan(i/2) diverges.
/// Formulas in parenthesis are from Keplerian notation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModifiedEquinoctial {
    /// Semi-latus rectum (semi-parameter) `p` \[m\].
    p: Distance,
    /// Eccentricity vector, component 1 (`f` = e · cos(ω + Ω)).
    f: f64,
    /// Eccentricity vector, component 2 (`g` = e · sin(ω + Ω)).
    g: f64,
    /// Node vector, component 1 (`h` = tan(i/2) · cos(Ω)).
    h: f64,
    /// Node vector, component 2 (`k` = tan(i/2) · sin(Ω)).
    k: f64,
    /// True longitude `l` = Ω + ω + ν.
    l: Angle,
}

/// Error returned when constructing or converting modified equinoctial elements.
#[derive(Debug, Clone, Error)]
pub enum ModifiedEquinoctialError {
    /// The anomaly conversion failed (e.g. attempting to convert or evaluate
    /// anomalies beyond physically hyperbolic bounds limits).
    #[error(transparent)]
    Anomaly(#[from] AnomalyError),
    /// The Keplerian conversion failed (e.g. during conversion back to Keplerian,
    /// encountering inconsistencies in reconstructing the orbital shape mapping).
    #[error(transparent)]
    Keplerian(#[from] KeplerianError),
}

impl ModifiedEquinoctial {
    /// Creates modified equinoctial elements from raw component values.
    ///
    /// `p` is the semi-latus rectum in meters, `f` = e·cos(ω+Ω), `g` = e·sin(ω+Ω),
    /// `h` = tan(i/2)·cos(Ω), `k` = tan(i/2)·sin(Ω), `l` = Ω + ω + ν.
    pub fn new(p: Distance, f: f64, g: f64, h: f64, k: f64, l: Angle) -> Self {
        Self { p, f, g, h, k, l }
    }

    /// Converts modified equinoctial elements to a Cartesian state.
    pub fn to_cartesian(
        &self,
        mu: GravitationalParameter,
    ) -> Result<Cartesian, ModifiedEquinoctialError> {
        let p = self.p.as_f64();
        let mu_f64 = mu.as_f64();

        let s2 = 1.0 + self.h * self.h + self.k * self.k;
        let f_hat = DVec3::new(
            1.0 + self.h * self.h - self.k * self.k,
            2.0 * self.h * self.k,
            -2.0 * self.k,
        ) / s2;
        let g_hat = DVec3::new(
            2.0 * self.h * self.k,
            1.0 - self.h * self.h + self.k * self.k,
            2.0 * self.h,
        ) / s2;

        let (sin_l, cos_l) = self.l.sin_cos();
        let w = 1.0 + self.f * cos_l + self.g * sin_l;
        let r_mag = p / w;

        let r_orb = DVec3::new(r_mag * cos_l, r_mag * sin_l, 0.0);
        let v_orb = DVec3::new(
            -sqrt(mu_f64 / p) * (sin_l + self.g),
            sqrt(mu_f64 / p) * (cos_l + self.f),
            0.0,
        );

        let r_vec = r_orb.x * f_hat + r_orb.y * g_hat;
        let v_vec = v_orb.x * f_hat + v_orb.y * g_hat;

        Ok(Cartesian::from_vecs(r_vec, v_vec))
    }

    /// Converts modified equinoctial elements back to Keplerian.
    pub fn to_keplerian(&self) -> Result<Keplerian, ModifiedEquinoctialError> {
        let e = self.eccentricity();
        let pomega = atan2(self.g, self.f); // ω + Ω
        let half_i_tan = sqrt(self.h * self.h + self.k * self.k);
        let i = 2.0 * atan(half_i_tan);
        let big_omega = atan2(self.k, self.h);

        let omega = pomega - big_omega;
        let nu = self.l.as_f64() - pomega;
        let semi_major_axis = Distance::new(self.p.as_f64() / (1.0 - e * e));

        let kep = Keplerian::builder()
            .with_semi_major_axis(semi_major_axis, e)
            .with_inclination(i.rad())
            .with_longitude_of_ascending_node(big_omega.rad().mod_two_pi())
            .with_argument_of_periapsis(omega.rad().mod_two_pi())
            .with_true_anomaly(nu.rad().mod_two_pi())
            .build()?;

        Ok(kep)
    }

    /// Returns the semi-latus rectum (semi-parameter).
    pub fn p(&self) -> Distance {
        self.p
    }

    /// Returns f = e·cos(ω + Ω).
    pub fn f(&self) -> f64 {
        self.f
    }

    /// Returns g = e·sin(ω + Ω).
    pub fn g(&self) -> f64 {
        self.g
    }

    /// Returns h = tan(i/2)·cos(Ω).
    pub fn h(&self) -> f64 {
        self.h
    }

    /// Returns k = tan(i/2)·sin(Ω).
    pub fn k(&self) -> f64 {
        self.k
    }

    /// Returns the true longitude `L` = Ω + ω + ν.
    pub fn l(&self) -> Angle {
        self.l
    }

    /// Returns the eccentricity e = √(f² + g²).
    pub fn eccentricity(&self) -> f64 {
        sqrt(self.f * self.f + self.g * self.g)
    }

    /// Returns the inclination i = 2·atan(√(h² + k²)) in radians.
    pub fn inclination(&self) -> f64 {
        2.0 * atan(sqrt(self.h * self.h + self.k * self.k))
    }
}

impl Cartesian {
    /// Converts the Cartesian state to modified equinoctial elements.
    pub fn to_modified_equinoctial(
        &self,
        mu: GravitationalParameter,
    ) -> Result<ModifiedEquinoctial, ModifiedEquinoctialError> {
        let r_vec = self.position();
        let v_vec = self.velocity();
        let r_mag = r_vec.length();
        let mu_f64 = mu.as_f64();

        // Angular momentum
        let h_vec = r_vec.cross(v_vec);
        let h_mag = h_vec.length();

        // Semi-parameter p
        let p = Distance::new((h_mag * h_mag) / mu_f64);

        // Eccentricity vector
        let e_vec = v_vec.cross(h_vec) / mu_f64 - r_vec / r_mag;

        // Equinoctial parameters h, k
        let h = -h_vec.y / (h_mag + h_vec.z);
        let k = h_vec.x / (h_mag + h_vec.z);

        // Equinoctial frame unit vectors
        let s2 = 1.0 + h * h + k * k;
        let f_hat = DVec3::new(1.0 + h * h - k * k, 2.0 * h * k, -2.0 * k) / s2;
        let g_hat = DVec3::new(2.0 * h * k, 1.0 - h * h + k * k, 2.0 * h) / s2;

        let f = e_vec.dot(f_hat);
        let g = e_vec.dot(g_hat);

        let x = r_vec.dot(f_hat);
        let y = r_vec.dot(g_hat);
        let l = Angle::from_atan2(y, x);

        Ok(ModifiedEquinoctial { p, f, g, h, k, l })
    }
}

impl Keplerian {
    /// Converts Keplerian elements to modified equinoctial.
    pub fn to_modified_equinoctial(&self) -> ModifiedEquinoctial {
        let e = self.eccentricity().as_f64();
        let i = self.inclination().as_f64();
        let omega = self.argument_of_periapsis().as_f64();
        let big_omega = self.longitude_of_ascending_node().as_f64();
        let nu = self.true_anomaly().as_f64();

        let p_val = self.semi_parameter();
        let pomega = omega + big_omega; // longitude of periapsis

        let f = e * cos(pomega);
        let g = e * sin(pomega);
        let half_i_tan = tan(i / 2.0);
        let h = half_i_tan * cos(big_omega);
        let k = half_i_tan * sin(big_omega);
        let l = Angle::new(pomega + nu);

        ModifiedEquinoctial {
            p: p_val,
            f,
            g,
            h,
            k,
            l,
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use lox_approx::assert_approx_eq;

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
        let eq = kep.to_modified_equinoctial();
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

        let eq = cart.to_modified_equinoctial(mu).unwrap();
        let cart2 = eq.to_cartesian(mu).unwrap();

        assert_approx_eq!(cart.position(), cart2.position(), rtol <= 1e-10);
        assert_approx_eq!(cart.velocity(), cart2.velocity(), rtol <= 1e-10);
    }

    #[test]
    fn test_circular_orbit() {
        // e = 0: f and g should be zero, l should be well-defined
        let kep = Keplerian::builder()
            .with_semi_major_axis(6878.137.km(), 0.0)
            .with_inclination(97.42_f64.to_radians().rad())
            .with_longitude_of_ascending_node(69.3_f64.to_radians().rad())
            .with_argument_of_periapsis(0.0.rad())
            .with_true_anomaly(45.0_f64.to_radians().rad())
            .build()
            .unwrap();

        let eq = kep.to_modified_equinoctial();

        assert_approx_eq!(eq.f(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.g(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.eccentricity(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.inclination(), 97.42_f64.to_radians(), atol <= 1e-12);
        assert!(eq.h().abs() > 0.1);
        assert!(eq.k().abs() > 0.1);

        let mu = GravitationalParameter::km3_per_s2(398600.4418);

        let c1 = kep.to_cartesian(mu);

        // Also test mapping directly from cartesian respects e=0
        let eq_from_c = c1.to_modified_equinoctial(mu).unwrap();
        assert_approx_eq!(eq_from_c.f(), 0.0, atol <= 1e-12);
        assert_approx_eq!(eq_from_c.g(), 0.0, atol <= 1e-12);

        let c2 = eq.to_cartesian(mu).unwrap();
        assert_approx_eq!(c1.position(), c2.position(), rtol <= 1e-10);
        assert_approx_eq!(c1.velocity(), c2.velocity(), rtol <= 1e-10);
    }

    #[test]
    fn test_equatorial_orbit() {
        // i = 0: h and k should be zero
        let kep = Keplerian::builder()
            .with_semi_major_axis(42164.0.km(), 0.001)
            .with_inclination(0.0.rad())
            .with_longitude_of_ascending_node(0.0.rad())
            .with_argument_of_periapsis(45.0_f64.to_radians().rad())
            .with_true_anomaly(30.0_f64.to_radians().rad())
            .build()
            .unwrap();

        let eq = kep.to_modified_equinoctial();

        assert_approx_eq!(eq.h(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.k(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.inclination(), 0.0, atol <= 1e-15);
        assert!(eq.eccentricity() > 0.0);

        let mu = GravitationalParameter::km3_per_s2(398600.4418);
        let c1 = kep.to_cartesian(mu);

        let eq_from_c = c1.to_modified_equinoctial(mu).unwrap();
        assert_approx_eq!(eq_from_c.h(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq_from_c.k(), 0.0, atol <= 1e-15);

        let c2 = eq.to_cartesian(mu).unwrap();
        assert_approx_eq!(c1.position(), c2.position(), rtol <= 1e-10);
        assert_approx_eq!(c1.velocity(), c2.velocity(), rtol <= 1e-10);
    }

    #[test]
    fn test_circular_equatorial_orbit() {
        // Both e = 0 and i = 0: f, g, h, k all zero
        let kep = Keplerian::builder()
            .with_semi_major_axis(42164.0.km(), 0.0)
            .with_inclination(0.0.rad())
            .with_longitude_of_ascending_node(0.0.rad())
            .with_argument_of_periapsis(0.0.rad())
            .with_true_anomaly(90.0_f64.to_radians().rad())
            .build()
            .unwrap();

        let eq = kep.to_modified_equinoctial();

        assert_approx_eq!(eq.f(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.g(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.h(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.k(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.eccentricity(), 0.0, atol <= 1e-15);
        assert_approx_eq!(eq.inclination(), 0.0, atol <= 1e-15);

        // lambda should equal the true anomaly
        assert_approx_eq!(eq.l().as_f64(), 90.0_f64.to_radians(), atol <= 1e-12);

        let mu = GravitationalParameter::km3_per_s2(398600.4418);
        let c1 = kep.to_cartesian(mu);
        let c2 = eq.to_cartesian(mu).unwrap();
        assert_approx_eq!(c1.position(), c2.position(), rtol <= 1e-10);
        assert_approx_eq!(c1.velocity(), c2.velocity(), rtol <= 1e-10);
    }

    #[test]
    fn test_parabolic_orbit_directly() {
        let mu = general_orbit_mu();
        let mu_f64 = mu.as_f64();
        // Construct a parabolic state (e = 1.0)
        // Set an arbitrary p
        let p_vec = 10000.0 * 1000.0; // 10,000 km

        // Let's create it on the x axis so r = p / 2, v = sqrt(2 mu / r)
        let r_mag = p_vec / 2.0;
        let v_mag = (2.0 * mu_f64 / r_mag).sqrt();

        let cart = Cartesian::builder()
            .position(r_mag.m(), 0.0.m(), 0.0.m())
            .velocity(0.0.mps(), v_mag.mps(), 0.0.mps())
            .build();

        let eq = cart.to_modified_equinoctial(mu).unwrap();

        // For this state, e must be exactly 1.0!
        assert_approx_eq!(eq.eccentricity(), 1.0, atol <= 1e-12);

        // Reconstruct and compare
        let cart2 = eq.to_cartesian(mu).unwrap();
        assert_approx_eq!(cart.position(), cart2.position(), rtol <= 1e-10);
        assert_approx_eq!(cart.velocity(), cart2.velocity(), rtol <= 1e-10);
    }

    #[test]
    fn test_error_and_new_constructor() {
        let mee = ModifiedEquinoctial::new(100.0.m(), 0.1, 0.2, 0.3, 0.4, 0.5.rad());
        assert_eq!(mee.p(), 100.0.m());
        assert_eq!(mee.f(), 0.1);
        assert_eq!(mee.g(), 0.2);
        assert_eq!(mee.h(), 0.3);
        assert_eq!(mee.k(), 0.4);
        assert_eq!(mee.l(), 0.5.rad());

        let err1: ModifiedEquinoctialError =
            crate::elements::keplerian::KeplerianError::MissingShape.into();
        assert_eq!(
            err1.to_string(),
            "no orbital shape parameters (semi-major axis and eccentricity, radii, or altitudes) were provided"
        );

        let err2: ModifiedEquinoctialError = crate::anomalies::AnomalyError::InvalidTrueAnomaly {
            nu: 1.0.rad(),
            max_nu: 0.5.rad(),
        }
        .into();
        assert!(err2.to_string().contains("outside valid range"));
    }
}
