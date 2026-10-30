// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Orbital Anomaly Conversions
//!
//! This module provides conversions between true anomaly, eccentric anomaly, and mean anomaly
//! for all conic section orbit types: circular, elliptic, parabolic, and hyperbolic.
//!
//! # Conventions
//! - All angles use the [`Angle`] type
//! - Conversions do not normalize: the result stays in the same revolution as the input, so
//!   each conversion is continuous and monotonic across revolutions. Use
//!   [`Angle::normalize_two_pi`] to reduce an anomaly to a single revolution
//! - Eccentricity uses the [`Eccentricity`] type which validates e >= 0
//! - Orbit types: circular (e≈0), elliptic (0<e<1), parabolic (e≈1), hyperbolic (e>1)
//! - True anomaly (ν): angle from periapsis to the position vector
//! - Eccentric anomaly (E): auxiliary angle for elliptic orbits
//! - Hyperbolic anomaly (F): hyperbolic angle for hyperbolic orbits (unbounded)
//! - Parabolic anomaly (D): tan(ν/2) for parabolic orbits (unbounded)
//! - Mean anomaly (M): time-like parameter proportional to time since periapsis
//!
//! # Eccentric Anomaly Convention
//! [`EccentricAnomaly`] uses an overloaded convention:
//! - For elliptic/circular orbits: represents eccentric anomaly E
//! - For hyperbolic orbits: represents hyperbolic anomaly F
//! - For parabolic orbits: represents parabolic anomaly D (stored as radians)

use lox_approx::ApproxEq;
use thiserror::Error;

use crate::math::float::{
    abs, atan, atanh, cbrt, cos, cosh, powi, round, sin, sinh, sqrt, tan, tanh,
};
use crate::math::roots::{BracketedHalley, FindBracketedRootWithSecondDerivative};
use crate::{
    elements::{Eccentricity, OrbitType},
    units::{Angle, AngleUnits},
};

use core::{
    f64::consts::{PI, TAU},
    fmt::Display,
    ops::{Add, Neg, Sub},
};

/// True anomaly (ν): angle from periapsis to the position vector
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct TrueAnomaly(#[approx(periodic = TAU)] Angle);

/// Eccentric anomaly: E for elliptic, F for hyperbolic, D for parabolic
///
/// Unlike [`TrueAnomaly`], approximate comparisons do not wrap around 2π, because the value is
/// unbounded for parabolic and hyperbolic orbits.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct EccentricAnomaly(Angle);

/// Mean anomaly: M for elliptic/circular, M_h for hyperbolic, M_p for parabolic
///
/// Unlike [`TrueAnomaly`], approximate comparisons do not wrap around 2π, because the value is
/// unbounded for parabolic and hyperbolic orbits.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct MeanAnomaly(Angle);

macro_rules! anomaly_impls {
    ($($anomaly:ident),*) => {
        $(
            impl $anomaly {
                /// Creates a new anomaly
                pub fn new(anomaly: Angle) -> Self {
                    Self(anomaly)
                }

                /// Returns the angle value
                pub fn as_angle(self) -> Angle {
                    self.0
                }

                /// Returns the raw f64 value
                pub fn as_f64(self) -> f64 {
                    self.0.as_f64()
                }
            }

            impl Display for $anomaly {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    self.0.fmt(f)
                }
            }

            impl Add for $anomaly {
                type Output = Self;

                fn add(self, rhs: Self) -> Self::Output {
                    Self(self.0 + rhs.0)
                }
            }

            impl Sub for $anomaly {
                type Output = Self;

                fn sub(self, rhs: Self) -> Self::Output {
                    Self(self.0 - rhs.0)
                }
            }

            impl Neg for $anomaly {
                type Output = Self;

                fn neg(self) -> Self::Output {
                    Self(-self.0)
                }
            }
        )*
    };
}

anomaly_impls!(TrueAnomaly, EccentricAnomaly, MeanAnomaly);

impl TrueAnomaly {
    /// Converts true anomaly to eccentric anomaly
    ///
    /// Can fail for hyperbolic orbits if true anomaly is outside asymptote limits
    pub fn to_eccentric(self, ecc: Eccentricity) -> Result<EccentricAnomaly> {
        let orbit = ecc.orbit_type();
        match orbit {
            OrbitType::Circular | OrbitType::Elliptic => {
                Ok(EccentricAnomaly::new(true_to_eccentric(self.0, ecc)))
            }
            OrbitType::Hyperbolic => Ok(EccentricAnomaly::new(true_to_hyperbolic(self.0, ecc)?)),
            OrbitType::Parabolic => {
                // For parabolic orbits, Eccentric represents parabolic anomaly D
                Ok(EccentricAnomaly::new(true_to_parabolic(self.0).rad()))
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
                Ok(MeanAnomaly::new(self.0))
            }
            OrbitType::Elliptic => Ok(MeanAnomaly::new(true_to_mean(self.0, ecc))),
            OrbitType::Parabolic => {
                // For parabolic orbits, store parabolic mean anomaly M_p as angle
                Ok(MeanAnomaly::new(true_to_mean_parabolic(self.0).rad()))
            }
            OrbitType::Hyperbolic => Ok(MeanAnomaly::new(true_to_mean_hyperbolic(self.0, ecc)?)),
        }
    }
}

impl EccentricAnomaly {
    /// Converts eccentric anomaly to true anomaly.
    pub fn to_true(self, ecc: Eccentricity) -> TrueAnomaly {
        let orbit = ecc.orbit_type();
        match orbit {
            OrbitType::Circular | OrbitType::Elliptic => {
                TrueAnomaly::new(eccentric_to_true(self.0, ecc))
            }
            OrbitType::Hyperbolic => TrueAnomaly::new(hyperbolic_to_true(self.0, ecc)),
            OrbitType::Parabolic => {
                // For parabolic orbits, Eccentric represents parabolic anomaly D
                TrueAnomaly::new(parabolic_to_true(self.0.as_f64()))
            }
        }
    }

    /// Converts eccentric anomaly to mean anomaly.
    pub fn to_mean(self, ecc: Eccentricity) -> MeanAnomaly {
        let orbit = ecc.orbit_type();
        match orbit {
            OrbitType::Circular | OrbitType::Elliptic => {
                MeanAnomaly::new(eccentric_to_mean(self.0, ecc))
            }
            OrbitType::Hyperbolic => MeanAnomaly::new(hyperbolic_to_mean(self.0, ecc)),
            OrbitType::Parabolic => {
                // For parabolic orbits, Eccentric represents parabolic anomaly D
                MeanAnomaly::new(parabolic_to_mean(self.0.as_f64()).rad())
            }
        }
    }
}

impl MeanAnomaly {
    /// Converts mean anomaly to true anomaly.
    ///
    /// # Errors
    ///
    /// Returns an [`AnomalyError`] if Kepler's equation cannot be solved for the mean anomaly,
    /// because it is not finite or the solver for hyperbolic orbits does not converge.
    pub fn to_true(self, ecc: Eccentricity) -> Result<TrueAnomaly> {
        let orbit = ecc.orbit_type();
        match orbit {
            OrbitType::Circular => {
                // For circular orbits, mean anomaly equals true anomaly
                Ok(TrueAnomaly::new(self.0))
            }
            OrbitType::Elliptic => Ok(TrueAnomaly::new(mean_to_true(self.0, ecc)?)),
            OrbitType::Parabolic => {
                // For parabolic orbits, value stores M_p
                Ok(TrueAnomaly::new(mean_parabolic_to_true(self.0.as_f64())))
            }
            OrbitType::Hyperbolic => Ok(TrueAnomaly::new(mean_hyperbolic_to_true(self.0, ecc)?)),
        }
    }

    /// Converts mean anomaly to eccentric anomaly.
    ///
    /// # Errors
    ///
    /// Returns an [`AnomalyError`] if Kepler's equation cannot be solved for the mean anomaly,
    /// because it is not finite or the solver for hyperbolic orbits does not converge.
    pub fn to_eccentric(self, ecc: Eccentricity) -> Result<EccentricAnomaly> {
        let orbit = ecc.orbit_type();
        match orbit {
            OrbitType::Circular | OrbitType::Elliptic => {
                Ok(EccentricAnomaly::new(mean_to_eccentric(self.0, ecc)?))
            }
            OrbitType::Hyperbolic => Ok(EccentricAnomaly::new(mean_to_hyperbolic(
                self.0, ecc, None, None,
            )?)),
            OrbitType::Parabolic => {
                // For parabolic orbits, Eccentric represents parabolic anomaly D
                Ok(EccentricAnomaly::new(
                    mean_to_parabolic(self.0.as_f64()).rad(),
                ))
            }
        }
    }
}

/// Error types for anomaly conversions
#[derive(Error, Debug, Clone, PartialEq)]
pub enum AnomalyError {
    /// The iterative solver did not converge within the allowed iterations.
    #[error("failed to converge after {iterations} iterations (residual: {residual})")]
    ConvergenceFailure {
        /// Number of iterations attempted.
        iterations: usize,
        /// Final residual value.
        residual: f64,
    },
    /// The true anomaly is outside the valid range for the given eccentricity.
    #[error("True anomaly {nu} rad outside valid range [-{max_nu} rad, {max_nu} rad]")]
    InvalidTrueAnomaly {
        /// The invalid true anomaly.
        nu: Angle,
        /// The maximum valid true anomaly magnitude.
        max_nu: Angle,
    },
    /// The anomaly is not finite.
    #[error("anomaly must be finite but was {0}")]
    NonFiniteAnomaly(Angle),
    /// The orbit is not circular or elliptic.
    #[error("expected a circular or elliptic orbit but the eccentricity was {0}")]
    NotElliptic(Eccentricity),
}

/// A type alias for `Result<T, AnomalyError>`.
pub type Result<T> = core::result::Result<T, AnomalyError>;

/// Convert true anomaly to eccentric anomaly for elliptic orbits.
///
/// Uses the half-angle formula: tan(E/2) = sqrt((1-e)/(1+e)) * tan(ν/2)
///
/// # Arguments
/// * `nu` - True anomaly
/// * `e` - Eccentricity (should be elliptic: 0 < e < 1)
///
/// # Returns
/// Eccentric anomaly, in the same revolution as `nu`
pub fn true_to_eccentric(nu: Angle, e: Eccentricity) -> Angle {
    let ecc = e.as_f64();
    // Half-angle formula (most numerically stable)
    let factor = sqrt((1.0_f64 - ecc) / (1.0_f64 + ecc));
    let principal = 2.0 * atan(factor * tan(nu.as_f64() / 2.0));
    Angle::new(same_revolution(principal, nu.as_f64()))
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
/// True anomaly, in the same revolution as `ecc`
pub fn eccentric_to_true(ecc: Angle, e: Eccentricity) -> Angle {
    let ecc_val = e.as_f64();
    // Half-angle formula
    let factor = sqrt((1.0_f64 + ecc_val) / (1.0_f64 - ecc_val));
    let principal = 2.0 * atan(factor * tan(ecc.as_f64() / 2.0));
    Angle::new(same_revolution(principal, ecc.as_f64()))
}

/// Shifts `principal` by whole revolutions to lie within half a revolution of `reference`.
///
/// True and eccentric anomaly always differ by less than half a revolution, so this recovers
/// the revolution of the input from the principal value of the half-angle formula.
fn same_revolution(principal: f64, reference: f64) -> f64 {
    principal + TAU * round((reference - principal) / TAU)
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
/// Mean anomaly, in the same revolution as `ecc`
pub fn eccentric_to_mean(ecc: Angle, e: Eccentricity) -> Angle {
    Angle::new(ecc.as_f64() - e.as_f64() * ecc.sin())
}

/// Convert mean anomaly to eccentric anomaly for elliptic orbits.
///
/// # Arguments
/// * `mean` - Mean anomaly
/// * `e` - Eccentricity of a circular or elliptic orbit
///
/// # Returns
/// Eccentric anomaly, in the same revolution as `mean`
///
/// # Errors
///
/// Returns an [`AnomalyError`] if `mean` is not finite or the orbit is not circular or elliptic.
pub fn mean_to_eccentric(mean: Angle, e: Eccentricity) -> Result<Angle> {
    if !mean.as_f64().is_finite() {
        return Err(AnomalyError::NonFiniteAnomaly(mean));
    }
    if !e.is_circular_or_elliptic() {
        return Err(AnomalyError::NotElliptic(e));
    }
    Ok(Angle::new(solve_kepler(mean.as_f64(), 0.0, e.as_f64())))
}

/// Solves the equinoctial form of Kepler's equation, λ = F - k·sin(F) + h·cos(F), for the
/// eccentric longitude F of an elliptic orbit with eccentricity vector (k, h), so h² + k² < 1.
///
/// The classical form M = E - e·sin(E) is the special case h = 0, k = e. The result lies in the
/// same revolution as λ.
pub(crate) fn solve_kepler(mean_longitude: f64, h: f64, k: f64) -> f64 {
    // Solve within a single revolution [-π, π) and add the whole revolutions back afterwards.
    let lambda = Angle::new(mean_longitude)
        .normalize_two_pi(Angle::ZERO)
        .as_f64();
    let revolutions = mean_longitude - lambda;

    // The residual increases monotonically and the solution lies within λ ± e. The bracket
    // λ ± π is wider, so the residual is at most e - π < 0 at its lower end and at least
    // π - e > 0 at its upper end, with a margin no rounding error can overcome. Knowing the signs,
    // the solver need not evaluate the ends. The tolerance exceeds the spacing of floats below 8,
    // so the iteration cannot stall between two neighbouring floats.
    let residual = |f: f64| {
        let (sin_f, cos_f) = (sin(f), cos(f));
        (
            f - k * sin_f + h * cos_f - lambda,
            1.0 - k * cos_f - h * sin_f,
            k * sin_f - h * cos_f,
        )
    };

    // For highly eccentric orbits, start half an eccentricity from λ towards the solution, on the
    // side given by the sign of e·sin(M) = k·sin(λ) - h·cos(λ). For h = 0, k·λ has the same sign
    // on [-π, π), which saves the trigonometry.
    let e = sqrt(h * h + k * k);
    let initial_guess = if e < 0.8 {
        lambda
    } else {
        let e_sin_mean = if h == 0.0 {
            k * lambda
        } else {
            k * sin(lambda) - h * cos(lambda)
        };
        if e_sin_mean >= 0.0 {
            lambda + e / 2.0
        } else {
            lambda - e / 2.0
        }
    };

    // The safeguard keeps every iterate inside the bracket and falls back to bisection, but its
    // worst case has no tight bound on the number of iterations. Kepler's equation converges in a
    // handful of iterations, so twice the default cap is ample headroom, not a proven limit.
    let f = BracketedHalley::default()
        .with_max_iter(200)
        .with_abs_tol(8.0 * f64::EPSILON)
        .with_rel_tol(0.0)
        .find_in_oriented_bracket_with_second_derivative_from(
            residual,
            (lambda - PI, lambda + PI),
            initial_guess,
        )
        .expect("the residual is finite, changes sign across the bracket and converges quickly");
    f + revolutions
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
///
/// # Errors
///
/// Returns an [`AnomalyError`] if `mean` is not finite or the orbit is not circular or elliptic.
pub fn mean_to_true(mean: Angle, e: Eccentricity) -> Result<Angle> {
    Ok(eccentric_to_true(mean_to_eccentric(mean, e)?, e))
}

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
    tan(nu.as_f64() / 2.0)
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
    Angle::new(2.0 * atan(parabolic))
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
    parabolic + powi(parabolic, 3) / 3.0
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
    let z = cbrt(a + sqrt(discriminant));

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
    if abs(nu.as_f64()) >= nu_max.as_f64() {
        return Err(AnomalyError::InvalidTrueAnomaly { nu, max_nu: nu_max });
    }

    // Half-angle formula
    let factor = sqrt((ecc - 1.0_f64) / (ecc + 1.0_f64));
    let tanh_f_half = factor * tan(nu.as_f64() / 2.0);

    // F = 2 * atanh(tanh(F/2))
    let f_half = atanh(tanh_f_half);
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
    let factor = sqrt((ecc + 1.0) / (ecc - 1.0));
    let tan_nu_half = factor * tanh(hyperbolic / 2.0);

    Angle::new(2.0 * atan(tan_nu_half))
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
    Angle::new(e.as_f64() * sinh(hyperbolic.as_f64()) - hyperbolic.as_f64())
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
    let mut f = crate::math::float::asinh(mean_hyperbolic / ecc);

    // Newton-Raphson iteration
    for _iteration in 0..max_iterations {
        let sinh_f = sinh(f);
        let cosh_f = cosh(f);

        // Function: g(F) = e*sinh(F) - F - M_h
        let g = ecc * sinh_f - f - mean_hyperbolic;

        // Derivative: g'(F) = e*cosh(F) - 1
        let dg = ecc * cosh_f - 1.0;

        let delta = g / dg;
        f -= delta;

        if abs(delta) < tol {
            return Ok(f.rad());
        }
    }

    Err(AnomalyError::ConvergenceFailure {
        iterations: max_iterations,
        residual: abs(ecc * sinh(f) - f - mean_hyperbolic),
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
    use core::f64::consts::PI;

    use lox_approx::{assert_approx_eq, assert_approx_ne};
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;

    const EPSILON: f64 = 1e-9;

    /// Eccentricities that `Eccentricity::orbit_type` classifies as circular or elliptic, with
    /// extra weight on the near-circular and near-parabolic ends. Eccentricities within 1e-8 of 1
    /// count as parabolic, so the upper bound stays below the bound for the longitudes.
    fn elliptic_eccentricity() -> impl Strategy<Value = f64> {
        prop_oneof![
            Just(0.0),
            0.0..1e-8,
            1e-8..0.9,
            0.9..0.9999,
            0.9999..1.0 - 2e-8,
        ]
    }

    proptest! {
        #[test]
        fn prop_mean_to_eccentric_round_trip(e in elliptic_eccentricity(), mean in -50.0..50.0f64) {
            let ecc = Eccentricity::try_new(e).unwrap();
            let eccentric = mean_to_eccentric(Angle::new(mean), ecc).unwrap().as_f64();
            // The solution stays within e of the mean anomaly, so in the same revolution.
            prop_assert!(abs(eccentric - mean) <= e + 8.0 * f64::EPSILON * abs(mean).max(1.0));
            let residual = eccentric_to_mean(Angle::new(eccentric), ecc).as_f64() - mean;
            prop_assert!(
                abs(residual) <= 8.0 * f64::EPSILON * abs(mean).max(1.0),
                "residual: {residual}"
            );
        }
    }

    #[test]
    fn test_true_anomaly_approx_eq_wraps_around() {
        let near_zero = Angle::new(1e-12);
        let near_two_pi = Angle::new(TAU - 1e-12);
        assert_approx_eq!(
            TrueAnomaly::new(near_two_pi),
            TrueAnomaly::new(near_zero),
            atol <= 1e-11
        );
        assert_approx_ne!(
            TrueAnomaly::new(Angle::new(-1e-20)),
            TrueAnomaly::new(Angle::ZERO),
            atol <= 0.0,
            rtol <= 0.0
        );
        // Eccentric and mean anomalies are unbounded for open orbits and do not wrap.
        assert_approx_ne!(
            EccentricAnomaly::new(near_two_pi),
            EccentricAnomaly::new(near_zero),
            atol <= 1e-11
        );
        assert_approx_ne!(
            MeanAnomaly::new(near_two_pi),
            MeanAnomaly::new(near_zero),
            atol <= 1e-11
        );
    }

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
        let ecc_back = mean_to_eccentric(mean, e).unwrap();

        assert!((ecc.as_f64() - ecc_back.as_f64()).abs() < EPSILON);
    }

    // Negative mean anomalies at high eccentricity, where the solver used to diverge
    #[rstest]
    #[case(-0.1, 0.95)]
    #[case(-0.5, 0.9)]
    #[case(-1.0, 0.85)]
    #[case(-2.0, 0.99)]
    fn test_keplers_equation_high_eccentricity(#[case] mean: f64, #[case] e: f64) {
        let e = Eccentricity::try_new(e).unwrap();

        let ecc = mean_to_eccentric(Angle::new(mean), e).unwrap();

        let residual = ecc.as_f64() - e.as_f64() * ecc.sin() - mean;
        assert!(residual.abs() < EPSILON, "residual: {residual}");
    }

    #[test]
    fn test_eccentric_to_mean_does_not_wrap_at_pi() {
        let e = Eccentricity::try_new(0.3).unwrap();

        let mean = eccentric_to_mean(Angle::PI, e);

        assert!((mean.as_f64() - Angle::PI.as_f64()).abs() < EPSILON);
    }

    #[test]
    fn test_conversions_keep_revolution() {
        let e = Eccentricity::try_new(0.3).unwrap();
        let rev = Angle::new(TAU);
        let nu = Angle::new(1.0);

        let ecc = true_to_eccentric(nu, e);
        let mean = eccentric_to_mean(ecc, e);

        let close = |a: Angle, b: Angle| (a - b).as_f64().abs() < EPSILON;
        assert!(close(true_to_eccentric(nu + rev, e), ecc + rev));
        assert!(close(eccentric_to_true(ecc - rev, e), nu - rev));
        assert!(close(eccentric_to_mean(ecc + rev, e), mean + rev));
        let ecc_back = mean_to_eccentric(mean + rev, e).unwrap();
        assert!(close(ecc_back, ecc + rev));
    }

    // Near-parabolic orbits close to periapsis, where the derivative of the residual vanishes.
    #[rstest]
    #[case(1e-15, 0.0, 1.0 - 1e-9)]
    #[case(-1e-15, 0.0, 1.0 - 1e-9)]
    #[case(1e-14, 0.0, 1.0 - 1e-10)]
    #[case(1.0, 0.8414709847237494, 0.5403023058141095)]
    #[case(-PI, 0.0, -(1.0 - 1e-10))]
    fn test_solve_kepler_near_parabolic(#[case] lambda: f64, #[case] h: f64, #[case] k: f64) {
        let f = solve_kepler(lambda, h, k);
        let residual = f - k * sin(f) + h * cos(f) - lambda;
        assert!(
            abs(residual) <= 8.0 * f64::EPSILON * abs(lambda).max(1.0),
            "residual: {residual}"
        );
    }

    #[rstest]
    #[case(f64::NAN)]
    #[case(f64::INFINITY)]
    #[case(f64::NEG_INFINITY)]
    fn test_mean_to_eccentric_rejects_non_finite_anomaly(#[case] mean: f64) {
        let e = Eccentricity::try_new(0.3).unwrap();
        assert!(matches!(
            mean_to_eccentric(Angle::new(mean), e),
            Err(AnomalyError::NonFiniteAnomaly(_))
        ));
        assert!(matches!(
            mean_to_true(Angle::new(mean), e),
            Err(AnomalyError::NonFiniteAnomaly(_))
        ));
    }

    #[rstest]
    #[case(1.0 - 5e-9)]
    #[case(1.0)]
    #[case(1.5)]
    fn test_mean_to_eccentric_rejects_open_orbits(#[case] e: f64) {
        let e = Eccentricity::try_new(e).unwrap();
        assert!(matches!(
            mean_to_eccentric(Angle::new(1.0), e),
            Err(AnomalyError::NotElliptic(_))
        ));
        assert!(matches!(
            mean_to_true(Angle::new(1.0), e),
            Err(AnomalyError::NotElliptic(_))
        ));
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
        // At apoapsis, eccentric anomaly should also be π
        let diff = (ecc.as_f64() - Angle::PI.as_f64()).abs();
        assert!(diff < EPSILON, "Expected π, got {}", ecc.as_f64());
    }

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

    macro_rules! newtype_tests {
        ($($module:ident: $anomaly:ident),*) => {
            $(
                mod $module {
                    use alloc::string::ToString;

                    use lox_approx::{assert_approx_eq, assert_approx_ne};

                    use super::*;

                    #[test]
                    fn test_accessors() {
                        let anomaly = $anomaly::new(1.5.rad());
                        assert_eq!(anomaly.as_angle(), 1.5.rad());
                        assert_eq!(anomaly.as_f64(), 1.5);
                    }

                    #[test]
                    fn test_default_is_zero() {
                        assert_eq!($anomaly::default().as_f64(), 0.0);
                    }

                    #[test]
                    fn test_arithmetic() {
                        let a = $anomaly::new(1.5.rad());
                        let b = $anomaly::new(0.5.rad());
                        assert_eq!(a + b, $anomaly::new(2.0.rad()));
                        assert_eq!(a - b, $anomaly::new(1.0.rad()));
                        assert_eq!(-a, $anomaly::new(-1.5.rad()));
                    }

                    #[test]
                    fn test_ordering() {
                        assert!($anomaly::new(0.5.rad()) < $anomaly::new(1.5.rad()));
                        assert!($anomaly::new(f64::NAN.rad())
                            .partial_cmp(&$anomaly::new(0.0.rad()))
                            .is_none());
                    }

                    #[test]
                    fn test_display_matches_angle() {
                        assert_eq!($anomaly::new(1.5.rad()).to_string(), 1.5.rad().to_string());
                    }

                    #[test]
                    fn test_approx_eq() {
                        let a = $anomaly::new(1.5.rad());
                        assert_approx_eq!(a, $anomaly::new((1.5 + 1e-12).rad()), rtol <= 1e-10);
                        assert_approx_ne!(a, $anomaly::new(1.6.rad()), rtol <= 1e-10);
                    }

                    #[cfg(feature = "serde")]
                    #[test]
                    fn test_serde_round_trip_as_bare_angle() {
                        let anomaly = $anomaly::new(1.5.rad());
                        let json = serde_json::to_string(&anomaly).unwrap();
                        assert_eq!(json, serde_json::to_string(&1.5.rad()).unwrap());
                        let round_trip: $anomaly = serde_json::from_str(&json).unwrap();
                        assert_eq!(anomaly, round_trip);
                    }
                }
            )*
        };
    }

    newtype_tests!(
        true_anomaly: TrueAnomaly,
        eccentric_anomaly: EccentricAnomaly,
        mean_anomaly: MeanAnomaly
    );
}
