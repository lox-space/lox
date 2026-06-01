// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Communication channel model (modulation, bandwidth, Es/N0, Eb/N0, DSSS).

use lox_core::units::{Decibel, Frequency};

/// Digital modulation scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Modulation {
    /// Binary Phase-Shift Keying (1 bit/symbol).
    Bpsk,
    /// Quadrature Phase-Shift Keying (2 bits/symbol).
    Qpsk,
    /// 8-Phase-Shift Keying (3 bits/symbol).
    Psk8,
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
}

impl Modulation {
    /// Returns the number of bits per symbol for this modulation scheme.
    pub fn bits_per_symbol(self) -> u8 {
        match self {
            Modulation::Bpsk => 1,
            Modulation::Qpsk => 2,
            Modulation::Psk8 => 3,
            Modulation::Qam16 => 4,
            Modulation::Qam32 => 5,
            Modulation::Qam64 => 6,
            Modulation::Qam128 => 7,
            Modulation::Qam256 => 8,
        }
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

impl std::fmt::Display for LinkDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkDirection::Uplink => write!(f, "uplink"),
            LinkDirection::Downlink => write!(f, "downlink"),
            LinkDirection::Crosslink => write!(f, "crosslink"),
        }
    }
}

impl std::str::FromStr for LinkDirection {
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

/// A communication channel.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Channel {
    /// Link direction.
    pub link_type: LinkDirection,
    /// Symbol rate.
    pub symbol_rate: Frequency,
    /// Required Eb/N0 for the target BER.
    pub required_eb_n0: Decibel,
    /// Required link margin.
    pub margin: Decibel,
    /// Modulation scheme.
    pub modulation: Modulation,
    /// Roll-off factor (excess bandwidth factor).
    pub roll_off: f64,
    /// Forward error correction code rate.
    pub fec: f64,
    /// Chip rate for DSSS systems (`None` for narrowband).
    pub chip_rate: Option<Frequency>,
}

impl Channel {
    /// Returns the raw bit rate in bits per second.
    ///
    /// R_b = R_s · b
    pub fn data_rate(&self) -> Frequency {
        self.modulation.bits_per_symbol() as f64 * self.symbol_rate
    }

    /// Returns the information (post-FEC) bit rate in bits per second.
    ///
    /// R_info = R_s · b · FEC
    pub fn information_rate(&self) -> Frequency {
        self.fec * self.data_rate()
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

    /// Computes Eb/N0 (energy per information bit to noise spectral density) from C/N0.
    ///
    /// Eb/N0 = Es/N0 − 10·log₁₀(b · FEC)
    pub fn eb_n0(&self, c_n0: Decibel) -> Decibel {
        let es_n0 = self.es_n0(c_n0);
        let bps_fec = self.modulation.bits_per_symbol() as f64 * self.fec;
        es_n0 - Decibel::from_linear(bps_fec)
    }

    /// Computes C/N (carrier-to-noise ratio) from C/N0.
    ///
    /// C/N = C/N0 − 10·log₁₀(BW_occupied)
    pub fn c_n(&self, c_n0: Decibel) -> Decibel {
        c_n0 - Decibel::from_linear(self.bandwidth().to_hertz())
    }

    /// Computes the link margin from a given Eb/N0.
    ///
    /// Margin = Eb/N0 − required_eb_n0 − margin
    pub fn link_margin(&self, eb_n0: Decibel) -> Decibel {
        eb_n0 - self.required_eb_n0 - self.margin
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

    /// Layers modulation/FEC figures onto a modulation-agnostic [`crate::link_budget::LinkStats`].
    ///
    /// Computes `Es/N0`, `Eb/N0`, and link margin from the channel's modulation, FEC,
    /// symbol rate, required `Eb/N0`, and required margin.
    pub fn apply(
        &self,
        link: crate::link_budget::LinkStats,
    ) -> crate::link_budget::ModulatedLinkStats {
        let es_n0 = self.es_n0(link.c_n0);
        let eb_n0 = self.eb_n0(link.c_n0);
        let margin = self.link_margin(eb_n0);
        crate::link_budget::ModulatedLinkStats {
            link,
            channel: self.clone(),
            symbol_rate: self.symbol_rate,
            es_n0,
            eb_n0,
            margin,
            interference: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use super::*;

    fn narrowband_channel() -> Channel {
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

    #[test]
    fn test_bits_per_symbol() {
        assert_eq!(Modulation::Bpsk.bits_per_symbol(), 1);
        assert_eq!(Modulation::Qpsk.bits_per_symbol(), 2);
        assert_eq!(Modulation::Psk8.bits_per_symbol(), 3);
        assert_eq!(Modulation::Qam16.bits_per_symbol(), 4);
        assert_eq!(Modulation::Qam32.bits_per_symbol(), 5);
        assert_eq!(Modulation::Qam64.bits_per_symbol(), 6);
        assert_eq!(Modulation::Qam128.bits_per_symbol(), 7);
        assert_eq!(Modulation::Qam256.bits_per_symbol(), 8);
    }

    #[test]
    fn test_data_rate() {
        let ch = narrowband_channel();
        // QPSK: 2 bits/symbol, symbol_rate=5 MHz → data_rate=10 MHz
        assert_approx_eq!(ch.data_rate().to_hertz(), 10e6, rtol <= 1e-10);
    }

    #[test]
    fn test_information_rate() {
        let ch = narrowband_channel();
        // data_rate=10 MHz, fec=0.5 → information_rate=5 MHz
        assert_approx_eq!(ch.information_rate().to_hertz(), 5e6, rtol <= 1e-10);
    }

    #[test]
    fn test_bandwidth_narrowband() {
        // QPSK, symbol_rate=5 MHz, roll-off=0.35
        // BW = 5e6 * 1.35 = 6.75 MHz
        let ch = narrowband_channel();
        assert_approx_eq!(ch.bandwidth().to_hertz(), 6.75e6, rtol <= 1e-10);
    }

    #[test]
    fn test_bandwidth_bpsk() {
        // BPSK, symbol_rate=1 MHz, roll-off=0.5
        // BW = 1e6 * 1.5 = 1.5 MHz
        let ch = Channel {
            link_type: LinkDirection::Downlink,
            symbol_rate: 1.0.mhz(),
            required_eb_n0: 10.0.db(),
            margin: 3.0.db(),
            modulation: Modulation::Bpsk,
            roll_off: 0.5,
            fec: 0.5,
            chip_rate: None,
        };
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
    fn test_eb_n0() {
        // C/N0 = 80 dBHz, symbol_rate = 5 MHz, QPSK (2 bps), fec = 0.5
        // Es/N0 = 80 - 10*log10(5e6) = 13.0103
        // Eb/N0 = 13.0103 - 10*log10(2 * 0.5) = 13.0103 - 0 = 13.0103
        let ch = narrowband_channel();
        let eb_n0 = ch.eb_n0(80.0.db());
        assert_approx_eq!(eb_n0.as_f64(), 13.0103, atol <= 0.001);
    }

    #[test]
    fn test_eb_n0_with_higher_code_rate() {
        // symbol_rate=5 MHz, QPSK (2 bps), fec=5/6
        // Es/N0 = 80 - 10*log10(5e6) = 13.0103
        // Eb/N0 = 13.0103 - 10*log10(2 * 5/6) = 13.0103 - 2.2185 = 10.7918
        let ch = Channel {
            fec: 5.0 / 6.0,
            ..narrowband_channel()
        };
        let eb_n0 = ch.eb_n0(80.0.db());
        let expected = 13.0103 - 10.0 * (2.0 * 5.0 / 6.0_f64).log10();
        assert_approx_eq!(eb_n0.as_f64(), expected, atol <= 0.001);
    }

    #[test]
    fn test_c_n() {
        // C/N0 = 80 dBHz, BW = 6.75 MHz
        // C/N = 80 - 10*log10(6.75e6) = 80 - 68.29 = 11.71 dB
        let ch = narrowband_channel();
        let c_n = ch.c_n(80.0.db());
        let expected = 80.0 - 10.0 * 6.75e6_f64.log10();
        assert_approx_eq!(c_n.as_f64(), expected, atol <= 0.001);
    }

    #[test]
    fn test_link_margin() {
        // Eb/N0 = 15 dB, required = 10 dB, margin = 3 dB → link margin = 2 dB
        let ch = narrowband_channel();
        let margin = ch.link_margin(15.0.db());
        assert_approx_eq!(margin.as_f64(), 2.0, atol <= 1e-10);
    }

    #[test]
    fn test_dsss_spreading_factor() {
        let ch = Channel {
            chip_rate: Some(4.0.mhz()),
            symbol_rate: 0.01.mhz(),
            modulation: Modulation::Bpsk,
            ..narrowband_channel()
        };
        assert_approx_eq!(ch.spreading_factor().unwrap(), 400.0, rtol <= 1e-10);
    }

    #[test]
    fn test_dsss_processing_gain() {
        let ch = Channel {
            chip_rate: Some(4.0.mhz()),
            symbol_rate: 0.01.mhz(),
            modulation: Modulation::Bpsk,
            ..narrowband_channel()
        };
        // PG = 10*log10(400) = 26.02 dB
        assert_approx_eq!(
            ch.processing_gain().unwrap().as_f64(),
            26.0206,
            atol <= 0.001
        );
    }

    #[test]
    fn test_dsss_bandwidth() {
        // DSSS: BW = chip_rate * (1 + roll_off) = 4e6 * 1.35 = 5.4 MHz
        let ch = Channel {
            chip_rate: Some(4.0.mhz()),
            symbol_rate: 0.01.mhz(),
            modulation: Modulation::Bpsk,
            ..narrowband_channel()
        };
        assert_approx_eq!(ch.bandwidth().to_hertz(), 5.4e6, rtol <= 1e-10);
    }

    #[test]
    fn test_dsss_es_n0_spread_vs_despread() {
        let ch = Channel {
            chip_rate: Some(4.0.mhz()),
            symbol_rate: 0.01.mhz(),
            modulation: Modulation::Bpsk,
            ..narrowband_channel()
        };
        let c_n0 = 60.0.db();
        let es_n0_spread = ch.es_n0_spread(c_n0).unwrap();
        let es_n0_despread = ch.es_n0(c_n0);
        // Difference should equal processing gain
        let diff = es_n0_despread - es_n0_spread;
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
    fn test_link_direction_display() {
        assert_eq!(LinkDirection::Uplink.to_string(), "uplink");
        assert_eq!(LinkDirection::Downlink.to_string(), "downlink");
        assert_eq!(LinkDirection::Crosslink.to_string(), "crosslink");
    }

    #[test]
    fn test_link_direction_from_str() {
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
