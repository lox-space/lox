// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! `const fn`-compatible float helpers for `f64`.
//!
//! `f64::floor`, `f64::round`, and `f64::round_ties_even` are inherent methods
//! that became `const fn` in Rust 1.83. They're available unconditionally in
//! `std` builds but absent from `core` (no_std). [`num_traits::Float`] provides
//! these in no_std, but the trait methods are not `const fn`, which would force
//! us to drop `const` from a chain of `TimeDelta` and `Subsecond` constructors.
//!
//! This module dispatches to the inherent `const fn` methods in `std` mode and
//! to manual `const fn` implementations (using `f64 -> i64 -> f64` casts) in
//! no_std mode. The manual implementations are correct for finite values within
//! `i64` range, which covers every call site in this crate.

/// `const fn` equivalent of `f64::floor`.
///
/// For finite inputs within `i64::MIN..=i64::MAX`, returns the largest integer
/// `<= x`. NaN and infinities are returned unchanged.
#[inline]
pub(crate) const fn floor(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.floor()
    }
    #[cfg(not(feature = "std"))]
    {
        if x.is_nan() || x.is_infinite() {
            return x;
        }
        // f64 -> i64 cast truncates toward zero (saturates on overflow, NaN -> 0).
        let i = x as i64 as f64;
        if x < i { i - 1.0 } else { i }
    }
}

/// `const fn` equivalent of `f64::round` (round half away from zero).
///
/// For finite inputs within `i64::MIN..=i64::MAX`, rounds ties away from zero.
/// NaN and infinities are returned unchanged.
#[inline]
pub(crate) const fn round(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.round()
    }
    #[cfg(not(feature = "std"))]
    {
        if x.is_nan() || x.is_infinite() {
            return x;
        }
        let i = x as i64 as f64;
        let frac = x - i;
        if frac >= 0.5 {
            i + 1.0
        } else if frac <= -0.5 {
            i - 1.0
        } else {
            i
        }
    }
}

/// `const fn` equivalent of `f64::round_ties_even` (banker's rounding).
///
/// For finite inputs within `i64::MIN..=i64::MAX`, rounds half-integers to the
/// nearest even integer. NaN and infinities are returned unchanged.
#[inline]
pub(crate) const fn round_ties_even(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.round_ties_even()
    }
    #[cfg(not(feature = "std"))]
    {
        if x.is_nan() || x.is_infinite() {
            return x;
        }
        let i = x as i64 as f64;
        let frac = x - i;
        if frac > 0.5 {
            i + 1.0
        } else if frac < -0.5 {
            i - 1.0
        } else if frac == 0.5 {
            if (i as i64) % 2 == 0 { i } else { i + 1.0 }
        } else if frac == -0.5 {
            if (i as i64) % 2 == 0 { i } else { i - 1.0 }
        } else {
            i
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(1.5, 1.0)]
    #[case(-1.5, -2.0)]
    #[case(0.0, 0.0)]
    #[case(-0.0, 0.0)]
    #[case(3.7, 3.0)]
    #[case(-3.7, -4.0)]
    fn test_floor(#[case] input: f64, #[case] expected: f64) {
        assert_eq!(floor(input), expected);
    }

    #[rstest]
    #[case(0.5, 1.0)]
    #[case(-0.5, -1.0)]
    #[case(1.5, 2.0)]
    #[case(-1.5, -2.0)]
    #[case(0.3, 0.0)]
    #[case(-0.3, 0.0)]
    fn test_round(#[case] input: f64, #[case] expected: f64) {
        assert_eq!(round(input), expected);
    }

    #[rstest]
    #[case(0.5, 0.0)]
    #[case(1.5, 2.0)]
    #[case(2.5, 2.0)]
    #[case(3.5, 4.0)]
    #[case(-0.5, 0.0)]
    #[case(-1.5, -2.0)]
    #[case(-2.5, -2.0)]
    #[case(0.3, 0.0)]
    #[case(0.7, 1.0)]
    fn test_round_ties_even(#[case] input: f64, #[case] expected: f64) {
        assert_eq!(round_ties_even(input), expected);
    }

    #[test]
    fn test_const_floor_compiles() {
        const X: f64 = floor(3.7);
        assert_eq!(X, 3.0);
    }

    #[test]
    fn test_const_round_compiles() {
        const X: f64 = round(0.5);
        assert_eq!(X, 1.0);
    }

    #[test]
    fn test_const_round_ties_even_compiles() {
        const X: f64 = round_ties_even(2.5);
        assert_eq!(X, 2.0);
    }
}
