// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Radio transmitter model.

use lox_core::units::{Angle, Decibel, Frequency};

use crate::antenna::AntennaGain;

/// A radio transmitter.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transmitter {
    /// Transmit frequency.
    pub frequency: Frequency,
    /// Transmit power in watts.
    pub power_w: f64,
    /// Feed/line loss.
    pub line_loss: Decibel,
    /// Output back-off.
    pub output_back_off: Decibel,
}

impl Transmitter {
    /// Creates a new transmitter with the given parameters.
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
    ///
    /// EIRP = G_tx(f, θ) + 10·log₁₀(P_w) − L_line − OBO
    pub fn eirp(&self, antenna: &impl AntennaGain, angle: Angle) -> Decibel {
        antenna.gain(self.frequency, angle) + Decibel::from_linear(self.power_w)
            - self.line_loss
            - self.output_back_off
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use crate::antenna::SimpleAntenna;

    use super::*;

    #[test]
    fn test_eirp_simple() {
        // 10 dBi antenna, 5 W power, 1 dB line loss, 0 dB OBO
        // EIRP = 10 + 10*log10(5) - 1 - 0 = 10 + 6.9897 - 1 = 15.9897 dBW
        let antenna = SimpleAntenna {
            gain: 10.0.db(),
            beamwidth: Angle::degrees(10.0),
        };
        let tx = Transmitter::new(29.0.ghz(), 5.0, 1.0.db(), 0.0.db());
        let eirp = tx.eirp(&antenna, Angle::radians(0.0));
        assert_approx_eq!(eirp.as_f64(), 15.9897, atol <= 0.001);
    }

    #[test]
    fn test_eirp_with_obo() {
        // 20 dBi antenna, 10 W power, 2 dB line loss, 3 dB OBO
        // EIRP = 20 + 10*log10(10) - 2 - 3 = 20 + 10 - 2 - 3 = 25 dBW
        let antenna = SimpleAntenna {
            gain: 20.0.db(),
            beamwidth: Angle::degrees(5.0),
        };
        let tx = Transmitter::new(29.0.ghz(), 10.0, 2.0.db(), 3.0.db());
        let eirp = tx.eirp(&antenna, Angle::radians(0.0));
        assert_approx_eq!(eirp.as_f64(), 25.0, atol <= 1e-10);
    }
}
