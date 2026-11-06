// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Itemized excess propagation losses.

use std::fmt::{Display, Formatter, Result};

use lox_core::units::Decibel;

/// The kind of an excess propagation loss, carrying its physical semantics.
///
/// Absorptive kinds attenuate the carrier *and* radiate thermally into the
/// receive antenna, raising its noise temperature; non-absorptive kinds only
/// affect the carrier. Custom losses declare their behaviour explicitly via
/// [`LossKind::Other`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LossKind {
    /// Rain attenuation (absorptive).
    Rain,
    /// Gaseous absorption (absorptive).
    Gaseous,
    /// Cloud attenuation (absorptive).
    Cloud,
    /// Tropospheric scintillation (non-absorptive).
    Scintillation,
    /// Depolarization loss (non-absorptive).
    Depolarization,
    /// A custom loss with an explicit absorptive flag.
    Other {
        /// Human-readable label, used as the budget line item.
        label: String,
        /// Whether the loss is absorptive (raises antenna noise temperature).
        absorptive: bool,
    },
}

impl LossKind {
    /// Returns whether this loss is absorptive, i.e. whether the lossy
    /// medium re-radiates thermally into the receive antenna.
    pub fn is_absorptive(&self) -> bool {
        match self {
            Self::Rain | Self::Gaseous | Self::Cloud => true,
            Self::Scintillation | Self::Depolarization => false,
            Self::Other { absorptive, .. } => *absorptive,
        }
    }

    /// Returns the budget line label, e.g. `"Rain attenuation"`.
    pub fn label(&self) -> &str {
        match self {
            Self::Rain => "Rain attenuation",
            Self::Gaseous => "Gaseous absorption",
            Self::Cloud => "Cloud attenuation",
            Self::Scintillation => "Scintillation",
            Self::Depolarization => "Depolarization",
            Self::Other { label, .. } => label,
        }
    }
}

impl Display for LossKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(self.label())
    }
}

/// One labelled propagation loss, valid by construction (finite, ≥ 0 dB).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "LossLineRepr")
)]
pub struct LossLine {
    kind: LossKind,
    value: Decibel,
}

/// Serde wire format for [`LossLine`]: forces deserialization through the
/// validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct LossLineRepr {
    kind: LossKind,
    value: Decibel,
}

#[cfg(feature = "serde")]
impl TryFrom<LossLineRepr> for LossLine {
    type Error = PropagationLossError;

    fn try_from(repr: LossLineRepr) -> core::result::Result<Self, Self::Error> {
        LossLine::new(repr.kind, repr.value)
    }
}

impl LossLine {
    /// Creates a loss line.
    ///
    /// Rejects non-finite and negative values — a gain is not a loss.
    pub fn new(kind: LossKind, value: Decibel) -> core::result::Result<Self, PropagationLossError> {
        let db = value.to_decibels();
        if !db.is_finite() || db < 0.0 {
            return Err(PropagationLossError::InvalidValue {
                label: String::from(kind.label()),
                value_db: db,
            });
        }
        Ok(Self { kind, value })
    }

    /// Returns the kind of this loss.
    pub fn kind(&self) -> &LossKind {
        &self.kind
    }

    /// Returns the loss value in dB.
    pub fn value(&self) -> Decibel {
        self.value
    }
}

/// Excess propagation losses along a link path, beyond free-space path loss.
///
/// An open, itemized collection: producers (e.g. the ITU-R models in
/// `lox-itur`) emit whatever lines describe the path, and consumers reduce
/// them to the figures they need — [`Self::total`] for carrier attenuation
/// and [`Self::absorptive`] for sky-noise degradation. Lines keep their
/// insertion order for reporting.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PropagationLosses {
    lines: Vec<LossLine>,
}

impl PropagationLosses {
    /// Returns zero propagation losses.
    pub fn none() -> Self {
        Self { lines: Vec::new() }
    }

    /// Creates propagation losses from a list of lines.
    pub fn new(lines: Vec<LossLine>) -> Self {
        Self { lines }
    }

    /// Starts building propagation losses line by line.
    pub fn builder() -> PropagationLossesBuilder {
        PropagationLossesBuilder {
            entries: Vec::new(),
        }
    }

    /// Returns the total excess loss in dB.
    pub fn total(&self) -> Decibel {
        Decibel::decibels(self.lines.iter().map(|line| line.value.to_decibels()).sum())
    }

    /// Returns the absorptive part of the loss in dB — the attenuation that
    /// also raises the receive antenna noise temperature.
    pub fn absorptive(&self) -> Decibel {
        Decibel::decibels(
            self.lines
                .iter()
                .filter(|line| line.kind.is_absorptive())
                .map(|line| line.value.to_decibels())
                .sum(),
        )
    }

    /// Returns the loss lines in insertion order.
    pub fn lines(&self) -> &[LossLine] {
        &self.lines
    }
}

/// Builder for [`PropagationLosses`].
///
/// Created via [`PropagationLosses::builder`]. Values are validated at
/// [`PropagationLossesBuilder::build`].
#[derive(Debug, Clone, Default)]
pub struct PropagationLossesBuilder {
    entries: Vec<(LossKind, Decibel)>,
}

impl PropagationLossesBuilder {
    /// Appends a loss line of the given kind.
    pub fn line(mut self, kind: LossKind, value: Decibel) -> Self {
        self.entries.push((kind, value));
        self
    }

    /// Appends a rain attenuation line.
    pub fn rain(self, value: Decibel) -> Self {
        self.line(LossKind::Rain, value)
    }

    /// Appends a gaseous absorption line.
    pub fn gaseous(self, value: Decibel) -> Self {
        self.line(LossKind::Gaseous, value)
    }

    /// Appends a cloud attenuation line.
    pub fn cloud(self, value: Decibel) -> Self {
        self.line(LossKind::Cloud, value)
    }

    /// Appends a scintillation line.
    pub fn scintillation(self, value: Decibel) -> Self {
        self.line(LossKind::Scintillation, value)
    }

    /// Appends a depolarization line.
    pub fn depolarization(self, value: Decibel) -> Self {
        self.line(LossKind::Depolarization, value)
    }

    /// Appends a custom loss line with an explicit absorptive flag.
    pub fn other(self, label: impl Into<String>, value: Decibel, absorptive: bool) -> Self {
        self.line(
            LossKind::Other {
                label: label.into(),
                absorptive,
            },
            value,
        )
    }

    /// Builds the propagation losses, validating all values.
    pub fn build(self) -> core::result::Result<PropagationLosses, PropagationLossError> {
        let lines = self
            .entries
            .into_iter()
            .map(|(kind, value)| LossLine::new(kind, value))
            .collect::<core::result::Result<Vec<_>, _>>()?;
        Ok(PropagationLosses::new(lines))
    }
}

/// Errors produced while constructing propagation losses.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum PropagationLossError {
    /// A loss value is non-finite or negative.
    #[error("{label} must be finite and non-negative, got {value_db} dB")]
    InvalidValue {
        /// Label of the offending line.
        label: String,
        /// Offending value in dB.
        value_db: f64,
    },
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;
    use lox_core::units::literals::DecibelLiterals;

    use super::*;

    fn p618_losses() -> PropagationLosses {
        PropagationLosses::builder()
            .rain(2.0.db())
            .gaseous(0.5.db())
            .cloud(0.2.db())
            .scintillation(0.3.db())
            .build()
            .unwrap()
    }

    #[test]
    fn test_propagation_losses_none_is_zero() {
        let losses = PropagationLosses::none();
        assert_approx_eq!(losses.total().to_decibels(), 0.0, atol <= 1e-15);
        assert_approx_eq!(losses.absorptive().to_decibels(), 0.0, atol <= 1e-15);
        assert!(losses.lines().is_empty());
    }

    #[test]
    fn test_propagation_losses_total_and_absorptive() {
        let losses = p618_losses();
        assert_approx_eq!(losses.total().to_decibels(), 3.0, atol <= 1e-12);
        // Scintillation is non-absorptive.
        assert_approx_eq!(losses.absorptive().to_decibels(), 2.7, atol <= 1e-12);
    }

    #[test]
    fn test_propagation_losses_other_respects_absorptive_flag() {
        let losses = PropagationLosses::builder()
            .other("Radome wetting", 0.5.db(), true)
            .other("Pointing margin", 0.4.db(), false)
            .build()
            .unwrap();
        assert_approx_eq!(losses.total().to_decibels(), 0.9, atol <= 1e-12);
        assert_approx_eq!(losses.absorptive().to_decibels(), 0.5, atol <= 1e-12);
        assert_eq!(losses.lines()[0].kind().label(), "Radome wetting");
    }

    #[test]
    fn test_propagation_losses_duplicate_kinds_sum() {
        let losses = PropagationLosses::builder()
            .rain(1.0.db())
            .rain(0.5.db())
            .build()
            .unwrap();
        assert_approx_eq!(losses.total().to_decibels(), 1.5, atol <= 1e-12);
        assert_eq!(losses.lines().len(), 2);
    }

    #[test]
    fn test_propagation_losses_preserve_insertion_order() {
        let losses = p618_losses();
        let kinds: Vec<_> = losses.lines().iter().map(LossLine::kind).collect();
        assert_eq!(
            kinds,
            [
                &LossKind::Rain,
                &LossKind::Gaseous,
                &LossKind::Cloud,
                &LossKind::Scintillation
            ]
        );
    }

    #[test]
    fn test_loss_line_rejects_negative_and_non_finite() {
        for value in [-0.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(LossLine::new(LossKind::Rain, Decibel::decibels(value)).is_err());
            assert!(
                PropagationLosses::builder()
                    .rain(Decibel::decibels(value))
                    .build()
                    .is_err()
            );
        }
        // Zero is a valid loss.
        assert!(LossLine::new(LossKind::Rain, Decibel::decibels(0.0)).is_ok());
    }

    #[test]
    fn test_loss_kind_labels() {
        assert_eq!(LossKind::Rain.to_string(), "Rain attenuation");
        assert_eq!(LossKind::Gaseous.label(), "Gaseous absorption");
        let err = LossLine::new(LossKind::Cloud, Decibel::decibels(-1.0)).unwrap_err();
        assert!(err.to_string().contains("Cloud attenuation"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_propagation_losses_serde_round_trip_and_validation() {
        let losses = p618_losses();
        let json = serde_json::to_string(&losses).unwrap();
        let round_trip: PropagationLosses = serde_json::from_str(&json).unwrap();
        assert_eq!(losses, round_trip);

        // Negative values must be rejected at deserialization time.
        let bad = r#"{"lines":[{"kind":"Rain","value":-2.0}]}"#;
        assert!(serde_json::from_str::<PropagationLosses>(bad).is_err());
    }
}
