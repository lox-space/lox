// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Antenna types and the [`AntennaGain`] trait.

use lox_core::glam::DVec3;
use lox_core::units::{Angle, Decibel, Frequency};

use crate::pattern::AntennaPattern;

/// Trait for types that can compute antenna gain and beamwidth.
pub trait AntennaGain {
    /// Returns the antenna gain in dBi at the given frequency and off-boresight angle.
    fn gain(&self, frequency: Frequency, angle: Angle) -> Decibel;

    /// Returns the half-power beamwidth at the given frequency, or `None` when
    /// the beamwidth is not well-defined for this antenna type or configuration.
    fn beamwidth(&self, frequency: Frequency) -> Option<Angle>;
}

/// Antenna with angle-independent gain and a nominal beamwidth.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstantAntenna {
    /// Peak gain in dBi.
    pub gain: Decibel,
    /// Half-power beamwidth.
    pub beamwidth: Angle,
}

impl AntennaGain for ConstantAntenna {
    fn gain(&self, _frequency: Frequency, _angle: Angle) -> Decibel {
        self.gain
    }

    fn beamwidth(&self, _frequency: Frequency) -> Option<Angle> {
        Some(self.beamwidth)
    }
}

/// An antenna with a physics-based gain pattern and a boresight vector.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatternedAntenna {
    /// The gain pattern model.
    pub pattern: AntennaPattern,
    /// Boresight direction vector (unit vector in the antenna's local frame).
    pub boresight: DVec3,
}

impl AntennaGain for PatternedAntenna {
    fn gain(&self, frequency: Frequency, angle: Angle) -> Decibel {
        self.pattern.gain(frequency, angle)
    }

    fn beamwidth(&self, frequency: Frequency) -> Option<Angle> {
        self.pattern.beamwidth(frequency)
    }
}

impl PatternedAntenna {
    /// Returns the peak gain in dBi at the given frequency.
    pub fn peak_gain(&self, frequency: Frequency) -> Decibel {
        self.pattern.peak_gain(frequency)
    }
}

/// An antenna, either constant-gain or pattern-based.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[non_exhaustive]
pub enum Antenna {
    /// Constant-gain antenna.
    Constant(ConstantAntenna),
    /// Pattern-based antenna with boresight direction.
    Patterned(PatternedAntenna),
}

impl AntennaGain for Antenna {
    fn gain(&self, frequency: Frequency, angle: Angle) -> Decibel {
        match self {
            Antenna::Constant(a) => a.gain(frequency, angle),
            Antenna::Patterned(a) => a.gain(frequency, angle),
        }
    }

    fn beamwidth(&self, frequency: Frequency) -> Option<Angle> {
        match self {
            Antenna::Constant(a) => a.beamwidth(frequency),
            Antenna::Patterned(a) => a.beamwidth(frequency),
        }
    }
}
