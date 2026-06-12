// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Link budget calculations and the contracts they consume.
//!
//! [`LinkBudget::new`] computes a modulation-agnostic budget between any
//! transmit end implementing [`Eirp`] and any receive end implementing
//! [`GOverT`], under the conditions captured in [`LinkConditions`]. The
//! built-in implementors live in [`terminal`](crate::terminal); custom ends
//! only need to answer the trait questions.

use lox_core::units::{Decibel, Distance, Frequency, Power, Temperature};

use crate::channel::{Channel, LinkDirection};
use crate::error::NonPhysicalError;
use crate::modcod::ModCod;
use crate::pointing::Pointing;
use crate::utils::free_space_path_loss;
use crate::{BOLTZMANN_CONSTANT, LinkBudgetError};
use lox_core::comms::FrequencyRange;

pub use lox_core::comms::PropagationLosses;

/// Boltzmann constant in dB(W/Hz/K).
const BOLTZMANN_CONSTANT_DB: Decibel = Decibel::new(-228.599_167_173_217_67);

/// Effective mean radiating temperature of the absorbing atmosphere
/// (ITU-R P.618 §8.2).
pub const MEDIUM_TEMPERATURE: Temperature = Temperature::kelvin(275.0);

/// Returns the sky-noise temperature seen by an antenna behind an absorbing
/// atmosphere (ITU-R P.618 §8.2).
///
/// An absorptive medium with linear loss `L ≥ 1` attenuates the clear-sky
/// contribution and re-radiates thermally at the mean medium temperature
/// `t_m`:
///
/// ```text
/// T_sky = T_clear / L + t_m · (1 − 1/L)
/// ```
///
/// Only absorptive loss terms apply (rain, gaseous, cloud — see
/// [`PropagationLosses::absorptive`]); scintillation and depolarization do
/// not heat the antenna.
pub fn degraded_sky_temperature(
    t_clear: Temperature,
    absorptive: Decibel,
    t_m: Temperature,
) -> Temperature {
    let l = absorptive.to_linear();
    Temperature::kelvin(t_clear.to_kelvin() / l + t_m.to_kelvin() * (1.0 - 1.0 / l))
}

/// The transmit end of a radio link.
///
/// Answers the only transmit-side question a link budget asks: the EIRP
/// toward a pointing at a carrier frequency.
pub trait Eirp {
    /// Returns the supported frequency range; carriers outside it are
    /// rejected.
    fn band(&self) -> FrequencyRange;

    /// Returns the EIRP in dBW at the given carrier and pointing.
    ///
    /// Errors when the carrier lies outside [`Self::band`].
    fn eirp_at(&self, carrier: Frequency, pointing: Pointing) -> Result<Decibel, LinkBudgetError>;
}

/// Absolute receive-side terms, both referred to the antenna flange.
///
/// Available when the receive end is more than an aggregate figure; they
/// allow absolute carrier and noise powers (and thus interference analysis).
#[derive(Debug, Clone, Copy)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "RxTermsRepr")
)]
pub struct RxTerms {
    gain: Decibel,
    system_noise_temperature: Temperature,
}

/// Serde wire format for [`RxTerms`]: forces deserialization through the
/// validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct RxTermsRepr {
    gain: Decibel,
    system_noise_temperature: Temperature,
}

#[cfg(feature = "serde")]
impl TryFrom<RxTermsRepr> for RxTerms {
    type Error = NonPhysicalError;

    fn try_from(repr: RxTermsRepr) -> Result<Self, Self::Error> {
        RxTerms::new(repr.gain, repr.system_noise_temperature)
    }
}

impl RxTerms {
    /// Creates new receive terms.
    ///
    /// Rejects a non-finite gain and a non-finite or non-positive system
    /// noise temperature.
    pub fn new(
        gain: Decibel,
        system_noise_temperature: Temperature,
    ) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_finite("receive system gain [dB]", gain.as_f64())?;
        NonPhysicalError::check_positive(
            "system noise temperature [K]",
            system_noise_temperature.to_kelvin(),
        )?;
        Ok(Self {
            gain,
            system_noise_temperature,
        })
    }

    /// Returns the receive system gain in dB at the antenna flange.
    pub fn gain(&self) -> Decibel {
        self.gain
    }

    /// Returns the system noise temperature at the antenna flange.
    pub fn system_noise_temperature(&self) -> Temperature {
        self.system_noise_temperature
    }

    /// Returns G/T = gain − 10·log₁₀(T_sys) in dB/K.
    pub fn gt(&self) -> Decibel {
        self.gain - Decibel::from_linear(self.system_noise_temperature.to_kelvin())
    }
}

/// The receive end of a radio link.
///
/// Answers the receive-side question a link budget asks — G/T toward a
/// pointing at a carrier frequency — and optionally decomposes it into
/// absolute [`RxTerms`].
pub trait GOverT {
    /// Returns the supported frequency range; carriers outside it are
    /// rejected.
    fn band(&self) -> FrequencyRange;

    /// Returns the gain-to-noise-temperature ratio (G/T) in dB/K at the
    /// given carrier and pointing.
    ///
    /// Errors when the carrier lies outside [`Self::band`].
    fn gt_at(&self, carrier: Frequency, pointing: Pointing) -> Result<Decibel, LinkBudgetError>;

    /// Returns the absolute receive terms when this end exposes them.
    ///
    /// `None` for ends characterised only by an aggregate G/T figure — the
    /// absolute gain and noise temperature are not recoverable from the
    /// ratio. Defaults to `None`.
    fn rx_terms(
        &self,
        carrier: Frequency,
        pointing: Pointing,
    ) -> Result<Option<RxTerms>, LinkBudgetError> {
        let _ = (carrier, pointing);
        Ok(None)
    }

    /// Returns the receive terms with the antenna noise temperature degraded
    /// by the absorptive part of the atmospheric loss (ITU-R P.618 §8.2).
    ///
    /// `None` for ends that cannot recompute their noise budget — aggregate
    /// G/T figures are clear-sky values and stay undegraded. Defaults to
    /// `None`.
    fn rx_terms_degraded(
        &self,
        carrier: Frequency,
        pointing: Pointing,
        absorptive: Decibel,
    ) -> Result<Option<RxTerms>, LinkBudgetError> {
        let _ = (carrier, pointing, absorptive);
        Ok(None)
    }
}

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

/// The conditions a link budget is evaluated under: carrier, slant range,
/// environmental losses, link direction, and the pointing at each end.
///
/// Valid by construction — [`LinkConditions::builder`] validates the
/// physical inputs, so [`LinkBudget::new`] only fails on physics (a
/// carrier outside a terminal's band or an unresolvable pointing).
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "LinkConditionsRepr")
)]
pub struct LinkConditions {
    carrier: Frequency,
    range: Distance,
    losses: PropagationLosses,
    tx_pointing: Pointing,
    rx_pointing: Pointing,
    direction: Option<LinkDirection>,
}

/// Serde wire format for [`LinkConditions`]: forces deserialization through
/// the validated builder.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct LinkConditionsRepr {
    carrier: Frequency,
    range: Distance,
    #[serde(default = "PropagationLosses::none")]
    losses: PropagationLosses,
    #[serde(default)]
    tx_pointing: Pointing,
    #[serde(default)]
    rx_pointing: Pointing,
    #[serde(default)]
    direction: Option<LinkDirection>,
}

#[cfg(feature = "serde")]
impl TryFrom<LinkConditionsRepr> for LinkConditions {
    type Error = NonPhysicalError;

    fn try_from(repr: LinkConditionsRepr) -> Result<Self, Self::Error> {
        let mut builder = LinkConditions::builder(repr.carrier, repr.range)
            .losses(repr.losses)
            .tx_pointing(repr.tx_pointing)
            .rx_pointing(repr.rx_pointing);
        if let Some(direction) = repr.direction {
            builder = builder.direction(direction);
        }
        builder.build()
    }
}

impl LinkConditions {
    /// Starts building link conditions for the given carrier and slant
    /// range.
    ///
    /// Losses default to none, both pointings to boresight, and the
    /// direction to unspecified.
    pub fn builder(carrier: Frequency, range: Distance) -> LinkConditionsBuilder {
        LinkConditionsBuilder {
            carrier,
            range,
            losses: PropagationLosses::none(),
            tx_pointing: Pointing::Boresight,
            rx_pointing: Pointing::Boresight,
            direction: None,
        }
    }

    /// Returns the carrier frequency.
    pub fn carrier(&self) -> Frequency {
        self.carrier
    }

    /// Returns the slant range between TX and RX.
    pub fn range(&self) -> Distance {
        self.range
    }

    /// Returns the environmental losses.
    pub fn losses(&self) -> &PropagationLosses {
        &self.losses
    }

    /// Returns the pointing at the transmit end.
    pub fn tx_pointing(&self) -> Pointing {
        self.tx_pointing
    }

    /// Returns the pointing at the receive end.
    pub fn rx_pointing(&self) -> Pointing {
        self.rx_pointing
    }

    /// Returns the link direction, when specified.
    pub fn direction(&self) -> Option<LinkDirection> {
        self.direction
    }
}

/// Builder for [`LinkConditions`].
///
/// Created via [`LinkConditions::builder`]. Inputs are validated at
/// [`LinkConditionsBuilder::build`].
#[derive(Debug, Clone)]
pub struct LinkConditionsBuilder {
    carrier: Frequency,
    range: Distance,
    losses: PropagationLosses,
    tx_pointing: Pointing,
    rx_pointing: Pointing,
    direction: Option<LinkDirection>,
}

impl LinkConditionsBuilder {
    /// Sets the environmental losses.
    pub fn losses(mut self, losses: PropagationLosses) -> Self {
        self.losses = losses;
        self
    }

    /// Sets the pointing at the transmit end.
    pub fn tx_pointing(mut self, pointing: Pointing) -> Self {
        self.tx_pointing = pointing;
        self
    }

    /// Sets the pointing at the receive end.
    pub fn rx_pointing(mut self, pointing: Pointing) -> Self {
        self.rx_pointing = pointing;
        self
    }

    /// Sets the link direction.
    ///
    /// On downlinks the receive antenna sits behind the absorbing
    /// atmosphere, so the budget recomputes the system noise temperature
    /// with the rain-degraded sky temperature (ITU-R P.618 §8.2). Uplink
    /// and crosslink receivers are unaffected, as is an unspecified
    /// direction.
    pub fn direction(mut self, direction: LinkDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Builds the link conditions, validating all physical inputs.
    ///
    /// Rejects a non-finite or non-positive carrier frequency or slant
    /// range.
    pub fn build(self) -> Result<LinkConditions, NonPhysicalError> {
        for (quantity, value) in [
            ("carrier frequency [Hz]", self.carrier.to_hertz()),
            ("slant range [m]", self.range.to_meters()),
        ] {
            NonPhysicalError::check_positive(quantity, value)?;
        }
        Ok(LinkConditions {
            carrier: self.carrier,
            range: self.range,
            losses: self.losses,
            tx_pointing: self.tx_pointing,
            rx_pointing: self.rx_pointing,
            direction: self.direction,
        })
    }
}

/// Modulation-agnostic link budget output.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LinkBudget {
    /// Slant range between TX and RX.
    pub slant_range: Distance,
    /// Carrier frequency.
    pub frequency: Frequency,
    /// Free-space path loss.
    pub fspl: Decibel,
    /// EIRP of the transmitter.
    pub eirp: Decibel,
    /// Receiver G/T under clear-sky conditions.
    pub gt: Decibel,
    /// Receiver G/T with the antenna noise temperature degraded by
    /// absorptive atmospheric loss. `Some` only on downlinks with a receive
    /// end that exposes degradable terms; this is the value the budget uses.
    pub gt_degraded: Option<Decibel>,
    /// Link direction, when specified.
    pub direction: Option<LinkDirection>,
    /// Environmental losses.
    pub losses: PropagationLosses,
    /// Received carrier power. `None` for receive ends without [`RxTerms`].
    pub carrier_rx_power: Option<Decibel>,
    /// Effective receive terms the budget was computed with (rain-degraded
    /// on downlinks). `None` for aggregate-figure ends.
    pub rx_terms: Option<RxTerms>,
    /// Carrier-to-noise density ratio (C/N₀).
    pub c_n0: Decibel,
}

impl LinkBudget {
    /// Computes a modulation-agnostic link budget between two link ends
    /// under the given conditions.
    ///
    /// The carrier must lie inside both ends' frequency ranges. Each end
    /// resolves its own pointing against its antenna. On downlinks the
    /// budget uses the rain-degraded G/T when the receive end exposes
    /// degradable terms (see [`GOverT::rx_terms_degraded`]); aggregate G/T
    /// figures stay at their clear-sky value.
    ///
    /// All outputs are bandwidth-free; carrier-to-noise ratio and noise
    /// power are derived views ([`Self::c_n`], [`Self::noise_power`]), and
    /// [`Self::modulate`] takes everything bandwidth-dependent from the
    /// channel.
    pub fn new(
        tx: &impl Eirp,
        rx: &impl GOverT,
        conditions: &LinkConditions,
    ) -> Result<Self, LinkBudgetError> {
        let params = conditions;
        let carrier = params.carrier;

        let eirp = tx.eirp_at(carrier, params.tx_pointing)?;
        let rx_terms = rx.rx_terms(carrier, params.rx_pointing)?;
        let gt = match &rx_terms {
            Some(terms) => terms.gt(),
            None => rx.gt_at(carrier, params.rx_pointing)?,
        };

        // Rain raises the receive noise temperature only on downlinks,
        // where the antenna looks up through the absorbing atmosphere and
        // picks up its thermal re-radiation. An uplink receiver is a
        // spacecraft antenna pointed at Earth: its configured T_ant already
        // reflects the ~290 K warm-Earth background, which dwarfs any extra
        // atmospheric emission. A crosslink has no atmosphere in its path.
        let degraded_terms = if params.direction == Some(LinkDirection::Downlink) {
            rx.rx_terms_degraded(carrier, params.rx_pointing, params.losses.absorptive())?
        } else {
            None
        };
        let gt_degraded = degraded_terms.as_ref().map(RxTerms::gt);
        let effective_terms = degraded_terms.or(rx_terms);
        let effective_gt = gt_degraded.unwrap_or(gt);

        let fspl = free_space_path_loss(params.range, carrier);
        let env_loss = params.losses.total();

        let c_n0 = eirp + effective_gt - fspl - env_loss - BOLTZMANN_CONSTANT_DB;

        let carrier_rx_power = effective_terms
            .as_ref()
            .map(|terms| eirp - fspl - env_loss + terms.gain());

        Ok(Self {
            slant_range: params.range,
            frequency: carrier,
            fspl,
            eirp,
            gt,
            gt_degraded,
            direction: params.direction,
            losses: params.losses.clone(),
            carrier_rx_power,
            rx_terms: effective_terms,
            c_n0,
        })
    }

    /// Returns the carrier-to-noise ratio in the given noise bandwidth:
    /// C/N = C/N₀ − 10·log₁₀(BW).
    ///
    /// Rejects a non-finite or non-positive bandwidth.
    pub fn c_n(&self, bandwidth: Frequency) -> Result<Decibel, LinkBudgetError> {
        NonPhysicalError::check_positive("noise bandwidth [Hz]", bandwidth.to_hertz())?;
        Ok(self.c_n0 - Decibel::from_linear(bandwidth.to_hertz()))
    }

    /// Returns the noise power in the given bandwidth, when the receive end
    /// exposes absolute terms: N = k·T_sys·BW.
    ///
    /// `Ok(None)` for aggregate-figure ends; rejects a non-finite or
    /// non-positive bandwidth.
    pub fn noise_power(&self, bandwidth: Frequency) -> Result<Option<Decibel>, LinkBudgetError> {
        NonPhysicalError::check_positive("noise bandwidth [Hz]", bandwidth.to_hertz())?;
        Ok(self.rx_terms.as_ref().map(|terms| {
            Decibel::from_linear(
                terms.system_noise_temperature().to_kelvin()
                    * BOLTZMANN_CONSTANT
                    * bandwidth.to_hertz(),
            )
        }))
    }

    /// Selects and applies the best MODCOD from a table: the
    /// highest-efficiency mode that closes on this channel with the given
    /// design margin (adaptive coding and modulation).
    ///
    /// Selection and evaluation use the same channel and margin, so the
    /// returned result always [`closes`](ModulatedLinkBudget::closes);
    /// `None` means no mode in the table closes.
    pub fn modulate_best(
        &self,
        channel: &Channel,
        table: &[ModCod],
        design_margin: Decibel,
    ) -> Option<ModulatedLinkBudget> {
        let es_n0 = channel.es_n0(self.c_n0);
        let modcod = ModCod::select(es_n0, design_margin, table)?;
        Some(self.modulate(channel, modcod, design_margin))
    }

    /// Applies a waveform and a modulation and coding scheme to this budget.
    ///
    /// Everything bandwidth-dependent derives from the channel: Es/N0 from
    /// its symbol rate, C/N from its occupied bandwidth, Eb/N0 through the
    /// MODCOD's exact information bits per symbol, and the link margin
    /// against the MODCOD's threshold plus the design margin.
    pub fn modulate(
        &self,
        channel: &Channel,
        modcod: &ModCod,
        design_margin: Decibel,
    ) -> ModulatedLinkBudget {
        let es_n0 = channel.es_n0(self.c_n0);
        let c_n = channel.c_n(self.c_n0);
        let eb_n0 = es_n0 - Decibel::from_linear(modcod.mode().info_bits_per_symbol());
        let margin = eb_n0 - modcod.required_eb_n0() - design_margin;
        ModulatedLinkBudget {
            budget: self.clone(),
            channel: channel.clone(),
            modcod: modcod.clone(),
            design_margin,
            es_n0,
            eb_n0,
            c_n,
            margin,
        }
    }
}

/// Link-budget output with a modulation and coding scheme applied.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModulatedLinkBudget {
    /// The modulation-agnostic link budget.
    pub budget: LinkBudget,
    /// The waveform the link is evaluated on.
    pub channel: Channel,
    /// The modulation and coding scheme the link is evaluated against.
    pub modcod: ModCod,
    /// Design margin applied on top of the MODCOD threshold.
    pub design_margin: Decibel,
    /// Es/N0 (energy per symbol to noise spectral density).
    pub es_n0: Decibel,
    /// Eb/N0 (energy per information bit to noise spectral density).
    pub eb_n0: Decibel,
    /// Carrier-to-noise ratio in the channel's occupied bandwidth.
    pub c_n: Decibel,
    /// Link margin: Eb/N0 − required Eb/N0 − design margin.
    pub margin: Decibel,
}

impl ModulatedLinkBudget {
    /// Returns whether the link closes: margin ≥ 0.
    pub fn closes(&self) -> bool {
        self.margin.as_f64() >= 0.0
    }

    /// Returns the symbol rate of the underlying channel.
    pub fn symbol_rate(&self) -> Frequency {
        self.channel.symbol_rate()
    }

    /// Returns the information bit rate: symbol rate × info bits per symbol.
    pub fn information_rate(&self) -> Frequency {
        self.modcod
            .mode()
            .information_rate(self.channel.symbol_rate())
    }

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
        let bandwidth = self.channel.bandwidth();
        let noise_linear = self
            .budget
            .noise_power(bandwidth)?
            .ok_or(LinkBudgetError::AbsolutePowerUnavailable)?
            .to_linear();
        let carrier = self
            .budget
            .carrier_rx_power
            .ok_or(LinkBudgetError::AbsolutePowerUnavailable)?;

        let total_ni = noise_linear + interference_power.to_watts();
        let c_n0i0 =
            carrier - Decibel::from_linear(total_ni) + Decibel::from_linear(bandwidth.to_hertz());
        let c_n0_to_eb_n0 = self.eb_n0 - self.budget.c_n0;
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

    use crate::antenna::ConstantAntenna;
    use crate::channel::{LinkDirection, Modulation};
    use crate::modcod::ErrorMetric;
    use crate::receiver::NoiseTempReceiver;
    use crate::terminal::{EirpModel, GtModel, RxChain, TxChain};
    use crate::transmitter::AmplifierTransmitter;

    use super::*;

    fn ka_band() -> FrequencyRange {
        FrequencyRange::new(27.0.ghz(), 31.0.ghz()).unwrap()
    }

    /// 46 dBi antenna, 10 W, 1 dB feed loss → EIRP = 55 dBW.
    fn tx_chain() -> TxChain {
        TxChain::new(
            ConstantAntenna::new(46.0.db()).unwrap(),
            AmplifierTransmitter::new(Power::watts(10.0), 0.0.db()).unwrap(),
            1.0.db(),
            ka_band(),
        )
        .unwrap()
    }

    /// 30 dBi antenna, T_sys = 500 K → G/T = 3.01 dB/K.
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

    fn channel() -> Channel {
        Channel::builder(5.0.mhz()).build().unwrap()
    }

    /// QPSK rate 1/2 requiring Eb/N0 = 10 dB at BER 1e-6.
    fn modcod() -> ModCod {
        ModCod::from_required_eb_n0(
            "test downlink",
            Modulation::Qpsk,
            0.5,
            10.0.db(),
            ErrorMetric::Ber,
            1e-6,
        )
        .unwrap()
    }

    fn link_conditions() -> LinkConditions {
        LinkConditions::builder(29.0.ghz(), Distance::kilometers(1000.0))
            .build()
            .unwrap()
    }

    fn component_stats() -> LinkBudget {
        LinkBudget::new(&tx_chain(), &rx_chain(), &link_conditions()).unwrap()
    }

    fn lumped_stats() -> LinkBudget {
        LinkBudget::new(
            &EirpModel::new(ka_band(), 55.0.db()).unwrap(),
            &GtModel::new(ka_band(), 3.01.db()).unwrap(),
            &link_conditions(),
        )
        .unwrap()
    }

    /// Custom link ends only need the trait surface: a fixed-EIRP transmitter
    /// and a fixed-G/T receiver relying on the default `rx_terms`.
    struct FixedTx;

    impl Eirp for FixedTx {
        fn band(&self) -> FrequencyRange {
            ka_band()
        }

        fn eirp_at(&self, _: Frequency, _: Pointing) -> Result<Decibel, LinkBudgetError> {
            Ok(55.0.db())
        }
    }

    struct FixedRx;

    impl GOverT for FixedRx {
        fn band(&self) -> FrequencyRange {
            ka_band()
        }

        fn gt_at(&self, _: Frequency, _: Pointing) -> Result<Decibel, LinkBudgetError> {
            Ok(3.01.db())
        }
    }

    #[test]
    fn test_for_link_accepts_custom_implementors() {
        // The traits are dyn-compatible.
        let tx: &dyn Eirp = &FixedTx;
        let rx: &dyn GOverT = &FixedRx;
        assert!(tx.band().contains(29.0.ghz()));
        assert!(rx.band().contains(29.0.ghz()));

        let stats = LinkBudget::new(&FixedTx, &FixedRx, &link_conditions()).unwrap();
        assert_approx_eq!(stats.c_n0.as_f64(), 104.913, atol <= 0.01);
        // The default rx_terms is None: no absolute powers.
        assert!(stats.carrier_rx_power.is_none());
        assert!(stats.rx_terms.is_none());
        assert!(stats.noise_power(5.0.mhz()).unwrap().is_none());
    }

    #[test]
    fn test_rx_terms_are_valid_by_construction() {
        let terms = RxTerms::new(30.0.db(), Temperature::kelvin(500.0)).unwrap();
        assert_approx_eq!(
            terms.gt().as_f64(),
            30.0 - 10.0 * 500.0_f64.log10(),
            atol <= 1e-12
        );
        // Custom GOverT implementors cannot smuggle non-physical terms past
        // the budget: zero, negative, and non-finite values are rejected.
        assert!(RxTerms::new(30.0.db(), Temperature::kelvin(0.0)).is_err());
        assert!(RxTerms::new(30.0.db(), Temperature::kelvin(-10.0)).is_err());
        assert!(RxTerms::new(30.0.db(), Temperature::kelvin(f64::NAN)).is_err());
        assert!(RxTerms::new(Decibel::new(f64::INFINITY), Temperature::kelvin(500.0)).is_err());
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
        assert!(stats.rx_terms.is_some());
        assert!(stats.noise_power(5.0.mhz()).unwrap().is_some());
    }

    #[test]
    fn test_for_link_c_n0_consistency() {
        // C/N0 must equal P_rx − P_noise + 10·log10(BW).
        let stats = component_stats();
        let bandwidth = 5.0.mhz();
        let c_n0_from_power = stats.carrier_rx_power.unwrap()
            - stats.noise_power(bandwidth).unwrap().unwrap()
            + Decibel::from_linear(bandwidth.to_hertz());
        assert_approx_eq!(stats.c_n0.as_f64(), c_n0_from_power.as_f64(), atol <= 1e-10);
    }

    #[test]
    fn test_for_link_lumped_budget() {
        let stats = lumped_stats();
        assert_approx_eq!(stats.c_n0.as_f64(), 104.913, atol <= 0.01);
        assert!(stats.carrier_rx_power.is_none());
        assert!(stats.rx_terms.is_none());
        assert!(stats.noise_power(5.0.mhz()).unwrap().is_none());
    }

    #[test]
    fn test_for_link_rejects_carrier_out_of_band() {
        // 29 GHz fits the TX band but not the RX band.
        let err = LinkBudget::new(
            &EirpModel::new(ka_band(), 55.0.db()).unwrap(),
            &GtModel::new(
                FrequencyRange::new(17.0.ghz(), 21.0.ghz()).unwrap(),
                3.01.db(),
            )
            .unwrap(),
            &link_conditions(),
        )
        .unwrap_err();

        assert!(matches!(err, LinkBudgetError::CarrierOutOfBand { .. }));
        assert!(err.to_string().contains("17.000–21.000 GHz"));
    }

    #[test]
    fn test_for_link_rejects_carrier_out_of_band_on_tx_side() {
        let err = LinkBudget::new(
            &EirpModel::new(
                FrequencyRange::new(17.0.ghz(), 21.0.ghz()).unwrap(),
                55.0.db(),
            )
            .unwrap(),
            &GtModel::new(ka_band(), 3.01.db()).unwrap(),
            &link_conditions(),
        )
        .unwrap_err();
        assert!(matches!(err, LinkBudgetError::CarrierOutOfBand { .. }));
    }

    #[test]
    fn test_link_conditions_reject_non_physical_inputs() {
        for (carrier, range) in [
            (Frequency::hertz(0.0), Distance::kilometers(1000.0)),
            (Frequency::hertz(f64::NAN), Distance::kilometers(1000.0)),
            (29.0.ghz(), Distance::kilometers(0.0)),
            (29.0.ghz(), Distance::kilometers(-1.0)),
        ] {
            assert!(LinkConditions::builder(carrier, range).build().is_err());
        }
    }

    #[test]
    fn test_link_parameters_defaults() {
        let params = link_conditions();
        assert_approx_eq!(params.losses().total().as_f64(), 0.0, atol <= 1e-15);
        assert_eq!(params.tx_pointing(), Pointing::Boresight);
        assert_eq!(params.rx_pointing(), Pointing::Boresight);
        assert_approx_eq!(params.carrier().to_gigahertz(), 29.0, rtol <= 1e-12);
        assert_approx_eq!(params.range().to_meters(), 1e6, rtol <= 1e-12);
    }

    fn p618_losses() -> PropagationLosses {
        // absorptive = 2.7 dB (rain + gaseous + cloud), total = 3.0 dB
        PropagationLosses::builder()
            .rain(2.0.db())
            .gaseous(0.5.db())
            .cloud(0.2.db())
            .scintillation(0.3.db())
            .build()
            .unwrap()
    }

    fn faded_params(direction: Option<LinkDirection>) -> LinkConditions {
        let mut builder =
            LinkConditions::builder(29.0.ghz(), Distance::kilometers(1000.0)).losses(p618_losses());
        if let Some(direction) = direction {
            builder = builder.direction(direction);
        }
        builder.build().unwrap()
    }

    #[test]
    fn test_degraded_sky_temperature() {
        // No absorption: the clear-sky temperature passes through.
        let t = degraded_sky_temperature(Temperature::kelvin(50.0), 0.0.db(), MEDIUM_TEMPERATURE);
        assert_approx_eq!(t.to_kelvin(), 50.0, rtol <= 1e-12);
        // Opaque atmosphere: the antenna sees the medium itself.
        let t = degraded_sky_temperature(Temperature::kelvin(50.0), 60.0.db(), MEDIUM_TEMPERATURE);
        assert_approx_eq!(t.to_kelvin(), 275.0, rtol <= 1e-5);
        // 3 dB of absorption: T = 50/L + 275·(1 − 1/L), L = 10^0.3.
        let t = degraded_sky_temperature(Temperature::kelvin(50.0), 3.0.db(), MEDIUM_TEMPERATURE);
        let l = 10.0_f64.powf(0.3);
        assert_approx_eq!(
            t.to_kelvin(),
            50.0 / l + 275.0 * (1.0 - 1.0 / l),
            rtol <= 1e-12
        );
    }

    #[test]
    fn test_for_link_downlink_degrades_gt() {
        let clear = LinkBudget::new(&tx_chain(), &rx_chain(), &faded_params(None)).unwrap();
        let faded = LinkBudget::new(
            &tx_chain(),
            &rx_chain(),
            &faded_params(Some(LinkDirection::Downlink)),
        )
        .unwrap();

        // T_sys: 500 K clear → 500 + 275·(1 − 1/L_abs) K degraded (T_ant = 0 K).
        let l_abs = 10.0_f64.powf(0.27);
        let t_sys_degraded = 500.0 + 275.0 * (1.0 - 1.0 / l_abs);
        let gt_degraded_exp = 30.0 - 10.0 * t_sys_degraded.log10();

        assert_approx_eq!(faded.gt.as_f64(), clear.gt.as_f64(), atol <= 1e-12);
        let gt_degraded = faded.gt_degraded.unwrap();
        assert_approx_eq!(gt_degraded.as_f64(), gt_degraded_exp, atol <= 1e-10);
        assert!(gt_degraded.as_f64() < faded.gt.as_f64());

        // The budget uses the degraded G/T.
        assert_approx_eq!(
            clear.c_n0.as_f64() - faded.c_n0.as_f64(),
            clear.gt.as_f64() - gt_degraded.as_f64(),
            atol <= 1e-10
        );

        // C/N0 consistency holds with the degraded noise power.
        let bandwidth = 5.0.mhz();
        let c_n0_from_power = faded.carrier_rx_power.unwrap()
            - faded.noise_power(bandwidth).unwrap().unwrap()
            + Decibel::from_linear(bandwidth.to_hertz());
        assert_approx_eq!(faded.c_n0.as_f64(), c_n0_from_power.as_f64(), atol <= 1e-10);
    }

    #[test]
    fn test_for_link_uplink_and_crosslink_stay_clear_sky() {
        let clear = LinkBudget::new(&tx_chain(), &rx_chain(), &faded_params(None)).unwrap();
        for direction in [LinkDirection::Uplink, LinkDirection::Crosslink] {
            let stats =
                LinkBudget::new(&tx_chain(), &rx_chain(), &faded_params(Some(direction))).unwrap();
            assert!(stats.gt_degraded.is_none());
            assert_eq!(stats.direction, Some(direction));
            assert_approx_eq!(stats.c_n0.as_f64(), clear.c_n0.as_f64(), atol <= 1e-12);
        }
    }

    #[test]
    fn test_for_link_lumped_downlink_stays_clear_sky() {
        // An aggregate G/T figure cannot be degraded — documented caveat.
        let tx = EirpModel::new(ka_band(), 55.0.db()).unwrap();
        let rx = GtModel::new(ka_band(), 3.01.db()).unwrap();
        let clear = LinkBudget::new(&tx, &rx, &faded_params(None)).unwrap();
        let faded =
            LinkBudget::new(&tx, &rx, &faded_params(Some(LinkDirection::Downlink))).unwrap();
        assert!(faded.gt_degraded.is_none());
        assert_approx_eq!(faded.c_n0.as_f64(), clear.c_n0.as_f64(), atol <= 1e-12);
    }

    #[test]
    fn test_for_link_downlink_without_absorption_is_a_no_op() {
        let params = LinkConditions::builder(29.0.ghz(), Distance::kilometers(1000.0))
            .direction(LinkDirection::Downlink)
            .build()
            .unwrap();
        let stats = LinkBudget::new(&tx_chain(), &rx_chain(), &params).unwrap();
        // L_abs = 1: degraded equals clear-sky.
        assert_approx_eq!(
            stats.gt_degraded.unwrap().as_f64(),
            stats.gt.as_f64(),
            atol <= 1e-12
        );
        assert_approx_eq!(stats.c_n0.as_f64(), 104.913, atol <= 0.01);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_link_parameters_serde_round_trip_and_validation() {
        let params = LinkConditions::builder(29.0.ghz(), Distance::kilometers(1000.0))
            .losses(p618_losses())
            .tx_pointing(Pointing::off_boresight(lox_core::units::Angle::degrees(
                2.0,
            )))
            .direction(LinkDirection::Downlink)
            .build()
            .unwrap();
        let json = serde_json::to_string(&params).unwrap();
        let round_trip: LinkConditions = serde_json::from_str(&json).unwrap();
        assert_eq!(round_trip.carrier(), params.carrier());
        assert_eq!(round_trip.range(), params.range());
        assert_eq!(round_trip.losses(), params.losses());
        assert_eq!(round_trip.tx_pointing(), params.tx_pointing());
        assert_eq!(round_trip.rx_pointing(), params.rx_pointing());
        assert_eq!(round_trip.direction(), params.direction());

        // Optional fields default: no losses, boresight, no direction.
        let minimal: LinkConditions =
            serde_json::from_str(r#"{"carrier":29.0e9,"range":1.0e6}"#).unwrap();
        assert_approx_eq!(minimal.losses().total().as_f64(), 0.0, atol <= 1e-15);
        assert_eq!(minimal.tx_pointing(), Pointing::Boresight);
        assert_eq!(minimal.rx_pointing(), Pointing::Boresight);
        assert_eq!(minimal.direction(), None);

        // Non-physical inputs are rejected at deserialization time.
        for bad in [
            r#"{"carrier":0.0,"range":1.0e6}"#,
            r#"{"carrier":29.0e9,"range":0.0}"#,
        ] {
            assert!(serde_json::from_str::<LinkConditions>(bad).is_err());
        }
    }

    #[test]
    fn test_for_link_applies_propagation_losses() {
        let losses = PropagationLosses::builder()
            .rain(2.0.db())
            .gaseous(0.5.db())
            .scintillation(0.3.db())
            .cloud(0.2.db())
            .depolarization(0.1.db())
            .build()
            .unwrap();
        let clear = lumped_stats();
        let params = LinkConditions::builder(29.0.ghz(), Distance::kilometers(1000.0))
            .losses(losses)
            .build()
            .unwrap();
        let faded = LinkBudget::new(
            &EirpModel::new(ka_band(), 55.0.db()).unwrap(),
            &GtModel::new(ka_band(), 3.01.db()).unwrap(),
            &params,
        )
        .unwrap();
        assert_approx_eq!(
            clear.c_n0.as_f64() - faded.c_n0.as_f64(),
            3.1,
            atol <= 1e-10
        );
    }

    #[test]
    fn test_modulate_produces_modulated_budget() {
        let stats = component_stats();
        let m = stats.modulate(&channel(), &modcod(), 3.0.db());
        // Eb/N0 ≈ 37.91 (QPSK 1/2 → 1 info bit/symbol, C/N0 ≈ 104.91 dB·Hz)
        assert_approx_eq!(m.eb_n0.as_f64(), 37.91, atol <= 0.02);
        // required_eb_n0 = 10, design margin = 3 → margin ≈ 24.91
        assert_approx_eq!(m.margin.as_f64(), 24.91, atol <= 0.02);
        // Information rate: 5 Msps × 1 info bit/symbol.
        assert_approx_eq!(m.information_rate().to_hertz(), 5e6, rtol <= 1e-12);
        assert_approx_eq!(m.symbol_rate().to_hertz(), 5e6, rtol <= 1e-12);
        // C/N derives from the channel's occupied bandwidth.
        assert_approx_eq!(
            m.c_n.as_f64(),
            m.budget.c_n(channel().bandwidth()).unwrap().as_f64(),
            atol <= 1e-12
        );
        // The link closes comfortably.
        assert!(m.closes());
    }

    #[test]
    fn test_views_reject_non_physical_bandwidth() {
        let budget = component_stats();
        for bandwidth in [
            Frequency::hertz(0.0),
            Frequency::hertz(-5e6),
            Frequency::hertz(f64::NAN),
            Frequency::hertz(f64::INFINITY),
        ] {
            assert!(matches!(
                budget.c_n(bandwidth),
                Err(LinkBudgetError::NonPhysical { .. })
            ));
            assert!(matches!(
                budget.noise_power(bandwidth),
                Err(LinkBudgetError::NonPhysical { .. })
            ));
        }
    }

    #[test]
    fn test_modulate_best_selects_and_closes() {
        use crate::modcod::dvb_s2;

        let budget = component_stats();
        let m = budget
            .modulate_best(&channel(), dvb_s2(), 3.0.db())
            .unwrap();
        // The integrated path agrees with manual selection + modulation.
        let es_n0 = channel().es_n0(budget.c_n0);
        let expected = ModCod::select(es_n0, 3.0.db(), dvb_s2()).unwrap();
        assert_eq!(&m.modcod, expected);
        assert_approx_eq!(
            m.margin.as_f64(),
            budget
                .modulate(&channel(), expected, 3.0.db())
                .margin
                .as_f64(),
            atol <= 1e-12
        );
        // Some always closes — that is the invariant.
        assert!(m.closes());
        // C/N0 ≈ 104.9 dB·Hz at 5 Msps → Es/N0 ≈ 37.9 dB: the top mode closes.
        assert_eq!(m.modcod.mode().name(), "32APSK 9/10");

        // An impossible margin closes nothing.
        assert!(
            budget
                .modulate_best(&channel(), dvb_s2(), 100.0.db())
                .is_none()
        );
    }

    #[test]
    fn test_modulate_closes_boundary() {
        // A MODCOD whose threshold sits exactly at the achieved Eb/N0 closes;
        // one requiring more does not.
        let stats = component_stats();
        let m = stats.modulate(&channel(), &modcod(), 0.0.db());
        let exact = ModCod::from_required_eb_n0(
            "exact",
            Modulation::Qpsk,
            0.5,
            m.eb_n0,
            ErrorMetric::Ber,
            1e-6,
        )
        .unwrap();
        assert!(stats.modulate(&channel(), &exact, 0.0.db()).closes());
        let too_hungry = ModCod::from_required_eb_n0(
            "too hungry",
            Modulation::Qpsk,
            0.5,
            m.eb_n0 + Decibel::new(0.1),
            ErrorMetric::Ber,
            1e-6,
        )
        .unwrap();
        assert!(!stats.modulate(&channel(), &too_hungry, 0.0.db()).closes());
    }

    #[test]
    fn test_modulated_with_interference_reduces_margin() {
        let m = component_stats().modulate(&channel(), &modcod(), 3.0.db());
        let interference = m.with_interference(Power::watts(1e-12)).unwrap();
        assert!(interference.margin_with_interference.as_f64() <= m.margin.as_f64());
        assert!(interference.eb_n0i0.as_f64() <= m.eb_n0.as_f64());
    }

    #[test]
    fn test_with_interference_rejects_non_physical_power() {
        let m = component_stats().modulate(&channel(), &modcod(), 3.0.db());
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
        let err = lumped_stats()
            .modulate(&channel(), &modcod(), 3.0.db())
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
