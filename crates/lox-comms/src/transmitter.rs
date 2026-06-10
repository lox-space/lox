// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Radio transmitter models.

use lox_core::units::{Decibel, Power};

use crate::band::FrequencyRange;
use crate::error::NonPhysicalError;

/// Transmitter output stage: amplifier power and back-off over a band.
///
/// Describes the RF output stage only. Feed/line losses belong to the
/// [`TxPort`](crate::payload::TxPort) wiring the transmitter to an antenna,
/// and lumped EIRP figures to [`EirpModel`](crate::payload::EirpModel).
///
/// Valid by construction: transmit power is finite and positive, output
/// back-off finite and non-negative.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "AmplifierTransmitterRepr")
)]
pub struct AmplifierTransmitter {
    band: FrequencyRange,
    power: Power,
    output_back_off: Decibel,
}

/// Serde wire format for [`AmplifierTransmitter`]: forces deserialization
/// through the validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct AmplifierTransmitterRepr {
    band: FrequencyRange,
    power: Power,
    output_back_off: Decibel,
}

#[cfg(feature = "serde")]
impl TryFrom<AmplifierTransmitterRepr> for AmplifierTransmitter {
    type Error = NonPhysicalError;

    fn try_from(repr: AmplifierTransmitterRepr) -> Result<Self, Self::Error> {
        AmplifierTransmitter::new(repr.band, repr.power, repr.output_back_off)
    }
}

impl AmplifierTransmitter {
    /// Creates a new amplifier transmitter.
    ///
    /// Rejects non-physical parameters: transmit power must be finite and
    /// positive, output back-off finite and non-negative.
    pub fn new(
        band: FrequencyRange,
        power: Power,
        output_back_off: Decibel,
    ) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_positive("transmit power [W]", power.to_watts())?;
        NonPhysicalError::check_non_negative("output back-off [dB]", output_back_off.as_f64())?;
        Ok(Self {
            band,
            power,
            output_back_off,
        })
    }

    /// Returns the supported frequency range.
    pub fn band(&self) -> FrequencyRange {
        self.band
    }

    /// Returns the transmit power.
    pub fn power(&self) -> Power {
        self.power
    }

    /// Returns the output back-off.
    pub fn output_back_off(&self) -> Decibel {
        self.output_back_off
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use super::*;

    fn ka_band() -> FrequencyRange {
        FrequencyRange::new(27.0.ghz(), 31.0.ghz()).unwrap()
    }

    #[test]
    fn test_amplifier_transmitter() {
        let tx = AmplifierTransmitter::new(ka_band(), Power::watts(10.0), 3.0.db()).unwrap();
        assert!(tx.band().contains(29.0.ghz()));
        assert_approx_eq!(tx.power().to_watts(), 10.0, atol <= 1e-15);
        assert_approx_eq!(tx.output_back_off().as_f64(), 3.0, atol <= 1e-15);
    }

    #[test]
    fn test_amplifier_transmitter_rejects_non_physical() {
        for power in [0.0, -10.0, f64::NAN, f64::INFINITY] {
            assert!(AmplifierTransmitter::new(ka_band(), Power::watts(power), 0.0.db()).is_err());
        }
        assert!(AmplifierTransmitter::new(ka_band(), Power::watts(10.0), (-1.0).db()).is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_amplifier_transmitter_serde_rejects_invalid() {
        let tx = AmplifierTransmitter::new(ka_band(), Power::watts(10.0), 0.0.db()).unwrap();
        let json = serde_json::to_string(&tx).unwrap();
        let round_trip: AmplifierTransmitter = serde_json::from_str(&json).unwrap();
        assert_approx_eq!(round_trip.power().to_watts(), 10.0, atol <= 1e-15);

        let bad = json.replace("\"power\":10.0", "\"power\":-10.0");
        assert!(serde_json::from_str::<AmplifierTransmitter>(&bad).is_err());
    }
}
