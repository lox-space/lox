// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Shared math for Kozai-based analytical propagators.
//!
//! Provides secular rate formulas (J2 and J4) and Kwok short-period
//! corrections used by the [`super::j2`] and [`super::j4`] propagator
//! modules.
//!
//! # References
//!
//! - Kozai, Y. (1959). "The Motion of a Close Earth Satellite."
//!   *The Astronomical Journal*, 64, 367.
//! - Vallado, D. A. (2013). *Fundamentals of Astrodynamics and
//!   Applications*, 4th ed. Microcosm Press. pp. 372, 647–654, 708–710.

use lox_core::anomalies::{AnomalyError, MeanAnomaly, TrueAnomaly};
use lox_core::coords::Cartesian;
use lox_core::elements::{Eccentricity, GravitationalParameter, Keplerian, MeanElements};
use lox_core::units::{AngleUnits, Distance};

/// Body constants needed by the propagator.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BodyConstants {
    /// Gravitational parameter [m³/s²].
    pub mu: f64,
    /// J2 zonal harmonic coefficient.
    pub j2: f64,
    /// Equatorial radius [m].
    pub r_eq: f64,
}

/// Pre-computed secular rates for a Kozai-based propagator.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SecularRates {
    /// Perturbed mean motion [rad/s].
    pub n_bar: f64,
    /// Secular rate of RAAN [rad/s].
    pub raan_dot: f64,
    /// Secular rate of argument of periapsis [rad/s].
    pub aop_dot: f64,
}

/// Compute first-order J2 secular rates (Kozai).
///
/// Reference: Vallado (2013), p. 372.
pub fn j2_secular_rates(a: f64, e: f64, i: f64, mu: f64, j2: f64, r_eq: f64) -> SecularRates {
    let p = a * (1.0 - e * e);
    let n0 = (mu / (a * a * a)).sqrt();
    let beta = (1.0 - e * e).sqrt();
    let sin_i = i.sin();
    let cos_i = i.cos();
    let sin_i2 = sin_i * sin_i;

    let kn2 = j2 * (r_eq / p).powi(2) * beta;
    let n_bar = n0 * (1.0 + 0.75 * kn2 * (2.0 - 3.0 * sin_i2));

    let k_bar_2 = n_bar * j2 * (r_eq / p).powi(2);
    let raan_dot = -1.5 * k_bar_2 * cos_i;
    let aop_dot = 0.75 * k_bar_2 * (4.0 - 5.0 * sin_i2);

    SecularRates {
        n_bar,
        raan_dot,
        aop_dot,
    }
}

/// Compute secular rates with J2, J2², and J4 terms.
///
/// Reference: Vallado (2013), pp. 647–654; Hoots & Roehrich (1980).
pub fn j4_secular_rates(
    a: f64,
    e: f64,
    i: f64,
    mu: f64,
    j2: f64,
    j4: f64,
    r_eq: f64,
) -> SecularRates {
    let e2 = e * e;
    let p = a * (1.0 - e2);
    let p2 = p * p;
    let p4 = p2 * p2;
    let n0 = (mu / (a * a * a)).sqrt();
    let beta = (1.0 - e2).sqrt();
    let sin_i = i.sin();
    let cos_i = i.cos();
    let sin_i2 = sin_i * sin_i;
    let sin_i4 = sin_i2 * sin_i2;
    let cos_i4 = cos_i * cos_i * cos_i * cos_i;
    let r_eq2 = r_eq * r_eq;
    let r_eq4 = r_eq2 * r_eq2;

    // Perturbed mean motion (Kozai + J2² + J4).
    let kn2 = j2 * r_eq2 / p2 * beta;
    let kn22 = j2 * j2 * r_eq4 / p4 * beta;
    let kn4 = j4 * r_eq4 / p4 * beta;
    let beta2 = beta * beta;

    let n_bar = n0
        * (1.0
            + 0.75 * kn2 * (2.0 - 3.0 * sin_i2)
            + (3.0 / 128.0)
                * kn22
                * (120.0 + 64.0 * beta - 40.0 * beta2
                    + (-240.0 - 192.0 * beta + 40.0 * beta2) * sin_i2
                    + (105.0 + 144.0 * beta + 25.0 * beta2) * sin_i4)
            - (45.0 / 128.0) * kn4 * e2 * (-8.0 + 40.0 * sin_i2 - 35.0 * sin_i4));

    // Secular rate coefficients.
    // Leading J2 terms use perturbed n_bar; higher-order use unperturbed n0.
    let k_bar_2 = n_bar * j2 * r_eq2 / p2;
    let k_bar_22 = n_bar * j2 * j2 * r_eq4 / p4;
    let k_22 = n0 * j2 * j2 * r_eq4 / p4;
    let k_4 = n0 * j4 * r_eq4 / p4;

    // RAAN secular rate.
    let raan_dot = -1.5 * k_bar_2 * cos_i
        + (3.0 / 32.0)
            * k_bar_22
            * cos_i
            * (-36.0 - 4.0 * e2 + 48.0 * beta + (40.0 - 5.0 * e2 - 72.0 * beta) * sin_i2)
        + (15.0 / 32.0) * k_4 * cos_i * (8.0 + 12.0 * e2 - (14.0 + 21.0 * e2) * sin_i2);

    // Argument of periapsis secular rate.
    let aop_dot = 0.75 * k_bar_2 * (4.0 - 5.0 * sin_i2)
        + (3.0 / 128.0)
            * k_bar_22
            * (384.0 + 96.0 * e2 - 384.0 * beta
                + (-824.0 - 116.0 * e2 + 1056.0 * beta) * sin_i2
                + (430.0 - 5.0 * e2 - 720.0 * beta) * sin_i4)
        - (15.0 / 16.0) * k_22 * e2 * cos_i4
        - (15.0 / 128.0)
            * k_4
            * (64.0 + 72.0 * e2 - (248.0 + 252.0 * e2) * sin_i2 + (196.0 + 189.0 * e2) * sin_i4);

    SecularRates {
        n_bar,
        raan_dot,
        aop_dot,
    }
}

/// Propagate mean elements forward by `dt` seconds using pre-computed
/// secular rates.
///
/// `a`, `e`, `i` are constant; `raan`, `aop`, `M` are linearly propagated.
pub fn propagate_mean(kep: &Keplerian, m0: f64, rates: &SecularRates, dt: f64) -> MeanElements {
    use std::f64::consts::TAU;
    MeanElements {
        a: kep.semi_major_axis().as_f64(),
        e: kep.eccentricity().as_f64(),
        i: kep.inclination().as_f64(),
        raan: (kep.longitude_of_ascending_node().as_f64() + rates.raan_dot * dt).rem_euclid(TAU),
        aop: (kep.argument_of_periapsis().as_f64() + rates.aop_dot * dt).rem_euclid(TAU),
        m: (m0 + rates.n_bar * dt).rem_euclid(TAU),
    }
}

/// Convert mean Keplerian elements to a Cartesian state.
///
/// Solves the Kepler equation for the true anomaly and builds the
/// position/velocity vector.
pub fn mean_to_cartesian(el: &MeanElements, mu: f64) -> Result<Cartesian, AnomalyError> {
    let ecc = Eccentricity::try_new(el.e).expect("eccentricity should be non-negative");
    let f = MeanAnomaly::new(el.m.rad()).to_true(ecc)?;
    let kep = Keplerian::builder()
        .with_semi_major_axis(Distance::new(el.a), el.e)
        .with_inclination(el.i.rad().mod_two_pi())
        .with_longitude_of_ascending_node(el.raan.rad().mod_two_pi())
        .with_argument_of_periapsis(el.aop.rad().mod_two_pi())
        .with_true_anomaly(f.as_angle())
        .build()
        .expect("propagated elements should be valid");
    Ok(kep.to_cartesian(GravitationalParameter::m3_per_s2(mu)))
}

/// Apply Kwok short-period J2 corrections to mean elements and return
/// the osculating Cartesian state.
///
/// The corrections are computed in the (r, ṙ, p, u) space which is
/// non-singular at e = 0. The osculating eccentricity is recovered as
/// `e_osc = √(A² + B²)` from the corrected radius and velocity.
///
/// Reference: Vallado (2013), pp. 708–710.
pub fn kwok_osculating_cartesian(
    el: &MeanElements,
    body: &BodyConstants,
) -> Result<Cartesian, AnomalyError> {
    let MeanElements {
        a,
        e,
        i,
        raan,
        aop,
        m,
    } = *el;
    let BodyConstants { mu, j2, r_eq } = *body;

    let ecc = Eccentricity::try_new(e).expect("eccentricity should be non-negative");
    let f = MeanAnomaly::new(m.rad()).to_true(ecc)?.as_f64();
    let (sin_f, cos_f) = f.sin_cos();

    let p = a * (1.0 - e * e);
    let beta = (1.0 - e * e).sqrt();
    let r = p / (1.0 + e * cos_f);
    let rdot = (mu / p).sqrt() * e * sin_f;
    let u = aop + f;
    let kj2 = j2 * r_eq * r_eq;

    let sin_i = i.sin();
    let cos_i = i.cos();
    let sin_i2 = sin_i * sin_i;
    let cos_i2 = cos_i * cos_i;
    let (sin_2u, cos_2u) = (2.0 * u).sin_cos();

    let e_cos_f = e * cos_f;
    let e_sin_f = e * sin_f;

    // Auxiliary combinations (Vallado p. 709).
    let aux1 = 3.0 * cos_2u + 3.0 * e * (aop * 2.0 + f).cos() + e * (aop * 2.0 + 3.0 * f).cos();

    // Short-period corrections.
    let di = kj2 * sin_i * cos_i / (4.0 * p * p) * aux1;

    let dp = kj2 * sin_i2 / (2.0 * p) * aux1;

    let draan = -kj2 * cos_i / (4.0 * p * p)
        * (6.0 * (f - m + e_sin_f)
            - 3.0 * sin_2u
            - 3.0 * e * (aop * 2.0 + f).sin()
            - e * (aop * 2.0 + 3.0 * f).sin());

    let dr = -kj2 / (4.0 * p)
        * ((3.0 * cos_i2 - 1.0) * (2.0 * beta / (1.0 + e_cos_f) + e_cos_f / (1.0 + beta) + 1.0)
            - sin_i2 * cos_2u);

    let drdot = kj2 * (mu / p).sqrt() / (4.0 * p * p.sqrt())
        * ((3.0 * cos_i2 - 1.0) * e_sin_f * (beta + (1.0 + e_cos_f).powi(2) / (1.0 + beta))
            - 2.0 * sin_i2 * (1.0 - e_cos_f).powi(2) * sin_2u);

    let du = kj2 / (8.0 * p * p)
        * ((6.0 - 30.0 * cos_i2) * (f - m)
            + 4.0 * e_sin_f * (1.0 - 6.0 * cos_i2 - (3.0 * cos_i2 - 1.0) / (1.0 + beta))
            - (3.0 * cos_i2 - 1.0) / (1.0 + beta) * e * e * (2.0 * f).sin()
            + (5.0 * cos_i2 - 2.0) * 2.0 * e * (aop * 2.0 + f).sin()
            + (7.0 * cos_i2 - 1.0) * sin_2u
            + 2.0 * cos_i2 * e * (aop * 2.0 + 3.0 * f).sin());

    // Reconstruct osculating elements.
    let r_osc = r + dr;
    let rdot_osc = rdot + drdot;
    let p_osc = p + dp;

    let aa = p_osc / r_osc - 1.0;
    let bb = (p_osc / mu).sqrt() * rdot_osc;
    let e_osc = (aa * aa + bb * bb).sqrt();
    let a_osc = p_osc / (1.0 - e_osc * e_osc);
    let i_osc = i + di;
    let raan_osc = raan + draan;
    let u_osc = u + du;
    let f_osc = bb.atan2(aa);
    let aop_osc = u_osc - f_osc;

    let ecc_osc = Eccentricity::try_new(e_osc).expect("osculating eccentricity should be valid");
    let ta = TrueAnomaly::new(f_osc.rad());
    let kep = Keplerian::builder()
        .with_semi_major_axis(Distance::new(a_osc), ecc_osc.as_f64())
        .with_inclination(i_osc.rad().mod_two_pi())
        .with_longitude_of_ascending_node(raan_osc.rad().mod_two_pi())
        .with_argument_of_periapsis(aop_osc.rad().mod_two_pi())
        .with_true_anomaly(ta.as_angle())
        .build()
        .expect("osculating elements should be valid");
    Ok(kep.to_cartesian(GravitationalParameter::m3_per_s2(mu)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_test_utils::assert_approx_eq;

    const MU_EARTH: f64 = 3.986004418e14;
    const J2_EARTH: f64 = 1.08262668e-3;
    const J4_EARTH: f64 = -1.65597e-6;
    const R_EQ: f64 = 6.378137e6;

    #[test]
    fn test_j2_secular_rates_sso() {
        // Sun-synchronous orbit: RAAN drift should be ~0.9856°/day eastward.
        let a = 6_878_137.0;
        let e = 0.0;
        let i = 97.42_f64.to_radians();
        let rates = j2_secular_rates(a, e, i, MU_EARTH, J2_EARTH, R_EQ);

        let raan_deg_per_day = rates.raan_dot.to_degrees() * 86400.0;
        // SSO drift ≈ +0.9856°/day
        assert_approx_eq!(raan_deg_per_day, 0.9856, atol <= 0.01);
    }

    #[test]
    fn test_j4_secular_rates_differ_from_j2() {
        let a = 7_000_000.0;
        let e = 0.001;
        let i = 51.6_f64.to_radians();

        let r2 = j2_secular_rates(a, e, i, MU_EARTH, J2_EARTH, R_EQ);
        let r4 = j4_secular_rates(a, e, i, MU_EARTH, J2_EARTH, J4_EARTH, R_EQ);

        // J4 rates should differ from J2 by a small but nonzero amount.
        let draan = (r4.raan_dot - r2.raan_dot).abs();
        let daop = (r4.aop_dot - r2.aop_dot).abs();
        assert!(draan > 0.0);
        assert!(daop > 0.0);
        // The correction should be O(J4/J2) ≈ 1e-3 relative.
        assert!(draan / r2.raan_dot.abs() < 0.01);
        assert!(daop / r2.aop_dot.abs() < 0.01);
    }
}
