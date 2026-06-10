// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Link budget types: environmental losses, interference, and link statistics.

use lox_core::units::{Angle, Decibel, Distance, Frequency};

use crate::channel::{Channel, LinkDirection};
use crate::endpoint::{RxEndpoint, TxEndpoint};
use crate::system::{CommunicationSystem, Pointing, resolve_pointing};
use crate::utils::free_space_path_loss;
use crate::{BOLTZMANN_CONSTANT, LinkBudgetError};

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
    /// Derived TX pattern polar angle from boresight (zero for lumped or
    /// constant-gain endpoints).
    pub tx_theta: Angle,
    /// Derived TX pattern azimuth about boresight (zero for lumped or
    /// constant-gain endpoints).
    pub tx_phi: Angle,
    /// Derived RX pattern polar angle from boresight (zero for lumped or
    /// constant-gain endpoints).
    pub rx_theta: Angle,
    /// Derived RX pattern azimuth about boresight (zero for lumped or
    /// constant-gain endpoints).
    pub rx_phi: Angle,
    /// Link direction, when known.
    ///
    /// Always `Some` for endpoint-based budgets ([`Self::for_link`]).
    /// Reserved for direction-dependent effects such as rain-degraded G/T;
    /// it does not affect the current calculation.
    pub direction: Option<LinkDirection>,
}

impl LinkStats {
    /// Computes a modulation-agnostic link budget.
    ///
    /// `bandwidth` is the noise bandwidth used to compute `noise_power` and `C/N` from
    /// `C/N₀`. It is independent of any modulation scheme.
    ///
    /// The pointings are resolved into pattern angles against each endpoint's
    /// antenna frame once and reported in the result for traceability.
    pub fn calculate(
        tx_system: &CommunicationSystem,
        rx_system: &CommunicationSystem,
        range: Distance,
        bandwidth: Frequency,
        losses: EnvironmentalLosses,
        tx_pointing: Pointing,
        rx_pointing: Pointing,
    ) -> Result<Self, LinkBudgetError> {
        let env_loss = losses.total();

        let (tx_theta, tx_phi) = resolve_pointing(&tx_system.antenna, tx_pointing)?;
        let (rx_theta, rx_phi) = resolve_pointing(&rx_system.antenna, rx_pointing)?;
        let tx_angles = Pointing::Angles {
            theta: tx_theta,
            phi: tx_phi,
        };
        let rx_angles = Pointing::Angles {
            theta: rx_theta,
            phi: rx_phi,
        };

        let c_n0 =
            tx_system.carrier_to_noise_density(rx_system, env_loss, range, tx_angles, rx_angles)?;
        let carrier_rx_power =
            tx_system.carrier_power(rx_system, env_loss, range, tx_angles, rx_angles)?;
        let noise_power = rx_system.noise_power(bandwidth.to_hertz())?;

        let tx = tx_system
            .transmitter
            .as_ref()
            .ok_or(LinkBudgetError::MissingTransmitter)?;
        let frequency = tx.frequency();
        let fspl = free_space_path_loss(range, frequency);

        let eirp = tx_system.eirp_at(tx_angles)?;
        let gt = rx_system.gt_at(rx_angles)?;

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
            tx_theta,
            tx_phi,
            rx_theta,
            rx_phi,
            direction: None,
        })
    }

    /// Computes a modulation-agnostic link budget between resolved endpoints.
    ///
    /// The carrier must lie inside both endpoints' supported frequency
    /// ranges. `bandwidth` is the noise bandwidth used to compute
    /// `noise_power` and `C/N` from `C/N₀`. The pointings are resolved into
    /// pattern angles against each endpoint's antenna frame once and
    /// reported in the result. `direction` is reserved for
    /// direction-dependent effects (e.g. rain-degraded G/T) and does not
    /// affect the current calculation.
    #[allow(clippy::too_many_arguments)]
    pub fn for_link(
        tx: &TxEndpoint<'_>,
        rx: &RxEndpoint<'_>,
        carrier: Frequency,
        bandwidth: Frequency,
        range: Distance,
        losses: EnvironmentalLosses,
        tx_pointing: Pointing,
        rx_pointing: Pointing,
        direction: LinkDirection,
    ) -> Result<Self, LinkBudgetError> {
        for (band, endpoint) in [
            (tx.band(), tx.terminal_name()),
            (rx.band(), rx.terminal_name()),
        ] {
            if let Some(band) = band
                && !band.contains(carrier)
            {
                return Err(LinkBudgetError::CarrierOutOfBand {
                    carrier,
                    band,
                    endpoint: endpoint.to_owned(),
                });
            }
        }

        let (tx_theta, tx_phi) = tx.pattern_angles(tx_pointing)?;
        let (rx_theta, rx_phi) = rx.pattern_angles(rx_pointing)?;
        let tx_angles = Pointing::Angles {
            theta: tx_theta,
            phi: tx_phi,
        };
        let rx_angles = Pointing::Angles {
            theta: rx_theta,
            phi: rx_phi,
        };

        let eirp = tx.eirp_at(carrier, tx_angles)?;
        let gt = rx.gt_at(carrier, rx_angles)?;
        let fspl = free_space_path_loss(range, carrier);
        let env_loss = losses.total();
        let k_db = Decibel::from_linear(BOLTZMANN_CONSTANT);

        let c_n0 = eirp + gt - fspl - env_loss - k_db;
        let c_n = c_n0 - Decibel::from_linear(bandwidth.to_hertz());

        let carrier_rx_power = rx
            .total_gain(carrier, rx_angles)?
            .map(|g_rx| eirp - fspl - env_loss + g_rx);
        let noise_power = rx
            .system_noise_temperature()
            .map(|t_sys| Decibel::from_linear(t_sys * BOLTZMANN_CONSTANT * bandwidth.to_hertz()));

        Ok(Self {
            slant_range: range,
            frequency: carrier,
            fspl,
            eirp,
            gt,
            losses,
            carrier_rx_power,
            noise_power,
            bandwidth,
            c_n0,
            c_n,
            tx_theta,
            tx_phi,
            rx_theta,
            rx_phi,
            direction: Some(direction),
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
    /// Returns an error when absolute carrier or noise power is unavailable
    /// (for example for lumped-`Gt` links).
    pub fn with_interference(
        &self,
        interference_power_w: f64,
    ) -> Result<InterferenceStats, LinkBudgetError> {
        let noise_linear = self
            .link
            .noise_power
            .ok_or(LinkBudgetError::AbsolutePowerUnavailable)?
            .to_linear();
        let carrier = self
            .link
            .carrier_rx_power
            .ok_or(LinkBudgetError::AbsolutePowerUnavailable)?;

        let total_ni = noise_linear + interference_power_w;
        let c_n0i0 = carrier - Decibel::from_linear(total_ni)
            + Decibel::from_linear(self.link.bandwidth.to_hertz());
        let c_n0_to_eb_n0 = self.eb_n0 - self.link.c_n0;
        let eb_n0i0 = c_n0i0 + c_n0_to_eb_n0;

        let threshold = self.eb_n0 - self.margin;
        let margin_with_interference = eb_n0i0 - threshold;

        Ok(InterferenceStats {
            interference_power_w,
            c_n0i0,
            eb_n0i0,
            margin_with_interference,
        })
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
            antenna: Some(Antenna::Constant(ConstantAntenna { gain: 46.0.db() })),
            receiver: None,
            transmitter: Some(Transmitter::Amplifier(AmplifierTransmitter::new(
                29.0.ghz(),
                10.0,
                1.0.db(),
                0.0.db(),
            ))),
        };
        let rx_sys = CommunicationSystem {
            antenna: Some(Antenna::Constant(ConstantAntenna { gain: 30.0.db() })),
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
            Pointing::Boresight,
            Pointing::Boresight,
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
        // Boresight pointing resolves to zero pattern angles
        assert_approx_eq!(stats.tx_theta.to_radians(), 0.0, atol <= 1e-15);
        assert_approx_eq!(stats.tx_phi.to_radians(), 0.0, atol <= 1e-15);
        assert_approx_eq!(stats.rx_theta.to_radians(), 0.0, atol <= 1e-15);
        assert_approx_eq!(stats.rx_phi.to_radians(), 0.0, atol <= 1e-15);
    }

    #[test]
    fn test_link_stats_reports_derived_pattern_angles() {
        use lox_core::glam::DVec3;

        use crate::antenna::{AntennaFrame, PatternedAntenna};
        use crate::pattern::{AntennaPattern, ParabolicPattern};

        // Dish boresight along +X; the line of sight along +Z is 90° off
        // boresight in the φ = 0 plane, and must be reported as such.
        let tx_sys = CommunicationSystem {
            antenna: Some(Antenna::Patterned(PatternedAntenna {
                pattern: AntennaPattern::Parabolic(ParabolicPattern::new(
                    Distance::meters(0.98),
                    0.45,
                )),
                frame: AntennaFrame::from_boresight_and_reference(DVec3::X, DVec3::Z).unwrap(),
            })),
            receiver: None,
            transmitter: Some(Transmitter::Amplifier(AmplifierTransmitter::new(
                29.0.ghz(),
                10.0,
                1.0.db(),
                0.0.db(),
            ))),
        };
        let (_, rx_sys, channel) = test_link();

        let stats = LinkStats::calculate(
            &tx_sys,
            &rx_sys,
            Distance::kilometers(1000.0),
            channel.bandwidth(),
            EnvironmentalLosses::none(),
            Pointing::Direction(DVec3::Z),
            Pointing::Boresight,
        )
        .unwrap();

        assert_approx_eq!(
            stats.tx_theta.to_radians(),
            std::f64::consts::FRAC_PI_2,
            atol <= 1e-12
        );
        assert_approx_eq!(stats.tx_phi.to_radians(), 0.0, atol <= 1e-12);
        // The EIRP must reflect the off-axis gain, far below the 55 dBW peak.
        assert!(stats.eirp.as_f64() < 0.0);
    }

    #[test]
    fn test_for_link_component_parity_with_legacy_calculate() {
        use crate::band::FrequencyRange;
        use crate::payload::{
            CommsPayload, RxChain, RxPort, Terminal, TerminalRole, TxChain, TxPort,
        };

        // The same physical link expressed both ways must produce identical
        // numbers: legacy puts the 1 dB feed loss on the transmitter
        // (line_loss), the payload puts it on the TX port.
        let (tx_sys, rx_sys, channel) = test_link();
        let legacy = LinkStats::calculate(
            &tx_sys,
            &rx_sys,
            Distance::kilometers(1000.0),
            channel.bandwidth(),
            EnvironmentalLosses::none(),
            Pointing::Boresight,
            Pointing::Boresight,
        )
        .unwrap();

        let mut payload = CommsPayload::new();
        let tx_antenna = payload.add_antenna(
            "tx antenna",
            Antenna::Constant(ConstantAntenna { gain: 46.0.db() }),
        );
        let rx_antenna = payload.add_antenna(
            "rx antenna",
            Antenna::Constant(ConstantAntenna { gain: 30.0.db() }),
        );
        let pa = payload.add_transmitter(
            "pa",
            AmplifierTransmitter::new(29.0.ghz(), 10.0, 0.0.db(), 0.0.db()),
        );
        let receiver = payload
            .add_receiver(
                "receiver",
                Receiver::NoiseTemperature(NoiseTempReceiver {
                    frequency: 29.0.ghz(),
                    system_noise_temperature: 500.0,
                }),
            )
            .unwrap();
        let band = FrequencyRange::new(27.0.ghz(), 31.0.ghz()).unwrap();
        let tx_port = payload
            .add_tx_port(TxPort {
                name: "tx feed".into(),
                antenna: tx_antenna,
                transmitter: pa,
                feed_loss: 1.0.db(),
                band: Some(band),
            })
            .unwrap();
        let rx_port = payload
            .add_rx_port(RxPort {
                name: "rx feed".into(),
                antenna: rx_antenna,
                receiver,
                feed_loss: 0.0.db(),
                antenna_noise_temperature: 150.0,
                band: Some(band),
            })
            .unwrap();
        let tx_terminal = payload
            .add_terminal(Terminal {
                name: "tx".into(),
                role: TerminalRole::Tx(TxChain::Component(tx_port)),
            })
            .unwrap();
        let rx_terminal = payload
            .add_terminal(Terminal {
                name: "rx".into(),
                role: TerminalRole::Rx(RxChain::Component(rx_port)),
            })
            .unwrap();

        let stats = LinkStats::for_link(
            &payload.tx_endpoint(tx_terminal).unwrap(),
            &payload.rx_endpoint(rx_terminal).unwrap(),
            29.0.ghz(),
            channel.bandwidth(),
            Distance::kilometers(1000.0),
            EnvironmentalLosses::none(),
            Pointing::Boresight,
            Pointing::Boresight,
            LinkDirection::Downlink,
        )
        .unwrap();

        assert_approx_eq!(stats.eirp.as_f64(), legacy.eirp.as_f64(), atol <= 1e-12);
        assert_approx_eq!(stats.gt.as_f64(), legacy.gt.as_f64(), atol <= 1e-12);
        assert_approx_eq!(stats.fspl.as_f64(), legacy.fspl.as_f64(), atol <= 1e-12);
        assert_approx_eq!(stats.c_n0.as_f64(), legacy.c_n0.as_f64(), atol <= 1e-12);
        assert_approx_eq!(stats.c_n.as_f64(), legacy.c_n.as_f64(), atol <= 1e-12);
        assert_approx_eq!(
            stats.carrier_rx_power.unwrap().as_f64(),
            legacy.carrier_rx_power.unwrap().as_f64(),
            atol <= 1e-12
        );
        assert_approx_eq!(
            stats.noise_power.unwrap().as_f64(),
            legacy.noise_power.unwrap().as_f64(),
            atol <= 1e-12
        );
        assert_eq!(stats.direction, Some(LinkDirection::Downlink));
        assert_eq!(legacy.direction, None);
    }

    #[test]
    fn test_for_link_lumped_parity_with_legacy_calculate() {
        use crate::band::FrequencyRange;
        use crate::payload::{
            CommsPayload, EirpModel, GtModel, RxChain, Terminal, TerminalRole, TxChain,
        };
        use crate::receiver::GtReceiver;
        use crate::transmitter::EirpTransmitter;

        let legacy_tx = CommunicationSystem::eirp_only(EirpTransmitter {
            frequency: 29.0.ghz(),
            eirp: 55.0.db(),
        });
        let legacy_rx = CommunicationSystem::gt_only(GtReceiver {
            frequency: 29.0.ghz(),
            gt: 3.01.db(),
        });
        let legacy = LinkStats::calculate(
            &legacy_tx,
            &legacy_rx,
            Distance::kilometers(1000.0),
            5.0.mhz(),
            EnvironmentalLosses::none(),
            Pointing::Boresight,
            Pointing::Boresight,
        )
        .unwrap();

        let band = FrequencyRange::new(27.0.ghz(), 31.0.ghz()).unwrap();
        let mut payload = CommsPayload::new();
        let eirp = payload.add_eirp_model(EirpModel {
            name: "eirp".into(),
            band,
            eirp: 55.0.db(),
        });
        let gt = payload.add_gt_model(GtModel {
            name: "gt".into(),
            band,
            gt: 3.01.db(),
        });
        let tx_terminal = payload
            .add_terminal(Terminal {
                name: "tx".into(),
                role: TerminalRole::Tx(TxChain::Lumped(eirp)),
            })
            .unwrap();
        let rx_terminal = payload
            .add_terminal(Terminal {
                name: "rx".into(),
                role: TerminalRole::Rx(RxChain::Lumped(gt)),
            })
            .unwrap();

        let stats = LinkStats::for_link(
            &payload.tx_endpoint(tx_terminal).unwrap(),
            &payload.rx_endpoint(rx_terminal).unwrap(),
            29.0.ghz(),
            5.0.mhz(),
            Distance::kilometers(1000.0),
            EnvironmentalLosses::none(),
            Pointing::Boresight,
            Pointing::Boresight,
            LinkDirection::Uplink,
        )
        .unwrap();

        assert_approx_eq!(stats.c_n0.as_f64(), legacy.c_n0.as_f64(), atol <= 1e-12);
        assert!(stats.carrier_rx_power.is_none());
        assert!(stats.noise_power.is_none());
    }

    #[test]
    fn test_for_link_rejects_carrier_out_of_band() {
        use crate::band::FrequencyRange;
        use crate::payload::{
            CommsPayload, EirpModel, GtModel, RxChain, Terminal, TerminalRole, TxChain,
        };

        let mut payload = CommsPayload::new();
        let eirp = payload.add_eirp_model(EirpModel {
            name: "eirp".into(),
            band: FrequencyRange::new(27.0.ghz(), 31.0.ghz()).unwrap(),
            eirp: 55.0.db(),
        });
        let gt = payload.add_gt_model(GtModel {
            name: "gt".into(),
            band: FrequencyRange::new(17.0.ghz(), 21.0.ghz()).unwrap(),
            gt: 3.01.db(),
        });
        let tx_terminal = payload
            .add_terminal(Terminal {
                name: "tx".into(),
                role: TerminalRole::Tx(TxChain::Lumped(eirp)),
            })
            .unwrap();
        let rx_terminal = payload
            .add_terminal(Terminal {
                name: "rx".into(),
                role: TerminalRole::Rx(RxChain::Lumped(gt)),
            })
            .unwrap();

        // 29 GHz fits the TX band but not the RX band.
        let err = LinkStats::for_link(
            &payload.tx_endpoint(tx_terminal).unwrap(),
            &payload.rx_endpoint(rx_terminal).unwrap(),
            29.0.ghz(),
            5.0.mhz(),
            Distance::kilometers(1000.0),
            EnvironmentalLosses::none(),
            Pointing::Boresight,
            Pointing::Boresight,
            LinkDirection::Downlink,
        )
        .unwrap_err();

        assert!(matches!(err, LinkBudgetError::CarrierOutOfBand { .. }));
        let message = err.to_string();
        assert!(message.contains("'rx'"));
        assert!(message.contains("17.000–21.000 GHz"));
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

    #[test]
    fn test_channel_apply_produces_modulated_stats() {
        let (tx_sys, rx_sys, channel) = test_link();
        let link = LinkStats::calculate(
            &tx_sys,
            &rx_sys,
            Distance::kilometers(1000.0),
            channel.bandwidth(),
            EnvironmentalLosses::none(),
            Pointing::Boresight,
            Pointing::Boresight,
        )
        .unwrap();
        let m = channel.apply(link);
        // Eb/N0 ≈ 37.91 (from existing test_link fixtures: QPSK, fec=0.5, C/N0≈104.9 dBHz)
        assert_approx_eq!(m.eb_n0.as_f64(), 37.91, atol <= 0.2);
        // required_eb_n0 = 10, margin field = 3 → link_margin ≈ 24.91
        assert_approx_eq!(m.margin.as_f64(), 24.91, atol <= 0.2);
    }

    #[test]
    fn test_modulated_with_interference_reduces_margin() {
        let (tx_sys, rx_sys, channel) = test_link();
        let link = LinkStats::calculate(
            &tx_sys,
            &rx_sys,
            Distance::kilometers(1000.0),
            channel.bandwidth(),
            EnvironmentalLosses::none(),
            Pointing::Boresight,
            Pointing::Boresight,
        )
        .unwrap();
        let m = channel.apply(link);
        let interference = m.with_interference(1e-12).unwrap();
        assert!(interference.margin_with_interference.as_f64() <= m.margin.as_f64());
        assert!(interference.eb_n0i0.as_f64() <= m.eb_n0.as_f64());
    }

    #[test]
    fn test_lumped_link_stats_carrier_and_noise_are_none() {
        use crate::receiver::GtReceiver;
        use crate::transmitter::EirpTransmitter;

        let tx = CommunicationSystem::eirp_only(EirpTransmitter {
            frequency: 29.0.ghz(),
            eirp: 55.0.db(),
        });
        let rx = CommunicationSystem::gt_only(GtReceiver {
            frequency: 29.0.ghz(),
            gt: 3.01.db(),
        });
        let stats = LinkStats::calculate(
            &tx,
            &rx,
            Distance::kilometers(1000.0),
            5.0.mhz(),
            EnvironmentalLosses::none(),
            Pointing::Boresight,
            Pointing::Boresight,
        )
        .unwrap();
        assert!(stats.carrier_rx_power.is_none());
        assert!(stats.noise_power.is_none());
        assert_approx_eq!(stats.c_n0.as_f64(), 104.913, atol <= 0.2);
    }

    #[test]
    fn test_lumped_modulated_with_interference_is_error() {
        use crate::receiver::GtReceiver;
        use crate::transmitter::EirpTransmitter;

        let tx = CommunicationSystem::eirp_only(EirpTransmitter {
            frequency: 29.0.ghz(),
            eirp: 55.0.db(),
        });
        let rx = CommunicationSystem::gt_only(GtReceiver {
            frequency: 29.0.ghz(),
            gt: 3.01.db(),
        });
        let channel = Channel {
            link_type: LinkDirection::Downlink,
            symbol_rate: 5.0.mhz(),
            required_eb_n0: 10.0.db(),
            margin: 3.0.db(),
            modulation: Modulation::Qpsk,
            roll_off: 0.0,
            fec: 0.5,
            chip_rate: None,
        };
        let link = LinkStats::calculate(
            &tx,
            &rx,
            Distance::kilometers(1000.0),
            channel.bandwidth(),
            EnvironmentalLosses::none(),
            Pointing::Boresight,
            Pointing::Boresight,
        )
        .unwrap();
        let err = channel.apply(link).with_interference(1e-12).unwrap_err();
        assert_eq!(err, LinkBudgetError::AbsolutePowerUnavailable);
    }
}
