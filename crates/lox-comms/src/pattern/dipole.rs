// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Dipole antenna gain pattern.
//!
//! Supports arbitrary-length linear dipoles including short, half-wave, and full-wave.

use std::f64::consts::PI;

use lox_core::units::{Angle, Decibel, Distance, Frequency};

use crate::antenna::AntennaGain;

/// Threshold below which `|sin(θ)|` is treated as zero to avoid division by zero.
const DIV_BY_ZERO_LIMIT: f64 = 1e-6;

/// Floor gain in linear representation (~-120 dB).
const MINF_GAIN_LINEAR: f64 = 1e-12;

/// Dipoles shorter than this fraction of a wavelength use the short-dipole approximation.
const SHORT_DIPOLE_LIMIT: f64 = 0.1;

/// Dipole antenna gain pattern.
pub struct DipolePattern {
    /// Dipole length.
    pub length: Distance,
}

impl DipolePattern {
    /// Creates a new dipole pattern with the given length.
    pub fn new(length: Distance) -> Self {
        Self { length }
    }

    /// Returns the normalized linear gain pattern F(θ) at the given angle.
    ///
    /// For a short dipole (L < 0.1λ): F(θ) = sin²(θ).
    /// For a general dipole: F(θ) = ((cos(kL/2·cosθ) − cos(kL/2)) / sinθ)².
    pub fn gain_pattern(&self, frequency: Frequency, angle: Angle) -> f64 {
        let wavelength_m = frequency.wavelength().to_meters();
        let l = self.length.to_meters();
        let theta = angle.to_radians();
        let sin_theta = theta.sin();

        if sin_theta.abs() < DIV_BY_ZERO_LIMIT {
            return MINF_GAIN_LINEAR;
        }

        if l / wavelength_m < SHORT_DIPOLE_LIMIT {
            // Short dipole: F(θ) = sin²(θ)
            sin_theta * sin_theta
        } else {
            // General dipole
            let k = 2.0 * PI / wavelength_m;
            let half_kl = k * l / 2.0;
            let numerator = (half_kl * theta.cos()).cos() - half_kl.cos();
            (numerator / sin_theta).powi(2)
        }
    }

    /// Returns the directivity as a linear factor.
    ///
    /// D = 2 / ∫₀^π F(θ)·sin(θ) dθ
    ///
    /// Uses composite Simpson's rule with 1000 intervals.
    pub fn directivity(&self, frequency: Frequency) -> f64 {
        let n = 1000;
        let integral = simpsons_rule(0.0, PI, n, |theta| {
            let angle = Angle::radians(theta);
            self.gain_pattern(frequency, angle) * theta.sin()
        });
        2.0 / integral
    }

    /// Returns the peak gain in dBi at the given frequency.
    ///
    /// Uses golden-section search to find the angle maximizing the gain pattern,
    /// then combines with directivity.
    pub fn peak_gain(&self, frequency: Frequency) -> Decibel {
        let d = self.directivity(frequency);
        let (_, max_pattern) = golden_section_max(0.0, PI, |theta| {
            self.gain_pattern(frequency, Angle::radians(theta))
        });
        Decibel::from_linear(d * max_pattern)
    }
}

impl AntennaGain for DipolePattern {
    fn gain(&self, frequency: Frequency, angle: Angle) -> Decibel {
        let d = self.directivity(frequency);
        let pattern = self.gain_pattern(frequency, angle);
        let gain_linear = d * pattern;
        if gain_linear < MINF_GAIN_LINEAR {
            Decibel::from_linear(MINF_GAIN_LINEAR)
        } else {
            Decibel::from_linear(gain_linear)
        }
    }

    fn beamwidth(&self, _frequency: Frequency) -> Angle {
        // Beamwidth is not well-defined for a dipole; return π as a sentinel.
        Angle::radians(PI)
    }
}

/// Composite Simpson's rule for numerical integration over [a, b] with n intervals.
///
/// `n` must be even.
fn simpsons_rule(a: f64, b: f64, n: usize, f: impl Fn(f64) -> f64) -> f64 {
    debug_assert!(
        n % 2 == 0,
        "Simpson's rule requires an even number of intervals"
    );
    let h = (b - a) / n as f64;
    let mut sum = f(a) + f(b);
    for i in 1..n {
        let x = a + i as f64 * h;
        let weight = if i % 2 == 0 { 2.0 } else { 4.0 };
        sum += weight * f(x);
    }
    sum * h / 3.0
}

/// Golden-section search for the maximum of `f` on [a, b].
///
/// Returns `(x_max, f(x_max))`.
fn golden_section_max(a: f64, b: f64, f: impl Fn(f64) -> f64) -> (f64, f64) {
    let phi = (5.0_f64.sqrt() - 1.0) / 2.0; // golden ratio conjugate
    let tol = 1e-12;
    let mut a = a;
    let mut b = b;

    let mut c = b - phi * (b - a);
    let mut d = a + phi * (b - a);
    let mut fc = f(c);
    let mut fd = f(d);

    while (b - a).abs() > tol {
        if fc < fd {
            // Maximum is in [c, b]
            a = c;
            c = d;
            fc = fd;
            d = a + phi * (b - a);
            fd = f(d);
        } else {
            // Maximum is in [a, d]
            b = d;
            d = c;
            fd = fc;
            c = b - phi * (b - a);
            fc = f(c);
        }
    }

    let x = (a + b) / 2.0;
    (x, f(x))
}

#[cfg(test)]
mod tests {
    use lox_core::units::FrequencyUnits;
    use lox_test_utils::assert_approx_eq;

    use super::*;

    fn test_frequency() -> Frequency {
        29.0.ghz()
    }

    fn half_wave_dipole() -> DipolePattern {
        let wavelength = test_frequency().wavelength();
        DipolePattern::new(Distance::meters(wavelength.to_meters() / 2.0))
    }

    #[test]
    fn test_half_wave_dipole_at_zero() {
        // Gain along axis of dipole (θ=0) should be very low
        let d = half_wave_dipole();
        let gain = d.gain(test_frequency(), Angle::radians(0.0));
        assert!(gain.as_f64() < -50.0);
    }

    #[test]
    fn test_half_wave_dipole_at_broadside() {
        // Half-wave dipole peak gain ≈ 2.15 dBi at θ=π/2
        let d = half_wave_dipole();
        let gain = d.gain(test_frequency(), Angle::radians(PI / 2.0));
        assert_approx_eq!(gain.as_f64(), 2.15, atol <= 0.01);
    }

    #[test]
    fn test_short_dipole_peak() {
        // Short dipole (L = λ/100) peak gain ≈ 1.76 dBi
        let wavelength = test_frequency().wavelength().to_meters();
        let d = DipolePattern::new(Distance::meters(wavelength / 100.0));
        let peak = d.peak_gain(test_frequency());
        assert_approx_eq!(peak.as_f64(), 1.76, atol <= 0.1);
    }

    #[test]
    fn test_1_25_wavelength_dipole_peak() {
        // 1.25λ dipole peak gain ≈ 5.2 dBi
        let wavelength = test_frequency().wavelength().to_meters();
        let d = DipolePattern::new(Distance::meters(1.25 * wavelength));
        let peak = d.peak_gain(test_frequency());
        assert_approx_eq!(peak.as_f64(), 5.2, atol <= 0.1);
    }

    #[test]
    fn test_1_5_wavelength_dipole_at_45_degrees() {
        // 1.5λ dipole at 45° ≈ 3.5 dBi
        let wavelength = test_frequency().wavelength().to_meters();
        let d = DipolePattern::new(Distance::meters(1.5 * wavelength));
        let gain = d.gain(test_frequency(), Angle::degrees(45.0));
        assert_approx_eq!(gain.as_f64(), 3.5, atol <= 0.1);
    }

    #[test]
    fn test_simpsons_rule_sine() {
        // ∫₀^π sin(x) dx = 2
        let result = simpsons_rule(0.0, PI, 1000, |x| x.sin());
        assert_approx_eq!(result, 2.0, atol <= 1e-10);
    }

    #[test]
    fn test_golden_section_max_parabola() {
        // Maximum of -(x-2)² + 5 is at x=2 with value 5
        let (x_max, f_max) = golden_section_max(0.0, 4.0, |x| -(x - 2.0).powi(2) + 5.0);
        assert_approx_eq!(x_max, 2.0, atol <= 1e-6);
        assert_approx_eq!(f_max, 5.0, atol <= 1e-6);
    }
}
