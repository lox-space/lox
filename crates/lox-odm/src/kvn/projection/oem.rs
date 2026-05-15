// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! KVN ↔ typed [`Oem`] projection.
//!
//! - [`From<&Oem> for KvnDocument`] emits a canonical AST.
//! - [`TryFrom<KvnDocument> for Oem`] validates and projects an AST
//!   produced by [`crate::kvn::parser`].
//! - [`read_oem`] / [`write_oem`] are the public free functions
//!   re-exported from [`crate::kvn`].

use std::collections::BTreeMap;
use std::convert::TryFrom;

use lox_core::coords::Cartesian;
use lox_core::units::{Distance, Velocity};

use crate::kvn::ast::{KvnDocument, KvnEntry, KvnField, KvnRow, KvnSection};
use crate::kvn::error::{KvnError, KvnErrorKind, Span};
use crate::kvn::parser::parse;
use crate::kvn::projection::{find_field, parse_epoch, parse_f64, require_field};
use crate::types::common::{MessageKind, OdmCenter, OdmFrame, OdmHeader};
use crate::types::oem::{Oem, OemCovariance, OemMetadata, OemSegment};

// ---------------------------------------------------------------------------
// Write helpers
// ---------------------------------------------------------------------------

/// Format an [`OdmTime`] as ISO-8601 only, without the trailing time-scale
/// abbreviation.
///
/// The KVN writer includes the scale abbreviation in `Display` (e.g.
/// `"2000-01-01T11:58:55.816 TAI"`), which is fine for keyword-value fields
/// where the value string is consumed as a unit. However, OEM positional rows
/// embed the epoch as the first whitespace-separated token, so the scale
/// abbreviation would be parsed as a separate (second) token and produce 8
/// values instead of 7. This helper strips everything after the first space.
fn epoch_iso(epoch: &crate::types::common::OdmTime) -> String {
    let full = format!("{epoch}");
    // `Display` emits `<iso> <scale>` for continuous scales; strip the suffix.
    full.split_whitespace().next().unwrap_or(&full).to_string()
}

/// Build a single `KEY = VALUE [unit]` entry.
fn fld(key: &str, value: impl ToString, unit: Option<&str>) -> KvnEntry {
    KvnEntry::Field(KvnField {
        key: key.to_string(),
        value: value.to_string(),
        unit: unit.map(|u| u.to_string()),
    })
}

fn build_header_section(oem: &Oem) -> KvnSection {
    let mut entries = Vec::new();

    for comment in &oem.header.comments {
        entries.push(KvnEntry::Comment(comment.clone()));
    }

    if let Some(cls) = &oem.header.classification {
        entries.push(fld("CLASSIFICATION", cls, None));
    }

    entries.push(fld(
        "CREATION_DATE",
        format!("{}", oem.header.creation_date),
        None,
    ));
    entries.push(fld("ORIGINATOR", &oem.header.originator, None));

    if let Some(mid) = &oem.header.message_id {
        entries.push(fld("MESSAGE_ID", mid, None));
    }

    KvnSection {
        keyword: "HEADER".to_string(),
        bracketed: false,
        entries,
    }
}

fn build_metadata_section(segment: &OemSegment) -> KvnSection {
    let meta = &segment.metadata;
    let mut entries = Vec::new();

    for comment in &meta.comments {
        entries.push(KvnEntry::Comment(comment.clone()));
    }

    entries.push(fld("OBJECT_NAME", &meta.object_name, None));
    entries.push(fld("OBJECT_ID", &meta.object_id, None));
    entries.push(fld("CENTER_NAME", meta.center.name(), None));
    entries.push(fld("REF_FRAME", meta.frame.name(), None));

    if let Some(epoch) = &meta.frame_epoch {
        entries.push(fld("REF_FRAME_EPOCH", format!("{epoch}"), None));
    }

    entries.push(fld("TIME_SYSTEM", meta.start_time.time_system(), None));
    entries.push(fld("START_TIME", format!("{}", meta.start_time), None));

    if let Some(t) = &meta.useable_start_time {
        entries.push(fld("USEABLE_START_TIME", format!("{t}"), None));
    }
    if let Some(t) = &meta.useable_stop_time {
        entries.push(fld("USEABLE_STOP_TIME", format!("{t}"), None));
    }

    entries.push(fld("STOP_TIME", format!("{}", meta.stop_time), None));

    if let Some(interp) = &meta.interpolation {
        entries.push(fld("INTERPOLATION", interp, None));
    }
    if let Some(deg) = meta.interpolation_degree {
        entries.push(fld("INTERPOLATION_DEGREE", format!("{deg}"), None));
    }

    KvnSection {
        keyword: "META".to_string(),
        bracketed: true,
        entries,
    }
}

fn build_data_section(segment: &OemSegment) -> KvnSection {
    let mut entries = Vec::new();

    for comment in &segment.data_comments {
        entries.push(KvnEntry::Comment(comment.clone()));
    }

    for (epoch, cart) in &segment.states {
        let pos = cart.position();
        let vel = cart.velocity();
        entries.push(KvnEntry::Row(KvnRow {
            values: vec![
                epoch_iso(epoch),
                format!("{}", pos.x / 1000.0),
                format!("{}", pos.y / 1000.0),
                format!("{}", pos.z / 1000.0),
                format!("{}", vel.x / 1000.0),
                format!("{}", vel.y / 1000.0),
                format!("{}", vel.z / 1000.0),
            ],
        }));
    }

    for cov in &segment.covariance_history {
        entries.push(KvnEntry::Subsection(build_covariance_subsection(cov)));
    }

    KvnSection {
        keyword: "DATA".to_string(),
        bracketed: false,
        entries,
    }
}

fn build_covariance_subsection(cov: &OemCovariance) -> KvnSection {
    let mut entries = Vec::new();

    for comment in &cov.comments {
        entries.push(KvnEntry::Comment(comment.clone()));
    }

    entries.push(fld("EPOCH", format!("{}", cov.epoch), None));

    if let Some(frame) = &cov.frame {
        entries.push(fld("COV_REF_FRAME", frame.name(), None));
    }

    // 6 lower-triangle rows: row i has i+1 values
    for i in 0..6usize {
        let values: Vec<String> = (0..=i).map(|j| format!("{}", cov.matrix[(i, j)])).collect();
        entries.push(KvnEntry::Row(KvnRow { values }));
    }

    KvnSection {
        keyword: "COVARIANCE".to_string(),
        bracketed: true,
        entries,
    }
}

impl From<&Oem> for KvnDocument {
    fn from(oem: &Oem) -> Self {
        let mut sections = Vec::new();

        sections.push(build_header_section(oem));

        for segment in &oem.segments {
            sections.push(build_metadata_section(segment));
            sections.push(build_data_section(segment));
        }

        // User-defined trailing section (BTreeMap iteration is sorted)
        if !oem.user_defined.is_empty() {
            let mut ud_entries = Vec::new();
            for (key, value) in &oem.user_defined {
                ud_entries.push(fld(&format!("USER_DEFINED_{key}"), value, None));
            }
            sections.push(KvnSection {
                keyword: "USER_DEFINED".to_string(),
                bracketed: false,
                entries: ud_entries,
            });
        }

        KvnDocument {
            message_kind: MessageKind::Oem,
            version: "3.0".to_string(),
            preamble: Vec::new(),
            sections,
        }
    }
}

/// Serialises an [`Oem`] to its canonical KVN text form.
pub fn write_oem(oem: &Oem) -> String {
    let doc: KvnDocument = oem.into();
    doc.to_string()
}

// ---------------------------------------------------------------------------
// Read direction helpers
// ---------------------------------------------------------------------------

/// Return the value string of an optional field (no type conversion).
fn parse_string_optional(entries: &[KvnEntry], keyword: &str) -> Result<Option<String>, KvnError> {
    match find_field(entries, keyword)? {
        Some(f) => Ok(Some(f.value.trim().to_string())),
        None => Ok(None),
    }
}

/// Return the value string of a required field (no type conversion).
fn parse_string_required(entries: &[KvnEntry], keyword: &str) -> Result<String, KvnError> {
    let f = require_field(entries, keyword)?;
    Ok(f.value.trim().to_string())
}

/// Parse metadata from a META bracketed section's entries.
fn parse_metadata(entries: &[KvnEntry]) -> Result<OemMetadata, KvnError> {
    let mut comments = Vec::new();
    for entry in entries {
        if let KvnEntry::Comment(c) = entry {
            comments.push(c.clone());
        }
    }

    let object_name = parse_string_required(entries, "OBJECT_NAME")?;
    let object_id = parse_string_required(entries, "OBJECT_ID")?;
    let center_name = parse_string_required(entries, "CENTER_NAME")?;
    let center = OdmCenter::from_wire(&center_name);
    let ref_frame = parse_string_required(entries, "REF_FRAME")?;
    let frame = OdmFrame::from_wire(&ref_frame);

    let time_system = parse_string_required(entries, "TIME_SYSTEM")?;

    let frame_epoch = match parse_string_optional(entries, "REF_FRAME_EPOCH")? {
        Some(s) => {
            let dummy_field = KvnField {
                key: "REF_FRAME_EPOCH".to_string(),
                value: s,
                unit: None,
            };
            Some(parse_epoch(&dummy_field, &time_system)?)
        }
        None => None,
    };

    let start_time_field = require_field(entries, "START_TIME")?;
    let start_time = parse_epoch(start_time_field, &time_system)?;

    let useable_start_time = match parse_string_optional(entries, "USEABLE_START_TIME")? {
        Some(s) => {
            let f = KvnField {
                key: "USEABLE_START_TIME".to_string(),
                value: s,
                unit: None,
            };
            Some(parse_epoch(&f, &time_system)?)
        }
        None => None,
    };

    let useable_stop_time = match parse_string_optional(entries, "USEABLE_STOP_TIME")? {
        Some(s) => {
            let f = KvnField {
                key: "USEABLE_STOP_TIME".to_string(),
                value: s,
                unit: None,
            };
            Some(parse_epoch(&f, &time_system)?)
        }
        None => None,
    };

    let stop_time_field = require_field(entries, "STOP_TIME")?;
    let stop_time = parse_epoch(stop_time_field, &time_system)?;

    let interpolation = parse_string_optional(entries, "INTERPOLATION")?;
    let interpolation_degree = match parse_string_optional(entries, "INTERPOLATION_DEGREE")? {
        Some(s) => {
            let v = s.trim().parse::<u64>().map_err(|e| KvnError {
                span: Span::default(),
                kind: KvnErrorKind::InvalidValue {
                    keyword: "INTERPOLATION_DEGREE".to_string(),
                    reason: e.to_string(),
                },
            })?;
            Some(v)
        }
        None => None,
    };

    Ok(OemMetadata {
        comments,
        object_name,
        object_id,
        center,
        frame,
        frame_epoch,
        start_time,
        useable_start_time,
        useable_stop_time,
        stop_time,
        interpolation,
        interpolation_degree,
    })
}

/// Parse a covariance block from a COVARIANCE bracketed section's entries.
fn parse_oem_covariance(
    entries: &[KvnEntry],
    time_system: &str,
) -> Result<OemCovariance, KvnError> {
    let mut comments = Vec::new();
    for entry in entries {
        if let KvnEntry::Comment(c) = entry {
            comments.push(c.clone());
        }
    }

    let epoch_field = require_field(entries, "EPOCH")?;
    let epoch = parse_epoch(epoch_field, time_system)?;

    let frame = parse_string_optional(entries, "COV_REF_FRAME")?.map(|s| OdmFrame::from_wire(&s));

    // Collect positional rows (6 lower-triangle rows)
    let rows: Vec<&KvnRow> = entries
        .iter()
        .filter_map(|e| {
            if let KvnEntry::Row(r) = e {
                Some(r)
            } else {
                None
            }
        })
        .collect();

    if rows.len() != 6 {
        return Err(KvnError {
            span: Span::default(),
            kind: KvnErrorKind::InvalidValue {
                keyword: "COVARIANCE".to_string(),
                reason: format!("expected 6 covariance rows, got {}", rows.len()),
            },
        });
    }

    let mut matrix = nalgebra::Matrix6::<f64>::zeros();
    for (i, row) in rows.iter().enumerate() {
        let expected = i + 1;
        if row.values.len() != expected {
            return Err(KvnError {
                span: Span::default(),
                kind: KvnErrorKind::InvalidValue {
                    keyword: "COVARIANCE".to_string(),
                    reason: format!(
                        "covariance row {i} has {} values, expected {expected}",
                        row.values.len()
                    ),
                },
            });
        }
        for (j, val_str) in row.values.iter().enumerate() {
            let v = val_str.trim().parse::<f64>().map_err(|e| KvnError {
                span: Span::default(),
                kind: KvnErrorKind::InvalidValue {
                    keyword: "COVARIANCE".to_string(),
                    reason: e.to_string(),
                },
            })?;
            matrix[(i, j)] = v;
            if i != j {
                matrix[(j, i)] = v;
            }
        }
    }

    Ok(OemCovariance {
        comments,
        epoch,
        frame,
        matrix,
    })
}

/// A partially built segment during the read pass.
struct SegmentBuilder {
    metadata: OemMetadata,
    data_comments: Vec<String>,
    states: Vec<(crate::types::common::OdmTime, Cartesian)>,
    covariance_history: Vec<OemCovariance>,
}

impl SegmentBuilder {
    fn new(metadata: OemMetadata) -> Self {
        SegmentBuilder {
            metadata,
            data_comments: Vec::new(),
            states: Vec::new(),
            covariance_history: Vec::new(),
        }
    }

    fn build(self) -> OemSegment {
        OemSegment {
            metadata: self.metadata,
            data_comments: self.data_comments,
            states: self.states,
            covariance_history: self.covariance_history,
        }
    }
}

/// Process a single state row from a positional row entry.
fn process_state_row(
    row: &KvnRow,
    time_system: &str,
) -> Result<(crate::types::common::OdmTime, Cartesian), KvnError> {
    if row.values.len() != 7 {
        return Err(KvnError {
            span: Span::default(),
            kind: KvnErrorKind::InvalidValue {
                keyword: "DATA".to_string(),
                reason: format!("state row has {} values, expected 7", row.values.len()),
            },
        });
    }

    let epoch_field = KvnField {
        key: "EPOCH".to_string(),
        value: row.values[0].clone(),
        unit: None,
    };
    let epoch = parse_epoch(&epoch_field, time_system)?;

    let x_f = KvnField {
        key: "X".to_string(),
        value: row.values[1].clone(),
        unit: None,
    };
    let y_f = KvnField {
        key: "Y".to_string(),
        value: row.values[2].clone(),
        unit: None,
    };
    let z_f = KvnField {
        key: "Z".to_string(),
        value: row.values[3].clone(),
        unit: None,
    };
    let vx_f = KvnField {
        key: "VX".to_string(),
        value: row.values[4].clone(),
        unit: None,
    };
    let vy_f = KvnField {
        key: "VY".to_string(),
        value: row.values[5].clone(),
        unit: None,
    };
    let vz_f = KvnField {
        key: "VZ".to_string(),
        value: row.values[6].clone(),
        unit: None,
    };

    let x = Distance::kilometers(parse_f64(&x_f)?);
    let y = Distance::kilometers(parse_f64(&y_f)?);
    let z = Distance::kilometers(parse_f64(&z_f)?);
    let vx = Velocity::kilometers_per_second(parse_f64(&vx_f)?);
    let vy = Velocity::kilometers_per_second(parse_f64(&vy_f)?);
    let vz = Velocity::kilometers_per_second(parse_f64(&vz_f)?);

    Ok((epoch, Cartesian::new(x, y, z, vx, vy, vz)))
}

/// Recursively process a section's entries, attributing them to the current
/// segment builder.
///
/// The KVN parser nests subsequent segments: after the first META_STOP opens
/// an implicit DATA section, any additional META_START/META_STOP within that
/// DATA section is recorded as a nested `KvnEntry::Subsection`. This function
/// walks those nested subsections and correctly splits them into segments.
fn process_entries(
    entries: &[KvnEntry],
    current_builder: &mut Option<SegmentBuilder>,
    segment_builders: &mut Vec<SegmentBuilder>,
    user_defined: &mut BTreeMap<String, String>,
) -> Result<(), KvnError> {
    for entry in entries {
        match entry {
            KvnEntry::Comment(c) => {
                if let Some(builder) = current_builder.as_mut() {
                    builder.data_comments.push(c.clone());
                }
            }
            KvnEntry::Row(row) => {
                let builder = current_builder.as_mut().ok_or_else(|| KvnError {
                    span: Span::default(),
                    kind: KvnErrorKind::UnexpectedKeyword("data row before META".to_string()),
                })?;
                let time_system = builder.metadata.start_time.time_system().to_string();
                let state = process_state_row(row, &time_system)?;
                builder.states.push(state);
            }
            KvnEntry::Subsection(sub) => {
                match sub.keyword.as_str() {
                    "COVARIANCE" => {
                        let builder = current_builder.as_mut().ok_or_else(|| KvnError {
                            span: Span::default(),
                            kind: KvnErrorKind::UnexpectedKeyword(
                                "COVARIANCE before META".to_string(),
                            ),
                        })?;
                        let time_system = builder.metadata.start_time.time_system().to_string();
                        let cov = parse_oem_covariance(&sub.entries, &time_system)?;
                        builder.covariance_history.push(cov);
                    }
                    "META" => {
                        // Nested META section (second+ segment after round-trip
                        // through the parser). Close the current segment.
                        if let Some(builder) = current_builder.take() {
                            segment_builders.push(builder);
                        }
                        let metadata = parse_metadata(&sub.entries)?;
                        *current_builder = Some(SegmentBuilder::new(metadata));
                    }
                    _ => {
                        // Recursively process unknown subsections to find
                        // any nested META/COVARIANCE/data rows within.
                        process_entries(
                            &sub.entries,
                            current_builder,
                            segment_builders,
                            user_defined,
                        )?;
                    }
                }
            }
            KvnEntry::Field(f) => {
                if let Some(suffix) = f.key.strip_prefix("USER_DEFINED_") {
                    user_defined.insert(suffix.to_string(), f.value.trim().to_string());
                }
                // Other fields in DATA sections are silently ignored; they
                // don't belong to the OEM data grammar.
            }
        }
    }
    Ok(())
}

impl TryFrom<KvnDocument> for Oem {
    type Error = KvnError;

    fn try_from(doc: KvnDocument) -> Result<Self, Self::Error> {
        // 1. Validate message kind.
        if doc.message_kind != MessageKind::Oem {
            return Err(KvnError {
                span: Span::default(),
                kind: KvnErrorKind::UnexpectedKeyword(format!("{}", doc.message_kind)),
            });
        }

        // 2. Collect header entries and comments from the first HEADER section.
        let mut header_entries: Vec<KvnEntry> = Vec::new();
        let mut header_comments: Vec<String> = Vec::new();
        let mut segment_builders: Vec<SegmentBuilder> = Vec::new();
        let mut current_builder: Option<SegmentBuilder> = None;
        let mut user_defined: BTreeMap<String, String> = BTreeMap::new();
        let mut in_header = true;

        // 3. Walk top-level sections.
        for section in &doc.sections {
            if in_header && !section.bracketed && section.keyword == "HEADER" {
                for entry in &section.entries {
                    match entry {
                        KvnEntry::Comment(c) => header_comments.push(c.clone()),
                        _ => header_entries.push(entry.clone()),
                    }
                }
                continue;
            }

            in_header = false;

            match (section.bracketed, section.keyword.as_str()) {
                (true, "META") => {
                    // Top-level META section — new segment.
                    if let Some(builder) = current_builder.take() {
                        segment_builders.push(builder);
                    }
                    let metadata = parse_metadata(&section.entries)?;
                    current_builder = Some(SegmentBuilder::new(metadata));
                }
                (false, "DATA") | (false, _) => {
                    // Implicit DATA (or unrecognised implicit) section.
                    // May contain data rows, nested COVARIANCE, nested META
                    // (for subsequent segments), or USER_DEFINED fields.
                    process_entries(
                        &section.entries,
                        &mut current_builder,
                        &mut segment_builders,
                        &mut user_defined,
                    )?;
                }
                (true, "COVARIANCE") => {
                    // Top-level COVARIANCE section.
                    let builder = current_builder.as_mut().ok_or_else(|| KvnError {
                        span: Span::default(),
                        kind: KvnErrorKind::UnexpectedKeyword("COVARIANCE before META".to_string()),
                    })?;
                    let time_system = builder.metadata.start_time.time_system().to_string();
                    let cov = parse_oem_covariance(&section.entries, &time_system)?;
                    builder.covariance_history.push(cov);
                }
                _ => {
                    // Bracketed section with unknown keyword — ignore.
                }
            }
        }

        // Close the last open segment.
        if let Some(builder) = current_builder.take() {
            segment_builders.push(builder);
        }

        // 4. Parse header fields.
        // OEM has no top-level TIME_SYSTEM (it lives inside each META block).
        // When `write_oem` serialises `CREATION_DATE`, the epoch Display
        // includes the scale abbreviation as a trailing token (e.g.
        // `"2000-01-01T11:58:55.816 TAI"`). Extract it from the value to
        // feed back to `parse_epoch`, falling back to "UTC" for externally-
        // produced files that omit the trailing abbreviation.
        let creation_date_field = require_field(&header_entries, "CREATION_DATE")?;
        let time_system_for_header = creation_date_field
            .value
            .split_whitespace()
            .nth(1)
            .unwrap_or("UTC");
        let creation_date = parse_epoch(creation_date_field, time_system_for_header)?;
        let originator = parse_string_required(&header_entries, "ORIGINATOR")?;
        let classification = parse_string_optional(&header_entries, "CLASSIFICATION")?;
        let message_id = parse_string_optional(&header_entries, "MESSAGE_ID")?;

        let header = OdmHeader {
            comments: header_comments,
            classification,
            creation_date,
            originator,
            message_id,
        };

        let segments: Vec<OemSegment> = segment_builders.into_iter().map(|b| b.build()).collect();

        Ok(Oem {
            header,
            segments,
            user_defined,
        })
    }
}

/// Parses a KVN-formatted string into a typed [`Oem`].
pub fn read_oem(input: &str) -> Result<Oem, KvnError> {
    let doc = parse(input)?;
    Oem::try_from(doc)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use lox_bodies::DynOrigin;
    use lox_core::units::{Distance, Velocity};
    use lox_frames::DynFrame;
    use nalgebra::Matrix6;

    use crate::kvn::error::KvnErrorKind;
    use crate::types::common::{OdmCenter, OdmFrame, OdmHeader, OdmTime};
    use crate::types::oem::{Oem, OemCovariance, OemMetadata, OemSegment};

    use super::*;

    fn sample_epoch() -> OdmTime {
        OdmTime::Time(lox_time::time::Time::j2000(
            lox_time::time_scales::DynTimeScale::Tai,
        ))
    }

    fn sample_epoch_plus(seconds: i64) -> OdmTime {
        use lox_time::deltas::TimeDelta;
        OdmTime::Time(
            lox_time::time::Time::j2000(lox_time::time_scales::DynTimeScale::Tai)
                + TimeDelta::from_seconds(seconds),
        )
    }

    fn sample_state(km: f64) -> Cartesian {
        Cartesian::new(
            Distance::kilometers(7000.0 + km),
            Distance::kilometers(0.0),
            Distance::kilometers(0.0),
            Velocity::kilometers_per_second(0.0),
            Velocity::kilometers_per_second(7.5),
            Velocity::kilometers_per_second(0.0),
        )
    }

    fn sample_metadata() -> OemMetadata {
        OemMetadata {
            comments: Vec::new(),
            object_name: "TEST-SAT".to_string(),
            object_id: "2024-000A".to_string(),
            center: OdmCenter::Known(DynOrigin::Earth),
            frame: OdmFrame::Known(DynFrame::Icrf),
            frame_epoch: None,
            start_time: sample_epoch(),
            useable_start_time: None,
            useable_stop_time: None,
            stop_time: sample_epoch_plus(60),
            interpolation: None,
            interpolation_degree: None,
        }
    }

    fn sample_segment() -> OemSegment {
        OemSegment {
            metadata: sample_metadata(),
            data_comments: Vec::new(),
            states: vec![
                (sample_epoch(), sample_state(0.0)),
                (sample_epoch_plus(60), sample_state(1.0)),
            ],
            covariance_history: Vec::new(),
        }
    }

    fn sample_oem() -> Oem {
        Oem {
            header: OdmHeader {
                comments: Vec::new(),
                classification: None,
                creation_date: sample_epoch(),
                originator: "TEST".to_string(),
                message_id: None,
            },
            segments: vec![sample_segment()],
            user_defined: BTreeMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Write direction tests
    // -----------------------------------------------------------------------

    #[test]
    fn write_minimal_oem_contains_expected_fields() {
        let oem = sample_oem();
        let output = write_oem(&oem);

        assert!(
            output.contains("CCSDS_OEM_VERS = 3.0"),
            "missing version line; got:\n{output}"
        );
        assert!(
            output.contains("ORIGINATOR = TEST"),
            "missing ORIGINATOR; got:\n{output}"
        );
        assert!(
            output.contains("META_START"),
            "missing META_START; got:\n{output}"
        );
        assert!(
            output.contains("META_STOP"),
            "missing META_STOP; got:\n{output}"
        );
        assert!(
            output.contains("OBJECT_NAME = TEST-SAT"),
            "missing OBJECT_NAME; got:\n{output}"
        );
        assert!(
            output.contains("CENTER_NAME = EARTH"),
            "missing CENTER_NAME; got:\n{output}"
        );
        assert!(
            output.contains("REF_FRAME = ICRF"),
            "missing REF_FRAME; got:\n{output}"
        );
        assert!(
            output.contains("TIME_SYSTEM = TAI"),
            "missing TIME_SYSTEM; got:\n{output}"
        );
        assert!(
            output.contains("START_TIME ="),
            "missing START_TIME; got:\n{output}"
        );
        assert!(
            output.contains("STOP_TIME ="),
            "missing STOP_TIME; got:\n{output}"
        );
        // Two ephemeris rows
        let row_count = output
            .lines()
            .filter(|l| {
                // A row line starts with a date-like token (contains 'T' for ISO datetime)
                // and is not a keyword-value pair
                !l.contains('=')
                    && !l.contains("_START")
                    && !l.contains("_STOP")
                    && !l.trim().is_empty()
                    && !l.starts_with("COMMENT")
            })
            .count();
        assert!(
            row_count >= 2,
            "expected at least 2 data rows; got:\n{output}"
        );
    }

    #[test]
    fn write_multi_segment_oem_has_two_meta_blocks() {
        let mut oem = sample_oem();
        oem.segments.push(sample_segment());

        let output = write_oem(&oem);

        let meta_start_count = output.matches("META_START").count();
        let meta_stop_count = output.matches("META_STOP").count();
        assert_eq!(meta_start_count, 2, "expected 2 META_START; got:\n{output}");
        assert_eq!(meta_stop_count, 2, "expected 2 META_STOP; got:\n{output}");
    }

    #[test]
    fn write_oem_with_covariance_emits_covariance_block() {
        let mut oem = sample_oem();
        oem.segments[0].covariance_history.push(OemCovariance {
            comments: Vec::new(),
            epoch: sample_epoch(),
            frame: None,
            matrix: Matrix6::identity(),
        });

        let output = write_oem(&oem);

        assert!(
            output.contains("COVARIANCE_START"),
            "missing COVARIANCE_START; got:\n{output}"
        );
        assert!(
            output.contains("COVARIANCE_STOP"),
            "missing COVARIANCE_STOP; got:\n{output}"
        );
        assert!(
            output.contains("EPOCH ="),
            "missing EPOCH in covariance; got:\n{output}"
        );
        // Check that 6 lower-triangle rows appear after COVARIANCE_START
        // Row 0 has 1 value (diagonal), row 5 has 6 values
        // We just check the EPOCH field and that rows appear
        let cov_start = output.find("COVARIANCE_START").unwrap();
        let cov_stop = output.find("COVARIANCE_STOP").unwrap();
        let cov_block = &output[cov_start..cov_stop];
        let row_count = cov_block
            .lines()
            .filter(|l| !l.contains('=') && !l.trim().is_empty() && !l.contains("_START"))
            .count();
        assert_eq!(
            row_count, 6,
            "expected 6 covariance rows; got:\n{cov_block}"
        );
    }

    #[test]
    fn write_oem_metadata_comments_appear_before_fields() {
        let mut oem = sample_oem();
        oem.segments[0]
            .metadata
            .comments
            .push("A metadata comment".to_string());

        let output = write_oem(&oem);

        let comment_pos = output
            .find("COMMENT A metadata comment")
            .expect("comment not found");
        let object_pos = output.find("OBJECT_NAME =").expect("OBJECT_NAME not found");
        assert!(
            comment_pos < object_pos,
            "comment should appear before OBJECT_NAME"
        );
    }

    #[test]
    fn write_oem_user_defined_emitted_at_end() {
        let mut oem = sample_oem();
        oem.user_defined
            .insert("OPERATOR".to_string(), "GSOC".to_string());

        let output = write_oem(&oem);

        assert!(
            output.contains("USER_DEFINED_OPERATOR = GSOC"),
            "missing USER_DEFINED_OPERATOR; got:\n{output}"
        );
        // Should appear after all META/DATA sections
        let ud_pos = output
            .find("USER_DEFINED_OPERATOR")
            .expect("USER_DEFINED_OPERATOR not found");
        let last_stop = output.rfind("META_STOP").expect("META_STOP not found");
        assert!(
            ud_pos > last_stop,
            "USER_DEFINED should appear after META_STOP"
        );
    }

    // -----------------------------------------------------------------------
    // Read direction tests
    // -----------------------------------------------------------------------

    #[test]
    fn round_trip_minimal_oem() {
        let oem = sample_oem();
        let written = write_oem(&oem);
        let parsed = read_oem(&written).expect("parse failed");
        assert_eq!(oem, parsed, "round-trip mismatch;\nwritten:\n{written}");
    }

    #[test]
    fn round_trip_multi_segment_oem() {
        let mut oem = sample_oem();
        oem.segments.push(sample_segment());

        let written = write_oem(&oem);
        let parsed = read_oem(&written).expect("parse failed");
        assert_eq!(
            parsed.segments.len(),
            2,
            "expected 2 segments; written:\n{written}"
        );
        assert_eq!(oem, parsed, "multi-segment round-trip mismatch");
    }

    #[test]
    fn round_trip_oem_with_covariance() {
        let mut matrix = Matrix6::<f64>::zeros();
        // Fill lower triangle with distinct values for thorough test
        for i in 0..6usize {
            for j in 0..=i {
                let v = (i * 10 + j) as f64 * 0.001;
                matrix[(i, j)] = v;
                matrix[(j, i)] = v;
            }
        }

        let mut oem = sample_oem();
        oem.segments[0].covariance_history.push(OemCovariance {
            comments: Vec::new(),
            epoch: sample_epoch(),
            frame: None,
            matrix,
        });

        let written = write_oem(&oem);
        let parsed = read_oem(&written).expect("parse failed");
        assert_eq!(
            oem, parsed,
            "covariance round-trip mismatch;\nwritten:\n{written}"
        );
    }

    #[test]
    fn read_missing_start_time_returns_error() {
        // Build a valid OEM string then remove the START_TIME line
        let oem = sample_oem();
        let written = write_oem(&oem);
        let without_start = written
            .lines()
            .filter(|l| !l.starts_with("START_TIME"))
            .collect::<Vec<_>>()
            .join("\n");

        let err = read_oem(&without_start).expect_err("should fail on missing START_TIME");
        assert!(
            matches!(err.kind, KvnErrorKind::MissingRequiredField(ref k) if k == "START_TIME"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_covariance_wrong_row_width_returns_error() {
        // An OEM with a COVARIANCE block where row 1 has 3 values (should be 2)
        let kvn = "\
CCSDS_OEM_VERS = 3.0
CREATION_DATE = 2000-01-01T11:58:55.816
ORIGINATOR = TEST
META_START
OBJECT_NAME = TEST-SAT
OBJECT_ID = 2024-000A
CENTER_NAME = EARTH
REF_FRAME = ICRF
TIME_SYSTEM = TAI
START_TIME = 2000-01-01T11:58:55.816
STOP_TIME = 2000-01-01T11:59:55.816
META_STOP
2000-01-01T11:58:55.816 7000.0 0.0 0.0 0.0 7.5 0.0
COVARIANCE_START
EPOCH = 2000-01-01T11:58:55.816
1.0
2.0 3.0 4.0
3.0 4.0 5.0
4.0 5.0 6.0 7.0
5.0 6.0 7.0 8.0 9.0
6.0 7.0 8.0 9.0 10.0 11.0
COVARIANCE_STOP
";
        let err = read_oem(kvn).expect_err("should fail on wrong row width");
        assert!(
            matches!(err.kind, KvnErrorKind::InvalidValue { .. }),
            "unexpected error kind: {err}"
        );
    }

    #[test]
    fn read_user_defined_preserved() {
        let mut oem = sample_oem();
        oem.user_defined
            .insert("OPERATOR".to_string(), "GSOC".to_string());

        let written = write_oem(&oem);
        let parsed = read_oem(&written).expect("parse failed");
        assert_eq!(
            parsed.user_defined.get("OPERATOR"),
            Some(&"GSOC".to_string()),
            "USER_DEFINED_OPERATOR not preserved"
        );
    }

    #[test]
    fn round_trip_oem_with_interpolation() {
        let mut oem = sample_oem();
        oem.segments[0].metadata.interpolation = Some("HERMITE".to_string());
        oem.segments[0].metadata.interpolation_degree = Some(7);

        let written = write_oem(&oem);
        let parsed = read_oem(&written).expect("parse failed");
        assert_eq!(
            parsed.segments[0].metadata.interpolation.as_deref(),
            Some("HERMITE")
        );
        assert_eq!(parsed.segments[0].metadata.interpolation_degree, Some(7));
    }

    // -----------------------------------------------------------------------
    // Additional tests for uncovered branches
    // -----------------------------------------------------------------------

    #[test]
    fn read_wrong_message_kind_returns_error() {
        let kvn = "\
CCSDS_OPM_VERS = 3.0
CREATION_DATE = 2024-01-01T00:00:00
ORIGINATOR = TEST
OBJECT_NAME = TEST-SAT
OBJECT_ID = 2024-000A
CENTER_NAME = EARTH
REF_FRAME = ICRF
TIME_SYSTEM = TAI
EPOCH = 2024-01-01T00:00:00
X = 7000.0 [km]
Y = 0.0 [km]
Z = 0.0 [km]
X_DOT = 0.0 [km/s]
Y_DOT = 7.5 [km/s]
Z_DOT = 0.0 [km/s]
";
        let err = read_oem(kvn).expect_err("should fail on wrong message kind");
        assert!(
            matches!(err.kind, KvnErrorKind::UnexpectedKeyword(_)),
            "unexpected error kind: {err}"
        );
    }

    #[test]
    fn read_state_row_wrong_width_returns_error() {
        // A state row that has only 6 values (missing vz) should fail.
        let kvn = "\
CCSDS_OEM_VERS = 3.0
CREATION_DATE = 2000-01-01T11:58:55.816
ORIGINATOR = TEST
META_START
OBJECT_NAME = TEST-SAT
OBJECT_ID = 2024-000A
CENTER_NAME = EARTH
REF_FRAME = ICRF
TIME_SYSTEM = TAI
START_TIME = 2000-01-01T11:58:55.816
STOP_TIME = 2000-01-01T11:59:55.816
META_STOP
2000-01-01T11:58:55.816 7000.0 0.0 0.0 0.0 7.5
";
        let err = read_oem(kvn).expect_err("should fail on wrong state row width");
        assert!(
            matches!(err.kind, KvnErrorKind::InvalidValue { .. }),
            "unexpected error kind: {err}"
        );
    }

    #[test]
    fn read_covariance_wrong_row_count_returns_error() {
        // COVARIANCE block with only 5 rows (need 6).
        let kvn = "\
CCSDS_OEM_VERS = 3.0
CREATION_DATE = 2000-01-01T11:58:55.816
ORIGINATOR = TEST
META_START
OBJECT_NAME = TEST-SAT
OBJECT_ID = 2024-000A
CENTER_NAME = EARTH
REF_FRAME = ICRF
TIME_SYSTEM = TAI
START_TIME = 2000-01-01T11:58:55.816
STOP_TIME = 2000-01-01T11:59:55.816
META_STOP
2000-01-01T11:58:55.816 7000.0 0.0 0.0 0.0 7.5 0.0
COVARIANCE_START
EPOCH = 2000-01-01T11:58:55.816
1.0
2.0 3.0
3.0 4.0 5.0
4.0 5.0 6.0 7.0
5.0 6.0 7.0 8.0 9.0
COVARIANCE_STOP
";
        let err = read_oem(kvn).expect_err("should fail on wrong covariance row count");
        assert!(
            matches!(err.kind, KvnErrorKind::InvalidValue { .. }),
            "unexpected error kind: {err}"
        );
    }

    #[test]
    fn write_oem_with_useable_times() {
        let mut oem = sample_oem();
        oem.segments[0].metadata.useable_start_time = Some(sample_epoch());
        oem.segments[0].metadata.useable_stop_time = Some(sample_epoch_plus(60));

        let output = write_oem(&oem);
        assert!(
            output.contains("USEABLE_START_TIME ="),
            "missing USEABLE_START_TIME; got:\n{output}"
        );
        assert!(
            output.contains("USEABLE_STOP_TIME ="),
            "missing USEABLE_STOP_TIME; got:\n{output}"
        );
    }

    #[test]
    fn round_trip_oem_with_useable_times() {
        let mut oem = sample_oem();
        oem.segments[0].metadata.useable_start_time = Some(sample_epoch());
        oem.segments[0].metadata.useable_stop_time = Some(sample_epoch_plus(60));

        let written = write_oem(&oem);
        let parsed = read_oem(&written).expect("parse failed");
        assert!(
            parsed.segments[0].metadata.useable_start_time.is_some(),
            "useable_start_time should survive round-trip"
        );
        assert!(
            parsed.segments[0].metadata.useable_stop_time.is_some(),
            "useable_stop_time should survive round-trip"
        );
    }

    #[test]
    fn round_trip_oem_segment_with_comments() {
        let mut oem = sample_oem();
        oem.segments[0]
            .data_comments
            .push("Data row comment".to_string());

        let written = write_oem(&oem);
        let parsed = read_oem(&written).expect("parse failed");
        assert_eq!(
            parsed.segments[0].data_comments.len(),
            1,
            "data_comments should survive round-trip"
        );
    }

    #[test]
    fn round_trip_oem_with_covariance_and_frame() {
        let mut oem = sample_oem();
        oem.segments[0].covariance_history.push(OemCovariance {
            comments: Vec::new(),
            epoch: sample_epoch(),
            frame: Some(OdmFrame::Known(DynFrame::Icrf)),
            matrix: Matrix6::identity(),
        });

        let written = write_oem(&oem);
        let parsed = read_oem(&written).expect("parse failed");
        let cov = &parsed.segments[0].covariance_history[0];
        assert!(
            cov.frame.is_some(),
            "covariance frame should survive round-trip"
        );
    }

    #[test]
    fn round_trip_oem_with_covariance_comments() {
        let mut oem = sample_oem();
        oem.segments[0].covariance_history.push(OemCovariance {
            comments: vec!["Covariance comment".to_string()],
            epoch: sample_epoch(),
            frame: None,
            matrix: Matrix6::identity(),
        });

        let written = write_oem(&oem);
        let parsed = read_oem(&written).expect("parse failed");
        let cov = &parsed.segments[0].covariance_history[0];
        assert_eq!(
            cov.comments.len(),
            1,
            "covariance comments should survive round-trip"
        );
    }

    #[test]
    fn round_trip_oem_header_classification_and_message_id() {
        let mut oem = sample_oem();
        oem.header.classification = Some("UNCLASSIFIED".to_string());
        oem.header.message_id = Some("OEM-001".to_string());

        let written = write_oem(&oem);
        let parsed = read_oem(&written).expect("parse failed");
        assert_eq!(
            parsed.header.classification.as_deref(),
            Some("UNCLASSIFIED")
        );
        assert_eq!(parsed.header.message_id.as_deref(), Some("OEM-001"));
    }

    #[test]
    fn round_trip_oem_frame_epoch_in_metadata() {
        let mut oem = sample_oem();
        oem.segments[0].metadata.frame_epoch = Some(sample_epoch());

        let written = write_oem(&oem);
        let parsed = read_oem(&written).expect("parse failed");
        assert!(
            parsed.segments[0].metadata.frame_epoch.is_some(),
            "frame_epoch should survive round-trip"
        );
    }

    #[test]
    fn read_interpolation_degree_invalid_returns_error() {
        // Build a valid OEM and replace the interpolation degree with a non-integer
        let mut oem = sample_oem();
        oem.segments[0].metadata.interpolation = Some("HERMITE".to_string());
        oem.segments[0].metadata.interpolation_degree = Some(7);

        let written = write_oem(&oem);
        // Inject an invalid interpolation degree
        let patched = written.replace("INTERPOLATION_DEGREE = 7", "INTERPOLATION_DEGREE = bad");

        let err = read_oem(&patched).expect_err("should fail on invalid INTERPOLATION_DEGREE");
        assert!(
            matches!(err.kind, KvnErrorKind::InvalidValue { ref keyword, .. } if keyword == "INTERPOLATION_DEGREE"),
            "unexpected error kind: {err}"
        );
    }

    #[test]
    fn round_trip_multi_segment_oem_with_covariance() {
        let mut oem = sample_oem();
        // Add covariance to first segment
        oem.segments[0].covariance_history.push(OemCovariance {
            comments: Vec::new(),
            epoch: sample_epoch(),
            frame: None,
            matrix: Matrix6::identity(),
        });
        // Add a second segment
        oem.segments.push(sample_segment());

        let written = write_oem(&oem);
        let parsed = read_oem(&written).expect("parse failed");
        assert_eq!(parsed.segments.len(), 2);
        assert_eq!(parsed.segments[0].covariance_history.len(), 1);
        assert_eq!(parsed.segments[1].covariance_history.len(), 0);
    }
}
