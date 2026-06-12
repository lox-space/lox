// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Shared communications vocabulary.
//!
//! Domain types exchanged between the propagation ([`lox-itur`]) and link
//! budget ([`lox-comms`]) crates: IEEE letter bands, explicit frequency
//! ranges, and itemized propagation losses.
//!
//! [`lox-itur`]: https://crates.io/crates/lox-itur
//! [`lox-comms`]: https://crates.io/crates/lox-comms

use core::fmt::{Display, Formatter, Result};
use core::str::FromStr;

use crate::units::{Distance, Frequency, SPEED_OF_LIGHT};

/// The letter code does not name a known frequency band.
#[derive(Copy, Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("unknown frequency band")]
pub struct ParseFrequencyBandError;

/// IEEE letter codes for frequency bands commonly used for satellite communications.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FrequencyBand {
    /// HF (High Frequency) – 3 to 30 MHz
    HF,
    /// VHF (Very High Frequency) – 30 to 300 MHz
    VHF,
    /// UHF (Ultra-High Frequency) – 0.3 to 1 GHz
    UHF,
    /// L – 1 to 2 GHz
    L,
    /// S – 2 to 4 GHz
    S,
    /// C – 4 to 8 GHz
    C,
    /// X – 8 to 12 GHz
    X,
    /// Kᵤ – 12 to 18 GHz
    Ku,
    /// K – 18 to 27 GHz
    K,
    /// Kₐ – 27 to 40 GHz
    Ka,
    /// V – 40 to 75 GHz
    V,
    /// W – 75 to 110 GHz
    W,
    /// G – 110 to 300 GHz
    G,
}

impl FrequencyBand {
    /// All IEEE letter bands in ascending frequency order.
    pub const ALL: [Self; 13] = [
        Self::HF,
        Self::VHF,
        Self::UHF,
        Self::L,
        Self::S,
        Self::C,
        Self::X,
        Self::Ku,
        Self::K,
        Self::Ka,
        Self::V,
        Self::W,
        Self::G,
    ];

    /// Returns the band's lower and upper frequency bounds.
    pub const fn bounds(&self) -> (Frequency, Frequency) {
        let (min_hz, max_hz) = match self {
            Self::HF => (3e6, 30e6),
            Self::VHF => (30e6, 300e6),
            Self::UHF => (300e6, 1e9),
            Self::L => (1e9, 2e9),
            Self::S => (2e9, 4e9),
            Self::C => (4e9, 8e9),
            Self::X => (8e9, 12e9),
            Self::Ku => (12e9, 18e9),
            Self::K => (18e9, 27e9),
            Self::Ka => (27e9, 40e9),
            Self::V => (40e9, 75e9),
            Self::W => (75e9, 110e9),
            Self::G => (110e9, 300e9),
        };
        (Frequency::hertz(min_hz), Frequency::hertz(max_hz))
    }

    /// Returns the band's lower frequency bound.
    pub const fn min(&self) -> Frequency {
        self.bounds().0
    }

    /// Returns the band's upper frequency bound.
    pub const fn max(&self) -> Frequency {
        self.bounds().1
    }

    /// Returns the IEEE letter code, e.g. `"Ka"`.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::HF => "HF",
            Self::VHF => "VHF",
            Self::UHF => "UHF",
            Self::L => "L",
            Self::S => "S",
            Self::C => "C",
            Self::X => "X",
            Self::Ku => "Ku",
            Self::K => "K",
            Self::Ka => "Ka",
            Self::V => "V",
            Self::W => "W",
            Self::G => "G",
        }
    }
}

impl Display for FrequencyBand {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(self.name())
    }
}

impl FromStr for FrequencyBand {
    type Err = ParseFrequencyBandError;

    /// Parses an IEEE letter code, ignoring ASCII case (`"Ka"`, `"ka"`, `"KA"`).
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|band| s.eq_ignore_ascii_case(band.name()))
            .ok_or(ParseFrequencyBandError)
    }
}

/// A contiguous frequency range with inclusive bounds.
///
/// Declares the capability of a link terminal: link validation checks that
/// the carrier lies inside the range. Unlike [`FrequencyBand`] — the IEEE
/// letter-band classification — a `FrequencyRange` carries explicit bounds
/// and can describe optical bands via [`Self::from_wavelengths`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "FrequencyRangeRepr", into = "FrequencyRangeRepr")
)]
pub struct FrequencyRange {
    min: Frequency,
    max: Frequency,
}

/// Serde wire format for [`FrequencyRange`]: forces deserialization through
/// the validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
struct FrequencyRangeRepr {
    min: Frequency,
    max: Frequency,
}

#[cfg(feature = "serde")]
impl From<FrequencyRange> for FrequencyRangeRepr {
    fn from(range: FrequencyRange) -> Self {
        Self {
            min: range.min,
            max: range.max,
        }
    }
}

#[cfg(feature = "serde")]
impl TryFrom<FrequencyRangeRepr> for FrequencyRange {
    type Error = FrequencyRangeError;

    fn try_from(repr: FrequencyRangeRepr) -> core::result::Result<Self, Self::Error> {
        FrequencyRange::new(repr.min, repr.max)
    }
}

impl FrequencyRange {
    /// Creates a frequency range from inclusive bounds.
    ///
    /// Both bounds must be finite and positive, with `min < max`.
    pub fn new(min: Frequency, max: Frequency) -> core::result::Result<Self, FrequencyRangeError> {
        let (min_hz, max_hz) = (min.to_hertz(), max.to_hertz());
        if !min_hz.is_finite() || !max_hz.is_finite() || min_hz <= 0.0 || max_hz <= 0.0 {
            return Err(FrequencyRangeError::InvalidBounds { min_hz, max_hz });
        }
        if min_hz >= max_hz {
            return Err(FrequencyRangeError::EmptyRange { min_hz, max_hz });
        }
        Ok(Self { min, max })
    }

    /// Creates a frequency range from inclusive wavelength bounds.
    ///
    /// The shorter wavelength maps to the upper frequency bound and vice
    /// versa. Both wavelengths must be finite and positive, with
    /// `min_wavelength < max_wavelength`.
    pub fn from_wavelengths(
        min_wavelength: Distance,
        max_wavelength: Distance,
    ) -> core::result::Result<Self, FrequencyRangeError> {
        let (min_m, max_m) = (min_wavelength.to_meters(), max_wavelength.to_meters());
        if !min_m.is_finite()
            || !max_m.is_finite()
            || min_m <= 0.0
            || max_m <= 0.0
            || min_m >= max_m
        {
            return Err(FrequencyRangeError::InvalidWavelengths { min_m, max_m });
        }
        Self::new(
            Frequency::hertz(SPEED_OF_LIGHT / max_m),
            Frequency::hertz(SPEED_OF_LIGHT / min_m),
        )
    }

    /// Returns the lower frequency bound.
    pub fn min(&self) -> Frequency {
        self.min
    }

    /// Returns the upper frequency bound.
    pub fn max(&self) -> Frequency {
        self.max
    }

    /// Returns whether the frequency lies within the range (bounds inclusive).
    pub fn contains(&self, frequency: Frequency) -> bool {
        let f = frequency.to_hertz();
        self.min.to_hertz() <= f && f <= self.max.to_hertz()
    }

    /// Returns the overlap of two ranges, or `None` when they are disjoint.
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let min = if self.min.to_hertz() >= other.min.to_hertz() {
            self.min
        } else {
            other.min
        };
        let max = if self.max.to_hertz() <= other.max.to_hertz() {
            self.max
        } else {
            other.max
        };
        Self::new(min, max).ok()
    }
}

impl From<FrequencyBand> for FrequencyRange {
    /// Converts an IEEE letter band into its explicit frequency range.
    fn from(band: FrequencyBand) -> Self {
        let (min, max) = band.bounds();
        Self { min, max }
    }
}

/// Above this lower bound a range is displayed in wavelength (nm) rather than
/// frequency — optical bands are conventionally quoted in nanometers.
const OPTICAL_DISPLAY_THRESHOLD_HZ: f64 = 10e12;

impl Display for FrequencyRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.min.to_hertz() >= OPTICAL_DISPLAY_THRESHOLD_HZ {
            // Shorter wavelength corresponds to the upper frequency bound.
            let min_nm = self.max.wavelength().to_meters() * 1e9;
            let max_nm = self.min.wavelength().to_meters() * 1e9;
            write!(f, "{min_nm:.1}–{max_nm:.1} nm")
        } else if self.max.to_hertz() >= 1e9 {
            write!(
                f,
                "{:.3}–{:.3} GHz",
                self.min.to_gigahertz(),
                self.max.to_gigahertz()
            )
        } else {
            write!(
                f,
                "{:.3}–{:.3} MHz",
                self.min.to_megahertz(),
                self.max.to_megahertz()
            )
        }
    }
}

/// Errors produced while constructing a [`FrequencyRange`].
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum FrequencyRangeError {
    /// A bound is non-finite or non-positive.
    #[error("frequency bounds must be finite and positive, got {min_hz} Hz and {max_hz} Hz")]
    InvalidBounds {
        /// Lower bound in Hz.
        min_hz: f64,
        /// Upper bound in Hz.
        max_hz: f64,
    },
    /// The lower bound is not below the upper bound.
    #[error("frequency range is empty: min {min_hz} Hz must be less than max {max_hz} Hz")]
    EmptyRange {
        /// Lower bound in Hz.
        min_hz: f64,
        /// Upper bound in Hz.
        max_hz: f64,
    },
    /// A wavelength bound is non-finite, non-positive, or inverted.
    #[error(
        "wavelength bounds must be finite, positive, and ascending, got {min_m} m and {max_m} m"
    )]
    InvalidWavelengths {
        /// Shorter wavelength in meters.
        min_m: f64,
        /// Longer wavelength in meters.
        max_m: f64,
    },
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use lox_test_utils::assert_approx_eq;
    use rstest::rstest;

    use crate::units::FrequencyUnits;

    use super::*;

    fn ka_uplink() -> FrequencyRange {
        FrequencyRange::new(27.5.ghz(), 31.0.ghz()).unwrap()
    }

    #[test]
    fn test_frequency_range_accessors() {
        let range = ka_uplink();
        assert_approx_eq!(range.min().to_gigahertz(), 27.5, rtol <= 1e-12);
        assert_approx_eq!(range.max().to_gigahertz(), 31.0, rtol <= 1e-12);
    }

    #[test]
    fn test_frequency_range_contains_is_inclusive() {
        let range = ka_uplink();
        assert!(range.contains(29.0.ghz()));
        assert!(range.contains(27.5.ghz()));
        assert!(range.contains(31.0.ghz()));
        assert!(!range.contains(27.499.ghz()));
        assert!(!range.contains(31.001.ghz()));
    }

    #[test]
    fn test_frequency_range_intersect_partial_overlap() {
        let a = FrequencyRange::new(27.0.ghz(), 30.0.ghz()).unwrap();
        let b = FrequencyRange::new(29.0.ghz(), 31.0.ghz()).unwrap();
        let overlap = a.intersect(&b).unwrap();
        assert_approx_eq!(overlap.min().to_gigahertz(), 29.0, rtol <= 1e-12);
        assert_approx_eq!(overlap.max().to_gigahertz(), 30.0, rtol <= 1e-12);
        // Intersection is symmetric
        assert_eq!(overlap, b.intersect(&a).unwrap());
    }

    #[test]
    fn test_frequency_range_intersect_containment() {
        let outer = FrequencyRange::new(20.0.ghz(), 40.0.ghz()).unwrap();
        let inner = ka_uplink();
        assert_eq!(outer.intersect(&inner).unwrap(), inner);
    }

    #[test]
    fn test_frequency_range_intersect_disjoint_is_none() {
        let a = FrequencyRange::new(2.0.ghz(), 4.0.ghz()).unwrap();
        let b = FrequencyRange::new(27.0.ghz(), 31.0.ghz()).unwrap();
        assert!(a.intersect(&b).is_none());
        // Touching at a single point is empty (min == max is not a range)
        let c = FrequencyRange::new(4.0.ghz(), 8.0.ghz()).unwrap();
        assert!(a.intersect(&c).is_none());
    }

    #[test]
    fn test_frequency_range_rejects_inverted_and_invalid_bounds() {
        assert!(matches!(
            FrequencyRange::new(31.0.ghz(), 27.5.ghz()),
            Err(FrequencyRangeError::EmptyRange { .. })
        ));
        assert!(matches!(
            FrequencyRange::new(29.0.ghz(), 29.0.ghz()),
            Err(FrequencyRangeError::EmptyRange { .. })
        ));
        assert!(matches!(
            FrequencyRange::new(Frequency::hertz(0.0), 29.0.ghz()),
            Err(FrequencyRangeError::InvalidBounds { .. })
        ));
        assert!(matches!(
            FrequencyRange::new(Frequency::hertz(f64::NAN), 29.0.ghz()),
            Err(FrequencyRangeError::InvalidBounds { .. })
        ));
    }

    #[test]
    fn test_frequency_range_from_wavelengths_optical_c_band() {
        // Optical C-band: 1530–1565 nm ≈ 191.6–195.9 THz
        let range =
            FrequencyRange::from_wavelengths(Distance::meters(1530e-9), Distance::meters(1565e-9))
                .unwrap();
        assert!(range.contains(Frequency::terahertz(193.4)));
        assert!(!range.contains(Frequency::terahertz(190.0)));
        assert_approx_eq!(range.min().to_terahertz(), 191.6, atol <= 0.1);
        assert_approx_eq!(range.max().to_terahertz(), 195.9, atol <= 0.1);
    }

    #[test]
    fn test_frequency_range_from_wavelengths_rejects_inverted() {
        assert!(matches!(
            FrequencyRange::from_wavelengths(Distance::meters(1565e-9), Distance::meters(1530e-9)),
            Err(FrequencyRangeError::InvalidWavelengths { .. })
        ));
    }

    #[test]
    fn test_frequency_range_from_letter_band() {
        let ka = FrequencyRange::from(FrequencyBand::Ka);
        assert!(ka.contains(29.0.ghz()));
        assert!(!ka.contains(45.0.ghz()));
        let uhf = FrequencyRange::from(FrequencyBand::UHF);
        assert!(uhf.contains(Frequency::megahertz(435.0)));
    }

    #[test]
    fn test_frequency_band_bounds_round_trip() {
        // Every IEEE letter band converts to a valid range whose midpoint
        // classifies back to the same band.
        for band in FrequencyBand::ALL {
            let range = FrequencyRange::from(band);
            assert!(range.min().to_hertz() < range.max().to_hertz());
            let midpoint =
                Frequency::hertz((range.min().to_hertz() + range.max().to_hertz()) / 2.0);
            assert!(range.contains(midpoint));
            assert_eq!(midpoint.band(), Some(band));
        }
    }

    #[test]
    fn test_frequency_band_name_parse_round_trip() {
        use alloc::string::ToString;

        for band in FrequencyBand::ALL {
            assert_eq!(band.name().parse::<FrequencyBand>(), Ok(band));
            assert_eq!(band.to_string(), band.name());
        }
        // Parsing ignores ASCII case.
        assert_eq!("ka".parse::<FrequencyBand>(), Ok(FrequencyBand::Ka));
        assert_eq!("KU".parse::<FrequencyBand>(), Ok(FrequencyBand::Ku));
        assert_eq!("Q".parse::<FrequencyBand>(), Err(ParseFrequencyBandError));
    }

    #[test]
    fn test_frequency_range_error_displays() {
        use alloc::string::ToString;

        let err = FrequencyRange::new(Frequency::hertz(-1.0), 29.0.ghz()).unwrap_err();
        assert!(err.to_string().contains("finite and positive"));
        let err = FrequencyRange::new(31.0.ghz(), 27.0.ghz()).unwrap_err();
        assert!(err.to_string().contains("empty"));
        let err =
            FrequencyRange::from_wavelengths(Distance::meters(-1.0), Distance::meters(1565e-9))
                .unwrap_err();
        assert!(err.to_string().contains("wavelength"));
    }

    #[test]
    fn test_frequency_range_display_rf_and_optical() {
        use alloc::string::ToString;

        assert_eq!(ka_uplink().to_string(), "27.500–31.000 GHz");
        let uhf =
            FrequencyRange::new(Frequency::megahertz(400.0), Frequency::megahertz(450.0)).unwrap();
        assert_eq!(uhf.to_string(), "400.000–450.000 MHz");
        let optical =
            FrequencyRange::from_wavelengths(Distance::meters(1530e-9), Distance::meters(1565e-9))
                .unwrap();
        assert_eq!(optical.to_string(), "1530.0–1565.0 nm");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_frequency_range_serde_round_trip_and_validation() {
        let range = ka_uplink();
        let json = serde_json::to_string(&range).unwrap();
        let round_trip: FrequencyRange = serde_json::from_str(&json).unwrap();
        assert_eq!(range, round_trip);

        // Inverted bounds must be rejected at deserialization time.
        let bad = r#"{"min":31.0e9,"max":27.5e9}"#;
        assert!(serde_json::from_str::<FrequencyRange>(bad).is_err());
    }

    #[rstest]
    #[case(0.0.hz(), None)]
    #[case(3.0.mhz(), Some(FrequencyBand::HF))]
    #[case(30.0.mhz(), Some(FrequencyBand::VHF))]
    #[case(300.0.mhz(), Some(FrequencyBand::UHF))]
    #[case(1.0.ghz(), Some(FrequencyBand::L))]
    #[case(2.0.ghz(), Some(FrequencyBand::S))]
    #[case(4.0.ghz(), Some(FrequencyBand::C))]
    #[case(8.0.ghz(), Some(FrequencyBand::X))]
    #[case(12.0.ghz(), Some(FrequencyBand::Ku))]
    #[case(18.0.ghz(), Some(FrequencyBand::K))]
    #[case(27.0.ghz(), Some(FrequencyBand::Ka))]
    #[case(40.0.ghz(), Some(FrequencyBand::V))]
    #[case(75.0.ghz(), Some(FrequencyBand::W))]
    #[case(110.0.ghz(), Some(FrequencyBand::G))]
    #[case(1.0.thz(), None)]
    fn test_frequency_band_classification(
        #[case] f: Frequency,
        #[case] exp: Option<FrequencyBand>,
    ) {
        assert_eq!(f.band(), exp)
    }
}
