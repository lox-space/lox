// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Wire-format detection and dispatch for ODM messages.

use crate::error::OdmError;

/// The three ODM wire formats this crate supports.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Format {
    /// CCSDS KVN (Keyword-Value Notation).
    Kvn,
    /// CCSDS XML.
    Xml,
    /// Space-Track / Celestrak JSON. **OMM only** in CCSDS 502.0-B-3.
    Json,
}

/// Sniff the first non-whitespace byte of `input` and return the
/// inferred format.
///
/// - `<` → [`Format::Xml`]
/// - `{` or `[` → [`Format::Json`]
/// - Anything else → [`Format::Kvn`] (KVN files start with either
///   `CCSDS_<KIND>_VERS = …` or one or more `COMMENT …` preamble lines)
///
/// Returns [`OdmError::UndetectableFormat`] for empty input.
pub fn detect_format(input: &str) -> Result<Format, OdmError> {
    let trimmed = input.trim_start();
    match trimmed.bytes().next() {
        Some(b'<') => Ok(Format::Xml),
        Some(b'{') | Some(b'[') => Ok(Format::Json),
        Some(_) => Ok(Format::Kvn),
        None => Err(OdmError::UndetectableFormat),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_kvn_via_version_header() {
        assert_eq!(
            detect_format("CCSDS_OPM_VERS = 3.0\n").unwrap(),
            Format::Kvn
        );
    }

    #[test]
    fn detect_kvn_via_comment_preamble() {
        assert_eq!(
            detect_format("COMMENT banner\nCCSDS_OPM_VERS = 3.0\n").unwrap(),
            Format::Kvn
        );
    }

    #[test]
    fn detect_xml_via_prolog() {
        assert_eq!(
            detect_format("<?xml version=\"1.0\"?>\n<opm/>").unwrap(),
            Format::Xml
        );
    }

    #[test]
    fn detect_xml_via_bare_root() {
        assert_eq!(detect_format("<opm/>").unwrap(), Format::Xml);
    }

    #[test]
    fn detect_json_object() {
        assert_eq!(
            detect_format("{\"OBJECT_NAME\":\"X\"}").unwrap(),
            Format::Json
        );
    }

    #[test]
    fn detect_json_array() {
        assert_eq!(
            detect_format("[{\"OBJECT_NAME\":\"X\"}]").unwrap(),
            Format::Json
        );
    }

    #[test]
    fn detect_skips_leading_whitespace() {
        assert_eq!(detect_format("   \n  <opm/>").unwrap(), Format::Xml);
    }

    #[test]
    fn detect_errors_on_empty_input() {
        assert!(matches!(
            detect_format("").unwrap_err(),
            OdmError::UndetectableFormat
        ));
        assert!(matches!(
            detect_format("   \n  ").unwrap_err(),
            OdmError::UndetectableFormat
        ));
    }
}
