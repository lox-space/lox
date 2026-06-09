// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Radio transmitter models.

use lox_core::units::{Angle, Decibel, Frequency};

use crate::antenna::AntennaGain;

/// Transmitter characterised by output power, line loss, and back-off; combined with
/// an antenna on the [`CommunicationSystem`](crate::system::CommunicationSystem).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AmplifierTransmitter {
    /// Transmit frequency.
    pub frequency: Frequency,
    /// Transmit power in watts.
    pub power_w: f64,
    /// Feed/line loss.
    pub line_loss: Decibel,
    /// Output back-off.
    pub output_back_off: Decibel,
}

impl AmplifierTransmitter {
    /// Creates a new amplifier transmitter.
    pub fn new(
        frequency: Frequency,
        power_w: f64,
        line_loss: Decibel,
        output_back_off: Decibel,
    ) -> Self {
        Self {
            frequency,
            power_w,
            line_loss,
            output_back_off,
        }
    }

    /// Returns the Effective Isotropic Radiated Power (EIRP) in dBW.
    pub fn eirp(&self, antenna: &impl AntennaGain, angle: Angle) -> Decibel {
        antenna.gain(self.frequency, angle) + Decibel::from_linear(self.power_w)
            - self.line_loss
            - self.output_back_off
    }
}

/// Lumped transmitter specified by a single EIRP figure.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EirpTransmitter {
    /// Transmit frequency.
    pub frequency: Frequency,
    /// Effective isotropic radiated power in dBW.
    pub eirp: Decibel,
}

/// A transmitter.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[non_exhaustive]
pub enum Transmitter {
    /// Lumped transmitter with aggregate EIRP.
    Eirp(EirpTransmitter),
    /// Power-amplifier transmitter (combined with an external antenna).
    Amplifier(AmplifierTransmitter),
}

impl Transmitter {
    /// Returns the transmit frequency.
    pub fn frequency(&self) -> Frequency {
        match self {
            Transmitter::Eirp(t) => t.frequency,
            Transmitter::Amplifier(t) => t.frequency,
        }
    }

    /// Returns the EIRP in dBW for the given antenna at the given off-boresight angle.
    ///
    /// For the lumped `Eirp` variant, `antenna` and `angle` are ignored.
    pub fn eirp(&self, antenna: &impl AntennaGain, angle: Angle) -> Decibel {
        match self {
            Transmitter::Eirp(t) => t.eirp,
            Transmitter::Amplifier(t) => t.eirp(antenna, angle),
        }
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use crate::antenna::ConstantAntenna;

    use super::*;

    #[test]
    fn test_eirp_simple() {
        let antenna = ConstantAntenna { gain: 10.0.db() };
        let tx = AmplifierTransmitter::new(29.0.ghz(), 5.0, 1.0.db(), 0.0.db());
        let eirp = tx.eirp(&antenna, Angle::radians(0.0));
        assert_approx_eq!(eirp.as_f64(), 15.9897, atol <= 0.001);
    }

    #[test]
    fn test_eirp_with_obo() {
        let antenna = ConstantAntenna { gain: 20.0.db() };
        let tx = AmplifierTransmitter::new(29.0.ghz(), 10.0, 2.0.db(), 3.0.db());
        let eirp = tx.eirp(&antenna, Angle::radians(0.0));
        assert_approx_eq!(eirp.as_f64(), 25.0, atol <= 1e-10);
    }

    #[test]
    fn test_enum_dispatch_amplifier() {
        let antenna = ConstantAntenna { gain: 10.0.db() };
        let tx = Transmitter::Amplifier(AmplifierTransmitter::new(
            29.0.ghz(),
            5.0,
            1.0.db(),
            0.0.db(),
        ));
        let eirp = tx.eirp(&antenna, Angle::radians(0.0));
        assert_approx_eq!(eirp.as_f64(), 15.9897, atol <= 0.001);
        assert_eq!(tx.frequency().to_hertz(), 29e9);
    }

    #[test]
    fn test_enum_dispatch_eirp_ignores_antenna_and_angle() {
        let antenna = ConstantAntenna {
            gain: 1000.0.db(), // Deliberately absurd value
        };
        let tx = Transmitter::Eirp(EirpTransmitter {
            frequency: 29.0.ghz(),
            eirp: 55.0.db(),
        });
        // For Eirp, antenna gain and angle are ignored — the stored figure is returned verbatim.
        let eirp = tx.eirp(&antenna, Angle::radians(0.0));
        assert_approx_eq!(eirp.as_f64(), 55.0, atol <= 1e-10);
        // Same at any off-boresight angle
        let eirp_off = tx.eirp(&antenna, Angle::radians(1.0));
        assert_approx_eq!(eirp_off.as_f64(), 55.0, atol <= 1e-10);
        assert_eq!(tx.frequency().to_hertz(), 29e9);
    }
}
