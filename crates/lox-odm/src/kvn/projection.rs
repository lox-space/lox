// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! KVN AST ↔ typed-message projection.
//!
//! Each ODM message kind has its own projection sub-module (added by
//! Phase 2b-{opm,oem,omm,ci} tasks). Shared helpers live at this
//! top-level — extracting scalar fields by keyword, parsing `f64`
//! values, decoding epoch fields against the message's `TIME_SYSTEM`.
//! The helpers operate against AST primitives
//! ([`KvnField`], [`KvnEntry`]) and emit [`KvnError`] with the original
//! wire keyword as context when something fails.

use crate::kvn::ast::{KvnEntry, KvnField};
use crate::kvn::error::{KvnError, KvnErrorKind, Span};
use crate::types::common::OdmTime;

pub mod oem;
pub mod opm;

/// Looks up the *single* field with the given keyword among a slice of
/// entries. Errors with [`KvnErrorKind::DuplicateField`] if more than
/// one such field is present. Returns `Ok(None)` if absent.
pub(crate) fn find_field<'a>(
    entries: &'a [KvnEntry],
    keyword: &str,
) -> Result<Option<&'a KvnField>, KvnError> {
    let mut found: Option<&KvnField> = None;
    for entry in entries {
        if let KvnEntry::Field(f) = entry
            && f.key == keyword
        {
            if found.is_some() {
                return Err(KvnError {
                    span: Span::default(),
                    kind: KvnErrorKind::DuplicateField(keyword.to_string()),
                });
            }
            found = Some(f);
        }
    }
    Ok(found)
}

/// Like [`find_field`] but errors with [`KvnErrorKind::MissingRequiredField`]
/// when the keyword is absent.
pub(crate) fn require_field<'a>(
    entries: &'a [KvnEntry],
    keyword: &str,
) -> Result<&'a KvnField, KvnError> {
    find_field(entries, keyword)?.ok_or_else(|| KvnError {
        span: Span::default(),
        kind: KvnErrorKind::MissingRequiredField(keyword.to_string()),
    })
}

/// Parses the value text of a field as `f64`. Wraps any
/// [`f64::from_str`] error in an [`KvnErrorKind::InvalidValue`] with
/// the wire keyword.
pub(crate) fn parse_f64(field: &KvnField) -> Result<f64, KvnError> {
    field.value.trim().parse::<f64>().map_err(|e| KvnError {
        span: Span::default(),
        kind: KvnErrorKind::InvalidValue {
            keyword: field.key.clone(),
            reason: e.to_string(),
        },
    })
}

/// Parses an epoch field against the file's `TIME_SYSTEM`. Returns an
/// [`OdmTime`] preserving the wire choice of scale.
pub(crate) fn parse_epoch(field: &KvnField, time_system: &str) -> Result<OdmTime, KvnError> {
    OdmTime::from_wire(time_system, field.value.trim()).map_err(|e| KvnError {
        span: Span::default(),
        kind: KvnErrorKind::InvalidEpoch {
            value: field.value.clone(),
            time_system: time_system.to_string(),
            reason: e.to_string(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kvn::ast::{KvnEntry, KvnField};

    fn field(key: &str, value: &str) -> KvnEntry {
        KvnEntry::Field(KvnField {
            key: key.to_string(),
            value: value.to_string(),
            unit: None,
        })
    }

    #[test]
    fn find_field_returns_none_when_absent() {
        let entries = vec![field("A", "1")];
        assert!(find_field(&entries, "B").unwrap().is_none());
    }

    #[test]
    fn find_field_returns_field_when_present() {
        let entries = vec![field("A", "1")];
        assert_eq!(find_field(&entries, "A").unwrap().unwrap().value, "1");
    }

    #[test]
    fn find_field_errors_on_duplicate() {
        let entries = vec![field("A", "1"), field("A", "2")];
        let err = find_field(&entries, "A").unwrap_err();
        assert!(matches!(err.kind, KvnErrorKind::DuplicateField(ref k) if k == "A"));
    }

    #[test]
    fn require_field_errors_when_absent() {
        let entries: Vec<KvnEntry> = vec![];
        let err = require_field(&entries, "A").unwrap_err();
        assert!(matches!(err.kind, KvnErrorKind::MissingRequiredField(ref k) if k == "A"));
    }

    #[test]
    fn parse_f64_strips_whitespace_and_parses() {
        let f = KvnField {
            key: "X".to_string(),
            value: " 7000.5 ".to_string(),
            unit: Some("km".to_string()),
        };
        assert_eq!(parse_f64(&f).unwrap(), 7000.5);
    }

    #[test]
    fn parse_f64_errors_on_garbage() {
        let f = KvnField {
            key: "X".to_string(),
            value: "not-a-number".to_string(),
            unit: None,
        };
        let err = parse_f64(&f).unwrap_err();
        assert!(matches!(err.kind, KvnErrorKind::InvalidValue { keyword: ref k, .. } if k == "X"));
    }

    #[test]
    fn parse_epoch_succeeds_under_tai() {
        let f = KvnField {
            key: "EPOCH".to_string(),
            value: "2024-01-01T00:00:00".to_string(),
            unit: None,
        };
        let t = parse_epoch(&f, "TAI").unwrap();
        assert!(matches!(t, OdmTime::Time(_)));
    }

    #[test]
    fn parse_epoch_errors_on_bad_value() {
        let f = KvnField {
            key: "EPOCH".to_string(),
            value: "not-a-date".to_string(),
            unit: None,
        };
        let err = parse_epoch(&f, "TAI").unwrap_err();
        assert!(
            matches!(err.kind, KvnErrorKind::InvalidEpoch { ref value, .. } if value == "not-a-date")
        );
    }
}
