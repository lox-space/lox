// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Link budget calculations and the contracts they consume.
//!
//! [`LinkStats::for_link`] computes a modulation-agnostic budget between any
//! transmit end implementing [`Eirp`] and any receive end implementing
//! [`GOverT`], under the conditions captured in [`LinkParameters`]. The
//! built-in implementors live in [`terminal`](crate::terminal); custom ends
//! only need to answer the trait questions.

use lox_core::units::{Decibel, Distance, Frequency, Power, Temperature};

use crate::channel::{Channel, LinkDirection};
use crate::error::NonPhysicalError;
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
pub struct RxTerms {
    gain: Decibel,
    system_noise_temperature: Temperature,
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

/// The conditions a link budget is evaluated under: carrier, noise
/// bandwidth, slant range, environmental losses, and the pointing at each
/// end.
///
/// Valid by construction — [`LinkParameters::builder`] validates the
/// physical inputs, so [`LinkStats::for_link`] only fails on physics (a
/// carrier outside a terminal's band or an unresolvable pointing).
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "LinkParametersRepr")
)]
pub struct LinkParameters {
    carrier: Frequency,
    bandwidth: Frequency,
    range: Distance,
    losses: PropagationLosses,
    tx_pointing: Pointing,
    rx_pointing: Pointing,
    direction: Option<LinkDirection>,
}

/// Serde wire format for [`LinkParameters`]: forces deserialization through
/// the validated builder.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct LinkParametersRepr {
    carrier: Frequency,
    bandwidth: Frequency,
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
impl TryFrom<LinkParametersRepr> for LinkParameters {
    type Error = NonPhysicalError;

    fn try_from(repr: LinkParametersRepr) -> Result<Self, Self::Error> {
        let mut builder = LinkParameters::builder(repr.carrier, repr.bandwidth, repr.range)
            .losses(repr.losses)
            .tx_pointing(repr.tx_pointing)
            .rx_pointing(repr.rx_pointing);
        if let Some(direction) = repr.direction {
            builder = builder.direction(direction);
        }
        builder.build()
    }
}

impl LinkParameters {
    /// Starts building link parameters for the given carrier, noise
    /// bandwidth, and slant range.
    ///
    /// Losses default to none and both pointings to boresight.
    pub fn builder(
        carrier: Frequency,
        bandwidth: Frequency,
        range: Distance,
    ) -> LinkParametersBuilder {
        LinkParametersBuilder {
            carrier,
            bandwidth,
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

    /// Returns the noise bandwidth.
    pub fn bandwidth(&self) -> Frequency {
        self.bandwidth
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

/// Builder for [`LinkParameters`].
///
/// Created via [`LinkParameters::builder`]. Inputs are validated at
/// [`LinkParametersBuilder::build`].
#[derive(Debug, Clone)]
pub struct LinkParametersBuilder {
    carrier: Frequency,
    bandwidth: Frequency,
    range: Distance,
    losses: PropagationLosses,
    tx_pointing: Pointing,
    rx_pointing: Pointing,
    direction: Option<LinkDirection>,
}

impl LinkParametersBuilder {
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

    /// Builds the link parameters, validating all physical inputs.
    ///
    /// Rejects a non-finite or non-positive carrier frequency, noise
    /// bandwidth, or slant range.
    pub fn build(self) -> Result<LinkParameters, NonPhysicalError> {
        for (quantity, value) in [
            ("carrier frequency [Hz]", self.carrier.to_hertz()),
            ("noise bandwidth [Hz]", self.bandwidth.to_hertz()),
            ("slant range [m]", self.range.to_meters()),
        ] {
            NonPhysicalError::check_positive(quantity, value)?;
        }
        Ok(LinkParameters {
            carrier: self.carrier,
            bandwidth: self.bandwidth,
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
pub struct LinkStats {
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
    /// Noise power in the channel bandwidth. `None` for receive ends without
    /// [`RxTerms`].
    pub noise_power: Option<Decibel>,
    /// Channel noise bandwidth.
    pub bandwidth: Frequency,
    /// Carrier-to-noise density ratio (C/N₀).
    pub c_n0: Decibel,
    /// Carrier-to-noise ratio (C/N).
    pub c_n: Decibel,
}

impl LinkStats {
    /// Computes a modulation-agnostic link budget between two link ends
    /// under the given conditions.
    ///
    /// The carrier must lie inside both ends' frequency ranges. Each end
    /// resolves its own pointing against its antenna. On downlinks the
    /// budget uses the rain-degraded G/T when the receive end exposes
    /// degradable terms (see [`GOverT::rx_terms_degraded`]); aggregate G/T
    /// figures stay at their clear-sky value.
    pub fn for_link(
        tx: &impl Eirp,
        rx: &impl GOverT,
        params: &LinkParameters,
    ) -> Result<Self, LinkBudgetError> {
        let carrier = params.carrier;
        let bandwidth = params.bandwidth;

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
        let c_n = c_n0 - Decibel::from_linear(bandwidth.to_hertz());

        let carrier_rx_power = effective_terms
            .as_ref()
            .map(|terms| eirp - fspl - env_loss + terms.gain());
        let noise_power = effective_terms.as_ref().map(|terms| {
            Decibel::from_linear(
                terms.system_noise_temperature().to_kelvin()
                    * BOLTZMANN_CONSTANT
                    * bandwidth.to_hertz(),
            )
        });

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
            noise_power,
            bandwidth,
            c_n0,
            c_n,
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

    use crate::antenna::ConstantAntenna;
    use crate::channel::{LinkDirection, Modulation};
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

    fn link_params(bandwidth: Frequency) -> LinkParameters {
        LinkParameters::builder(29.0.ghz(), bandwidth, Distance::kilometers(1000.0))
            .build()
            .unwrap()
    }

    fn component_stats() -> LinkStats {
        LinkStats::for_link(
            &tx_chain(),
            &rx_chain(),
            &link_params(channel().bandwidth()),
        )
        .unwrap()
    }

    fn lumped_stats() -> LinkStats {
        LinkStats::for_link(
            &EirpModel::new(ka_band(), 55.0.db()).unwrap(),
            &GtModel::new(ka_band(), 3.01.db()).unwrap(),
            &link_params(5.0.mhz()),
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

        let stats = LinkStats::for_link(&FixedTx, &FixedRx, &link_params(5.0.mhz())).unwrap();
        assert_approx_eq!(stats.c_n0.as_f64(), 104.913, atol <= 0.01);
        // The default rx_terms is None: no absolute powers.
        assert!(stats.carrier_rx_power.is_none());
        assert!(stats.noise_power.is_none());
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
        assert!(stats.noise_power.is_some());
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
        let stats = lumped_stats();
        assert_approx_eq!(stats.c_n0.as_f64(), 104.913, atol <= 0.01);
        assert!(stats.carrier_rx_power.is_none());
        assert!(stats.noise_power.is_none());
    }

    #[test]
    fn test_for_link_rejects_carrier_out_of_band() {
        // 29 GHz fits the TX band but not the RX band.
        let err = LinkStats::for_link(
            &EirpModel::new(ka_band(), 55.0.db()).unwrap(),
            &GtModel::new(
                FrequencyRange::new(17.0.ghz(), 21.0.ghz()).unwrap(),
                3.01.db(),
            )
            .unwrap(),
            &link_params(5.0.mhz()),
        )
        .unwrap_err();

        assert!(matches!(err, LinkBudgetError::CarrierOutOfBand { .. }));
        assert!(err.to_string().contains("17.000–21.000 GHz"));
    }

    #[test]
    fn test_for_link_rejects_carrier_out_of_band_on_tx_side() {
        let err = LinkStats::for_link(
            &EirpModel::new(
                FrequencyRange::new(17.0.ghz(), 21.0.ghz()).unwrap(),
                55.0.db(),
            )
            .unwrap(),
            &GtModel::new(ka_band(), 3.01.db()).unwrap(),
            &link_params(5.0.mhz()),
        )
        .unwrap_err();
        assert!(matches!(err, LinkBudgetError::CarrierOutOfBand { .. }));
    }

    #[test]
    fn test_link_parameters_reject_non_physical_inputs() {
        for (carrier, bandwidth, range) in [
            (
                Frequency::hertz(0.0),
                5.0.mhz(),
                Distance::kilometers(1000.0),
            ),
            (
                Frequency::hertz(f64::NAN),
                5.0.mhz(),
                Distance::kilometers(1000.0),
            ),
            (
                29.0.ghz(),
                Frequency::hertz(0.0),
                Distance::kilometers(1000.0),
            ),
            (
                29.0.ghz(),
                Frequency::hertz(-5e6),
                Distance::kilometers(1000.0),
            ),
            (29.0.ghz(), 5.0.mhz(), Distance::kilometers(0.0)),
            (29.0.ghz(), 5.0.mhz(), Distance::kilometers(-1.0)),
        ] {
            assert!(
                LinkParameters::builder(carrier, bandwidth, range)
                    .build()
                    .is_err()
            );
        }
    }

    #[test]
    fn test_link_parameters_defaults() {
        let params = link_params(5.0.mhz());
        assert_approx_eq!(params.losses().total().as_f64(), 0.0, atol <= 1e-15);
        assert_eq!(params.tx_pointing(), Pointing::Boresight);
        assert_eq!(params.rx_pointing(), Pointing::Boresight);
        assert_approx_eq!(params.carrier().to_gigahertz(), 29.0, rtol <= 1e-12);
        assert_approx_eq!(params.bandwidth().to_megahertz(), 5.0, rtol <= 1e-12);
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

    fn faded_params(direction: Option<LinkDirection>) -> LinkParameters {
        let mut builder =
            LinkParameters::builder(29.0.ghz(), 5.0.mhz(), Distance::kilometers(1000.0))
                .losses(p618_losses());
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
        let clear = LinkStats::for_link(&tx_chain(), &rx_chain(), &faded_params(None)).unwrap();
        let faded = LinkStats::for_link(
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
        let c_n0_from_power = faded.carrier_rx_power.unwrap() - faded.noise_power.unwrap()
            + Decibel::from_linear(faded.bandwidth.to_hertz());
        assert_approx_eq!(faded.c_n0.as_f64(), c_n0_from_power.as_f64(), atol <= 1e-10);
    }

    #[test]
    fn test_for_link_uplink_and_crosslink_stay_clear_sky() {
        let clear = LinkStats::for_link(&tx_chain(), &rx_chain(), &faded_params(None)).unwrap();
        for direction in [LinkDirection::Uplink, LinkDirection::Crosslink] {
            let stats =
                LinkStats::for_link(&tx_chain(), &rx_chain(), &faded_params(Some(direction)))
                    .unwrap();
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
        let clear = LinkStats::for_link(&tx, &rx, &faded_params(None)).unwrap();
        let faded =
            LinkStats::for_link(&tx, &rx, &faded_params(Some(LinkDirection::Downlink))).unwrap();
        assert!(faded.gt_degraded.is_none());
        assert_approx_eq!(faded.c_n0.as_f64(), clear.c_n0.as_f64(), atol <= 1e-12);
    }

    #[test]
    fn test_for_link_downlink_without_absorption_is_a_no_op() {
        let params = LinkParameters::builder(29.0.ghz(), 5.0.mhz(), Distance::kilometers(1000.0))
            .direction(LinkDirection::Downlink)
            .build()
            .unwrap();
        let stats = LinkStats::for_link(&tx_chain(), &rx_chain(), &params).unwrap();
        // L_abs = 1: degraded equals clear-sky.
        assert_approx_eq!(
            stats.gt_degraded.unwrap().as_f64(),
            stats.gt.as_f64(),
            atol <= 1e-12
        );
        assert_approx_eq!(stats.c_n0.as_f64(), 104.913, atol <= 0.01);
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
        let params = LinkParameters::builder(29.0.ghz(), 5.0.mhz(), Distance::kilometers(1000.0))
            .losses(losses)
            .build()
            .unwrap();
        let faded = LinkStats::for_link(
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
        let err = channel()
            .apply(lumped_stats())
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
