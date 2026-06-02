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

#[cfg(test)]
mod tests {
    use lox_core::glam::DVec3;
    use lox_core::units::{Angle, Decibel, DecibelUnits, Distance, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use crate::pattern::{AntennaPattern, ParabolicPattern};

    use super::*;

    fn parabolic() -> PatternedAntenna {
        PatternedAntenna {
            pattern: AntennaPattern::Parabolic(ParabolicPattern::new(Distance::meters(0.98), 0.45)),
            boresight: DVec3::Z,
        }
    }

    #[test]
    fn test_constant_antenna_gain_dispatch() {
        let a = ConstantAntenna {
            gain: 10.0.db(),
            beamwidth: Angle::degrees(5.0),
        };
        let g = a.gain(29.0.ghz(), Angle::radians(0.0));
        assert_approx_eq!(g.as_f64(), 10.0, atol <= 1e-10);
        assert_approx_eq!(
            a.beamwidth(29.0.ghz()).unwrap().to_degrees(),
            5.0,
            atol <= 1e-10
        );
    }

    #[test]
    fn test_patterned_antenna_gain_and_beamwidth() {
        let a = parabolic();
        let f = 29.0.ghz();
        let peak = a.peak_gain(f);
        let on_axis = a.gain(f, Angle::radians(0.0));
        // On-axis gain equals peak gain
        assert_approx_eq!(on_axis.as_f64(), peak.as_f64(), atol <= 1e-10);
        // Beamwidth is defined for the parabolic pattern
        assert!(a.beamwidth(f).is_some());
    }

    #[test]
    fn test_antenna_enum_constant_dispatch() {
        let a = Antenna::Constant(ConstantAntenna {
            gain: Decibel::new(20.0),
            beamwidth: Angle::degrees(2.0),
        });
        let g = a.gain(29.0.ghz(), Angle::radians(0.0));
        assert_approx_eq!(g.as_f64(), 20.0, atol <= 1e-10);
        assert_approx_eq!(
            a.beamwidth(29.0.ghz()).unwrap().to_degrees(),
            2.0,
            atol <= 1e-10
        );
    }

    #[test]
    fn test_antenna_enum_patterned_dispatch() {
        let a = Antenna::Patterned(parabolic());
        let f = 29.0.ghz();
        let on_axis = a.gain(f, Angle::radians(0.0));
        assert!(on_axis.as_f64() > 40.0);
        assert!(a.beamwidth(f).is_some());
    }
}
