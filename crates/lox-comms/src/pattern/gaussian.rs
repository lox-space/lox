// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Gaussian antenna gain pattern.
//!
//! Reference: MATLAB `gaussianAntenna`.

use std::f64::consts::PI;

use lox_core::units::{Angle, Decibel, Distance, Frequency};

use crate::antenna::AntennaGain;

/// Gaussian antenna gain pattern.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GaussianPattern {
    /// Antenna diameter.
    pub diameter: Distance,
    /// Aperture efficiency (0, 1].
    pub efficiency: f64,
}

impl GaussianPattern {
    /// Creates a new Gaussian pattern with the given diameter and efficiency.
    pub fn new(diameter: Distance, efficiency: f64) -> Self {
        Self {
            diameter,
            efficiency,
        }
    }

    /// Returns the peak gain in dBi at the given frequency.
    pub fn peak_gain(&self, frequency: Frequency) -> Decibel {
        let wavelength_m = frequency.wavelength().to_meters();
        let d = self.diameter.to_meters();
        let gain_linear = self.efficiency * (PI * d / wavelength_m).powi(2);
        Decibel::from_linear(gain_linear)
    }
}

impl AntennaGain for GaussianPattern {
    fn gain(&self, frequency: Frequency, angle: Angle) -> Decibel {
        let theta = angle.to_radians();
        let bw = self.beamwidth(frequency).to_radians();
        // Gaussian roll-off: G = G_peak · exp(-4·ln(2)·(θ/θ_3dB)²)
        let exponent = -4.0 * 2.0_f64.ln() * (theta / bw).powi(2);
        self.peak_gain(frequency) + Decibel::from_linear(exponent.exp())
    }

    fn beamwidth(&self, frequency: Frequency) -> Angle {
        let wavelength_m = frequency.wavelength().to_meters();
        let d = self.diameter.to_meters();
        // θ_3dB = 70·λ/D (in degrees), convert to radians
        Angle::degrees(70.0 * wavelength_m / d)
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

    fn test_pattern() -> GaussianPattern {
        GaussianPattern::new(Distance::meters(0.98), 0.45)
    }

    #[test]
    fn test_gaussian_peak_gain_matches_parabolic() {
        // Gaussian and parabolic share the same peak gain formula:
        // G = η·(π·D/λ)²
        let p = test_pattern();
        let peak = p.peak_gain(test_frequency());
        // Same parameters as parabolic test: D=0.98m, η=0.45, f=29GHz
        assert_approx_eq!(peak.as_f64(), 46.01119000490658, rtol <= 1e-6);
    }

    #[test]
    fn test_gaussian_on_axis_equals_peak() {
        let p = test_pattern();
        let gain = p.gain(test_frequency(), Angle::radians(0.0));
        let peak = p.peak_gain(test_frequency());
        assert_approx_eq!(gain.as_f64(), peak.as_f64(), atol <= 1e-10);
    }

    #[test]
    fn test_gaussian_3db_down_at_half_beamwidth() {
        // HPBW is the full width; the -3dB point is at θ = HPBW/2
        let p = test_pattern();
        let f = test_frequency();
        let half_bw = Angle::radians(p.beamwidth(f).to_radians() / 2.0);
        let peak = p.peak_gain(f);
        let gain_at_half_bw = p.gain(f, half_bw);
        let diff = peak.as_f64() - gain_at_half_bw.as_f64();
        assert_approx_eq!(diff, 3.0103, atol <= 0.01);
    }

    #[test]
    fn test_gaussian_symmetric() {
        let p = test_pattern();
        let f = test_frequency();
        let angle = Angle::degrees(1.0);
        let gain_pos = p.gain(f, angle);
        let gain_neg = p.gain(f, Angle::degrees(-1.0));
        assert_approx_eq!(gain_pos.as_f64(), gain_neg.as_f64(), atol <= 1e-10);
    }

    #[test]
    fn test_gaussian_monotonic_decrease() {
        let p = test_pattern();
        let f = test_frequency();
        let g0 = p.gain(f, Angle::degrees(0.0));
        let g1 = p.gain(f, Angle::degrees(0.5));
        let g2 = p.gain(f, Angle::degrees(1.0));
        let g3 = p.gain(f, Angle::degrees(2.0));
        assert!(g0.as_f64() > g1.as_f64());
        assert!(g1.as_f64() > g2.as_f64());
        assert!(g2.as_f64() > g3.as_f64());
    }
}
