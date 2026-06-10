// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Frequency ranges for hardware capability declarations.

use std::fmt;

use lox_core::units::{Distance, Frequency, FrequencyBand, SPEED_OF_LIGHT};
use thiserror::Error;

/// A contiguous frequency range with inclusive bounds.
///
/// Declares the capability of a radio, antenna, or port: link validation
/// checks that the carrier lies inside the intersection of all ranges on the
/// resolved chain. Unlike [`FrequencyBand`] — the IEEE letter-band
/// classification in `lox-core` — a `FrequencyRange` carries explicit bounds
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

    fn try_from(repr: FrequencyRangeRepr) -> Result<Self, Self::Error> {
        FrequencyRange::new(repr.min, repr.max)
    }
}

impl FrequencyRange {
    /// Creates a frequency range from inclusive bounds.
    ///
    /// Both bounds must be finite and positive, with `min < max`.
    pub fn new(min: Frequency, max: Frequency) -> Result<Self, FrequencyRangeError> {
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
    ) -> Result<Self, FrequencyRangeError> {
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
        let (min_hz, max_hz) = match band {
            FrequencyBand::HF => (3e6, 30e6),
            FrequencyBand::VHF => (30e6, 300e6),
            FrequencyBand::UHF => (300e6, 1e9),
            FrequencyBand::L => (1e9, 2e9),
            FrequencyBand::S => (2e9, 4e9),
            FrequencyBand::C => (4e9, 8e9),
            FrequencyBand::X => (8e9, 12e9),
            FrequencyBand::Ku => (12e9, 18e9),
            FrequencyBand::K => (18e9, 27e9),
            FrequencyBand::Ka => (27e9, 40e9),
            FrequencyBand::V => (40e9, 75e9),
            FrequencyBand::W => (75e9, 110e9),
            FrequencyBand::G => (110e9, 300e9),
        };
        Self {
            min: Frequency::hertz(min_hz),
            max: Frequency::hertz(max_hz),
        }
    }
}

/// Above this lower bound a range is displayed in wavelength (nm) rather than
/// frequency — optical bands are conventionally quoted in nanometers.
const OPTICAL_DISPLAY_THRESHOLD_HZ: f64 = 10e12;

impl fmt::Display for FrequencyRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
#[derive(Debug, Clone, PartialEq, Error)]
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
    use lox_core::units::FrequencyUnits;
    use lox_test_utils::assert_approx_eq;

    use super::*;

    fn ka_uplink() -> FrequencyRange {
        FrequencyRange::new(27.5.ghz(), 31.0.ghz()).unwrap()
    }

    #[test]
    fn test_range_accessors() {
        let range = ka_uplink();
        assert_approx_eq!(range.min().to_gigahertz(), 27.5, rtol <= 1e-12);
        assert_approx_eq!(range.max().to_gigahertz(), 31.0, rtol <= 1e-12);
    }

    #[test]
    fn test_contains_is_inclusive() {
        let range = ka_uplink();
        assert!(range.contains(29.0.ghz()));
        assert!(range.contains(27.5.ghz()));
        assert!(range.contains(31.0.ghz()));
        assert!(!range.contains(27.499.ghz()));
        assert!(!range.contains(31.001.ghz()));
    }

    #[test]
    fn test_intersect_partial_overlap() {
        let a = FrequencyRange::new(27.0.ghz(), 30.0.ghz()).unwrap();
        let b = FrequencyRange::new(29.0.ghz(), 31.0.ghz()).unwrap();
        let overlap = a.intersect(&b).unwrap();
        assert_approx_eq!(overlap.min().to_gigahertz(), 29.0, rtol <= 1e-12);
        assert_approx_eq!(overlap.max().to_gigahertz(), 30.0, rtol <= 1e-12);
        // Intersection is symmetric
        assert_eq!(overlap, b.intersect(&a).unwrap());
    }

    #[test]
    fn test_intersect_containment() {
        let outer = FrequencyRange::new(20.0.ghz(), 40.0.ghz()).unwrap();
        let inner = ka_uplink();
        assert_eq!(outer.intersect(&inner).unwrap(), inner);
    }

    #[test]
    fn test_intersect_disjoint_is_none() {
        let a = FrequencyRange::new(2.0.ghz(), 4.0.ghz()).unwrap();
        let b = FrequencyRange::new(27.0.ghz(), 31.0.ghz()).unwrap();
        assert!(a.intersect(&b).is_none());
        // Touching at a single point is empty (min == max is not a range)
        let c = FrequencyRange::new(4.0.ghz(), 8.0.ghz()).unwrap();
        assert!(a.intersect(&c).is_none());
    }

    #[test]
    fn test_new_rejects_inverted_and_invalid_bounds() {
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
    fn test_from_wavelengths_optical_c_band() {
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
    fn test_from_wavelengths_rejects_inverted() {
        assert!(matches!(
            FrequencyRange::from_wavelengths(Distance::meters(1565e-9), Distance::meters(1530e-9)),
            Err(FrequencyRangeError::InvalidWavelengths { .. })
        ));
    }

    #[test]
    fn test_from_letter_band() {
        let ka = FrequencyRange::from(FrequencyBand::Ka);
        assert!(ka.contains(29.0.ghz()));
        assert!(!ka.contains(45.0.ghz()));
        let uhf = FrequencyRange::from(FrequencyBand::UHF);
        assert!(uhf.contains(Frequency::megahertz(435.0)));
    }

    #[test]
    fn test_display_rf_and_optical() {
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
    fn test_serde_round_trip_and_validation() {
        let range = ka_uplink();
        let json = serde_json::to_string(&range).unwrap();
        let round_trip: FrequencyRange = serde_json::from_str(&json).unwrap();
        assert_eq!(range, round_trip);

        // Inverted bounds must be rejected at deserialization time.
        let bad = r#"{"min":31.0e9,"max":27.5e9}"#;
        assert!(serde_json::from_str::<FrequencyRange>(bad).is_err());
    }
}
