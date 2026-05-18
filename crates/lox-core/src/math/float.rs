// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! `f64` math dispatched per build mode: inherent methods under `std`,
//! [`libm`] under no_std. Free-function form (`sqrt(x)` not `x.sqrt()`) so
//! call sites don't need `cfg`-gated trait imports.

// Rounding helpers
// ================

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

/// Fused multiply-add: `a * b + c` with a single rounding.
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
}
