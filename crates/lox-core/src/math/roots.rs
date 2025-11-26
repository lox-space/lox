// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_test_utils::approx_eq;
use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq)]
pub enum RootFinderError {
    #[error("not converged after {0} iterations, residual {1}")]
    NotConverged(u32, f64),
    #[error("root not in bracket")]
    NotInBracket,
    #[error("callback error: {0}")]
    CallbackError(String),
}

pub trait FindRoot<F, E>
where
    F: Fn(f64) -> Result<f64, E>,
    E: std::fmt::Display,
{
    fn find(&self, f: F, initial_guess: f64) -> Result<f64, RootFinderError>;
}

pub trait FindRootWithDerivative<F, D, E>
where
    F: Fn(f64) -> Result<f64, E>,
    D: Fn(f64) -> Result<f64, E>,
    E: std::fmt::Display,
{
    fn find_with_derivative(
        &self,
        f: F,
        derivative: D,
        initial_guess: f64,
    ) -> Result<f64, RootFinderError>;
}

pub trait FindBracketedRoot<F, E>
where
    F: Fn(f64) -> Result<f64, E>,
    E: std::fmt::Display,
{
    fn find_in_bracket(&self, f: F, bracket: (f64, f64)) -> Result<f64, RootFinderError>;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Steffensen {
    max_iter: u32,
    tolerance: f64,
}

impl Default for Steffensen {
    fn default() -> Self {
        Self {
            max_iter: 1000,
            tolerance: f64::EPSILON.sqrt(),
        }
    }
}

impl<F, E> FindRoot<F, E> for Steffensen
where
    F: Fn(f64) -> Result<f64, E>,
    E: std::fmt::Display,
{
    fn find(&self, f: F, initial_guess: f64) -> Result<f64, RootFinderError> {
        let mut p0 = initial_guess;
        for _ in 0..self.max_iter {
            let f1 = p0 + f(p0).map_err(|e| RootFinderError::CallbackError(e.to_string()))?;
            let f2 = f1 + f(f1).map_err(|e| RootFinderError::CallbackError(e.to_string()))?;
            let p = p0 - (f1 - p0).powi(2) / (f2 - 2.0 * f1 + p0);
            if approx_eq!(p, p0, atol <= self.tolerance) {
                return Ok(p);
            }
            p0 = p;
        }
        Err(RootFinderError::NotConverged(self.max_iter, p0))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Newton {
    max_iter: u32,
    tolerance: f64,
}

impl Default for Newton {
    fn default() -> Self {
        Self {
            max_iter: 50,
            tolerance: f64::EPSILON.sqrt(),
        }
    }
}

impl<F, D, E> FindRootWithDerivative<F, D, E> for Newton
where
    F: Fn(f64) -> Result<f64, E>,
    D: Fn(f64) -> Result<f64, E>,
    E: std::fmt::Display,
{
    fn find_with_derivative(
        &self,
        f: F,
        derivative: D,
        initial_guess: f64,
    ) -> Result<f64, RootFinderError> {
        let mut p0 = initial_guess;
        for _ in 0..self.max_iter {
            let p = p0
                - f(p0).map_err(|e| RootFinderError::CallbackError(e.to_string()))?
                    / derivative(p0).map_err(|e| RootFinderError::CallbackError(e.to_string()))?;
            if approx_eq!(p, p0, atol <= self.tolerance) {
                return Ok(p);
            }
            p0 = p;
        }
        Err(RootFinderError::NotConverged(self.max_iter, p0))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Brent {
    max_iter: u32,
    abs_tol: f64,
    rel_tol: f64,
}

impl Default for Brent {
    fn default() -> Self {
        Self {
            max_iter: 100,
            abs_tol: 1e-6,
            rel_tol: f64::EPSILON.sqrt(),
        }
    }
}

impl<F, E> FindBracketedRoot<F, E> for Brent
where
    F: Fn(f64) -> Result<f64, E>,
    E: std::fmt::Display,
{
    fn find_in_bracket(&self, f: F, bracket: (f64, f64)) -> Result<f64, RootFinderError> {
        let mut fblk = 0.0;
        let mut xblk = 0.0;
        let (mut xpre, mut xcur) = bracket;
        let mut spre = 0.0;
        let mut scur = 0.0;

        let mut fpre = f(xpre).map_err(|e| RootFinderError::CallbackError(e.to_string()))?;
        let mut fcur = f(xcur).map_err(|e| RootFinderError::CallbackError(e.to_string()))?;

        if fpre * fcur > 0.0 {
            return Err(RootFinderError::NotInBracket);
        }

        if approx_eq!(fpre, 0.0, atol <= self.abs_tol) {
            return Ok(xpre);
        }

        if approx_eq!(fcur, 0.0, atol <= self.abs_tol) {
            return Ok(xcur);
        }

        for _ in 0..self.max_iter {
            if fpre * fcur < 0.0 {
                xblk = xpre;
                fblk = fpre;
                spre = xcur - xpre;
                scur = xcur - xpre;
            }

            if fblk.abs() < fcur.abs() {
                xpre = xcur;
                xcur = xblk;
                xblk = xpre;
                fpre = fcur;
                fcur = fblk;
                fblk = fpre;
            }

            let delta = (self.abs_tol + self.rel_tol * xcur.abs()) / 2.0;
            let sbis = (xblk - xcur) / 2.0;

            if approx_eq!(fcur, 0.0, atol <= self.abs_tol) || sbis.abs() < delta {
                return Ok(xcur);
            }

            if spre.abs() > delta && fcur.abs() < fpre.abs() {
                let stry = if approx_eq!(xpre, xblk, rtol <= self.rel_tol) {
                    // interpolate
                    -fcur * (xcur - xpre) / (fcur - fpre)
                } else {
                    // extrapolate
                    let dpre = (fpre - fcur) / (xpre - xcur);
                    let dblk = (fblk - fcur) / (xblk - xcur);
                    -fcur * (fblk * dblk - fpre * dpre) / (dblk * dpre * (fblk - fpre))
                };

                if 2.0 * stry.abs() < spre.abs().min(3.0 * sbis.abs() - delta) {
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

            if scur.abs() > delta {
                xcur += scur
            } else {
                xcur += if sbis > 0.0 { delta } else { -delta };
            }

            fcur = f(xcur).map_err(|e| RootFinderError::CallbackError(e.to_string()))?;
        }

        Err(RootFinderError::NotConverged(self.max_iter, fcur))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Secant {
    max_iter: u32,
    rel_tol: f64,
    abs_tol: f64,
}

impl Default for Secant {
    fn default() -> Self {
        Self {
            max_iter: 100,
            rel_tol: f64::EPSILON.sqrt(),
            abs_tol: 1e-6,
        }
    }
}

impl<F, E> FindBracketedRoot<F, E> for Secant
where
    F: Fn(f64) -> Result<f64, E>,
    E: std::fmt::Display,
{
    fn find_in_bracket(&self, f: F, bracket: (f64, f64)) -> Result<f64, RootFinderError> {
        let (x0, x1) = bracket;
        let mut p0 = x0;
        let mut p1 = x1;
        let mut q0 = f(p0).map_err(|e| RootFinderError::CallbackError(e.to_string()))?;
        let mut q1 = f(p1).map_err(|e| RootFinderError::CallbackError(e.to_string()))?;
        if q1.abs() < q0.abs() {
            std::mem::swap(&mut p0, &mut p1);
            std::mem::swap(&mut q0, &mut q1);
        }
        for i in 0..self.max_iter {
            if q1 == q0 {
                if p1 != p0 {
                    // XXX: unsure about p0 parameter
                    return Err(RootFinderError::NotConverged(i, p0));
                }
                return Ok((p1 + p0) / 2.0);
            }
            let p = if q1.abs() > q0.abs() {
                (-q0 / q1 * p1 + p0) / (1.0 - q0 / q1)
            } else {
                (-q1 / q0 * p0 + p1) / (1.0 - q1 / q0)
            };
            if approx_eq!(p, p1, rtol <= self.rel_tol, atol <= self.abs_tol) {
                return Ok(p);
            }
            p0 = p1;
            q0 = q1;
            p1 = p;
            q1 = f(p).map_err(|e| RootFinderError::CallbackError(e.to_string()))?;
        }
        Err(RootFinderError::NotConverged(self.max_iter, p0))
    }
}

impl<F, E> FindRoot<F, E> for Secant
where
    F: Fn(f64) -> Result<f64, E>,
    E: std::fmt::Display,
{
    fn find(&self, f: F, initial_guess: f64) -> Result<f64, RootFinderError> {
        let x0 = initial_guess;
        let eps = 1e-4;
        let mut x1 = x0 * (1.0 + eps);
        x1 += if x1 > x0 { eps } else { -eps };
        self.find_in_bracket(f, (x0, x1))
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;
    use std::convert::Infallible;
    use std::f64::consts::PI;

    use super::*;

    pub type Fallible = Result<f64, Infallible>;

    #[test]
    fn test_newton_kepler() {
        fn mean_to_ecc(mean: f64, eccentricity: f64) -> Result<f64, RootFinderError> {
            let newton = Newton::default();
            newton.find_with_derivative(
                |e| -> Fallible { Ok(e - eccentricity * e.sin() - mean) },
                |e| -> Fallible { Ok(1.0 - eccentricity * e.cos()) },
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
                |x| -> Fallible { Ok(x.powi(3) + 4.0 * x.powi(2) - 10.0) },
                |x| -> Fallible { Ok(2.0 * x.powi(2) + 8.0 * x) },
                1.5,
            )
            .expect("should converge");
        assert_approx_eq!(act, 1.3652300134140969, rtol <= 1e-8);
    }

    #[test]
    fn test_steffensen_cubic() {
        let steffensen = Steffensen::default();
        let act = steffensen
            .find(
                |x| -> Fallible { Ok(x.powi(3) + 4.0 * x.powi(2) - 10.0) },
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
                |x| -> Fallible { Ok(x.powi(3) + 4.0 * x.powi(2) - 10.0) },
                (1.0, 1.5),
            )
            .expect("should converge");
        assert_approx_eq!(act, 1.3652300134140969, rtol <= 1e-8);
    }

    #[test]
    fn test_secant_cubic() {
        let secant = Secant::default();
        let act = secant
            .find_in_bracket(
                |x| -> Fallible { Ok(x.powi(3) + 4.0 * x.powi(2) - 10.0) },
                (1.0, 1.5),
            )
            .expect("should converge");
        assert_approx_eq!(act, 1.3652300134140969, rtol <= 1e-8);

        let act = secant
            .find(
                |x| -> Fallible { Ok(x.powi(3) + 4.0 * x.powi(2) - 10.0) },
                1.0,
            )
            .expect("should converge");
        assert_approx_eq!(act, 1.3652300134140969, rtol <= 1e-8);
    }

    use std::fmt;
    // Simple error type to simulate unexpected callback failures
    #[derive(Debug, Clone)]
    struct TestError(&'static str);
    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[test]
    fn test_newton_kepler_callback_error() {
        let newton = Newton::default();
        // derivative intentionally errors to test propagation
        let res = newton.find_with_derivative(
            |e| -> Result<f64, TestError> { Ok(e) },
            |_e| -> Result<f64, TestError> { Err(TestError("derivative failed")) },
            1.0,
        );
        match res {
            Err(RootFinderError::CallbackError(msg)) => assert!(msg.contains("derivative failed")),
            _ => panic!("expected CallbackError"),
        }
    }

    #[test]
    fn test_steffensen_cubic_error() {
        let steffensen = Steffensen::default();
        // function errors immediately
        let res = steffensen.find(
            |_x| -> Result<f64, TestError> { Err(TestError("f failed")) },
            1.0,
        );
        match res {
            Err(RootFinderError::CallbackError(msg)) => assert!(msg.contains("f failed")),
            _ => panic!("expected CallbackError"),
        }
    }

    #[test]
    fn test_brent_cubic_error() {
        let brent = Brent::default();
        // error at bracket endpoint, then during iteration
        let res = brent.find_in_bracket(
            |x| -> Result<f64, TestError> {
                if x.is_sign_negative() {
                    Err(TestError("negative x"))
                } else {
                    Ok(x * x - 2.0)
                }
            },
            (-1.0, 2.0),
        );
        match res {
            Err(RootFinderError::CallbackError(msg)) => assert!(msg.contains("negative x")),
            _ => panic!("expected CallbackError"),
        }
    }
}
