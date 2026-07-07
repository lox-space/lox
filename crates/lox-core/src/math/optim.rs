// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Bracketed optimization algorithms.

use lox_approx::approx_eq;
use thiserror::Error;

use crate::error::LoxError;
use crate::math::callback::Callback;
use crate::math::float::abs;

/// Error returned by bracketed minimization algorithms.
#[derive(Debug, Error)]
pub enum MinimizerError {
    /// The algorithm did not converge within the maximum number of iterations.
    #[error(
        "minimization did not converge after {iterations} iterations at x = {x}, f(x) = {value}"
    )]
    NotConverged {
        /// Number of iterations performed before giving up.
        iterations: u32,
        /// The best minimizer estimate reached.
        x: f64,
        /// The objective value `f(x)` at the best estimate.
        value: f64,
    },
    /// The objective function returned an error.
    #[error(transparent)]
    Callback(#[from] LoxError),
}

/// Finds the minimum of a function within a bracket.
pub trait FindBracketedMinimum<F>
where
    F: Callback,
{
    /// Finds the x value that minimizes `f` within the given `bracket`.
    fn find_minimum_in_bracket(&self, f: F, bracket: (f64, f64)) -> Result<f64, MinimizerError>;
}

/// Brent's method for finding the minimum of a unimodal function in a bracket.
///
/// Combines golden section search with parabolic interpolation.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BrentMinimizer {
    /// Maximum number of iterations.
    pub max_iter: u32,
    /// Absolute tolerance for convergence.
    pub abs_tol: f64,
}

impl Default for BrentMinimizer {
    fn default() -> Self {
        Self {
            max_iter: 500,
            abs_tol: 1e-10,
        }
    }
}

/// Golden ratio constant used in Brent minimization.
const GOLDEN: f64 = 0.381_966_011_250_105_1; // (3 - sqrt(5)) / 2

impl<F> FindBracketedMinimum<F> for BrentMinimizer
where
    F: Callback,
{
    fn find_minimum_in_bracket(&self, f: F, bracket: (f64, f64)) -> Result<f64, MinimizerError> {
        let (mut a, mut b) = bracket;
        if a > b {
            core::mem::swap(&mut a, &mut b);
        }

        // x is the point with the least function value found so far.
        // w is the point with the second least value.
        // v is the previous value of w.
        let mut x = a + GOLDEN * (b - a);
        let mut w = x;
        let mut v = x;
        let mut fx = f.call(x)?;
        let mut fw = fx;
        let mut fv = fx;

        // e is the distance moved on the step before last.
        // d is the distance moved on the last step.
        let mut e = 0.0_f64;
        let mut d = 0.0_f64;

        for _ in 0..self.max_iter {
            let midpoint = 0.5 * (a + b);
            let tol1 = self.abs_tol * abs(x) + 1e-10;
            let tol2 = 2.0 * tol1;

            // Check convergence.
            if abs(x - midpoint) <= tol2 - 0.5 * (b - a) {
                return Ok(x);
            }

            // Try parabolic interpolation.
            let mut use_golden = true;
            if abs(e) > tol1 {
                // Fit parabola through x, v, w.
                let r = (x - w) * (fx - fv);
                let q = (x - v) * (fx - fw);
                let p = (x - v) * q - (x - w) * r;
                let q = 2.0 * (q - r);
                let (p, q) = if q > 0.0 { (-p, q) } else { (p, -q) };

                // Is the parabola acceptable?
                if abs(p) < abs(0.5 * q * e) && p > q * (a - x) && p < q * (b - x) {
                    e = d;
                    d = p / q;
                    let u = x + d;

                    // f must not be evaluated too close to a or b.
                    if (u - a) < tol2 || (b - u) < tol2 {
                        d = if x < midpoint { tol1 } else { -tol1 };
                    }
                    use_golden = false;
                }
            }

            if use_golden {
                // Golden section step.
                e = if x < midpoint { b - x } else { a - x };
                d = GOLDEN * e;
            }

            // f must not be evaluated too close to x.
            let u = if abs(d) >= tol1 {
                x + d
            } else if d > 0.0 {
                x + tol1
            } else {
                x - tol1
            };

            let fu = f.call(u)?;

            // Update a, b, v, w, x.
            if fu <= fx {
                if u < x {
                    b = x;
                } else {
                    a = x;
                }
                v = w;
                fv = fw;
                w = x;
                fw = fx;
                x = u;
                fx = fu;
            } else {
                if u < x {
                    a = u;
                } else {
                    b = u;
                }
                if fu <= fw || approx_eq!(w, x, atol <= 1e-15) {
                    v = w;
                    fv = fw;
                    w = u;
                    fw = fu;
                } else if fu <= fv
                    || approx_eq!(v, x, atol <= 1e-15)
                    || approx_eq!(v, w, atol <= 1e-15)
                {
                    v = u;
                    fv = fu;
                }
            }
        }

        Err(MinimizerError::NotConverged {
            iterations: self.max_iter,
            x,
            value: fx,
        })
    }
}

#[cfg(test)]
mod tests {
    use core::f64::consts::PI;
    use lox_approx::assert_approx_eq;

    use super::*;
    use crate::math::float::{cos, powi};

    #[test]
    fn test_brent_minimizer_quadratic() {
        let minimizer = BrentMinimizer::default();
        let x = minimizer
            .find_minimum_in_bracket(|x: f64| powi(x - 3.0, 2), (0.0, 5.0))
            .expect("should converge");
        assert_approx_eq!(x, 3.0, atol <= 1e-8);
    }

    #[test]
    fn test_brent_minimizer_cosine() {
        // cos(x) has a minimum at PI in [PI/2, 3*PI/2]
        let minimizer = BrentMinimizer::default();
        let x = minimizer
            .find_minimum_in_bracket(|x: f64| cos(x), (PI / 2.0, 3.0 * PI / 2.0))
            .expect("should converge");
        assert_approx_eq!(x, PI, atol <= 1e-8);
    }

    #[test]
    fn test_brent_minimizer_reversed_bracket() {
        let minimizer = BrentMinimizer::default();
        let x = minimizer
            .find_minimum_in_bracket(|x: f64| powi(x - 2.0, 2), (5.0, 0.0))
            .expect("should converge");
        assert_approx_eq!(x, 2.0, atol <= 1e-8);
    }

    #[test]
    fn test_brent_minimizer_custom_tolerance() {
        let minimizer = BrentMinimizer {
            max_iter: 100,
            abs_tol: 1e-4,
        };
        let x = minimizer
            .find_minimum_in_bracket(|x: f64| powi(x - 1.0, 2), (-2.0, 5.0))
            .expect("should converge");
        assert_approx_eq!(x, 1.0, atol <= 1e-3);
    }

    #[test]
    fn test_brent_minimizer_not_converged() {
        let minimizer = BrentMinimizer {
            max_iter: 0,
            abs_tol: 1e-15,
        };
        let result = minimizer.find_minimum_in_bracket(|x: f64| powi(x - 1.0, 2), (0.0, 5.0));
        assert!(result.is_err());
    }
}
