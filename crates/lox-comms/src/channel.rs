// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Waveform occupancy: how symbols occupy spectrum.
//!
//! A [`Channel`] describes the physical waveform — symbol rate, pulse
//! shaping, and optional DSSS spreading — and nothing else. What is
//! transmitted on it (modulation and coding) is a
//! [`ModCod`](crate::modcod::ModCod); link direction lives on
//! [`LinkParameters`](crate::link_budget::LinkParameters); acceptance
//! criteria (required Eb/N0, design margin) are evaluation inputs.

use core::fmt;
use std::str::FromStr;

use lox_core::units::{Decibel, Frequency};

use crate::error::NonPhysicalError;

/// Digital modulation scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Modulation {
    /// Binary Phase-Shift Keying (1 bit/symbol).
    Bpsk,
    /// Quadrature Phase-Shift Keying (2 bits/symbol).
    Qpsk,
    /// Offset Quadrature Phase-Shift Keying (2 bits/symbol).
    Oqpsk,
    /// 8-Phase-Shift Keying (3 bits/symbol).
    Psk8,
    /// 8-Amplitude-Phase-Shift Keying (3 bits/symbol, DVB-S2X).
    Apsk8,
    /// 16-Amplitude-Phase-Shift Keying (4 bits/symbol, DVB-S2).
    Apsk16,
    /// 32-Amplitude-Phase-Shift Keying (5 bits/symbol, DVB-S2).
    Apsk32,
    /// 64-Amplitude-Phase-Shift Keying (6 bits/symbol, DVB-S2X).
    Apsk64,
    /// 16-Quadrature Amplitude Modulation (4 bits/symbol).
    Qam16,
    /// 32-Quadrature Amplitude Modulation (5 bits/symbol).
    Qam32,
    /// 64-Quadrature Amplitude Modulation (6 bits/symbol).
    Qam64,
    /// 128-Quadrature Amplitude Modulation (7 bits/symbol).
    Qam128,
    /// 256-Quadrature Amplitude Modulation (8 bits/symbol).
    Qam256,
    /// Gaussian Minimum-Shift Keying (1 bit/symbol).
    Gmsk,
    /// Binary Frequency-Shift Keying (1 bit/symbol).
    Fsk2,
    /// Quaternary Frequency-Shift Keying (2 bits/symbol).
    Fsk4,
}

impl Modulation {
    /// Returns the number of bits per symbol for this modulation scheme.
    pub fn bits_per_symbol(self) -> u8 {
        match self {
            Modulation::Bpsk | Modulation::Gmsk | Modulation::Fsk2 => 1,
            Modulation::Qpsk | Modulation::Oqpsk | Modulation::Fsk4 => 2,
            Modulation::Psk8 | Modulation::Apsk8 => 3,
            Modulation::Qam16 | Modulation::Apsk16 => 4,
            Modulation::Qam32 | Modulation::Apsk32 => 5,
            Modulation::Qam64 | Modulation::Apsk64 => 6,
            Modulation::Qam128 => 7,
            Modulation::Qam256 => 8,
        }
    }

    /// Returns the conventional name, e.g. `"16APSK"`.
    pub fn name(self) -> &'static str {
        match self {
            Modulation::Bpsk => "BPSK",
            Modulation::Qpsk => "QPSK",
            Modulation::Oqpsk => "OQPSK",
            Modulation::Psk8 => "8PSK",
            Modulation::Apsk8 => "8APSK",
            Modulation::Apsk16 => "16APSK",
            Modulation::Apsk32 => "32APSK",
            Modulation::Apsk64 => "64APSK",
            Modulation::Qam16 => "16QAM",
            Modulation::Qam32 => "32QAM",
            Modulation::Qam64 => "64QAM",
            Modulation::Qam128 => "128QAM",
            Modulation::Qam256 => "256QAM",
            Modulation::Gmsk => "GMSK",
            Modulation::Fsk2 => "2FSK",
            Modulation::Fsk4 => "4FSK",
        }
    }

    /// All supported modulation schemes.
    pub const ALL: [Self; 16] = [
        Self::Bpsk,
        Self::Qpsk,
        Self::Oqpsk,
        Self::Psk8,
        Self::Apsk8,
        Self::Apsk16,
        Self::Apsk32,
        Self::Apsk64,
        Self::Qam16,
        Self::Qam32,
        Self::Qam64,
        Self::Qam128,
        Self::Qam256,
        Self::Gmsk,
        Self::Fsk2,
        Self::Fsk4,
    ];
}

impl fmt::Display for Modulation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The name does not match a known modulation scheme.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown modulation: '{0}'")]
pub struct ParseModulationError(String);

impl FromStr for Modulation {
    type Err = ParseModulationError;

    /// Parses a conventional modulation name, ignoring ASCII case.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|m| s.eq_ignore_ascii_case(m.name()))
            .ok_or_else(|| ParseModulationError(s.to_owned()))
    }
}

/// Link direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum LinkDirection {
    /// Ground-to-space link.
    Uplink,
    /// Space-to-ground link.
    Downlink,
    /// Inter-satellite link.
    Crosslink,
}

impl fmt::Display for LinkDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkDirection::Uplink => write!(f, "uplink"),
            LinkDirection::Downlink => write!(f, "downlink"),
            LinkDirection::Crosslink => write!(f, "crosslink"),
        }
    }
}

impl FromStr for LinkDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "uplink" => Ok(LinkDirection::Uplink),
            "downlink" => Ok(LinkDirection::Downlink),
            "crosslink" => Ok(LinkDirection::Crosslink),
            _ => Err(format!(
                "unknown link direction: '{s}', expected 'uplink', 'downlink', or 'crosslink'"
            )),
        }
    }
}

/// A communication channel: the waveform's occupancy of spectrum.
///
/// Holds the symbol rate, the pulse-shaping roll-off, and the optional DSSS
/// chip rate. Valid by construction via [`Channel::builder`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "ChannelRepr")
)]
pub struct Channel {
    symbol_rate: Frequency,
    roll_off: f64,
    chip_rate: Option<Frequency>,
}

/// Serde wire format for [`Channel`]: forces deserialization through the
/// validated builder.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct ChannelRepr {
    symbol_rate: Frequency,
    #[serde(default = "default_roll_off")]
    roll_off: f64,
    #[serde(default)]
    chip_rate: Option<Frequency>,
}

#[cfg(feature = "serde")]
fn default_roll_off() -> f64 {
    DEFAULT_ROLL_OFF
}

#[cfg(feature = "serde")]
impl TryFrom<ChannelRepr> for Channel {
    type Error = NonPhysicalError;

    fn try_from(repr: ChannelRepr) -> Result<Self, Self::Error> {
        let mut builder = Channel::builder(repr.symbol_rate).roll_off(repr.roll_off);
        if let Some(chip_rate) = repr.chip_rate {
            builder = builder.chip_rate(chip_rate);
        }
        builder.build()
    }
}

/// Default pulse-shaping roll-off factor.
const DEFAULT_ROLL_OFF: f64 = 0.35;

impl Channel {
    /// Starts building a channel with the given symbol rate.
    ///
    /// The roll-off defaults to 0.35 and the chip rate to `None`
    /// (narrowband).
    pub fn builder(symbol_rate: Frequency) -> ChannelBuilder {
        ChannelBuilder {
            symbol_rate,
            roll_off: DEFAULT_ROLL_OFF,
            chip_rate: None,
        }
    }

    /// Returns the symbol rate.
    pub fn symbol_rate(&self) -> Frequency {
        self.symbol_rate
    }

    /// Returns the pulse-shaping roll-off factor.
    pub fn roll_off(&self) -> f64 {
        self.roll_off
    }

    /// Returns the DSSS chip rate, or `None` for narrowband channels.
    pub fn chip_rate(&self) -> Option<Frequency> {
        self.chip_rate
    }

    /// Returns the occupied channel bandwidth.
    ///
    /// For narrowband: BW = R_s · (1 + α)
    /// For DSSS: BW = R_chip · (1 + α)
    pub fn bandwidth(&self) -> Frequency {
        let rate = self.chip_rate.unwrap_or(self.symbol_rate);
        (1.0 + self.roll_off) * rate
    }

    /// Computes Es/N0 (energy per symbol to noise spectral density) from C/N0.
    ///
    /// Es/N0 = C/N0 − 10·log₁₀(R_s)
    pub fn es_n0(&self, c_n0: Decibel) -> Decibel {
        c_n0 - Decibel::from_linear(self.symbol_rate.to_hertz())
    }

    /// Computes C/N (carrier-to-noise ratio) from C/N0.
    ///
    /// C/N = C/N0 − 10·log₁₀(BW_occupied)
    pub fn c_n(&self, c_n0: Decibel) -> Decibel {
        c_n0 - Decibel::from_linear(self.bandwidth().to_hertz())
    }

    /// Returns the DSSS spreading factor, or `None` for narrowband channels.
    ///
    /// SF = R_chip / R_s
    pub fn spreading_factor(&self) -> Option<f64> {
        self.chip_rate
            .map(|cr| cr.to_hertz() / self.symbol_rate.to_hertz())
    }

    /// Returns the DSSS processing gain in dB, or `None` for narrowband channels.
    ///
    /// PG = 10·log₁₀(SF)
    pub fn processing_gain(&self) -> Option<Decibel> {
        self.spreading_factor().map(Decibel::from_linear)
    }

    /// Computes Es/N0 at the chip rate (before despreading), or `None` for narrowband.
    ///
    /// Es/N0_spread = C/N0 − 10·log₁₀(R_chip)
    pub fn es_n0_spread(&self, c_n0: Decibel) -> Option<Decibel> {
        self.chip_rate
            .map(|cr| c_n0 - Decibel::from_linear(cr.to_hertz()))
    }
}

/// Builder for [`Channel`].
///
/// Created via [`Channel::builder`]. Inputs are validated at
/// [`ChannelBuilder::build`].
#[derive(Debug, Clone)]
pub struct ChannelBuilder {
    symbol_rate: Frequency,
    roll_off: f64,
    chip_rate: Option<Frequency>,
}

impl ChannelBuilder {
    /// Sets the pulse-shaping roll-off factor.
    pub fn roll_off(mut self, roll_off: f64) -> Self {
        self.roll_off = roll_off;
        self
    }

    /// Sets the DSSS chip rate.
    pub fn chip_rate(mut self, chip_rate: Frequency) -> Self {
        self.chip_rate = Some(chip_rate);
        self
    }

    /// Builds the channel, validating all inputs.
    ///
    /// Rejects a non-finite or non-positive symbol rate, a non-finite or
    /// negative roll-off, and a chip rate below the symbol rate.
    pub fn build(self) -> Result<Channel, NonPhysicalError> {
        NonPhysicalError::check_positive("symbol rate [Hz]", self.symbol_rate.to_hertz())?;
        NonPhysicalError::check_non_negative("roll-off factor", self.roll_off)?;
        if let Some(chip_rate) = self.chip_rate {
            NonPhysicalError::check_positive("chip rate [Hz]", chip_rate.to_hertz())?;
            NonPhysicalError::check_non_negative(
                "chip rate minus symbol rate [Hz]",
                chip_rate.to_hertz() - self.symbol_rate.to_hertz(),
            )?;
        }
        Ok(Channel {
            symbol_rate: self.symbol_rate,
            roll_off: self.roll_off,
            chip_rate: self.chip_rate,
        })
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use super::*;

    fn narrowband_channel() -> Channel {
        Channel::builder(5.0.mhz()).build().unwrap()
    }

    fn dsss_channel() -> Channel {
        Channel::builder(0.01.mhz())
            .chip_rate(4.0.mhz())
            .build()
            .unwrap()
    }

    #[test]
    fn test_bits_per_symbol() {
        assert_eq!(Modulation::Bpsk.bits_per_symbol(), 1);
        assert_eq!(Modulation::Qpsk.bits_per_symbol(), 2);
        assert_eq!(Modulation::Oqpsk.bits_per_symbol(), 2);
        assert_eq!(Modulation::Psk8.bits_per_symbol(), 3);
        assert_eq!(Modulation::Apsk8.bits_per_symbol(), 3);
        assert_eq!(Modulation::Apsk16.bits_per_symbol(), 4);
        assert_eq!(Modulation::Apsk32.bits_per_symbol(), 5);
        assert_eq!(Modulation::Apsk64.bits_per_symbol(), 6);
        assert_eq!(Modulation::Qam16.bits_per_symbol(), 4);
        assert_eq!(Modulation::Qam32.bits_per_symbol(), 5);
        assert_eq!(Modulation::Qam64.bits_per_symbol(), 6);
        assert_eq!(Modulation::Qam128.bits_per_symbol(), 7);
        assert_eq!(Modulation::Qam256.bits_per_symbol(), 8);
        assert_eq!(Modulation::Gmsk.bits_per_symbol(), 1);
        assert_eq!(Modulation::Fsk2.bits_per_symbol(), 1);
        assert_eq!(Modulation::Fsk4.bits_per_symbol(), 2);
    }

    #[test]
    fn test_modulation_name_parse_round_trip() {
        for m in Modulation::ALL {
            assert_eq!(m.name().parse::<Modulation>(), Ok(m));
            assert_eq!(m.to_string(), m.name());
        }
        // Parsing ignores ASCII case.
        assert_eq!("qpsk".parse::<Modulation>(), Ok(Modulation::Qpsk));
        assert_eq!("16apsk".parse::<Modulation>(), Ok(Modulation::Apsk16));
        assert!("3PSK".parse::<Modulation>().is_err());
    }

    #[test]
    fn test_bandwidth_narrowband() {
        // symbol_rate=5 MHz, default roll-off=0.35: BW = 5e6 * 1.35 = 6.75 MHz
        let ch = narrowband_channel();
        assert_approx_eq!(ch.bandwidth().to_hertz(), 6.75e6, rtol <= 1e-10);
    }

    #[test]
    fn test_bandwidth_with_roll_off() {
        // symbol_rate=1 MHz, roll-off=0.5: BW = 1e6 * 1.5 = 1.5 MHz
        let ch = Channel::builder(1.0.mhz()).roll_off(0.5).build().unwrap();
        assert_approx_eq!(ch.bandwidth().to_hertz(), 1.5e6, rtol <= 1e-10);
    }

    #[test]
    fn test_es_n0() {
        // C/N0 = 80 dBHz, symbol_rate = 5 MHz
        // Es/N0 = 80 - 10*log10(5e6) = 80 - 66.99 = 13.01 dB
        let ch = narrowband_channel();
        let es_n0 = ch.es_n0(80.0.db());
        assert_approx_eq!(es_n0.as_f64(), 13.0103, atol <= 0.001);
    }

    #[test]
    fn test_c_n() {
        // C/N0 = 80 dBHz, BW = 6.75 MHz
        let ch = narrowband_channel();
        let c_n = ch.c_n(80.0.db());
        let expected = 80.0 - 10.0 * 6.75e6_f64.log10();
        assert_approx_eq!(c_n.as_f64(), expected, atol <= 0.001);
    }

    #[test]
    fn test_dsss_spreading_factor_and_processing_gain() {
        let ch = dsss_channel();
        assert_approx_eq!(ch.chip_rate().unwrap().to_hertz(), 4e6, rtol <= 1e-12);
        assert_approx_eq!(ch.symbol_rate().to_hertz(), 1e4, rtol <= 1e-12);
        assert_approx_eq!(ch.spreading_factor().unwrap(), 400.0, rtol <= 1e-10);
        // PG = 10*log10(400) = 26.02 dB
        assert_approx_eq!(
            ch.processing_gain().unwrap().as_f64(),
            26.0206,
            atol <= 0.001
        );
        // DSSS: BW = chip_rate * (1 + roll_off) = 4e6 * 1.35 = 5.4 MHz
        assert_approx_eq!(ch.bandwidth().to_hertz(), 5.4e6, rtol <= 1e-10);
    }

    #[test]
    fn test_dsss_es_n0_spread_vs_despread() {
        let ch = dsss_channel();
        let c_n0 = 60.0.db();
        let diff = ch.es_n0(c_n0) - ch.es_n0_spread(c_n0).unwrap();
        assert_approx_eq!(
            diff.as_f64(),
            ch.processing_gain().unwrap().as_f64(),
            atol <= 1e-10
        );
    }

    #[test]
    fn test_narrowband_no_dsss() {
        let ch = narrowband_channel();
        assert!(ch.spreading_factor().is_none());
        assert!(ch.processing_gain().is_none());
        assert!(ch.es_n0_spread(80.0.db()).is_none());
    }

    #[test]
    fn test_channel_rejects_non_physical_inputs() {
        assert!(Channel::builder(Frequency::hertz(0.0)).build().is_err());
        assert!(Channel::builder(Frequency::hertz(-5e6)).build().is_err());
        assert!(Channel::builder(5.0.mhz()).roll_off(-0.1).build().is_err());
        assert!(
            Channel::builder(5.0.mhz())
                .roll_off(f64::NAN)
                .build()
                .is_err()
        );
        // Chip rate below the symbol rate is not a spreading system.
        assert!(
            Channel::builder(5.0.mhz())
                .chip_rate(1.0.mhz())
                .build()
                .is_err()
        );
        // Chip rate equal to the symbol rate is allowed (SF = 1).
        assert!(
            Channel::builder(5.0.mhz())
                .chip_rate(5.0.mhz())
                .build()
                .is_ok()
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_channel_serde_round_trip_and_validation() {
        let ch = dsss_channel();
        let json = serde_json::to_string(&ch).unwrap();
        let round_trip: Channel = serde_json::from_str(&json).unwrap();
        assert_eq!(ch, round_trip);

        // Roll-off defaults when omitted.
        let minimal: Channel = serde_json::from_str(r#"{"symbol_rate":5.0e6}"#).unwrap();
        assert_approx_eq!(minimal.roll_off(), 0.35, atol <= 1e-15);

        // Non-physical inputs are rejected at deserialization time.
        assert!(serde_json::from_str::<Channel>(r#"{"symbol_rate":-5.0e6}"#).is_err());
        assert!(
            serde_json::from_str::<Channel>(r#"{"symbol_rate":5.0e6,"roll_off":-1.0}"#).is_err()
        );
    }

    #[test]
    fn test_link_direction_display_and_parse() {
        assert_eq!(LinkDirection::Uplink.to_string(), "uplink");
        assert_eq!(LinkDirection::Downlink.to_string(), "downlink");
        assert_eq!(LinkDirection::Crosslink.to_string(), "crosslink");
        assert_eq!(
            "uplink".parse::<LinkDirection>().unwrap(),
            LinkDirection::Uplink
        );
        assert_eq!(
            "downlink".parse::<LinkDirection>().unwrap(),
            LinkDirection::Downlink
        );
        assert_eq!(
            "crosslink".parse::<LinkDirection>().unwrap(),
            LinkDirection::Crosslink
        );
        assert!("invalid".parse::<LinkDirection>().is_err());
    }
}
