// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Result types for approximate equality comparisons.
//!
//! This module contains the result types used internally by the approximate equality system.
//! These types are typically not used directly but are returned by the [`ApproxEq`](crate::approx_eq::ApproxEq) trait
//! implementations.

use std::{borrow::Cow, fmt::Display};

/// The result of a single approximate equality comparison.
///
/// This type represents the outcome of comparing two `f64` values. It is designed to be
/// lightweight and is `Copy`, making it efficient to pass around and store.
///
/// # Variants
///
/// - [`Pass`](ApproxEqResult::Pass): The values are approximately equal within tolerance
/// - [`Fail`](ApproxEqResult::Fail): The values differ beyond the tolerance, with details
///   about the difference
#[derive(Debug, Clone, Copy)]
pub enum ApproxEqResult {
    /// The comparison passed - values are approximately equal.
    Pass,
    /// The comparison failed - values differ beyond tolerance.
    Fail {
        /// The left-hand side value.
        left: f64,
        /// The right-hand side value.
        right: f64,
        /// The absolute difference between values (if both are finite).
        diff: Option<f64>,
        /// The effective tolerance used (if both values are finite).
        tol: Option<f64>,
    },
}

impl ApproxEqResult {
    /// Creates a new comparison result for two `f64` values.
    ///
    /// # Parameters
    ///
    /// - `left`: The left-hand side value
    /// - `right`: The right-hand side value
    /// - `atol`: Absolute tolerance
    /// - `rtol`: Relative tolerance
    ///
    /// # Algorithm
    ///
    /// The effective tolerance is computed as:
    /// ```text
    /// tol = max(atol, rtol * max(|left|, |right|))
    /// ```
    ///
    /// The values are considered equal if `|left - right| ≤ tol`.
    ///
    /// # Special Cases
    ///
    /// If either value is non-finite (NaN or infinity), the comparison automatically fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_test_utils::approx_eq::ApproxEqResult;
    ///
    /// let result = ApproxEqResult::new(1.0, 1.0001, 0.001, 0.0);
    /// assert!(result.is_pass());
    ///
    /// let result = ApproxEqResult::new(1.0, 2.0, 0.0, 0.0);
    /// assert!(!result.is_pass());
    /// ```
    #[inline]
    pub fn new(left: f64, right: f64, atol: f64, rtol: f64) -> Self {
        if !left.is_finite() || !right.is_finite() {
            return Self::Fail {
                left,
                right,
                diff: None,
                tol: None,
            };
        }
        // Effective tolerance
        let tol = f64::max(atol, rtol * f64::max(left.abs(), right.abs()));
        let diff = (left - right).abs();
        if diff > tol {
            Self::Fail {
                left,
                right,
                diff: Some(diff),
                tol: Some(tol),
            }
        } else {
            Self::Pass
        }
    }

    /// Returns `true` if the comparison passed.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_test_utils::approx_eq::ApproxEqResult;
    ///
    /// let pass = ApproxEqResult::new(1.0, 1.0, 0.0, 0.0);
    /// assert!(pass.is_pass());
    ///
    /// let fail = ApproxEqResult::new(1.0, 2.0, 0.0, 0.0);
    /// assert!(!fail.is_pass());
    /// ```
    #[inline]
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }
}

/// A collection of comparison results.
///
/// This type is the return value of [`ApproxEq::approx_eq`](crate::approx_eq::ApproxEq::approx_eq)
/// and can represent either a single comparison result or multiple results for composite types.
///
/// # Variants
///
/// - [`Single`](ApproxEqResults::Single): A single comparison result
/// - [`Multiple`](ApproxEqResults::Multiple): Multiple results with field names
#[derive(Debug)]
pub enum ApproxEqResults {
    /// Single result - used for f64 and other scalar types.
    Single(ApproxEqResult),
    /// Multiple results with field names.
    Multiple(Vec<(Cow<'static, str>, ApproxEqResult)>),
}

impl Default for ApproxEqResults {
    fn default() -> Self {
        Self::new()
    }
}

impl ApproxEqResults {
    /// Creates a new empty results collection.
    ///
    /// This creates a [`Multiple`](ApproxEqResults::Multiple) variant with an empty `Vec`.
    /// Use [`single`](ApproxEqResults::single) for single-value comparisons.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_test_utils::approx_eq::ApproxEqResults;
    ///
    /// let results = ApproxEqResults::new();
    /// assert!(results.is_approx_eq()); // Empty results pass
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::Multiple(Vec::new())
    }

    /// Creates a results collection containing a single comparison result.
    ///
    /// This is used for scalar comparisons like `f64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_test_utils::approx_eq::{ApproxEqResult, ApproxEqResults};
    ///
    /// let result = ApproxEqResult::new(1.0, 1.0001, 0.001, 0.0);
    /// let results = ApproxEqResults::single(result);
    /// assert!(results.is_approx_eq());
    /// ```
    #[inline]
    pub fn single(result: ApproxEqResult) -> Self {
        Self::Single(result)
    }

    /// Returns `true` if all comparisons passed (values are approximately equal).
    ///
    /// For empty results collections, returns `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_test_utils::approx_eq::{ApproxEqResult, ApproxEqResults};
    ///
    /// let pass = ApproxEqResult::new(1.0, 1.0, 0.0, 0.0);
    /// let results = ApproxEqResults::single(pass);
    /// assert!(results.is_approx_eq());
    ///
    /// let fail = ApproxEqResult::new(1.0, 2.0, 0.0, 0.0);
    /// let results = ApproxEqResults::single(fail);
    /// assert!(!results.is_approx_eq());
    /// ```
    #[inline]
    pub fn is_approx_eq(&self) -> bool {
        match self {
            Self::Single(result) => result.is_pass(),
            Self::Multiple(results) => results.iter().all(|(_, r)| r.is_pass()),
        }
    }

    /// Returns `true` if any comparison failed (values are not approximately equal).
    ///
    /// This is the logical negation of [`is_approx_eq`](ApproxEqResults::is_approx_eq).
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_test_utils::approx_eq::{ApproxEqResult, ApproxEqResults};
    ///
    /// let fail = ApproxEqResult::new(1.0, 2.0, 0.0, 0.0);
    /// let results = ApproxEqResults::single(fail);
    /// assert!(results.is_approx_ne());
    /// ```
    #[inline]
    pub fn is_approx_ne(&self) -> bool {
        !self.is_approx_eq()
    }

    /// Inserts a single comparison result with an associated field name.
    ///
    /// This is primarily used internally when building up results for composite types.
    ///
    /// # Parameters
    ///
    /// - `field`: The field name to associate with this result
    /// - `result`: The comparison result to insert
    ///
    /// # Returns
    ///
    /// A mutable reference to `self` for method chaining.
    pub fn insert(&mut self, field: Cow<'static, str>, result: ApproxEqResult) -> &mut Self {
        match self {
            Self::Single(_) => {
                // Should not happen in normal usage, but handle it gracefully
                let old = std::mem::replace(self, Self::Multiple(Vec::new()));
                if let Self::Single(old_result) = old
                    && let Self::Multiple(vec) = self
                {
                    vec.push((Cow::Borrowed(""), old_result));
                    vec.push((field, result));
                }
            }
            Self::Multiple(vec) => {
                vec.push((field, result));
            }
        }
        self
    }

    /// Merges another results collection into this one, prefixing field names.
    ///
    /// This is used when comparing composite types. Each field name in `other` is prefixed
    /// with the provided `field` name, creating a hierarchical structure like `"x.y"`.
    ///
    /// # Parameters
    ///
    /// - `field`: The prefix to add to field names from `other`
    /// - `other`: The results collection to merge
    ///
    /// # Returns
    ///
    /// A mutable reference to `self` for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_test_utils::approx_eq::{ApproxEq, ApproxEqResults};
    ///
    /// let mut results = ApproxEqResults::new();
    /// results.merge("x", 1.0.approx_eq(&1.0, 0.0, 0.0));
    /// results.merge("y", 2.0.approx_eq(&2.0, 0.0, 0.0));
    /// assert!(results.is_approx_eq());
    /// ```
    pub fn merge<S: Into<Cow<'static, str>>>(&mut self, field: S, other: Self) -> &mut Self {
        let field: Cow<'static, str> = field.into();
        match other {
            Self::Single(result) => {
                self.insert(field, result);
            }
            Self::Multiple(other_results) => {
                // Ensure we're in Multiple mode
                if let Self::Single(old_result) = self {
                    *self = Self::Multiple(vec![(Cow::Borrowed(""), *old_result)]);
                }

                if let Self::Multiple(vec) = self {
                    for (other_field, result) in other_results {
                        let combined_field = if other_field.is_empty() {
                            field.clone()
                        } else {
                            Cow::Owned(format!("{}.{}", field, other_field))
                        };
                        vec.push((combined_field, result));
                    }
                }
            }
        }
        self
    }
}

impl Display for ApproxEqResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let results_iter: Box<dyn Iterator<Item = (&str, &ApproxEqResult)>> = match self {
            Self::Single(result) => Box::new(std::iter::once(("", result))),
            Self::Multiple(results) => Box::new(results.iter().map(|(s, r)| (s.as_ref(), r))),
        };

        for (field, result) in results_iter {
            match result {
                ApproxEqResult::Pass => continue,
                ApproxEqResult::Fail {
                    left,
                    right,
                    diff,
                    tol,
                } => {
                    if !field.is_empty() {
                        writeln!(f, "Field: {}", field)?;
                    }
                    writeln!(f, "Left:  {:?}", left)?;
                    writeln!(f, "Right: {:?}", right)?;
                    if let Some(diff) = diff {
                        writeln!(f, "Diff:  {:?}", diff)?;
                    }
                    if let Some(tol) = tol {
                        writeln!(f, "Tol:   {:?}\n", tol)?;
                    }
                }
            }
        }
        write!(f, "")
    }
}
