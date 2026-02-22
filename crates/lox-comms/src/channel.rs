// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Communication channel model (modulation, bandwidth, Eb/N0).

use lox_core::units::Decibel;

/// Digital modulation scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Modulation {
    Bpsk,
    Qpsk,
    Psk8,
    Qam16,
    Qam32,
    Qam64,
    Qam128,
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
pub enum LinkDirection {
    Uplink,
    Downlink,
}

/// A communication channel.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Channel {
    /// Link direction.
    pub link_type: LinkDirection,
    /// Data rate in bits per second.
    pub data_rate: f64,
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
}

impl Channel {
    /// Returns the channel bandwidth in Hz.
    ///
    /// BW = R_b · (1 + α) / (b · FEC)
    pub fn bandwidth(&self) -> f64 {
        let b = self.modulation.bits_per_symbol() as f64;
        self.data_rate * (1.0 + self.roll_off) / (b * self.fec)
    }

    /// Computes Eb/N0 from a given C/N0.
    ///
    /// Eb/N0 = C/N0 − 10·log₁₀(R_b)
    pub fn eb_n0(&self, c_n0: Decibel) -> Decibel {
        c_n0 - Decibel::from_linear(self.data_rate)
    }

    /// Computes the link margin from a given Eb/N0.
    ///
    /// Margin = Eb/N0 − required_eb_n0 − margin
    pub fn link_margin(&self, eb_n0: Decibel) -> Decibel {
        eb_n0 - self.required_eb_n0 - self.margin
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::DecibelUnits;
    use lox_test_utils::assert_approx_eq;

    use super::*;

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
    fn test_bandwidth_bpsk() {
        // BPSK, 1 Mbit/s, roll-off=0.5, FEC=0.5
        // BW = 1e6 * 1.5 / (1 * 0.5) = 3 MHz
        let ch = Channel {
            link_type: LinkDirection::Downlink,
            data_rate: 1e6,
            required_eb_n0: 10.0.db(),
            margin: 3.0.db(),
            modulation: Modulation::Bpsk,
            roll_off: 0.5,
            fec: 0.5,
        };
        assert_approx_eq!(ch.bandwidth(), 3e6, rtol <= 1e-10);
    }

    #[test]
    fn test_bandwidth_qpsk() {
        // QPSK, 10 Mbit/s, roll-off=0.35, FEC=0.75
        // BW = 10e6 * 1.35 / (2 * 0.75) = 9 MHz
        let ch = Channel {
            link_type: LinkDirection::Downlink,
            data_rate: 10e6,
            required_eb_n0: 10.0.db(),
            margin: 3.0.db(),
            modulation: Modulation::Qpsk,
            roll_off: 0.35,
            fec: 0.75,
        };
        assert_approx_eq!(ch.bandwidth(), 9e6, rtol <= 1e-10);
    }

    #[test]
    fn test_eb_n0_from_c_n0() {
        // C/N0 = 80 dB·Hz, data_rate = 1 Mbit/s
        // Eb/N0 = 80 - 10*log10(1e6) = 80 - 60 = 20 dB
        let ch = Channel {
            link_type: LinkDirection::Downlink,
            data_rate: 1e6,
            required_eb_n0: 10.0.db(),
            margin: 3.0.db(),
            modulation: Modulation::Qpsk,
            roll_off: 0.35,
            fec: 0.5,
        };
        let eb_n0 = ch.eb_n0(80.0.db());
        assert_approx_eq!(eb_n0.as_f64(), 20.0, atol <= 1e-10);
    }

    #[test]
    fn test_link_margin() {
        // Eb/N0 = 15 dB, required = 10 dB, margin = 3 dB → link margin = 2 dB
        let ch = Channel {
            link_type: LinkDirection::Downlink,
            data_rate: 1e6,
            required_eb_n0: 10.0.db(),
            margin: 3.0.db(),
            modulation: Modulation::Qpsk,
            roll_off: 0.35,
            fec: 0.5,
        };
        let margin = ch.link_margin(15.0.db());
        assert_approx_eq!(margin.as_f64(), 2.0, atol <= 1e-10);
    }
}
