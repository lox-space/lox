// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Radio receiver models (known noise temperature and N-stage cascade).
//!
//! Receivers describe the RF input stage from its input connector onward.
//! Feed losses and the clear-sky antenna noise temperature belong to the
//! [`RxPort`](crate::payload::RxPort) wiring the receiver to an antenna, and
//! lumped G/T figures to [`GtModel`](crate::payload::GtModel).

use lox_core::units::{Decibel, Kelvin};

use crate::ROOM_TEMPERATURE;
use crate::band::FrequencyRange;

/// Converts a noise figure in dB to an equivalent noise temperature in Kelvin.
///
/// T = T_room · (10^(NF/10) − 1)
pub fn noise_figure_to_temperature(nf: Decibel) -> Kelvin {
    ROOM_TEMPERATURE * (nf.to_linear() - 1.0)
}

/// A receiver characterised by a single equivalent noise temperature.
///
/// The figure is referred to the receiver's input connector, exactly like a
/// [`CascadeReceiver`]'s chain: the system noise temperature at the antenna
/// flange is assembled at link-budget setup as
/// `T_sys = T_ant + T_feed + T_rx / G_feed` from the port's antenna noise
/// temperature and feed loss. For a datasheet figure that already includes
/// the antenna and feed contributions, set both port values to zero.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NoiseTempReceiver {
    /// Supported frequency range.
    pub band: FrequencyRange,
    /// Equivalent noise temperature in Kelvin, referred to the receiver's
    /// input connector.
    pub noise_temperature: Kelvin,
}

/// A single stage in an RF receiver chain.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NoiseStage {
    /// Stage gain in dB.
    pub gain: Decibel,
    /// Stage equivalent noise temperature in Kelvin.
    pub noise_temperature: Kelvin,
}

/// An N-stage cascade receiver using the Friis noise formula.
///
/// The chain is described strictly from the receiver's input connector
/// onward; the antenna noise temperature and feed loss are supplied by the
/// port at link-budget setup.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CascadeReceiver {
    /// Supported frequency range.
    pub band: FrequencyRange,
    /// Ordered chain of RF stages (LNA first, then filters, mixers, etc.).
    pub stages: Vec<NoiseStage>,
    /// Demodulator implementation loss.
    pub demodulator_loss: Decibel,
    /// Other implementation losses.
    pub implementation_loss: Decibel,
}

impl CascadeReceiver {
    /// Creates a two-stage receiver model: LNA → receiver (from noise figure).
    pub fn from_lna_and_noise_figure(
        band: FrequencyRange,
        lna_gain: Decibel,
        lna_noise_temperature: Kelvin,
        receiver_noise_figure: Decibel,
        demodulator_loss: Decibel,
        implementation_loss: Decibel,
    ) -> Self {
        let lna_stage = NoiseStage {
            gain: lna_gain,
            noise_temperature: lna_noise_temperature,
        };
        let rx_stage = NoiseStage {
            gain: Decibel::new(0.0),
            noise_temperature: noise_figure_to_temperature(receiver_noise_figure),
        };
        Self {
            band,
            stages: vec![lna_stage, rx_stage],
            demodulator_loss,
            implementation_loss,
        }
    }

    /// Returns the chain's equivalent noise temperature in Kelvin, referred
    /// to its input connector, via the Friis formula.
    ///
    /// T_chain = T_1 + T_2/G_1 + T_3/(G_1·G_2) + ...
    pub fn chain_noise_temperature(&self) -> Kelvin {
        let mut t_chain = 0.0;
        let mut cumulative_gain_linear = 1.0;
        for stage in &self.stages {
            t_chain += stage.noise_temperature / cumulative_gain_linear;
            cumulative_gain_linear *= stage.gain.to_linear();
        }
        t_chain
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
            noise_figure_to_temperature(5.0.db()),
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
            175.0,
            2.0.db(),
            0.0.db(),
            0.0.db(),
        );
        assert_approx_eq!(rx.chain_noise_temperature(), 176.696, atol <= 0.01);
    }

    #[test]
    fn test_cascade_receiver_three_stage() {
        // Stages: LNA(G=20dB,T=50K), Filter(G=-3dB,T=290K), Rx(G=30dB,T=500K)
        let rx = CascadeReceiver {
            band: ka_band(),
            stages: vec![
                NoiseStage {
                    gain: 20.0.db(),
                    noise_temperature: 50.0,
                },
                NoiseStage {
                    gain: (-3.0).db(),
                    noise_temperature: 290.0,
                },
                NoiseStage {
                    gain: 30.0.db(),
                    noise_temperature: 500.0,
                },
            ],
            demodulator_loss: 0.0.db(),
            implementation_loss: 0.0.db(),
        };
        let g1 = 100.0_f64; // 10^(20/10)
        let g2 = 10.0_f64.powf(-3.0 / 10.0); // ~0.5012
        let expected = 50.0 + 290.0 / g1 + 500.0 / (g1 * g2);
        assert_approx_eq!(rx.chain_noise_temperature(), expected, rtol <= 1e-6);
    }

    #[test]
    fn test_cascade_receiver_chain_gain() {
        let rx = CascadeReceiver {
            band: ka_band(),
            stages: vec![
                NoiseStage {
                    gain: 20.0.db(),
                    noise_temperature: 50.0,
                },
                NoiseStage {
                    gain: (-3.0).db(),
                    noise_temperature: 290.0,
                },
                NoiseStage {
                    gain: 30.0.db(),
                    noise_temperature: 500.0,
                },
            ],
            demodulator_loss: 0.0.db(),
            implementation_loss: 0.0.db(),
        };
        // chain_gain = 20 + (-3) + 30 = 47 dB
        assert_approx_eq!(rx.chain_gain().as_f64(), 47.0, atol <= 1e-10);
    }

    #[test]
    fn test_receiver_band_accessor() {
        let rx = Receiver::NoiseTemperature(NoiseTempReceiver {
            band: ka_band(),
            noise_temperature: 500.0,
        });
        assert!(rx.band().contains(29.0.ghz()));
    }
}
