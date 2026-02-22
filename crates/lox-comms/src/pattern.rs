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

    fn beamwidth(&self, frequency: Frequency) -> Angle {
        match self {
            AntennaPattern::Parabolic(p) => p.beamwidth(frequency),
            AntennaPattern::Gaussian(p) => p.beamwidth(frequency),
            AntennaPattern::Dipole(p) => p.beamwidth(frequency),
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
}
