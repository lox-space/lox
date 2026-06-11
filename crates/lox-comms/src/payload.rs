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

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use lox_core::units::{Decibel, Temperature};
use slotmap::{SlotMap, new_key_type};
use thiserror::Error;

use crate::antenna::Antenna;
use crate::band::FrequencyRange;
use crate::error::NonPhysicalError;
use crate::receiver::Receiver;
use crate::transmitter::AmplifierTransmitter;

new_key_type! {
    struct AntennaKey;
    struct TransmitterKey;
    struct ReceiverKey;
    struct EirpModelKey;
    struct GtModelKey;
    struct TxPortKey;
    struct RxPortKey;
    struct TerminalKey;
}

/// Identity of the [`CommsPayload`] that minted an ID.
///
/// Fresh payloads draw a process-unique tag and clones share it, so IDs
/// remain valid across clones but are rejected by any other payload.
/// Deserialization mints a new tag. The default tag matches no payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct PayloadTag(u64);

impl PayloadTag {
    fn mint() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

macro_rules! payload_id_type {
    ($(#[$doc:meta])* $id:ident($key:ident)) => {
        $(#[$doc])*
        ///
        /// Only meaningful for the [`CommsPayload`] that minted it (or a
        /// clone of it): every other payload rejects it. Deserialization
        /// mints a fresh payload identity, so IDs issued before
        /// serialization are rejected as well; look inventory up again by
        /// name instead. The default value matches no payload.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        #[cfg_attr(
            feature = "serde",
            derive(serde::Serialize, serde::Deserialize),
            serde(transparent)
        )]
        pub struct $id {
            #[cfg_attr(feature = "serde", serde(skip))]
            tag: PayloadTag,
            key: $key,
        }
    };
}

payload_id_type!(
    /// Identifier of an antenna in a [`CommsPayload`].
    AntennaId(AntennaKey)
);
payload_id_type!(
    /// Identifier of a component-tier transmitter in a [`CommsPayload`].
    TransmitterId(TransmitterKey)
);
payload_id_type!(
    /// Identifier of a component-tier receiver in a [`CommsPayload`].
    ReceiverId(ReceiverKey)
);
payload_id_type!(
    /// Identifier of a lumped EIRP model in a [`CommsPayload`].
    EirpModelId(EirpModelKey)
);
payload_id_type!(
    /// Identifier of a lumped G/T model in a [`CommsPayload`].
    GtModelId(GtModelKey)
);
payload_id_type!(
    /// Identifier of a transmit port in a [`CommsPayload`].
    TxPortId(TxPortKey)
);
payload_id_type!(
    /// Identifier of a receive port in a [`CommsPayload`].
    RxPortId(RxPortKey)
);
payload_id_type!(
    /// Identifier of a terminal in a [`CommsPayload`].
    TerminalId(TerminalKey)
);

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
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "EirpModelRepr")
)]
pub struct EirpModel {
    name: String,
    band: FrequencyRange,
    eirp: Decibel,
}

/// Serde wire format for [`EirpModel`]: forces deserialization through the
/// validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct EirpModelRepr {
    name: String,
    band: FrequencyRange,
    eirp: Decibel,
}

#[cfg(feature = "serde")]
impl TryFrom<EirpModelRepr> for EirpModel {
    type Error = NonPhysicalError;

    fn try_from(repr: EirpModelRepr) -> Result<Self, Self::Error> {
        EirpModel::new(repr.name, repr.band, repr.eirp)
    }
}

impl EirpModel {
    /// Creates a new lumped EIRP model.
    ///
    /// Rejects a non-finite EIRP figure (negative dBW values are valid).
    pub fn new(
        name: impl Into<String>,
        band: FrequencyRange,
        eirp: Decibel,
    ) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_finite("EIRP [dBW]", eirp.as_f64())?;
        Ok(Self {
            name: name.into(),
            band,
            eirp,
        })
    }

    /// Returns the display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the frequency range over which the figure applies.
    pub fn band(&self) -> FrequencyRange {
        self.band
    }

    /// Returns the effective isotropic radiated power in dBW.
    pub fn eirp(&self) -> Decibel {
        self.eirp
    }
}

/// Lumped receiver model: an aggregate G/T figure over a band.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "GtModelRepr")
)]
pub struct GtModel {
    name: String,
    band: FrequencyRange,
    gt: Decibel,
}

/// Serde wire format for [`GtModel`]: forces deserialization through the
/// validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct GtModelRepr {
    name: String,
    band: FrequencyRange,
    gt: Decibel,
}

#[cfg(feature = "serde")]
impl TryFrom<GtModelRepr> for GtModel {
    type Error = NonPhysicalError;

    fn try_from(repr: GtModelRepr) -> Result<Self, Self::Error> {
        GtModel::new(repr.name, repr.band, repr.gt)
    }
}

impl GtModel {
    /// Creates a new lumped G/T model.
    ///
    /// Rejects a non-finite G/T figure (negative dB/K values are valid).
    pub fn new(
        name: impl Into<String>,
        band: FrequencyRange,
        gt: Decibel,
    ) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_finite("G/T [dB/K]", gt.as_f64())?;
        Ok(Self {
            name: name.into(),
            band,
            gt,
        })
    }

    /// Returns the display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the frequency range over which the figure applies.
    pub fn band(&self) -> FrequencyRange {
        self.band
    }

    /// Returns the gain-to-noise-temperature ratio in dB/K.
    pub fn gt(&self) -> Decibel {
        self.gt
    }
}

/// One transmit signal path: an antenna fed by a transmitter.
///
/// TX feed loss subtracts from EIRP. Multiple ports may reference the same
/// antenna (diplexer) or the same transmitter (switchable antennas).
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "TxPortRepr")
)]
pub struct TxPort {
    name: String,
    antenna: AntennaId,
    transmitter: TransmitterId,
    feed_loss: Decibel,
    band: Option<FrequencyRange>,
}

/// Serde wire format for [`TxPort`]: forces deserialization through the
/// validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct TxPortRepr {
    name: String,
    antenna: AntennaId,
    transmitter: TransmitterId,
    feed_loss: Decibel,
    band: Option<FrequencyRange>,
}

#[cfg(feature = "serde")]
impl TryFrom<TxPortRepr> for TxPort {
    type Error = NonPhysicalError;

    fn try_from(repr: TxPortRepr) -> Result<Self, Self::Error> {
        TxPort::new(
            repr.name,
            repr.antenna,
            repr.transmitter,
            repr.feed_loss,
            repr.band,
        )
    }
}

/// Builder for [`TxPort`].
///
/// Created via [`TxPort::builder`]. Unset fields default to a lossless feed
/// (0 dB) and an unconstrained band.
#[derive(Debug, Clone)]
pub struct TxPortBuilder {
    name: String,
    antenna: AntennaId,
    transmitter: TransmitterId,
    feed_loss: Decibel,
    band: Option<FrequencyRange>,
}

impl TxPortBuilder {
    /// Sets the feed loss between transmitter output and antenna.
    pub fn feed_loss(mut self, feed_loss: Decibel) -> Self {
        self.feed_loss = feed_loss;
        self
    }

    /// Narrows the supported frequency range for this path.
    pub fn band(mut self, band: FrequencyRange) -> Self {
        self.band = Some(band);
        self
    }

    /// Builds the port, validating its physical parameters.
    pub fn build(self) -> Result<TxPort, NonPhysicalError> {
        TxPort::new(
            self.name,
            self.antenna,
            self.transmitter,
            self.feed_loss,
            self.band,
        )
    }
}

impl TxPort {
    /// Starts building a transmit port wiring `antenna` to `transmitter`.
    ///
    /// Unset fields default to a lossless feed and an unconstrained band.
    pub fn builder(
        name: impl Into<String>,
        antenna: AntennaId,
        transmitter: TransmitterId,
    ) -> TxPortBuilder {
        TxPortBuilder {
            name: name.into(),
            antenna,
            transmitter,
            feed_loss: Decibel::new(0.0),
            band: None,
        }
    }

    /// Creates a new transmit port.
    ///
    /// Rejects a non-finite or negative feed loss.
    pub fn new(
        name: impl Into<String>,
        antenna: AntennaId,
        transmitter: TransmitterId,
        feed_loss: Decibel,
        band: Option<FrequencyRange>,
    ) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_non_negative("feed loss [dB]", feed_loss.as_f64())?;
        Ok(Self {
            name: name.into(),
            antenna,
            transmitter,
            feed_loss,
            band,
        })
    }

    /// Returns the display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the antenna this path radiates from.
    pub fn antenna(&self) -> AntennaId {
        self.antenna
    }

    /// Returns the transmitter driving this path.
    pub fn transmitter(&self) -> TransmitterId {
        self.transmitter
    }

    /// Returns the feed loss between transmitter output and antenna.
    pub fn feed_loss(&self) -> Decibel {
        self.feed_loss
    }

    /// Returns the optional narrowing of the supported frequency range.
    pub fn band(&self) -> Option<FrequencyRange> {
        self.band
    }
}

/// One receive signal path: a receiver fed by an antenna.
///
/// RX feed loss is a noise contribution: link-budget setup synthesizes a
/// passive 290 K attenuator stage from it ahead of the receiver chain.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "RxPortRepr")
)]
pub struct RxPort {
    name: String,
    antenna: AntennaId,
    receiver: ReceiverId,
    feed_loss: Decibel,
    antenna_noise_temperature: Temperature,
    band: Option<FrequencyRange>,
}

/// Serde wire format for [`RxPort`]: forces deserialization through the
/// validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct RxPortRepr {
    name: String,
    antenna: AntennaId,
    receiver: ReceiverId,
    feed_loss: Decibel,
    antenna_noise_temperature: Temperature,
    band: Option<FrequencyRange>,
}

#[cfg(feature = "serde")]
impl TryFrom<RxPortRepr> for RxPort {
    type Error = NonPhysicalError;

    fn try_from(repr: RxPortRepr) -> Result<Self, Self::Error> {
        RxPort::new(
            repr.name,
            repr.antenna,
            repr.receiver,
            repr.feed_loss,
            repr.antenna_noise_temperature,
            repr.band,
        )
    }
}

/// Builder for [`RxPort`].
///
/// Created via [`RxPort::builder`]. Unset fields default to a lossless feed
/// (0 dB) and an unconstrained band.
#[derive(Debug, Clone)]
pub struct RxPortBuilder {
    name: String,
    antenna: AntennaId,
    receiver: ReceiverId,
    feed_loss: Decibel,
    antenna_noise_temperature: Temperature,
    band: Option<FrequencyRange>,
}

impl RxPortBuilder {
    /// Sets the feed loss between antenna and receiver input.
    pub fn feed_loss(mut self, feed_loss: Decibel) -> Self {
        self.feed_loss = feed_loss;
        self
    }

    /// Narrows the supported frequency range for this path.
    pub fn band(mut self, band: FrequencyRange) -> Self {
        self.band = Some(band);
        self
    }

    /// Builds the port, validating its physical parameters.
    pub fn build(self) -> Result<RxPort, NonPhysicalError> {
        RxPort::new(
            self.name,
            self.antenna,
            self.receiver,
            self.feed_loss,
            self.antenna_noise_temperature,
            self.band,
        )
    }
}

impl RxPort {
    /// Starts building a receive port wiring `antenna` to `receiver`.
    ///
    /// Unset fields default to a lossless feed and an unconstrained band.
    pub fn builder(
        name: impl Into<String>,
        antenna: AntennaId,
        receiver: ReceiverId,
        antenna_noise_temperature: Temperature,
    ) -> RxPortBuilder {
        RxPortBuilder {
            name: name.into(),
            antenna,
            receiver,
            feed_loss: Decibel::new(0.0),
            antenna_noise_temperature,
            band: None,
        }
    }

    /// Creates a new receive port.
    ///
    /// Rejects a non-finite or negative feed loss or antenna noise
    /// temperature.
    pub fn new(
        name: impl Into<String>,
        antenna: AntennaId,
        receiver: ReceiverId,
        feed_loss: Decibel,
        antenna_noise_temperature: Temperature,
        band: Option<FrequencyRange>,
    ) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_non_negative("feed loss [dB]", feed_loss.as_f64())?;
        NonPhysicalError::check_non_negative(
            "antenna noise temperature [K]",
            antenna_noise_temperature.to_kelvin(),
        )?;
        Ok(Self {
            name: name.into(),
            antenna,
            receiver,
            feed_loss,
            antenna_noise_temperature,
            band,
        })
    }

    /// Returns the display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the antenna this path receives from.
    pub fn antenna(&self) -> AntennaId {
        self.antenna
    }

    /// Returns the receiver terminating this path.
    pub fn receiver(&self) -> ReceiverId {
        self.receiver
    }

    /// Returns the feed loss between antenna and receiver input.
    pub fn feed_loss(&self) -> Decibel {
        self.feed_loss
    }

    /// Returns the clear-sky antenna noise temperature at this port.
    pub fn antenna_noise_temperature(&self) -> Temperature {
        self.antenna_noise_temperature
    }

    /// Returns the optional narrowing of the supported frequency range.
    pub fn band(&self) -> Option<FrequencyRange> {
        self.band
    }
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

/// An operational identity exposed by a [`CommsPayload`].
///
/// Terminals are the durable, named configuration that mission operations
/// and link analysis address; resolve one into a borrowed
/// [`ResolvedTxTerminal`](crate::resolve::ResolvedTxTerminal) or
/// [`ResolvedRxTerminal`](crate::resolve::ResolvedRxTerminal) view for
/// link-budget evaluation. A switchable-antenna transmitter is modelled as
/// multiple terminals sharing one transmitter through different ports —
/// selecting the terminal *is* the switch state, so the payload itself
/// stays stateless.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Terminal {
    /// Display name.
    pub name: String,
    /// The chains this terminal exposes.
    pub role: TerminalRole,
}

impl Terminal {
    /// Creates a transmit-only terminal.
    pub fn tx(name: impl Into<String>, chain: TxChain) -> Self {
        Self {
            name: name.into(),
            role: TerminalRole::Tx(chain),
        }
    }

    /// Creates a receive-only terminal.
    pub fn rx(name: impl Into<String>, chain: RxChain) -> Self {
        Self {
            name: name.into(),
            role: TerminalRole::Rx(chain),
        }
    }

    /// Creates a transceiver terminal with one chain per direction.
    pub fn transceiver(name: impl Into<String>, tx: TxChain, rx: RxChain) -> Self {
        Self {
            name: name.into(),
            role: TerminalRole::Transceiver { tx, rx },
        }
    }
}

/// Communications hardware inventory and wiring for one platform.
///
/// IDs are scoped to the payload that minted them (clones included);
/// methods reject IDs minted by a different payload.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "CommsPayloadRepr")
)]
pub struct CommsPayload {
    #[cfg_attr(feature = "serde", serde(skip))]
    tag: PayloadTag,
    antennas: SlotMap<AntennaKey, Named<Antenna>>,
    transmitters: SlotMap<TransmitterKey, Named<AmplifierTransmitter>>,
    receivers: SlotMap<ReceiverKey, Named<Receiver>>,
    eirp_models: SlotMap<EirpModelKey, EirpModel>,
    gt_models: SlotMap<GtModelKey, GtModel>,
    tx_ports: SlotMap<TxPortKey, TxPort>,
    rx_ports: SlotMap<RxPortKey, RxPort>,
    terminals: SlotMap<TerminalKey, Terminal>,
}

impl Default for CommsPayload {
    fn default() -> Self {
        Self {
            tag: PayloadTag::mint(),
            antennas: SlotMap::with_key(),
            transmitters: SlotMap::with_key(),
            receivers: SlotMap::with_key(),
            eirp_models: SlotMap::with_key(),
            gt_models: SlotMap::with_key(),
            tx_ports: SlotMap::with_key(),
            rx_ports: SlotMap::with_key(),
            terminals: SlotMap::with_key(),
        }
    }
}

/// Serde wire format for [`CommsPayload`]: mirrors the field layout so the
/// representation is unchanged, but forces deserialization through
/// [`CommsPayload::validate`] so persisted inventories uphold the same
/// invariants as the construction API. The deserialized payload carries a
/// fresh identity: IDs minted before serialization are rejected.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct CommsPayloadRepr {
    antennas: SlotMap<AntennaKey, Named<Antenna>>,
    transmitters: SlotMap<TransmitterKey, Named<AmplifierTransmitter>>,
    receivers: SlotMap<ReceiverKey, Named<Receiver>>,
    eirp_models: SlotMap<EirpModelKey, EirpModel>,
    gt_models: SlotMap<GtModelKey, GtModel>,
    tx_ports: SlotMap<TxPortKey, TxPort>,
    rx_ports: SlotMap<RxPortKey, RxPort>,
    terminals: SlotMap<TerminalKey, Terminal>,
}

#[cfg(feature = "serde")]
impl TryFrom<CommsPayloadRepr> for CommsPayload {
    type Error = PayloadError;

    fn try_from(repr: CommsPayloadRepr) -> Result<Self, Self::Error> {
        let mut payload = CommsPayload {
            tag: PayloadTag::mint(),
            antennas: repr.antennas,
            transmitters: repr.transmitters,
            receivers: repr.receivers,
            eirp_models: repr.eirp_models,
            gt_models: repr.gt_models,
            tx_ports: repr.tx_ports,
            rx_ports: repr.rx_ports,
            terminals: repr.terminals,
        };
        payload.retag();
        payload.validate()?;
        Ok(payload)
    }
}

impl CommsPayload {
    /// Creates an empty payload.
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts building a single-terminal transmit-only payload.
    ///
    /// Wires `antenna` to `transmitter` through one TX port and exposes it
    /// as a terminal named `name`. Unset fields default to a lossless feed
    /// and an unconstrained band.
    pub fn transmitter_only(
        name: impl Into<String>,
        antenna: Antenna,
        transmitter: AmplifierTransmitter,
    ) -> TransmitterOnlyBuilder {
        TransmitterOnlyBuilder {
            name: name.into(),
            antenna,
            transmitter,
            feed_loss: Decibel::new(0.0),
            band: None,
        }
    }

    /// Starts building a single-terminal receive-only payload.
    ///
    /// Wires `antenna` to `receiver` through one RX port and exposes it as a
    /// terminal named `name`. Unset fields default to a lossless feed and an
    /// unconstrained band.
    pub fn receiver_only(
        name: impl Into<String>,
        antenna: Antenna,
        receiver: impl Into<Receiver>,
        antenna_noise_temperature: Temperature,
    ) -> ReceiverOnlyBuilder {
        ReceiverOnlyBuilder {
            name: name.into(),
            antenna,
            receiver: receiver.into(),
            feed_loss: Decibel::new(0.0),
            antenna_noise_temperature,
            band: None,
        }
    }

    /// Starts building a single-terminal transceiver payload sharing one
    /// antenna.
    ///
    /// Wires `antenna` to both `transmitter` and `receiver` through one TX
    /// and one RX port (diplexer-style) and exposes them as one transceiver
    /// terminal named `name`. Unset fields default to lossless feeds and an
    /// unconstrained band.
    pub fn transceiver(
        name: impl Into<String>,
        antenna: Antenna,
        transmitter: AmplifierTransmitter,
        receiver: impl Into<Receiver>,
        antenna_noise_temperature: Temperature,
    ) -> TransceiverBuilder {
        TransceiverBuilder {
            name: name.into(),
            antenna,
            transmitter,
            receiver: receiver.into(),
            tx_feed_loss: Decibel::new(0.0),
            rx_feed_loss: Decibel::new(0.0),
            antenna_noise_temperature,
            band: None,
        }
    }

    /// Creates a single-terminal payload from a lumped EIRP model.
    pub fn eirp_only(model: EirpModel) -> (Self, TerminalId) {
        let name = model.name.clone();
        let mut payload = Self::new();
        let model = payload.add_eirp_model(model);
        let terminal = payload
            .add_terminal(Terminal {
                name,
                role: TerminalRole::Tx(TxChain::Lumped(model)),
            })
            .expect("freshly inserted IDs are valid");
        (payload, terminal)
    }

    /// Creates a single-terminal payload from a lumped G/T model.
    pub fn gt_only(model: GtModel) -> (Self, TerminalId) {
        let name = model.name.clone();
        let mut payload = Self::new();
        let model = payload.add_gt_model(model);
        let terminal = payload
            .add_terminal(Terminal {
                name,
                role: TerminalRole::Rx(RxChain::Lumped(model)),
            })
            .expect("freshly inserted IDs are valid");
        (payload, terminal)
    }

    /// Adds an antenna to the inventory.
    pub fn add_antenna(&mut self, name: impl Into<String>, antenna: Antenna) -> AntennaId {
        AntennaId {
            tag: self.tag,
            key: self.antennas.insert(Named {
                name: name.into(),
                value: antenna,
            }),
        }
    }

    /// Adds a component-tier transmitter to the inventory.
    ///
    /// Rejects non-physical parameters: transmit power must be finite and
    /// positive, output back-off finite and non-negative.
    pub fn add_transmitter(
        &mut self,
        name: impl Into<String>,
        transmitter: AmplifierTransmitter,
    ) -> TransmitterId {
        TransmitterId {
            tag: self.tag,
            key: self.transmitters.insert(Named {
                name: name.into(),
                value: transmitter,
            }),
        }
    }

    /// Adds a component-tier receiver to the inventory.
    pub fn add_receiver(&mut self, name: impl Into<String>, receiver: Receiver) -> ReceiverId {
        ReceiverId {
            tag: self.tag,
            key: self.receivers.insert(Named {
                name: name.into(),
                value: receiver,
            }),
        }
    }

    /// Adds a lumped EIRP model to the inventory.
    pub fn add_eirp_model(&mut self, model: EirpModel) -> EirpModelId {
        EirpModelId {
            tag: self.tag,
            key: self.eirp_models.insert(model),
        }
    }

    /// Adds a lumped G/T model to the inventory.
    pub fn add_gt_model(&mut self, model: GtModel) -> GtModelId {
        GtModelId {
            tag: self.tag,
            key: self.gt_models.insert(model),
        }
    }

    /// Adds a transmit port, validating that the referenced antenna and
    /// transmitter exist in this payload.
    pub fn add_tx_port(&mut self, port: TxPort) -> Result<TxPortId, PayloadError> {
        self.validate_tx_port_wiring(&port)?;
        Ok(TxPortId {
            tag: self.tag,
            key: self.tx_ports.insert(port),
        })
    }

    /// Adds a receive port, validating that the referenced antenna and
    /// receiver exist in this payload.
    pub fn add_rx_port(&mut self, port: RxPort) -> Result<RxPortId, PayloadError> {
        self.validate_rx_port_wiring(&port)?;
        Ok(RxPortId {
            tag: self.tag,
            key: self.rx_ports.insert(port),
        })
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
        Ok(TerminalId {
            tag: self.tag,
            key: self.terminals.insert(terminal),
        })
    }

    /// Re-validates the payload's wiring invariants.
    ///
    /// Physical values are valid by construction of the component types;
    /// construction through the `add_*` methods maintains the wiring
    /// invariants incrementally. This checks the whole payload at once,
    /// e.g. after deserialization.
    pub fn validate(&self) -> Result<(), PayloadError> {
        for (_, port) in &self.tx_ports {
            self.validate_tx_port_wiring(port)?;
        }
        for (_, port) in &self.rx_ports {
            self.validate_rx_port_wiring(port)?;
        }
        for (_, terminal) in &self.terminals {
            match terminal.role {
                TerminalRole::Tx(tx) => self.validate_tx_chain(tx)?,
                TerminalRole::Rx(rx) => self.validate_rx_chain(rx)?,
                TerminalRole::Transceiver { tx, rx } => {
                    self.validate_tx_chain(tx)?;
                    self.validate_rx_chain(rx)?;
                }
            }
        }
        Ok(())
    }

    /// Adds a transmit-only terminal, validating the referenced chain.
    pub fn add_tx_terminal(
        &mut self,
        name: impl Into<String>,
        chain: TxChain,
    ) -> Result<TerminalId, PayloadError> {
        self.add_terminal(Terminal::tx(name, chain))
    }

    /// Adds a receive-only terminal, validating the referenced chain.
    pub fn add_rx_terminal(
        &mut self,
        name: impl Into<String>,
        chain: RxChain,
    ) -> Result<TerminalId, PayloadError> {
        self.add_terminal(Terminal::rx(name, chain))
    }

    /// Adds a transceiver terminal, validating the referenced chains.
    pub fn add_transceiver_terminal(
        &mut self,
        name: impl Into<String>,
        tx: TxChain,
        rx: RxChain,
    ) -> Result<TerminalId, PayloadError> {
        self.add_terminal(Terminal::transceiver(name, tx, rx))
    }

    fn validate_tx_port_wiring(&self, port: &TxPort) -> Result<(), PayloadError> {
        if self.antenna(port.antenna).is_none() {
            return Err(PayloadError::UnknownAntenna(port.antenna));
        }
        if self.transmitter(port.transmitter).is_none() {
            return Err(PayloadError::UnknownTransmitter(port.transmitter));
        }
        Ok(())
    }

    fn validate_rx_port_wiring(&self, port: &RxPort) -> Result<(), PayloadError> {
        if self.antenna(port.antenna).is_none() {
            return Err(PayloadError::UnknownAntenna(port.antenna));
        }
        if self.receiver(port.receiver).is_none() {
            return Err(PayloadError::UnknownReceiver(port.receiver));
        }
        Ok(())
    }

    fn validate_tx_chain(&self, chain: TxChain) -> Result<(), PayloadError> {
        match chain {
            TxChain::Component(port) if self.tx_port(port).is_none() => {
                Err(PayloadError::UnknownTxPort(port))
            }
            TxChain::Lumped(model) if self.eirp_model(model).is_none() => {
                Err(PayloadError::UnknownEirpModel(model))
            }
            _ => Ok(()),
        }
    }

    fn validate_rx_chain(&self, chain: RxChain) -> Result<(), PayloadError> {
        match chain {
            RxChain::Component(port) if self.rx_port(port).is_none() => {
                Err(PayloadError::UnknownRxPort(port))
            }
            RxChain::Lumped(model) if self.gt_model(model).is_none() => {
                Err(PayloadError::UnknownGtModel(model))
            }
            _ => Ok(()),
        }
    }

    /// Stamps this payload's identity onto all stored wiring references.
    ///
    /// Deserialization strips ID tags from the wire format; this restores
    /// them against the freshly minted payload identity.
    #[cfg(feature = "serde")]
    fn retag(&mut self) {
        fn retag_tx(chain: &mut TxChain, tag: PayloadTag) {
            match chain {
                TxChain::Component(id) => id.tag = tag,
                TxChain::Lumped(id) => id.tag = tag,
            }
        }
        fn retag_rx(chain: &mut RxChain, tag: PayloadTag) {
            match chain {
                RxChain::Component(id) => id.tag = tag,
                RxChain::Lumped(id) => id.tag = tag,
            }
        }
        let tag = self.tag;
        for port in self.tx_ports.values_mut() {
            port.antenna.tag = tag;
            port.transmitter.tag = tag;
        }
        for port in self.rx_ports.values_mut() {
            port.antenna.tag = tag;
            port.receiver.tag = tag;
        }
        for terminal in self.terminals.values_mut() {
            match &mut terminal.role {
                TerminalRole::Tx(tx) => retag_tx(tx, tag),
                TerminalRole::Rx(rx) => retag_rx(rx, tag),
                TerminalRole::Transceiver { tx, rx } => {
                    retag_tx(tx, tag);
                    retag_rx(rx, tag);
                }
            }
        }
    }

    /// Returns the antenna with the given ID.
    pub fn antenna(&self, id: AntennaId) -> Option<&Named<Antenna>> {
        if id.tag != self.tag {
            return None;
        }
        self.antennas.get(id.key)
    }

    /// Returns the transmitter with the given ID.
    pub fn transmitter(&self, id: TransmitterId) -> Option<&Named<AmplifierTransmitter>> {
        if id.tag != self.tag {
            return None;
        }
        self.transmitters.get(id.key)
    }

    /// Returns the receiver with the given ID.
    pub fn receiver(&self, id: ReceiverId) -> Option<&Named<Receiver>> {
        if id.tag != self.tag {
            return None;
        }
        self.receivers.get(id.key)
    }

    /// Returns the lumped EIRP model with the given ID.
    pub fn eirp_model(&self, id: EirpModelId) -> Option<&EirpModel> {
        if id.tag != self.tag {
            return None;
        }
        self.eirp_models.get(id.key)
    }

    /// Returns the lumped G/T model with the given ID.
    pub fn gt_model(&self, id: GtModelId) -> Option<&GtModel> {
        if id.tag != self.tag {
            return None;
        }
        self.gt_models.get(id.key)
    }

    /// Returns the transmit port with the given ID.
    pub fn tx_port(&self, id: TxPortId) -> Option<&TxPort> {
        if id.tag != self.tag {
            return None;
        }
        self.tx_ports.get(id.key)
    }

    /// Returns the receive port with the given ID.
    pub fn rx_port(&self, id: RxPortId) -> Option<&RxPort> {
        if id.tag != self.tag {
            return None;
        }
        self.rx_ports.get(id.key)
    }

    /// Returns the terminal with the given ID.
    pub fn terminal(&self, id: TerminalId) -> Option<&Terminal> {
        if id.tag != self.tag {
            return None;
        }
        self.terminals.get(id.key)
    }

    /// Iterates over all terminals.
    pub fn terminals(&self) -> impl Iterator<Item = (TerminalId, &Terminal)> {
        let tag = self.tag;
        self.terminals
            .iter()
            .map(move |(key, terminal)| (TerminalId { tag, key }, terminal))
    }

    /// Returns the first terminal with the given name, if any.
    ///
    /// Names are not unique; this is a convenience for scripts and tests.
    pub fn find_terminal(&self, name: &str) -> Option<TerminalId> {
        self.terminals
            .iter()
            .find(|(_, terminal)| terminal.name == name)
            .map(|(key, _)| TerminalId { tag: self.tag, key })
    }

    /// Returns the first antenna with the given name, if any.
    pub fn find_antenna(&self, name: &str) -> Option<AntennaId> {
        self.antennas
            .iter()
            .find(|(_, antenna)| antenna.name == name)
            .map(|(key, _)| AntennaId { tag: self.tag, key })
    }
}

impl CommsPayload {
    fn describe_tx_chain(&self, f: &mut fmt::Formatter<'_>, chain: TxChain) -> fmt::Result {
        match chain {
            TxChain::Component(port_id) => match self.tx_port(port_id) {
                Some(port) => {
                    let antenna = self
                        .antenna(port.antenna)
                        .map_or("<missing>", |a| a.name.as_str());
                    let transmitter = self
                        .transmitter(port.transmitter)
                        .map_or("<missing>", |t| t.name.as_str());
                    write!(
                        f,
                        "port '{}': antenna '{antenna}' ← transmitter '{transmitter}', feed {} dB",
                        port.name,
                        port.feed_loss.as_f64()
                    )?;
                    match port.band {
                        Some(band) => write!(f, ", band {band}"),
                        None => Ok(()),
                    }
                }
                None => write!(f, "port <missing>"),
            },
            TxChain::Lumped(model_id) => match self.eirp_model(model_id) {
                Some(model) => write!(
                    f,
                    "EIRP model '{}': {} dBW, band {}",
                    model.name,
                    model.eirp.as_f64(),
                    model.band
                ),
                None => write!(f, "EIRP model <missing>"),
            },
        }
    }

    fn describe_rx_chain(&self, f: &mut fmt::Formatter<'_>, chain: RxChain) -> fmt::Result {
        match chain {
            RxChain::Component(port_id) => match self.rx_port(port_id) {
                Some(port) => {
                    let antenna = self
                        .antenna(port.antenna)
                        .map_or("<missing>", |a| a.name.as_str());
                    let receiver = self
                        .receiver(port.receiver)
                        .map_or("<missing>", |r| r.name.as_str());
                    write!(
                        f,
                        "port '{}': antenna '{antenna}' → receiver '{receiver}', feed {} dB, T_ant {} K",
                        port.name,
                        port.feed_loss.as_f64(),
                        port.antenna_noise_temperature.to_kelvin()
                    )?;
                    match port.band {
                        Some(band) => write!(f, ", band {band}"),
                        None => Ok(()),
                    }
                }
                None => write!(f, "port <missing>"),
            },
            RxChain::Lumped(model_id) => match self.gt_model(model_id) {
                Some(model) => write!(
                    f,
                    "G/T model '{}': {} dB/K, band {}",
                    model.name,
                    model.gt.as_f64(),
                    model.band
                ),
                None => write!(f, "G/T model <missing>"),
            },
        }
    }
}

impl fmt::Display for CommsPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CommsPayload")?;
        for (_, terminal) in self.terminals() {
            match terminal.role {
                TerminalRole::Tx(tx) => {
                    write!(f, "└─ terminal '{}' (tx): ", terminal.name)?;
                    self.describe_tx_chain(f, tx)?;
                    writeln!(f)?;
                }
                TerminalRole::Rx(rx) => {
                    write!(f, "└─ terminal '{}' (rx): ", terminal.name)?;
                    self.describe_rx_chain(f, rx)?;
                    writeln!(f)?;
                }
                TerminalRole::Transceiver { tx, rx } => {
                    writeln!(f, "└─ terminal '{}' (transceiver)", terminal.name)?;
                    write!(f, "   ├─ tx: ")?;
                    self.describe_tx_chain(f, tx)?;
                    writeln!(f)?;
                    write!(f, "   └─ rx: ")?;
                    self.describe_rx_chain(f, rx)?;
                    writeln!(f)?;
                }
            }
        }
        Ok(())
    }
}

/// Builder for a single-terminal transmit-only [`CommsPayload`].
///
/// Created via [`CommsPayload::transmitter_only`].
#[derive(Debug, Clone)]
pub struct TransmitterOnlyBuilder {
    name: String,
    antenna: Antenna,
    transmitter: AmplifierTransmitter,
    feed_loss: Decibel,
    band: Option<FrequencyRange>,
}

impl TransmitterOnlyBuilder {
    /// Sets the feed loss between transmitter output and antenna.
    pub fn feed_loss(mut self, feed_loss: Decibel) -> Self {
        self.feed_loss = feed_loss;
        self
    }

    /// Narrows the supported frequency range for the path.
    pub fn band(mut self, band: FrequencyRange) -> Self {
        self.band = Some(band);
        self
    }

    /// Builds the payload and returns it with its terminal.
    pub fn build(self) -> Result<(CommsPayload, TerminalId), PayloadError> {
        let mut payload = CommsPayload::new();
        let antenna = payload.add_antenna(format!("{} antenna", self.name), self.antenna);
        let transmitter =
            payload.add_transmitter(format!("{} transmitter", self.name), self.transmitter);
        let port = payload.add_tx_port(TxPort::new(
            format!("{} feed", self.name),
            antenna,
            transmitter,
            self.feed_loss,
            self.band,
        )?)?;
        let terminal = payload.add_tx_terminal(self.name, TxChain::Component(port))?;
        Ok((payload, terminal))
    }
}

/// Builder for a single-terminal receive-only [`CommsPayload`].
///
/// Created via [`CommsPayload::receiver_only`].
#[derive(Debug, Clone)]
pub struct ReceiverOnlyBuilder {
    name: String,
    antenna: Antenna,
    receiver: Receiver,
    feed_loss: Decibel,
    antenna_noise_temperature: Temperature,
    band: Option<FrequencyRange>,
}

impl ReceiverOnlyBuilder {
    /// Sets the feed loss between antenna and receiver input.
    pub fn feed_loss(mut self, feed_loss: Decibel) -> Self {
        self.feed_loss = feed_loss;
        self
    }

    /// Narrows the supported frequency range for the path.
    pub fn band(mut self, band: FrequencyRange) -> Self {
        self.band = Some(band);
        self
    }

    /// Builds the payload and returns it with its terminal.
    pub fn build(self) -> Result<(CommsPayload, TerminalId), PayloadError> {
        let mut payload = CommsPayload::new();
        let antenna = payload.add_antenna(format!("{} antenna", self.name), self.antenna);
        let receiver = payload.add_receiver(format!("{} receiver", self.name), self.receiver);
        let port = payload.add_rx_port(RxPort::new(
            format!("{} feed", self.name),
            antenna,
            receiver,
            self.feed_loss,
            self.antenna_noise_temperature,
            self.band,
        )?)?;
        let terminal = payload.add_rx_terminal(self.name, RxChain::Component(port))?;
        Ok((payload, terminal))
    }
}

/// Builder for a single-terminal transceiver [`CommsPayload`] sharing one
/// antenna.
///
/// Created via [`CommsPayload::transceiver`].
#[derive(Debug, Clone)]
pub struct TransceiverBuilder {
    name: String,
    antenna: Antenna,
    transmitter: AmplifierTransmitter,
    receiver: Receiver,
    tx_feed_loss: Decibel,
    rx_feed_loss: Decibel,
    antenna_noise_temperature: Temperature,
    band: Option<FrequencyRange>,
}

impl TransceiverBuilder {
    /// Sets the feed loss between transmitter output and antenna.
    pub fn tx_feed_loss(mut self, feed_loss: Decibel) -> Self {
        self.tx_feed_loss = feed_loss;
        self
    }

    /// Sets the feed loss between antenna and receiver input.
    pub fn rx_feed_loss(mut self, feed_loss: Decibel) -> Self {
        self.rx_feed_loss = feed_loss;
        self
    }

    /// Narrows the supported frequency range for both paths.
    pub fn band(mut self, band: FrequencyRange) -> Self {
        self.band = Some(band);
        self
    }

    /// Builds the payload and returns it with its terminal.
    pub fn build(self) -> Result<(CommsPayload, TerminalId), PayloadError> {
        let mut payload = CommsPayload::new();
        let antenna = payload.add_antenna(format!("{} antenna", self.name), self.antenna);
        let transmitter =
            payload.add_transmitter(format!("{} transmitter", self.name), self.transmitter);
        let receiver = payload.add_receiver(format!("{} receiver", self.name), self.receiver);
        let tx_port = payload.add_tx_port(TxPort::new(
            format!("{} tx feed", self.name),
            antenna,
            transmitter,
            self.tx_feed_loss,
            self.band,
        )?)?;
        let rx_port = payload.add_rx_port(RxPort::new(
            format!("{} rx feed", self.name),
            antenna,
            receiver,
            self.rx_feed_loss,
            self.antenna_noise_temperature,
            self.band,
        )?)?;
        let terminal = payload.add_transceiver_terminal(
            self.name,
            TxChain::Component(tx_port),
            RxChain::Component(rx_port),
        )?;
        Ok((payload, terminal))
    }
}

/// Errors produced while assembling a [`CommsPayload`].
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum PayloadError {
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
    /// A physical quantity is outside its valid domain.
    #[error(transparent)]
    NonPhysical(#[from] NonPhysicalError),
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits, Power, Temperature};
    use lox_test_utils::assert_approx_eq;

    use crate::antenna::ConstantAntenna;
    use crate::receiver::NoiseTempReceiver;

    use super::*;

    fn ka_band() -> FrequencyRange {
        FrequencyRange::new(17.0.ghz(), 31.0.ghz()).unwrap()
    }

    /// One dish, one TX, one RX through a diplexer, exposed as one
    /// transceiver terminal — the canonical sharing example.
    fn diplexer_payload() -> (CommsPayload, TerminalId) {
        let mut payload = CommsPayload::new();
        let dish = payload.add_antenna(
            "dish",
            Antenna::Constant(ConstantAntenna::new(46.0.db()).unwrap()),
        );
        let tx = payload.add_transmitter(
            "pa",
            AmplifierTransmitter::new(ka_band(), Power::watts(10.0), 0.0.db()).unwrap(),
        );
        let rx = payload.add_receiver(
            "lnb",
            Receiver::NoiseTemperature(
                NoiseTempReceiver::new(ka_band(), Temperature::kelvin(500.0)).unwrap(),
            ),
        );
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
                antenna_noise_temperature: Temperature::kelvin(150.0),
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
            Antenna::Constant(ConstantAntenna::new(46.0.db()).unwrap()),
        );
        let low_gain = payload.add_antenna(
            "low gain",
            Antenna::Constant(ConstantAntenna::new(6.0.db()).unwrap()),
        );
        let tx = payload.add_transmitter(
            "pa",
            AmplifierTransmitter::new(ka_band(), Power::watts(10.0), 0.0.db()).unwrap(),
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
    fn test_port_builders() {
        let mut payload = CommsPayload::new();
        let antenna = payload.add_antenna(
            "dish",
            Antenna::Constant(ConstantAntenna::new(46.0.db()).unwrap()),
        );
        let transmitter = payload.add_transmitter(
            "pa",
            AmplifierTransmitter::new(ka_band(), Power::watts(10.0), 0.0.db()).unwrap(),
        );
        let receiver = payload.add_receiver(
            "rx",
            Receiver::NoiseTemperature(
                NoiseTempReceiver::new(ka_band(), Temperature::kelvin(500.0)).unwrap(),
            ),
        );

        // Defaults: lossless feed, unconstrained band, 0 K antenna temperature.
        let bare = TxPort::builder("bare", antenna, transmitter)
            .build()
            .unwrap();
        assert_approx_eq!(bare.feed_loss().as_f64(), 0.0, atol <= 1e-15);
        assert!(bare.band().is_none());

        let tx_port = payload
            .add_tx_port(
                TxPort::builder("tx feed", antenna, transmitter)
                    .feed_loss(0.8.db())
                    .band(ka_band())
                    .build()
                    .unwrap(),
            )
            .unwrap();
        let rx_port = payload
            .add_rx_port(
                RxPort::builder("rx feed", antenna, receiver, Temperature::kelvin(60.0))
                    .feed_loss(0.3.db())
                    .build()
                    .unwrap(),
            )
            .unwrap();
        assert_approx_eq!(
            payload.tx_port(tx_port).unwrap().feed_loss().as_f64(),
            0.8,
            atol <= 1e-15
        );
        assert_approx_eq!(
            payload
                .rx_port(rx_port)
                .unwrap()
                .antenna_noise_temperature()
                .to_kelvin(),
            60.0,
            atol <= 1e-15
        );
        // Validation still applies at build().
        assert!(
            TxPort::builder("bad", antenna, transmitter)
                .feed_loss((-1.0).db())
                .build()
                .is_err()
        );
    }

    #[test]
    fn test_terminal_constructors_and_helpers() {
        let mut payload = CommsPayload::new();
        let eirp =
            payload.add_eirp_model(EirpModel::new("datasheet", ka_band(), 55.0.db()).unwrap());
        let gt = payload.add_gt_model(GtModel::new("datasheet", ka_band(), 3.01.db()).unwrap());

        let tx = payload
            .add_tx_terminal("tx", TxChain::Lumped(eirp))
            .unwrap();
        let rx = payload.add_rx_terminal("rx", RxChain::Lumped(gt)).unwrap();
        let both = payload
            .add_transceiver_terminal("both", TxChain::Lumped(eirp), RxChain::Lumped(gt))
            .unwrap();

        assert!(matches!(
            payload.terminal(tx).unwrap().role,
            TerminalRole::Tx(_)
        ));
        assert!(matches!(
            payload.terminal(rx).unwrap().role,
            TerminalRole::Rx(_)
        ));
        assert_eq!(payload.terminal(both).unwrap().name, "both");
    }

    #[test]
    fn test_lumped_models_are_inventory_citizens() {
        let mut payload = CommsPayload::new();
        let eirp =
            payload.add_eirp_model(EirpModel::new("datasheet eirp", ka_band(), 55.0.db()).unwrap());
        let gt = payload.add_gt_model(GtModel::new("datasheet gt", ka_band(), 3.01.db()).unwrap());
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
    fn test_add_port_rejects_dangling_keys() {
        let mut payload = CommsPayload::new();
        let tx = payload.add_transmitter(
            "pa",
            AmplifierTransmitter::new(ka_band(), Power::watts(10.0), 0.0.db()).unwrap(),
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

    #[test]
    fn test_transceiver_convenience_constructor() {
        let (payload, terminal) = CommsPayload::transceiver(
            "ka terminal",
            Antenna::Constant(ConstantAntenna::new(46.0.db()).unwrap()),
            AmplifierTransmitter::new(ka_band(), Power::watts(10.0), 0.0.db()).unwrap(),
            NoiseTempReceiver::new(ka_band(), Temperature::kelvin(500.0)).unwrap(),
            Temperature::kelvin(150.0),
        )
        .tx_feed_loss(1.0.db())
        .rx_feed_loss(0.5.db())
        .band(ka_band())
        .build()
        .unwrap();

        let terminal = payload.terminal(terminal).unwrap();
        assert_eq!(terminal.name, "ka terminal");
        let TerminalRole::Transceiver { tx, rx } = terminal.role else {
            panic!("expected transceiver terminal");
        };
        let (TxChain::Component(tx_port), RxChain::Component(rx_port)) = (tx, rx) else {
            panic!("expected component chains");
        };
        // Diplexer-style: both ports share the single antenna.
        assert_eq!(
            payload.tx_port(tx_port).unwrap().antenna,
            payload.rx_port(rx_port).unwrap().antenna
        );
    }

    #[test]
    fn test_lumped_convenience_constructors() {
        let (payload, terminal) =
            CommsPayload::eirp_only(EirpModel::new("datasheet", ka_band(), 55.0.db()).unwrap());
        assert_eq!(payload.terminal(terminal).unwrap().name, "datasheet");

        let (payload, terminal) =
            CommsPayload::gt_only(GtModel::new("datasheet", ka_band(), 3.01.db()).unwrap());
        assert!(matches!(
            payload.terminal(terminal).unwrap().role,
            TerminalRole::Rx(RxChain::Lumped(_))
        ));
    }

    #[test]
    fn test_non_physical_inputs_are_rejected_at_construction() {
        // Component types are valid by construction; the payload cannot
        // receive invalid values.
        for power in [0.0, -10.0, f64::NAN, f64::INFINITY] {
            assert!(AmplifierTransmitter::new(ka_band(), Power::watts(power), 0.0.db()).is_err());
        }
        assert!(NoiseTempReceiver::new(ka_band(), Temperature::kelvin(-10.0)).is_err());
        let antenna = AntennaId::default();
        let transmitter = TransmitterId::default();
        assert!(TxPort::new("feed", antenna, transmitter, (-1.0).db(), None).is_err());
        assert!(
            RxPort::new(
                "feed",
                antenna,
                ReceiverId::default(),
                0.0.db(),
                Temperature::kelvin(-150.0),
                None
            )
            .is_err()
        );
    }

    #[test]
    fn test_non_finite_lumped_figures_are_rejected() {
        assert!(EirpModel::new("bad", ka_band(), Decibel::new(f64::NAN)).is_err());
        assert!(GtModel::new("bad", ka_band(), Decibel::new(f64::INFINITY)).is_err());
        // Negative figures are physically meaningful and accepted.
        assert!(GtModel::new("small terminal", ka_band(), Decibel::new(-5.0)).is_ok());
    }

    #[test]
    fn test_display_covers_all_terminal_shapes() {
        let mut payload = CommsPayload::new();
        let antenna = payload.add_antenna(
            "dish",
            Antenna::Constant(ConstantAntenna::new(46.0.db()).unwrap()),
        );
        let transmitter = payload.add_transmitter(
            "pa",
            AmplifierTransmitter::new(ka_band(), Power::watts(10.0), 0.0.db()).unwrap(),
        );
        let receiver = payload.add_receiver(
            "lnb",
            Receiver::NoiseTemperature(
                NoiseTempReceiver::new(ka_band(), Temperature::kelvin(500.0)).unwrap(),
            ),
        );
        let tx_port = payload
            .add_tx_port(
                TxPort::builder("tx leg", antenna, transmitter)
                    .feed_loss(1.0.db())
                    .band(ka_band())
                    .build()
                    .unwrap(),
            )
            .unwrap();
        let rx_port = payload
            .add_rx_port(
                RxPort::builder("rx leg", antenna, receiver, Temperature::kelvin(60.0))
                    .build()
                    .unwrap(),
            )
            .unwrap();
        let eirp = payload.add_eirp_model(EirpModel::new("eirp", ka_band(), 55.0.db()).unwrap());
        let gt = payload.add_gt_model(GtModel::new("gt", ka_band(), 3.01.db()).unwrap());

        payload
            .add_tx_terminal("tx only", TxChain::Component(tx_port))
            .unwrap();
        payload
            .add_rx_terminal("rx only", RxChain::Component(rx_port))
            .unwrap();
        payload
            .add_tx_terminal("lumped tx", TxChain::Lumped(eirp))
            .unwrap();
        payload
            .add_rx_terminal("lumped rx", RxChain::Lumped(gt))
            .unwrap();
        payload
            .add_transceiver_terminal(
                "both",
                TxChain::Component(tx_port),
                RxChain::Component(rx_port),
            )
            .unwrap();

        let rendered = payload.to_string();
        for needle in [
            "tx only",
            "rx only",
            "lumped tx",
            "lumped rx",
            "both",
            "dish",
            "pa",
            "lnb",
            "55 dBW",
            "3.01 dB/K",
            "T_ant 60 K",
            "(transceiver)",
            "(tx)",
            "(rx)",
        ] {
            assert!(
                rendered.contains(needle),
                "missing {needle:?} in:\n{rendered}"
            );
        }
    }

    #[test]
    fn test_accessors_and_error_displays() {
        let mut payload = CommsPayload::new();
        let antenna = payload.add_antenna(
            "dish",
            Antenna::Constant(ConstantAntenna::new(46.0.db()).unwrap()),
        );
        assert_eq!(payload.find_antenna("dish"), Some(antenna));
        assert_eq!(payload.find_antenna("nope"), None);
        assert_eq!(payload.antenna(antenna).unwrap().name, "dish");

        let eirp_model = EirpModel::new("eirp", ka_band(), 55.0.db()).unwrap();
        assert_eq!(eirp_model.name(), "eirp");
        assert!(eirp_model.band().contains(29.0.ghz()));
        assert_approx_eq!(eirp_model.eirp().as_f64(), 55.0, atol <= 1e-15);
        let gt_model = GtModel::new("gt", ka_band(), 3.01.db()).unwrap();
        assert_eq!(gt_model.name(), "gt");
        assert!(gt_model.band().contains(29.0.ghz()));
        assert_approx_eq!(gt_model.gt().as_f64(), 3.01, atol <= 1e-15);

        let transmitter = payload.add_transmitter(
            "pa",
            AmplifierTransmitter::new(ka_band(), Power::watts(10.0), 0.0.db()).unwrap(),
        );
        let port = TxPort::builder("leg", antenna, transmitter)
            .band(ka_band())
            .build()
            .unwrap();
        assert_eq!(port.name(), "leg");
        assert_eq!(port.antenna(), antenna);
        assert_eq!(port.transmitter(), transmitter);
        assert!(port.band().unwrap().contains(29.0.ghz()));

        for (error, needle) in [
            (
                PayloadError::UnknownAntenna(AntennaId::default()),
                "unknown antenna",
            ),
            (
                PayloadError::UnknownTransmitter(TransmitterId::default()),
                "unknown transmitter",
            ),
            (
                PayloadError::UnknownReceiver(ReceiverId::default()),
                "unknown receiver",
            ),
            (
                PayloadError::UnknownEirpModel(EirpModelId::default()),
                "EIRP model",
            ),
            (
                PayloadError::UnknownGtModel(GtModelId::default()),
                "G/T model",
            ),
            (PayloadError::UnknownTxPort(TxPortId::default()), "TX port"),
            (PayloadError::UnknownRxPort(RxPortId::default()), "RX port"),
        ] {
            assert!(error.to_string().contains(needle));
        }
    }

    #[test]
    fn test_single_terminal_builder_defaults() {
        let (payload, terminal) = CommsPayload::transmitter_only(
            "tx",
            Antenna::Constant(ConstantAntenna::new(46.0.db()).unwrap()),
            AmplifierTransmitter::new(ka_band(), Power::watts(10.0), 0.0.db()).unwrap(),
        )
        .build()
        .unwrap();
        assert!(payload.terminal(terminal).is_some());

        let (payload, terminal) = CommsPayload::receiver_only(
            "rx",
            Antenna::Constant(ConstantAntenna::new(30.0.db()).unwrap()),
            NoiseTempReceiver::new(ka_band(), Temperature::kelvin(500.0)).unwrap(),
            Temperature::kelvin(60.0),
        )
        .feed_loss(0.3.db())
        .band(ka_band())
        .build()
        .unwrap();
        assert!(payload.terminal(terminal).is_some());

        // Validation propagates from the port construction.
        assert!(
            CommsPayload::transmitter_only(
                "tx",
                Antenna::Constant(ConstantAntenna::new(46.0.db()).unwrap()),
                AmplifierTransmitter::new(ka_band(), Power::watts(10.0), 0.0.db()).unwrap(),
            )
            .feed_loss((-1.0).db())
            .build()
            .is_err()
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_payload_serde_rejects_invalid_inventory() {
        // Tampered persisted payloads must fail deserialization with the
        // same invariants as the construction API.
        let (payload, _) = diplexer_payload();
        let json = serde_json::to_string(&payload).unwrap();

        // Negative transmit power
        let bad = json.replace("\"power\":10.0", "\"power\":-10.0");
        assert_ne!(bad, json);
        let err = serde_json::from_str::<CommsPayload>(&bad).unwrap_err();
        assert!(err.to_string().contains("transmit power"));

        // Negative feed loss on the RX port (0.5 dB in the fixture)
        let bad = json.replace("\"feed_loss\":0.5", "\"feed_loss\":-0.5");
        assert_ne!(bad, json);
        let err = serde_json::from_str::<CommsPayload>(&bad).unwrap_err();
        assert!(err.to_string().contains("feed loss"));

        // Negative antenna noise temperature
        let bad = json.replace(
            "\"antenna_noise_temperature\":150.0",
            "\"antenna_noise_temperature\":-150.0",
        );
        assert_ne!(bad, json);
        let err = serde_json::from_str::<CommsPayload>(&bad).unwrap_err();
        assert!(err.to_string().contains("antenna noise temperature"));

        // The untampered payload still round-trips.
        assert!(serde_json::from_str::<CommsPayload>(&json).is_ok());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_payload_serde_rejects_dangling_wiring() {
        // Corrupt a wiring key so the TX port references a missing
        // transmitter slot: the wiring re-validation must reject it.
        let (payload, _) = diplexer_payload();
        let json = serde_json::to_string(&payload).unwrap();
        let bad = json.replace(
            "\"transmitter\":{\"idx\":1,\"version\":1}",
            "\"transmitter\":{\"idx\":9,\"version\":1}",
        );
        assert_ne!(bad, json);
        let err = serde_json::from_str::<CommsPayload>(&bad).unwrap_err();
        assert!(err.to_string().contains("unknown transmitter"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_payload_serde_round_trip_preserves_wiring() {
        let (payload, terminal) = diplexer_payload();
        let json = serde_json::to_string(&payload).unwrap();
        let round_trip: CommsPayload = serde_json::from_str(&json).unwrap();

        // The deserialized payload has a fresh identity: IDs minted before
        // serialization are rejected; look the terminal up by name instead.
        assert!(round_trip.terminal(terminal).is_none());
        let restored = round_trip
            .terminal(round_trip.find_terminal("ka transceiver").unwrap())
            .unwrap();
        assert_eq!(restored.name, "ka transceiver");
        let TerminalRole::Transceiver { tx, .. } = restored.role else {
            panic!("expected transceiver terminal");
        };
        let TxChain::Component(tx_port) = tx else {
            panic!("expected component TX chain");
        };
        assert_eq!(round_trip.tx_port(tx_port).unwrap().name, "diplexer tx leg");
    }

    #[test]
    fn test_foreign_ids_are_rejected() {
        // Two payloads built with the same insertion order mint colliding
        // slotmap keys; the payload identity tag must keep them apart.
        let (payload_a, terminal_a) =
            CommsPayload::eirp_only(EirpModel::new("a", ka_band(), 55.0.db()).unwrap());
        let (payload_b, terminal_b) =
            CommsPayload::eirp_only(EirpModel::new("b", ka_band(), 99.0.db()).unwrap());

        assert_ne!(terminal_a, terminal_b);
        assert!(payload_b.terminal(terminal_a).is_none());
        assert!(payload_a.terminal(terminal_b).is_none());

        // Foreign inventory IDs are rejected at wiring time, too.
        let mut payload = CommsPayload::new();
        let mut other = CommsPayload::new();
        let antenna = other.add_antenna(
            "dish",
            Antenna::Constant(ConstantAntenna::new(46.0.db()).unwrap()),
        );
        let transmitter = payload.add_transmitter(
            "pa",
            AmplifierTransmitter::new(ka_band(), Power::watts(10.0), 0.0.db()).unwrap(),
        );
        let err = payload
            .add_tx_port(
                TxPort::builder("feed", antenna, transmitter)
                    .build()
                    .unwrap(),
            )
            .unwrap_err();
        assert!(matches!(err, PayloadError::UnknownAntenna(_)));
    }

    #[test]
    fn test_clones_share_payload_identity() {
        let (payload, terminal) =
            CommsPayload::eirp_only(EirpModel::new("a", ka_band(), 55.0.db()).unwrap());
        let clone = payload.clone();
        assert_eq!(clone.terminal(terminal).unwrap().name, "a");
    }
}
