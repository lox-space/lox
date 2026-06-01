// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Errors produced by link-budget calculations.

use lox_core::units::Frequency;

/// Errors that can arise when computing a link budget.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum LinkBudgetError {
    /// The transmitting system has no transmitter configured.
    MissingTransmitter,
    /// The receiving system has no receiver configured.
    MissingReceiver,
    /// A component-tier transmitter/receiver requires an antenna but none was provided.
    MissingAntenna,
    /// A lumped (`Eirp`/`Gt`) transmitter/receiver must not be paired with an antenna.
    UnexpectedAntenna,
    /// Absolute carrier and noise powers are required but are unavailable.
    AbsolutePowerUnavailable,
    /// Transmitter and receiver frequencies disagree.
    FrequencyMismatch {
        /// Transmitter frequency.
        tx: Frequency,
        /// Receiver frequency.
        rx: Frequency,
    },
}

impl std::fmt::Display for LinkBudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkBudgetError::MissingTransmitter => {
                write!(f, "transmitting system has no transmitter configured")
            }
            LinkBudgetError::MissingReceiver => {
                write!(f, "receiving system has no receiver configured")
            }
            LinkBudgetError::MissingAntenna => {
                write!(f, "component-tier transmitter/receiver requires an antenna")
            }
            LinkBudgetError::UnexpectedAntenna => write!(
                f,
                "lumped (Eirp/Gt) transmitter/receiver must not be paired with an antenna"
            ),
            LinkBudgetError::AbsolutePowerUnavailable => write!(
                f,
                "absolute carrier and noise powers are unavailable for this link"
            ),
            LinkBudgetError::FrequencyMismatch { tx, rx } => write!(
                f,
                "transmitter frequency {} Hz differs from receiver frequency {} Hz",
                tx.to_hertz(),
                rx.to_hertz()
            ),
        }
    }
}

impl std::error::Error for LinkBudgetError {}

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
