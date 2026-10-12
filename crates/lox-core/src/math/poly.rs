// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Polynomial evaluation.

use crate::math::float::mul_add_relaxed;

/// Evaluates the polynomial `c₀ + c₁x + c₂x² + … + cₙ₋₁xⁿ⁻¹` at `x` using Horner's method.
///
/// Coefficients are in ascending order of degree, i.e. `coeffs[0]` is the constant term. An
/// empty coefficient array evaluates to zero.
///
/// The degree is a const generic, so the loop is fully unrolled for each `N`. Each step is a
/// [`mul_add_relaxed`], which is fused only where the target has hardware FMA, so results may
/// differ in the last bits between targets.
///
/// # Examples
///
/// ```
/// use lox_core::math::poly::poly_array;
///
/// // 1 + 2x + 3x² at x = 2
/// assert_eq!(poly_array(2.0, &[1.0, 2.0, 3.0]), 17.0);
/// ```
#[inline]
pub fn poly_array<const N: usize>(x: f64, coeffs: &[f64; N]) -> f64 {
    // Seeding with the leading coefficient instead of 0.0 saves a multiply, which the compiler
    // cannot elide itself because `0.0 * x` is not zero for infinite or NaN `x`
    let Some((&last, rest)) = coeffs.split_last() else {
        return 0.0;
    };
    rest.iter()
        .rev()
        .fold(last, |acc, &c| mul_add_relaxed(acc, x, c))
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;
    use crate::math::float::{abs, powi};

    #[test]
    fn test_poly_array_empty_is_zero() {
        assert_eq!(poly_array(3.0, &[]), 0.0);
    }

    #[test]
    fn test_poly_array_constant() {
        assert_eq!(poly_array(3.0, &[42.0]), 42.0);
        // Not multiplied by x, so an infinite x leaves a constant untouched
        assert_eq!(poly_array(f64::INFINITY, &[42.0]), 42.0);
    }

    #[test]
    fn test_poly_array_ascending_order() {
        // 1 + 2x + 3x² vs. the reversed 3 + 2x + x²
        assert_eq!(poly_array(2.0, &[1.0, 2.0, 3.0]), 17.0);
        assert_eq!(poly_array(2.0, &[3.0, 2.0, 1.0]), 11.0);
    }

    // Small dyadic values keep every intermediate exact, so fused and unfused results agree
    #[rstest]
    #[case(0.0, 1.0)]
    #[case(1.0, 1.0 - 2.0 + 0.5 + 0.25 - 0.125)]
    #[case(-1.0, 1.0 + 2.0 + 0.5 - 0.25 - 0.125)]
    #[case(0.5, 1.0 - 1.0 + 0.125 + 0.03125 - 0.0078125)]
    #[case(-2.0, 1.0 + 4.0 + 2.0 - 2.0 - 2.0)]
    fn test_poly_array_exact(#[case] x: f64, #[case] expected: f64) {
        let coeffs = [1.0, -2.0, 0.5, 0.25, -0.125];
        assert_eq!(poly_array(x, &coeffs), expected);
    }

    #[test]
    fn test_poly_array_derivative_coefficients() {
        // Matches the derivative evaluation in `Series`: d/dx (1 + 2x + 3x² + 4x³)
        let c = [1.0, 2.0, 3.0, 4.0];
        let x = 1.5;
        let expected = 2.0 + 6.0 * x + 12.0 * x * x;
        assert_eq!(poly_array(x, &[c[1], 2.0 * c[2], 3.0 * c[3]]), expected);
    }

    #[test]
    fn test_poly_array_propagates_nan() {
        assert!(poly_array(f64::NAN, &[1.0, 2.0]).is_nan());
        assert!(poly_array(1.0, &[1.0, f64::NAN]).is_nan());
    }

    proptest! {
        #[test]
        fn prop_poly_array_matches_naive_sum(
            x in -2.0..2.0f64,
            coeffs in prop::array::uniform6(-1e3..1e3f64),
        ) {
            let naive: f64 = coeffs
                .iter()
                .enumerate()
                .map(|(i, &c)| c * powi(x, i as i32))
                .sum();
            // Both are backward stable, so bound the error by the magnitude of the terms
            let scale: f64 = coeffs
                .iter()
                .enumerate()
                .map(|(i, &c)| abs(c * powi(x, i as i32)))
                .sum();
            let actual = poly_array(x, &coeffs);
            prop_assert!(
                abs(actual - naive) <= 32.0 * f64::EPSILON * scale,
                "actual = {actual}, naive = {naive}, scale = {scale}"
            );
        }
    }
}
