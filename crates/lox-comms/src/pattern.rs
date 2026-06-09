// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Antenna gain pattern models.

use lox_core::units::{Angle, Decibel, Frequency};

use crate::antenna::AntennaGain;

pub mod dipole;
pub mod gaussian;
pub mod parabolic;

pub use dipole::DipolePattern;
pub use gaussian::GaussianPattern;
pub use parabolic::ParabolicPattern;

/// An antenna gain pattern model.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[non_exhaustive]
pub enum AntennaPattern {
    /// Parabolic (uniform illuminated aperture) pattern.
    Parabolic(ParabolicPattern),
    /// Gaussian pattern.
    Gaussian(GaussianPattern),
    /// Dipole pattern.
    Dipole(DipolePattern),
}

impl AntennaGain for AntennaPattern {
    fn gain(&self, frequency: Frequency, angle: Angle) -> Decibel {
        match self {
            AntennaPattern::Parabolic(p) => p.gain(frequency, angle),
            AntennaPattern::Gaussian(p) => p.gain(frequency, angle),
            AntennaPattern::Dipole(p) => p.gain(frequency, angle),
        }
    }
}

impl AntennaPattern {
    /// Returns the peak gain in dBi at the given frequency.
    pub fn peak_gain(&self, frequency: Frequency) -> Decibel {
        match self {
            AntennaPattern::Parabolic(p) => p.peak_gain(frequency),
            AntennaPattern::Gaussian(p) => p.peak_gain(frequency),
            AntennaPattern::Dipole(p) => p.peak_gain(frequency),
        }
    }

    /// Returns the half-power beamwidth at the given frequency, or `None`
    /// when the underlying pattern does not define one (dipole) or the
    /// aperture is sub-wavelength (parabolic with `D < ~0.51·λ`).
    pub fn beamwidth(&self, frequency: Frequency) -> Option<Angle> {
        match self {
            AntennaPattern::Parabolic(p) => p.beamwidth(frequency),
            AntennaPattern::Gaussian(p) => Some(p.beamwidth(frequency)),
            AntennaPattern::Dipole(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::{Distance, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use crate::antenna::AntennaGain;

    use super::*;

    fn test_frequency() -> lox_core::units::Frequency {
        29.0.ghz()
    }

    #[test]
    fn test_pattern_enum_parabolic_dispatch() {
        let p = AntennaPattern::Parabolic(ParabolicPattern::new(Distance::meters(0.98), 0.45));
        let f = test_frequency();
        let gain = p.gain(f, lox_core::units::Angle::radians(0.0));
        let peak = p.peak_gain(f);
        assert_approx_eq!(gain.as_f64(), peak.as_f64(), atol <= 1e-10);
    }

    #[test]
    fn test_pattern_enum_gaussian_dispatch() {
        let p = AntennaPattern::Gaussian(GaussianPattern::new(Distance::meters(0.98), 0.45));
        let f = test_frequency();
        let gain = p.gain(f, lox_core::units::Angle::radians(0.0));
        let peak = p.peak_gain(f);
        assert_approx_eq!(gain.as_f64(), peak.as_f64(), atol <= 1e-10);
    }

    #[test]
    fn test_pattern_enum_dipole_dispatch() {
        let f = test_frequency();
        let wavelength = f.wavelength().to_meters();
        let p = AntennaPattern::Dipole(DipolePattern::new(Distance::meters(wavelength / 2.0)));
        // Broadside of a half-wave dipole — finite gain
        let gain = p.gain(
            f,
            lox_core::units::Angle::radians(std::f64::consts::PI / 2.0),
        );
        assert_approx_eq!(gain.as_f64(), 2.15, atol <= 0.01);
        let peak = p.peak_gain(f);
        // Peak gain is at broadside for a half-wave
        assert!(peak.as_f64() > 2.0);
    }
}
