// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Orbital Anomaly Conversions
//!
//! This module provides conversions between true anomaly, eccentric anomaly, and mean anomaly
//! for all conic section orbit types: circular, elliptic, parabolic, and hyperbolic.
//!
//! # Conventions
//! - All angles use the [`Angle`] type, normalized to (-π, π] unless otherwise specified
//! - Eccentricity uses the [`Eccentricity`] type which validates e >= 0
//! - Orbit types: circular (e≈0), elliptic (0<e<1), parabolic (e≈1), hyperbolic (e>1)
//! - True anomaly (ν): angle from periapsis to the position vector
//! - Eccentric anomaly (E): auxiliary angle for elliptic orbits, range (-π, π]
//! - Hyperbolic anomaly (F): hyperbolic angle for hyperbolic orbits (unbounded)
//! - Parabolic anomaly (D): tan(ν/2) for parabolic orbits (unbounded)
//! - Mean anomaly (M): time-like parameter proportional to time since periapsis, range (-π, π]
//!
//! # Anomaly Enum Convention
//! The [`Anomaly`] enum uses an overloaded convention for `Anomaly::Eccentric`:
//! - For elliptic/circular orbits: represents eccentric anomaly E
//! - For hyperbolic orbits: represents hyperbolic anomaly F
//! - For parabolic orbits: represents parabolic anomaly D (stored as radians)

use lox_test_utils::approx_eq::{ApproxEq, ApproxEqResults};
use thiserror::Error;

use crate::{
    elements::{Eccentricity, OrbitType},
    units::{Angle, AngleUnits},
};

use core::marker::PhantomData;
use std::{
    fmt::Display,
    ops::{Add, Neg, Sub},
};

/// Sealed marker trait for anomaly kinds
pub trait AnomalyKind: sealed::Sealed + std::fmt::Debug {}

mod sealed {
    /// Sealed trait to prevent external implementation
    pub trait Sealed {}
    impl Sealed for super::TrueKind {}
    impl Sealed for super::EccentricKind {}
    impl Sealed for super::MeanKind {}
}

/// Marker type for true anomaly
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TrueKind;

/// Marker type for eccentric anomaly (E for elliptic, F for hyperbolic, D for parabolic)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EccentricKind;

/// Marker type for mean anomaly
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MeanKind;

impl AnomalyKind for TrueKind {}
impl AnomalyKind for EccentricKind {}
impl AnomalyKind for MeanKind {}

/// Generic anomaly type parameterized by kind marker
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Anomaly<K: AnomalyKind> {
    anomaly: Angle,
    _kind: PhantomData<K>,
}

impl<K> Display for Anomaly<K>
where
    K: AnomalyKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.anomaly.fmt(f)
    }
}

impl<K> Anomaly<K>
where
    K: AnomalyKind,
{
    /// Returns the angle value
    pub fn as_angle(self) -> Angle {
        self.anomaly
    }

    /// Returns the raw f64 value
    pub fn as_f64(self) -> f64 {
        self.anomaly.as_f64()
    }
}

impl<K> Add for Anomaly<K>
where
    K: AnomalyKind,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Anomaly {
            anomaly: self.anomaly + rhs.anomaly,
            _kind: PhantomData,
        }
    }
}

impl<K> Sub for Anomaly<K>
where
    K: AnomalyKind,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Anomaly {
            anomaly: self.anomaly - rhs.anomaly,
            _kind: PhantomData,
        }
    }
}

impl<K> Neg for Anomaly<K>
where
    K: AnomalyKind,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Anomaly {
            anomaly: -self.anomaly,
            _kind: PhantomData,
        }
    }
}

/// True anomaly (ν): angle from periapsis to the position vector
pub type TrueAnomaly = Anomaly<TrueKind>;

/// Eccentric anomaly: E for elliptic, F for hyperbolic, D for parabolic
pub type EccentricAnomaly = Anomaly<EccentricKind>;

/// Mean anomaly: M for elliptic/circular, M_h for hyperbolic, M_p for parabolic
pub type MeanAnomaly = Anomaly<MeanKind>;

// ============================================================================
// CONVERSION IMPLEMENTATIONS
// ============================================================================

impl TrueAnomaly {
    /// Creates a new anomaly
    pub fn new(anomaly: Angle) -> Self {
        Self {
            anomaly,
            _kind: PhantomData,
        }
    }

    /// Converts true anomaly to eccentric anomaly
    ///
    /// Can fail for hyperbolic orbits if true anomaly is outside asymptote limits
    pub fn to_eccentric(self, ecc: Eccentricity) -> Result<EccentricAnomaly> {
        let orbit = ecc.orbit_type();
        match orbit {
            OrbitType::Circular | OrbitType::Elliptic => {
                Ok(EccentricAnomaly::new(true_to_eccentric(self.anomaly, ecc)))
            }
            OrbitType::Hyperbolic => Ok(EccentricAnomaly::new(true_to_hyperbolic(
                self.anomaly,
                ecc,
            )?)),
            OrbitType::Parabolic => {
                // For parabolic orbits, Eccentric represents parabolic anomaly D
                Ok(EccentricAnomaly::new(true_to_parabolic(self.anomaly).rad()))
            }
        }
    }

    /// Converts true anomaly to mean anomaly
    ///
    /// Can fail for hyperbolic orbits if true anomaly is outside asymptote limits
    pub fn to_mean(self, ecc: Eccentricity) -> Result<MeanAnomaly> {
        let orbit = ecc.orbit_type();
        match orbit {
            OrbitType::Circular => {
                // For circular orbits, mean anomaly equals true anomaly
                Ok(MeanAnomaly::new(self.anomaly))
            }
            OrbitType::Elliptic => Ok(MeanAnomaly::new(true_to_mean(self.anomaly, ecc))),
            OrbitType::Parabolic => {
                // For parabolic orbits, store parabolic mean anomaly M_p as angle
                Ok(MeanAnomaly::new(true_to_mean_parabolic(self.anomaly).rad()))
            }
            OrbitType::Hyperbolic => Ok(MeanAnomaly::new(true_to_mean_hyperbolic(
                self.anomaly,
                ecc,
            )?)),
        }
    }
}

impl EccentricAnomaly {
    /// Creates a new anomaly
    pub fn new(anomaly: Angle) -> Self {
        Self {
            anomaly,
            _kind: PhantomData,
        }
    }

    /// Converts eccentric anomaly to true anomaly.
    pub fn to_true(self, ecc: Eccentricity) -> TrueAnomaly {
        let orbit = ecc.orbit_type();
        match orbit {
            OrbitType::Circular | OrbitType::Elliptic => {
                TrueAnomaly::new(eccentric_to_true(self.anomaly, ecc))
            }
            OrbitType::Hyperbolic => TrueAnomaly::new(hyperbolic_to_true(self.anomaly, ecc)),
            OrbitType::Parabolic => {
                // For parabolic orbits, Eccentric represents parabolic anomaly D
                TrueAnomaly::new(parabolic_to_true(self.anomaly.as_f64()))
            }
        }
    }

    /// Converts eccentric anomaly to mean anomaly.
    pub fn to_mean(self, ecc: Eccentricity) -> MeanAnomaly {
        let orbit = ecc.orbit_type();
        match orbit {
            OrbitType::Circular | OrbitType::Elliptic => {
                MeanAnomaly::new(eccentric_to_mean(self.anomaly, ecc))
            }
            OrbitType::Hyperbolic => MeanAnomaly::new(hyperbolic_to_mean(self.anomaly, ecc)),
            OrbitType::Parabolic => {
                // For parabolic orbits, Eccentric represents parabolic anomaly D
                MeanAnomaly::new(parabolic_to_mean(self.anomaly.as_f64()).rad())
            }
        }
    }
}

impl MeanAnomaly {
    /// Creates a new anomaly
    pub fn new(anomaly: Angle) -> Self {
        Self {
            anomaly,
            _kind: PhantomData,
        }
    }

    /// Converts mean anomaly to true anomaly
    ///
    /// Can fail due to convergence issues in iterative solvers
    pub fn to_true(self, ecc: Eccentricity) -> Result<TrueAnomaly> {
        let orbit = ecc.orbit_type();
        match orbit {
            OrbitType::Circular => {
                // For circular orbits, mean anomaly equals true anomaly
                Ok(TrueAnomaly::new(self.anomaly))
            }
            OrbitType::Elliptic => Ok(TrueAnomaly::new(mean_to_true(self.anomaly, ecc)?)),
            OrbitType::Parabolic => {
                // For parabolic orbits, value stores M_p
                Ok(TrueAnomaly::new(mean_parabolic_to_true(
                    self.anomaly.as_f64(),
                )))
            }
            OrbitType::Hyperbolic => Ok(TrueAnomaly::new(mean_hyperbolic_to_true(
                self.anomaly,
                ecc,
            )?)),
        }
    }

    /// Converts mean anomaly to eccentric anomaly
    ///
    /// Can fail due to convergence issues in iterative solvers
    pub fn to_eccentric(self, ecc: Eccentricity) -> Result<EccentricAnomaly> {
        let orbit = ecc.orbit_type();
        match orbit {
            OrbitType::Circular | OrbitType::Elliptic => Ok(EccentricAnomaly::new(
                mean_to_eccentric(self.anomaly, ecc, None, None)?,
            )),
            OrbitType::Hyperbolic => Ok(EccentricAnomaly::new(mean_to_hyperbolic(
                self.anomaly,
                ecc,
                None,
                None,
            )?)),
            OrbitType::Parabolic => {
                // For parabolic orbits, Eccentric represents parabolic anomaly D
                Ok(EccentricAnomaly::new(
                    mean_to_parabolic(self.anomaly.as_f64()).rad(),
                ))
            }
        }
    }
}

impl<K: AnomalyKind> ApproxEq for Anomaly<K> {
    fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> ApproxEqResults {
        self.anomaly.approx_eq(&rhs.anomaly, atol, rtol)
    }
}

/// Error types for anomaly conversions
#[derive(Error, Debug, Clone, PartialEq)]
pub enum AnomalyError {
    #[error("failed to converge after {iterations} iterations (residual: {residual})")]
    ConvergenceFailure { iterations: usize, residual: f64 },
    #[error("True anomaly {nu} rad outside valid range [-{max_nu} rad, {max_nu} rad]")]
    InvalidTrueAnomaly { nu: Angle, max_nu: Angle },
}

pub type Result<T> = std::result::Result<T, AnomalyError>;

// ============================================================================
// ELLIPTIC ORBIT CONVERSIONS (0 < e < 1)
// ============================================================================

/// Convert true anomaly to eccentric anomaly for elliptic orbits.
///
/// Uses the half-angle formula: tan(E/2) = sqrt((1-e)/(1+e)) * tan(ν/2)
///
/// # Arguments
/// * `nu` - True anomaly
/// * `e` - Eccentricity (should be elliptic: 0 < e < 1)
///
/// # Returns
/// Eccentric anomaly, normalized to (-π, π]
pub fn true_to_eccentric(nu: Angle, e: Eccentricity) -> Angle {
    let ecc = e.as_f64();
    // Half-angle formula (most numerically stable)
    let factor = ((1.0_f64 - ecc) / (1.0_f64 + ecc)).sqrt();
    let ecc_half = Angle::from_atan(factor * (nu.as_f64() / 2.0).tan());
    let ecc_angle = Angle::new(2.0 * ecc_half.as_f64());

    // Normalize to (-π, π]
    ecc_angle.normalize_two_pi(Angle::ZERO)
}

/// Convert eccentric anomaly to true anomaly for elliptic orbits.
///
/// Uses the half-angle formula: tan(ν/2) = sqrt((1+e)/(1-e)) * tan(E/2)
///
/// # Arguments
/// * `ecc` - Eccentric anomaly
/// * `e` - Eccentricity (should be elliptic: 0 < e < 1)
///
/// # Returns
/// True anomaly, normalized to (-π, π]
pub fn eccentric_to_true(ecc: Angle, e: Eccentricity) -> Angle {
    let ecc_val = e.as_f64();
    // Half-angle formula
    let factor = ((1.0_f64 + ecc_val) / (1.0_f64 - ecc_val)).sqrt();
    let nu_half = Angle::from_atan(factor * (ecc.as_f64() / 2.0).tan());
    let nu = Angle::new(2.0 * nu_half.as_f64());

    // Normalize to (-π, π]
    nu.normalize_two_pi(Angle::ZERO)
}

/// Convert eccentric anomaly to mean anomaly for elliptic orbits.
///
/// Uses Kepler's equation: M = E - e*sin(E)
///
/// # Arguments
/// * `ecc` - Eccentric anomaly
/// * `e` - Eccentricity (should be elliptic: 0 < e < 1)
///
/// # Returns
/// Mean anomaly, normalized to (-π, π]
pub fn eccentric_to_mean(ecc: Angle, e: Eccentricity) -> Angle {
    let mean = Angle::new(ecc.as_f64() - e.as_f64() * ecc.sin());
    mean.normalize_two_pi(Angle::ZERO)
}

/// Convert mean anomaly to eccentric anomaly for elliptic orbits.
///
/// Solves Kepler's equation: M = E - e*sin(E) iteratively using Newton-Raphson method.
///
/// # Arguments
/// * `mean` - Mean anomaly
/// * `e` - Eccentricity (should be elliptic: 0 < e < 1)
/// * `tolerance` - Convergence tolerance (default: 1e-10)
/// * `max_iter` - Maximum iterations (default: 50)
///
/// # Returns
/// Eccentric anomaly
pub fn mean_to_eccentric(
    mean: Angle,
    e: Eccentricity,
    tolerance: Option<f64>,
    max_iter: Option<usize>,
) -> Result<Angle> {
    let tol = tolerance.unwrap_or(1e-10);
    let max_iterations = max_iter.unwrap_or(50);
    let ecc = e.as_f64();

    // Normalize mean anomaly to (-π, π]
    let m = mean.normalize_two_pi(Angle::ZERO).as_f64();

    // Initial guess (NASA approach)
    let mut ecc_anomaly = if ecc < 0.8 {
        m
    } else if m < Angle::PI.as_f64() {
        m + ecc / 2.0
    } else {
        m - ecc / 2.0
    };

    // Newton-Raphson iteration with second-order correction
    for _iteration in 0..max_iterations {
        let sin_e = ecc_anomaly.sin();
        let cos_e = ecc_anomaly.cos();

        // Function: f(E) = E - e*sin(E) - M
        let f = ecc_anomaly - ecc * sin_e - m;

        // First derivative: f'(E) = 1 - e*cos(E)
        let df = 1.0 - ecc * cos_e;

        // Second-order correction (NASA algorithm)
        let d_prime = df + 0.5 * f * ecc * sin_e / df;

        let delta = f / d_prime;
        ecc_anomaly -= delta;

        if delta.abs() < tol {
            return Ok(Angle::new(ecc_anomaly).normalize_two_pi(Angle::ZERO));
        }
    }

    Err(AnomalyError::ConvergenceFailure {
        iterations: max_iterations,
        residual: (ecc_anomaly - ecc * ecc_anomaly.sin() - m).abs(),
    })
}

/// Convert true anomaly to mean anomaly for elliptic orbits.
///
/// This is a convenience function that chains true_to_eccentric and eccentric_to_mean.
pub fn true_to_mean(nu: Angle, e: Eccentricity) -> Angle {
    let ecc = true_to_eccentric(nu, e);
    eccentric_to_mean(ecc, e)
}

/// Convert mean anomaly to true anomaly for elliptic orbits.
///
/// This is a convenience function that chains mean_to_eccentric and eccentric_to_true.
pub fn mean_to_true(mean: Angle, e: Eccentricity) -> Result<Angle> {
    let ecc = mean_to_eccentric(mean, e, None, None)?;
    Ok(eccentric_to_true(ecc, e))
}

// ============================================================================
// PARABOLIC ORBIT CONVERSIONS (e = 1)
// ============================================================================

/// Convert true anomaly to parabolic anomaly.
///
/// Parabolic anomaly D = tan(ν/2)
///
/// # Arguments
/// * `nu` - True anomaly
///
/// # Returns
/// Parabolic anomaly (dimensionless, unbounded)
pub fn true_to_parabolic(nu: Angle) -> f64 {
    (nu.as_f64() / 2.0).tan()
}

/// Convert parabolic anomaly to true anomaly.
///
/// True anomaly ν = 2*arctan(D)
///
/// # Arguments
/// * `parabolic` - Parabolic anomaly (dimensionless)
///
/// # Returns
/// True anomaly, in range (-π, π)
pub fn parabolic_to_true(parabolic: f64) -> Angle {
    Angle::new(2.0 * parabolic.atan())
}

/// Convert parabolic anomaly to parabolic mean anomaly.
///
/// Barker's equation: M_p = D + D³/3
///
/// # Arguments
/// * `parabolic` - Parabolic anomaly (dimensionless)
///
/// # Returns
/// Parabolic mean anomaly
pub fn parabolic_to_mean(parabolic: f64) -> f64 {
    parabolic + parabolic.powi(3) / 3.0
}

/// Convert parabolic mean anomaly to parabolic anomaly.
///
/// Solves Barker's equation: M_p = D + D³/3 using the analytical cubic solution.
///
/// # Arguments
/// * `mean_parabolic` - Parabolic mean anomaly
///
/// # Returns
/// Parabolic anomaly
pub fn mean_to_parabolic(mean_parabolic: f64) -> f64 {
    // Analytical solution using Cardano's method
    // Solve: D³ + 3D - 3M_p = 0
    // Let A = 3M_p/2
    let a = 1.5 * mean_parabolic;

    // z = cbrt(A + sqrt(A² + 1))
    let discriminant = a * a + 1.0;
    let z = (a + discriminant.sqrt()).cbrt();

    // D = z - 1/z
    z - 1.0 / z
}

/// Convert true anomaly to parabolic mean anomaly.
pub fn true_to_mean_parabolic(nu: Angle) -> f64 {
    let d = true_to_parabolic(nu);
    parabolic_to_mean(d)
}

/// Convert parabolic mean anomaly to true anomaly.
pub fn mean_parabolic_to_true(mean_parabolic: f64) -> Angle {
    let d = mean_to_parabolic(mean_parabolic);
    parabolic_to_true(d)
}

// ============================================================================
// HYPERBOLIC ORBIT CONVERSIONS (e > 1)
// ============================================================================

/// Convert true anomaly to hyperbolic eccentric anomaly.
///
/// Uses the half-angle formula: tanh(F/2) = sqrt((e-1)/(e+1)) * tan(ν/2)
///
/// # Arguments
/// * `nu` - True anomaly (should be within asymptote limits)
/// * `e` - Eccentricity (should be hyperbolic: e > 1)
///
/// # Returns
/// Hyperbolic eccentric anomaly (unbounded)
///
/// # Errors
/// Returns `InvalidTrueAnomaly` if true anomaly is outside asymptote limits
pub fn true_to_hyperbolic(nu: Angle, e: Eccentricity) -> Result<Angle> {
    let ecc = e.as_f64();
    // Check if true anomaly is within asymptote limits
    let nu_max = Angle::from_acos(-1.0 / ecc);
    if nu.as_f64().abs() >= nu_max.as_f64() {
        return Err(AnomalyError::InvalidTrueAnomaly { nu, max_nu: nu_max });
    }

    // Half-angle formula
    let factor = ((ecc - 1.0_f64) / (ecc + 1.0_f64)).sqrt();
    let tanh_f_half = factor * (nu.as_f64() / 2.0).tan();

    // F = 2 * atanh(tanh(F/2))
    let f_half = tanh_f_half.atanh();
    Ok(Angle::new(2.0 * f_half))
}

/// Convert hyperbolic eccentric anomaly to true anomaly.
///
/// Uses the half-angle formula: tan(ν/2) = sqrt((e+1)/(e-1)) * tanh(F/2)
///
/// # Arguments
/// * `hyperbolic` - Hyperbolic eccentric anomaly
/// * `e` - Eccentricity (should be hyperbolic: e > 1)
///
/// # Returns
/// True anomaly
pub fn hyperbolic_to_true(hyperbolic: Angle, e: Eccentricity) -> Angle {
    let hyperbolic = hyperbolic.as_f64();
    let ecc = e.as_f64();
    // Half-angle formula
    let factor = ((ecc + 1.0) / (ecc - 1.0)).sqrt();
    let tan_nu_half = factor * (hyperbolic / 2.0).tanh();

    Angle::new(2.0 * tan_nu_half.atan())
}

/// Convert hyperbolic eccentric anomaly to hyperbolic mean anomaly.
///
/// M_h = e*sinh(F) - F
///
/// # Arguments
/// * `hyperbolic` - Hyperbolic eccentric anomaly
/// * `e` - Eccentricity (should be hyperbolic: e > 1)
///
/// # Returns
/// Hyperbolic mean anomaly
pub fn hyperbolic_to_mean(hyperbolic: Angle, e: Eccentricity) -> Angle {
    Angle::new(e.as_f64() * hyperbolic.sinh() - hyperbolic.as_f64())
}

/// Convert hyperbolic mean anomaly to hyperbolic eccentric anomaly.
///
/// Solves: M_h = e*sinh(F) - F iteratively using Newton-Raphson method.
///
/// # Arguments
/// * `mean_hyperbolic` - Hyperbolic mean anomaly
/// * `e` - Eccentricity (should be hyperbolic: e > 1)
/// * `tolerance` - Convergence tolerance (default: 1e-10)
/// * `max_iter` - Maximum iterations (default: 50)
///
/// # Returns
/// Hyperbolic eccentric anomaly
pub fn mean_to_hyperbolic(
    mean_hyperbolic: Angle,
    e: Eccentricity,
    tolerance: Option<f64>,
    max_iter: Option<usize>,
) -> Result<Angle> {
    let tol = tolerance.unwrap_or(1e-10);
    let max_iterations = max_iter.unwrap_or(50);
    let ecc = e.as_f64();
    let mean_hyperbolic = mean_hyperbolic.as_f64();

    // Initial guess using domain-informed approximation
    // arcsinh(M/e) provides excellent starting point for hyperbolic case
    let mut f = (mean_hyperbolic / ecc).asinh();

    // Newton-Raphson iteration
    for _iteration in 0..max_iterations {
        let sinh_f = f.sinh();
        let cosh_f = f.cosh();

        // Function: g(F) = e*sinh(F) - F - M_h
        let g = ecc * sinh_f - f - mean_hyperbolic;

        // Derivative: g'(F) = e*cosh(F) - 1
        let dg = ecc * cosh_f - 1.0;

        let delta = g / dg;
        f -= delta;

        if delta.abs() < tol {
            return Ok(f.rad());
        }
    }

    Err(AnomalyError::ConvergenceFailure {
        iterations: max_iterations,
        residual: (ecc * f.sinh() - f - mean_hyperbolic).abs(),
    })
}

/// Convert true anomaly to hyperbolic mean anomaly.
pub fn true_to_mean_hyperbolic(nu: Angle, e: Eccentricity) -> Result<Angle> {
    let f = true_to_hyperbolic(nu, e)?;
    Ok(hyperbolic_to_mean(f, e))
}

/// Convert hyperbolic mean anomaly to true anomaly.
pub fn mean_hyperbolic_to_true(mean_hyperbolic: Angle, e: Eccentricity) -> Result<Angle> {
    let f = mean_to_hyperbolic(mean_hyperbolic, e, None, None)?;
    Ok(hyperbolic_to_true(f, e))
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Get the maximum true anomaly for a hyperbolic orbit (asymptote angle)
///
/// # Arguments
/// * `e` - Eccentricity (should be hyperbolic: e > 1)
///
/// # Returns
/// Asymptote angle
pub fn hyperbolic_asymptote_angle(e: Eccentricity) -> Angle {
    Angle::from_acos(-1.0 / e.as_f64())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-9;

    #[test]
    fn test_elliptic_round_trip() {
        let e = Eccentricity::try_new(0.5).unwrap();
        let nu = Angle::new(1.0);

        let ecc = true_to_eccentric(nu, e);
        let nu_back = eccentric_to_true(ecc, e);

        assert!((nu.as_f64() - nu_back.as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_keplers_equation() {
        let e = Eccentricity::try_new(0.3).unwrap();
        let ecc = Angle::new(1.5);

        let mean = eccentric_to_mean(ecc, e);
        let ecc_back = mean_to_eccentric(mean, e, None, None).unwrap();

        assert!((ecc.as_f64() - ecc_back.as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_parabolic_round_trip() {
        let nu = Angle::new(1.0);

        let d = true_to_parabolic(nu);
        let nu_back = parabolic_to_true(d);

        assert!((nu.as_f64() - nu_back.as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_barkers_equation() {
        let d = 0.5;

        let mp = parabolic_to_mean(d);
        let d_back = mean_to_parabolic(mp);

        assert!((d - d_back).abs() < EPSILON);
    }

    #[test]
    fn test_hyperbolic_round_trip() {
        let e = Eccentricity::try_new(1.5).unwrap();
        let nu = Angle::new(0.5);

        let f = true_to_hyperbolic(nu, e).unwrap();
        let nu_back = hyperbolic_to_true(f, e);

        assert!((nu.as_f64() - nu_back.as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_hyperbolic_keplers_equation() {
        let e = Eccentricity::try_new(2.0).unwrap();
        let f = 0.8.rad();

        let mh = hyperbolic_to_mean(f, e);
        let f_back = mean_to_hyperbolic(mh, e, None, None).unwrap();

        assert!((f - f_back).abs().as_f64() < EPSILON);
    }

    #[test]
    fn test_periapsis() {
        let e = Eccentricity::try_new(0.7).unwrap();
        let nu = Angle::ZERO;

        let ecc = true_to_eccentric(nu, e);
        assert!(ecc.as_f64().abs() < EPSILON);

        let mean = eccentric_to_mean(ecc, e);
        assert!(mean.as_f64().abs() < EPSILON);
    }

    #[test]
    fn test_apoapsis() {
        let e = Eccentricity::try_new(0.6).unwrap();
        let nu = Angle::PI;

        let ecc = true_to_eccentric(nu, e);
        // At apoapsis, eccentric anomaly should also be π (or -π, which is equivalent)
        // The normalize_two_pi function normalizes to (-π, π], so π stays as π, but -π is also valid
        let diff = (ecc.as_f64().abs() - Angle::PI.as_f64()).abs();
        assert!(diff < EPSILON, "Expected π or -π, got {}", ecc.as_f64());
    }
}

#[cfg(test)]
mod anomaly_tests {
    use super::*;

    const EPSILON: f64 = 1e-9;

    // ========================================================================
    // ELLIPTIC ORBIT TESTS
    // ========================================================================

    #[test]
    fn test_anomaly_elliptic_true_to_eccentric() {
        let e = Eccentricity::try_new(0.5).unwrap();
        let nu = Angle::new(1.0);

        let anomaly = TrueAnomaly::new(nu);
        let eccentric = anomaly.to_eccentric(e).unwrap();

        let true_back = eccentric.to_true(e);
        assert!((nu.as_f64() - true_back.as_angle().as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_anomaly_elliptic_true_to_mean() {
        let e = Eccentricity::try_new(0.3).unwrap();
        let nu = Angle::new(1.5);

        let anomaly = TrueAnomaly::new(nu);
        let mean = anomaly.to_mean(e).unwrap();

        // Convert back to verify
        let true_back = mean.to_true(e).unwrap();
        assert!((nu.as_f64() - true_back.as_angle().as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_anomaly_elliptic_eccentric_to_mean() {
        let e = Eccentricity::try_new(0.4).unwrap();
        let ecc = Angle::new(0.8);

        let anomaly = EccentricAnomaly::new(ecc);
        let mean = anomaly.to_mean(e);

        // Convert back to verify
        let ecc_back = mean.to_eccentric(e).unwrap();
        assert!((ecc.as_f64() - ecc_back.as_angle().as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_anomaly_elliptic_mean_to_true() {
        let e = Eccentricity::try_new(0.6).unwrap();
        let mean = Angle::new(2.0);

        let anomaly = MeanAnomaly::new(mean);
        let true_anom = anomaly.to_true(e).unwrap();

        // Convert back to verify
        let mean_back = true_anom.to_mean(e).unwrap();
        assert!((mean.as_f64() - mean_back.as_angle().as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_anomaly_elliptic_mean_to_eccentric() {
        let e = Eccentricity::try_new(0.5).unwrap();
        let mean = Angle::new(1.2);

        let anomaly = MeanAnomaly::new(mean);
        let ecc = anomaly.to_eccentric(e).unwrap();

        // Convert back to verify
        let mean_back = ecc.to_mean(e);
        assert!((mean.as_f64() - mean_back.as_angle().as_f64()).abs() < EPSILON);
    }

    // ========================================================================
    // CIRCULAR ORBIT TESTS
    // ========================================================================

    #[test]
    fn test_anomaly_circular_mean_equals_true() {
        let e = Eccentricity::try_new(0.0).unwrap();
        let angle = Angle::new(1.5);

        let true_anom = TrueAnomaly::new(angle);
        let mean = true_anom.to_mean(e).unwrap();

        // For circular orbits, mean anomaly should equal true anomaly
        assert!((angle.as_f64() - mean.as_angle().as_f64()).abs() < EPSILON);

        // And converting back should give the same value
        let true_back = mean.to_true(e).unwrap();
        assert!((angle.as_f64() - true_back.as_angle().as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_anomaly_circular_eccentric_conversion() {
        let e = Eccentricity::try_new(1e-10).unwrap(); // Nearly circular
        let nu = Angle::new(0.5);

        let anomaly = TrueAnomaly::new(nu);
        let ecc = anomaly.to_eccentric(e).unwrap();
        let true_back = ecc.to_true(e);

        assert!((nu.as_f64() - true_back.as_angle().as_f64()).abs() < EPSILON);
    }

    // ========================================================================
    // HYPERBOLIC ORBIT TESTS
    // ========================================================================

    #[test]
    fn test_anomaly_hyperbolic_true_to_eccentric() {
        let e = Eccentricity::try_new(1.5).unwrap();
        let nu = Angle::new(0.5); // Within asymptote limits

        let anomaly = TrueAnomaly::new(nu);
        let eccentric = anomaly.to_eccentric(e).unwrap();

        // Convert back to verify
        let true_back = eccentric.to_true(e);
        assert!((nu.as_f64() - true_back.as_angle().as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_anomaly_hyperbolic_true_to_mean() {
        let e = Eccentricity::try_new(2.0).unwrap();
        let nu = Angle::new(0.8);

        let anomaly = TrueAnomaly::new(nu);
        let mean = anomaly.to_mean(e).unwrap();

        // Convert back to verify
        let true_back = mean.to_true(e).unwrap();
        assert!((nu.as_f64() - true_back.as_angle().as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_anomaly_hyperbolic_eccentric_to_mean() {
        let e = Eccentricity::try_new(1.8).unwrap();
        let f = Angle::new(0.5);

        let anomaly = EccentricAnomaly::new(f);
        let mean = anomaly.to_mean(e);

        // Convert back to verify
        let ecc_back = mean.to_eccentric(e).unwrap();
        assert!((f.as_f64() - ecc_back.as_angle().as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_anomaly_hyperbolic_mean_to_true() {
        let e = Eccentricity::try_new(1.5).unwrap();
        let mean = Angle::new(1.0);

        let anomaly = MeanAnomaly::new(mean);
        let true_anom = anomaly.to_true(e).unwrap();

        // Convert back to verify
        let mean_back = true_anom.to_mean(e).unwrap();
        assert!((mean.as_f64() - mean_back.as_angle().as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_anomaly_hyperbolic_asymptote_error() {
        let e = Eccentricity::try_new(1.5).unwrap();
        let nu_max = hyperbolic_asymptote_angle(e);

        // Try to convert true anomaly beyond asymptote limit
        let nu_invalid = Angle::new(nu_max.as_f64() + 0.1);
        let anomaly = TrueAnomaly::new(nu_invalid);

        let result = anomaly.to_eccentric(e);
        assert!(result.is_err());
        match result {
            Err(AnomalyError::InvalidTrueAnomaly { .. }) => (),
            _ => panic!("Expected InvalidTrueAnomaly error"),
        }
    }

    // ========================================================================
    // PARABOLIC ORBIT TESTS
    // ========================================================================

    #[test]
    fn test_anomaly_parabolic_true_to_mean() {
        let e = Eccentricity::try_new(1.0).unwrap();
        let nu = Angle::new(1.0);

        let anomaly = TrueAnomaly::new(nu);
        let mean = anomaly.to_mean(e).unwrap();

        // Convert back to verify
        let true_back = mean.to_true(e).unwrap();
        assert!((nu.as_f64() - true_back.as_angle().as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_anomaly_parabolic_mean_to_true() {
        let e = Eccentricity::try_new(1.0).unwrap();
        let mean = Angle::new(0.5);

        let anomaly = MeanAnomaly::new(mean);
        let true_anom = anomaly.to_true(e).unwrap();

        // Convert back to verify
        let mean_back = true_anom.to_mean(e).unwrap();
        assert!((mean.as_f64() - mean_back.as_angle().as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_anomaly_parabolic_eccentric_round_trip() {
        let e = Eccentricity::try_new(1.0).unwrap();
        let d = 0.5; // Parabolic anomaly D

        // For parabolic orbits, Eccentric stores parabolic anomaly D
        let anomaly = EccentricAnomaly::new(d.rad());

        // Convert to true anomaly
        let true_anom = anomaly.to_true(e);

        // Convert back to eccentric (parabolic anomaly D)
        let ecc_back = true_anom.to_eccentric(e).unwrap();

        assert!((d - ecc_back.as_angle().as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_anomaly_parabolic_eccentric_to_mean() {
        let e = Eccentricity::try_new(1.0).unwrap();
        let d = 0.5; // Parabolic anomaly D

        // For parabolic orbits, Eccentric stores parabolic anomaly D
        let anomaly = EccentricAnomaly::new(d.rad());

        // Convert to mean anomaly
        let mean = anomaly.to_mean(e);

        // Convert back to eccentric (parabolic anomaly D)
        let ecc_back = mean.to_eccentric(e).unwrap();

        assert!((d - ecc_back.as_angle().as_f64()).abs() < EPSILON);
    }

    // ========================================================================
    // ROUND-TRIP TESTS
    // ========================================================================

    #[test]
    fn test_anomaly_elliptic_full_round_trip() {
        let e = Eccentricity::try_new(0.5).unwrap();
        let nu = Angle::new(1.2);

        // True -> Eccentric -> Mean -> Eccentric -> True
        let true_anom = TrueAnomaly::new(nu);
        let ecc = true_anom.to_eccentric(e).unwrap();
        let mean = ecc.to_mean(e);
        let ecc2 = mean.to_eccentric(e).unwrap();
        let true_back = ecc2.to_true(e);

        assert!((nu.as_f64() - true_back.as_angle().as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_anomaly_hyperbolic_full_round_trip() {
        let e = Eccentricity::try_new(2.0).unwrap();
        let nu = Angle::new(0.5);

        // True -> Eccentric -> Mean -> Eccentric -> True
        let true_anom = TrueAnomaly::new(nu);
        let ecc = true_anom.to_eccentric(e).unwrap();
        let mean = ecc.to_mean(e);
        let ecc2 = mean.to_eccentric(e).unwrap();
        let true_back = ecc2.to_true(e);

        assert!((nu.as_f64() - true_back.as_angle().as_f64()).abs() < EPSILON);
    }
}
