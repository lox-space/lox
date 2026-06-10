// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Errors produced by link-budget calculations.

use lox_core::units::Frequency;
use thiserror::Error;

use crate::antenna::AntennaFrameError;
use crate::band::FrequencyRange;

/// A physical quantity is outside its valid domain.
#[derive(Debug, Clone, PartialEq, Error)]
#[error("non-physical {quantity}: {value}")]
pub struct NonPhysicalError {
    /// Name of the offending quantity.
    pub quantity: &'static str,
    /// The rejected value.
    pub value: f64,
}

impl NonPhysicalError {
    /// Validates that a quantity is finite.
    pub(crate) fn check_finite(quantity: &'static str, value: f64) -> Result<(), Self> {
        if !value.is_finite() {
            return Err(Self { quantity, value });
        }
        Ok(())
    }

    /// Validates that a quantity is finite and strictly positive.
    pub(crate) fn check_positive(quantity: &'static str, value: f64) -> Result<(), Self> {
        if !value.is_finite() || value <= 0.0 {
            return Err(Self { quantity, value });
        }
        Ok(())
    }

    /// Validates that a quantity is finite and non-negative.
    pub(crate) fn check_non_negative(quantity: &'static str, value: f64) -> Result<(), Self> {
        if !value.is_finite() || value < 0.0 {
            return Err(Self { quantity, value });
        }
        Ok(())
    }

    /// Validates that a quantity lies in the half-open unit interval (0, 1].
    pub(crate) fn check_unit_interval(quantity: &'static str, value: f64) -> Result<(), Self> {
        if !(value > 0.0 && value <= 1.0) {
            return Err(Self { quantity, value });
        }
        Ok(())
    }
}

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
    /// A physical quantity is outside its valid domain.
    #[error(transparent)]
    NonPhysical(#[from] NonPhysicalError),
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
