// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Errors produced by link-budget calculations.

use lox_core::units::Frequency;
use thiserror::Error;

/// Errors that can arise when computing a link budget.
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum LinkBudgetError {
    /// The transmitting system has no transmitter configured.
    #[error("transmitting system has no transmitter configured")]
    MissingTransmitter,
    /// The receiving system has no receiver configured.
    #[error("receiving system has no receiver configured")]
    MissingReceiver,
    /// A component-tier transmitter/receiver requires an antenna but none was provided.
    #[error("component-tier transmitter/receiver requires an antenna")]
    MissingAntenna,
    /// A lumped (`Eirp`/`Gt`) transmitter/receiver must not be paired with an antenna.
    #[error("lumped (Eirp/Gt) transmitter/receiver must not be paired with an antenna")]
    UnexpectedAntenna,
    /// Absolute carrier and noise powers are required but are unavailable.
    #[error("absolute carrier and noise powers are unavailable for this link")]
    AbsolutePowerUnavailable,
    /// Transmitter and receiver frequencies disagree.
    #[error(
        "transmitter frequency {} Hz differs from receiver frequency {} Hz",
        tx.to_hertz(),
        rx.to_hertz()
    )]
    FrequencyMismatch {
        /// Transmitter frequency.
        tx: Frequency,
        /// Receiver frequency.
        rx: Frequency,
    },
}

#[cfg(test)]
mod tests {
    use lox_core::units::FrequencyUnits;

    use super::*;

    #[test]
    fn test_display_missing_transmitter() {
        let s = LinkBudgetError::MissingTransmitter.to_string();
        assert!(s.contains("transmitter"));
    }

    #[test]
    fn test_display_frequency_mismatch() {
        let err = LinkBudgetError::FrequencyMismatch {
            tx: 29.0.ghz(),
            rx: 30.0.ghz(),
        };
        assert_eq!(
            err.to_string(),
            "transmitter frequency 29000000000 Hz differs from receiver frequency 30000000000 Hz"
        );
    }

    #[test]
    fn test_display_missing_receiver() {
        let s = LinkBudgetError::MissingReceiver.to_string();
        assert!(s.contains("receiver"));
    }

    #[test]
    fn test_display_missing_antenna() {
        let s = LinkBudgetError::MissingAntenna.to_string();
        assert!(s.contains("antenna"));
    }

    #[test]
    fn test_display_unexpected_antenna() {
        let s = LinkBudgetError::UnexpectedAntenna.to_string();
        assert!(s.contains("antenna"));
    }

    #[test]
    fn test_display_absolute_power_unavailable() {
        let s = LinkBudgetError::AbsolutePowerUnavailable.to_string();
        assert!(s.contains("absolute carrier and noise powers"));
    }

    #[test]
    fn test_is_error() {
        fn assert_is_error<E: std::error::Error>(_: &E) {}
        assert_is_error(&LinkBudgetError::MissingReceiver);
    }
}
