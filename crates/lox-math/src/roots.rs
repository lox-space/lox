use crate::is_close::IsClose;
use float_eq::float_eq;
use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("root finder did not converge after {0} iterations")]
pub struct NotConverged(u32);

pub trait FindRoot<F: Fn(f64) -> f64> {
    type Error;

    fn find(&self, f: F, initial_guess: f64) -> Result<f64, Self::Error>;
}

pub trait FindRootWithDerivative<F: Fn(f64) -> f64, D: Fn(f64) -> f64> {
    type Error;

    fn find_with_derivative(
        &self,
        f: F,
        derivative: D,
        initial_guess: f64,
    ) -> Result<f64, Self::Error>;
}

pub trait FindBracketedRoot<F: Fn(f64) -> f64> {
    type Error: std::fmt::Debug;

    fn find_in_bracket(&self, f: F, bracket: (f64, f64)) -> Result<f64, Self::Error>;
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
    F: Fn(f64) -> f64,
{
    type Error = NotConverged;

    fn find(&self, f: F, initial_guess: f64) -> Result<f64, Self::Error> {
        let mut p0 = initial_guess;
        for _ in 0..self.max_iter {
            let f1 = p0 + f(p0);
            let f2 = f1 + f(f1);
            let p = p0 - (f1 - p0).powi(2) / (f2 - 2.0 * f1 + p0);
            if float_eq!(p, p0, abs <= self.tolerance) {
                return Ok(p);
            }
            p0 = p;
        }
        Err(NotConverged(self.max_iter))
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
    F: Fn(f64) -> f64,
    D: Fn(f64) -> f64,
{
    type Error = NotConverged;

    fn find_with_derivative(
        &self,
        f: F,
        derivative: D,
        initial_guess: f64,
    ) -> Result<f64, Self::Error> {
        let mut p0 = initial_guess;
        for _ in 0..self.max_iter {
            let p = p0 - f(p0) / derivative(p0);
            if float_eq!(p, p0, abs <= self.tolerance) {
                return Ok(p);
            }
            p0 = p;
        }
        Err(NotConverged(self.max_iter))
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum BracketError {
    #[error(transparent)]
    NotConverged(#[from] NotConverged),
    #[error("root not in bracket")]
    NotInBracket,
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
    F: Fn(f64) -> f64,
{
    type Error = BracketError;

    fn find_in_bracket(&self, f: F, bracket: (f64, f64)) -> Result<f64, Self::Error> {
        let mut fblk = 0.0;
        let mut xblk = 0.0;
        let (mut xpre, mut xcur) = bracket;
        let mut spre = 0.0;
        let mut scur = 0.0;

        let mut fpre = f(xpre);
        let mut fcur = f(xcur);

        if fpre * fcur > 0.0 {
            return Err(BracketError::NotInBracket);
        }

        if float_eq!(fpre, 0.0, abs <= self.abs_tol) {
            return Ok(xpre);
        }

        if float_eq!(fcur, 0.0, abs <= self.abs_tol) {
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

            if float_eq!(fcur, 0.0, abs <= self.abs_tol) || sbis.abs() < delta {
                return Ok(xcur);
            }

            if spre.abs() > delta && fcur.abs() < fpre.abs() {
                let stry = if float_eq!(xpre, xblk, rmax <= self.rel_tol) {
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

            fcur = f(xcur);
        }

        Err(BracketError::NotConverged(NotConverged(self.max_iter)))
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
    F: Fn(f64) -> f64,
{
    type Error = BracketError;

    fn find_in_bracket(&self, f: F, bracket: (f64, f64)) -> Result<f64, Self::Error> {
        let (x0, x1) = bracket;
        let mut p0 = x0;
        let mut p1 = x1;
        let mut q0 = f(p0);
        let mut q1 = f(p1);
        if q1.abs() < q0.abs() {
            std::mem::swap(&mut p0, &mut p1);
            std::mem::swap(&mut q0, &mut q1);
        }
        for i in 0..self.max_iter {
            if q1 == q0 {
                if p1 != p0 {
                    return Err(BracketError::NotConverged(NotConverged(i)));
                }
                return Ok((p1 + p0) / 2.0);
            }
            let p = if q1.abs() > q0.abs() {
                (-q0 / q1 * p1 + p0) / (1.0 - q0 / q1)
            } else {
                (-q1 / q0 * p0 + p1) / (1.0 - q1 / q0)
            };
            if p.is_close_with_tolerances(&p1, self.rel_tol, self.abs_tol) {
                return Ok(p);
            }
            p0 = p1;
            q0 = q1;
            p1 = p;
            q1 = f(p);
        }
        Err(BracketError::NotConverged(NotConverged(self.max_iter)))
    }
}

impl<F> FindRoot<F> for Secant
where
    F: Fn(f64) -> f64,
{
    type Error = BracketError;

    fn find(&self, f: F, initial_guess: f64) -> Result<f64, Self::Error> {
        let x0 = initial_guess;
        let eps = 1e-4;
        let mut x1 = x0 * (1.0 + eps);
        x1 += if x1 > x0 { eps } else { -eps };
        self.find_in_bracket(f, (x0, x1))
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use float_eq::assert_float_eq;

    use super::*;

    #[test]
    fn test_newton_kepler() {
        fn mean_to_ecc(mean: f64, eccentricity: f64) -> Result<f64, NotConverged> {
            let newton = Newton::default();
            newton.find_with_derivative(
                |e| e - eccentricity * e.sin() - mean,
                |e| 1.0 - eccentricity * e.cos(),
                mean,
            )
        }
        let act = mean_to_ecc(PI / 2.0, 0.3).expect("should converge");
        assert_float_eq!(act, 1.85846841205333, rel <= 1e-8);
    }

    #[test]
    fn test_newton_cubic() {
        let newton = Newton::default();
        let act = newton
            .find_with_derivative(
                |x| x.powi(3) + 4.0 * x.powi(2) - 10.0,
                |x| 2.0 * x.powi(2) + 8.0 * x,
                1.5,
            )
            .expect("should converge");
        assert_float_eq!(act, 1.3652300134140969, rel <= 1e-8);
    }

    #[test]
    fn test_steffensen_cubic() {
        let steffensen = Steffensen::default();
        let act = steffensen
            .find(|x| x.powi(3) + 4.0 * x.powi(2) - 10.0, 1.5)
            .expect("should converge");
        assert_float_eq!(act, 1.3652300134140969, rel <= 1e-8);
    }

    #[test]
    fn test_brent_cubic() {
        let brent = Brent::default();
        let act = brent
            .find_in_bracket(|x| x.powi(3) + 4.0 * x.powi(2) - 10.0, (1.0, 1.5))
            .expect("should converge");
        assert_float_eq!(act, 1.3652300134140969, rel <= 1e-8);
    }

    #[test]
    fn test_secant_cubic() {
        let secant = Secant::default();
        let act = secant
            .find_in_bracket(|x| x.powi(3) + 4.0 * x.powi(2) - 10.0, (1.0, 1.5))
            .expect("should converge");
        assert_float_eq!(act, 1.3652300134140969, rel <= 1e-8);
        let act = secant
            .find(|x| x.powi(3) + 4.0 * x.powi(2) - 10.0, 1.0)
            .expect("should converge");
        assert_float_eq!(act, 1.3652300134140969, rel <= 1e-8);
    }
}
