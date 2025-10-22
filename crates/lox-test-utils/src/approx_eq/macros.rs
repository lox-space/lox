// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Macros for approximate equality testing.
//!
//! This module defines the core macros for testing floating-point equality:
//! - [`approx_eq!`](crate::approx_eq!) - Returns `bool`, checks if values are approximately equal
//! - [`approx_ne!`](crate::approx_ne!) - Returns `bool`, checks if values are not approximately equal
//! - [`assert_approx_eq!`](crate::assert_approx_eq!) - Panics if values are not approximately equal (for tests)
//! - [`assert_approx_ne!`](crate::assert_approx_ne!) - Panics if values are approximately equal (for tests)

/// Checks if two values are approximately equal.
///
/// Returns `true` if the values are approximately equal within the specified tolerances,
/// `false` otherwise. This macro is useful for conditional logic, while
/// [`assert_approx_eq!`](crate::assert_approx_eq!) should be used in tests.
///
/// # Syntax
///
/// ```text
/// approx_eq!(left, right)                          // Default tolerances
/// approx_eq!(left, right, atol <= tolerance)       // Custom absolute tolerance
/// approx_eq!(left, right, rtol <= tolerance)       // Custom relative tolerance
/// approx_eq!(left, right, atol <= a, rtol <= r)    // Both tolerances
/// approx_eq!(left, right, rtol <= r, atol <= a)    // Order doesn't matter
/// ```
///
/// # Default Tolerances
///
/// When no tolerances are specified:
/// - `atol = 0.0`
/// - `rtol = sqrt(f64::EPSILON)` ≈ 1.49e-8
///
/// # Examples
///
/// ```
/// use lox_test_utils::approx_eq;
///
/// // Default tolerances
/// assert!(approx_eq!(1.0, 1.0 + f64::EPSILON));
/// assert!(!approx_eq!(1.0, 1.1));
///
/// // Custom absolute tolerance
/// assert!(approx_eq!(1.0, 1.001, atol <= 0.01));
///
/// // Custom relative tolerance (1% difference allowed)
/// assert!(approx_eq!(100.0, 100.5, rtol <= 0.01));
///
/// // Both tolerances
/// assert!(approx_eq!(1.0, 1.001, atol <= 0.01, rtol <= 0.01));
///
/// // Works with vectors
/// use glam::DVec3;
/// let v1 = DVec3::new(1.0, 2.0, 3.0);
/// let v2 = DVec3::new(1.0 + f64::EPSILON, 2.0, 3.0);
/// assert!(approx_eq!(v1, v2));
/// ```
///
/// # See Also
///
/// - [`approx_ne!`](crate::approx_ne!) - For checking inequality
/// - [`assert_approx_eq!`](crate::assert_approx_eq!) - For test assertions with error messages
#[macro_export]
macro_rules! approx_eq {
    ($lhs:expr, $rhs:expr) => {
        approx_eq!(
            $lhs,
            $rhs,
            atol <= 0.0,
            rtol <= $crate::approx_eq::default_rtol(0.0)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr) => {
        approx_eq!($lhs, $rhs, atol <= 0.0, rtol <= $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr) => {
        approx_eq!(
            $lhs,
            $rhs,
            atol <= $atol,
            rtol <= $crate::approx_eq::default_rtol($atol)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr, atol <= $atol:expr) => {
        approx_eq!($lhs, $rhs, atol <= $atol, rtol <= $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr, rtol <= $rtol:expr) => {
        $crate::approx_eq::approx_eq_helper(&$lhs, &$rhs, $atol, $rtol).is_approx_eq()
    };
}

/// Checks if two values are not approximately equal.
///
/// Returns `true` if the values are not approximately equal (differ beyond the specified
/// tolerances), `false` otherwise. This is the logical negation of [`approx_eq!`](crate::approx_eq!).
///
/// # Syntax
///
/// ```text
/// approx_ne!(left, right)                          // Default tolerances
/// approx_ne!(left, right, atol <= tolerance)       // Custom absolute tolerance
/// approx_ne!(left, right, rtol <= tolerance)       // Custom relative tolerance
/// approx_ne!(left, right, atol <= a, rtol <= r)    // Both tolerances
/// approx_ne!(left, right, rtol <= r, atol <= a)    // Order doesn't matter
/// ```
///
/// # Examples
///
/// ```
/// use lox_test_utils::approx_ne;
///
/// // Values that differ significantly
/// assert!(approx_ne!(1.0, 2.0));
/// assert!(approx_ne!(1.0, 1.1));
///
/// // Values within epsilon are considered equal, so approx_ne returns false
/// assert!(!approx_ne!(1.0, 1.0 + f64::EPSILON));
///
/// // Custom tolerances
/// assert!(approx_ne!(1.0, 1.1, atol <= 0.01));
/// assert!(!approx_ne!(1.0, 1.001, atol <= 0.01));
/// ```
///
/// # See Also
///
/// - [`approx_eq!`](crate::approx_eq!) - For checking equality
/// - [`assert_approx_ne!`](crate::assert_approx_ne!) - For test assertions with error messages
#[macro_export]
macro_rules! approx_ne {
    ($lhs:expr, $rhs:expr) => {
        approx_ne!(
            $lhs,
            $rhs,
            atol <= 0.0,
            rtol <= $crate::approx_eq::default_rtol(0.0)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr) => {
        approx_ne!($lhs, $rhs, atol <= 0.0, rtol <= $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr) => {
        approx_ne!(
            $lhs,
            $rhs,
            atol <= $atol,
            rtol <= $crate::approx_eq::default_rtol($atol)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr, atol <= $atol:expr) => {
        approx_ne!($lhs, $rhs, atol <= $atol, rtol <= $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr, rtol <= $rtol:expr) => {
        $crate::approx_eq::approx_eq_helper(&$lhs, &$rhs, $atol, $rtol).is_approx_ne()
    };
}

/// Asserts that two values are approximately equal.
///
/// This macro is intended for use in tests. If the values are not approximately equal,
/// it panics with a detailed error message showing:
/// - The left and right values
/// - The tolerances used
/// - Which specific fields failed (for composite types)
/// - The actual difference and effective tolerance for failed comparisons
///
/// # Syntax
///
/// ```text
/// assert_approx_eq!(left, right)                          // Default tolerances
/// assert_approx_eq!(left, right, atol <= tolerance)       // Custom absolute tolerance
/// assert_approx_eq!(left, right, rtol <= tolerance)       // Custom relative tolerance
/// assert_approx_eq!(left, right, atol <= a, rtol <= r)    // Both tolerances
/// assert_approx_eq!(left, right, rtol <= r, atol <= a)    // Order doesn't matter
/// ```
///
/// # Examples
///
/// ```
/// use lox_test_utils::assert_approx_eq;
///
/// // This passes - values are within default tolerance
/// assert_approx_eq!(1.0, 1.0 + f64::EPSILON);
///
/// // Custom tolerance
/// assert_approx_eq!(1.0, 1.005, atol <= 0.01);
///
/// // Works with vectors
/// use glam::DVec3;
/// let v1 = DVec3::new(1.0, 2.0, 3.0);
/// let v2 = DVec3::new(1.0, 2.0, 3.0 + f64::EPSILON);
/// assert_approx_eq!(v1, v2);
/// ```
///
/// # Panics
///
/// Panics with a detailed error message if the values are not approximately equal:
///
/// ```should_panic
/// use lox_test_utils::assert_approx_eq;
///
/// // This will panic with a detailed error message
/// assert_approx_eq!(1.0, 2.0);
/// ```
///
/// # See Also
///
/// - [`approx_eq!`](crate::approx_eq!) - For non-panicking boolean check
/// - [`assert_approx_ne!`](crate::assert_approx_ne!) - For asserting inequality
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
        assert_approx_eq!($lhs, $rhs, atol <= 0.0, rtol <= $rtol);
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr) => {
        assert_approx_eq!(
            &$lhs,
            &$rhs,
            atol <= $atol,
            rtol <= $crate::approx_eq::default_rtol($atol)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr, atol <= $atol:expr) => {
        assert_approx_eq!($lhs, $rhs, atol <= $atol, rtol <= $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr, rtol <= $rtol:expr) => {{
        let result = $crate::approx_eq::approx_eq_helper(&$lhs, &$rhs, $atol, $rtol);
        assert!(
            result.is_approx_eq(),
            "{:?} ≉ {:?}\n\nAbsolute tolerance: {:?}\nRelative tolerance: {:?}\n\n{}",
            $lhs,
            $rhs,
            $atol,
            $rtol,
            result
        )
    }};
}

/// Asserts that two values are not approximately equal.
///
/// This macro is intended for use in tests. If the values are approximately equal
/// (within the specified tolerances), it panics with an error message.
///
/// # Syntax
///
/// ```text
/// assert_approx_ne!(left, right)                          // Default tolerances
/// assert_approx_ne!(left, right, atol <= tolerance)       // Custom absolute tolerance
/// assert_approx_ne!(left, right, rtol <= tolerance)       // Custom relative tolerance
/// assert_approx_ne!(left, right, atol <= a, rtol <= r)    // Both tolerances
/// assert_approx_ne!(left, right, rtol <= r, atol <= a)    // Order doesn't matter
/// ```
///
/// # Examples
///
/// ```
/// use lox_test_utils::assert_approx_ne;
///
/// // Values differ significantly
/// assert_approx_ne!(1.0, 2.0);
/// assert_approx_ne!(1.0, 1.1);
///
/// // Custom tolerance - these values differ beyond 1%
/// assert_approx_ne!(1.0, 1.5, atol <= 0.01);
///
/// // Works with vectors
/// use glam::DVec3;
/// let v1 = DVec3::new(1.0, 2.0, 3.0);
/// let v2 = DVec3::new(1.0, 2.0, 5.0);  // z differs significantly
/// assert_approx_ne!(v1, v2);
/// ```
///
/// # Panics
///
/// Panics if the values are approximately equal within the tolerances:
///
/// ```should_panic
/// use lox_test_utils::assert_approx_ne;
///
/// // This will panic because the values are equal
/// assert_approx_ne!(1.0, 1.0);
/// ```
///
/// # See Also
///
/// - [`approx_ne!`](crate::approx_ne!) - For non-panicking boolean check
/// - [`assert_approx_eq!`](crate::assert_approx_eq!) - For asserting equality
#[macro_export]
macro_rules! assert_approx_ne {
    ($lhs:expr, $rhs:expr) => {
        assert_approx_ne!(
            &$lhs,
            &$rhs,
            atol <= 0.0,
            rtol <= $crate::approx_eq::default_rtol(0.0)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr) => {
        assert_approx_ne!($lhs, $rhs, 0.0, $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr) => {
        assert_approx_ne!(
            $lhs,
            $rhs,
            atol <= $atol,
            rtol <= $crate::approx_eq::default_rtol($atol)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr, atol <= $atol:expr) => {
        assert_approx_ne!($lhs, $rhs, atol <= $atol, rtol <= $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr, rtol <= $rtol:expr) => {{
        let result = $crate::approx_eq::approx_eq_helper(&$lhs, &$rhs, $atol, $rtol);
        assert!(
            result.is_approx_ne(),
            "{:?} ≈ {:?}\n\nAbsolute tolerance: {:?}\nRelative tolerance: {:?}",
            $lhs,
            $rhs,
            $atol,
            $rtol,
        )
    }};
}
