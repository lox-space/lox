// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! `f64` math dispatched per build mode: inherent methods under `std`,
//! [`libm`](https://docs.rs/libm) under no_std. Free-function form (`sqrt(x)` not `x.sqrt()`) so
//! call sites don't need `cfg`-gated trait imports.
//!
//! Exception: [`mul_add_relaxed`] dispatches on the target's FMA support rather than the build
//! mode, see its docs and [`mul_add`] for the strict variant.

// Rounding helpers
// ================

/// Largest integer `<= x`.
#[inline]
pub fn floor(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.floor()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::floor(x)
    }
}

/// Rounds to the nearest integer, ties away from zero.
#[inline]
pub fn round(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.round()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::round(x)
    }
}

/// Truncates toward zero.
#[inline]
pub fn trunc(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.trunc()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::trunc(x)
    }
}

/// Returns the fractional part (`x - trunc(x)`).
#[inline]
pub fn fract(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.fract()
    }
    #[cfg(not(feature = "std"))]
    {
        x - libm::trunc(x)
    }
}

/// Least non-negative remainder of `x` modulo `y`, i.e. in `[0, |y|)` for finite operands.
#[inline]
pub fn rem_euclid(x: f64, y: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.rem_euclid(y)
    }
    #[cfg(not(feature = "std"))]
    {
        let r = x % y;
        if r < 0.0 { r + libm::fabs(y) } else { r }
    }
}

// Algebraic
// =========

/// Absolute value.
#[inline]
pub fn abs(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.abs()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::fabs(x)
    }
}

/// Sign function: `1.0`, `-1.0`, or `NaN`.
#[inline]
pub fn signum(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.signum()
    }
    #[cfg(not(feature = "std"))]
    {
        if x.is_nan() {
            f64::NAN
        } else {
            libm::copysign(1.0, x)
        }
    }
}

/// Magnitude of `x` with the sign of `sign`.
#[inline]
pub fn copysign(x: f64, sign: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.copysign(sign)
    }
    #[cfg(not(feature = "std"))]
    {
        libm::copysign(x, sign)
    }
}

/// Euclidean norm `sqrt(x² + y²)` without undue overflow or underflow.
#[inline]
pub fn hypot(x: f64, y: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.hypot(y)
    }
    #[cfg(not(feature = "std"))]
    {
        libm::hypot(x, y)
    }
}

/// Square root.
#[inline]
pub fn sqrt(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.sqrt()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::sqrt(x)
    }
}

/// Cube root.
#[inline]
pub fn cbrt(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.cbrt()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::cbrt(x)
    }
}

/// Raises `x` to an integer power.
#[inline]
pub fn powi(x: f64, n: i32) -> f64 {
    #[cfg(feature = "std")]
    {
        x.powi(n)
    }
    #[cfg(not(feature = "std"))]
    {
        libm::pow(x, n as f64)
    }
}

/// Raises `x` to a floating-point power.
#[inline]
pub fn powf(x: f64, n: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.powf(n)
    }
    #[cfg(not(feature = "std"))]
    {
        libm::pow(x, n)
    }
}

/// Multiply-add `a * b + c`, fused only where the target has hardware FMA.
///
/// Faster than [`mul_add`] on targets without hardware FMA, where a fused multiply-add is a
/// slow library call, at the cost of target-dependent rounding in the last bit.
#[inline]
pub fn mul_add_relaxed(a: f64, b: f64, c: f64) -> f64 {
    #[cfg(all(feature = "std", any(target_feature = "fma", target_arch = "aarch64")))]
    {
        a.mul_add(b, c)
    }
    #[cfg(not(all(feature = "std", any(target_feature = "fma", target_arch = "aarch64"))))]
    {
        a * b + c
    }
}

/// Fused multiply-add `a * b + c` with a single rounding.
#[inline]
pub fn mul_add(a: f64, b: f64, c: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        a.mul_add(b, c)
    }
    #[cfg(not(feature = "std"))]
    {
        libm::fma(a, b, c)
    }
}

// Trigonometric
// =============

/// Sine.
#[inline]
pub fn sin(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.sin()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::sin(x)
    }
}

/// Cosine.
#[inline]
pub fn cos(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.cos()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::cos(x)
    }
}

/// Tangent.
#[inline]
pub fn tan(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.tan()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::tan(x)
    }
}

/// Sine and cosine in a single call.
#[inline]
pub fn sin_cos(x: f64) -> (f64, f64) {
    #[cfg(feature = "std")]
    {
        x.sin_cos()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::sincos(x)
    }
}

/// Arcsine.
#[inline]
pub fn asin(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.asin()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::asin(x)
    }
}

/// Arccosine.
#[inline]
pub fn acos(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.acos()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::acos(x)
    }
}

/// Arctangent.
#[inline]
pub fn atan(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.atan()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::atan(x)
    }
}

/// Four-quadrant arctangent of `y/x`.
#[inline]
pub fn atan2(y: f64, x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        y.atan2(x)
    }
    #[cfg(not(feature = "std"))]
    {
        libm::atan2(y, x)
    }
}

// Hyperbolic
// ==========

/// Hyperbolic sine.
#[inline]
pub fn sinh(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.sinh()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::sinh(x)
    }
}

/// Hyperbolic cosine.
#[inline]
pub fn cosh(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.cosh()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::cosh(x)
    }
}

/// Hyperbolic tangent.
#[inline]
pub fn tanh(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.tanh()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::tanh(x)
    }
}

/// Inverse hyperbolic sine.
#[inline]
pub fn asinh(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.asinh()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::asinh(x)
    }
}

/// Inverse hyperbolic cosine.
#[inline]
pub fn acosh(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.acosh()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::acosh(x)
    }
}

/// Inverse hyperbolic tangent.
#[inline]
pub fn atanh(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.atanh()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::atanh(x)
    }
}

// Exponential and logarithmic
// ===========================

/// Exponential `eˣ`.
#[inline]
pub fn exp(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.exp()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::exp(x)
    }
}

/// `eˣ - 1`, accurate for `x` near zero.
#[inline]
pub fn exp_m1(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.exp_m1()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::expm1(x)
    }
}

/// Natural logarithm.
#[inline]
pub fn ln(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.ln()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::log(x)
    }
}

/// `ln(1 + x)`, accurate for `x` near zero.
#[inline]
pub fn ln_1p(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.ln_1p()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::log1p(x)
    }
}

/// Base-2 logarithm.
#[inline]
pub fn log2(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.log2()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::log2(x)
    }
}

/// Base-10 logarithm.
#[inline]
pub fn log10(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.log10()
    }
    #[cfg(not(feature = "std"))]
    {
        libm::log10(x)
    }
}

// Angle conversion (pure arithmetic, identical in both modes)
// ===========================================================

/// Converts radians to degrees.
#[inline]
pub const fn to_degrees(x: f64) -> f64 {
    x * (180.0 / core::f64::consts::PI)
}

/// Converts degrees to radians.
#[inline]
pub const fn to_radians(x: f64) -> f64 {
    x * (core::f64::consts::PI / 180.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt() {
        assert!(abs(sqrt(2.0) - core::f64::consts::SQRT_2) < 1e-15);
    }

    #[test]
    fn test_sin_cos_matches() {
        let (s, c) = sin_cos(0.5);
        assert!(abs(s - sin(0.5)) < 1e-15);
        assert!(abs(c - cos(0.5)) < 1e-15);
    }

    #[test]
    fn test_atan2() {
        assert!(abs(atan2(1.0, 1.0) - core::f64::consts::FRAC_PI_4) < 1e-15);
    }

    #[test]
    fn test_mul_add_single_rounding() {
        // (1 + 2⁻³⁰)² = 1 + 2⁻²⁹ + 2⁻⁶⁰; only a fused operation recovers the 2⁻⁶⁰ term
        let a = 1.0 + powi(2.0, -30);
        let p = a * a;
        assert_eq!(mul_add(a, a, -p), powi(2.0, -60));
    }

    #[test]
    fn test_mul_add_relaxed_exact_inputs() {
        assert_eq!(mul_add_relaxed(3.0, 4.0, 5.0), 17.0);
        assert_eq!(mul_add_relaxed(-0.5, 0.25, 1.0), 0.875);
    }

    #[test]
    fn test_mul_add_relaxed_is_fused_or_unfused() {
        let a = 1.0 + powi(2.0, -30);
        let p = a * a;
        let fused = mul_add(a, a, -p);
        let unfused = a * a - p;
        let actual = mul_add_relaxed(a, a, -p);
        #[cfg(all(feature = "std", any(target_feature = "fma", target_arch = "aarch64")))]
        assert_eq!(actual, fused);
        #[cfg(not(all(feature = "std", any(target_feature = "fma", target_arch = "aarch64"))))]
        assert_eq!(actual, unfused);
        assert!(actual == fused || actual == unfused);
    }

    #[test]
    fn test_exp_ln_round_trip() {
        assert!(abs(exp(1.0) - core::f64::consts::E) < 1e-15);
        assert!(abs(ln(exp(2.5)) - 2.5) < 1e-15);
    }

    #[test]
    fn test_exp_m1_ln_1p_small_argument() {
        // exp(x) - 1 and ln(1 + x) lose all significant digits at this magnitude
        let x = 1e-20;
        assert_eq!(exp_m1(x), x);
        assert_eq!(ln_1p(x), x);
    }

    #[test]
    fn test_log2() {
        assert_eq!(log2(1024.0), 10.0);
        assert_eq!(log2(0.125), -3.0);
    }

    #[test]
    fn test_hypot_avoids_overflow() {
        assert_eq!(hypot(3.0, 4.0), 5.0);
        let big = 1e300;
        assert!(abs(hypot(big, big) / big - core::f64::consts::SQRT_2) < 1e-15);
    }

    #[test]
    fn test_rem_euclid() {
        assert_eq!(rem_euclid(7.0, 4.0), 3.0);
        assert_eq!(rem_euclid(-7.0, 4.0), 1.0);
        assert_eq!(rem_euclid(7.0, -4.0), 3.0);
        assert_eq!(rem_euclid(-7.0, -4.0), 1.0);
        assert!(rem_euclid(1.0, 0.0).is_nan());
    }

    #[test]
    fn test_copysign() {
        assert_eq!(copysign(2.0, -0.0), -2.0);
        assert_eq!(copysign(-2.0, 1.0), 2.0);
        assert!(copysign(f64::NAN, -1.0).is_sign_negative());
    }
}
