// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Link budget types: environmental losses, interference, and link statistics.

use lox_core::units::{Angle, Decibel, Distance, Frequency};

use crate::LinkBudgetError;
use crate::channel::Channel;
use crate::system::CommunicationSystem;
use crate::utils::free_space_path_loss;

pub use lox_itur::EnvironmentalLosses;

/// Interference statistics for a link.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InterferenceStats {
    /// Interference power in watts.
    pub interference_power_w: f64,
    /// Carrier-to-noise-plus-interference density ratio.
    pub c_n0i0: Decibel,
    /// Eb/(N0+I0).
    pub eb_n0i0: Decibel,
    /// Link margin with interference.
    pub margin_with_interference: Decibel,
}

/// Modulation-agnostic link budget output.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LinkStats {
    /// Slant range between TX and RX.
    pub slant_range: Distance,
    /// Link frequency.
    pub frequency: Frequency,
    /// Free-space path loss.
    pub fspl: Decibel,
    /// EIRP of the transmitter.
    pub eirp: Decibel,
    /// Receiver G/T.
    pub gt: Decibel,
    /// Environmental losses.
    pub losses: EnvironmentalLosses,
    /// Received carrier power. `None` for lumped-`Gt` receivers.
    pub carrier_rx_power: Option<Decibel>,
    /// Noise power in the channel bandwidth. `None` for lumped-`Gt` receivers.
    pub noise_power: Option<Decibel>,
    /// Channel noise bandwidth.
    pub bandwidth: Frequency,
    /// Carrier-to-noise density ratio (C/N₀).
    pub c_n0: Decibel,
    /// Carrier-to-noise ratio (C/N).
    pub c_n: Decibel,
    /// Off-boresight angle at the transmitter (0 for lumped `Eirp`).
    pub tx_angle: Angle,
    /// Off-boresight angle at the receiver (0 for lumped `Gt`).
    pub rx_angle: Angle,
}

impl LinkStats {
    /// Computes a modulation-agnostic link budget.
    ///
    /// `bandwidth` is the noise bandwidth used to compute `noise_power` and `C/N` from
    /// `C/N₀`. It is independent of any modulation scheme.
    pub fn calculate(
        tx_system: &CommunicationSystem,
        rx_system: &CommunicationSystem,
        range: Distance,
        bandwidth: Frequency,
        losses: EnvironmentalLosses,
        tx_angle: Angle,
        rx_angle: Angle,
    ) -> Result<Self, LinkBudgetError> {
        let env_loss = losses.total();

        let c_n0 =
            tx_system.carrier_to_noise_density(rx_system, env_loss, range, tx_angle, rx_angle)?;
        let carrier_rx_power =
            tx_system.carrier_power(rx_system, env_loss, range, tx_angle, rx_angle)?;
        let noise_power = rx_system.noise_power(bandwidth.to_hertz())?;

        let tx = tx_system
            .transmitter
            .as_ref()
            .ok_or(LinkBudgetError::MissingTransmitter)?;
        let frequency = tx.frequency();
        let fspl = free_space_path_loss(range, frequency);

        let eirp = tx_system.eirp_at(tx_angle)?;
        let gt = rx_system.gt_at(rx_angle)?;

        let c_n = c_n0 - Decibel::from_linear(bandwidth.to_hertz());

        Ok(Self {
            slant_range: range,
            frequency,
            fspl,
            eirp,
            gt,
            losses,
            carrier_rx_power,
            noise_power,
            bandwidth,
            c_n0,
            c_n,
            tx_angle,
            rx_angle,
        })
    }
}

/// Link-budget output with modulation/coding figures applied.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModulatedLinkStats {
    /// The modulation-agnostic link budget.
    pub link: LinkStats,
    /// The channel (modulation, FEC, required Eb/N₀, margin) applied.
    pub channel: Channel,
    /// Symbol rate from the channel.
    pub symbol_rate: Frequency,
    /// Es/N0 (energy per symbol to noise spectral density).
    pub es_n0: Decibel,
    /// Eb/N0 (energy per information bit to noise spectral density).
    pub eb_n0: Decibel,
    /// Link margin.
    pub margin: Decibel,
    /// Interference statistics (if applicable).
    pub interference: Option<InterferenceStats>,
}

impl ModulatedLinkStats {
    /// Returns interference statistics for the given interferer power.
    ///
    /// For lumped-`Gt` links, the underlying noise power is unknown, so interference
    /// is treated as the dominant noise contribution (noise floor = 0). The resulting
    /// figures are upper-bound estimates rather than exact values.
    pub fn with_interference(&self, interference_power_w: f64) -> InterferenceStats {
        let noise_linear = self.link.noise_power.map(|n| n.to_linear()).unwrap_or(0.0);
        let carrier = self.link.carrier_rx_power.unwrap_or_else(|| {
            // Synthesise a carrier power from C/N0 + bandwidth + noise so the
            // interference arithmetic stays self-consistent. Only used for lumped
            // links where noise is treated as zero in the absence of T_sys.
            self.link.c_n0 + Decibel::from_linear(noise_linear.max(1e-30))
        });

        let total_ni = noise_linear + interference_power_w;
        let c_n0i0 = carrier - Decibel::from_linear(total_ni)
            + Decibel::from_linear(self.link.bandwidth.to_hertz());
        let c_n0_to_eb_n0 = self.eb_n0 - self.link.c_n0;
        let eb_n0i0 = c_n0i0 + c_n0_to_eb_n0;

        let threshold = self.eb_n0 - self.margin;
        let margin_with_interference = eb_n0i0 - threshold;

        InterferenceStats {
            interference_power_w,
            c_n0i0,
            eb_n0i0,
            margin_with_interference,
        }
    }
}

/// Computes the frequency overlap factor between a receiver and an interfering transmitter.
///
/// Returns a value in [0, 1] representing the fraction of the interferer's bandwidth
/// that falls within the receiver's passband.
pub fn frequency_overlap_factor(rx_freq: f64, rx_bw: f64, tx_freq: f64, tx_bw: f64) -> f64 {
    let rx_lo = rx_freq - rx_bw / 2.0;
    let rx_hi = rx_freq + rx_bw / 2.0;
    let tx_lo = tx_freq - tx_bw / 2.0;
    let tx_hi = tx_freq + tx_bw / 2.0;

    let overlap = (rx_hi.min(tx_hi) - rx_lo.max(tx_lo)).max(0.0);
    if tx_bw > 0.0 { overlap / tx_bw } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use crate::antenna::{Antenna, ConstantAntenna};
    use crate::channel::{LinkDirection, Modulation};
    use crate::receiver::{NoiseTempReceiver, Receiver};
    use crate::transmitter::{AmplifierTransmitter, Transmitter};

    use super::*;

    fn test_link() -> (CommunicationSystem, CommunicationSystem, Channel) {
        let tx_sys = CommunicationSystem {
            antenna: Some(Antenna::Constant(ConstantAntenna {
                gain: 46.0.db(),
                beamwidth: Angle::degrees(0.7),
            })),
            receiver: None,
            transmitter: Some(Transmitter::Amplifier(AmplifierTransmitter::new(
                29.0.ghz(),
                10.0,
                1.0.db(),
                0.0.db(),
            ))),
        };
        let rx_sys = CommunicationSystem {
            antenna: Some(Antenna::Constant(ConstantAntenna {
                gain: 30.0.db(),
                beamwidth: Angle::degrees(3.0),
            })),
            receiver: Some(Receiver::NoiseTemperature(NoiseTempReceiver {
                frequency: 29.0.ghz(),
                system_noise_temperature: 500.0,
            })),
            transmitter: None,
        };
        let channel = Channel {
            link_type: LinkDirection::Downlink,
            symbol_rate: 5.0.mhz(),
            required_eb_n0: 10.0.db(),
            margin: 3.0.db(),
            modulation: Modulation::Qpsk,
            roll_off: 0.35,
            fec: 0.5,
            chip_rate: None,
        };
        (tx_sys, rx_sys, channel)
    }

    #[test]
    fn test_environmental_losses_none() {
        let losses = EnvironmentalLosses::none();
        assert_approx_eq!(losses.total().as_f64(), 0.0, atol <= 1e-15);
    }

    #[test]
    fn test_environmental_losses_total() {
        let losses = EnvironmentalLosses {
            rain: 2.0.db(),
            gaseous: 0.5.db(),
            scintillation: 0.3.db(),
            atmospheric: 1.0.db(),
            cloud: 0.2.db(),
            depolarization: 0.1.db(),
        };
        assert_approx_eq!(losses.total().as_f64(), 4.1, atol <= 1e-10);
    }

    #[test]
    fn test_link_stats_calculate() {
        let (tx_sys, rx_sys, channel) = test_link();
        let stats = LinkStats::calculate(
            &tx_sys,
            &rx_sys,
            Distance::kilometers(1000.0),
            channel.bandwidth(),
            EnvironmentalLosses::none(),
            Angle::radians(0.0),
            Angle::radians(0.0),
        )
        .unwrap();

        // EIRP = 46 + 10 - 1 = 55 dBW
        assert_approx_eq!(stats.eirp.as_f64(), 55.0, atol <= 0.01);
        // FSPL at 1000 km, 29 GHz ≈ 181.696 dB
        assert_approx_eq!(stats.fspl.as_f64(), 181.696, atol <= 0.1);
        // C/N0 ≈ 104.9 dB·Hz
        assert_approx_eq!(stats.c_n0.as_f64(), 104.9, atol <= 0.2);
        // carrier_rx_power should be Some for component-tier
        assert!(stats.carrier_rx_power.is_some());
        // noise_power should be Some for component-tier
        assert!(stats.noise_power.is_some());
    }

    #[test]
    fn test_frequency_overlap_full() {
        // Identical bands → full overlap
        let factor = frequency_overlap_factor(10e9, 1e6, 10e9, 1e6);
        assert_approx_eq!(factor, 1.0, atol <= 1e-10);
    }

    #[test]
    fn test_frequency_overlap_none() {
        // Completely separated → no overlap
        let factor = frequency_overlap_factor(10e9, 1e6, 12e9, 1e6);
        assert_approx_eq!(factor, 0.0, atol <= 1e-10);
    }

    #[test]
    fn test_frequency_overlap_partial() {
        // RX: [9.5, 10.5] GHz, TX: [10.0, 11.0] GHz → 0.5 GHz overlap out of 1 GHz TX BW
        let factor = frequency_overlap_factor(10e9, 1e9, 10.5e9, 1e9);
        assert_approx_eq!(factor, 0.5, atol <= 1e-10);
    }

    #[test]
    fn test_frequency_overlap_rx_contains_tx() {
        // RX band fully contains TX band → full overlap
        let factor = frequency_overlap_factor(10e9, 2e9, 10e9, 0.5e9);
        assert_approx_eq!(factor, 1.0, atol <= 1e-10);
    }
}
