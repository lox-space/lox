// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Modulation and coding: what is transmitted on a [`Channel`].
//!
//! A [`LinkMode`] pairs a modulation with an ordered chain of [`Code`]s —
//! every PHY component that adds overhead (FEC codes, framing structures)
//! contributes one rate factor, so concatenated codes and framing are
//! handled uniformly and the information rate is exact. A
//! [`ModePerformance`] records the published operating point (metric, error
//! rate, Eb/N0 threshold, provenance); together they form a [`ModCod`].
//! Tables like [`dvb_s2`] hold the standard mode sets;
//! [`ModCod::select`] picks the best closing mode for adaptive coding and
//! modulation.
//!
//! [`Channel`]: crate::channel::Channel

use core::fmt;
use std::sync::LazyLock;

use lox_core::units::{Decibel, Frequency};

use crate::channel::Modulation;
use crate::error::NonPhysicalError;

/// One PHY-layer component that adds overhead: an FEC code, a framing
/// structure, or any other element with a fixed information-to-channel-bit
/// ratio.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "CodeRepr")
)]
pub struct Code {
    name: String,
    rate: f64,
}

/// Serde wire format for [`Code`]: forces deserialization through the
/// validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct CodeRepr {
    name: String,
    rate: f64,
}

#[cfg(feature = "serde")]
impl TryFrom<CodeRepr> for Code {
    type Error = NonPhysicalError;

    fn try_from(repr: CodeRepr) -> Result<Self, Self::Error> {
        Code::new(repr.name, repr.rate)
    }
}

impl Code {
    /// Creates a code with the given rate.
    ///
    /// The rate is the ratio of information bits to total bits and must lie
    /// in (0, 1].
    pub fn new(name: impl Into<String>, rate: f64) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_unit_interval("code rate", rate)?;
        Ok(Self {
            name: name.into(),
            rate,
        })
    }

    /// Returns the name of the code.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the code rate (information bits / total bits).
    pub fn rate(&self) -> f64 {
        self.rate
    }
}

/// A modulation paired with its coding chain: the structure of what is
/// transmitted.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LinkMode {
    name: String,
    modulation: Modulation,
    codes: Vec<Code>,
    reference: String,
}

impl LinkMode {
    /// Creates a link mode from a modulation and a coding chain.
    ///
    /// `codes` lists every overhead-adding PHY component in encoding order
    /// from outermost to innermost; an empty list means uncoded.
    /// `reference` records the source of the definition (may be empty).
    pub fn new(
        name: impl Into<String>,
        modulation: Modulation,
        codes: Vec<Code>,
        reference: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            modulation,
            codes,
            reference: reference.into(),
        }
    }

    /// Returns the name of this mode.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the modulation scheme.
    pub fn modulation(&self) -> Modulation {
        self.modulation
    }

    /// Returns the coding chain in encoding order.
    pub fn codes(&self) -> &[Code] {
        &self.codes
    }

    /// Returns the source of this mode definition.
    pub fn reference(&self) -> &str {
        &self.reference
    }

    /// Returns the overall code rate: the product of the chain's rates.
    pub fn code_rate(&self) -> f64 {
        self.codes.iter().map(Code::rate).product()
    }

    /// Returns the information bits per symbol — the true spectral
    /// efficiency including all framing and coding overhead.
    pub fn info_bits_per_symbol(&self) -> f64 {
        self.modulation.bits_per_symbol() as f64 * self.code_rate()
    }

    /// Returns the information bit rate at the given symbol rate.
    pub fn information_rate(&self, symbol_rate: Frequency) -> Frequency {
        self.info_bits_per_symbol() * symbol_rate
    }

    /// Returns the symbol rate required for the given information bit rate.
    pub fn symbol_rate_for(&self, information_rate: Frequency) -> Frequency {
        Frequency::hertz(information_rate.to_hertz() / self.info_bits_per_symbol())
    }
}

/// The error metric a performance figure refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ErrorMetric {
    /// Bit error rate.
    Ber,
    /// Codeword error rate.
    Wer,
    /// Frame error rate.
    Fer,
    /// Packet error rate.
    Per,
}

impl fmt::Display for ErrorMetric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorMetric::Ber => write!(f, "BER"),
            ErrorMetric::Wer => write!(f, "WER"),
            ErrorMetric::Fer => write!(f, "FER"),
            ErrorMetric::Per => write!(f, "PER"),
        }
    }
}

/// A published operating point for a link mode: the Eb/N0 at which the mode
/// achieves a given error rate.
///
/// This is the threshold form of performance data (e.g. the quasi-error-free
/// points standards like DVB-S2 publish). Interpolated error-rate curves are
/// a future extension.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "ModePerformanceRepr")
)]
pub struct ModePerformance {
    metric: ErrorMetric,
    error_rate: f64,
    eb_n0: Decibel,
    reference: String,
}

/// Serde wire format for [`ModePerformance`]: forces deserialization through
/// the validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct ModePerformanceRepr {
    metric: ErrorMetric,
    error_rate: f64,
    eb_n0: Decibel,
    #[serde(default)]
    reference: String,
}

#[cfg(feature = "serde")]
impl TryFrom<ModePerformanceRepr> for ModePerformance {
    type Error = NonPhysicalError;

    fn try_from(repr: ModePerformanceRepr) -> Result<Self, Self::Error> {
        ModePerformance::new(repr.metric, repr.error_rate, repr.eb_n0, repr.reference)
    }
}

impl ModePerformance {
    /// Creates a performance threshold.
    ///
    /// Rejects an error rate outside (0, 1] and a non-finite Eb/N0.
    pub fn new(
        metric: ErrorMetric,
        error_rate: f64,
        eb_n0: Decibel,
        reference: impl Into<String>,
    ) -> Result<Self, NonPhysicalError> {
        NonPhysicalError::check_unit_interval("error rate", error_rate)?;
        NonPhysicalError::check_finite("Eb/N0 threshold [dB]", eb_n0.as_f64())?;
        Ok(Self {
            metric,
            error_rate,
            eb_n0,
            reference: reference.into(),
        })
    }

    /// Returns the error metric.
    pub fn metric(&self) -> ErrorMetric {
        self.metric
    }

    /// Returns the error rate at the threshold.
    pub fn error_rate(&self) -> f64 {
        self.error_rate
    }

    /// Returns the Eb/N0 threshold.
    pub fn eb_n0(&self) -> Decibel {
        self.eb_n0
    }

    /// Returns the source of the performance data.
    pub fn reference(&self) -> &str {
        &self.reference
    }
}

/// A link mode together with its performance: one row of a MODCOD table.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModCod {
    mode: LinkMode,
    performance: ModePerformance,
}

impl ModCod {
    /// Creates a MODCOD from a mode and its performance.
    pub fn new(mode: LinkMode, performance: ModePerformance) -> Self {
        Self { mode, performance }
    }

    /// Creates a MODCOD from the classic hand-rolled specification: a
    /// modulation, an aggregate code rate, and the Eb/N0 required for a
    /// target error rate.
    pub fn from_required_eb_n0(
        name: impl Into<String>,
        modulation: Modulation,
        code_rate: f64,
        required_eb_n0: Decibel,
        metric: ErrorMetric,
        error_rate: f64,
    ) -> Result<Self, NonPhysicalError> {
        let name = name.into();
        let codes = if code_rate == 1.0 {
            Vec::new()
        } else {
            vec![Code::new("FEC", code_rate)?]
        };
        let mode = LinkMode::new(name, modulation, codes, "");
        let performance = ModePerformance::new(metric, error_rate, required_eb_n0, "")?;
        Ok(Self { mode, performance })
    }

    /// Returns the link mode.
    pub fn mode(&self) -> &LinkMode {
        &self.mode
    }

    /// Returns the performance threshold.
    pub fn performance(&self) -> &ModePerformance {
        &self.performance
    }

    /// Returns the Eb/N0 this MODCOD requires.
    pub fn required_eb_n0(&self) -> Decibel {
        self.performance.eb_n0
    }

    /// Returns the Es/N0 this MODCOD requires:
    /// Es/N0 = Eb/N0 + 10·log₁₀(info bits per symbol).
    pub fn required_es_n0(&self) -> Decibel {
        self.performance.eb_n0 + Decibel::from_linear(self.mode.info_bits_per_symbol())
    }

    /// Selects the highest-efficiency MODCOD that closes at the given Es/N0
    /// with the given margin, or `None` when none does.
    ///
    /// Selection compares Es/N0 because it is fixed by the link and the
    /// symbol rate, while Eb/N0 differs between candidate modes.
    pub fn select(es_n0: Decibel, margin: Decibel, table: &[ModCod]) -> Option<&ModCod> {
        table
            .iter()
            .filter(|mc| mc.required_es_n0().as_f64() + margin.as_f64() <= es_n0.as_f64())
            .max_by(|a, b| {
                a.mode
                    .info_bits_per_symbol()
                    .total_cmp(&b.mode.info_bits_per_symbol())
            })
    }
}

/// DVB-S2 reference for the table entries.
const DVB_S2_REF: &str = "ETSI EN 302 307-1 V1.4.1";

/// Builds one DVB-S2 MODCOD (normal FECFRAME, no pilots).
///
/// The coding chain carries the exact overheads: the 80-bit BBHEADER, the
/// BCH and LDPC codes, and the 90-symbol PLHEADER per `slots`-slot PLFRAME.
/// Its product reproduces the spectral efficiencies of Table 13 exactly;
/// the Eb/N0 threshold derives from the table's ideal Es/N0 at
/// quasi-error-free operation (PER ≤ 1e-7).
fn dvb_s2_modcod(
    modulation: Modulation,
    rate_name: &str,
    k_bch: u32,
    n_bch: u32,
    slots: u32,
    es_n0_db: f64,
) -> ModCod {
    let codes = vec![
        Code::new(
            "BBFRAME overhead (80 bits)",
            (k_bch - 80) as f64 / k_bch as f64,
        )
        .unwrap(),
        Code::new(
            format!("BCH ({n_bch}, {k_bch})"),
            k_bch as f64 / n_bch as f64,
        )
        .unwrap(),
        Code::new(format!("LDPC (64800, {n_bch})"), n_bch as f64 / 64800.0).unwrap(),
        Code::new(
            "PLFRAME overhead (no pilots)",
            slots as f64 / (slots + 1) as f64,
        )
        .unwrap(),
    ];
    let mode = LinkMode::new(
        format!("{modulation} {rate_name}"),
        modulation,
        codes,
        DVB_S2_REF,
    );
    let eb_n0 = Decibel::new(es_n0_db) - Decibel::from_linear(mode.info_bits_per_symbol());
    let performance = ModePerformance::new(ErrorMetric::Per, 1e-7, eb_n0, DVB_S2_REF).unwrap();
    ModCod::new(mode, performance)
}

static DVB_S2: LazyLock<Vec<ModCod>> = LazyLock::new(|| {
    use Modulation::{Apsk16, Apsk32, Psk8, Qpsk};
    vec![
        // (modulation, rate, k_bch, n_bch = k_ldpc, PLFRAME slots, Es/N0 [dB])
        dvb_s2_modcod(Qpsk, "1/4", 16008, 16200, 360, -2.35),
        dvb_s2_modcod(Qpsk, "1/3", 21408, 21600, 360, -1.24),
        dvb_s2_modcod(Qpsk, "2/5", 25728, 25920, 360, -0.30),
        dvb_s2_modcod(Qpsk, "1/2", 32208, 32400, 360, 1.00),
        dvb_s2_modcod(Qpsk, "3/5", 38688, 38880, 360, 2.23),
        dvb_s2_modcod(Qpsk, "2/3", 43040, 43200, 360, 3.10),
        dvb_s2_modcod(Qpsk, "3/4", 48408, 48600, 360, 4.03),
        dvb_s2_modcod(Qpsk, "4/5", 51648, 51840, 360, 4.68),
        dvb_s2_modcod(Qpsk, "5/6", 53840, 54000, 360, 5.18),
        dvb_s2_modcod(Qpsk, "8/9", 57472, 57600, 360, 6.20),
        dvb_s2_modcod(Qpsk, "9/10", 58192, 58320, 360, 6.42),
        dvb_s2_modcod(Psk8, "3/5", 38688, 38880, 240, 5.50),
        dvb_s2_modcod(Psk8, "2/3", 43040, 43200, 240, 6.62),
        dvb_s2_modcod(Psk8, "3/4", 48408, 48600, 240, 7.91),
        dvb_s2_modcod(Psk8, "5/6", 53840, 54000, 240, 9.35),
        dvb_s2_modcod(Psk8, "8/9", 57472, 57600, 240, 10.69),
        dvb_s2_modcod(Psk8, "9/10", 58192, 58320, 240, 10.98),
        dvb_s2_modcod(Apsk16, "2/3", 43040, 43200, 180, 8.97),
        dvb_s2_modcod(Apsk16, "3/4", 48408, 48600, 180, 10.21),
        dvb_s2_modcod(Apsk16, "4/5", 51648, 51840, 180, 11.03),
        dvb_s2_modcod(Apsk16, "5/6", 53840, 54000, 180, 11.61),
        dvb_s2_modcod(Apsk16, "8/9", 57472, 57600, 180, 12.89),
        dvb_s2_modcod(Apsk16, "9/10", 58192, 58320, 180, 13.13),
        dvb_s2_modcod(Apsk32, "3/4", 48408, 48600, 144, 12.73),
        dvb_s2_modcod(Apsk32, "4/5", 51648, 51840, 144, 13.64),
        dvb_s2_modcod(Apsk32, "5/6", 53840, 54000, 144, 14.28),
        dvb_s2_modcod(Apsk32, "8/9", 57472, 57600, 144, 15.69),
        dvb_s2_modcod(Apsk32, "9/10", 58192, 58320, 144, 16.05),
    ]
});

/// Returns the DVB-S2 MODCOD table (normal FECFRAME, no pilots).
///
/// 28 modes from QPSK 1/4 through 32APSK 9/10 with ideal Es/N0 thresholds
/// at quasi-error-free operation (PER ≤ 1e-7), per ETSI EN 302 307-1
/// Table 13. DVB-S2X and CCSDS tables are not included yet.
pub fn dvb_s2() -> &'static [ModCod] {
    &DVB_S2
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;
    use lox_core::units::{DecibelUnits, FrequencyUnits};

    use super::*;

    #[test]
    fn test_code_rejects_invalid_rates() {
        for rate in [0.0, -0.5, 1.5, f64::NAN] {
            assert!(Code::new("FEC", rate).is_err());
        }
        assert!(Code::new("uncoded", 1.0).is_ok());
    }

    #[test]
    fn test_link_mode_chain_rate_is_product() {
        // CCSDS-style concatenated coding: RS(255,223) + convolutional 1/2.
        let mode = LinkMode::new(
            "CCSDS RS+conv",
            Modulation::Bpsk,
            vec![
                Code::new("Reed-Solomon (255, 223)", 223.0 / 255.0).unwrap(),
                Code::new("Convolutional (7, 1/2)", 0.5).unwrap(),
            ],
            "CCSDS 131.0-B-5",
        );
        assert_approx_eq!(mode.code_rate(), 223.0 / 510.0, rtol <= 1e-12);
        assert_approx_eq!(mode.info_bits_per_symbol(), 223.0 / 510.0, rtol <= 1e-12);
        // 1 Msps → 437.25 kbit/s information rate, and back.
        let info = mode.information_rate(1.0.mhz());
        assert_approx_eq!(info.to_hertz(), 1e6 * 223.0 / 510.0, rtol <= 1e-12);
        assert_approx_eq!(mode.symbol_rate_for(info).to_hertz(), 1e6, rtol <= 1e-12);
    }

    #[test]
    fn test_uncoded_mode_has_unity_rate() {
        let mode = LinkMode::new("uncoded QPSK", Modulation::Qpsk, Vec::new(), "");
        assert_approx_eq!(mode.code_rate(), 1.0, atol <= 1e-15);
        assert_approx_eq!(mode.info_bits_per_symbol(), 2.0, atol <= 1e-15);
    }

    /// The coding chains must reproduce the spectral efficiencies of
    /// ETSI EN 302 307-1 Table 13 exactly.
    #[test]
    fn test_dvb_s2_efficiencies_match_table_13() {
        let table = dvb_s2();
        assert_eq!(table.len(), 28);
        for (name, expected) in [
            ("QPSK 1/4", 0.490243),
            ("QPSK 1/2", 0.988858),
            ("QPSK 2/3", 1.322253),
            ("QPSK 9/10", 1.788612),
            ("8PSK 3/5", 1.779991),
            ("8PSK 3/4", 2.228124),
            ("8PSK 9/10", 2.679207),
            ("16APSK 2/3", 2.637201),
            ("16APSK 9/10", 3.567342),
            ("32APSK 3/4", 3.703295),
            ("32APSK 9/10", 4.453027),
        ] {
            let mc = table
                .iter()
                .find(|mc| mc.mode().name() == name)
                .unwrap_or_else(|| panic!("missing {name}"));
            assert_approx_eq!(mc.mode().info_bits_per_symbol(), expected, atol <= 5e-7);
        }
    }

    /// Eb/N0 thresholds must round-trip to the published Es/N0 values.
    #[test]
    fn test_dvb_s2_thresholds_round_trip_to_es_n0() {
        for (name, es_n0) in [
            ("QPSK 1/4", -2.35),
            ("QPSK 1/2", 1.00),
            ("8PSK 3/4", 7.91),
            ("16APSK 2/3", 8.97),
            ("32APSK 9/10", 16.05),
        ] {
            let mc = dvb_s2().iter().find(|mc| mc.mode().name() == name).unwrap();
            assert_approx_eq!(mc.required_es_n0().as_f64(), es_n0, atol <= 1e-10);
        }
        // Cross-check against spacelink's independently derived Eb/N0 for
        // 16APSK 2/3 (8.97 − 10·log10(2.637201) = 4.76 dB).
        let mc = dvb_s2()
            .iter()
            .find(|mc| mc.mode().name() == "16APSK 2/3")
            .unwrap();
        assert_approx_eq!(mc.required_eb_n0().as_f64(), 4.76, atol <= 5e-3);
    }

    #[test]
    fn test_dvb_s2_is_monotonic_in_es_n0_and_efficiency() {
        // Within each modulation, higher code rate needs more Es/N0 and
        // yields more bits per symbol.
        let table = dvb_s2();
        for pair in table.windows(2) {
            if pair[0].mode().modulation() == pair[1].mode().modulation() {
                assert!(pair[0].required_es_n0().as_f64() < pair[1].required_es_n0().as_f64());
                assert!(
                    pair[0].mode().info_bits_per_symbol() < pair[1].mode().info_bits_per_symbol()
                );
            }
        }
    }

    #[test]
    fn test_select_picks_highest_closing_efficiency() {
        let table = dvb_s2();
        // Plenty of Es/N0: the top mode closes.
        let best = ModCod::select(20.0.db(), 0.0.db(), table).unwrap();
        assert_eq!(best.mode().name(), "32APSK 9/10");
        // 10 dB with 1 dB margin: best mode with Es/N0 ≤ 9 dB is 16APSK 2/3
        // (8.97 dB, 2.637 bit/sym) — more efficient than 8PSK 3/4 (7.91 dB,
        // 2.228 bit/sym) which also closes.
        let mc = ModCod::select(10.0.db(), 1.0.db(), table).unwrap();
        assert_eq!(mc.mode().name(), "16APSK 2/3");
        // Below the weakest mode nothing closes.
        assert!(ModCod::select(-3.0.db(), 0.0.db(), table).is_none());
        // Exactly on a threshold closes (inclusive).
        let mc = ModCod::select(1.0.db(), 0.0.db(), table).unwrap();
        assert_eq!(mc.mode().name(), "QPSK 1/2");
    }

    #[test]
    fn test_from_required_eb_n0() {
        let mc = ModCod::from_required_eb_n0(
            "my downlink",
            Modulation::Qpsk,
            0.5,
            10.0.db(),
            ErrorMetric::Ber,
            1e-6,
        )
        .unwrap();
        assert_approx_eq!(mc.mode().info_bits_per_symbol(), 1.0, atol <= 1e-15);
        assert_approx_eq!(mc.required_eb_n0().as_f64(), 10.0, atol <= 1e-15);
        // Es/N0 = Eb/N0 + 10·log10(1.0) = Eb/N0.
        assert_approx_eq!(mc.required_es_n0().as_f64(), 10.0, atol <= 1e-12);
        assert!(
            ModCod::from_required_eb_n0(
                "bad",
                Modulation::Qpsk,
                1.5,
                10.0.db(),
                ErrorMetric::Ber,
                1e-6
            )
            .is_err()
        );
    }

    #[test]
    fn test_accessors_round_trip() {
        let code = Code::new("LDPC", 0.5).unwrap();
        assert_eq!(code.name(), "LDPC");
        assert_approx_eq!(code.rate(), 0.5, atol <= 1e-15);

        let mode = LinkMode::new("test", Modulation::Qpsk, vec![code.clone()], "a ref");
        assert_eq!(mode.name(), "test");
        assert_eq!(mode.modulation(), Modulation::Qpsk);
        assert_eq!(mode.codes(), &[code]);
        assert_eq!(mode.reference(), "a ref");

        let perf = ModePerformance::new(ErrorMetric::Wer, 1e-5, 4.0.db(), "b ref").unwrap();
        assert_eq!(perf.metric(), ErrorMetric::Wer);
        assert_approx_eq!(perf.error_rate(), 1e-5, atol <= 1e-20);
        assert_approx_eq!(perf.eb_n0().as_f64(), 4.0, atol <= 1e-15);
        assert_eq!(perf.reference(), "b ref");

        let mc = ModCod::new(mode.clone(), perf.clone());
        assert_eq!(mc.mode(), &mode);
        assert_eq!(mc.performance(), &perf);
    }

    #[test]
    fn test_error_metric_display() {
        assert_eq!(ErrorMetric::Ber.to_string(), "BER");
        assert_eq!(ErrorMetric::Wer.to_string(), "WER");
        assert_eq!(ErrorMetric::Fer.to_string(), "FER");
        assert_eq!(ErrorMetric::Per.to_string(), "PER");
    }

    #[test]
    fn test_from_required_eb_n0_uncoded() {
        // A unity code rate yields an empty coding chain.
        let mc = ModCod::from_required_eb_n0(
            "uncoded",
            Modulation::Bpsk,
            1.0,
            9.6.db(),
            ErrorMetric::Ber,
            1e-5,
        )
        .unwrap();
        assert!(mc.mode().codes().is_empty());
        assert_approx_eq!(mc.mode().info_bits_per_symbol(), 1.0, atol <= 1e-15);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_modcod_serde_round_trip_and_validation() {
        let mc = dvb_s2()[3].clone();
        let json = serde_json::to_string(&mc).unwrap();
        let round_trip: ModCod = serde_json::from_str(&json).unwrap();
        assert_eq!(mc, round_trip);

        // Invalid code rates and error rates are rejected.
        assert!(serde_json::from_str::<Code>(r#"{"name":"FEC","rate":1.5}"#).is_err());
        assert!(
            serde_json::from_str::<ModePerformance>(
                r#"{"metric":"Ber","error_rate":2.0,"eb_n0":10.0}"#
            )
            .is_err()
        );
    }
}
