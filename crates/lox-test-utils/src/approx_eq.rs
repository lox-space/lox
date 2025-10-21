//! Approximate equality testing for floating-point types.
//!
//! This module provides a robust framework for comparing floating-point values with configurable
//! tolerances. It implements the NumPy-style `isclose` algorithm, which uses both absolute and
//! relative tolerances to determine equality.
//!
//! # Algorithm
//!
//! Two values `a` and `b` are considered approximately equal if:
//!
//! ```text
//! |a - b| ≤ max(atol, rtol * max(|a|, |b|))
//! ```
//!
//! Where:
//! - `atol` is the absolute tolerance (default: 0.0)
//! - `rtol` is the relative tolerance (default: sqrt(f64::EPSILON) ≈ 1.49e-8)
//!
//! # Usage
//!
//! The primary interface is through the [`approx_eq!`](crate::approx_eq!), [`approx_ne!`](crate::approx_ne!), [`assert_approx_eq!`](crate::assert_approx_eq!),
//! and [`assert_approx_ne!`](crate::assert_approx_ne!) macros:
//!
//! ```
//! use lox_test_utils::{approx_eq, assert_approx_eq};
//!
//! // Default tolerances - uses rtol = sqrt(f64::EPSILON)
//! assert!(approx_eq!(1.0, 1.0 + f64::EPSILON));
//!
//! // Custom tolerances
//! assert!(approx_eq!(1.0, 1.001, atol <= 0.01));
//! assert!(approx_eq!(100.0, 100.1, rtol <= 0.01));
//! assert!(approx_eq!(1.0, 1.001, atol <= 0.01, rtol <= 0.01));
//!
//! // Assert macros with helpful error messages
//! assert_approx_eq!(1.0, 1.0 + f64::EPSILON);
//! ```
//!
//! # Supported Types
//!
//! The following types implement [`ApproxEq`] out of the box:
//! - `f64` - scalar floating-point values
//! - `glam::DVec3` - 3D vectors
//! - `glam::DMat3` - 3×3 matrices
//! - `Vec<T>` where `T: ApproxEq`
//! - `[T; N]` where `T: ApproxEq`
//!
//! # Custom Types
//!
//! You can implement [`ApproxEq`] for your own types either manually or using the derive macro
//! (requires the `derive` feature):
//!
//! ```ignore
//! use lox_test_utils::ApproxEq;
//!
//! #[derive(ApproxEq, Debug)]
//! struct Vec3 {
//!     x: f64,
//!     y: f64,
//!     z: f64,
//! }
//! ```
//!
//! # Performance
//!
//! This implementation is highly optimized for performance. Single scalar comparisons
//! (e.g., `f64`) complete in sub-nanosecond time.

use std::iter::zip;

use glam::{DMat3, DVec3};

pub mod macros;
pub mod results;

pub use results::{ApproxEqResult, ApproxEqResults};

/// Returns the default relative tolerance based on the absolute tolerance.
///
/// If `atol > 0.0`, returns `0.0` (only absolute tolerance is used).
/// Otherwise, returns `sqrt(f64::EPSILON)` ≈ 1.49e-8.
///
/// # Examples
///
/// ```
/// use lox_test_utils::approx_eq::default_rtol;
///
/// assert_eq!(default_rtol(0.0), f64::EPSILON.sqrt());
/// assert_eq!(default_rtol(0.01), 0.0);
/// ```
pub fn default_rtol(atol: f64) -> f64 {
    if atol > 0.0 { 0.0 } else { f64::EPSILON.sqrt() }
}

/// Trait for types that can be compared for approximate equality.
///
/// This trait is the foundation of the approximate equality system. Types implementing
/// this trait can be used with the [`approx_eq!`](crate::approx_eq!), [`approx_ne!`](crate::approx_ne!), [`assert_approx_eq!`](crate::assert_approx_eq!),
/// and [`assert_approx_ne!`](crate::assert_approx_ne!) macros.
///
/// # Type Parameters
///
/// - `Rhs`: The right-hand side type for comparison (defaults to `Self`)
///
/// # Examples
///
/// Implementing for a custom type:
///
/// ```
/// use lox_test_utils::approx_eq::{ApproxEq, ApproxEqResults, ApproxEqResult};
///
/// #[derive(Debug)]
/// struct Complex {
///     real: f64,
///     imag: f64,
/// }
///
/// impl ApproxEq for Complex {
///     fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> ApproxEqResults {
///         let mut results = ApproxEqResults::new();
///         results.merge("real", self.real.approx_eq(&rhs.real, atol, rtol));
///         results.merge("imag", self.imag.approx_eq(&rhs.imag, atol, rtol));
///         results
///     }
/// }
/// ```
pub trait ApproxEq<Rhs = Self>: std::fmt::Debug {
    /// Compares `self` with `rhs` for approximate equality.
    ///
    /// # Parameters
    ///
    /// - `rhs`: The value to compare against
    /// - `atol`: Absolute tolerance
    /// - `rtol`: Relative tolerance
    ///
    /// # Returns
    ///
    /// An [`ApproxEqResults`] object containing the comparison results.
    fn approx_eq(&self, rhs: &Rhs, atol: f64, rtol: f64) -> ApproxEqResults;
}

#[doc(hidden)]
pub fn approx_eq_helper<T: ApproxEq>(lhs: &T, rhs: &T, atol: f64, rtol: f64) -> ApproxEqResults {
    lhs.approx_eq(rhs, atol, rtol)
}

impl ApproxEq for f64 {
    #[inline]
    fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> ApproxEqResults {
        ApproxEqResults::single(ApproxEqResult::new(*self, *rhs, atol, rtol))
    }
}

impl ApproxEq for DVec3 {
    #[inline]
    fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> ApproxEqResults {
        let mut results = ApproxEqResults::new();
        results.merge("x", self.x.approx_eq(&rhs.x, atol, rtol));
        results.merge("y", self.y.approx_eq(&rhs.y, atol, rtol));
        results.merge("z", self.z.approx_eq(&rhs.z, atol, rtol));
        results
    }
}

impl ApproxEq for DMat3 {
    #[inline]
    fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> ApproxEqResults {
        let mut results = ApproxEqResults::new();
        results.merge("x_axis", self.x_axis.approx_eq(&rhs.x_axis, atol, rtol));
        results.merge("y_axis", self.y_axis.approx_eq(&rhs.y_axis, atol, rtol));
        results.merge("z_axis", self.z_axis.approx_eq(&rhs.z_axis, atol, rtol));
        results
    }
}

impl<T> ApproxEq for Vec<T>
where
    T: ApproxEq,
{
    fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> ApproxEqResults {
        let mut results = ApproxEqResults::new();
        for (idx, (left, right)) in zip(self, rhs).enumerate() {
            results.merge(format!("{}", idx), left.approx_eq(right, atol, rtol));
        }
        results
    }
}

impl<T, const N: usize> ApproxEq for [T; N]
where
    T: ApproxEq,
{
    fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> ApproxEqResults {
        let mut results = ApproxEqResults::new();
        for (idx, (left, right)) in zip(self, rhs).enumerate() {
            results.merge(format!("{}", idx), left.approx_eq(right, atol, rtol));
        }
        results
    }
}

impl<T, U> ApproxEq<&U> for &T
where
    T: ApproxEq<U>,
{
    fn approx_eq(&self, rhs: &&U, atol: f64, rtol: f64) -> ApproxEqResults {
        (*self).approx_eq(*rhs, atol, rtol)
    }
}

#[cfg(test)]
mod tests {
    use crate::{approx_eq, approx_ne, assert_approx_eq, assert_approx_ne};

    use super::*;

    #[cfg(feature = "derive")]
    mod derive_tests {
        use super::*;

        use crate::{ApproxEq, assert_approx_eq, assert_approx_ne};

        #[derive(ApproxEq, Debug)]
        struct Vec3 {
            x: f64,
            y: f64,
            z: f64,
        }

        #[test]
        fn test_approx_eq_derive_struct() {
            let v1 = Vec3 {
                x: 1.0,
                y: 1.0,
                z: 4.0,
            };
            let v2 = Vec3 {
                x: 1.0,
                y: 1.0,
                z: 4.00000000000001,
            };
            let v3 = Vec3 {
                x: 1.0,
                y: 1.0,
                z: 4.000000002,
            };
            let v4 = Vec3 {
                x: 1.0,
                y: 1.0,
                z: 4.00300002,
            };
            assert_approx_eq!(v1, v2);
            assert_approx_ne!(v3, v4);
        }

        #[derive(ApproxEq, Debug)]
        struct Pair(f64, f64);

        #[test]
        fn test_approx_eq_derive_tuple_struct() {
            let p1 = Pair(1.0, 4.0);
            let p2 = Pair(1.0, 4.00000000000001);
            let p3 = Pair(1.0, 4.000000002);
            let p4 = Pair(1.0, 4.00300002);

            assert_approx_eq!(p1, p2);
            assert_approx_ne!(p3, p4);
        }

        #[derive(ApproxEq, Debug)]
        struct Nested {
            pair: Pair,
            vec3: Vec3,
        }

        #[test]
        fn test_approx_eq_derive_nested() {
            let v1 = Vec3 {
                x: 1.0,
                y: 1.0,
                z: 4.0,
            };
            let v2 = Vec3 {
                x: 1.0,
                y: 1.0,
                z: 4.00000000000001,
            };
            let v3 = Vec3 {
                x: 1.0,
                y: 1.0,
                z: 4.000000002,
            };
            let v4 = Vec3 {
                x: 1.0,
                y: 1.0,
                z: 4.00300002,
            };

            let p1 = Pair(1.0, 4.0);
            let p2 = Pair(1.0, 4.00000000000001);
            let p3 = Pair(1.0, 4.000000002);
            let p4 = Pair(1.0, 4.00300002);

            let n1 = Nested { pair: p1, vec3: v1 };
            let n2 = Nested { pair: p2, vec3: v2 };
            let n3 = Nested { pair: p3, vec3: v3 };
            let n4 = Nested { pair: p4, vec3: v4 };

            assert_approx_eq!(n1, n2);
            assert_approx_ne!(n3, n4);
        }

        #[derive(ApproxEq, Debug)]
        struct Inner(f64);

        #[derive(ApproxEq, Debug)]
        struct Outer {
            inner: Inner,
        }

        #[derive(ApproxEq, Debug)]
        struct Hyper(Outer);

        #[test]
        fn test_approx_eq_derive_display_results() {
            let h1 = Hyper(Outer {
                inner: Inner(4.000000002),
            });
            let h2 = Hyper(Outer {
                inner: Inner(4.00300002),
            });
            let result = h1.approx_eq(&h2, 0.0, 1e-8).to_string();
            assert!(result.contains("Field: 0.inner.0"));
        }
    }

    #[test]
    fn test_approx_eq_f64() {
        assert_approx_eq!(4.00000000000001, 4.0);
        assert_approx_eq!(5.0, 4.999999999999993);
        assert_approx_ne!(4.000000002, 4.00300002);
        assert_approx_eq!(4.32, 4.3, rtol <= 0.1, atol <= 0.01);
        assert_approx_eq!(1.001, 1.002, rtol <= 0.001, atol <= 0.0001);
        assert_approx_ne!(4.5, 4.9, rtol <= 0.001, atol <= 0.001);
    }

    #[test]
    fn test_approx_eq_macros() {
        assert!(approx_eq!(4.00000000000001, 4.0));
        assert!(approx_eq!(5.0, 4.999999999999993));
        assert!(approx_ne!(4.000000002, 4.00300002));
        assert!(approx_eq!(4.32, 4.3, rtol <= 0.1, atol <= 0.01));
        assert!(approx_eq!(1.001, 1.002, rtol <= 0.001, atol <= 0.0001));
        assert!(approx_ne!(4.5, 4.9, rtol <= 0.001, atol <= 0.001));
    }

    #[test]
    fn test_approx_eq_dvec3() {
        let v1 = DVec3::new(1.0, 1.0, 4.0);
        let v2 = DVec3::new(1.0, 1.0, 4.00000000000001);
        let v3 = DVec3::new(1.0, 1.0, 4.000000002);
        let v4 = DVec3::new(1.0, 1.0, 4.00300002);
        assert_approx_eq!(v1, v2);
        assert_approx_ne!(v3, v4);
    }

    #[test]
    fn test_approx_eq_vec() {
        let v1 = vec![1.0, 1.0, 4.0];
        let v2 = vec![1.0, 1.0, 4.00000000000001];
        let v3 = vec![1.0, 1.0, 4.000000002];
        let v4 = vec![1.0, 1.0, 4.00300002];
        assert_approx_eq!(v1, v2);
        assert_approx_ne!(v3, v4);
    }

    #[test]
    fn test_approx_eq_array() {
        let v1 = [1.0, 1.0, 4.0];
        let v2 = [1.0, 1.0, 4.00000000000001];
        let v3 = [1.0, 1.0, 4.000000002];
        let v4 = [1.0, 1.0, 4.00300002];
        assert_approx_eq!(v1, v2);
        assert_approx_ne!(v3, v4);
    }

    #[test]
    fn test_approx_eq_display_results() {
        let v1 = DVec3::new(1.0, 1.0, 4.000000002);
        let v2 = DVec3::new(1.0, 1.0, 4.00300002);
        let result = v1.approx_eq(&v2, 0.0, 1e-8).to_string();
        assert!(result.contains("Field: z"));
    }
}
