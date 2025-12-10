// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_test_utils::approx_eq;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RootFinderError {
    #[error("not converged after {0} iterations, residual {1}")]
    NotConverged(u32, f64),
    #[error("root not in bracket")]
    NotInBracket,
    #[error(transparent)]
    Callback(#[from] CallbackError),
}

pub type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct CallbackError(BoxedError);

impl From<&str> for CallbackError {
    fn from(s: &str) -> Self {
        CallbackError(s.into())
    }
}

pub trait Callback {
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

pub trait FindRoot<F>
where
    F: Callback,
{
    fn find(&self, f: F, initial_guess: f64) -> Result<f64, RootFinderError>;
}

pub trait FindRootWithDerivative<F, D>
where
    F: Callback,
    D: Callback,
{
    fn find_with_derivative(
        &self,
        f: F,
        derivative: D,
        initial_guess: f64,
    ) -> Result<f64, RootFinderError>;
}

pub trait FindBracketedRoot<F>
where
    F: Callback,
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

impl<F> FindRoot<F> for Steffensen
where
    F: Callback,
{
    fn find(&self, f: F, initial_guess: f64) -> Result<f64, RootFinderError> {
        let mut p0 = initial_guess;
        for _ in 0..self.max_iter {
            let f1 = p0 + f.call(p0).map_err(RootFinderError::Callback)?;
            let f2 = f1 + f.call(f1).map_err(RootFinderError::Callback)?;
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
            let p = p0
                - f.call(p0).map_err(RootFinderError::Callback)?
                    / derivative.call(p0).map_err(RootFinderError::Callback)?;
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

impl<F> FindBracketedRoot<F> for Brent
where
    F: Callback,
{
    fn find_in_bracket(&self, f: F, bracket: (f64, f64)) -> Result<f64, RootFinderError> {
        let mut fblk = 0.0;
        let mut xblk = 0.0;
        let (mut xpre, mut xcur) = bracket;
        let mut spre = 0.0;
        let mut scur = 0.0;

        let mut fpre = f.call(xpre).map_err(RootFinderError::Callback)?;
        let mut fcur = f.call(xcur).map_err(RootFinderError::Callback)?;

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

            fcur = f.call(xcur).map_err(RootFinderError::Callback)?;
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

impl<F> FindBracketedRoot<F> for Secant
where
    F: Callback,
{
    fn find_in_bracket(&self, f: F, bracket: (f64, f64)) -> Result<f64, RootFinderError> {
        let (x0, x1) = bracket;
        let mut p0 = x0;
        let mut p1 = x1;
        let mut q0 = f.call(p0).map_err(RootFinderError::Callback)?;
        let mut q1 = f.call(p1).map_err(RootFinderError::Callback)?;
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
            q1 = f.call(p).map_err(RootFinderError::Callback)?;
        }
        Err(RootFinderError::NotConverged(self.max_iter, p0))
    }
}

impl<F> FindRoot<F> for Secant
where
    F: Callback,
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
    use std::f64::consts::PI;

    use super::*;

    type Result = std::result::Result<f64, BoxedError>;

    #[test]
    fn test_newton_kepler() {
        fn mean_to_ecc(mean: f64, eccentricity: f64) -> std::result::Result<f64, RootFinderError> {
            let newton = Newton::default();
            newton.find_with_derivative(
                |e: f64| -> Result { Ok(e - eccentricity * e.sin() - mean) },
                |e: f64| -> Result { Ok(1.0 - eccentricity * e.cos()) },
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
                |x: f64| -> Result { Ok(x.powi(3) + 4.0 * x.powi(2) - 10.0) },
                |x: f64| -> Result { Ok(2.0 * x.powi(2) + 8.0 * x) },
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
                |x: f64| -> Result { Ok(x.powi(3) + 4.0 * x.powi(2) - 10.0) },
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
                |x: f64| -> Result { Ok(x.powi(3) + 4.0 * x.powi(2) - 10.0) },
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
                |x: f64| -> Result { Ok(x.powi(3) + 4.0 * x.powi(2) - 10.0) },
                (1.0, 1.5),
            )
            .expect("should converge");
        assert_approx_eq!(act, 1.3652300134140969, rtol <= 1e-8);

        let act = secant
            .find(
                |x: f64| -> Result { Ok(x.powi(3) + 4.0 * x.powi(2) - 10.0) },
                1.0,
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
}
