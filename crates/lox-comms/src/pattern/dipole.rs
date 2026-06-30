// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Dipole antenna gain pattern.
//!
//! Supports arbitrary-length linear dipoles including short, half-wave, and full-wave.

use std::f64::consts::{FRAC_PI_2, PI};

use lox_core::units::{Angle, Decibel, Distance, Frequency};

use crate::antenna::AntennaGain;
use crate::error::NonPhysicalError;
use crate::pattern::GAIN_FLOOR_LINEAR;

/// Threshold below which `|sin(θ)|` is treated as zero to avoid division by zero.
const DIV_BY_ZERO_LIMIT: f64 = 1e-6;

/// Dipoles shorter than this fraction of a wavelength use the short-dipole approximation.
const SHORT_DIPOLE_LIMIT: f64 = 0.1;

/// Euler–Mascheroni constant γ.
const EULER_GAMMA: f64 = 0.577_215_664_901_532_9;

/// Dipole antenna gain pattern.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "DipolePatternRepr")
)]
pub struct DipolePattern {
    length: Distance,
}

/// Serde wire format for [`DipolePattern`]: forces deserialization through
/// the validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct DipolePatternRepr {
    length: Distance,
}

#[cfg(feature = "serde")]
impl TryFrom<DipolePatternRepr> for DipolePattern {
    type Error = NonPhysicalError;

    fn try_from(repr: DipolePatternRepr) -> Result<Self, Self::Error> {
        DipolePattern::new(repr.length)
    }
}

impl DipolePattern {
    /// Creates a new dipole pattern with the given length.
    ///
    /// Rejects a non-finite or non-positive length.
    pub fn new(length: Distance) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_positive("dipole length [m]", length.to_meters())?;
        Ok(Self { length })
    }

    /// Returns the dipole length.
    pub fn length(&self) -> Distance {
        self.length
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
            return GAIN_FLOOR_LINEAR;
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
    pub fn directivity(&self, frequency: Frequency) -> f64 {
        let wavelength_m = frequency.wavelength().to_meters();
        let l = self.length.to_meters();
        if l / wavelength_m < SHORT_DIPOLE_LIMIT {
            // Short dipole: ∫₀^π sin³(θ) dθ = 4/3 → D = 3/2
            return 1.5;
        }
        let kl = 2.0 * PI * l / wavelength_m;
        2.0 / pattern_integral(kl)
    }

    /// Returns the peak gain in dBi at the given frequency.
    pub fn peak_gain(&self, frequency: Frequency) -> Decibel {
        let d = self.directivity(frequency);
        let (_, max_pattern) = golden_section_max(0.0, PI, |theta| {
            self.gain_pattern(frequency, Angle::radians(theta))
        });
        Decibel::from_linear(d * max_pattern)
    }
}

impl AntennaGain for DipolePattern {
    fn gain(&self, frequency: Frequency, theta: Angle, _phi: Angle) -> Decibel {
        let d = self.directivity(frequency);
        let pattern = self.gain_pattern(frequency, theta);
        let gain_linear = d * pattern;
        if gain_linear < GAIN_FLOOR_LINEAR {
            Decibel::from_linear(GAIN_FLOOR_LINEAR)
        } else {
            Decibel::from_linear(gain_linear)
        }
    }
}

/// Closed form of ∫₀^π F(θ)·sin(θ) dθ for the general dipole pattern.
///
/// # References
///
/// Balanis, *Antenna Theory: Analysis and Design* (4th ed.), Eq. 4-68.
fn pattern_integral(kl: f64) -> f64 {
    let (si_kl, ci_kl) = sin_cos_integrals(kl);
    let (si_2kl, ci_2kl) = sin_cos_integrals(2.0 * kl);
    EULER_GAMMA + kl.ln() - ci_kl
        + 0.5 * kl.sin() * (si_2kl - 2.0 * si_kl)
        + 0.5 * kl.cos() * (EULER_GAMMA + (kl / 2.0).ln() + ci_2kl - 2.0 * ci_kl)
}

/// Crossover between the power series and the asymptotic expansion in
/// [`sin_cos_integrals`]. Worst-case relative error at the crossover is ~1e-7.
const SIN_COS_INTEGRAL_CROSSOVER: f64 = 16.0;

/// Sine and cosine integrals `(Si(x), Ci(x))` for `x > 0`.
///
/// # References
///
/// Abramowitz & Stegun, *Handbook of Mathematical Functions*, 5.2.1/5.2.2
/// (definitions), 5.2.8/5.2.9 (power series), 5.2.34/5.2.35 (asymptotic
/// auxiliary functions).
fn sin_cos_integrals(x: f64) -> (f64, f64) {
    debug_assert!(x > 0.0, "sine/cosine integrals require x > 0");
    if x <= SIN_COS_INTEGRAL_CROSSOVER {
        // Si(x)  = Σ_{k≥0} (−1)^k · x^(2k+1) / ((2k+1)·(2k+1)!)
        // Cin(x) = Σ_{k≥1} (−1)^(k+1) · x^(2k) / ((2k)·(2k)!)
        // Ci(x)  = γ + ln(x) − Cin(x)
        let x2 = x * x;
        let mut si = x;
        let mut cin = 0.0;
        // u = (−1)^k · x^(2k+1) / (2k+1)!, v = (−1)^(k+1) · x^(2k) / (2k)!
        let mut u = x;
        let mut v = x2 / 2.0;
        for k in 1..=64usize {
            cin += v / (2 * k) as f64;
            u *= -x2 / ((2 * k) as f64 * (2 * k + 1) as f64);
            si += u / (2 * k + 1) as f64;
            if u.abs() < 1e-17 && v.abs() < 1e-17 {
                break;
            }
            v *= -x2 / ((2 * k + 1) as f64 * (2 * k + 2) as f64);
        }
        (si, EULER_GAMMA + x.ln() - cin)
    } else {
        // f(x) ≈ (1/x)·Σ (−1)^k·(2k)!/x^(2k), g(x) ≈ (1/x²)·Σ (−1)^k·(2k+1)!/x^(2k),
        // truncated at the smallest term.
        let x2 = x * x;
        let mut f = 1.0;
        let mut g = 1.0;
        let mut tf = 1.0;
        let mut tg = 1.0;
        for k in 1..=16usize {
            let next_tf = tf * (-(((2 * k - 1) * (2 * k)) as f64) / x2);
            let next_tg = tg * (-(((2 * k) * (2 * k + 1)) as f64) / x2);
            if next_tf.abs() >= tf.abs() || next_tg.abs() >= tg.abs() {
                break;
            }
            tf = next_tf;
            tg = next_tg;
            f += tf;
            g += tg;
        }
        let f = f / x;
        let g = g / x2;
        let (sin_x, cos_x) = x.sin_cos();
        // Si(x) = π/2 − f·cos(x) − g·sin(x), Ci(x) = f·sin(x) − g·cos(x)
        (FRAC_PI_2 - f * cos_x - g * sin_x, f * sin_x - g * cos_x)
    }
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
    use lox_approx::assert_approx_eq;
    use lox_core::units::FrequencyUnits;

    use super::*;

    /// Composite Simpson's rule, kept as the reference oracle for the
    /// closed-form directivity. `n` must be even.
    fn simpsons_rule(a: f64, b: f64, n: usize, f: impl Fn(f64) -> f64) -> f64 {
        assert!(
            n.is_multiple_of(2),
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

    fn test_frequency() -> Frequency {
        29.0.ghz()
    }

    fn half_wave_dipole() -> DipolePattern {
        let wavelength = test_frequency().wavelength();
        DipolePattern::new(Distance::meters(wavelength.to_meters() / 2.0)).unwrap()
    }

    #[test]
    fn test_length_accessor() {
        let d = DipolePattern::new(Distance::meters(0.5)).unwrap();
        assert_approx_eq!(d.length().to_meters(), 0.5, atol <= 1e-15);
    }

    #[test]
    fn test_gain_pattern_regimes() {
        let c = 299792458.0;
        let wavelength = c / 29e9;
        let f = Frequency::hertz(29e9);

        // Short dipole: F(90 deg) = sin^2 = 1.
        let short = DipolePattern::new(Distance::meters(wavelength / 100.0)).unwrap();
        assert_approx_eq!(
            short.gain_pattern(f, Angle::degrees(90.0)),
            1.0,
            atol <= 1e-12
        );

        // General (half-wave) dipole peaks broadside with F(90 deg) = (1 - cos(pi/2))^2 = 1.
        let half_wave = DipolePattern::new(Distance::meters(wavelength / 2.0)).unwrap();
        assert_approx_eq!(
            half_wave.gain_pattern(f, Angle::degrees(90.0)),
            1.0,
            atol <= 1e-9
        );

        // On-axis the pattern floors out instead of dividing by zero.
        assert!(half_wave.gain_pattern(f, Angle::degrees(0.0)) < 1e-9);
    }

    #[test]
    fn test_sin_cos_integrals_known_values() {
        // Reference values from Abramowitz & Stegun, Table 5.1
        let (si, ci) = sin_cos_integrals(1.0);
        assert_approx_eq!(si, 0.946_083_070_367_183, atol <= 1e-9);
        assert_approx_eq!(ci, 0.337_403_922_900_968, atol <= 1e-9);

        let (si, ci) = sin_cos_integrals(5.0);
        assert_approx_eq!(si, 1.549_931_244_944_674, atol <= 1e-9);
        assert_approx_eq!(ci, -0.190_029_749_656_644, atol <= 1e-9);

        let (si, ci) = sin_cos_integrals(10.0);
        assert_approx_eq!(si, 1.658_347_594_218_874, atol <= 1e-9);
        assert_approx_eq!(ci, -0.045_456_433_004_455, atol <= 1e-9);

        let (si, ci) = sin_cos_integrals(20.0);
        assert_approx_eq!(si, 1.548_241_701_043_44, atol <= 1e-6);
        assert_approx_eq!(ci, 0.044_419_820_845_353, atol <= 1e-6);
    }

    #[test]
    fn test_sin_cos_integrals_continuous_at_crossover() {
        let below = sin_cos_integrals(SIN_COS_INTEGRAL_CROSSOVER - 1e-9);
        let above = sin_cos_integrals(SIN_COS_INTEGRAL_CROSSOVER + 1e-9);
        assert_approx_eq!(below.0, above.0, atol <= 1e-6);
        assert_approx_eq!(below.1, above.1, atol <= 1e-6);
    }

    #[test]
    fn test_directivity_matches_numerical_integration() {
        // The closed form must agree with direct integration of the pattern.
        let f = test_frequency();
        let wavelength = f.wavelength().to_meters();
        for length_factor in [0.5, 1.0, 1.25, 1.5, 2.3] {
            let d = DipolePattern::new(Distance::meters(length_factor * wavelength)).unwrap();
            let integral = simpsons_rule(0.0, PI, 1000, |theta| {
                d.gain_pattern(f, Angle::radians(theta)) * theta.sin()
            });
            let reference = 2.0 / integral;
            assert_approx_eq!(d.directivity(f), reference, rtol <= 1e-5);
        }
    }

    #[test]
    fn test_half_wave_directivity() {
        // Textbook value: D = 4/Cin(2π) ≈ 1.6409 (2.15 dBi)
        let d = half_wave_dipole();
        assert_approx_eq!(d.directivity(test_frequency()), 1.6409, atol <= 1e-3);
    }

    #[test]
    fn test_short_dipole_directivity_is_exactly_three_halves() {
        let wavelength = test_frequency().wavelength().to_meters();
        let d = DipolePattern::new(Distance::meters(wavelength / 100.0)).unwrap();
        assert_approx_eq!(d.directivity(test_frequency()), 1.5, atol <= 1e-12);
    }

    #[test]
    fn test_half_wave_dipole_at_zero() {
        // Gain along axis of dipole (θ=0) should be very low
        let d = half_wave_dipole();
        let gain = d.gain(test_frequency(), Angle::ZERO, Angle::ZERO);
        assert!(gain.as_f64() < -50.0);
    }

    #[test]
    fn test_half_wave_dipole_at_broadside() {
        // Half-wave dipole peak gain ≈ 2.15 dBi at θ=π/2
        let d = half_wave_dipole();
        let gain = d.gain(test_frequency(), Angle::radians(PI / 2.0), Angle::ZERO);
        assert_approx_eq!(gain.as_f64(), 2.15, atol <= 0.01);
    }

    #[test]
    fn test_short_dipole_peak() {
        // Short dipole (L = λ/100) peak gain ≈ 1.76 dBi
        let wavelength = test_frequency().wavelength().to_meters();
        let d = DipolePattern::new(Distance::meters(wavelength / 100.0)).unwrap();
        let peak = d.peak_gain(test_frequency());
        assert_approx_eq!(peak.as_f64(), 1.76, atol <= 0.1);
    }

    #[test]
    fn test_1_25_wavelength_dipole_peak() {
        // 1.25λ dipole peak gain ≈ 5.2 dBi
        let wavelength = test_frequency().wavelength().to_meters();
        let d = DipolePattern::new(Distance::meters(1.25 * wavelength)).unwrap();
        let peak = d.peak_gain(test_frequency());
        assert_approx_eq!(peak.as_f64(), 5.2, atol <= 0.1);
    }

    #[test]
    fn test_1_5_wavelength_dipole_at_45_degrees() {
        // 1.5λ dipole at 45° ≈ 3.5 dBi
        let wavelength = test_frequency().wavelength().to_meters();
        let d = DipolePattern::new(Distance::meters(1.5 * wavelength)).unwrap();
        let gain = d.gain(test_frequency(), Angle::degrees(45.0), Angle::ZERO);
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
