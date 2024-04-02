use float_eq::float_eq;
use std::f64::EPSILON;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum RootError {
    #[error("not converged")]
    NotConverged,
    #[error("root not in bracket")]
    NotInBracket,
}

pub trait FindRoot<F: Fn(f64) -> f64> {
    fn find_root(&self, f: F, initial_guess: f64) -> Result<f64, RootError>;
}

pub trait FindRootWithDerivative<F: Fn(f64) -> f64, D: Fn(f64) -> f64> {
    fn find_root(&self, f: F, deriv: D, initial_guess: f64) -> Result<f64, RootError>;
}

pub trait FindBracketedRoot<F: Fn(f64) -> f64> {
    fn find_root(&self, f: F, bracket: (f64, f64)) -> Result<f64, RootError>;
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
            tolerance: EPSILON.sqrt(),
        }
    }
}

impl<F> FindRoot<F> for Steffensen
where
    F: Fn(f64) -> f64,
{
    fn find_root(&self, f: F, initial_guess: f64) -> Result<f64, RootError> {
        let mut p0 = initial_guess;
        let mut i = 0u32;
        while i < self.max_iter {
            let f1 = p0 + f(p0);
            let f2 = f1 + f(f1);
            let p = p0 - (f1 - p0).powi(2) / (f2 - 2.0 * f1 + p0);
            if float_eq!(p, p0, abs <= self.tolerance) {
                return Ok(p);
            }
            p0 = p;
            i += 1;
        }
        Err(RootError::NotConverged)
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
            tolerance: EPSILON.sqrt(),
        }
    }
}

impl<F, D> FindRootWithDerivative<F, D> for Newton
where
    F: Fn(f64) -> f64,
    D: Fn(f64) -> f64,
{
    fn find_root(&self, f: F, deriv: D, initial_guess: f64) -> Result<f64, RootError> {
        let mut p0 = initial_guess;
        let mut i = 0u32;
        while i < self.max_iter {
            let p = p0 - f(p0) / deriv(p0);
            if float_eq!(p, p0, abs <= self.tolerance) {
                return Ok(p);
            }
            p0 = p;
            i += 1;
        }
        Err(RootError::NotConverged)
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
            rel_tol: EPSILON.sqrt(),
        }
    }
}

impl<F> FindBracketedRoot<F> for Brent
where
    F: Fn(f64) -> f64,
{
    fn find_root(&self, f: F, bracket: (f64, f64)) -> Result<f64, RootError> {
        let mut i = 0u32;
        let mut fblk = 0.0;
        let mut xblk = 0.0;
        let (mut xpre, mut xcur) = bracket;
        let mut spre = 0.0;
        let mut scur = 0.0;

        let mut fpre = f(xpre);
        let mut fcur = f(xcur);

        if fpre * fcur > 0.0 {
            return Err(RootError::NotInBracket);
        }

        if float_eq!(fpre, 0.0, abs <= self.abs_tol) {
            return Ok(xpre);
        }

        if float_eq!(fcur, 0.0, abs <= self.abs_tol) {
            return Ok(xcur);
        }

        while i < self.max_iter {
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

            let tol = self.abs_tol + self.rel_tol * xcur.abs();
            let sbis = (xblk - xcur) / 2.0;

            if float_eq!(fcur, 0.0, abs <= self.abs_tol) || sbis.abs() < tol {
                return Ok(xcur);
            }

            if spre.abs() > tol && fcur.abs() < fpre.abs() {
                let stry = if float_eq!(xpre, xblk, rmax <= self.rel_tol) {
                    -fcur * (xcur - xpre) / (fcur - fpre)
                } else {
                    let dpre = (fpre - fcur) / (xpre - xcur);
                    let dblk = (fblk - fcur) / (xblk - xcur);
                    -fcur * (fblk * dblk - fpre * dpre) / (dblk * dpre * (fblk - fpre))
                };

                if 2.0 * stry.abs() < spre.abs().min(3.0 * sbis.abs() - tol) {
                    spre = scur;
                    scur = stry;
                } else {
                    spre = sbis;
                    scur = sbis;
                }
            } else {
                spre = sbis;
                scur = sbis;
            }

            xpre = xcur;
            fpre = fcur;

            if scur.abs() > tol {
                xcur += scur
            } else {
                if sbis > 0.0 {
                    xcur += tol
                }
                if sbis < 0.0 {
                    xcur -= tol
                }
            }

            fcur = f(xcur);

            i += 1;
        }

        Err(RootError::NotConverged)
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use float_eq::assert_float_eq;

    use super::*;

    #[test]
    fn test_newton_kepler() {
        fn mean_to_ecc(mean: f64, eccentricity: f64) -> Result<f64, RootError> {
            let newton = Newton::default();
            newton.find_root(
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
            .find_root(
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
            .find_root(|x| x.powi(3) + 4.0 * x.powi(2) - 10.0, 1.5)
            .expect("should converge");
        assert_float_eq!(act, 1.3652300134140969, rel <= 1e-8);
    }

    #[test]
    fn test_brent_cubic() {
        let brent = Brent::default();
        let act = brent
            .find_root(|x| x.powi(3) + 4.0 * x.powi(2) - 10.0, (1.0, 1.5))
            .expect("should converge");
        assert_float_eq!(act, 1.3652300134140969, rel <= 1e-8);
    }
}
