// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Radio transmitter models.

use lox_core::units::Decibel;

use crate::band::FrequencyRange;

/// Transmitter output stage: amplifier power and back-off over a band.
///
/// Describes the RF output stage only. Feed/line losses belong to the
/// [`TxPort`](crate::payload::TxPort) wiring the transmitter to an antenna,
/// and lumped EIRP figures to [`EirpModel`](crate::payload::EirpModel).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AmplifierTransmitter {
    /// Supported frequency range.
    pub band: FrequencyRange,
    /// Transmit power in watts.
    pub power_w: f64,
    /// Output back-off.
    pub output_back_off: Decibel,
}

impl AmplifierTransmitter {
    /// Creates a new amplifier transmitter.
    pub fn new(band: FrequencyRange, power_w: f64, output_back_off: Decibel) -> Self {
        Self {
            band,
            power_w,
            output_back_off,
        }
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_amplifier_transmitter() {
        let band = FrequencyRange::new(27.0.ghz(), 31.0.ghz()).unwrap();
        let tx = AmplifierTransmitter::new(band, 10.0, 3.0.db());
        assert!(tx.band.contains(29.0.ghz()));
        assert_approx_eq!(tx.power_w, 10.0, atol <= 1e-15);
        assert_approx_eq!(tx.output_back_off.as_f64(), 3.0, atol <= 1e-15);
    }
}
