// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Link terminals: the transmit and receive ends of a radio link.
//!
//! A terminal is an owned value describing one end of a link. The component
//! tier composes an antenna with a radio and a feed loss ([`TxChain`],
//! [`RxChain`]); the lumped tier is an aggregate figure over a band
//! ([`EirpModel`], [`GtModel`]). [`TxTerminal`] and [`RxTerminal`] are the
//! storable either-tier types; all of them implement the link-end contracts
//! [`Eirp`] and [`GOverT`] consumed by
//! [`LinkStats::for_link`](crate::link_budget::LinkStats::for_link).
//!
//! Terminals are anonymous values: naming, selection, and grouping (e.g. a
//! transceiver sharing one dish, or a transmitter switchable between
//! antennas) belong to the collection holding them. Shared hardware is
//! modelled by cloning the component values — they are cheap and immutable.

use lox_core::units::{Angle, Decibel, Frequency, Temperature};

use crate::antenna::{Antenna, AntennaGain};
use crate::error::NonPhysicalError;
use crate::link_budget::{Eirp, GOverT, MEDIUM_TEMPERATURE, RxTerms, degraded_sky_temperature};
use crate::pointing::Pointing;
use crate::receiver::Receiver;
use crate::transmitter::AmplifierTransmitter;
use crate::{LinkBudgetError, ROOM_TEMPERATURE};
use lox_core::comms::FrequencyRange;

/// A transmit chain: an antenna fed by a transmitter.
///
/// The feed loss between transmitter output and antenna subtracts from EIRP.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "TxChainRepr")
)]
pub struct TxChain {
    antenna: Antenna,
    transmitter: AmplifierTransmitter,
    feed_loss: Decibel,
    band: FrequencyRange,
}

/// Serde wire format for [`TxChain`]: forces deserialization through the
/// validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct TxChainRepr {
    antenna: Antenna,
    transmitter: AmplifierTransmitter,
    feed_loss: Decibel,
    band: FrequencyRange,
}

#[cfg(feature = "serde")]
impl TryFrom<TxChainRepr> for TxChain {
    type Error = NonPhysicalError;

    fn try_from(repr: TxChainRepr) -> Result<Self, Self::Error> {
        TxChain::new(repr.antenna, repr.transmitter, repr.feed_loss, repr.band)
    }
}

impl TxChain {
    /// Creates a new transmit chain.
    ///
    /// `band` accepts anything convertible into a [`FrequencyRange`],
    /// including [`FrequencyBand`](lox_core::comms::FrequencyBand) letter
    /// codes. Rejects a non-finite or negative feed loss.
    pub fn new(
        antenna: impl Into<Antenna>,
        transmitter: AmplifierTransmitter,
        feed_loss: Decibel,
        band: impl Into<FrequencyRange>,
    ) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_non_negative("feed loss [dB]", feed_loss.as_f64())?;
        Ok(Self {
            antenna: antenna.into(),
            transmitter,
            feed_loss,
            band: band.into(),
        })
    }

    /// Returns the antenna radiating this chain.
    pub fn antenna(&self) -> &Antenna {
        &self.antenna
    }

    /// Returns the transmitter driving this chain.
    pub fn transmitter(&self) -> &AmplifierTransmitter {
        &self.transmitter
    }

    /// Returns the feed loss between transmitter output and antenna.
    pub fn feed_loss(&self) -> Decibel {
        self.feed_loss
    }

    /// Resolves a pointing into pattern angles against this chain's antenna.
    pub fn pattern_angles(&self, pointing: Pointing) -> Result<(Angle, Angle), LinkBudgetError> {
        pattern_angles(&self.antenna, pointing)
    }
}

impl Eirp for TxChain {
    fn band(&self) -> FrequencyRange {
        self.band
    }

    /// Returns the EIRP in dBW at the given carrier and pointing:
    ///
    /// EIRP = G_ant(carrier, θ, φ) + 10·log₁₀(P) − OBO − feed loss
    fn eirp_at(&self, carrier: Frequency, pointing: Pointing) -> Result<Decibel, LinkBudgetError> {
        check_carrier(carrier, self.band())?;
        let (theta, phi) = self.pattern_angles(pointing)?;
        Ok(self.antenna.gain(carrier, theta, phi)
            + Decibel::from_linear(self.transmitter.power().to_watts())
            - self.transmitter.output_back_off()
            - self.feed_loss)
    }
}

/// A receive chain: a receiver fed by an antenna.
///
/// The feed loss between antenna and receiver input is a noise contribution:
/// it is synthesized as a passive attenuator at 290 K ahead of the receiver
/// and referred back to the antenna flange.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "RxChainRepr")
)]
pub struct RxChain {
    antenna: Antenna,
    receiver: Receiver,
    feed_loss: Decibel,
    antenna_noise_temperature: Temperature,
    band: FrequencyRange,
}

/// Serde wire format for [`RxChain`]: forces deserialization through the
/// validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct RxChainRepr {
    antenna: Antenna,
    receiver: Receiver,
    feed_loss: Decibel,
    antenna_noise_temperature: Temperature,
    band: FrequencyRange,
}

#[cfg(feature = "serde")]
impl TryFrom<RxChainRepr> for RxChain {
    type Error = NonPhysicalError;

    fn try_from(repr: RxChainRepr) -> Result<Self, Self::Error> {
        RxChain::new(
            repr.antenna,
            repr.receiver,
            repr.feed_loss,
            repr.antenna_noise_temperature,
            repr.band,
        )
    }
}

impl RxChain {
    /// Creates a new receive chain.
    ///
    /// `band` accepts anything convertible into a [`FrequencyRange`],
    /// including [`FrequencyBand`](lox_core::comms::FrequencyBand) letter
    /// codes. Rejects a non-finite or negative feed loss or antenna noise
    /// temperature.
    pub fn new(
        antenna: impl Into<Antenna>,
        receiver: impl Into<Receiver>,
        feed_loss: Decibel,
        antenna_noise_temperature: Temperature,
        band: impl Into<FrequencyRange>,
    ) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_non_negative("feed loss [dB]", feed_loss.as_f64())?;
        NonPhysicalError::check_non_negative(
            "antenna noise temperature [K]",
            antenna_noise_temperature.to_kelvin(),
        )?;
        Ok(Self {
            antenna: antenna.into(),
            receiver: receiver.into(),
            feed_loss,
            antenna_noise_temperature,
            band: band.into(),
        })
    }

    /// Returns the antenna feeding this chain.
    pub fn antenna(&self) -> &Antenna {
        &self.antenna
    }

    /// Returns the receiver terminating this chain.
    pub fn receiver(&self) -> &Receiver {
        &self.receiver
    }

    /// Returns the feed loss between antenna and receiver input.
    pub fn feed_loss(&self) -> Decibel {
        self.feed_loss
    }

    /// Returns the clear-sky antenna noise temperature at the flange.
    pub fn antenna_noise_temperature(&self) -> Temperature {
        self.antenna_noise_temperature
    }

    /// Resolves a pointing into pattern angles against this chain's antenna.
    pub fn pattern_angles(&self, pointing: Pointing) -> Result<(Angle, Angle), LinkBudgetError> {
        pattern_angles(&self.antenna, pointing)
    }

    /// Returns the system noise temperature referred to the antenna flange.
    ///
    /// The feed loss is synthesized as a passive attenuator at 290 K ahead of
    /// the receiver and the Friis formula referred back to the flange:
    ///
    /// T_sys = T_ant + T_feed + T_rx / G_feed
    ///
    /// where `T_rx` is the receiver's input-referred (chain) noise
    /// temperature, T_feed = 290·(L − 1), and G_feed = 1/L.
    pub fn system_noise_temperature(&self) -> Temperature {
        self.system_noise_temperature_with(self.antenna_noise_temperature)
    }

    /// Returns the system noise temperature with the antenna noise
    /// temperature degraded by absorptive atmospheric loss
    /// (see [`degraded_sky_temperature`]).
    pub fn degraded_system_noise_temperature(&self, absorptive: Decibel) -> Temperature {
        self.system_noise_temperature_with(degraded_sky_temperature(
            self.antenna_noise_temperature,
            absorptive,
            MEDIUM_TEMPERATURE,
        ))
    }

    /// Flange-referred system noise temperature for a given antenna noise
    /// temperature (see [`Self::system_noise_temperature`] for the formula).
    fn system_noise_temperature_with(&self, antenna_noise_temperature: Temperature) -> Temperature {
        let chain_temperature = match &self.receiver {
            Receiver::NoiseTemperature(rx) => rx.noise_temperature(),
            Receiver::Cascade(rx) => rx.chain_noise_temperature(),
        };
        let loss_linear = self.feed_loss.to_linear();
        let feed_temperature = ROOM_TEMPERATURE.to_kelvin() * (loss_linear - 1.0);
        Temperature::kelvin(
            antenna_noise_temperature.to_kelvin()
                + feed_temperature
                + chain_temperature.to_kelvin() * loss_linear,
        )
    }
}

impl GOverT for RxChain {
    fn band(&self) -> FrequencyRange {
        self.band
    }

    /// Returns G/T in dB/K with both terms referred to the antenna flange:
    ///
    /// G/T = G_total(carrier, θ, φ) − 10·log₁₀(T_sys)
    ///
    /// The feed loss enters through the noise referral, never as a gain
    /// reduction.
    fn gt_at(&self, carrier: Frequency, pointing: Pointing) -> Result<Decibel, LinkBudgetError> {
        let terms = self
            .rx_terms(carrier, pointing)?
            .expect("component chains expose receive terms");
        Ok(terms.gt())
    }

    /// Returns the receive system gain at the flange and the system noise
    /// temperature.
    ///
    /// With the noise input-referred to the flange the signal gain is the
    /// antenna gain (less demodulator/implementation losses for cascade
    /// chains). Errors when the carrier lies outside the chain's band or the
    /// system noise temperature is not strictly positive.
    fn rx_terms(
        &self,
        carrier: Frequency,
        pointing: Pointing,
    ) -> Result<Option<RxTerms>, LinkBudgetError> {
        let gain = self.flange_gain(carrier, pointing)?;
        Ok(Some(RxTerms::new(gain, self.system_noise_temperature())?))
    }

    /// Returns the receive terms with the antenna noise temperature degraded
    /// by absorptive atmospheric loss; the signal gain is unaffected.
    fn rx_terms_degraded(
        &self,
        carrier: Frequency,
        pointing: Pointing,
        absorptive: Decibel,
    ) -> Result<Option<RxTerms>, LinkBudgetError> {
        let gain = self.flange_gain(carrier, pointing)?;
        Ok(Some(RxTerms::new(
            gain,
            self.degraded_system_noise_temperature(absorptive),
        )?))
    }
}

impl RxChain {
    /// Flange-referred receive system gain toward a pointing.
    fn flange_gain(
        &self,
        carrier: Frequency,
        pointing: Pointing,
    ) -> Result<Decibel, LinkBudgetError> {
        check_carrier(carrier, self.band)?;
        let (theta, phi) = self.pattern_angles(pointing)?;
        let antenna_gain = self.antenna.gain(carrier, theta, phi);
        Ok(match &self.receiver {
            Receiver::NoiseTemperature(_) => antenna_gain,
            Receiver::Cascade(rx) => {
                antenna_gain - rx.demodulator_loss() - rx.implementation_loss()
            }
        })
    }
}

/// Lumped transmit terminal: an aggregate EIRP figure over a band.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "EirpModelRepr")
)]
pub struct EirpModel {
    band: FrequencyRange,
    eirp: Decibel,
}

/// Serde wire format for [`EirpModel`]: forces deserialization through the
/// validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct EirpModelRepr {
    band: FrequencyRange,
    eirp: Decibel,
}

#[cfg(feature = "serde")]
impl TryFrom<EirpModelRepr> for EirpModel {
    type Error = NonPhysicalError;

    fn try_from(repr: EirpModelRepr) -> Result<Self, Self::Error> {
        EirpModel::new(repr.band, repr.eirp)
    }
}

impl EirpModel {
    /// Creates a new lumped EIRP model.
    ///
    /// Rejects a non-finite EIRP figure (negative dBW values are valid).
    pub fn new(band: impl Into<FrequencyRange>, eirp: Decibel) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_finite("EIRP [dBW]", eirp.as_f64())?;
        Ok(Self {
            band: band.into(),
            eirp,
        })
    }

    /// Returns the effective isotropic radiated power in dBW.
    pub fn eirp(&self) -> Decibel {
        self.eirp
    }
}

impl Eirp for EirpModel {
    fn band(&self) -> FrequencyRange {
        self.band
    }

    /// Returns the stored figure; the pointing is ignored.
    fn eirp_at(&self, carrier: Frequency, _pointing: Pointing) -> Result<Decibel, LinkBudgetError> {
        check_carrier(carrier, self.band)?;
        Ok(self.eirp)
    }
}

/// Lumped receive terminal: an aggregate G/T figure over a band.
///
/// The absolute gain and noise temperature are not recoverable from a G/T
/// figure, so lumped receive terminals expose no
/// [`RxTerms`] and links using them carry no absolute carrier or noise
/// powers.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "GtModelRepr")
)]
pub struct GtModel {
    band: FrequencyRange,
    gt: Decibel,
}

/// Serde wire format for [`GtModel`]: forces deserialization through the
/// validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct GtModelRepr {
    band: FrequencyRange,
    gt: Decibel,
}

#[cfg(feature = "serde")]
impl TryFrom<GtModelRepr> for GtModel {
    type Error = NonPhysicalError;

    fn try_from(repr: GtModelRepr) -> Result<Self, Self::Error> {
        GtModel::new(repr.band, repr.gt)
    }
}

impl GtModel {
    /// Creates a new lumped G/T model.
    ///
    /// Rejects a non-finite G/T figure (negative dB/K values are valid).
    pub fn new(band: impl Into<FrequencyRange>, gt: Decibel) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_finite("G/T [dB/K]", gt.as_f64())?;
        Ok(Self {
            band: band.into(),
            gt,
        })
    }

    /// Returns the gain-to-noise-temperature ratio in dB/K.
    pub fn gt(&self) -> Decibel {
        self.gt
    }
}

impl GOverT for GtModel {
    fn band(&self) -> FrequencyRange {
        self.band
    }

    /// Returns the stored figure; the pointing is ignored.
    fn gt_at(&self, carrier: Frequency, _pointing: Pointing) -> Result<Decibel, LinkBudgetError> {
        check_carrier(carrier, self.band)?;
        Ok(self.gt)
    }
}

/// A transmit terminal: a component-tier chain or a lumped EIRP figure.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[non_exhaustive]
pub enum TxTerminal {
    /// Component tier: antenna + transmitter + feed loss.
    Component(TxChain),
    /// Lumped tier: aggregate EIRP figure.
    Lumped(EirpModel),
}

impl From<TxChain> for TxTerminal {
    fn from(chain: TxChain) -> Self {
        TxTerminal::Component(chain)
    }
}

impl From<EirpModel> for TxTerminal {
    fn from(model: EirpModel) -> Self {
        TxTerminal::Lumped(model)
    }
}

impl Eirp for TxTerminal {
    fn band(&self) -> FrequencyRange {
        match self {
            TxTerminal::Component(chain) => chain.band(),
            TxTerminal::Lumped(model) => Eirp::band(model),
        }
    }

    fn eirp_at(&self, carrier: Frequency, pointing: Pointing) -> Result<Decibel, LinkBudgetError> {
        match self {
            TxTerminal::Component(chain) => chain.eirp_at(carrier, pointing),
            TxTerminal::Lumped(model) => model.eirp_at(carrier, pointing),
        }
    }
}

/// A receive terminal: a component-tier chain or a lumped G/T figure.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[non_exhaustive]
pub enum RxTerminal {
    /// Component tier: antenna + feed loss + receiver.
    Component(RxChain),
    /// Lumped tier: aggregate G/T figure.
    Lumped(GtModel),
}

impl From<RxChain> for RxTerminal {
    fn from(chain: RxChain) -> Self {
        RxTerminal::Component(chain)
    }
}

impl From<GtModel> for RxTerminal {
    fn from(model: GtModel) -> Self {
        RxTerminal::Lumped(model)
    }
}

impl GOverT for RxTerminal {
    fn band(&self) -> FrequencyRange {
        match self {
            RxTerminal::Component(chain) => chain.band(),
            RxTerminal::Lumped(model) => GOverT::band(model),
        }
    }

    fn gt_at(&self, carrier: Frequency, pointing: Pointing) -> Result<Decibel, LinkBudgetError> {
        match self {
            RxTerminal::Component(chain) => chain.gt_at(carrier, pointing),
            RxTerminal::Lumped(model) => model.gt_at(carrier, pointing),
        }
    }

    fn rx_terms(
        &self,
        carrier: Frequency,
        pointing: Pointing,
    ) -> Result<Option<RxTerms>, LinkBudgetError> {
        match self {
            RxTerminal::Component(chain) => chain.rx_terms(carrier, pointing),
            RxTerminal::Lumped(model) => model.rx_terms(carrier, pointing),
        }
    }

    fn rx_terms_degraded(
        &self,
        carrier: Frequency,
        pointing: Pointing,
        absorptive: Decibel,
    ) -> Result<Option<RxTerms>, LinkBudgetError> {
        match self {
            RxTerminal::Component(chain) => chain.rx_terms_degraded(carrier, pointing, absorptive),
            RxTerminal::Lumped(model) => model.rx_terms_degraded(carrier, pointing, absorptive),
        }
    }
}

/// Rejects a carrier outside a terminal's frequency range.
fn check_carrier(carrier: Frequency, band: FrequencyRange) -> Result<(), LinkBudgetError> {
    if !band.contains(carrier) {
        return Err(LinkBudgetError::CarrierOutOfBand { carrier, band });
    }
    Ok(())
}

/// Resolves a pointing into pattern angles against an antenna.
fn pattern_angles(
    antenna: &Antenna,
    pointing: Pointing,
) -> Result<(Angle, Angle), LinkBudgetError> {
    match pointing {
        Pointing::Boresight => Ok((Angle::ZERO, Angle::ZERO)),
        Pointing::Angles { theta, phi } => Ok((theta, phi)),
        Pointing::Direction(direction) => Ok(antenna.pattern_angles(direction)?),
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits, Power};
    use lox_test_utils::assert_approx_eq;

    use crate::antenna::ConstantAntenna;
    use crate::receiver::{CascadeReceiver, NoiseStage, NoiseTempReceiver};

    use super::*;

    fn ka_band() -> FrequencyRange {
        FrequencyRange::new(27.0.ghz(), 31.0.ghz()).unwrap()
    }

    /// 46 dBi dish, 10 W amplifier, 1 dB feed loss → EIRP 55 dBW.
    fn tx_chain() -> TxChain {
        TxChain::new(
            ConstantAntenna::new(46.0.db()).unwrap(),
            AmplifierTransmitter::new(Power::watts(10.0), 0.0.db()).unwrap(),
            1.0.db(),
            ka_band(),
        )
        .unwrap()
    }

    /// 30 dBi antenna, 500 K noise-temp receiver, lossless feed, T_ant 0 K.
    fn rx_chain() -> RxChain {
        RxChain::new(
            ConstantAntenna::new(30.0.db()).unwrap(),
            NoiseTempReceiver::new(Temperature::kelvin(500.0)).unwrap(),
            0.0.db(),
            Temperature::kelvin(0.0),
            ka_band(),
        )
        .unwrap()
    }

    #[test]
    fn test_tx_chain_eirp_with_feed_loss() {
        // EIRP = 46 + 10·log10(10) − 0 (OBO) − 1 (feed) = 55 dBW
        let eirp = tx_chain().eirp_at(29.0.ghz(), Pointing::Boresight).unwrap();
        assert_approx_eq!(eirp.as_f64(), 55.0, atol <= 1e-10);
    }

    #[test]
    fn test_rx_chain_gt_noise_temp_receiver() {
        let rx = rx_chain();
        // G/T = 30 − 10·log10(500) = 3.0103 dB/K
        let gt = rx.gt_at(29.0.ghz(), Pointing::Boresight).unwrap();
        assert_approx_eq!(gt.as_f64(), 3.0103, atol <= 1e-3);
        assert_approx_eq!(
            rx.system_noise_temperature().to_kelvin(),
            500.0,
            atol <= 1e-12
        );
    }

    #[test]
    fn test_flange_referral_through_lossy_feed() {
        // A known-T_rx receiver behind a 2 dB feed at T_ant = 150 K:
        // L = 10^0.2 = 1.5849, T_feed = 290·(L−1) = 169.62 K,
        // T_sys = 150 + 169.62 + 500·1.5849 = 1112.07 K.
        let rx = RxChain::new(
            ConstantAntenna::new(30.0.db()).unwrap(),
            NoiseTempReceiver::new(Temperature::kelvin(500.0)).unwrap(),
            2.0.db(),
            Temperature::kelvin(150.0),
            ka_band(),
        )
        .unwrap();
        let loss = 10f64.powf(0.2);
        let expected = 150.0 + 290.0 * (loss - 1.0) + 500.0 * loss;
        assert_approx_eq!(
            rx.system_noise_temperature().to_kelvin(),
            expected,
            rtol <= 1e-12
        );
        // The feed loss enters through the noise referral, not the gain.
        let terms = rx
            .rx_terms(29.0.ghz(), Pointing::Boresight)
            .unwrap()
            .unwrap();
        assert_approx_eq!(terms.gain().as_f64(), 30.0, atol <= 1e-12);
        let gt = rx.gt_at(29.0.ghz(), Pointing::Boresight).unwrap();
        assert_approx_eq!(gt.as_f64(), 30.0 - 10.0 * expected.log10(), atol <= 1e-12);
    }

    #[test]
    fn test_degraded_system_noise_temperature() {
        // Same chain as the flange-referral test; 3 dB of absorption first
        // degrades T_ant: T_ant' = 150/L_abs + 275·(1 − 1/L_abs), then the
        // identical referral applies.
        let rx = RxChain::new(
            ConstantAntenna::new(30.0.db()).unwrap(),
            NoiseTempReceiver::new(Temperature::kelvin(500.0)).unwrap(),
            2.0.db(),
            Temperature::kelvin(150.0),
            ka_band(),
        )
        .unwrap();
        let l_abs = 10f64.powf(0.3);
        let t_ant = 150.0 / l_abs + 275.0 * (1.0 - 1.0 / l_abs);
        let l_feed = 10f64.powf(0.2);
        let expected = t_ant + 290.0 * (l_feed - 1.0) + 500.0 * l_feed;
        assert_approx_eq!(
            rx.degraded_system_noise_temperature(3.0.db()).to_kelvin(),
            expected,
            rtol <= 1e-12
        );
        // Zero absorption reproduces the clear-sky figure.
        assert_approx_eq!(
            rx.degraded_system_noise_temperature(0.0.db()).to_kelvin(),
            rx.system_noise_temperature().to_kelvin(),
            rtol <= 1e-12
        );
        // The signal gain is unaffected by the noise degradation.
        let degraded = rx
            .rx_terms_degraded(29.0.ghz(), Pointing::Boresight, 3.0.db())
            .unwrap()
            .unwrap();
        assert_approx_eq!(degraded.gain().as_f64(), 30.0, atol <= 1e-12);
        assert_approx_eq!(
            degraded.system_noise_temperature().to_kelvin(),
            expected,
            rtol <= 1e-12
        );
    }

    #[test]
    fn test_cascade_chain_losses_reduce_gain_not_gt_noise() {
        let cascade = CascadeReceiver::new(
            vec![NoiseStage::new(20.0.db(), Temperature::kelvin(50.0)).unwrap()],
            0.5.db(),
            0.25.db(),
        )
        .unwrap();
        let rx = RxChain::new(
            ConstantAntenna::new(30.0.db()).unwrap(),
            cascade,
            0.0.db(),
            Temperature::kelvin(100.0),
            ka_band(),
        )
        .unwrap();
        let terms = rx
            .rx_terms(29.0.ghz(), Pointing::Boresight)
            .unwrap()
            .unwrap();
        // Gain = 30 − 0.5 (demod) − 0.25 (impl) = 29.25 dB
        assert_approx_eq!(terms.gain().as_f64(), 29.25, atol <= 1e-12);
        // T_sys = 100 + 50 = 150 K
        assert_approx_eq!(
            terms.system_noise_temperature().to_kelvin(),
            150.0,
            atol <= 1e-12
        );
    }

    #[test]
    fn test_lumped_terminals_return_stored_figures() {
        let eirp = EirpModel::new(ka_band(), 55.0.db()).unwrap();
        assert_approx_eq!(
            eirp.eirp_at(29.0.ghz(), Pointing::Boresight)
                .unwrap()
                .as_f64(),
            55.0,
            atol <= 1e-15
        );
        let gt = GtModel::new(ka_band(), 3.0.db()).unwrap();
        assert_approx_eq!(
            gt.gt_at(29.0.ghz(), Pointing::off_boresight(Angle::degrees(2.0)))
                .unwrap()
                .as_f64(),
            3.0,
            atol <= 1e-15
        );
        // Lumped receive terminals expose no absolute terms.
        assert!(
            gt.rx_terms(29.0.ghz(), Pointing::Boresight)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn test_carrier_out_of_band_is_rejected() {
        let err = tx_chain().eirp_at(10.0.ghz(), Pointing::Boresight);
        assert!(matches!(err, Err(LinkBudgetError::CarrierOutOfBand { .. })));
        let err = rx_chain().gt_at(10.0.ghz(), Pointing::Boresight);
        assert!(matches!(err, Err(LinkBudgetError::CarrierOutOfBand { .. })));
        let err = EirpModel::new(ka_band(), 55.0.db())
            .unwrap()
            .eirp_at(10.0.ghz(), Pointing::Boresight);
        assert!(matches!(err, Err(LinkBudgetError::CarrierOutOfBand { .. })));
        let err = GtModel::new(ka_band(), 3.0.db())
            .unwrap()
            .gt_at(10.0.ghz(), Pointing::Boresight);
        assert!(matches!(err, Err(LinkBudgetError::CarrierOutOfBand { .. })));
    }

    #[test]
    fn test_zero_system_noise_temperature_is_rejected() {
        let rx = RxChain::new(
            ConstantAntenna::new(30.0.db()).unwrap(),
            CascadeReceiver::new(
                vec![NoiseStage::new(20.0.db(), Temperature::kelvin(0.0)).unwrap()],
                0.0.db(),
                0.0.db(),
            )
            .unwrap(),
            0.0.db(),
            Temperature::kelvin(0.0),
            ka_band(),
        )
        .unwrap();
        assert!(rx.rx_terms(29.0.ghz(), Pointing::Boresight).is_err());
        assert!(rx.gt_at(29.0.ghz(), Pointing::Boresight).is_err());
    }

    #[test]
    fn test_chains_reject_non_physical_inputs() {
        let antenna = ConstantAntenna::new(46.0.db()).unwrap();
        let transmitter = AmplifierTransmitter::new(Power::watts(10.0), 0.0.db()).unwrap();
        assert!(
            TxChain::new(antenna.clone(), transmitter.clone(), (-1.0).db(), ka_band()).is_err()
        );

        let receiver = NoiseTempReceiver::new(Temperature::kelvin(500.0)).unwrap();
        assert!(
            RxChain::new(
                antenna.clone(),
                receiver.clone(),
                (-1.0).db(),
                Temperature::kelvin(0.0),
                ka_band()
            )
            .is_err()
        );
        assert!(
            RxChain::new(
                antenna,
                receiver,
                0.0.db(),
                Temperature::kelvin(-1.0),
                ka_band()
            )
            .is_err()
        );
        assert!(EirpModel::new(ka_band(), Decibel::new(f64::NAN)).is_err());
        assert!(GtModel::new(ka_band(), Decibel::new(f64::INFINITY)).is_err());
    }

    #[test]
    fn test_band_accepts_ieee_letter_bands() {
        use lox_core::comms::FrequencyBand;

        let tx = TxChain::new(
            ConstantAntenna::new(46.0.db()).unwrap(),
            AmplifierTransmitter::new(Power::watts(10.0), 0.0.db()).unwrap(),
            0.0.db(),
            FrequencyBand::Ka,
        )
        .unwrap();
        // IEEE Ka band: 27-40 GHz
        assert!(tx.band().contains(29.0.ghz()));
        assert!(!tx.band().contains(12.0.ghz()));
        let gt = GtModel::new(FrequencyBand::X, 3.0.db()).unwrap();
        assert!(GOverT::band(&gt).contains(8.2.ghz()));
    }

    #[test]
    fn test_chain_accessors() {
        let tx = tx_chain();
        assert!(matches!(tx.antenna(), Antenna::Constant(_)));
        assert_approx_eq!(tx.transmitter().power().to_watts(), 10.0, atol <= 1e-15);
        assert_approx_eq!(tx.feed_loss().as_f64(), 1.0, atol <= 1e-15);

        let rx = rx_chain();
        assert!(matches!(rx.antenna(), Antenna::Constant(_)));
        assert!(matches!(rx.receiver(), Receiver::NoiseTemperature(_)));
        assert_approx_eq!(rx.feed_loss().as_f64(), 0.0, atol <= 1e-15);
        assert_approx_eq!(
            rx.antenna_noise_temperature().to_kelvin(),
            0.0,
            atol <= 1e-15
        );
    }

    #[test]
    fn test_chain_pattern_angles_from_direction() {
        use lox_core::glam::DVec3;

        use crate::antenna::AntennaFrame;
        use crate::pattern::ParabolicPattern;

        // Dish boresight along +X; a line of sight along +Z is 90° off.
        let frame = AntennaFrame::from_boresight_and_reference(DVec3::X, DVec3::Z).unwrap();
        let antenna = crate::antenna::PatternedAntenna::new(
            ParabolicPattern::new(lox_core::units::Distance::meters(0.98), 0.45).unwrap(),
        )
        .with_frame(frame);
        let tx = TxChain::new(
            antenna,
            AmplifierTransmitter::new(Power::watts(10.0), 0.0.db()).unwrap(),
            0.0.db(),
            ka_band(),
        )
        .unwrap();

        let (theta, _) = tx.pattern_angles(Pointing::Direction(DVec3::X)).unwrap();
        assert_approx_eq!(theta.to_radians(), 0.0, atol <= 1e-12);
        let (theta, _) = tx.pattern_angles(Pointing::Direction(DVec3::Z)).unwrap();
        assert_approx_eq!(theta.to_degrees(), 90.0, atol <= 1e-9);
        // EIRP drops off boresight.
        let on = tx
            .eirp_at(29.0.ghz(), Pointing::Direction(DVec3::X))
            .unwrap();
        let off = tx
            .eirp_at(29.0.ghz(), Pointing::Direction(DVec3::Z))
            .unwrap();
        assert!(off.as_f64() < on.as_f64());
        // Invalid directions are rejected.
        assert!(matches!(
            tx.eirp_at(29.0.ghz(), Pointing::Direction(DVec3::ZERO)),
            Err(LinkBudgetError::InvalidPointing(_))
        ));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_cascade_chain_and_model_serde_round_trips() {
        let rx: RxTerminal = RxChain::new(
            ConstantAntenna::new(30.0.db()).unwrap(),
            CascadeReceiver::new(
                vec![NoiseStage::new(20.0.db(), Temperature::kelvin(50.0)).unwrap()],
                0.5.db(),
                0.25.db(),
            )
            .unwrap(),
            0.5.db(),
            Temperature::kelvin(100.0),
            ka_band(),
        )
        .unwrap()
        .into();
        let json = serde_json::to_string(&rx).unwrap();
        let round_trip: RxTerminal = serde_json::from_str(&json).unwrap();
        assert_approx_eq!(
            round_trip
                .gt_at(29.0.ghz(), Pointing::Boresight)
                .unwrap()
                .as_f64(),
            rx.gt_at(29.0.ghz(), Pointing::Boresight).unwrap().as_f64(),
            atol <= 1e-12
        );

        let tx: TxTerminal = EirpModel::new(ka_band(), 55.0.db()).unwrap().into();
        let json = serde_json::to_string(&tx).unwrap();
        assert!(serde_json::from_str::<TxTerminal>(&json).is_ok());
        let bad = json.replace("\"eirp\":55.0", "\"eirp\":null");
        assert!(serde_json::from_str::<TxTerminal>(&bad).is_err());

        let gt: RxTerminal = GtModel::new(ka_band(), 3.0.db()).unwrap().into();
        let json = serde_json::to_string(&gt).unwrap();
        assert!(serde_json::from_str::<RxTerminal>(&json).is_ok());
    }

    #[test]
    fn test_terminal_enums_delegate() {
        let tx: TxTerminal = tx_chain().into();
        assert!(tx.band().contains(29.0.ghz()));
        assert_approx_eq!(
            tx.eirp_at(29.0.ghz(), Pointing::Boresight)
                .unwrap()
                .as_f64(),
            55.0,
            atol <= 1e-10
        );

        let tx: TxTerminal = EirpModel::new(ka_band(), 55.0.db()).unwrap().into();
        assert!(matches!(tx, TxTerminal::Lumped(_)));
        assert!(tx.band().contains(29.0.ghz()));

        let rx: RxTerminal = rx_chain().into();
        assert!(rx.band().contains(29.0.ghz()));
        assert!(
            rx.rx_terms(29.0.ghz(), Pointing::Boresight)
                .unwrap()
                .is_some()
        );
        // The degraded terms delegate through the enum as well.
        assert!(
            rx.rx_terms_degraded(29.0.ghz(), Pointing::Boresight, 3.0.db())
                .unwrap()
                .is_some()
        );
        // Component G/T delegates through the enum.
        assert_approx_eq!(
            rx.gt_at(29.0.ghz(), Pointing::Boresight).unwrap().as_f64(),
            3.0103,
            atol <= 1e-3
        );

        let rx: RxTerminal = GtModel::new(ka_band(), 3.0.db()).unwrap().into();
        assert!(
            rx.rx_terms(29.0.ghz(), Pointing::Boresight)
                .unwrap()
                .is_none()
        );
        assert_approx_eq!(
            rx.gt_at(29.0.ghz(), Pointing::Boresight).unwrap().as_f64(),
            3.0,
            atol <= 1e-15
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_terminal_serde_round_trip_rejects_invalid() {
        let tx: TxTerminal = tx_chain().into();
        let json = serde_json::to_string(&tx).unwrap();
        assert!(serde_json::from_str::<TxTerminal>(&json).is_ok());
        let bad = json.replace("\"feed_loss\":1.0", "\"feed_loss\":-1.0");
        assert!(serde_json::from_str::<TxTerminal>(&bad).is_err());

        let rx: RxTerminal = rx_chain().into();
        let json = serde_json::to_string(&rx).unwrap();
        assert!(serde_json::from_str::<RxTerminal>(&json).is_ok());
        let bad = json.replace(
            "\"antenna_noise_temperature\":0.0",
            "\"antenna_noise_temperature\":-1.0",
        );
        assert!(serde_json::from_str::<RxTerminal>(&bad).is_err());
    }
}
