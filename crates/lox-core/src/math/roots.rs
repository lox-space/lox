// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Root-finding algorithms: Steffensen, Newton, and Brent methods.

use alloc::boxed::Box;
use lox_approx::approx_eq;
use thiserror::Error;

use crate::math::float::{abs, powi, sqrt};

/// Error returned by root-finding algorithms.
#[derive(Debug, Error)]
pub enum RootFinderError {
    /// The algorithm did not converge within the maximum number of iterations.
    #[error("not converged after {iterations} iterations at x = {x}, residual {residual}")]
    NotConverged {
        /// Number of iterations performed before giving up.
        iterations: u32,
        /// The best root estimate reached.
        x: f64,
        /// The residual `f(x)` at the best estimate.
        residual: f64,
    },
    /// The root is not within the given bracket.
    #[error("root not in bracket")]
    NotInBracket,
    /// The objective function returned a non-finite value.
    #[error("function returned a non-finite value: {0}")]
    NonFinite(f64),
    /// The objective function returned an error.
    #[error(transparent)]
    Callback(#[from] CallbackError),
}

/// A boxed error type for use in root-finding callbacks.
pub type BoxedError = Box<dyn core::error::Error + Send + Sync + 'static>;

/// An error returned by a root-finding callback function.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct CallbackError(BoxedError);

impl From<&str> for CallbackError {
    fn from(s: &str) -> Self {
        CallbackError(s.into())
    }
}

impl From<BoxedError> for CallbackError {
    fn from(e: BoxedError) -> Self {
        CallbackError(e)
    }
}

/// A callable function for root-finding algorithms.
pub trait Callback {
    /// Evaluates the function at `v`.
    fn call(&self, v: f64) -> Result<f64, CallbackError>;
}

impl<F> Callback for F
where
    F: Fn(f64) -> Result<f64, BoxedError>,
{
    fn call(&self, v: f64) -> Result<f64, CallbackError> {
        self(v).map_err(CallbackError)
    }
}

/// Finds a root of `f` starting from an initial guess.
pub trait FindRoot<F>
where
    F: Callback,
{
    /// Finds a root of `f` starting from `initial_guess`.
    fn find(&self, f: F, initial_guess: f64) -> Result<f64, RootFinderError>;
}

/// Finds a root of `f` using both the function and its derivative.
pub trait FindRootWithDerivative<F, D>
where
    F: Callback,
    D: Callback,
{
    /// Finds a root of `f` using `derivative`, starting from `initial_guess`.
    fn find_with_derivative(
        &self,
        f: F,
        derivative: D,
        initial_guess: f64,
    ) -> Result<f64, RootFinderError>;
}

/// Finds a root of `f` within a bracket `(a, b)`.
pub trait FindBracketedRoot<F>
where
    F: Callback,
{
    /// Finds a root of `f` within `bracket`, reusing the function values at the
    /// bracket endpoints instead of evaluating them again.
    ///
    /// `values` must equal `(f(bracket.0), f(bracket.1))`.
    fn find_in_bracket_with_values(
        &self,
        f: F,
        bracket: (f64, f64),
        values: (f64, f64),
    ) -> Result<f64, RootFinderError>;

    /// Finds a root of `f` within the given `bracket`.
    fn find_in_bracket(&self, f: F, bracket: (f64, f64)) -> Result<f64, RootFinderError> {
        let fa = f.call(bracket.0).map_err(RootFinderError::Callback)?;
        let fb = f.call(bracket.1).map_err(RootFinderError::Callback)?;
        self.find_in_bracket_with_values(f, bracket, (fa, fb))
    }
}

/// Direction of a zero-crossing of a scalar function between two samples.
///
/// Classification uses a half-open convention: a sample of `0.0` belongs to the
/// non-negative ("active") side, so a crossing is a transition between a
/// negative sample and a non-negative one. This matches the `value >= 0`
/// convention used by interval/event detection. `NaN` samples do not define a
/// direction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ZeroCrossing {
    /// The signal crosses from negative to non-negative.
    Up,
    /// The signal crosses from non-negative to negative.
    Down,
}

impl ZeroCrossing {
    /// Classifies the crossing direction between two consecutive samples `s0`
    /// and `s1`, returning `None` when there is no crossing or either sample is
    /// `NaN`.
    ///
    /// A value of `0.0` counts as the non-negative side, so brackets are
    /// half-open.
    pub fn new(s0: f64, s1: f64) -> Option<ZeroCrossing> {
        if s0.is_nan() || s1.is_nan() {
            return None;
        }
        match (s0 < 0.0, s1 < 0.0) {
            (true, false) => Some(ZeroCrossing::Up),
            (false, true) => Some(ZeroCrossing::Down),
            _ => None,
        }
    }
}

impl core::fmt::Display for ZeroCrossing {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ZeroCrossing::Up => write!(f, "up"),
            ZeroCrossing::Down => write!(f, "down"),
        }
    }
}

/// Steffensen's method for root-finding (derivative-free).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Steffensen {
    max_iter: u32,
    /// Absolute tolerance on the root location.
    abs_tol: f64,
    /// Relative tolerance on the root location.
    rel_tol: f64,
}

impl Default for Steffensen {
    fn default() -> Self {
        Self {
            max_iter: 1000,
            abs_tol: sqrt(f64::EPSILON),
            rel_tol: sqrt(f64::EPSILON),
        }
    }
}

impl<F> FindRoot<F> for Steffensen
where
    F: Callback,
{
    fn find(&self, f: F, initial_guess: f64) -> Result<f64, RootFinderError> {
        let mut p0 = initial_guess;
        for _ in 0..self.max_iter {
            let fp0 = f.call(p0).map_err(RootFinderError::Callback)?;
            if !fp0.is_finite() {
                return Err(RootFinderError::NonFinite(fp0));
            }
            // An initial guess that is already a root is returned directly,
            // avoiding a 0/0 update.
            if fp0 == 0.0 {
                return Ok(p0);
            }
            let f1 = p0 + fp0;
            let ff1 = f.call(f1).map_err(RootFinderError::Callback)?;
            if !ff1.is_finite() {
                return Err(RootFinderError::NonFinite(ff1));
            }
            let f2 = f1 + ff1;
            let p = p0 - powi(f1 - p0, 2) / (f2 - 2.0 * f1 + p0);
            if !p.is_finite() {
                return Err(RootFinderError::NonFinite(p));
            }
            if approx_eq!(p, p0, rtol <= self.rel_tol, atol <= self.abs_tol) {
                return Ok(p);
            }
            p0 = p;
        }
        let residual = f.call(p0).map_err(RootFinderError::Callback)?;
        Err(RootFinderError::NotConverged {
            iterations: self.max_iter,
            x: p0,
            residual,
        })
    }
}

/// Newton-Raphson method for root-finding (requires derivative).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Newton {
    max_iter: u32,
    /// Absolute tolerance on the root location.
    abs_tol: f64,
    /// Relative tolerance on the root location.
    rel_tol: f64,
}

impl Default for Newton {
    fn default() -> Self {
        Self {
            max_iter: 50,
            abs_tol: sqrt(f64::EPSILON),
            rel_tol: sqrt(f64::EPSILON),
        }
    }
}

impl<F, D> FindRootWithDerivative<F, D> for Newton
where
    F: Callback,
    D: Callback,
{
    fn find_with_derivative(
        &self,
        f: F,
        derivative: D,
        initial_guess: f64,
    ) -> Result<f64, RootFinderError> {
        let mut p0 = initial_guess;
        for _ in 0..self.max_iter {
            let fx = f.call(p0).map_err(RootFinderError::Callback)?;
            if !fx.is_finite() {
                return Err(RootFinderError::NonFinite(fx));
            }
            // An initial guess that is already a root is returned directly,
            // avoiding a 0/0 update at a stationary point.
            if fx == 0.0 {
                return Ok(p0);
            }
            let dfx = derivative.call(p0).map_err(RootFinderError::Callback)?;
            if !dfx.is_finite() {
                return Err(RootFinderError::NonFinite(dfx));
            }
            let p = p0 - fx / dfx;
            if !p.is_finite() {
                return Err(RootFinderError::NonFinite(p));
            }
            if approx_eq!(p, p0, rtol <= self.rel_tol, atol <= self.abs_tol) {
                return Ok(p);
            }
            p0 = p;
        }
        let residual = f.call(p0).map_err(RootFinderError::Callback)?;
        Err(RootFinderError::NotConverged {
            iterations: self.max_iter,
            x: p0,
            residual,
        })
    }
}

/// Brent's method for bracketed root-finding.
///
/// Convergence is governed by the width of the bracket: iteration stops once it
/// is narrower than `abs_tol + rel_tol * |x|`. Both tolerances are on the root
/// location `x`, not on the residual `f(x)`, so the result is independent of how
/// the objective is scaled.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Brent {
    max_iter: u32,
    /// Absolute tolerance on the root location.
    abs_tol: f64,
    /// Relative tolerance on the root location.
    rel_tol: f64,
}

impl Default for Brent {
    fn default() -> Self {
        Self {
            max_iter: 100,
            abs_tol: 1e-6,
            rel_tol: sqrt(f64::EPSILON),
        }
    }
}

impl<F> FindBracketedRoot<F> for Brent
where
    F: Callback,
{
    fn find_in_bracket_with_values(
        &self,
        f: F,
        bracket: (f64, f64),
        values: (f64, f64),
    ) -> Result<f64, RootFinderError> {
        let mut fblk = 0.0;
        let mut xblk = 0.0;
        let (mut xpre, mut xcur) = bracket;
        let mut spre = 0.0;
        let mut scur = 0.0;

        let (mut fpre, mut fcur) = values;

        if !fpre.is_finite() {
            return Err(RootFinderError::NonFinite(fpre));
        }
        if !fcur.is_finite() {
            return Err(RootFinderError::NonFinite(fcur));
        }

        // An endpoint that is exactly a root is returned directly.
        if fpre == 0.0 {
            return Ok(xpre);
        }
        if fcur == 0.0 {
            return Ok(xcur);
        }

        // The endpoints must straddle the root. Comparing the sign bits of the
        // two finite, non-zero values avoids the underflow and NaN hazards of
        // testing the sign of their product.
        if fpre.is_sign_negative() == fcur.is_sign_negative() {
            return Err(RootFinderError::NotInBracket);
        }

        for _ in 0..self.max_iter {
            if fpre * fcur < 0.0 {
                xblk = xpre;
                fblk = fpre;
                spre = xcur - xpre;
                scur = xcur - xpre;
            }

            if abs(fblk) < abs(fcur) {
                xpre = xcur;
                xcur = xblk;
                xblk = xpre;
                fpre = fcur;
                fcur = fblk;
                fblk = fpre;
            }

            let delta = (self.abs_tol + self.rel_tol * abs(xcur)) / 2.0;
            let sbis = (xblk - xcur) / 2.0;

            if fcur == 0.0 || abs(sbis) < delta {
                return Ok(xcur);
            }

            if abs(spre) > delta && abs(fcur) < abs(fpre) {
                let stry = if approx_eq!(xpre, xblk, rtol <= self.rel_tol) {
                    // interpolate
                    -fcur * (xcur - xpre) / (fcur - fpre)
                } else {
                    // extrapolate
                    let dpre = (fpre - fcur) / (xpre - xcur);
                    let dblk = (fblk - fcur) / (xblk - xcur);
                    -fcur * (fblk * dblk - fpre * dpre) / (dblk * dpre * (fblk - fpre))
                };

                if 2.0 * abs(stry) < abs(spre).min(3.0 * abs(sbis) - delta) {
                    spre = scur;
                    scur = stry;
                } else {
                    // bisect
                    spre = sbis;
                    scur = sbis;
                }
            } else {
                // bisect
                spre = sbis;
                scur = sbis;
            }

            xpre = xcur;
            fpre = fcur;

            if abs(scur) > delta {
                xcur += scur
            } else {
                xcur += if sbis > 0.0 { delta } else { -delta };
            }

            fcur = f.call(xcur).map_err(RootFinderError::Callback)?;
        }

        Err(RootFinderError::NotConverged {
            iterations: self.max_iter,
            x: xcur,
            residual: fcur,
        })
    }
}

#[cfg(test)]
mod tests {
    use core::f64::consts::PI;
    use lox_approx::assert_approx_eq;

    use super::*;
    use crate::math::float::{cos, sin};

    type Result = core::result::Result<f64, BoxedError>;

    #[test]
    fn test_newton_kepler() {
        fn mean_to_ecc(mean: f64, eccentricity: f64) -> core::result::Result<f64, RootFinderError> {
            let newton = Newton::default();
            newton.find_with_derivative(
                |e: f64| -> Result { Ok(e - eccentricity * sin(e) - mean) },
                |e: f64| -> Result { Ok(1.0 - eccentricity * cos(e)) },
                mean,
            )
        }
        let act = mean_to_ecc(PI / 2.0, 0.3).expect("should converge");
        assert_approx_eq!(act, 1.85846841205333, rtol <= 1e-8);
    }

    #[test]
    fn test_newton_cubic() {
        let newton = Newton::default();
        let act = newton
            .find_with_derivative(
                |x: f64| -> Result { Ok(powi(x, 3) + 4.0 * powi(x, 2) - 10.0) },
                |x: f64| -> Result { Ok(2.0 * powi(x, 2) + 8.0 * x) },
                1.5,
            )
            .expect("should converge");
        assert_approx_eq!(act, 1.3652300134140969, rtol <= 1e-8);
    }

    #[test]
    fn test_newton_exact_root_initial_guess() {
        // f(x) = x^2, f'(x) = 2x. At x = 0 both are zero; the guess is already
        // the root and must be returned rather than producing 0/0 = NaN.
        let newton = Newton::default();
        let act = newton
            .find_with_derivative(
                |x: f64| -> Result { Ok(powi(x, 2)) },
                |x: f64| -> Result { Ok(2.0 * x) },
                0.0,
            )
            .expect("guess is already the root");
        assert_eq!(act, 0.0);
    }

    #[test]
    fn test_newton_zero_derivative_is_non_finite() {
        // f(x) = x^2 + 1 has no real root; at x = 0 the derivative is zero, so
        // the step diverges and must be reported as a non-finite error.
        let newton = Newton::default();
        let err = newton
            .find_with_derivative(
                |x: f64| -> Result { Ok(powi(x, 2) + 1.0) },
                |x: f64| -> Result { Ok(2.0 * x) },
                0.0,
            )
            .unwrap_err();
        assert!(matches!(err, RootFinderError::NonFinite(_)));
    }

    #[test]
    fn test_newton_large_root_relative_tolerance() {
        // A root at 1e8 is unreachable by an absolute step tolerance of
        // sqrt(EPSILON); the relative tolerance lets it converge.
        let newton = Newton::default();
        let act = newton
            .find_with_derivative(
                |x: f64| -> Result { Ok(powi(x, 2) - 1e16) },
                |x: f64| -> Result { Ok(2.0 * x) },
                9e7,
            )
            .expect("should converge");
        assert_approx_eq!(act, 1e8, rtol <= 1e-9);
    }

    #[test]
    fn test_steffensen_exact_root_initial_guess() {
        // f(x) = x^2 - 4 has a root at 2; the guess is already the root.
        let steffensen = Steffensen::default();
        let act = steffensen
            .find(|x: f64| -> Result { Ok(powi(x, 2) - 4.0) }, 2.0)
            .expect("guess is already the root");
        assert_eq!(act, 2.0);
    }

    #[test]
    fn test_steffensen_zero_denominator_is_non_finite() {
        // A constant non-zero residual makes the Aitken denominator vanish; the
        // update diverges and must be reported as a non-finite error.
        let steffensen = Steffensen::default();
        let err = steffensen
            .find(|_x: f64| -> Result { Ok(1.0) }, 0.0)
            .unwrap_err();
        assert!(matches!(err, RootFinderError::NonFinite(_)));
    }

    #[test]
    fn test_steffensen_cubic() {
        let steffensen = Steffensen::default();
        let act = steffensen
            .find(
                |x: f64| -> Result { Ok(powi(x, 3) + 4.0 * powi(x, 2) - 10.0) },
                1.5,
            )
            .expect("should converge");
        assert_approx_eq!(act, 1.3652300134140969, rtol <= 1e-8);
    }

    #[test]
    fn test_brent_cubic() {
        let brent = Brent::default();
        let act = brent
            .find_in_bracket(
                |x: f64| -> Result { Ok(powi(x, 3) + 4.0 * powi(x, 2) - 10.0) },
                (1.0, 1.5),
            )
            .expect("should converge");
        assert_approx_eq!(act, 1.3652300134140969, rtol <= 1e-8);
    }

    #[test]
    #[should_panic(expected = "derivative failed")]
    fn test_newton_kepler_callback_error() {
        let newton = Newton::default();
        newton
            .find_with_derivative(
                |e: f64| -> Result { Ok(e) },
                |_e: f64| -> Result { Err("derivative failed".into()) },
                1.0,
            )
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "f failed")]
    fn test_steffensen_cubic_error() {
        let steffensen = Steffensen::default();
        // function errors immediately
        steffensen
            .find(|_x| -> Result { Err("f failed".into()) }, 1.0)
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "negative x")]
    fn test_brent_cubic_error() {
        let brent = Brent::default();
        // error at bracket endpoint, then during iteration
        brent
            .find_in_bracket(
                |x: f64| -> Result {
                    if x.is_sign_negative() {
                        Err("negative x".into())
                    } else {
                        Ok(x * x - 2.0)
                    }
                },
                (-1.0, 2.0),
            )
            .unwrap();
    }

    #[test]
    fn test_zero_crossing() {
        // Negative -> positive is Up; positive -> negative is Down.
        assert_eq!(ZeroCrossing::new(-1.0, 1.0), Some(ZeroCrossing::Up));
        assert_eq!(ZeroCrossing::new(1.0, -1.0), Some(ZeroCrossing::Down));
        // Same side -> no crossing.
        assert_eq!(ZeroCrossing::new(-1.0, -2.0), None);
        assert_eq!(ZeroCrossing::new(1.0, 2.0), None);
        // Half-open: zero counts as the non-negative side.
        assert_eq!(ZeroCrossing::new(-1.0, 0.0), Some(ZeroCrossing::Up));
        assert_eq!(ZeroCrossing::new(0.0, -1.0), Some(ZeroCrossing::Down));
        assert_eq!(ZeroCrossing::new(0.0, 1.0), None);
        assert_eq!(ZeroCrossing::new(1.0, 0.0), None);
        // The sign of zero does not affect classification.
        assert_eq!(ZeroCrossing::new(-1.0, -0.0), Some(ZeroCrossing::Up));
        assert_eq!(ZeroCrossing::new(-0.0, -1.0), Some(ZeroCrossing::Down));
        // NaN never defines a direction.
        assert_eq!(ZeroCrossing::new(f64::NAN, 1.0), None);
        assert_eq!(ZeroCrossing::new(-1.0, f64::NAN), None);
    }

    #[test]
    fn test_find_in_bracket_with_values_reuses_endpoints() {
        use core::cell::Cell;

        let f = |x: f64| -> Result { Ok(powi(x, 3) + 4.0 * powi(x, 2) - 10.0) };
        let (a, b) = (1.0, 1.5);
        let fa = powi(a, 3) + 4.0 * powi(a, 2) - 10.0;
        let fb = powi(b, 3) + 4.0 * powi(b, 2) - 10.0;

        // The value-reuse entry point agrees with the recomputing one.
        let brent = Brent::default();
        let via_values = brent
            .find_in_bracket_with_values(f, (a, b), (fa, fb))
            .expect("should converge");
        let via_recompute = brent.find_in_bracket(f, (a, b)).expect("should converge");
        assert_approx_eq!(via_values, via_recompute, rtol <= 1e-12);

        // The supplied endpoints are not re-evaluated.
        let count = Cell::new(0usize);
        let counting = |x: f64| -> Result {
            if x == a || x == b {
                count.set(count.get() + 1);
            }
            Ok(powi(x, 3) + 4.0 * powi(x, 2) - 10.0)
        };
        Brent::default()
            .find_in_bracket_with_values(counting, (a, b), (fa, fb))
            .expect("should converge");
        assert_eq!(count.get(), 0, "endpoints must not be re-evaluated");
    }

    #[test]
    fn test_brent_rejects_non_finite_endpoint() {
        let brent = Brent::default();
        let err = brent
            .find_in_bracket_with_values(
                |_x: f64| -> Result { Ok(1.0) },
                (0.0, 1.0),
                (f64::NAN, 1.0),
            )
            .unwrap_err();
        assert!(matches!(err, RootFinderError::NonFinite(_)));
    }

    #[test]
    fn test_brent_rejects_same_sign_underflowing_bracket() {
        // Same-sign endpoints whose product underflows to 0.0 must still be
        // rejected rather than accepted as a bracket (and returned as a root).
        let brent = Brent::default();
        let err = brent
            .find_in_bracket(|x: f64| -> Result { Ok(1e-200 * (x + 1.0)) }, (0.0, 1.0))
            .unwrap_err();
        assert!(matches!(err, RootFinderError::NotInBracket));
    }

    #[test]
    fn test_brent_scale_independent() {
        // A heavily down-scaled objective: f(0) = -1e-6 must not be mistaken for
        // a root by a residual tolerance. The true root is at x = 1e6.
        let brent = Brent::default();
        let act = brent
            .find_in_bracket(|x: f64| -> Result { Ok(1e-12 * (x - 1e6)) }, (0.0, 2e6))
            .expect("should converge");
        assert_approx_eq!(act, 1e6, rtol <= 1e-5);
    }
}
