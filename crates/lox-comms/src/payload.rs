// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Communications hardware inventory and wiring for one platform.
//!
//! A [`CommsPayload`] owns the antennas, radios, lumped models, ports, and
//! terminals of a single platform. It is pure inventory plus wiring: no
//! carrier frequency, bandwidth, modulation, coding, or pointing state —
//! those are link-level inputs.
//!
//! Wiring is by ID: ports reference an antenna and a radio, terminals
//! reference ports (or lumped models). All `add_*` methods validate
//! referenced IDs at insertion, so a payload's wiring is valid by
//! construction. Names are display-only and not required to be unique;
//! addressing is always by key.

use lox_core::units::{Decibel, Kelvin};
use slotmap::{SlotMap, new_key_type};
use thiserror::Error;

use crate::antenna::Antenna;
use crate::band::FrequencyRange;
use crate::receiver::Receiver;
use crate::transmitter::AmplifierTransmitter;

new_key_type! {
    /// Identifier of an antenna in a [`CommsPayload`].
    pub struct AntennaId;
    /// Identifier of a component-tier transmitter in a [`CommsPayload`].
    pub struct TransmitterId;
    /// Identifier of a component-tier receiver in a [`CommsPayload`].
    pub struct ReceiverId;
    /// Identifier of a lumped EIRP model in a [`CommsPayload`].
    pub struct EirpModelId;
    /// Identifier of a lumped G/T model in a [`CommsPayload`].
    pub struct GtModelId;
    /// Identifier of a transmit port in a [`CommsPayload`].
    pub struct TxPortId;
    /// Identifier of a receive port in a [`CommsPayload`].
    pub struct RxPortId;
    /// Identifier of a terminal in a [`CommsPayload`].
    pub struct TerminalId;
}

/// A named inventory item.
///
/// Wraps types that do not carry their own `name` field. Names are
/// display-only; wiring is always by ID.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Named<T> {
    /// Display name.
    pub name: String,
    /// The wrapped inventory item.
    pub value: T,
}

/// Lumped transmitter model: an aggregate EIRP figure over a band.
///
/// Inventory citizen of the lumped tier; the component tier uses
/// [`AmplifierTransmitter`] wired to an antenna through a [`TxPort`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EirpModel {
    /// Display name.
    pub name: String,
    /// Frequency range over which the figure applies.
    pub band: FrequencyRange,
    /// Effective isotropic radiated power in dBW.
    pub eirp: Decibel,
}

/// Lumped receiver model: an aggregate G/T figure over a band.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GtModel {
    /// Display name.
    pub name: String,
    /// Frequency range over which the figure applies.
    pub band: FrequencyRange,
    /// Gain-to-noise-temperature ratio in dB/K.
    pub gt: Decibel,
}

/// One transmit signal path: an antenna fed by a transmitter.
///
/// TX feed loss subtracts from EIRP. Multiple ports may reference the same
/// antenna (diplexer) or the same transmitter (switchable antennas).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TxPort {
    /// Display name.
    pub name: String,
    /// The antenna this path radiates from.
    pub antenna: AntennaId,
    /// The transmitter driving this path.
    pub transmitter: TransmitterId,
    /// Feed loss between transmitter output and antenna (cable, diplexer leg).
    pub feed_loss: Decibel,
    /// Optional narrowing of the supported frequency range for this path.
    pub band: Option<FrequencyRange>,
}

/// One receive signal path: a receiver fed by an antenna.
///
/// RX feed loss is a noise contribution: link-budget setup synthesizes a
/// passive 290 K attenuator stage from it ahead of the receiver chain.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RxPort {
    /// Display name.
    pub name: String,
    /// The antenna this path receives from.
    pub antenna: AntennaId,
    /// The receiver terminating this path.
    pub receiver: ReceiverId,
    /// Feed loss between antenna and receiver input (cable, diplexer leg).
    pub feed_loss: Decibel,
    /// Clear-sky antenna noise temperature seen at this port, in Kelvin.
    pub antenna_noise_temperature: Kelvin,
    /// Optional narrowing of the supported frequency range for this path.
    pub band: Option<FrequencyRange>,
}

/// A transmit chain: a component-tier port or a lumped EIRP model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TxChain {
    /// Component tier: transmitter + feed + antenna via a [`TxPort`].
    Component(TxPortId),
    /// Lumped tier: aggregate EIRP figure.
    Lumped(EirpModelId),
}

/// A receive chain: a component-tier port or a lumped G/T model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RxChain {
    /// Component tier: antenna + feed + receiver via an [`RxPort`].
    Component(RxPortId),
    /// Lumped tier: aggregate G/T figure.
    Lumped(GtModelId),
}

/// The operational role of a terminal: one chain per direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TerminalRole {
    /// Transmit-only terminal.
    Tx(TxChain),
    /// Receive-only terminal.
    Rx(RxChain),
    /// Transceiver terminal with one chain per direction.
    Transceiver {
        /// The transmit chain.
        tx: TxChain,
        /// The receive chain.
        rx: RxChain,
    },
}

/// An operational endpoint exposed by a [`CommsPayload`].
///
/// Terminals are what mission operations and link analysis address. A
/// switchable-antenna transmitter is modelled as multiple terminals sharing
/// one transmitter through different ports — selecting the terminal *is* the
/// switch state, so the payload itself stays stateless.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Terminal {
    /// Display name.
    pub name: String,
    /// The chains this terminal exposes.
    pub role: TerminalRole,
}

/// Communications hardware inventory and wiring for one platform.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommsPayload {
    antennas: SlotMap<AntennaId, Named<Antenna>>,
    transmitters: SlotMap<TransmitterId, Named<AmplifierTransmitter>>,
    receivers: SlotMap<ReceiverId, Named<Receiver>>,
    eirp_models: SlotMap<EirpModelId, EirpModel>,
    gt_models: SlotMap<GtModelId, GtModel>,
    tx_ports: SlotMap<TxPortId, TxPort>,
    rx_ports: SlotMap<RxPortId, RxPort>,
    terminals: SlotMap<TerminalId, Terminal>,
}

impl CommsPayload {
    /// Creates an empty payload.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an antenna to the inventory.
    pub fn add_antenna(&mut self, name: impl Into<String>, antenna: Antenna) -> AntennaId {
        self.antennas.insert(Named {
            name: name.into(),
            value: antenna,
        })
    }

    /// Adds a component-tier transmitter to the inventory.
    pub fn add_transmitter(
        &mut self,
        name: impl Into<String>,
        transmitter: AmplifierTransmitter,
    ) -> TransmitterId {
        self.transmitters.insert(Named {
            name: name.into(),
            value: transmitter,
        })
    }

    /// Adds a component-tier receiver to the inventory.
    ///
    /// Returns [`PayloadError::LumpedReceiverInInventory`] for
    /// [`Receiver::Gt`]: lumped G/T figures belong in [`Self::add_gt_model`].
    pub fn add_receiver(
        &mut self,
        name: impl Into<String>,
        receiver: Receiver,
    ) -> Result<ReceiverId, PayloadError> {
        if matches!(receiver, Receiver::Gt(_)) {
            return Err(PayloadError::LumpedReceiverInInventory);
        }
        Ok(self.receivers.insert(Named {
            name: name.into(),
            value: receiver,
        }))
    }

    /// Adds a lumped EIRP model to the inventory.
    pub fn add_eirp_model(&mut self, model: EirpModel) -> EirpModelId {
        self.eirp_models.insert(model)
    }

    /// Adds a lumped G/T model to the inventory.
    pub fn add_gt_model(&mut self, model: GtModel) -> GtModelId {
        self.gt_models.insert(model)
    }

    /// Adds a transmit port, validating that the referenced antenna and
    /// transmitter exist in this payload.
    pub fn add_tx_port(&mut self, port: TxPort) -> Result<TxPortId, PayloadError> {
        if !self.antennas.contains_key(port.antenna) {
            return Err(PayloadError::UnknownAntenna(port.antenna));
        }
        if !self.transmitters.contains_key(port.transmitter) {
            return Err(PayloadError::UnknownTransmitter(port.transmitter));
        }
        Ok(self.tx_ports.insert(port))
    }

    /// Adds a receive port, validating that the referenced antenna and
    /// receiver exist in this payload.
    pub fn add_rx_port(&mut self, port: RxPort) -> Result<RxPortId, PayloadError> {
        if !self.antennas.contains_key(port.antenna) {
            return Err(PayloadError::UnknownAntenna(port.antenna));
        }
        if !self.receivers.contains_key(port.receiver) {
            return Err(PayloadError::UnknownReceiver(port.receiver));
        }
        Ok(self.rx_ports.insert(port))
    }

    /// Adds a terminal, validating that all referenced chains exist in this
    /// payload.
    pub fn add_terminal(&mut self, terminal: Terminal) -> Result<TerminalId, PayloadError> {
        match terminal.role {
            TerminalRole::Tx(tx) => self.validate_tx_chain(tx)?,
            TerminalRole::Rx(rx) => self.validate_rx_chain(rx)?,
            TerminalRole::Transceiver { tx, rx } => {
                self.validate_tx_chain(tx)?;
                self.validate_rx_chain(rx)?;
            }
        }
        Ok(self.terminals.insert(terminal))
    }

    fn validate_tx_chain(&self, chain: TxChain) -> Result<(), PayloadError> {
        match chain {
            TxChain::Component(port) if !self.tx_ports.contains_key(port) => {
                Err(PayloadError::UnknownTxPort(port))
            }
            TxChain::Lumped(model) if !self.eirp_models.contains_key(model) => {
                Err(PayloadError::UnknownEirpModel(model))
            }
            _ => Ok(()),
        }
    }

    fn validate_rx_chain(&self, chain: RxChain) -> Result<(), PayloadError> {
        match chain {
            RxChain::Component(port) if !self.rx_ports.contains_key(port) => {
                Err(PayloadError::UnknownRxPort(port))
            }
            RxChain::Lumped(model) if !self.gt_models.contains_key(model) => {
                Err(PayloadError::UnknownGtModel(model))
            }
            _ => Ok(()),
        }
    }

    /// Returns the antenna with the given ID.
    pub fn antenna(&self, id: AntennaId) -> Option<&Named<Antenna>> {
        self.antennas.get(id)
    }

    /// Returns the transmitter with the given ID.
    pub fn transmitter(&self, id: TransmitterId) -> Option<&Named<AmplifierTransmitter>> {
        self.transmitters.get(id)
    }

    /// Returns the receiver with the given ID.
    pub fn receiver(&self, id: ReceiverId) -> Option<&Named<Receiver>> {
        self.receivers.get(id)
    }

    /// Returns the lumped EIRP model with the given ID.
    pub fn eirp_model(&self, id: EirpModelId) -> Option<&EirpModel> {
        self.eirp_models.get(id)
    }

    /// Returns the lumped G/T model with the given ID.
    pub fn gt_model(&self, id: GtModelId) -> Option<&GtModel> {
        self.gt_models.get(id)
    }

    /// Returns the transmit port with the given ID.
    pub fn tx_port(&self, id: TxPortId) -> Option<&TxPort> {
        self.tx_ports.get(id)
    }

    /// Returns the receive port with the given ID.
    pub fn rx_port(&self, id: RxPortId) -> Option<&RxPort> {
        self.rx_ports.get(id)
    }

    /// Returns the terminal with the given ID.
    pub fn terminal(&self, id: TerminalId) -> Option<&Terminal> {
        self.terminals.get(id)
    }

    /// Iterates over all terminals.
    pub fn terminals(&self) -> impl Iterator<Item = (TerminalId, &Terminal)> {
        self.terminals.iter()
    }

    /// Returns the first terminal with the given name, if any.
    ///
    /// Names are not unique; this is a convenience for scripts and tests.
    pub fn find_terminal(&self, name: &str) -> Option<TerminalId> {
        self.terminals
            .iter()
            .find(|(_, terminal)| terminal.name == name)
            .map(|(id, _)| id)
    }

    /// Returns the first antenna with the given name, if any.
    pub fn find_antenna(&self, name: &str) -> Option<AntennaId> {
        self.antennas
            .iter()
            .find(|(_, antenna)| antenna.name == name)
            .map(|(id, _)| id)
    }
}

/// Errors produced while assembling a [`CommsPayload`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum PayloadError {
    /// Lumped G/T figures are modelled as [`GtModel`]s, not receivers.
    #[error("lumped G/T receivers belong in gt_models, not the receiver inventory")]
    LumpedReceiverInInventory,
    /// The referenced antenna does not exist in this payload.
    #[error("unknown antenna ID {0:?}")]
    UnknownAntenna(AntennaId),
    /// The referenced transmitter does not exist in this payload.
    #[error("unknown transmitter ID {0:?}")]
    UnknownTransmitter(TransmitterId),
    /// The referenced receiver does not exist in this payload.
    #[error("unknown receiver ID {0:?}")]
    UnknownReceiver(ReceiverId),
    /// The referenced EIRP model does not exist in this payload.
    #[error("unknown EIRP model ID {0:?}")]
    UnknownEirpModel(EirpModelId),
    /// The referenced G/T model does not exist in this payload.
    #[error("unknown G/T model ID {0:?}")]
    UnknownGtModel(GtModelId),
    /// The referenced transmit port does not exist in this payload.
    #[error("unknown TX port ID {0:?}")]
    UnknownTxPort(TxPortId),
    /// The referenced receive port does not exist in this payload.
    #[error("unknown RX port ID {0:?}")]
    UnknownRxPort(RxPortId),
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits};

    use crate::antenna::ConstantAntenna;
    use crate::receiver::{GtReceiver, NoiseTempReceiver};

    use super::*;

    fn ka_band() -> FrequencyRange {
        FrequencyRange::new(27.0.ghz(), 31.0.ghz()).unwrap()
    }

    /// One dish, one TX, one RX through a diplexer, exposed as one
    /// transceiver terminal — the canonical sharing example.
    fn diplexer_payload() -> (CommsPayload, TerminalId) {
        let mut payload = CommsPayload::new();
        let dish = payload.add_antenna(
            "dish",
            Antenna::Constant(ConstantAntenna { gain: 46.0.db() }),
        );
        let tx = payload.add_transmitter(
            "pa",
            AmplifierTransmitter::new(29.0.ghz(), 10.0, 0.0.db(), 0.0.db()),
        );
        let rx = payload
            .add_receiver(
                "lnb",
                Receiver::NoiseTemperature(NoiseTempReceiver {
                    frequency: 19.7.ghz(),
                    system_noise_temperature: 500.0,
                }),
            )
            .unwrap();
        let tx_port = payload
            .add_tx_port(TxPort {
                name: "diplexer tx leg".into(),
                antenna: dish,
                transmitter: tx,
                feed_loss: 1.0.db(),
                band: Some(ka_band()),
            })
            .unwrap();
        let rx_port = payload
            .add_rx_port(RxPort {
                name: "diplexer rx leg".into(),
                antenna: dish,
                receiver: rx,
                feed_loss: 0.5.db(),
                antenna_noise_temperature: 150.0,
                band: None,
            })
            .unwrap();
        let terminal = payload
            .add_terminal(Terminal {
                name: "ka transceiver".into(),
                role: TerminalRole::Transceiver {
                    tx: TxChain::Component(tx_port),
                    rx: RxChain::Component(rx_port),
                },
            })
            .unwrap();
        (payload, terminal)
    }

    #[test]
    fn test_diplexer_payload_shares_one_antenna() {
        let (payload, terminal) = diplexer_payload();
        let TerminalRole::Transceiver { tx, rx } = payload.terminal(terminal).unwrap().role else {
            panic!("expected transceiver terminal");
        };
        let TxChain::Component(tx_port) = tx else {
            panic!("expected component TX chain");
        };
        let RxChain::Component(rx_port) = rx else {
            panic!("expected component RX chain");
        };
        // Both ports reference the same antenna.
        assert_eq!(
            payload.tx_port(tx_port).unwrap().antenna,
            payload.rx_port(rx_port).unwrap().antenna
        );
    }

    #[test]
    fn test_switchable_antenna_tx_is_two_terminals() {
        let mut payload = CommsPayload::new();
        let high_gain = payload.add_antenna(
            "high gain",
            Antenna::Constant(ConstantAntenna { gain: 46.0.db() }),
        );
        let low_gain = payload.add_antenna(
            "low gain",
            Antenna::Constant(ConstantAntenna { gain: 6.0.db() }),
        );
        let tx = payload.add_transmitter(
            "pa",
            AmplifierTransmitter::new(29.0.ghz(), 10.0, 0.0.db(), 0.0.db()),
        );
        let high_port = payload
            .add_tx_port(TxPort {
                name: "hga path".into(),
                antenna: high_gain,
                transmitter: tx,
                feed_loss: 1.0.db(),
                band: None,
            })
            .unwrap();
        let low_port = payload
            .add_tx_port(TxPort {
                name: "lga path".into(),
                antenna: low_gain,
                transmitter: tx,
                feed_loss: 0.2.db(),
                band: None,
            })
            .unwrap();
        let hga = payload
            .add_terminal(Terminal {
                name: "hga".into(),
                role: TerminalRole::Tx(TxChain::Component(high_port)),
            })
            .unwrap();
        let lga = payload
            .add_terminal(Terminal {
                name: "lga".into(),
                role: TerminalRole::Tx(TxChain::Component(low_port)),
            })
            .unwrap();

        assert_ne!(hga, lga);
        assert_eq!(payload.find_terminal("hga"), Some(hga));
        // Both ports share one transmitter.
        assert_eq!(
            payload.tx_port(high_port).unwrap().transmitter,
            payload.tx_port(low_port).unwrap().transmitter
        );
    }

    #[test]
    fn test_lumped_models_are_inventory_citizens() {
        let mut payload = CommsPayload::new();
        let eirp = payload.add_eirp_model(EirpModel {
            name: "datasheet eirp".into(),
            band: ka_band(),
            eirp: 55.0.db(),
        });
        let gt = payload.add_gt_model(GtModel {
            name: "datasheet gt".into(),
            band: ka_band(),
            gt: 3.01.db(),
        });
        let terminal = payload
            .add_terminal(Terminal {
                name: "lumped transceiver".into(),
                role: TerminalRole::Transceiver {
                    tx: TxChain::Lumped(eirp),
                    rx: RxChain::Lumped(gt),
                },
            })
            .unwrap();
        assert!(payload.terminal(terminal).is_some());
        assert_eq!(payload.eirp_model(eirp).unwrap().name, "datasheet eirp");
        assert_eq!(payload.gt_model(gt).unwrap().name, "datasheet gt");
    }

    #[test]
    fn test_add_receiver_rejects_lumped_gt() {
        let mut payload = CommsPayload::new();
        let err = payload
            .add_receiver(
                "wrong tier",
                Receiver::Gt(GtReceiver {
                    frequency: 29.0.ghz(),
                    gt: 3.01.db(),
                }),
            )
            .unwrap_err();
        assert_eq!(err, PayloadError::LumpedReceiverInInventory);
    }

    #[test]
    fn test_add_port_rejects_dangling_keys() {
        let mut payload = CommsPayload::new();
        let tx = payload.add_transmitter(
            "pa",
            AmplifierTransmitter::new(29.0.ghz(), 10.0, 0.0.db(), 0.0.db()),
        );
        let err = payload
            .add_tx_port(TxPort {
                name: "dangling antenna".into(),
                antenna: AntennaId::default(),
                transmitter: tx,
                feed_loss: 0.0.db(),
                band: None,
            })
            .unwrap_err();
        assert!(matches!(err, PayloadError::UnknownAntenna(_)));
    }

    #[test]
    fn test_add_terminal_rejects_dangling_chain() {
        let mut payload = CommsPayload::new();
        let err = payload
            .add_terminal(Terminal {
                name: "dangling".into(),
                role: TerminalRole::Tx(TxChain::Component(TxPortId::default())),
            })
            .unwrap_err();
        assert!(matches!(err, PayloadError::UnknownTxPort(_)));

        let err = payload
            .add_terminal(Terminal {
                name: "dangling lumped".into(),
                role: TerminalRole::Rx(RxChain::Lumped(GtModelId::default())),
            })
            .unwrap_err();
        assert!(matches!(err, PayloadError::UnknownGtModel(_)));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_payload_serde_round_trip_preserves_wiring() {
        let (payload, terminal) = diplexer_payload();
        let json = serde_json::to_string(&payload).unwrap();
        let round_trip: CommsPayload = serde_json::from_str(&json).unwrap();

        // The terminal ID minted before serialization stays valid after.
        let restored = round_trip.terminal(terminal).unwrap();
        assert_eq!(restored.name, "ka transceiver");
        let TerminalRole::Transceiver { tx, .. } = restored.role else {
            panic!("expected transceiver terminal");
        };
        let TxChain::Component(tx_port) = tx else {
            panic!("expected component TX chain");
        };
        assert_eq!(round_trip.tx_port(tx_port).unwrap().name, "diplexer tx leg");
    }
}
