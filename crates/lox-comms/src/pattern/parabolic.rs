// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Parabolic (uniform illuminated aperture) antenna gain pattern.
//!
//! Reference: Equation 17 of ALMA Memo 456.

use std::f64::consts::PI;

use lox_core::units::{Angle, Decibel, Distance, Frequency};

use crate::antenna::AntennaGain;

/// First zero of the Bessel function J₁.
const BESSEL_J1_FIRST_ZERO: f64 = 3.831_705_970_207_512;

/// Threshold below which we treat `u` as zero to avoid division by zero.
const DIV_BY_ZERO_LIMIT: f64 = 1e-6;

/// Floor gain in linear representation (~-120 dB).
const MINF_GAIN_LINEAR: f64 = 1e-12;

/// Parabolic antenna gain pattern (uniform illuminated aperture).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParabolicPattern {
    /// Antenna diameter.
    pub diameter: Distance,
    /// Aperture efficiency (0, 1].
    pub efficiency: f64,
}

impl ParabolicPattern {
    /// Creates a new parabolic pattern with the given diameter and efficiency.
    pub fn new(diameter: Distance, efficiency: f64) -> Self {
        Self {
            diameter,
            efficiency,
        }
    }

    /// Creates a parabolic pattern from a desired beamwidth at a given frequency.
    ///
    /// Uses `diameter = 1.22 · λ / beamwidth`.
    pub fn from_beamwidth(beamwidth: Angle, frequency: Frequency, efficiency: f64) -> Self {
        let wavelength_m = frequency.wavelength().to_meters();
        let diameter_m = 1.22 * wavelength_m / beamwidth.to_radians();
        Self {
            diameter: Distance::meters(diameter_m),
            efficiency,
        }
    }

    /// Returns the physical aperture area in m².
    pub fn area(&self) -> f64 {
        let d = self.diameter.to_meters();
        PI * d * d / 4.0
    }

    /// Returns the peak gain in dBi at the given frequency.
    pub fn peak_gain(&self, frequency: Frequency) -> Decibel {
        let wavelength_m = frequency.wavelength().to_meters();
        let aperture_gain = 4.0 * PI * self.area() / (wavelength_m * wavelength_m);
        Decibel::from_linear(aperture_gain) + Decibel::from_linear(self.efficiency)
    }
}

impl AntennaGain for ParabolicPattern {
    fn gain(&self, frequency: Frequency, angle: Angle) -> Decibel {
        let wavelength_m = frequency.wavelength().to_meters();
        let d = self.diameter.to_meters();
        let theta = angle.to_radians();
        let u = PI * d / wavelength_m * theta.sin();

        let pattern_loss_linear = if theta.cos() < (PI / 2.0).cos() {
            // Beyond hemisphere — floor gain
            MINF_GAIN_LINEAR
        } else if u.abs() < DIV_BY_ZERO_LIMIT {
            // On-axis — no pattern loss
            1.0
        } else {
            let j1u = bessel_j1(u);
            let ratio = 2.0 * j1u / u;
            ratio * ratio
        };

        self.peak_gain(frequency) + Decibel::from_linear(pattern_loss_linear)
    }

    fn beamwidth(&self, frequency: Frequency) -> Angle {
        let wavelength_m = frequency.wavelength().to_meters();
        let d = self.diameter.to_meters();
        let arg = BESSEL_J1_FIRST_ZERO * wavelength_m / (PI * d);
        Angle::radians(arg.asin())
    }
}

/// Bessel function of the first kind, order 1.
///
/// Uses polynomial approximation from Abramowitz and Stegun (9.4.4 and 9.4.6)
/// for |x| <= 3 and a phase-amplitude form for |x| > 3.
fn bessel_j1(x: f64) -> f64 {
    let ax = x.abs();
    if ax < 8.0 {
        // Rational approximation for |x| < 8
        let y = x * x;
        let num = x
            * (72362614232.0
                + y * (-7895059235.0
                    + y * (242396853.1
                        + y * (-2972611.439 + y * (15704.4826 + y * (-30.16036606))))));
        let den = 144725228442.0
            + y * (2300535178.0 + y * (18583304.74 + y * (99447.43394 + y * (376.9991397 + y))));
        num / den
    } else {
        // Hankel asymptotic expansion for |x| >= 8
        let z = 8.0 / ax;
        let y = z * z;
        let xx = ax - 2.356_194_490_192_345; // ax - 3*PI/4

        let p1 = 1.0
            + y * (0.183105e-2
                + y * (-0.3516396496e-4 + y * (0.2457520174e-5 + y * (-0.240337019e-6))));
        let q1 = 0.04687499995
            + y * (-0.2002690873e-3
                + y * (0.8449199096e-5 + y * (-0.88228987e-6 + y * 0.105787412e-6)));

        let amplitude = (0.636_619_772 / ax).sqrt();
        let result = amplitude * (xx.cos() * p1 - z * xx.sin() * q1);
        if x < 0.0 { -result } else { result }
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::FrequencyUnits;
    use lox_test_utils::assert_approx_eq;

    use super::*;

    fn test_frequency() -> Frequency {
        29.0.ghz()
    }

    fn test_pattern() -> ParabolicPattern {
        ParabolicPattern::new(Distance::meters(0.98), 0.45)
    }

    #[test]
    fn test_bessel_j1_at_zero() {
        assert_approx_eq!(bessel_j1(0.0), 0.0, atol <= 1e-15);
    }

    #[test]
    fn test_bessel_j1_known_values() {
        // J1(3.8317) ≈ 0 (first zero)
        assert_approx_eq!(bessel_j1(BESSEL_J1_FIRST_ZERO), 0.0, atol <= 1e-6);
        // J1(1.0) ≈ 0.44005058574...
        assert_approx_eq!(bessel_j1(1.0), 0.440_050_585_74, rtol <= 1e-8);
        // J1(5.0) ≈ -0.32757913759...
        assert_approx_eq!(bessel_j1(5.0), -0.327_579_137_59, rtol <= 1e-8);
        // J1(10.0) ≈ 0.04347274616...
        assert_approx_eq!(bessel_j1(10.0), 0.043_472_746_16, rtol <= 1e-6);
    }

    #[test]
    fn test_parabolic_beamwidth() {
        let p = test_pattern();
        let bw = p.beamwidth(test_frequency());
        let exp = Angle::degrees(0.7371800047831003);
        assert_approx_eq!(bw.to_radians(), exp.to_radians(), rtol <= 1e-6);
    }

    #[test]
    fn test_parabolic_peak_gain() {
        let p = test_pattern();
        let gain = p.gain(test_frequency(), Angle::radians(0.0));
        let peak = p.peak_gain(test_frequency());
        assert_approx_eq!(gain.as_f64(), 46.01119000490658, rtol <= 1e-6);
        assert_approx_eq!(peak.as_f64(), 46.01119000490658, rtol <= 1e-6);
    }

    #[test]
    fn test_parabolic_gain_at_pi() {
        let p = test_pattern();
        let gain = p.gain(test_frequency(), Angle::radians(std::f64::consts::PI));
        assert!(gain.as_f64() < -50.0);
    }

    #[test]
    fn test_parabolic_gain_at_hpbw() {
        let p = test_pattern();
        let bw = p.beamwidth(test_frequency());
        let gain = p.gain(test_frequency(), bw);
        assert!(gain.as_f64() < -100.0);
    }

    #[test]
    fn test_parabolic_from_beamwidth_roundtrip() {
        let beamwidth = Angle::radians(0.1);
        let f = 2.0.ghz();
        let p = ParabolicPattern::from_beamwidth(beamwidth, f, 0.65);
        let actual_bw = p.beamwidth(f);
        assert_approx_eq!(actual_bw.to_radians(), beamwidth.to_radians(), rtol <= 0.01);
    }
}
