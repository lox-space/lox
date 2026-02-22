// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Link budget types: environmental losses, interference, and link statistics.

use lox_core::units::{Angle, Decibel, Distance, Frequency};

use crate::channel::Channel;
use crate::system::CommunicationSystem;
use crate::utils::free_space_path_loss;

/// Environmental losses (rain, atmospheric, etc.).
///
/// ITU-R computation is out of scope; construct manually or use [`EnvironmentalLosses::none`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EnvironmentalLosses {
    /// Rain attenuation.
    pub rain: Decibel,
    /// Gaseous absorption.
    pub gaseous: Decibel,
    /// Scintillation loss.
    pub scintillation: Decibel,
    /// General atmospheric loss.
    pub atmospheric: Decibel,
    /// Cloud attenuation.
    pub cloud: Decibel,
    /// Depolarization loss.
    pub depolarization: Decibel,
}

impl EnvironmentalLosses {
    /// Returns zero environmental losses.
    pub fn none() -> Self {
        Self {
            rain: Decibel::new(0.0),
            gaseous: Decibel::new(0.0),
            scintillation: Decibel::new(0.0),
            atmospheric: Decibel::new(0.0),
            cloud: Decibel::new(0.0),
            depolarization: Decibel::new(0.0),
        }
    }

    /// Returns the total environmental loss in dB.
    pub fn total(&self) -> Decibel {
        self.rain
            + self.gaseous
            + self.scintillation
            + self.atmospheric
            + self.cloud
            + self.depolarization
    }
}

/// Interference statistics for a link.
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

/// Complete link budget statistics.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LinkStats {
    /// Slant range between TX and RX.
    pub slant_range: Distance,
    /// Free-space path loss.
    pub fspl: Decibel,
    /// Off-boresight angle at the transmitter.
    pub tx_angle: Angle,
    /// Off-boresight angle at the receiver.
    pub rx_angle: Angle,
    /// EIRP of the transmitter.
    pub eirp: Decibel,
    /// Receiver G/T.
    pub gt: Decibel,
    /// Carrier-to-noise density ratio.
    pub c_n0: Decibel,
    /// Eb/N0.
    pub eb_n0: Decibel,
    /// Link margin.
    pub margin: Decibel,
    /// Environmental losses.
    pub losses: EnvironmentalLosses,
    /// Received carrier power.
    pub carrier_rx_power: Decibel,
    /// Data rate in bits per second.
    pub data_rate: f64,
    /// Channel bandwidth in Hz.
    pub bandwidth_hz: f64,
    /// Link frequency.
    pub frequency: Frequency,
    /// Noise power.
    pub noise_power: Decibel,
    /// Interference statistics (if applicable).
    pub interference: Option<InterferenceStats>,
}

impl LinkStats {
    /// Computes a full link budget.
    pub fn calculate(
        tx_system: &CommunicationSystem,
        rx_system: &CommunicationSystem,
        channel: &Channel,
        range: Distance,
        tx_angle: Angle,
        rx_angle: Angle,
        losses: EnvironmentalLosses,
    ) -> Self {
        let env_loss = losses.total();

        let c_n0 =
            tx_system.carrier_to_noise_density(rx_system, env_loss, range, tx_angle, rx_angle);
        let carrier_rx_power =
            tx_system.carrier_power(rx_system, env_loss, range, tx_angle, rx_angle);

        let tx = tx_system
            .transmitter
            .as_ref()
            .expect("TX system must have a transmitter");
        let receiver = rx_system
            .receiver
            .as_ref()
            .expect("RX system must have a receiver");

        let frequency = tx.frequency;
        let eirp = tx.eirp(&tx_system.antenna, tx_angle);
        let gt = receiver.gain_to_noise_temperature(&rx_system.antenna, rx_angle);
        let fspl = free_space_path_loss(range, frequency);
        let bandwidth_hz = channel.bandwidth();
        let noise_power = rx_system.noise_power(bandwidth_hz);
        let eb_n0 = channel.eb_n0(c_n0);
        let margin = channel.link_margin(eb_n0);

        Self {
            slant_range: range,
            fspl,
            tx_angle,
            rx_angle,
            eirp,
            gt,
            c_n0,
            eb_n0,
            margin,
            losses,
            carrier_rx_power,
            data_rate: channel.data_rate,
            bandwidth_hz,
            frequency,
            noise_power,
            interference: None,
        }
    }

    /// Returns a copy of this link budget with interference statistics added.
    pub fn with_interference(&self, interference_power_w: f64) -> InterferenceStats {
        let noise_linear = self.noise_power.to_linear();
        let total_ni = noise_linear + interference_power_w;
        let c_n0i0 = self.carrier_rx_power - Decibel::from_linear(total_ni)
            + Decibel::from_linear(self.bandwidth_hz);
        let eb_n0i0 = c_n0i0 - Decibel::from_linear(self.data_rate);

        // margin = eb_n0 - (required_eb_n0 + required_margin)
        // So the threshold (required_eb_n0 + required_margin) = eb_n0 - margin
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

    use crate::antenna::{Antenna, SimpleAntenna};
    use crate::channel::{Channel, LinkDirection, Modulation};
    use crate::receiver::{Receiver, SimpleReceiver};
    use crate::transmitter::Transmitter;

    use super::*;

    fn test_link() -> (CommunicationSystem, CommunicationSystem, Channel) {
        let tx_sys = CommunicationSystem {
            antenna: Antenna::Simple(SimpleAntenna {
                gain: 46.0.db(),
                beamwidth: Angle::degrees(0.7),
            }),
            receiver: None,
            transmitter: Some(Transmitter::new(29.0.ghz(), 10.0, 1.0.db(), 0.0.db())),
        };
        let rx_sys = CommunicationSystem {
            antenna: Antenna::Simple(SimpleAntenna {
                gain: 30.0.db(),
                beamwidth: Angle::degrees(3.0),
            }),
            receiver: Some(Receiver::Simple(SimpleReceiver {
                frequency: 29.0.ghz(),
                system_noise_temperature: 500.0,
            })),
            transmitter: None,
        };
        let channel = Channel {
            link_type: LinkDirection::Downlink,
            data_rate: 10e6,
            required_eb_n0: 10.0.db(),
            margin: 3.0.db(),
            modulation: Modulation::Qpsk,
            roll_off: 0.35,
            fec: 0.5,
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
            &channel,
            Distance::kilometers(1000.0),
            Angle::radians(0.0),
            Angle::radians(0.0),
            EnvironmentalLosses::none(),
        );

        // EIRP = 46 + 10 - 1 = 55 dBW
        assert_approx_eq!(stats.eirp.as_f64(), 55.0, atol <= 0.01);
        // FSPL at 1000 km, 29 GHz ≈ 181.696 dB
        assert_approx_eq!(stats.fspl.as_f64(), 181.696, atol <= 0.1);
        // C/N0 ≈ 104.9 dB·Hz
        assert_approx_eq!(stats.c_n0.as_f64(), 104.9, atol <= 0.2);
        // Eb/N0 = C/N0 - 10*log10(10e6) = 104.9 - 70 = 34.9
        assert_approx_eq!(stats.eb_n0.as_f64(), 34.9, atol <= 0.2);
        // Margin = Eb/N0 - 10 - 3 = 21.9
        assert_approx_eq!(stats.margin.as_f64(), 21.9, atol <= 0.2);
    }

    #[test]
    fn test_link_stats_with_interference() {
        let (tx_sys, rx_sys, channel) = test_link();
        let stats = LinkStats::calculate(
            &tx_sys,
            &rx_sys,
            &channel,
            Distance::kilometers(1000.0),
            Angle::radians(0.0),
            Angle::radians(0.0),
            EnvironmentalLosses::none(),
        );

        // Adding interference should reduce margin
        let interference = stats.with_interference(1e-12);
        assert!(interference.margin_with_interference.as_f64() <= stats.margin.as_f64());
        assert!(interference.eb_n0i0.as_f64() <= stats.eb_n0.as_f64());
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
