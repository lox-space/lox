// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Link budget types: environmental losses, interference, and link statistics.

use lox_core::units::{Angle, Decibel, Distance, Frequency, Power};

use crate::channel::{Channel, LinkDirection};
use crate::error::NonPhysicalError;
use crate::pointing::Pointing;
use crate::resolve::{ResolvedRxTerminal, ResolvedTxTerminal};
use crate::utils::free_space_path_loss;
use crate::{BOLTZMANN_CONSTANT, LinkBudgetError};

pub use lox_itur::EnvironmentalLosses;

/// Interference statistics for a link.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InterferenceStats {
    /// Interference power.
    pub interference_power: Power,
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
    /// Carrier frequency.
    pub frequency: Frequency,
    /// Free-space path loss.
    pub fspl: Decibel,
    /// EIRP of the transmitter.
    pub eirp: Decibel,
    /// Receiver G/T.
    pub gt: Decibel,
    /// Environmental losses.
    pub losses: EnvironmentalLosses,
    /// Received carrier power. `None` for lumped-G/T receive chains.
    pub carrier_rx_power: Option<Decibel>,
    /// Noise power in the channel bandwidth. `None` for lumped-G/T receive chains.
    pub noise_power: Option<Decibel>,
    /// Channel noise bandwidth.
    pub bandwidth: Frequency,
    /// Carrier-to-noise density ratio (C/N₀).
    pub c_n0: Decibel,
    /// Carrier-to-noise ratio (C/N).
    pub c_n: Decibel,
    /// Derived TX pattern polar angle from boresight (zero for lumped or
    /// constant-gain chains).
    pub tx_theta: Angle,
    /// Derived TX pattern azimuth about boresight (zero for lumped or
    /// constant-gain chains).
    pub tx_phi: Angle,
    /// Derived RX pattern polar angle from boresight (zero for lumped or
    /// constant-gain chains).
    pub rx_theta: Angle,
    /// Derived RX pattern azimuth about boresight (zero for lumped or
    /// constant-gain chains).
    pub rx_phi: Angle,
    /// Link direction.
    ///
    /// Reserved for direction-dependent effects such as rain-degraded G/T;
    /// it does not affect the current calculation.
    pub direction: LinkDirection,
}

impl LinkStats {
    /// Computes a modulation-agnostic link budget between resolved terminals.
    ///
    /// The carrier must lie inside both terminals' effective frequency
    /// ranges. `bandwidth` is the noise bandwidth used to compute
    /// `noise_power` and `C/N` from `C/N₀`. The pointings are resolved into
    /// pattern angles against each terminal's antenna frame once and
    /// reported in the result. `direction` is reserved for
    /// direction-dependent effects (e.g. rain-degraded G/T) and does not
    /// affect the current calculation.
    #[allow(clippy::too_many_arguments)]
    pub fn for_link(
        tx: &ResolvedTxTerminal<'_>,
        rx: &ResolvedRxTerminal<'_>,
        carrier: Frequency,
        bandwidth: Frequency,
        range: Distance,
        losses: EnvironmentalLosses,
        tx_pointing: Pointing,
        rx_pointing: Pointing,
        direction: LinkDirection,
    ) -> Result<Self, LinkBudgetError> {
        for (quantity, value) in [
            ("noise bandwidth [Hz]", bandwidth.to_hertz()),
            ("slant range [m]", range.to_meters()),
        ] {
            NonPhysicalError::check_positive(quantity, value)?;
        }

        for (band, terminal) in [
            (tx.band(), tx.terminal_name()),
            (rx.band(), rx.terminal_name()),
        ] {
            if !band.contains(carrier) {
                return Err(LinkBudgetError::CarrierOutOfBand {
                    carrier,
                    band,
                    terminal: terminal.to_owned(),
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
        let noise_power = rx.system_noise_temperature().map(|t_sys| {
            Decibel::from_linear(t_sys.to_kelvin() * BOLTZMANN_CONSTANT * bandwidth.to_hertz())
        });

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
            direction,
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
}

impl ModulatedLinkStats {
    /// Returns interference statistics for the given interferer power.
    ///
    /// Returns an error when absolute carrier or noise power is unavailable
    /// (for example for lumped-G/T links).
    pub fn with_interference(
        &self,
        interference_power: Power,
    ) -> Result<InterferenceStats, LinkBudgetError> {
        NonPhysicalError::check_non_negative(
            "interference power [W]",
            interference_power.to_watts(),
        )?;
        let noise_linear = self
            .link
            .noise_power
            .ok_or(LinkBudgetError::AbsolutePowerUnavailable)?
            .to_linear();
        let carrier = self
            .link
            .carrier_rx_power
            .ok_or(LinkBudgetError::AbsolutePowerUnavailable)?;

        let total_ni = noise_linear + interference_power.to_watts();
        let c_n0i0 = carrier - Decibel::from_linear(total_ni)
            + Decibel::from_linear(self.link.bandwidth.to_hertz());
        let c_n0_to_eb_n0 = self.eb_n0 - self.link.c_n0;
        let eb_n0i0 = c_n0i0 + c_n0_to_eb_n0;

        let threshold = self.eb_n0 - self.margin;
        let margin_with_interference = eb_n0i0 - threshold;

        Ok(InterferenceStats {
            interference_power,
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
pub fn frequency_overlap_factor(
    rx_frequency: Frequency,
    rx_bandwidth: Frequency,
    tx_frequency: Frequency,
    tx_bandwidth: Frequency,
) -> f64 {
    let rx_freq = rx_frequency.to_hertz();
    let rx_bw = rx_bandwidth.to_hertz();
    let tx_freq = tx_frequency.to_hertz();
    let tx_bw = tx_bandwidth.to_hertz();
    let rx_lo = rx_freq - rx_bw / 2.0;
    let rx_hi = rx_freq + rx_bw / 2.0;
    let tx_lo = tx_freq - tx_bw / 2.0;
    let tx_hi = tx_freq + tx_bw / 2.0;

    let overlap = (rx_hi.min(tx_hi) - rx_lo.max(tx_lo)).max(0.0);
    if tx_bw > 0.0 { overlap / tx_bw } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits, Power, Temperature};
    use lox_test_utils::assert_approx_eq;

    use crate::antenna::{Antenna, ConstantAntenna};
    use crate::band::FrequencyRange;
    use crate::channel::Modulation;
    use crate::payload::{CommsPayload, EirpModel, GtModel, TerminalId};
    use crate::receiver::{NoiseTempReceiver, Receiver};
    use crate::transmitter::AmplifierTransmitter;

    use super::*;

    fn ka_band() -> FrequencyRange {
        FrequencyRange::new(27.0.ghz(), 31.0.ghz()).unwrap()
    }

    /// TX: 46 dBi antenna, 10 W, 1 dB feed loss → EIRP = 55 dBW.
    /// RX: 30 dBi antenna, T_sys = 500 K → G/T = 3.01 dB/K.
    fn component_link() -> (CommsPayload, TerminalId, CommsPayload, TerminalId) {
        let (tx_payload, tx_terminal) = CommsPayload::transmitter_only(
            "tx",
            Antenna::Constant(ConstantAntenna::new(46.0.db()).unwrap()),
            AmplifierTransmitter::new(ka_band(), Power::watts(10.0), 0.0.db()).unwrap(),
            1.0.db(),
            None,
        )
        .unwrap();
        let (rx_payload, rx_terminal) = CommsPayload::receiver_only(
            "rx",
            Antenna::Constant(ConstantAntenna::new(30.0.db()).unwrap()),
            Receiver::NoiseTemperature(
                NoiseTempReceiver::new(ka_band(), Temperature::kelvin(500.0)).unwrap(),
            ),
            0.0.db(),
            Temperature::kelvin(0.0),
            None,
        )
        .unwrap();
        (tx_payload, tx_terminal, rx_payload, rx_terminal)
    }

    fn lumped_link() -> (CommsPayload, TerminalId, CommsPayload, TerminalId) {
        let (tx_payload, tx_terminal) =
            CommsPayload::eirp_only(EirpModel::new("eirp", ka_band(), 55.0.db()).unwrap());
        let (rx_payload, rx_terminal) =
            CommsPayload::gt_only(GtModel::new("gt", ka_band(), 3.01.db()).unwrap());
        (tx_payload, tx_terminal, rx_payload, rx_terminal)
    }

    fn channel() -> Channel {
        Channel {
            link_type: LinkDirection::Downlink,
            symbol_rate: 5.0.mhz(),
            required_eb_n0: 10.0.db(),
            margin: 3.0.db(),
            modulation: Modulation::Qpsk,
            roll_off: 0.35,
            fec: 0.5,
            chip_rate: None,
        }
    }

    fn component_stats() -> LinkStats {
        let (tx_payload, tx_terminal, rx_payload, rx_terminal) = component_link();
        LinkStats::for_link(
            &tx_payload.resolve_tx(tx_terminal).unwrap(),
            &rx_payload.resolve_rx(rx_terminal).unwrap(),
            29.0.ghz(),
            channel().bandwidth(),
            Distance::kilometers(1000.0),
            EnvironmentalLosses::none(),
            Pointing::Boresight,
            Pointing::Boresight,
            LinkDirection::Downlink,
        )
        .unwrap()
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
    fn test_for_link_component_budget() {
        let stats = component_stats();
        // EIRP = 46 + 10·log10(10) − 1 = 55 dBW
        assert_approx_eq!(stats.eirp.as_f64(), 55.0, atol <= 1e-10);
        // G/T = 30 − 10·log10(500) = 3.0103 dB/K
        assert_approx_eq!(stats.gt.as_f64(), 3.0103, atol <= 1e-3);
        // FSPL at 1000 km, 29 GHz ≈ 181.696 dB
        assert_approx_eq!(stats.fspl.as_f64(), 181.696, atol <= 0.01);
        // C/N0 = 55 + 3.01 − 181.696 + 228.599 ≈ 104.913 dB·Hz
        assert_approx_eq!(stats.c_n0.as_f64(), 104.913, atol <= 0.01);
        // P_rx = 55 − 181.696 + 30 = −96.696 dBW
        assert_approx_eq!(
            stats.carrier_rx_power.unwrap().as_f64(),
            -96.696,
            atol <= 0.01
        );
        assert!(stats.noise_power.is_some());
        assert_eq!(stats.direction, LinkDirection::Downlink);
        // Boresight pointing resolves to zero pattern angles
        assert_approx_eq!(stats.tx_theta.to_radians(), 0.0, atol <= 1e-15);
        assert_approx_eq!(stats.rx_phi.to_radians(), 0.0, atol <= 1e-15);
    }

    #[test]
    fn test_for_link_c_n0_consistency() {
        // C/N0 must equal P_rx − P_noise + 10·log10(BW).
        let stats = component_stats();
        let c_n0_from_power = stats.carrier_rx_power.unwrap() - stats.noise_power.unwrap()
            + Decibel::from_linear(stats.bandwidth.to_hertz());
        assert_approx_eq!(stats.c_n0.as_f64(), c_n0_from_power.as_f64(), atol <= 1e-10);
    }

    #[test]
    fn test_for_link_lumped_budget() {
        let (tx_payload, tx_terminal, rx_payload, rx_terminal) = lumped_link();
        let stats = LinkStats::for_link(
            &tx_payload.resolve_tx(tx_terminal).unwrap(),
            &rx_payload.resolve_rx(rx_terminal).unwrap(),
            29.0.ghz(),
            5.0.mhz(),
            Distance::kilometers(1000.0),
            EnvironmentalLosses::none(),
            Pointing::Boresight,
            Pointing::Boresight,
            LinkDirection::Uplink,
        )
        .unwrap();
        assert_approx_eq!(stats.c_n0.as_f64(), 104.913, atol <= 0.01);
        assert!(stats.carrier_rx_power.is_none());
        assert!(stats.noise_power.is_none());
    }

    #[test]
    fn test_for_link_rejects_carrier_out_of_band() {
        let (tx_payload, tx_terminal) =
            CommsPayload::eirp_only(EirpModel::new("tx", ka_band(), 55.0.db()).unwrap());
        let (rx_payload, rx_terminal) = CommsPayload::gt_only(
            GtModel::new(
                "rx",
                FrequencyRange::new(17.0.ghz(), 21.0.ghz()).unwrap(),
                3.01.db(),
            )
            .unwrap(),
        );

        // 29 GHz fits the TX band but not the RX band.
        let err = LinkStats::for_link(
            &tx_payload.resolve_tx(tx_terminal).unwrap(),
            &rx_payload.resolve_rx(rx_terminal).unwrap(),
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
    fn test_for_link_rejects_non_physical_inputs() {
        let (tx_payload, tx_terminal, rx_payload, rx_terminal) = lumped_link();
        for (bandwidth, range) in [
            (Frequency::hertz(0.0), Distance::kilometers(1000.0)),
            (Frequency::hertz(-5e6), Distance::kilometers(1000.0)),
            (5.0.mhz(), Distance::kilometers(0.0)),
            (5.0.mhz(), Distance::kilometers(-1.0)),
        ] {
            let err = LinkStats::for_link(
                &tx_payload.resolve_tx(tx_terminal).unwrap(),
                &rx_payload.resolve_rx(rx_terminal).unwrap(),
                29.0.ghz(),
                bandwidth,
                range,
                EnvironmentalLosses::none(),
                Pointing::Boresight,
                Pointing::Boresight,
                LinkDirection::Downlink,
            )
            .unwrap_err();
            assert!(matches!(err, LinkBudgetError::NonPhysical { .. }));
        }
    }

    #[test]
    fn test_channel_apply_produces_modulated_stats() {
        let stats = component_stats();
        let m = channel().apply(stats);
        // Eb/N0 ≈ 37.91 (QPSK, fec=0.5, C/N0 ≈ 104.91 dB·Hz)
        assert_approx_eq!(m.eb_n0.as_f64(), 37.91, atol <= 0.02);
        // required_eb_n0 = 10, margin field = 3 → link_margin ≈ 24.91
        assert_approx_eq!(m.margin.as_f64(), 24.91, atol <= 0.02);
    }

    #[test]
    fn test_modulated_with_interference_reduces_margin() {
        let m = channel().apply(component_stats());
        let interference = m.with_interference(Power::watts(1e-12)).unwrap();
        assert!(interference.margin_with_interference.as_f64() <= m.margin.as_f64());
        assert!(interference.eb_n0i0.as_f64() <= m.eb_n0.as_f64());
    }

    #[test]
    fn test_with_interference_rejects_non_physical_power() {
        let m = channel().apply(component_stats());
        for power in [-1e-12, f64::NAN, f64::INFINITY] {
            let err = m.with_interference(Power::watts(power)).unwrap_err();
            assert!(matches!(err, LinkBudgetError::NonPhysical { .. }));
        }
        // Zero interference is valid and must not change the margin.
        let interference = m.with_interference(Power::watts(0.0)).unwrap();
        assert_approx_eq!(
            interference.margin_with_interference.as_f64(),
            m.margin.as_f64(),
            atol <= 1e-10
        );
    }

    #[test]
    fn test_lumped_modulated_with_interference_is_error() {
        let (tx_payload, tx_terminal, rx_payload, rx_terminal) = lumped_link();
        let ch = channel();
        let stats = LinkStats::for_link(
            &tx_payload.resolve_tx(tx_terminal).unwrap(),
            &rx_payload.resolve_rx(rx_terminal).unwrap(),
            29.0.ghz(),
            ch.bandwidth(),
            Distance::kilometers(1000.0),
            EnvironmentalLosses::none(),
            Pointing::Boresight,
            Pointing::Boresight,
            LinkDirection::Downlink,
        )
        .unwrap();
        let err = ch
            .apply(stats)
            .with_interference(Power::watts(1e-12))
            .unwrap_err();
        assert_eq!(err, LinkBudgetError::AbsolutePowerUnavailable);
    }

    #[test]
    fn test_frequency_overlap_full() {
        // Identical bands → full overlap
        let factor = frequency_overlap_factor(10.0.ghz(), 1.0.mhz(), 10.0.ghz(), 1.0.mhz());
        assert_approx_eq!(factor, 1.0, atol <= 1e-10);
    }

    #[test]
    fn test_frequency_overlap_none() {
        // Completely separated → no overlap
        let factor = frequency_overlap_factor(10.0.ghz(), 1.0.mhz(), 12.0.ghz(), 1.0.mhz());
        assert_approx_eq!(factor, 0.0, atol <= 1e-10);
    }

    #[test]
    fn test_frequency_overlap_partial() {
        // RX: [9.5, 10.5] GHz, TX: [10.0, 11.0] GHz → 0.5 GHz overlap out of 1 GHz TX BW
        let factor = frequency_overlap_factor(10.0.ghz(), 1.0.ghz(), 10.5.ghz(), 1.0.ghz());
        assert_approx_eq!(factor, 0.5, atol <= 1e-10);
    }

    #[test]
    fn test_frequency_overlap_rx_contains_tx() {
        // RX band fully contains TX band → full overlap
        let factor = frequency_overlap_factor(10.0.ghz(), 2.0.ghz(), 10.0.ghz(), 0.5.ghz());
        assert_approx_eq!(factor, 1.0, atol <= 1e-10);
    }
}
