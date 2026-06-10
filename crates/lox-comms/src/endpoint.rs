// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Resolved link endpoints.
//!
//! [`CommsPayload::tx_endpoint`] and [`CommsPayload::rx_endpoint`] resolve a
//! terminal into borrowed endpoint views with the wiring already followed:
//! radio, antenna, feed loss (or lumped model), and the effective frequency
//! range. Resolution fails on wiring problems ([`ResolveError`]); the
//! link-budget calculation that consumes the views only fails on physics.
//!
//! Endpoints are cheap to construct but meant to be resolved once per pass
//! and reused across time steps.

use lox_core::units::{Angle, Decibel, Frequency, Kelvin};
use thiserror::Error;

use crate::LinkBudgetError;
use crate::ROOM_TEMPERATURE;
use crate::antenna::{Antenna, AntennaGain};
use crate::band::FrequencyRange;
use crate::payload::{
    CommsPayload, EirpModel, GtModel, RxChain, TerminalId, TerminalRole, TxChain,
};
use crate::pointing::Pointing;
use crate::receiver::{CascadeReceiver, NoiseStage, Receiver};
use crate::transmitter::AmplifierTransmitter;

/// A resolved transmit endpoint.
#[derive(Debug, Clone, Copy)]
pub struct TxEndpoint<'a> {
    terminal_name: &'a str,
    kind: TxEndpointKind<'a>,
}

/// The resolved transmit chain behind a [`TxEndpoint`].
#[derive(Debug, Clone, Copy)]
pub enum TxEndpointKind<'a> {
    /// Component tier: transmitter + feed + antenna.
    Component {
        /// The antenna radiating this path.
        antenna: &'a Antenna,
        /// The transmitter driving this path.
        transmitter: &'a AmplifierTransmitter,
        /// Feed loss between transmitter output and antenna.
        feed_loss: Decibel,
        /// Effective frequency range: transmitter band ∩ port band.
        band: FrequencyRange,
    },
    /// Lumped tier: aggregate EIRP figure.
    Lumped(&'a EirpModel),
}

/// A resolved receive endpoint.
#[derive(Debug, Clone, Copy)]
pub struct RxEndpoint<'a> {
    terminal_name: &'a str,
    kind: RxEndpointKind<'a>,
}

/// The resolved receive chain behind an [`RxEndpoint`].
#[derive(Debug, Clone, Copy)]
pub enum RxEndpointKind<'a> {
    /// Component tier: antenna + feed + receiver.
    Component {
        /// The antenna feeding this path.
        antenna: &'a Antenna,
        /// The receiver terminating this path.
        receiver: &'a Receiver,
        /// Feed loss between antenna and receiver input.
        feed_loss: Decibel,
        /// Clear-sky antenna noise temperature at this port, in Kelvin.
        antenna_noise_temperature: Kelvin,
        /// Effective frequency range: receiver band ∩ port band.
        band: FrequencyRange,
    },
    /// Lumped tier: aggregate G/T figure.
    Lumped(&'a GtModel),
}

impl CommsPayload {
    /// Resolves the transmit chain of a terminal into an endpoint view.
    ///
    /// Fails when the terminal does not exist in this payload, has no
    /// transmit chain, or its wiring references missing inventory (only
    /// possible with IDs minted by a different payload).
    pub fn tx_endpoint(&self, id: TerminalId) -> Result<TxEndpoint<'_>, ResolveError> {
        let terminal = self.terminal(id).ok_or(ResolveError::UnknownTerminal(id))?;
        let chain = match terminal.role {
            TerminalRole::Tx(tx) | TerminalRole::Transceiver { tx, .. } => tx,
            TerminalRole::Rx(_) => {
                return Err(ResolveError::NotATransmitTerminal(terminal.name.clone()));
            }
        };
        let kind = match chain {
            TxChain::Component(port_id) => {
                let port = self
                    .tx_port(port_id)
                    .ok_or(ResolveError::BrokenTxPort(port_id))?;
                let transmitter = &self
                    .transmitter(port.transmitter)
                    .ok_or(ResolveError::BrokenTransmitter(port.transmitter))?
                    .value;
                TxEndpointKind::Component {
                    antenna: &self
                        .antenna(port.antenna)
                        .ok_or(ResolveError::BrokenAntenna(port.antenna))?
                        .value,
                    transmitter,
                    feed_loss: port.feed_loss,
                    band: effective_band(transmitter.band, port.band, &terminal.name)?,
                }
            }
            TxChain::Lumped(model_id) => TxEndpointKind::Lumped(
                self.eirp_model(model_id)
                    .ok_or(ResolveError::BrokenEirpModel(model_id))?,
            ),
        };
        Ok(TxEndpoint {
            terminal_name: &terminal.name,
            kind,
        })
    }

    /// Resolves the receive chain of a terminal into an endpoint view.
    ///
    /// Fails when the terminal does not exist in this payload, has no
    /// receive chain, or its wiring references missing inventory (only
    /// possible with IDs minted by a different payload).
    pub fn rx_endpoint(&self, id: TerminalId) -> Result<RxEndpoint<'_>, ResolveError> {
        let terminal = self.terminal(id).ok_or(ResolveError::UnknownTerminal(id))?;
        let chain = match terminal.role {
            TerminalRole::Rx(rx) | TerminalRole::Transceiver { rx, .. } => rx,
            TerminalRole::Tx(_) => {
                return Err(ResolveError::NotAReceiveTerminal(terminal.name.clone()));
            }
        };
        let kind = match chain {
            RxChain::Component(port_id) => {
                let port = self
                    .rx_port(port_id)
                    .ok_or(ResolveError::BrokenRxPort(port_id))?;
                let receiver = &self
                    .receiver(port.receiver)
                    .ok_or(ResolveError::BrokenReceiver(port.receiver))?
                    .value;
                RxEndpointKind::Component {
                    antenna: &self
                        .antenna(port.antenna)
                        .ok_or(ResolveError::BrokenAntenna(port.antenna))?
                        .value,
                    receiver,
                    feed_loss: port.feed_loss,
                    antenna_noise_temperature: port.antenna_noise_temperature,
                    band: effective_band(receiver.band(), port.band, &terminal.name)?,
                }
            }
            RxChain::Lumped(model_id) => RxEndpointKind::Lumped(
                self.gt_model(model_id)
                    .ok_or(ResolveError::BrokenGtModel(model_id))?,
            ),
        };
        Ok(RxEndpoint {
            terminal_name: &terminal.name,
            kind,
        })
    }
}

impl<'a> TxEndpoint<'a> {
    /// Returns the name of the terminal this endpoint was resolved from.
    pub fn terminal_name(&self) -> &'a str {
        self.terminal_name
    }

    /// Returns the resolved transmit chain.
    pub fn kind(&self) -> &TxEndpointKind<'a> {
        &self.kind
    }

    /// Returns the effective frequency range of this endpoint.
    pub fn band(&self) -> FrequencyRange {
        match &self.kind {
            TxEndpointKind::Component { band, .. } => *band,
            TxEndpointKind::Lumped(model) => model.band,
        }
    }

    /// Resolves a pointing into pattern angles against this endpoint's antenna.
    ///
    /// Lumped endpoints ignore the pointing and resolve to zero angles.
    pub fn pattern_angles(&self, pointing: Pointing) -> Result<(Angle, Angle), LinkBudgetError> {
        resolve_pointing(self.antenna(), pointing)
    }

    /// Returns the EIRP in dBW at the given carrier and pointing.
    ///
    /// For lumped endpoints the carrier and pointing are ignored and the
    /// stored figure is returned. For component endpoints:
    ///
    /// EIRP = G_ant(carrier, θ, φ) + 10·log₁₀(P) − OBO − feed loss
    pub fn eirp_at(
        &self,
        carrier: Frequency,
        pointing: Pointing,
    ) -> Result<Decibel, LinkBudgetError> {
        match &self.kind {
            TxEndpointKind::Lumped(model) => Ok(model.eirp),
            TxEndpointKind::Component {
                antenna,
                transmitter,
                feed_loss,
                ..
            } => {
                let (theta, phi) = self.pattern_angles(pointing)?;
                Ok(
                    antenna.gain(carrier, theta, phi) + Decibel::from_linear(transmitter.power_w)
                        - transmitter.output_back_off
                        - *feed_loss,
                )
            }
        }
    }

    fn antenna(&self) -> Option<&'a Antenna> {
        match &self.kind {
            TxEndpointKind::Component { antenna, .. } => Some(antenna),
            TxEndpointKind::Lumped(_) => None,
        }
    }
}

impl<'a> RxEndpoint<'a> {
    /// Returns the name of the terminal this endpoint was resolved from.
    pub fn terminal_name(&self) -> &'a str {
        self.terminal_name
    }

    /// Returns the resolved receive chain.
    pub fn kind(&self) -> &RxEndpointKind<'a> {
        &self.kind
    }

    /// Returns the effective frequency range of this endpoint.
    pub fn band(&self) -> FrequencyRange {
        match &self.kind {
            RxEndpointKind::Component { band, .. } => *band,
            RxEndpointKind::Lumped(model) => model.band,
        }
    }

    /// Resolves a pointing into pattern angles against this endpoint's antenna.
    ///
    /// Lumped endpoints ignore the pointing and resolve to zero angles.
    pub fn pattern_angles(&self, pointing: Pointing) -> Result<(Angle, Angle), LinkBudgetError> {
        resolve_pointing(self.antenna(), pointing)
    }

    /// Returns the system noise temperature in Kelvin, when the chain exposes one.
    ///
    /// For cascade receivers the port feed loss is synthesized as a passive
    /// attenuator at 290 K ahead of the chain and the port's antenna noise
    /// temperature replaces the receiver's own. For known-T_sys receivers the
    /// stored figure is returned unchanged. Lumped G/T endpoints return `None`.
    pub fn system_noise_temperature(&self) -> Option<Kelvin> {
        match &self.kind {
            RxEndpointKind::Lumped(_) => None,
            RxEndpointKind::Component {
                receiver,
                feed_loss,
                antenna_noise_temperature,
                ..
            } => match receiver {
                Receiver::NoiseTemperature(rx) => Some(rx.system_noise_temperature),
                Receiver::Cascade(rx) => Some(cascade_noise_temperature(
                    rx,
                    *feed_loss,
                    *antenna_noise_temperature,
                )),
            },
        }
    }

    /// Returns the clear-sky antenna noise temperature at this endpoint's
    /// port, when the chain exposes one.
    pub fn antenna_noise_temperature(&self) -> Option<Kelvin> {
        match &self.kind {
            RxEndpointKind::Component {
                antenna_noise_temperature,
                ..
            } => Some(*antenna_noise_temperature),
            RxEndpointKind::Lumped(_) => None,
        }
    }

    /// Returns the gain-to-noise-temperature ratio (G/T) in dB/K at the given
    /// carrier and pointing.
    ///
    /// For lumped endpoints the carrier and pointing are ignored and the
    /// stored figure is returned. For component endpoints:
    ///
    /// G/T = G_total(carrier, θ, φ) − 10·log₁₀(T_sys)
    ///
    /// where `T_sys` is computed per [`Self::system_noise_temperature`].
    /// For known-T_sys receivers the port feed loss reduces the gain; the
    /// stored figure is assumed to already include the feed's noise.
    pub fn gt_at(
        &self,
        carrier: Frequency,
        pointing: Pointing,
    ) -> Result<Decibel, LinkBudgetError> {
        match &self.kind {
            RxEndpointKind::Lumped(model) => Ok(model.gt),
            RxEndpointKind::Component {
                antenna,
                receiver,
                feed_loss,
                antenna_noise_temperature,
                ..
            } => {
                let (theta, phi) = self.pattern_angles(pointing)?;
                let gain = antenna.gain(carrier, theta, phi);
                match receiver {
                    Receiver::NoiseTemperature(rx) => {
                        Ok(gain - *feed_loss - Decibel::from_linear(rx.system_noise_temperature))
                    }
                    Receiver::Cascade(rx) => {
                        let t_sys =
                            cascade_noise_temperature(rx, *feed_loss, *antenna_noise_temperature);
                        Ok(gain
                            - rx.demodulator_loss
                            - rx.implementation_loss
                            - Decibel::from_linear(t_sys))
                    }
                }
            }
        }
    }

    /// Returns the receive system gain in dB before noise referral, when the
    /// chain exposes one.
    ///
    /// `None` for lumped G/T endpoints — the absolute gain is not recoverable
    /// from a G/T figure. Consistent with [`Self::gt_at`]:
    /// `total_gain − 10·log₁₀(T_sys) == G/T`.
    pub fn total_gain(
        &self,
        carrier: Frequency,
        pointing: Pointing,
    ) -> Result<Option<Decibel>, LinkBudgetError> {
        match &self.kind {
            RxEndpointKind::Lumped(_) => Ok(None),
            RxEndpointKind::Component {
                antenna,
                receiver,
                feed_loss,
                ..
            } => {
                let (theta, phi) = self.pattern_angles(pointing)?;
                let gain = antenna.gain(carrier, theta, phi);
                match receiver {
                    Receiver::NoiseTemperature(_) => Ok(Some(gain - *feed_loss)),
                    Receiver::Cascade(rx) => {
                        Ok(Some(gain - rx.demodulator_loss - rx.implementation_loss))
                    }
                }
            }
        }
    }

    fn antenna(&self) -> Option<&'a Antenna> {
        match &self.kind {
            RxEndpointKind::Component { antenna, .. } => Some(antenna),
            RxEndpointKind::Lumped(_) => None,
        }
    }
}

/// Friis cascade with the port feed synthesized as the first stage.
///
/// T_sys = T_ant + T_feed + T_1/G_feed + T_2/(G_feed·G_1) + ...
///
/// The receiver's own `antenna_noise_temperature` is ignored; the port's
/// value is authoritative.
fn cascade_noise_temperature(
    receiver: &CascadeReceiver,
    feed_loss: Decibel,
    antenna_noise_temperature: Kelvin,
) -> Kelvin {
    let feed_stage = NoiseStage {
        gain: -feed_loss,
        noise_temperature: ROOM_TEMPERATURE * (feed_loss.to_linear() - 1.0),
    };
    let mut t_sys = antenna_noise_temperature;
    let mut cumulative_gain_linear = 1.0;
    for stage in std::iter::once(&feed_stage).chain(receiver.stages.iter()) {
        t_sys += stage.noise_temperature / cumulative_gain_linear;
        cumulative_gain_linear *= stage.gain.to_linear();
    }
    t_sys
}

/// Intersects a radio band with an optional port narrowing.
fn effective_band(
    radio: FrequencyRange,
    port: Option<FrequencyRange>,
    terminal: &str,
) -> Result<FrequencyRange, ResolveError> {
    match port {
        None => Ok(radio),
        Some(port) => radio
            .intersect(&port)
            .ok_or_else(|| ResolveError::EmptyBandIntersection(terminal.to_owned())),
    }
}

/// Resolves a pointing into pattern angles against an optional antenna.
fn resolve_pointing(
    antenna: Option<&Antenna>,
    pointing: Pointing,
) -> Result<(Angle, Angle), LinkBudgetError> {
    match pointing {
        Pointing::Boresight => Ok((Angle::ZERO, Angle::ZERO)),
        Pointing::Angles { theta, phi } => Ok((theta, phi)),
        Pointing::Direction(direction) => match antenna {
            None => Ok((Angle::ZERO, Angle::ZERO)),
            Some(antenna) => Ok(antenna.pattern_angles(direction)?),
        },
    }
}

/// Errors produced while resolving a terminal into an endpoint view.
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum ResolveError {
    /// The terminal does not exist in this payload.
    #[error("unknown terminal ID {0:?}")]
    UnknownTerminal(TerminalId),
    /// The terminal has no transmit chain.
    #[error("terminal '{0}' has no transmit chain")]
    NotATransmitTerminal(String),
    /// The terminal has no receive chain.
    #[error("terminal '{0}' has no receive chain")]
    NotAReceiveTerminal(String),
    /// The terminal references a transmit port missing from this payload.
    #[error("terminal references TX port {0:?} missing from this payload")]
    BrokenTxPort(crate::payload::TxPortId),
    /// The terminal references a receive port missing from this payload.
    #[error("terminal references RX port {0:?} missing from this payload")]
    BrokenRxPort(crate::payload::RxPortId),
    /// A port references an antenna missing from this payload.
    #[error("port references antenna {0:?} missing from this payload")]
    BrokenAntenna(crate::payload::AntennaId),
    /// A port references a transmitter missing from this payload.
    #[error("port references transmitter {0:?} missing from this payload")]
    BrokenTransmitter(crate::payload::TransmitterId),
    /// A port references a receiver missing from this payload.
    #[error("port references receiver {0:?} missing from this payload")]
    BrokenReceiver(crate::payload::ReceiverId),
    /// The terminal references an EIRP model missing from this payload.
    #[error("terminal references EIRP model {0:?} missing from this payload")]
    BrokenEirpModel(crate::payload::EirpModelId),
    /// The terminal references a G/T model missing from this payload.
    #[error("terminal references G/T model {0:?} missing from this payload")]
    BrokenGtModel(crate::payload::GtModelId),
    /// The radio and port frequency ranges do not overlap.
    #[error("terminal '{0}' has no usable frequency range: radio and port bands are disjoint")]
    EmptyBandIntersection(String),
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use crate::antenna::ConstantAntenna;
    use crate::payload::{RxPort, Terminal, TxPort};
    use crate::receiver::{NoiseTempReceiver, noise_figure_to_temperature};

    use super::*;

    fn ka_band() -> FrequencyRange {
        FrequencyRange::new(27.0.ghz(), 31.0.ghz()).unwrap()
    }

    /// TX: 46 dBi dish, 10 W amplifier, 1 dB feed loss → EIRP 55 dBW.
    /// RX: 30 dBi antenna, T_sys = 500 K noise-temp receiver, 0 dB feed.
    fn transceiver_payload() -> (CommsPayload, TerminalId) {
        let mut payload = CommsPayload::new();
        let dish = payload.add_antenna(
            "dish",
            Antenna::Constant(ConstantAntenna { gain: 46.0.db() }),
        );
        let rx_antenna = payload.add_antenna(
            "rx antenna",
            Antenna::Constant(ConstantAntenna { gain: 30.0.db() }),
        );
        let pa =
            payload.add_transmitter("pa", AmplifierTransmitter::new(ka_band(), 10.0, 0.0.db()));
        let rx = payload.add_receiver(
            "receiver",
            Receiver::NoiseTemperature(NoiseTempReceiver {
                band: ka_band(),
                system_noise_temperature: 500.0,
            }),
        );
        let tx_port = payload
            .add_tx_port(TxPort {
                name: "tx feed".into(),
                antenna: dish,
                transmitter: pa,
                feed_loss: 1.0.db(),
                band: None,
            })
            .unwrap();
        let rx_port = payload
            .add_rx_port(RxPort {
                name: "rx feed".into(),
                antenna: rx_antenna,
                receiver: rx,
                feed_loss: 0.0.db(),
                antenna_noise_temperature: 150.0,
                band: None,
            })
            .unwrap();
        let terminal = payload
            .add_terminal(Terminal {
                name: "transceiver".into(),
                role: TerminalRole::Transceiver {
                    tx: TxChain::Component(tx_port),
                    rx: RxChain::Component(rx_port),
                },
            })
            .unwrap();
        (payload, terminal)
    }

    #[test]
    fn test_tx_endpoint_eirp_with_port_feed_loss() {
        let (payload, terminal) = transceiver_payload();
        let tx = payload.tx_endpoint(terminal).unwrap();
        // EIRP = 46 + 10·log10(10) − 0 (OBO) − 1 (feed) = 55 dBW
        let eirp = tx.eirp_at(29.0.ghz(), Pointing::Boresight).unwrap();
        assert_approx_eq!(eirp.as_f64(), 55.0, atol <= 1e-10);
        assert_eq!(tx.terminal_name(), "transceiver");
        // With no port narrowing the radio band is the effective band.
        assert!(tx.band().contains(29.0.ghz()));
    }

    #[test]
    fn test_rx_endpoint_gt_noise_temp_receiver() {
        let (payload, terminal) = transceiver_payload();
        let rx = payload.rx_endpoint(terminal).unwrap();
        // G/T = 30 − 10·log10(500) = 3.0103 dB/K
        let gt = rx.gt_at(29.0.ghz(), Pointing::Boresight).unwrap();
        assert_approx_eq!(gt.as_f64(), 3.0103, atol <= 1e-3);
        assert_approx_eq!(rx.system_noise_temperature().unwrap(), 500.0, atol <= 1e-12);
        assert_approx_eq!(
            rx.antenna_noise_temperature().unwrap(),
            150.0,
            atol <= 1e-12
        );
    }

    #[test]
    fn test_rx_endpoint_cascade_synthesizes_feed_stage() {
        // T_ant=265 K on the port, feed=3 dB on the port, chain: single stage
        // NF=5 dB / G=20 dB. Friis with the synthesized feed:
        // T_sys = T_ant + 290·(L−1) + T_rx/(1/L)
        let chain = CascadeReceiver {
            band: ka_band(),
            stages: vec![NoiseStage {
                gain: 20.0.db(),
                noise_temperature: noise_figure_to_temperature(5.0.db()),
            }],
            demodulator_loss: 0.0.db(),
            implementation_loss: 0.0.db(),
        };
        let mut payload = CommsPayload::new();
        let antenna = payload.add_antenna(
            "antenna",
            Antenna::Constant(ConstantAntenna { gain: 30.0.db() }),
        );
        let rx = payload.add_receiver("receiver", Receiver::Cascade(chain));
        let port = payload
            .add_rx_port(RxPort {
                name: "feed".into(),
                antenna,
                receiver: rx,
                feed_loss: 3.0.db(),
                antenna_noise_temperature: 265.0,
                band: None,
            })
            .unwrap();
        let terminal = payload
            .add_terminal(Terminal {
                name: "rx".into(),
                role: TerminalRole::Rx(RxChain::Component(port)),
            })
            .unwrap();

        let endpoint = payload.rx_endpoint(terminal).unwrap();
        let feed_linear = 10.0_f64.powf(-3.0 / 10.0);
        let t_feed = 290.0 * (10.0_f64.powf(3.0 / 10.0) - 1.0);
        let t_rx = noise_figure_to_temperature(5.0.db());
        let expected = 265.0 + t_feed + t_rx / feed_linear;
        assert_approx_eq!(
            endpoint.system_noise_temperature().unwrap(),
            expected,
            rtol <= 1e-12
        );
    }

    #[test]
    fn test_lumped_endpoints_ignore_carrier_and_pointing() {
        let mut payload = CommsPayload::new();
        let eirp = payload.add_eirp_model(EirpModel {
            name: "eirp".into(),
            band: ka_band(),
            eirp: 55.0.db(),
        });
        let gt = payload.add_gt_model(GtModel {
            name: "gt".into(),
            band: ka_band(),
            gt: 3.01.db(),
        });
        let terminal = payload
            .add_terminal(Terminal {
                name: "lumped".into(),
                role: TerminalRole::Transceiver {
                    tx: TxChain::Lumped(eirp),
                    rx: RxChain::Lumped(gt),
                },
            })
            .unwrap();

        let tx = payload.tx_endpoint(terminal).unwrap();
        let rx = payload.rx_endpoint(terminal).unwrap();
        let pointing = Pointing::off_boresight(Angle::degrees(45.0));
        assert_approx_eq!(
            tx.eirp_at(29.0.ghz(), pointing).unwrap().as_f64(),
            55.0,
            atol <= 1e-12
        );
        assert_approx_eq!(
            rx.gt_at(29.0.ghz(), pointing).unwrap().as_f64(),
            3.01,
            atol <= 1e-12
        );
        assert!(rx.system_noise_temperature().is_none());
        assert!(rx.antenna_noise_temperature().is_none());
        assert!(tx.band().contains(29.0.ghz()));
    }

    #[test]
    fn test_port_band_narrows_radio_band() {
        let mut payload = CommsPayload::new();
        let dish = payload.add_antenna(
            "dish",
            Antenna::Constant(ConstantAntenna { gain: 46.0.db() }),
        );
        let pa =
            payload.add_transmitter("pa", AmplifierTransmitter::new(ka_band(), 10.0, 0.0.db()));
        let narrow = FrequencyRange::new(28.0.ghz(), 29.5.ghz()).unwrap();
        let port = payload
            .add_tx_port(TxPort {
                name: "feed".into(),
                antenna: dish,
                transmitter: pa,
                feed_loss: 0.0.db(),
                band: Some(narrow),
            })
            .unwrap();
        let terminal = payload
            .add_terminal(Terminal {
                name: "tx".into(),
                role: TerminalRole::Tx(TxChain::Component(port)),
            })
            .unwrap();

        let tx = payload.tx_endpoint(terminal).unwrap();
        assert_eq!(tx.band(), narrow);
    }

    #[test]
    fn test_disjoint_radio_and_port_bands_is_error() {
        let mut payload = CommsPayload::new();
        let dish = payload.add_antenna(
            "dish",
            Antenna::Constant(ConstantAntenna { gain: 46.0.db() }),
        );
        let pa =
            payload.add_transmitter("pa", AmplifierTransmitter::new(ka_band(), 10.0, 0.0.db()));
        let disjoint = FrequencyRange::new(17.0.ghz(), 21.0.ghz()).unwrap();
        let port = payload
            .add_tx_port(TxPort {
                name: "feed".into(),
                antenna: dish,
                transmitter: pa,
                feed_loss: 0.0.db(),
                band: Some(disjoint),
            })
            .unwrap();
        let terminal = payload
            .add_terminal(Terminal {
                name: "tx".into(),
                role: TerminalRole::Tx(TxChain::Component(port)),
            })
            .unwrap();

        let err = payload.tx_endpoint(terminal).unwrap_err();
        assert!(matches!(err, ResolveError::EmptyBandIntersection(_)));
        assert!(err.to_string().contains("disjoint"));
    }

    #[test]
    fn test_wrong_direction_terminal_is_error() {
        let mut payload = CommsPayload::new();
        let gt = payload.add_gt_model(GtModel {
            name: "gt".into(),
            band: ka_band(),
            gt: 3.01.db(),
        });
        let rx_only = payload
            .add_terminal(Terminal {
                name: "rx only".into(),
                role: TerminalRole::Rx(RxChain::Lumped(gt)),
            })
            .unwrap();

        let err = payload.tx_endpoint(rx_only).unwrap_err();
        assert!(matches!(err, ResolveError::NotATransmitTerminal(_)));
        assert!(err.to_string().contains("no transmit chain"));
        assert!(payload.rx_endpoint(rx_only).is_ok());
    }

    #[test]
    fn test_unknown_terminal_is_error() {
        let payload = CommsPayload::new();
        let err = payload.tx_endpoint(TerminalId::default()).unwrap_err();
        assert!(matches!(err, ResolveError::UnknownTerminal(_)));
    }
}
