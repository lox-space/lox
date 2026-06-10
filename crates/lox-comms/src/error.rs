// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Errors produced by link-budget calculations.

use lox_core::units::Frequency;
use thiserror::Error;

use crate::antenna::AntennaFrameError;
use crate::band::FrequencyRange;

/// Errors that can arise when computing a link budget.
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum LinkBudgetError {
    /// Absolute carrier and noise powers are required but are unavailable.
    #[error("absolute carrier and noise powers are unavailable for this link")]
    AbsolutePowerUnavailable,
    /// A line-of-sight direction could not be converted to pattern angles.
    #[error("invalid pointing: {0}")]
    InvalidPointing(#[from] AntennaFrameError),
    /// The carrier frequency lies outside an endpoint's supported range.
    #[error("carrier {} Hz outside the supported range {band} of endpoint '{endpoint}'", carrier.to_hertz())]
    CarrierOutOfBand {
        /// The requested carrier frequency.
        carrier: Frequency,
        /// The endpoint's supported frequency range.
        band: FrequencyRange,
        /// Name of the terminal whose endpoint rejected the carrier.
        endpoint: String,
    },
}

#[cfg(test)]
mod tests {
    use lox_core::units::FrequencyUnits;

    use crate::band::FrequencyRange;

    use super::*;

    #[test]
    fn test_display_carrier_out_of_band() {
        let err = LinkBudgetError::CarrierOutOfBand {
            carrier: 29.0.ghz(),
            band: FrequencyRange::new(17.0.ghz(), 21.0.ghz()).unwrap(),
            endpoint: "rx".to_owned(),
        };
        assert_eq!(
            err.to_string(),
            "carrier 29000000000 Hz outside the supported range 17.000–21.000 GHz of endpoint 'rx'"
        );
    }

    #[test]
    fn test_display_absolute_power_unavailable() {
        let s = LinkBudgetError::AbsolutePowerUnavailable.to_string();
        assert!(s.contains("absolute carrier and noise powers"));
    }

    #[test]
    fn test_is_error() {
        fn assert_is_error<E: std::error::Error>(_: &E) {}
        assert_is_error(&LinkBudgetError::AbsolutePowerUnavailable);
    }
}
