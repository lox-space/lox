// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Antenna types and the [`AntennaGain`] trait.

use glam::DVec3;
use lox_core::units::{Angle, Decibel, Frequency};

use crate::pattern::AntennaPattern;

/// Trait for types that can compute antenna gain and beamwidth.
pub trait AntennaGain {
    /// Returns the antenna gain in dBi at the given frequency and off-boresight angle.
    fn gain(&self, frequency: Frequency, angle: Angle) -> Decibel;

    /// Returns the half-power beamwidth in radians at the given frequency.
    fn beamwidth(&self, frequency: Frequency) -> Angle;
}

/// A simple antenna with constant gain and beamwidth.
pub struct SimpleAntenna {
    /// Peak gain in dBi.
    pub gain: Decibel,
    /// Half-power beamwidth.
    pub beamwidth: Angle,
}

impl AntennaGain for SimpleAntenna {
    fn gain(&self, _frequency: Frequency, _angle: Angle) -> Decibel {
        self.gain
    }

    fn beamwidth(&self, _frequency: Frequency) -> Angle {
        self.beamwidth
    }
}

/// An antenna with a physics-based gain pattern and a boresight vector.
pub struct ComplexAntenna {
    /// The gain pattern model.
    pub pattern: AntennaPattern,
    /// Boresight direction vector (unit vector in the antenna's local frame).
    pub boresight: DVec3,
}

impl AntennaGain for ComplexAntenna {
    fn gain(&self, frequency: Frequency, angle: Angle) -> Decibel {
        self.pattern.gain(frequency, angle)
    }

    fn beamwidth(&self, frequency: Frequency) -> Angle {
        self.pattern.beamwidth(frequency)
    }
}

impl ComplexAntenna {
    /// Returns the peak gain in dBi at the given frequency.
    pub fn peak_gain(&self, frequency: Frequency) -> Decibel {
        self.pattern.peak_gain(frequency)
    }
}

/// An antenna, either simple (constant gain) or complex (pattern-based).
pub enum Antenna {
    Simple(SimpleAntenna),
    Complex(ComplexAntenna),
}

impl AntennaGain for Antenna {
    fn gain(&self, frequency: Frequency, angle: Angle) -> Decibel {
        match self {
            Antenna::Simple(a) => a.gain(frequency, angle),
            Antenna::Complex(a) => a.gain(frequency, angle),
        }
    }

    fn beamwidth(&self, frequency: Frequency) -> Angle {
        match self {
            Antenna::Simple(a) => a.beamwidth(frequency),
            Antenna::Complex(a) => a.beamwidth(frequency),
        }
    }
}
