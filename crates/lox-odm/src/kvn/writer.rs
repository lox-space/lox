// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Canonical KVN writer.
//!
//! Implements [`Display`] for [`KvnDocument`] and recursively for its
//! sub-structures. The writer normalises whitespace and casing for
//! semantic-lossless round-trip — see spec section 7.1.

use std::fmt::{self, Display, Formatter};

use crate::kvn::ast::{KvnDocument, KvnEntry, KvnField, KvnRow, KvnSection};
use crate::types::common::MessageKind;

impl Display for KvnDocument {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Header line: CCSDS_<KIND>_VERS = <version>
        let kind_token = match self.message_kind {
            MessageKind::Opm => "OPM",
            MessageKind::Oem => "OEM",
            MessageKind::Omm => "OMM",
            MessageKind::Ocm => "OCM",
            MessageKind::Ci => "NDM",
        };
        writeln!(f, "CCSDS_{kind_token}_VERS = {}", self.version)?;

        for (idx, section) in self.sections.iter().enumerate() {
            // Blank line between top-level sections (but not before the first).
            if idx > 0 {
                writeln!(f)?;
            }
            write_section(f, section)?;
        }
        Ok(())
    }
}

fn write_section(f: &mut Formatter<'_>, section: &KvnSection) -> fmt::Result {
    for comment in &section.leading_comments {
        writeln!(f, "COMMENT {comment}")?;
    }
    if section.bracketed {
        writeln!(f, "{}_START", section.keyword)?;
    }
    for entry in &section.entries {
        write_entry(f, entry)?;
    }
    if section.bracketed {
        writeln!(f, "{}_STOP", section.keyword)?;
    }
    Ok(())
}

fn write_entry(f: &mut Formatter<'_>, entry: &KvnEntry) -> fmt::Result {
    match entry {
        KvnEntry::Field(field) => write_field(f, field),
        KvnEntry::Row(row) => write_row(f, row),
        KvnEntry::Subsection(section) => write_section(f, section),
    }
}

fn write_field(f: &mut Formatter<'_>, field: &KvnField) -> fmt::Result {
    match &field.unit {
        Some(unit) => writeln!(f, "{} = {} [{unit}]", field.key, field.value)?,
        None => writeln!(f, "{} = {}", field.key, field.value)?,
    }
    for comment in &field.trailing_comments {
        writeln!(f, "COMMENT {comment}")?;
    }
    Ok(())
}

fn write_row(f: &mut Formatter<'_>, row: &KvnRow) -> fmt::Result {
    let joined = row.values.join(" ");
    writeln!(f, "{joined}")?;
    for comment in &row.trailing_comments {
        writeln!(f, "COMMENT {comment}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_opm_emits_only_version_header() {
        let doc = KvnDocument {
            message_kind: MessageKind::Opm,
            version: "3.0".to_string(),
            sections: Vec::new(),
        };
        assert_eq!(format!("{doc}"), "CCSDS_OPM_VERS = 3.0\n");
    }

    #[test]
    fn header_section_emits_fields_without_start_stop() {
        let doc = KvnDocument {
            message_kind: MessageKind::Opm,
            version: "3.0".to_string(),
            sections: vec![KvnSection {
                keyword: "HEADER".to_string(),
                bracketed: false,
                leading_comments: Vec::new(),
                entries: vec![
                    KvnEntry::Field(KvnField {
                        key: "CREATION_DATE".to_string(),
                        value: "2024-01-01T00:00:00".to_string(),
                        unit: None,
                        trailing_comments: Vec::new(),
                    }),
                    KvnEntry::Field(KvnField {
                        key: "ORIGINATOR".to_string(),
                        value: "TEST".to_string(),
                        unit: None,
                        trailing_comments: Vec::new(),
                    }),
                ],
            }],
        };
        let expected = "CCSDS_OPM_VERS = 3.0\n\
                        CREATION_DATE = 2024-01-01T00:00:00\n\
                        ORIGINATOR = TEST\n";
        assert_eq!(format!("{doc}"), expected);
    }

    #[test]
    fn bracketed_section_emits_start_stop_markers() {
        let doc = KvnDocument {
            message_kind: MessageKind::Opm,
            version: "3.0".to_string(),
            sections: vec![KvnSection {
                keyword: "META".to_string(),
                bracketed: true,
                leading_comments: Vec::new(),
                entries: vec![KvnEntry::Field(KvnField {
                    key: "OBJECT_NAME".to_string(),
                    value: "ISS".to_string(),
                    unit: None,
                    trailing_comments: Vec::new(),
                })],
            }],
        };
        let expected = "CCSDS_OPM_VERS = 3.0\n\
                        META_START\n\
                        OBJECT_NAME = ISS\n\
                        META_STOP\n";
        assert_eq!(format!("{doc}"), expected);
    }

    #[test]
    fn field_with_unit_renders_brackets() {
        let doc = KvnDocument {
            message_kind: MessageKind::Opm,
            version: "3.0".to_string(),
            sections: vec![KvnSection {
                keyword: "DATA".to_string(),
                bracketed: false,
                leading_comments: Vec::new(),
                entries: vec![KvnEntry::Field(KvnField {
                    key: "X".to_string(),
                    value: "7000.0".to_string(),
                    unit: Some("km".to_string()),
                    trailing_comments: Vec::new(),
                })],
            }],
        };
        let expected = "CCSDS_OPM_VERS = 3.0\n\
                        X = 7000.0 [km]\n";
        assert_eq!(format!("{doc}"), expected);
    }

    #[test]
    fn leading_comments_emit_before_section_content() {
        let doc = KvnDocument {
            message_kind: MessageKind::Opm,
            version: "3.0".to_string(),
            sections: vec![KvnSection {
                keyword: "META".to_string(),
                bracketed: true,
                leading_comments: vec!["Reference orbit".to_string()],
                entries: vec![KvnEntry::Field(KvnField {
                    key: "OBJECT_NAME".to_string(),
                    value: "ISS".to_string(),
                    unit: None,
                    trailing_comments: Vec::new(),
                })],
            }],
        };
        let expected = "CCSDS_OPM_VERS = 3.0\n\
                        COMMENT Reference orbit\n\
                        META_START\n\
                        OBJECT_NAME = ISS\n\
                        META_STOP\n";
        assert_eq!(format!("{doc}"), expected);
    }

    #[test]
    fn trailing_comments_emit_after_field() {
        let doc = KvnDocument {
            message_kind: MessageKind::Opm,
            version: "3.0".to_string(),
            sections: vec![KvnSection {
                keyword: "DATA".to_string(),
                bracketed: false,
                leading_comments: Vec::new(),
                entries: vec![
                    KvnEntry::Field(KvnField {
                        key: "X".to_string(),
                        value: "7000.0".to_string(),
                        unit: Some("km".to_string()),
                        trailing_comments: vec!["operator estimate".to_string()],
                    }),
                    KvnEntry::Field(KvnField {
                        key: "Y".to_string(),
                        value: "0.0".to_string(),
                        unit: Some("km".to_string()),
                        trailing_comments: Vec::new(),
                    }),
                ],
            }],
        };
        let expected = "CCSDS_OPM_VERS = 3.0\n\
                        X = 7000.0 [km]\n\
                        COMMENT operator estimate\n\
                        Y = 0.0 [km]\n";
        assert_eq!(format!("{doc}"), expected);
    }

    #[test]
    fn ephemeris_row_emits_space_separated() {
        let doc = KvnDocument {
            message_kind: MessageKind::Oem,
            version: "3.0".to_string(),
            sections: vec![KvnSection {
                keyword: "DATA".to_string(),
                bracketed: false,
                leading_comments: Vec::new(),
                entries: vec![KvnEntry::Row(KvnRow {
                    values: vec![
                        "2024-01-01T00:00:00".to_string(),
                        "7000.0".to_string(),
                        "0.0".to_string(),
                        "0.0".to_string(),
                        "0.0".to_string(),
                        "7.5".to_string(),
                        "0.0".to_string(),
                    ],
                    trailing_comments: Vec::new(),
                })],
            }],
        };
        let expected = "CCSDS_OEM_VERS = 3.0\n\
                        2024-01-01T00:00:00 7000.0 0.0 0.0 0.0 7.5 0.0\n";
        assert_eq!(format!("{doc}"), expected);
    }

    #[test]
    fn nested_subsection_renders_inline_in_parent() {
        // Models a single maneuver block inside the OPM DATA section.
        let doc = KvnDocument {
            message_kind: MessageKind::Opm,
            version: "3.0".to_string(),
            sections: vec![KvnSection {
                keyword: "DATA".to_string(),
                bracketed: false,
                leading_comments: Vec::new(),
                entries: vec![KvnEntry::Subsection(KvnSection {
                    keyword: "MANEUVER".to_string(),
                    bracketed: false,
                    leading_comments: Vec::new(),
                    entries: vec![KvnEntry::Field(KvnField {
                        key: "MAN_EPOCH_IGNITION".to_string(),
                        value: "2024-01-02T00:00:00".to_string(),
                        unit: None,
                        trailing_comments: Vec::new(),
                    })],
                })],
            }],
        };
        let expected = "CCSDS_OPM_VERS = 3.0\n\
                        MAN_EPOCH_IGNITION = 2024-01-02T00:00:00\n";
        assert_eq!(format!("{doc}"), expected);
    }

    #[test]
    fn multiple_top_level_sections_separated_by_blank_line() {
        let doc = KvnDocument {
            message_kind: MessageKind::Opm,
            version: "3.0".to_string(),
            sections: vec![
                KvnSection {
                    keyword: "HEADER".to_string(),
                    bracketed: false,
                    leading_comments: Vec::new(),
                    entries: vec![KvnEntry::Field(KvnField {
                        key: "ORIGINATOR".to_string(),
                        value: "TEST".to_string(),
                        unit: None,
                        trailing_comments: Vec::new(),
                    })],
                },
                KvnSection {
                    keyword: "META".to_string(),
                    bracketed: true,
                    leading_comments: Vec::new(),
                    entries: vec![KvnEntry::Field(KvnField {
                        key: "OBJECT_NAME".to_string(),
                        value: "ISS".to_string(),
                        unit: None,
                        trailing_comments: Vec::new(),
                    })],
                },
            ],
        };
        let expected = "CCSDS_OPM_VERS = 3.0\n\
                        ORIGINATOR = TEST\n\
                        \n\
                        META_START\n\
                        OBJECT_NAME = ISS\n\
                        META_STOP\n";
        assert_eq!(format!("{doc}"), expected);
    }
}
