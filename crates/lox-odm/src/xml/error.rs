// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Error type for the XML reader/writer.

/// Error returned by the XML readers.
#[derive(Debug, thiserror::Error)]
pub enum XmlError {
    /// Underlying `quick-xml` deserialisation failure.
    #[error(transparent)]
    Xml(#[from] quick_xml::DeError),
    /// A required wire field was absent or empty.
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
    /// `read_ci` received a root element name it doesn't dispatch.
    #[error("unsupported XML root element `<{0}>` for OdmCi")]
    UnsupportedCiRoot(String),
}
