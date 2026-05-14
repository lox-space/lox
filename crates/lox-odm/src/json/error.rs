// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Error type for the JSON OMM reader/writer.

/// Error returned by the JSON OMM reader.
#[derive(Debug, thiserror::Error)]
pub enum JsonError {
    /// Underlying `serde_json` parse failure. Preserves the original
    /// line/column accessors via `source()`.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// A required wire keyword was absent or empty.
    #[error("required field `{0}` is missing")]
    MissingRequiredField(String),
    /// A wire value couldn't be projected into the typed form.
    #[error("invalid value for `{keyword}`: {reason}")]
    InvalidValue {
        /// Wire keyword whose value failed to project.
        keyword: String,
        /// Underlying conversion error message.
        reason: String,
    },
    /// An epoch field couldn't be parsed under the message's TIME_SYSTEM.
    #[error("invalid epoch `{value}` under TIME_SYSTEM `{time_system}`: {reason}")]
    InvalidEpoch {
        /// Raw value on the wire.
        value: String,
        /// TIME_SYSTEM keyword the value was parsed under.
        time_system: String,
        /// Underlying parse error.
        reason: String,
    },
}
