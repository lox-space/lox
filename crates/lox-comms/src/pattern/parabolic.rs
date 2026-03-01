// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Parabolic (uniform illuminated aperture) antenna gain pattern.
//!
//! Reference: Equation 17 of ALMA Memo 456.

use std::f64::consts::{FRAC_2_PI, PI};

use lox_core::units::{Angle, Decibel, Distance, Frequency};

use crate::antenna::AntennaGain;

/// Argument `u` at which the Airy disk pattern `|2·J₁(u)/u|²` equals 0.5 (−3 dB).
///
/// This is the first positive solution of `2·J₁(u) = u/√2`, i.e. the half-power point
/// used to compute the half-power beamwidth (HPBW) of a uniformly illuminated circular aperture.
const BESSEL_J1_HPBW: f64 = 1.616_330_8;

/// Threshold below which we treat `u` as zero to avoid division by zero.
const DIV_BY_ZERO_LIMIT: f64 = 1e-6;

/// Floor gain in linear representation (~-120 dB).
const MINF_GAIN_LINEAR: f64 = 1e-12;

/// Parabolic antenna gain pattern (uniform illuminated aperture).
#[derive(Debug, Clone)]
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

    /// Creates a parabolic pattern from a desired half-power beamwidth at a given frequency.
    ///
    /// `beamwidth` is the full HPBW (from −3 dB to +3 dB).
    /// Uses the exact Airy-disk inversion: `diameter = BESSEL_J1_HPBW · λ / (π · sin(HPBW/2))`.
    pub fn from_beamwidth(beamwidth: Angle, frequency: Frequency, efficiency: f64) -> Self {
        let wavelength_m = frequency.wavelength().to_meters();
        let half_bw = beamwidth.to_radians() / 2.0;
        let diameter_m = BESSEL_J1_HPBW * wavelength_m / (PI * half_bw.sin());
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

    fn beamwidth(&self, frequency: Frequency) -> Option<Angle> {
        let wavelength_m = frequency.wavelength().to_meters();
        let d = self.diameter.to_meters();
        let arg = BESSEL_J1_HPBW * wavelength_m / (PI * d);
        // When d < ~0.51λ the argument exceeds 1.0 and asin is undefined.
        if arg > 1.0 {
            None
        } else {
            // Full HPBW = 2 · arcsin(u_3dB · λ / (π · D))
            Some(Angle::radians(2.0 * arg.asin()))
        }
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

        let amplitude = (FRAC_2_PI / ax).sqrt();
        let result = amplitude * (xx.cos() * p1 - z * xx.sin() * q1);
        if x < 0.0 { -result } else { result }
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::FrequencyUnits;
    use lox_test_utils::assert_approx_eq;

    use super::*;

    /// First zero of J₁, used only to verify the Bessel approximation.
    const BESSEL_J1_FIRST_ZERO: f64 = 3.831_705_970_207_512;

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
    fn test_beamwidth_none_for_sub_wavelength_diameter() {
        let f = 1.0.ghz(); // λ ≈ 0.300 m
        let p = ParabolicPattern::new(Distance::meters(0.1), 0.65); // D ≈ 0.33λ < 0.51λ
        assert!(p.beamwidth(f).is_none());
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
        // D=0.98m, f=29GHz: full HPBW = 2·arcsin(1.6163308·λ/(π·D)) ≈ 0.6219°
        // (old first-null value was 0.7372°)
        let p = test_pattern();
        let bw = p.beamwidth(test_frequency()).unwrap();
        let expected = 2.0
            * (BESSEL_J1_HPBW * test_frequency().wavelength().to_meters()
                / (PI * p.diameter.to_meters()))
            .asin()
            .to_degrees();
        assert_approx_eq!(bw.to_degrees(), expected, rtol <= 1e-6);
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
    fn test_parabolic_3db_down_at_half_hpbw() {
        // The −3 dB point is at half the full HPBW.
        let p = test_pattern();
        let f = test_frequency();
        let half_bw = Angle::radians(p.beamwidth(f).unwrap().to_radians() / 2.0);
        let peak = p.peak_gain(f);
        let gain_at_half_bw = p.gain(f, half_bw);
        let diff = (peak - gain_at_half_bw).as_f64();
        assert_approx_eq!(diff, 3.0103, atol <= 0.5);
    }

    #[test]
    fn test_parabolic_from_beamwidth_roundtrip() {
        let beamwidth = Angle::radians(0.1);
        let f = 2.0.ghz();
        let p = ParabolicPattern::from_beamwidth(beamwidth, f, 0.65);
        let actual_bw = p.beamwidth(f).unwrap();
        assert_approx_eq!(actual_bw.to_radians(), beamwidth.to_radians(), rtol <= 0.01);
    }
}
