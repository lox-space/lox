use std::f64::EPSILON;

pub trait FindRoot<F: Fn(f64) -> f64> {
    fn find_root(&self, f: F, initial_guess: f64) -> Option<f64>;
}

pub trait FindRootWithDerivative<F: Fn(f64) -> f64, D: Fn(f64) -> f64> {
    fn find_root(&self, f: F, deriv: D, initial_guess: f64) -> Option<f64>;
}

pub trait FindBracketedRoot<F: Fn(f64) -> f64> {
    fn find_root(&self, f: F, bracket: (f64, f64)) -> Option<f64>;
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Newton {
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
    fn find_root(&self, f: F, deriv: D, initial_guess: f64) -> Option<f64> {
        let mut p0 = initial_guess;
        let mut i = 0u32;
        while i < self.max_iter {
            let p = p0 - f(p0) / deriv(p0);
            if (p - p0).abs() < self.tolerance {
                return Some(p);
            }
            p0 = p;
            i += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use float_eq::assert_float_eq;

    use super::*;

    #[test]
    fn test_newton_kepler() {
        fn mean_to_ecc(mean: f64, eccentricity: f64) -> Option<f64> {
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
}
