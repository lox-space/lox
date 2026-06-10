// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Radio receiver models (known noise temperature and N-stage cascade).
//!
//! Receivers describe the RF input stage from its input connector onward.
//! Feed losses and the clear-sky antenna noise temperature belong to the
//! [`RxPort`](crate::payload::RxPort) wiring the receiver to an antenna, and
//! lumped G/T figures to [`GtModel`](crate::payload::GtModel).

use lox_core::units::{Decibel, Temperature};

use crate::ROOM_TEMPERATURE;
use crate::band::FrequencyRange;
use crate::error::NonPhysicalError;

/// Converts a noise figure in dB to an equivalent noise temperature in Kelvin.
///
/// T = T_room · (10^(NF/10) − 1)
pub fn noise_figure_to_temperature(nf: Decibel) -> Temperature {
    Temperature::kelvin(ROOM_TEMPERATURE.to_kelvin() * (nf.to_linear() - 1.0))
}

/// A receiver characterised by a single equivalent noise temperature.
///
/// The figure is referred to the receiver's input connector, exactly like a
/// [`CascadeReceiver`]'s chain: the system noise temperature at the antenna
/// flange is assembled at link-budget setup as
/// `T_sys = T_ant + T_feed + T_rx / G_feed` from the port's antenna noise
/// temperature and feed loss. For a datasheet figure that already includes
/// the antenna and feed contributions, set both port values to zero.
///
/// Valid by construction: the noise temperature is finite and positive.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "NoiseTempReceiverRepr")
)]
pub struct NoiseTempReceiver {
    band: FrequencyRange,
    noise_temperature: Temperature,
}

/// Serde wire format for [`NoiseTempReceiver`]: forces deserialization
/// through the validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct NoiseTempReceiverRepr {
    band: FrequencyRange,
    noise_temperature: Temperature,
}

#[cfg(feature = "serde")]
impl TryFrom<NoiseTempReceiverRepr> for NoiseTempReceiver {
    type Error = NonPhysicalError;

    fn try_from(repr: NoiseTempReceiverRepr) -> Result<Self, Self::Error> {
        NoiseTempReceiver::new(repr.band, repr.noise_temperature)
    }
}

impl NoiseTempReceiver {
    /// Creates a new known-noise-temperature receiver.
    ///
    /// Rejects a non-finite or non-positive noise temperature.
    pub fn new(
        band: FrequencyRange,
        noise_temperature: Temperature,
    ) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_positive(
            "receiver noise temperature [K]",
            noise_temperature.to_kelvin(),
        )?;
        Ok(Self {
            band,
            noise_temperature,
        })
    }

    /// Returns the supported frequency range.
    pub fn band(&self) -> FrequencyRange {
        self.band
    }

    /// Returns the equivalent noise temperature, referred to the receiver's
    /// input connector.
    pub fn noise_temperature(&self) -> Temperature {
        self.noise_temperature
    }
}

/// A single stage in an RF receiver chain.
///
/// Valid by construction: the noise temperature is finite and non-negative,
/// the gain finite.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "NoiseStageRepr")
)]
pub struct NoiseStage {
    gain: Decibel,
    noise_temperature: Temperature,
}

/// Serde wire format for [`NoiseStage`]: forces deserialization through the
/// validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct NoiseStageRepr {
    gain: Decibel,
    noise_temperature: Temperature,
}

#[cfg(feature = "serde")]
impl TryFrom<NoiseStageRepr> for NoiseStage {
    type Error = NonPhysicalError;

    fn try_from(repr: NoiseStageRepr) -> Result<Self, Self::Error> {
        NoiseStage::new(repr.gain, repr.noise_temperature)
    }
}

impl NoiseStage {
    /// Creates a new RF chain stage.
    ///
    /// Rejects a non-finite gain and a non-finite or negative noise
    /// temperature.
    pub fn new(gain: Decibel, noise_temperature: Temperature) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_finite("stage gain [dB]", gain.as_f64())?;
        NonPhysicalError::check_non_negative(
            "stage noise temperature [K]",
            noise_temperature.to_kelvin(),
        )?;
        Ok(Self {
            gain,
            noise_temperature,
        })
    }

    /// Returns the stage gain in dB.
    pub fn gain(&self) -> Decibel {
        self.gain
    }

    /// Returns the stage equivalent noise temperature.
    pub fn noise_temperature(&self) -> Temperature {
        self.noise_temperature
    }
}

/// An N-stage cascade receiver using the Friis noise formula.
///
/// The chain is described strictly from the receiver's input connector
/// onward; the antenna noise temperature and feed loss are supplied by the
/// port at link-budget setup.
///
/// Valid by construction: stages uphold [`NoiseStage`]'s invariants and the
/// demodulator/implementation losses are finite and non-negative.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "CascadeReceiverRepr")
)]
pub struct CascadeReceiver {
    band: FrequencyRange,
    stages: Vec<NoiseStage>,
    demodulator_loss: Decibel,
    implementation_loss: Decibel,
}

/// Serde wire format for [`CascadeReceiver`]: forces deserialization through
/// the validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct CascadeReceiverRepr {
    band: FrequencyRange,
    stages: Vec<NoiseStage>,
    demodulator_loss: Decibel,
    implementation_loss: Decibel,
}

#[cfg(feature = "serde")]
impl TryFrom<CascadeReceiverRepr> for CascadeReceiver {
    type Error = NonPhysicalError;

    fn try_from(repr: CascadeReceiverRepr) -> Result<Self, Self::Error> {
        CascadeReceiver::new(
            repr.band,
            repr.stages,
            repr.demodulator_loss,
            repr.implementation_loss,
        )
    }
}

impl CascadeReceiver {
    /// Creates a new cascade receiver from an ordered chain of stages
    /// (LNA first, then filters, mixers, etc.).
    ///
    /// Rejects non-finite or negative demodulator/implementation losses.
    pub fn new(
        band: FrequencyRange,
        stages: Vec<NoiseStage>,
        demodulator_loss: Decibel,
        implementation_loss: Decibel,
    ) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_non_negative("demodulator loss [dB]", demodulator_loss.as_f64())?;
        NonPhysicalError::check_non_negative(
            "implementation loss [dB]",
            implementation_loss.as_f64(),
        )?;
        Ok(Self {
            band,
            stages,
            demodulator_loss,
            implementation_loss,
        })
    }

    /// Creates a two-stage receiver model: LNA → receiver (from noise figure).
    pub fn from_lna_and_noise_figure(
        band: FrequencyRange,
        lna_gain: Decibel,
        lna_noise_temperature: Temperature,
        receiver_noise_figure: Decibel,
        demodulator_loss: Decibel,
        implementation_loss: Decibel,
    ) -> Result<Self, NonPhysicalError> {
        let lna_stage = NoiseStage::new(lna_gain, lna_noise_temperature)?;
        let rx_stage = NoiseStage::new(
            Decibel::new(0.0),
            noise_figure_to_temperature(receiver_noise_figure),
        )?;
        Self::new(
            band,
            vec![lna_stage, rx_stage],
            demodulator_loss,
            implementation_loss,
        )
    }

    /// Returns the supported frequency range.
    pub fn band(&self) -> FrequencyRange {
        self.band
    }

    /// Returns the ordered chain of RF stages.
    pub fn stages(&self) -> &[NoiseStage] {
        &self.stages
    }

    /// Returns the demodulator implementation loss.
    pub fn demodulator_loss(&self) -> Decibel {
        self.demodulator_loss
    }

    /// Returns the other implementation losses.
    pub fn implementation_loss(&self) -> Decibel {
        self.implementation_loss
    }

    /// Returns the chain's equivalent noise temperature, referred to its
    /// input connector, via the Friis formula.
    ///
    /// T_chain = T_1 + T_2/G_1 + T_3/(G_1·G_2) + ...
    pub fn chain_noise_temperature(&self) -> Temperature {
        let mut t_chain = 0.0;
        let mut cumulative_gain_linear = 1.0;
        for stage in &self.stages {
            t_chain += stage.noise_temperature.to_kelvin() / cumulative_gain_linear;
            cumulative_gain_linear *= stage.gain.to_linear();
        }
        Temperature::kelvin(t_chain)
    }

    /// Returns the total RF chain gain in dB (sum of stage gains).
    pub fn chain_gain(&self) -> Decibel {
        self.stages
            .iter()
            .fold(Decibel::new(0.0), |acc, s| acc + s.gain)
    }
}

/// A component-tier receiver: known T_sys or an N-stage Friis cascade.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[non_exhaustive]
pub enum Receiver {
    /// Receiver with a known system noise temperature.
    NoiseTemperature(NoiseTempReceiver),
    /// Receiver with an N-stage cascade noise model.
    Cascade(CascadeReceiver),
}

impl Receiver {
    /// Returns the supported frequency range.
    pub fn band(&self) -> FrequencyRange {
        match self {
            Receiver::NoiseTemperature(r) => r.band,
            Receiver::Cascade(r) => r.band,
        }
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use super::*;

    fn ka_band() -> FrequencyRange {
        FrequencyRange::new(17.0.ghz(), 31.0.ghz()).unwrap()
    }

    #[test]
    fn test_noise_figure_to_temperature() {
        // NF = 5 dB → T = 290 * (10^(5/10) - 1) = 290 * 2.16228 = 627.06
        assert_approx_eq!(
            noise_figure_to_temperature(5.0.db()).to_kelvin(),
            627.0605214,
            rtol <= 1e-6
        );
    }

    #[test]
    fn test_from_lna_and_noise_figure() {
        // LNA(G=20dB, T=175K), Rx(NF=2dB)
        // T_rx = 290*(10^(2/10)-1) = 169.619 K
        // T_chain = 175 + 169.619/100 = 176.696 K
        let rx = CascadeReceiver::from_lna_and_noise_figure(
            ka_band(),
            20.0.db(),
            Temperature::kelvin(175.0),
            2.0.db(),
            0.0.db(),
            0.0.db(),
        )
        .unwrap();
        assert_approx_eq!(
            rx.chain_noise_temperature().to_kelvin(),
            176.696,
            atol <= 0.01
        );
    }

    #[test]
    fn test_cascade_receiver_three_stage() {
        // Stages: LNA(G=20dB,T=50K), Filter(G=-3dB,T=290K), Rx(G=30dB,T=500K)
        let rx = CascadeReceiver::new(
            ka_band(),
            vec![
                NoiseStage::new(20.0.db(), Temperature::kelvin(50.0)).unwrap(),
                NoiseStage::new((-3.0).db(), Temperature::kelvin(290.0)).unwrap(),
                NoiseStage::new(30.0.db(), Temperature::kelvin(500.0)).unwrap(),
            ],
            0.0.db(),
            0.0.db(),
        )
        .unwrap();
        let g1 = 100.0_f64; // 10^(20/10)
        let g2 = 10.0_f64.powf(-3.0 / 10.0); // ~0.5012
        let expected = 50.0 + 290.0 / g1 + 500.0 / (g1 * g2);
        assert_approx_eq!(
            rx.chain_noise_temperature().to_kelvin(),
            expected,
            rtol <= 1e-6
        );
    }

    #[test]
    fn test_cascade_receiver_chain_gain() {
        let rx = CascadeReceiver::new(
            ka_band(),
            vec![
                NoiseStage::new(20.0.db(), Temperature::kelvin(50.0)).unwrap(),
                NoiseStage::new((-3.0).db(), Temperature::kelvin(290.0)).unwrap(),
                NoiseStage::new(30.0.db(), Temperature::kelvin(500.0)).unwrap(),
            ],
            0.0.db(),
            0.0.db(),
        )
        .unwrap();
        // chain_gain = 20 + (-3) + 30 = 47 dB
        assert_approx_eq!(rx.chain_gain().as_f64(), 47.0, atol <= 1e-10);
    }

    #[test]
    fn test_receiver_band_accessor() {
        let rx = Receiver::NoiseTemperature(
            NoiseTempReceiver::new(ka_band(), Temperature::kelvin(500.0)).unwrap(),
        );
        assert!(rx.band().contains(29.0.ghz()));
    }

    #[test]
    fn test_receivers_reject_non_physical_inputs() {
        for temperature in [0.0, -10.0, f64::NAN] {
            assert!(NoiseTempReceiver::new(ka_band(), Temperature::kelvin(temperature)).is_err());
        }
        assert!(NoiseStage::new(Decibel::new(f64::NAN), Temperature::kelvin(50.0)).is_err());
        assert!(NoiseStage::new(20.0.db(), Temperature::kelvin(-1.0)).is_err());
        assert!(CascadeReceiver::new(ka_band(), vec![], (-1.0).db(), 0.0.db()).is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_receiver_serde_rejects_invalid() {
        let rx = NoiseTempReceiver::new(ka_band(), Temperature::kelvin(500.0)).unwrap();
        let json = serde_json::to_string(&rx).unwrap();
        assert!(serde_json::from_str::<NoiseTempReceiver>(&json).is_ok());

        let bad = json.replace(
            "\"noise_temperature\":500.0",
            "\"noise_temperature\":-500.0",
        );
        assert!(serde_json::from_str::<NoiseTempReceiver>(&bad).is_err());
    }
}
