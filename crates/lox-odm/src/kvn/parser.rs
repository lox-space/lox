// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Document composer for the KVN parser.
//!
//! Walks the classified-line stream produced by [`super::grammar`] and
//! builds a [`KvnDocument`]. Comments are first-class entries — they
//! preserve their wire-order position alongside fields and rows with
//! no special ownership bookkeeping.

use crate::kvn::ast::{KvnDocument, KvnEntry, KvnField, KvnRow, KvnSection};
use crate::kvn::error::{KvnError, KvnErrorKind, Span};
use crate::kvn::grammar::{LineClass, classify_line};
use crate::types::common::MessageKind;

/// Parse a KVN-formatted string into a [`KvnDocument`].
pub fn parse(input: &str) -> Result<KvnDocument, KvnError> {
    let mut lines = input
        .lines()
        .enumerate()
        .map(|(idx, content)| (idx + 1, content));

    // Step 1: find the version header line. Any COMMENT lines preceding
    // it are collected as the document's `preamble`.
    let mut preamble: Vec<String> = Vec::new();
    let (header_line_no, kind_token, version) = loop {
        let Some((line_no, content)) = lines.next() else {
            return Err(KvnError {
                span: Span {
                    line: 0,
                    col_start: 0,
                    col_end: 0,
                },
                kind: KvnErrorKind::EmptyInput,
            });
        };
        match classify_line(content) {
            Ok(None) => continue,
            Ok(Some(LineClass::Comment(text))) => {
                preamble.push(text.to_string());
            }
            Ok(Some(LineClass::VersionHeader {
                kind_token,
                version,
            })) => {
                break (line_no, kind_token.to_string(), version.to_string());
            }
            _ => {
                return Err(KvnError {
                    span: Span::whole_line(line_no, content.len()),
                    kind: KvnErrorKind::MissingVersionHeader,
                });
            }
        }
    };

    let message_kind = match kind_token.as_str() {
        "OPM" => MessageKind::Opm,
        "OEM" => MessageKind::Oem,
        "OMM" => MessageKind::Omm,
        "OCM" => MessageKind::Ocm,
        "NDM" => MessageKind::Ci,
        _ => {
            return Err(KvnError {
                span: Span::whole_line(header_line_no, 0),
                kind: KvnErrorKind::UnknownMessageKind(kind_token),
            });
        }
    };

    // Step 2: state-machine pass.
    let mut top_level: Vec<KvnSection> = Vec::new();
    let mut current = KvnSection {
        keyword: "HEADER".to_string(),
        bracketed: false,
        entries: Vec::new(),
    };
    let mut section_stack: Vec<KvnSection> = Vec::new();

    for (line_no, content) in lines {
        let cls = match classify_line(content) {
            Ok(None) => continue,
            Ok(Some(c)) => c,
            Err(_) => {
                return Err(KvnError {
                    span: Span::whole_line(line_no, content.len()),
                    kind: KvnErrorKind::MalformedLine(content.to_string()),
                });
            }
        };

        match cls {
            LineClass::VersionHeader { .. } => {
                return Err(KvnError {
                    span: Span::whole_line(line_no, content.len()),
                    kind: KvnErrorKind::MalformedLine(content.to_string()),
                });
            }
            LineClass::Comment(text) => {
                current.entries.push(KvnEntry::Comment(text.to_string()));
            }
            LineClass::SectionStart { keyword } => {
                section_stack.push(std::mem::replace(
                    &mut current,
                    KvnSection {
                        keyword: keyword.to_string(),
                        bracketed: true,
                        entries: Vec::new(),
                    },
                ));
            }
            LineClass::SectionStop { keyword } => {
                if !current.bracketed {
                    return Err(KvnError {
                        span: Span::whole_line(line_no, content.len()),
                        kind: KvnErrorKind::StraySectionStop(keyword.to_string()),
                    });
                }
                if current.keyword != keyword {
                    return Err(KvnError {
                        span: Span::whole_line(line_no, content.len()),
                        kind: KvnErrorKind::UnexpectedStop {
                            found: keyword.to_string(),
                            expected: current.keyword.clone(),
                        },
                    });
                }
                // Invariant: a bracketed `current` is always paired with a
                // parent section pushed onto `section_stack` at the matching
                // `_START`, so this `pop` cannot fail.
                let parent = section_stack
                    .pop()
                    .expect("bracketed section without parent on stack");
                let closed = std::mem::replace(&mut current, parent);
                if section_stack.is_empty() && current.keyword == "HEADER" {
                    // Top-level bracketed section closed; flush HEADER and
                    // open an implicit DATA section for subsequent entries.
                    let prev_header = std::mem::replace(
                        &mut current,
                        KvnSection {
                            keyword: "DATA".to_string(),
                            bracketed: false,
                            entries: Vec::new(),
                        },
                    );
                    push_if_nonempty(&mut top_level, prev_header);
                    top_level.push(closed);
                } else {
                    // Nested in another section (bracketed or implicit DATA).
                    current.entries.push(KvnEntry::Subsection(closed));
                }
            }
            LineClass::Field { key, value, unit } => {
                current.entries.push(KvnEntry::Field(KvnField {
                    key: key.to_string(),
                    value: value.to_string(),
                    unit: unit.map(|u| u.to_string()),
                }));
            }
            LineClass::PositionalRow(values) => {
                current.entries.push(KvnEntry::Row(KvnRow {
                    values: values.iter().map(|v| v.to_string()).collect(),
                }));
            }
        }
    }

    // EOF cleanup.
    if !section_stack.is_empty() {
        let unterminated = current.keyword.clone();
        return Err(KvnError {
            span: Span {
                line: 0,
                col_start: 0,
                col_end: 0,
            },
            kind: KvnErrorKind::UnterminatedSection(unterminated),
        });
    }
    push_if_nonempty(&mut top_level, current);

    Ok(KvnDocument {
        message_kind,
        version,
        preamble,
        sections: top_level,
    })
}

fn push_if_nonempty(sections: &mut Vec<KvnSection>, section: KvnSection) {
    if !section.entries.is_empty() {
        sections.push(section);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(key: &str, value: &str) -> KvnEntry {
        KvnEntry::Field(KvnField {
            key: key.to_string(),
            value: value.to_string(),
            unit: None,
        })
    }

    fn field_with_unit(key: &str, value: &str, unit: &str) -> KvnEntry {
        KvnEntry::Field(KvnField {
            key: key.to_string(),
            value: value.to_string(),
            unit: Some(unit.to_string()),
        })
    }

    fn comment(text: &str) -> KvnEntry {
        KvnEntry::Comment(text.to_string())
    }

    #[test]
    fn parses_version_header_only() {
        let input = "CCSDS_OPM_VERS = 3.0\n";
        let doc = parse(input).unwrap();
        assert_eq!(doc.message_kind, MessageKind::Opm);
        assert_eq!(doc.version, "3.0");
        assert!(doc.preamble.is_empty());
        assert!(doc.sections.is_empty());
    }

    #[test]
    fn captures_preamble_comments_before_version_header() {
        let input = "\
COMMENT Generated by ABC
COMMENT v1.2.3
CCSDS_OPM_VERS = 3.0
";
        let doc = parse(input).unwrap();
        assert_eq!(doc.preamble, vec!["Generated by ABC", "v1.2.3"]);
        assert!(doc.sections.is_empty());
    }

    #[test]
    fn rejects_empty_input() {
        let result = parse("");
        assert!(matches!(result.unwrap_err().kind, KvnErrorKind::EmptyInput));
    }

    #[test]
    fn rejects_missing_version_header() {
        let result = parse("CREATION_DATE = 2024-01-01T00:00:00\n");
        assert!(matches!(
            result.unwrap_err().kind,
            KvnErrorKind::MissingVersionHeader
        ));
    }

    #[test]
    fn rejects_unknown_message_kind() {
        let result = parse("CCSDS_XYZ_VERS = 1.0\n");
        let err = result.unwrap_err();
        assert!(matches!(err.kind, KvnErrorKind::UnknownMessageKind(_)));
    }

    #[test]
    fn parses_header_fields_into_header_section() {
        let input = "\
CCSDS_OPM_VERS = 3.0
CREATION_DATE = 2024-01-01T00:00:00
ORIGINATOR = TEST
";
        let doc = parse(input).unwrap();
        assert_eq!(doc.sections.len(), 1);
        let header = &doc.sections[0];
        assert_eq!(header.keyword, "HEADER");
        assert!(!header.bracketed);
        assert_eq!(header.entries.len(), 2);
    }

    #[test]
    fn parses_bracketed_section() {
        let input = "\
CCSDS_OPM_VERS = 3.0
META_START
OBJECT_NAME = ISS
META_STOP
";
        let doc = parse(input).unwrap();
        assert_eq!(doc.sections.len(), 1);
        let meta = &doc.sections[0];
        assert_eq!(meta.keyword, "META");
        assert!(meta.bracketed);
        assert_eq!(meta.entries.len(), 1);
    }

    #[test]
    fn parses_field_with_unit() {
        let input = "\
CCSDS_OPM_VERS = 3.0
X = 7000.0 [km]
";
        let doc = parse(input).unwrap();
        let header = &doc.sections[0];
        let KvnEntry::Field(field) = &header.entries[0] else {
            panic!("expected Field");
        };
        assert_eq!(field.key, "X");
        assert_eq!(field.value, "7000.0");
        assert_eq!(field.unit.as_deref(), Some("km"));
    }

    #[test]
    fn parses_comment_into_section_entries_in_order() {
        let input = "\
CCSDS_OPM_VERS = 3.0
META_START
COMMENT Reference orbit
OBJECT_NAME = ISS
COMMENT additional name
META_STOP
";
        let doc = parse(input).unwrap();
        let meta = doc.sections.iter().find(|s| s.keyword == "META").unwrap();
        assert_eq!(meta.entries.len(), 3);
        assert!(matches!(&meta.entries[0], KvnEntry::Comment(c) if c == "Reference orbit"));
        assert!(matches!(&meta.entries[1], KvnEntry::Field(f) if f.key == "OBJECT_NAME"));
        assert!(matches!(&meta.entries[2], KvnEntry::Comment(c) if c == "additional name"));
    }

    #[test]
    fn rejects_mismatched_stop() {
        let input = "\
CCSDS_OPM_VERS = 3.0
META_START
OBJECT_NAME = ISS
COVARIANCE_STOP
";
        let err = parse(input).unwrap_err();
        assert!(matches!(err.kind, KvnErrorKind::UnexpectedStop { .. }));
    }

    #[test]
    fn rejects_stray_stop_against_implicit_header() {
        let input = "\
CCSDS_OPM_VERS = 3.0
HEADER_STOP
";
        let err = parse(input).unwrap_err();
        assert!(
            matches!(&err.kind, KvnErrorKind::StraySectionStop(k) if k == "HEADER"),
            "expected StraySectionStop(HEADER), got {:?}",
            err.kind
        );
    }

    #[test]
    fn rejects_stray_stop_against_implicit_data() {
        let input = "\
CCSDS_OPM_VERS = 3.0
META_START
OBJECT_NAME = ISS
META_STOP
DATA_STOP
";
        let err = parse(input).unwrap_err();
        assert!(
            matches!(&err.kind, KvnErrorKind::StraySectionStop(k) if k == "DATA"),
            "expected StraySectionStop(DATA), got {:?}",
            err.kind
        );
    }

    #[test]
    fn rejects_unterminated_section() {
        let input = "\
CCSDS_OPM_VERS = 3.0
META_START
OBJECT_NAME = ISS
";
        let err = parse(input).unwrap_err();
        assert!(matches!(err.kind, KvnErrorKind::UnterminatedSection(_)));
    }

    // Round-trip property tests: validate parse(write(doc)) == doc.

    fn round_trip(doc: &KvnDocument) {
        let written = format!("{doc}");
        let reparsed = parse(&written).expect("write→parse round trip");
        assert_eq!(
            &reparsed, doc,
            "round trip mismatch.\noriginal:\n{:#?}\nwritten:\n{}\nreparsed:\n{:#?}",
            doc, written, reparsed
        );
    }

    fn opm_doc(sections: Vec<KvnSection>) -> KvnDocument {
        KvnDocument {
            message_kind: MessageKind::Opm,
            version: "3.0".to_string(),
            preamble: Vec::new(),
            sections,
        }
    }

    #[test]
    fn round_trip_empty_opm() {
        round_trip(&opm_doc(Vec::new()));
    }

    #[test]
    fn round_trip_preamble_comments() {
        let doc = KvnDocument {
            message_kind: MessageKind::Opm,
            version: "3.0".to_string(),
            preamble: vec!["banner".to_string(), "more banner".to_string()],
            sections: Vec::new(),
        };
        round_trip(&doc);
    }

    #[test]
    fn round_trip_header_and_metadata() {
        let doc = opm_doc(vec![
            KvnSection {
                keyword: "HEADER".to_string(),
                bracketed: false,
                entries: vec![
                    field("CREATION_DATE", "2024-01-01T00:00:00"),
                    field("ORIGINATOR", "TEST"),
                ],
            },
            KvnSection {
                keyword: "META".to_string(),
                bracketed: true,
                entries: vec![field("OBJECT_NAME", "ISS")],
            },
        ]);
        round_trip(&doc);
    }

    #[test]
    fn round_trip_field_with_unit() {
        let doc = opm_doc(vec![KvnSection {
            keyword: "HEADER".to_string(),
            bracketed: false,
            entries: vec![field_with_unit("X", "7000.0", "km")],
        }]);
        round_trip(&doc);
    }

    #[test]
    fn round_trip_comments_interleaved_with_fields() {
        let doc = opm_doc(vec![KvnSection {
            keyword: "META".to_string(),
            bracketed: true,
            entries: vec![
                comment("Reference orbit"),
                field("OBJECT_NAME", "ISS"),
                comment("primary designator"),
            ],
        }]);
        round_trip(&doc);
    }

    #[test]
    fn round_trip_oem_ephemeris_rows() {
        let doc = KvnDocument {
            message_kind: MessageKind::Oem,
            version: "3.0".to_string(),
            preamble: Vec::new(),
            sections: vec![
                KvnSection {
                    keyword: "META".to_string(),
                    bracketed: true,
                    entries: vec![field("OBJECT_NAME", "TEST")],
                },
                KvnSection {
                    keyword: "DATA".to_string(),
                    bracketed: false,
                    entries: vec![
                        KvnEntry::Row(KvnRow {
                            values: vec![
                                "2024-01-01T00:00:00".to_string(),
                                "7000.0".to_string(),
                                "0.0".to_string(),
                                "0.0".to_string(),
                                "0.0".to_string(),
                                "7.5".to_string(),
                                "0.0".to_string(),
                            ],
                        }),
                        KvnEntry::Row(KvnRow {
                            values: vec![
                                "2024-01-01T00:01:00".to_string(),
                                "6999.5".to_string(),
                                "450.0".to_string(),
                                "0.0".to_string(),
                                "-0.07".to_string(),
                                "7.49".to_string(),
                                "0.0".to_string(),
                            ],
                        }),
                    ],
                },
            ],
        };
        round_trip(&doc);
    }
}
