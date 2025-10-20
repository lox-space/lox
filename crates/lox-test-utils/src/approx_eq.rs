use std::{collections::HashMap, fmt::Display};

use glam::DVec3;

pub const fn default_rtol(atol: f64) -> f64 {
    if atol > 0.0 { 0.0 } else { 1e-8 }
}

pub enum ApproxEqResult {
    Pass,
    Fail { left: f64, right: f64 },
}

impl ApproxEqResult {
    pub fn new(left: f64, right: f64, atol: f64, rtol: f64) -> Self {
        let approx_eq =
            (left - right).abs() <= f64::max(rtol * f64::max(left.abs(), right.abs()), atol);
        if !approx_eq {
            Self::Fail { left, right }
        } else {
            Self::Pass
        }
    }
}

pub struct ApproxEqResults(HashMap<&'static str, ApproxEqResult>);

impl ApproxEqResults {
    pub fn is_approx_eq(&self) -> bool {
        self.0.values().all(|v| matches!(v, ApproxEqResult::Pass))
    }
}

impl Display for ApproxEqResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (&field, result) in &self.0 {
            match result {
                ApproxEqResult::Pass => continue,
                ApproxEqResult::Fail { left, right } => {
                    if field != "None" {
                        writeln!(f, "Field: {}", field)?;
                    }
                    writeln!(f, "Left:  {:?}", left)?;
                    writeln!(f, "Right: {:?}", right)?;
                    writeln!(f, "Diff:  {:?}", (left - right).abs())?;
                }
            }
        }
        write!(f, "")
    }
}

pub trait ApproxEq {
    fn approx_eq(self, rhs: Self, atol: f64, rtol: f64) -> ApproxEqResults;
}

#[doc(hidden)]
pub fn approx_eq_helper<T: ApproxEq>(lhs: T, rhs: T, atol: f64, rtol: f64) -> ApproxEqResults {
    lhs.approx_eq(rhs, atol, rtol)
}

impl ApproxEq for f64 {
    fn approx_eq(self, rhs: Self, atol: f64, rtol: f64) -> ApproxEqResults {
        ApproxEqResults(HashMap::from([(
            "None",
            ApproxEqResult::new(self, rhs, atol, rtol),
        )]))
    }
}

impl ApproxEq for DVec3 {
    fn approx_eq(self, rhs: Self, atol: f64, rtol: f64) -> ApproxEqResults {
        ApproxEqResults(HashMap::from([
            ("x", ApproxEqResult::new(self.x, rhs.x, atol, rtol)),
            ("y", ApproxEqResult::new(self.y, rhs.y, atol, rtol)),
            ("z", ApproxEqResult::new(self.z, rhs.z, atol, rtol)),
        ]))
    }
}

#[macro_export]
macro_rules! approx_eq {
    ($lhs:expr, $rhs:expr) => {
        approx_eq!($lhs, $rhs, 0.0, $crate::approx_eq::default_rtol(0.0))
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr) => {
        approx_eq!($lhs, $rhs, 0.0, $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr) => {
        approx_eq!($lhs, $rhs, $atol, $crate::approx_eq::default_rtol($atol))
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr, rtol <= $rtol:expr) => {
        $crate::approx_eq::approx_eq_helper($lhs, $rhs, $atol, $rtol).is_approx_eq()
    };
}

#[macro_export]
macro_rules! assert_approx_eq {
    ($lhs:expr, $rhs:expr) => {
        assert_approx_eq!(
            $lhs,
            $rhs,
            atol <= 0.0,
            rtol <= $crate::approx_eq::default_rtol(0.0)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr) => {
        assert_approx_eq!($lhs, $rhs, 0.0, $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr) => {
        assert_approx_eq!($lhs, $rhs, $atol, $crate::approx_eq::default_rtol($atol))
    };
    ($lhs:expr, $rhs:expr, atol <= $rtol:expr, rtol <= $atol:expr) => {
        assert!(
            $crate::approx_eq!($lhs, $rhs, atol <= $atol, rtol <= $rtol),
            "{:?} ≉ {:?}\n\nAbsolute tolerance: {:?}\nRelative tolerance: {:?}\n\n{}",
            $lhs,
            $rhs,
            $atol,
            $rtol,
            $crate::approx_eq::approx_eq_helper($lhs, $rhs, $atol, $rtol)
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approx_eq_f64() {
        assert_approx_eq!(1.0, 1.1)
    }

    #[test]
    fn test_approx_eq_dvec3() {
        let lhs = DVec3::new(1.0, 1.0, 1.1);
        let rhs = DVec3::new(1.0, 1.0, 1.0);
        assert_approx_eq!(lhs, rhs)
    }
}
