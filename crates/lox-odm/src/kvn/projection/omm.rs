// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! KVN ↔ typed [`Omm`] projection.
//!
//! - [`From<&Omm> for KvnDocument`] emits a canonical AST.
//! - [`TryFrom<KvnDocument> for Omm`] validates and projects an AST
//!   that the [`crate::kvn::parser`] produced.
//! - [`read_omm`] / [`write_omm`] are the public free functions
//!   re-exported from [`crate::kvn`].

use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::f64::consts::PI;

use lox_core::elements::{GravitationalParameter, MeanElements};
use lox_core::units::{Area, AreaToMass, Mass};

use crate::kvn::ast::{KvnDocument, KvnEntry, KvnField, KvnSection};
use crate::kvn::error::{KvnError, KvnErrorKind, Span};
use crate::kvn::parser::parse;
use crate::kvn::projection::{find_field, parse_epoch, parse_f64, require_field};
use crate::types::common::{
    Covariance, MessageKind, OdmCenter, OdmFrame, OdmHeader, SpacecraftParameters,
};
use crate::types::omm::{Omm, OmmMeanElements, OmmMetadata, TleParameters};

// ---------------------------------------------------------------------------
// Write direction
// ---------------------------------------------------------------------------

/// Build a single `KEY = VALUE [unit]` entry.
fn fld(key: &str, value: impl ToString, unit: Option<&str>) -> KvnEntry {
    KvnEntry::Field(KvnField {
        key: key.to_string(),
        value: value.to_string(),
        unit: unit.map(|u| u.to_string()),
    })
}

fn build_header_section(omm: &Omm) -> KvnSection {
    let mut entries = Vec::new();

    for comment in &omm.header.comments {
        entries.push(KvnEntry::Comment(comment.clone()));
    }

    if let Some(cls) = &omm.header.classification {
        entries.push(fld("CLASSIFICATION", cls, None));
    }

    entries.push(fld("CREATION_DATE", omm.header.creation_date.iso(), None));
    entries.push(fld("ORIGINATOR", &omm.header.originator, None));

    if let Some(mid) = &omm.header.message_id {
        entries.push(fld("MESSAGE_ID", mid, None));
    }

    KvnSection {
        keyword: "HEADER".to_string(),
        bracketed: false,
        entries,
    }
}

fn build_metadata_section(omm: &Omm) -> KvnSection {
    let mut entries = Vec::new();

    for comment in &omm.metadata.comments {
        entries.push(KvnEntry::Comment(comment.clone()));
    }

    entries.push(fld("OBJECT_NAME", &omm.metadata.object_name, None));
    entries.push(fld("OBJECT_ID", &omm.metadata.object_id, None));
    entries.push(fld("CENTER_NAME", omm.metadata.center.name(), None));
    entries.push(fld("REF_FRAME", omm.metadata.frame.name(), None));

    if let Some(epoch) = &omm.metadata.frame_epoch {
        entries.push(fld("REF_FRAME_EPOCH", epoch.iso(), None));
    }

    entries.push(fld("TIME_SYSTEM", omm.epoch.time_system(), None));
    entries.push(fld(
        "MEAN_ELEMENT_THEORY",
        &omm.metadata.mean_element_theory,
        None,
    ));

    KvnSection {
        keyword: "METADATA".to_string(),
        bracketed: false,
        entries,
    }
}

fn build_data_section(omm: &Omm) -> KvnSection {
    let mut entries = Vec::new();

    // Mean-elements comments
    for comment in &omm.mean_elements.comments {
        entries.push(KvnEntry::Comment(comment.clone()));
    }

    // Epoch
    entries.push(fld("EPOCH", omm.epoch.iso(), None));

    // Semi-major axis (meters → km on wire)
    entries.push(fld(
        "SEMI_MAJOR_AXIS",
        format!("{}", omm.mean_elements.elements.a / 1000.0),
        Some("km"),
    ));
    entries.push(fld(
        "ECCENTRICITY",
        format!("{}", omm.mean_elements.elements.e),
        None,
    ));
    entries.push(fld(
        "INCLINATION",
        format!("{}", omm.mean_elements.elements.i.to_degrees()),
        Some("deg"),
    ));
    entries.push(fld(
        "RA_OF_ASC_NODE",
        format!("{}", omm.mean_elements.elements.raan.to_degrees()),
        Some("deg"),
    ));
    entries.push(fld(
        "ARG_OF_PERICENTER",
        format!("{}", omm.mean_elements.elements.aop.to_degrees()),
        Some("deg"),
    ));
    entries.push(fld(
        "MEAN_ANOMALY",
        format!("{}", omm.mean_elements.elements.m.to_degrees()),
        Some("deg"),
    ));

    // Optional wire GM — emit only when stored on the mean-elements block
    if let Some(gm) = omm.mean_elements.gm {
        // GravitationalParameter stores m³/s²; wire format is km³/s²
        entries.push(fld(
            "GM",
            format!("{}", gm.as_f64() / 1e9),
            Some("km**3/s**2"),
        ));
    }

    // Optional TLE-parameters block
    if let Some(tle) = &omm.tle_parameters {
        for comment in &tle.comments {
            entries.push(KvnEntry::Comment(comment.clone()));
        }

        if let Some(et) = tle.ephemeris_type {
            entries.push(fld("EPHEMERIS_TYPE", format!("{}", et), None));
        }
        if let Some(ct) = &tle.classification_type {
            entries.push(fld("CLASSIFICATION_TYPE", ct, None));
        }
        if let Some(id) = tle.norad_cat_id {
            entries.push(fld("NORAD_CAT_ID", format!("{}", id), None));
        }
        if let Some(esn) = tle.element_set_no {
            entries.push(fld("ELEMENT_SET_NO", format!("{}", esn), None));
        }
        if let Some(rev) = tle.rev_at_epoch {
            entries.push(fld("REV_AT_EPOCH", format!("{}", rev), None));
        }
        if let Some(bs) = tle.bstar {
            entries.push(fld("BSTAR", format!("{}", bs), None));
        }
        if let Some(bt) = tle.bterm {
            entries.push(fld(
                "BTERM",
                format!("{}", bt.to_square_meters_per_kilogram()),
                Some("m**2/kg"),
            ));
        }
        if let Some(mmd) = tle.mean_motion_dot {
            entries.push(fld("MEAN_MOTION_DOT", format!("{}", mmd), None));
        }
        if let Some(mmdd) = tle.mean_motion_ddot {
            entries.push(fld("MEAN_MOTION_DDOT", format!("{}", mmdd), None));
        }
        if let Some(ag) = tle.agom {
            entries.push(fld(
                "AGOM",
                format!("{}", ag.to_square_meters_per_kilogram()),
                Some("m**2/kg"),
            ));
        }
    }

    // Optional spacecraft parameters
    if let Some(sp) = &omm.spacecraft {
        for comment in &sp.comments {
            entries.push(KvnEntry::Comment(comment.clone()));
        }

        if let Some(mass) = sp.mass {
            entries.push(fld("MASS", format!("{}", mass.to_kilograms()), Some("kg")));
        }
        if let Some(sra) = sp.solar_rad_area {
            entries.push(fld(
                "SOLAR_RAD_AREA",
                format!("{}", sra.to_square_meters()),
                Some("m**2"),
            ));
        }
        if let Some(src) = sp.solar_rad_coeff {
            entries.push(fld("SOLAR_RAD_COEFF", format!("{}", src), None));
        }
        if let Some(da) = sp.drag_area {
            entries.push(fld(
                "DRAG_AREA",
                format!("{}", da.to_square_meters()),
                Some("m**2"),
            ));
        }
        if let Some(dc) = sp.drag_coeff {
            entries.push(fld("DRAG_COEFF", format!("{}", dc), None));
        }
    }

    // Optional covariance block — emitted as a bracketed subsection
    if let Some(cov) = &omm.covariance {
        let mut cov_entries = Vec::new();

        for comment in &cov.comments {
            cov_entries.push(KvnEntry::Comment(comment.clone()));
        }

        if let Some(frame) = &cov.frame {
            cov_entries.push(fld("COV_REF_FRAME", frame.name(), None));
        }

        // 21 lower-triangle fields in CCSDS-canonical order
        let cov_fields: &[(&str, usize, usize)] = &[
            ("CX_X", 0, 0),
            ("CY_X", 1, 0),
            ("CY_Y", 1, 1),
            ("CZ_X", 2, 0),
            ("CZ_Y", 2, 1),
            ("CZ_Z", 2, 2),
            ("CX_DOT_X", 3, 0),
            ("CX_DOT_Y", 3, 1),
            ("CX_DOT_Z", 3, 2),
            ("CX_DOT_X_DOT", 3, 3),
            ("CY_DOT_X", 4, 0),
            ("CY_DOT_Y", 4, 1),
            ("CY_DOT_Z", 4, 2),
            ("CY_DOT_X_DOT", 4, 3),
            ("CY_DOT_Y_DOT", 4, 4),
            ("CZ_DOT_X", 5, 0),
            ("CZ_DOT_Y", 5, 1),
            ("CZ_DOT_Z", 5, 2),
            ("CZ_DOT_X_DOT", 5, 3),
            ("CZ_DOT_Y_DOT", 5, 4),
            ("CZ_DOT_Z_DOT", 5, 5),
        ];

        for (name, row, col) in cov_fields {
            cov_entries.push(fld(name, format!("{}", cov.matrix[(*row, *col)]), None));
        }

        entries.push(KvnEntry::Subsection(KvnSection {
            keyword: "COVARIANCE".to_string(),
            bracketed: true,
            entries: cov_entries,
        }));
    }

    // User-defined parameters (BTreeMap iteration is sorted by key)
    for (key, value) in &omm.user_defined {
        entries.push(fld(&format!("USER_DEFINED_{key}"), value, None));
    }

    KvnSection {
        keyword: "DATA".to_string(),
        bracketed: false,
        entries,
    }
}

impl From<&Omm> for KvnDocument {
    fn from(omm: &Omm) -> Self {
        let header_section = build_header_section(omm);
        let metadata_section = build_metadata_section(omm);
        let data_section = build_data_section(omm);

        KvnDocument {
            message_kind: MessageKind::Omm,
            version: "3.0".to_string(),
            preamble: Vec::new(),
            sections: vec![header_section, metadata_section, data_section],
        }
    }
}

/// Serialises an [`Omm`] to its canonical KVN text form.
pub fn write_omm(omm: &Omm) -> String {
    let doc: KvnDocument = omm.into();
    doc.to_string()
}

// ---------------------------------------------------------------------------
// Read direction helpers
// ---------------------------------------------------------------------------

/// Parse an optional `f64` from a flat entry slice by keyword.
fn parse_f64_optional(entries: &[KvnEntry], keyword: &str) -> Result<Option<f64>, KvnError> {
    match find_field(entries, keyword)? {
        Some(f) => Ok(Some(parse_f64(f)?)),
        None => Ok(None),
    }
}

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

/// Parse an optional integer field.
fn parse_i32_optional(entries: &[KvnEntry], keyword: &str) -> Result<Option<i32>, KvnError> {
    match find_field(entries, keyword)? {
        Some(f) => {
            let v = f.value.trim().parse::<i32>().map_err(|e| KvnError {
                span: Span::default(),
                kind: KvnErrorKind::InvalidValue {
                    keyword: f.key.clone(),
                    reason: e.to_string(),
                },
            })?;
            Ok(Some(v))
        }
        None => Ok(None),
    }
}

/// Parse an optional i64 field.
fn parse_i64_optional(entries: &[KvnEntry], keyword: &str) -> Result<Option<i64>, KvnError> {
    match find_field(entries, keyword)? {
        Some(f) => {
            let v = f.value.trim().parse::<i64>().map_err(|e| KvnError {
                span: Span::default(),
                kind: KvnErrorKind::InvalidValue {
                    keyword: f.key.clone(),
                    reason: e.to_string(),
                },
            })?;
            Ok(Some(v))
        }
        None => Ok(None),
    }
}

/// Parse an optional u64 field.
fn parse_u64_optional(entries: &[KvnEntry], keyword: &str) -> Result<Option<u64>, KvnError> {
    match find_field(entries, keyword)? {
        Some(f) => {
            let v = f.value.trim().parse::<u64>().map_err(|e| KvnError {
                span: Span::default(),
                kind: KvnErrorKind::InvalidValue {
                    keyword: f.key.clone(),
                    reason: e.to_string(),
                },
            })?;
            Ok(Some(v))
        }
        None => Ok(None),
    }
}

/// Classify a keyword into a broad role for the OMM read state-machine.
enum KeywordRole {
    Header,
    Metadata,
    MeanElement,
    Tle,
    Spacecraft,
    UserDefined(String),
    Unknown,
}

fn classify_keyword(key: &str) -> KeywordRole {
    if let Some(suffix) = key.strip_prefix("USER_DEFINED_") {
        return KeywordRole::UserDefined(suffix.to_string());
    }
    match key {
        "CLASSIFICATION" | "CREATION_DATE" | "ORIGINATOR" | "MESSAGE_ID" => KeywordRole::Header,
        "OBJECT_NAME"
        | "OBJECT_ID"
        | "CENTER_NAME"
        | "REF_FRAME"
        | "REF_FRAME_EPOCH"
        | "TIME_SYSTEM"
        | "MEAN_ELEMENT_THEORY" => KeywordRole::Metadata,
        "EPOCH" | "SEMI_MAJOR_AXIS" | "MEAN_MOTION" | "ECCENTRICITY" | "INCLINATION"
        | "RA_OF_ASC_NODE" | "ARG_OF_PERICENTER" | "MEAN_ANOMALY" | "GM" => {
            KeywordRole::MeanElement
        }
        "EPHEMERIS_TYPE"
        | "CLASSIFICATION_TYPE"
        | "NORAD_CAT_ID"
        | "ELEMENT_SET_NO"
        | "REV_AT_EPOCH"
        | "BSTAR"
        | "BTERM"
        | "MEAN_MOTION_DOT"
        | "MEAN_MOTION_DDOT"
        | "AGOM" => KeywordRole::Tle,
        "MASS" | "SOLAR_RAD_AREA" | "SOLAR_RAD_COEFF" | "DRAG_AREA" | "DRAG_COEFF" => {
            KeywordRole::Spacecraft
        }
        _ => KeywordRole::Unknown,
    }
}

/// Collect all flat `KvnEntry` items from all top-level sections (in order),
/// skipping top-level bracketed sections (which become their own sections in
/// the AST), and separately extract the COVARIANCE section.
///
/// The KVN parser promotes bracketed sections like `COVARIANCE_START` /
/// `COVARIANCE_STOP` to top-level `KvnSection`s (with `bracketed = true`)
/// rather than nesting them as `KvnEntry::Subsection`. Non-bracketed
/// (implicit) sections like HEADER and DATA have their entries flattened
/// into `flat`.
fn flatten_entries(doc: &KvnDocument) -> (Vec<&KvnEntry>, Option<&KvnSection>) {
    let mut flat: Vec<&KvnEntry> = Vec::new();
    let mut covariance: Option<&KvnSection> = None;

    for section in &doc.sections {
        if section.bracketed && section.keyword == "COVARIANCE" {
            if covariance.is_none() {
                covariance = Some(section);
            }
            continue;
        }
        if section.bracketed {
            // Other bracketed sections ignored for OMM
            continue;
        }
        // Implicit (non-bracketed) section: flatten its entries.
        for entry in &section.entries {
            match entry {
                KvnEntry::Subsection(sub) if sub.keyword == "COVARIANCE" => {
                    // Also handle the nested-subsection case for robustness.
                    if covariance.is_none() {
                        covariance = Some(sub);
                    }
                }
                KvnEntry::Subsection(_) => {
                    // Other subsections ignored for OMM
                }
                _ => {
                    flat.push(entry);
                }
            }
        }
    }
    (flat, covariance)
}

/// Parse the 6×6 symmetric covariance matrix from the bracketed COVARIANCE
/// section entries.
fn parse_covariance(section: &KvnSection) -> Result<Covariance, KvnError> {
    let entries = &section.entries;

    let mut comments: Vec<String> = Vec::new();
    for entry in entries {
        if let KvnEntry::Comment(c) = entry {
            comments.push(c.clone());
        }
    }

    let frame = parse_string_optional(entries, "COV_REF_FRAME")?.map(|s| OdmFrame::from_wire(&s));

    // 21 lower-triangle fields in canonical order
    let cov_fields: &[(&str, usize, usize)] = &[
        ("CX_X", 0, 0),
        ("CY_X", 1, 0),
        ("CY_Y", 1, 1),
        ("CZ_X", 2, 0),
        ("CZ_Y", 2, 1),
        ("CZ_Z", 2, 2),
        ("CX_DOT_X", 3, 0),
        ("CX_DOT_Y", 3, 1),
        ("CX_DOT_Z", 3, 2),
        ("CX_DOT_X_DOT", 3, 3),
        ("CY_DOT_X", 4, 0),
        ("CY_DOT_Y", 4, 1),
        ("CY_DOT_Z", 4, 2),
        ("CY_DOT_X_DOT", 4, 3),
        ("CY_DOT_Y_DOT", 4, 4),
        ("CZ_DOT_X", 5, 0),
        ("CZ_DOT_Y", 5, 1),
        ("CZ_DOT_Z", 5, 2),
        ("CZ_DOT_X_DOT", 5, 3),
        ("CZ_DOT_Y_DOT", 5, 4),
        ("CZ_DOT_Z_DOT", 5, 5),
    ];

    let mut matrix = nalgebra::Matrix6::<f64>::zeros();
    for &(name, row, col) in cov_fields {
        let f = require_field(entries, name)?;
        let v = parse_f64(f)?;
        matrix[(row, col)] = v;
        if row != col {
            matrix[(col, row)] = v;
        }
    }

    Ok(Covariance {
        comments,
        frame,
        matrix,
    })
}

impl TryFrom<KvnDocument> for Omm {
    type Error = KvnError;

    fn try_from(doc: KvnDocument) -> Result<Self, Self::Error> {
        // 1. Validate message kind.
        if doc.message_kind != MessageKind::Omm {
            return Err(KvnError {
                span: Span::default(),
                kind: KvnErrorKind::UnexpectedKeyword(format!("{}", doc.message_kind)),
            });
        }

        // 2. Flatten entries and extract COVARIANCE subsection.
        let (flat, covariance_section) = flatten_entries(&doc);

        // 3. Walk the flat list with a state machine to accumulate fields by role.
        let mut header_entries: Vec<KvnEntry> = Vec::new();
        let mut metadata_entries: Vec<KvnEntry> = Vec::new();
        let mut mean_element_entries: Vec<KvnEntry> = Vec::new();
        let mut tle_entries: Vec<KvnEntry> = Vec::new();
        let mut spacecraft_entries: Vec<KvnEntry> = Vec::new();
        let mut user_defined: BTreeMap<String, String> = BTreeMap::new();

        // Comment routing: pending comments go to the first field role that fires after them.
        let mut pending_comments: Vec<String> = Vec::new();
        let mut header_comments: Vec<String> = Vec::new();
        let mut metadata_comments: Vec<String> = Vec::new();
        let mut mean_element_comments: Vec<String> = Vec::new();
        let mut tle_comments: Vec<String> = Vec::new();
        let mut spacecraft_comments: Vec<String> = Vec::new();

        for entry in &flat {
            match entry {
                KvnEntry::Comment(text) => {
                    pending_comments.push(text.clone());
                }
                KvnEntry::Field(field) => {
                    let role = classify_keyword(&field.key);
                    match role {
                        KeywordRole::Header => {
                            header_comments.append(&mut pending_comments);
                            header_entries.push(KvnEntry::Field(field.clone()));
                        }
                        KeywordRole::Metadata => {
                            metadata_comments.append(&mut pending_comments);
                            metadata_entries.push(KvnEntry::Field(field.clone()));
                        }
                        KeywordRole::MeanElement => {
                            mean_element_comments.append(&mut pending_comments);
                            mean_element_entries.push(KvnEntry::Field(field.clone()));
                        }
                        KeywordRole::Tle => {
                            tle_comments.append(&mut pending_comments);
                            tle_entries.push(KvnEntry::Field(field.clone()));
                        }
                        KeywordRole::Spacecraft => {
                            spacecraft_comments.append(&mut pending_comments);
                            spacecraft_entries.push(KvnEntry::Field(field.clone()));
                        }
                        KeywordRole::UserDefined(suffix) => {
                            pending_comments.clear();
                            user_defined.insert(suffix, field.value.trim().to_string());
                        }
                        KeywordRole::Unknown => {
                            return Err(KvnError {
                                span: Span::default(),
                                kind: KvnErrorKind::UnexpectedKeyword(field.key.clone()),
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        // 4. Parse required header fields.
        // TIME_SYSTEM is in metadata but needed early for epoch parsing.
        let time_system = parse_string_required(&metadata_entries, "TIME_SYSTEM")?;

        let creation_date_field = require_field(&header_entries, "CREATION_DATE")?;
        let creation_date = parse_epoch(creation_date_field, &time_system)?;
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

        // 5. Parse metadata fields.
        let object_name = parse_string_required(&metadata_entries, "OBJECT_NAME")?;
        let object_id = parse_string_required(&metadata_entries, "OBJECT_ID")?;
        let center_name = parse_string_required(&metadata_entries, "CENTER_NAME")?;
        let center = OdmCenter::from_wire(&center_name);
        let ref_frame = parse_string_required(&metadata_entries, "REF_FRAME")?;
        let frame = OdmFrame::from_wire(&ref_frame);
        let frame_epoch = match parse_string_optional(&metadata_entries, "REF_FRAME_EPOCH")? {
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
        let mean_element_theory = parse_string_required(&metadata_entries, "MEAN_ELEMENT_THEORY")?;

        let metadata = OmmMetadata {
            comments: metadata_comments,
            object_name,
            object_id,
            center,
            frame,
            frame_epoch,
            mean_element_theory,
        };

        // 6. Parse epoch.
        let epoch_field = require_field(&mean_element_entries, "EPOCH")?;
        let epoch = parse_epoch(epoch_field, &time_system)?;

        // 7. Parse mean elements.
        //    Wire GM (optional) is needed for MEAN_MOTION → SMA conversion.
        let wire_gm_km3 = parse_f64_optional(&mean_element_entries, "GM")?;
        let wire_gm = wire_gm_km3.map(GravitationalParameter::km3_per_s2);

        // Resolve GM for MEAN_MOTION conversion:
        //  1. Wire GM (preferred)
        //  2. Canonical body GM from center
        //  3. Error if neither available
        let resolve_gm_for_mean_motion = || -> Result<f64, KvnError> {
            if let Some(gm) = wire_gm {
                return Ok(gm.as_f64());
            }
            metadata
                .center
                .known()
                .and_then(|o| {
                    use lox_bodies::TryPointMass;
                    o.try_gravitational_parameter().ok()
                })
                .map(|gm| gm.as_f64())
                .ok_or_else(|| KvnError {
                    span: Span::default(),
                    kind: KvnErrorKind::MissingRequiredField("GM".to_string()),
                })
        };

        // Prefer SEMI_MAJOR_AXIS; fall back to MEAN_MOTION with GM conversion.
        let sma_km = parse_f64_optional(&mean_element_entries, "SEMI_MAJOR_AXIS")?;
        let a_m =
            if let Some(km) = sma_km {
                km * 1000.0
            } else {
                // MEAN_MOTION in rev/day → SMA in meters via Kepler's third law.
                let mm_rev_day = parse_f64_optional(&mean_element_entries, "MEAN_MOTION")?
                    .ok_or_else(|| KvnError {
                        span: Span::default(),
                        kind: KvnErrorKind::MissingRequiredField("SEMI_MAJOR_AXIS".to_string()),
                    })?;
                let mu = resolve_gm_for_mean_motion()?;
                let n = mm_rev_day * 2.0 * PI / 86400.0; // rad/s
                (mu / (n * n)).cbrt()
            };

        let ecc =
            parse_f64_optional(&mean_element_entries, "ECCENTRICITY")?.ok_or_else(|| KvnError {
                span: Span::default(),
                kind: KvnErrorKind::MissingRequiredField("ECCENTRICITY".to_string()),
            })?;
        let inc_deg =
            parse_f64_optional(&mean_element_entries, "INCLINATION")?.ok_or_else(|| KvnError {
                span: Span::default(),
                kind: KvnErrorKind::MissingRequiredField("INCLINATION".to_string()),
            })?;
        let raan_deg =
            parse_f64_optional(&mean_element_entries, "RA_OF_ASC_NODE")?.ok_or_else(|| {
                KvnError {
                    span: Span::default(),
                    kind: KvnErrorKind::MissingRequiredField("RA_OF_ASC_NODE".to_string()),
                }
            })?;
        let aop_deg =
            parse_f64_optional(&mean_element_entries, "ARG_OF_PERICENTER")?.ok_or_else(|| {
                KvnError {
                    span: Span::default(),
                    kind: KvnErrorKind::MissingRequiredField("ARG_OF_PERICENTER".to_string()),
                }
            })?;
        let ma_deg =
            parse_f64_optional(&mean_element_entries, "MEAN_ANOMALY")?.ok_or_else(|| KvnError {
                span: Span::default(),
                kind: KvnErrorKind::MissingRequiredField("MEAN_ANOMALY".to_string()),
            })?;

        let elements = MeanElements {
            a: a_m,
            e: ecc,
            i: inc_deg.to_radians(),
            raan: raan_deg.to_radians(),
            aop: aop_deg.to_radians(),
            m: ma_deg.to_radians(),
        };

        let mean_elements = OmmMeanElements {
            comments: mean_element_comments,
            elements,
            gm: wire_gm,
        };

        // 8. Parse optional TLE parameters.
        let tle_parameters = if tle_entries.is_empty() {
            None
        } else {
            let ephemeris_type = parse_i32_optional(&tle_entries, "EPHEMERIS_TYPE")?;
            let classification_type = parse_string_optional(&tle_entries, "CLASSIFICATION_TYPE")?;
            let norad_cat_id = parse_i32_optional(&tle_entries, "NORAD_CAT_ID")?;
            let element_set_no = parse_i64_optional(&tle_entries, "ELEMENT_SET_NO")?;
            let rev_at_epoch = parse_u64_optional(&tle_entries, "REV_AT_EPOCH")?;
            let bstar = parse_f64_optional(&tle_entries, "BSTAR")?;
            let bterm = parse_f64_optional(&tle_entries, "BTERM")?
                .map(AreaToMass::square_meters_per_kilogram);
            let mean_motion_dot = parse_f64_optional(&tle_entries, "MEAN_MOTION_DOT")?;
            let mean_motion_ddot = parse_f64_optional(&tle_entries, "MEAN_MOTION_DDOT")?;
            let agom = parse_f64_optional(&tle_entries, "AGOM")?
                .map(AreaToMass::square_meters_per_kilogram);
            Some(TleParameters {
                comments: tle_comments,
                ephemeris_type,
                classification_type,
                norad_cat_id,
                element_set_no,
                rev_at_epoch,
                bstar,
                bterm,
                mean_motion_dot,
                mean_motion_ddot,
                agom,
            })
        };

        // 9. Parse optional spacecraft parameters.
        let spacecraft = if spacecraft_entries.is_empty() {
            None
        } else {
            let mass = parse_f64_optional(&spacecraft_entries, "MASS")?.map(Mass::kilograms);
            let solar_rad_area =
                parse_f64_optional(&spacecraft_entries, "SOLAR_RAD_AREA")?.map(Area::square_meters);
            let solar_rad_coeff = parse_f64_optional(&spacecraft_entries, "SOLAR_RAD_COEFF")?;
            let drag_area =
                parse_f64_optional(&spacecraft_entries, "DRAG_AREA")?.map(Area::square_meters);
            let drag_coeff = parse_f64_optional(&spacecraft_entries, "DRAG_COEFF")?;
            Some(SpacecraftParameters {
                comments: spacecraft_comments,
                mass,
                solar_rad_area,
                solar_rad_coeff,
                drag_area,
                drag_coeff,
            })
        };

        // 10. Parse optional covariance.
        let covariance = match covariance_section {
            Some(sec) => Some(parse_covariance(sec)?),
            None => None,
        };

        Ok(Omm {
            header,
            metadata,
            epoch,
            mean_elements,
            tle_parameters,
            spacecraft,
            covariance,
            user_defined,
        })
    }
}

/// Parses a KVN-formatted string into a typed [`Omm`].
pub fn read_omm(input: &str) -> Result<Omm, KvnError> {
    let doc = parse(input)?;
    Omm::try_from(doc)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::f64::consts::PI;

    use lox_bodies::DynOrigin;
    use lox_core::elements::{GravitationalParameter, MeanElements};
    use lox_core::units::{Area, AreaToMass, Mass};
    use lox_frames::DynFrame;
    use nalgebra::Matrix6;

    use crate::kvn::error::KvnErrorKind;
    use crate::types::common::{Covariance, OdmCenter, OdmFrame, OdmHeader, OdmTime};
    use crate::types::omm::{Omm, OmmMeanElements, OmmMetadata, TleParameters};

    use super::*;

    fn sample_epoch() -> OdmTime {
        OdmTime::Time(lox_time::time::Time::j2000(
            lox_time::time_scales::DynTimeScale::Tai,
        ))
    }

    fn sample_omm() -> Omm {
        let epoch = sample_epoch();
        Omm {
            header: OdmHeader {
                comments: Vec::new(),
                classification: None,
                creation_date: epoch,
                originator: "TEST".to_string(),
                message_id: None,
            },
            metadata: OmmMetadata {
                comments: Vec::new(),
                object_name: "TEST-SAT".to_string(),
                object_id: "2024-000A".to_string(),
                center: OdmCenter::Known(DynOrigin::Earth),
                frame: OdmFrame::Known(DynFrame::Teme),
                frame_epoch: None,
                mean_element_theory: "SGP/SGP4".to_string(),
            },
            epoch,
            mean_elements: OmmMeanElements {
                comments: Vec::new(),
                elements: MeanElements {
                    a: 6_859_961.0,
                    e: 0.001_335_6,
                    i: 1.697_775,
                    raan: 1.159_523,
                    aop: 1.931_018,
                    m: 5.842_034,
                },
                gm: None,
            },
            tle_parameters: None,
            spacecraft: None,
            covariance: None,
            user_defined: BTreeMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Write direction tests
    // -----------------------------------------------------------------------

    #[test]
    fn write_minimal_omm_contains_expected_fields() {
        let omm = sample_omm();
        let output = write_omm(&omm);

        assert!(
            output.contains("CCSDS_OMM_VERS = 3.0"),
            "missing version line; got:\n{output}"
        );
        assert!(
            output.contains("ORIGINATOR = TEST"),
            "missing ORIGINATOR; got:\n{output}"
        );
        assert!(
            output.contains("OBJECT_NAME = TEST-SAT"),
            "missing OBJECT_NAME; got:\n{output}"
        );
        assert!(
            output.contains("OBJECT_ID = 2024-000A"),
            "missing OBJECT_ID; got:\n{output}"
        );
        assert!(
            output.contains("CENTER_NAME = EARTH"),
            "missing CENTER_NAME; got:\n{output}"
        );
        assert!(
            output.contains("TIME_SYSTEM = TAI"),
            "missing TIME_SYSTEM; got:\n{output}"
        );
        assert!(
            output.contains("MEAN_ELEMENT_THEORY = SGP/SGP4"),
            "missing MEAN_ELEMENT_THEORY; got:\n{output}"
        );
        assert!(output.contains("EPOCH = "), "missing EPOCH; got:\n{output}");
        assert!(
            output.contains("SEMI_MAJOR_AXIS ="),
            "missing SEMI_MAJOR_AXIS; got:\n{output}"
        );
        assert!(
            output.contains("ECCENTRICITY ="),
            "missing ECCENTRICITY; got:\n{output}"
        );
        assert!(
            output.contains("INCLINATION ="),
            "missing INCLINATION; got:\n{output}"
        );
        assert!(
            output.contains("RA_OF_ASC_NODE ="),
            "missing RA_OF_ASC_NODE; got:\n{output}"
        );
        assert!(
            output.contains("ARG_OF_PERICENTER ="),
            "missing ARG_OF_PERICENTER; got:\n{output}"
        );
        assert!(
            output.contains("MEAN_ANOMALY ="),
            "missing MEAN_ANOMALY; got:\n{output}"
        );
        // No TLE section, so these must be absent
        assert!(
            !output.contains("EPHEMERIS_TYPE"),
            "unexpected EPHEMERIS_TYPE; got:\n{output}"
        );
        assert!(
            !output.contains("GM ="),
            "unexpected GM field; got:\n{output}"
        );
    }

    #[test]
    fn write_omm_with_tle_parameters() {
        let mut omm = sample_omm();
        omm.tle_parameters = Some(TleParameters {
            comments: Vec::new(),
            ephemeris_type: Some(0),
            classification_type: Some("U".to_string()),
            norad_cat_id: Some(45018),
            element_set_no: Some(999),
            rev_at_epoch: Some(5327),
            bstar: Some(8.4553e-5),
            bterm: None,
            mean_motion_dot: Some(2.241e-5),
            mean_motion_ddot: Some(0.0),
            agom: None,
        });

        let output = write_omm(&omm);

        assert!(
            output.contains("EPHEMERIS_TYPE = 0"),
            "missing EPHEMERIS_TYPE; got:\n{output}"
        );
        assert!(
            output.contains("CLASSIFICATION_TYPE = U"),
            "missing CLASSIFICATION_TYPE; got:\n{output}"
        );
        assert!(
            output.contains("NORAD_CAT_ID = 45018"),
            "missing NORAD_CAT_ID; got:\n{output}"
        );
        assert!(output.contains("BSTAR ="), "missing BSTAR; got:\n{output}");
        assert!(
            output.contains("MEAN_MOTION_DOT ="),
            "missing MEAN_MOTION_DOT; got:\n{output}"
        );
    }

    #[test]
    fn write_omm_with_covariance() {
        let mut omm = sample_omm();
        omm.covariance = Some(Covariance {
            comments: Vec::new(),
            frame: None,
            matrix: Matrix6::identity(),
        });

        let output = write_omm(&omm);

        assert!(
            output.contains("COVARIANCE_START"),
            "missing COVARIANCE_START; got:\n{output}"
        );
        assert!(
            output.contains("COVARIANCE_STOP"),
            "missing COVARIANCE_STOP; got:\n{output}"
        );
        assert!(output.contains("CX_X = 1"), "missing CX_X; got:\n{output}");
        assert!(output.contains("CY_X = 0"), "missing CY_X; got:\n{output}");
    }

    #[test]
    fn write_omm_with_gm_emits_gm_field() {
        let mut omm = sample_omm();
        omm.mean_elements.gm = Some(GravitationalParameter::km3_per_s2(398600.4415));

        let output = write_omm(&omm);

        assert!(
            output.contains("GM =") && output.contains("[km**3/s**2]"),
            "missing GM field; got:\n{output}"
        );
        assert!(
            output.contains("398600.4415"),
            "wrong GM value; got:\n{output}"
        );
    }

    #[test]
    fn write_omm_with_bterm_and_agom() {
        let mut omm = sample_omm();
        omm.tle_parameters = Some(TleParameters {
            bterm: Some(AreaToMass::square_meters_per_kilogram(0.05)),
            agom: Some(AreaToMass::square_meters_per_kilogram(0.03)),
            ..TleParameters::default()
        });

        let output = write_omm(&omm);

        assert!(
            output.contains("BTERM = 0.05 [m**2/kg]"),
            "missing BTERM; got:\n{output}"
        );
        assert!(
            output.contains("AGOM = 0.03 [m**2/kg]"),
            "missing AGOM; got:\n{output}"
        );
    }

    #[test]
    fn write_omm_with_user_defined() {
        let mut omm = sample_omm();
        omm.user_defined
            .insert("OPERATOR".to_string(), "GSOC".to_string());

        let output = write_omm(&omm);

        assert!(
            output.contains("USER_DEFINED_OPERATOR = GSOC"),
            "missing USER_DEFINED_OPERATOR; got:\n{output}"
        );
    }

    // -----------------------------------------------------------------------
    // Round-trip tests
    // -----------------------------------------------------------------------

    #[test]
    fn round_trip_minimal_omm() {
        let omm = sample_omm();
        let written = write_omm(&omm);
        let parsed = read_omm(&written).expect("parse failed");
        // Compare key fields; floating-point round-trip via degrees may have tiny drift.
        assert_eq!(parsed.header, omm.header, "header mismatch");
        assert_eq!(parsed.metadata, omm.metadata, "metadata mismatch");
        assert_eq!(parsed.mean_elements.gm, omm.mean_elements.gm, "gm mismatch");
        let eps = 1e-9_f64;
        assert!(
            (parsed.mean_elements.elements.a - omm.mean_elements.elements.a).abs() < 1.0,
            "SMA round-trip error too large"
        );
        assert!(
            (parsed.mean_elements.elements.e - omm.mean_elements.elements.e).abs() < eps,
            "eccentricity round-trip error"
        );
    }

    #[test]
    fn round_trip_omm_with_tle_parameters() {
        let mut omm = sample_omm();
        omm.tle_parameters = Some(TleParameters {
            comments: Vec::new(),
            ephemeris_type: Some(0),
            classification_type: Some("U".to_string()),
            norad_cat_id: Some(45018),
            element_set_no: Some(999),
            rev_at_epoch: Some(5327),
            bstar: Some(8.4553e-5),
            bterm: None,
            mean_motion_dot: Some(2.241e-5),
            mean_motion_ddot: Some(0.0),
            agom: None,
        });

        let written = write_omm(&omm);
        let parsed = read_omm(&written).expect("parse failed");

        let tle = parsed.tle_parameters.expect("missing TLE parameters");
        assert_eq!(tle.ephemeris_type, Some(0));
        assert_eq!(tle.classification_type.as_deref(), Some("U"));
        assert_eq!(tle.norad_cat_id, Some(45018));
        assert_eq!(tle.element_set_no, Some(999));
        assert_eq!(tle.rev_at_epoch, Some(5327));
        assert!((tle.bstar.unwrap() - 8.4553e-5).abs() < 1e-10);
    }

    #[test]
    fn round_trip_omm_with_covariance() {
        let mut omm = sample_omm();
        omm.covariance = Some(Covariance {
            comments: Vec::new(),
            frame: None,
            matrix: Matrix6::identity(),
        });

        let written = write_omm(&omm);
        let parsed = read_omm(&written).expect("parse failed");

        let cov = parsed.covariance.expect("missing covariance");
        assert_eq!(cov.matrix, Matrix6::identity());
    }

    // -----------------------------------------------------------------------
    // MEAN_MOTION → SMA conversion tests
    // -----------------------------------------------------------------------

    fn minimal_omm_kvn_with_mean_motion(extra_fields: &str) -> String {
        format!(
            "CCSDS_OMM_VERS = 3.0\n\
             CREATION_DATE = 2024-01-01T00:00:00\n\
             ORIGINATOR = TEST\n\
             OBJECT_NAME = TEST-SAT\n\
             OBJECT_ID = 2024-000A\n\
             CENTER_NAME = EARTH\n\
             REF_FRAME = TEME\n\
             TIME_SYSTEM = TAI\n\
             MEAN_ELEMENT_THEORY = SGP/SGP4\n\
             EPOCH = 2024-01-01T00:00:00\n\
             {extra_fields}\
             MEAN_MOTION = 15.5 [rev/day]\n\
             ECCENTRICITY = 0.001\n\
             INCLINATION = 45.0 [deg]\n\
             RA_OF_ASC_NODE = 0.0 [deg]\n\
             ARG_OF_PERICENTER = 0.0 [deg]\n\
             MEAN_ANOMALY = 0.0 [deg]\n"
        )
    }

    #[test]
    fn read_mean_motion_with_wire_gm_computes_sma() {
        // MEAN_MOTION = 15.5 rev/day, GM = 398600.4415 km³/s²
        // n = 15.5 * 2π / 86400 = 1.12737... × 10⁻³ rad/s
        // a = (mu/n²)^(1/3)
        let kvn = minimal_omm_kvn_with_mean_motion("GM = 398600.4415 [km**3/s**2]\n");
        let parsed = read_omm(&kvn).expect("parse failed");

        let mu = 398600.4415e9_f64;
        let n = 15.5 * 2.0 * PI / 86400.0;
        let expected_a_m = (mu / (n * n)).cbrt();

        let diff = (parsed.mean_elements.elements.a - expected_a_m).abs();
        assert!(
            diff < 1.0,
            "SMA mismatch: expected {expected_a_m:.3} m, got {:.3} m (diff {diff:.3} m)",
            parsed.mean_elements.elements.a
        );
        // Wire GM should be preserved
        assert!(parsed.mean_elements.gm.is_some());
    }

    #[test]
    fn read_mean_motion_without_wire_gm_uses_canonical_earth_gm() {
        // No GM on wire → use Earth's canonical GM
        let kvn = minimal_omm_kvn_with_mean_motion("");
        let parsed = read_omm(&kvn).expect("parse failed");
        // Just check it parsed successfully and gives a plausible SMA
        assert!(
            parsed.mean_elements.elements.a > 6_000_000.0,
            "SMA unrealistically small: {}",
            parsed.mean_elements.elements.a
        );
        assert!(parsed.mean_elements.gm.is_none());
    }

    #[test]
    fn read_mean_motion_without_wire_gm_and_custom_center_errors() {
        let kvn = "CCSDS_OMM_VERS = 3.0\n\
                   CREATION_DATE = 2024-01-01T00:00:00\n\
                   ORIGINATOR = TEST\n\
                   OBJECT_NAME = TEST-SAT\n\
                   OBJECT_ID = 2024-000A\n\
                   CENTER_NAME = APOPHIS\n\
                   REF_FRAME = TEME\n\
                   TIME_SYSTEM = TAI\n\
                   MEAN_ELEMENT_THEORY = SGP/SGP4\n\
                   EPOCH = 2024-01-01T00:00:00\n\
                   MEAN_MOTION = 15.5 [rev/day]\n\
                   ECCENTRICITY = 0.001\n\
                   INCLINATION = 45.0 [deg]\n\
                   RA_OF_ASC_NODE = 0.0 [deg]\n\
                   ARG_OF_PERICENTER = 0.0 [deg]\n\
                   MEAN_ANOMALY = 0.0 [deg]\n";

        let err = read_omm(kvn).expect_err("should fail without GM for custom center");
        assert!(
            matches!(err.kind, KvnErrorKind::MissingRequiredField(ref k) if k == "GM"),
            "unexpected error: {err}"
        );
    }

    // -----------------------------------------------------------------------
    // Error handling tests
    // -----------------------------------------------------------------------

    fn minimal_omm_kvn() -> String {
        "CCSDS_OMM_VERS = 3.0\n\
         CREATION_DATE = 2024-01-01T00:00:00\n\
         ORIGINATOR = TEST\n\
         OBJECT_NAME = TEST-SAT\n\
         OBJECT_ID = 2024-000A\n\
         CENTER_NAME = EARTH\n\
         REF_FRAME = TEME\n\
         TIME_SYSTEM = TAI\n\
         MEAN_ELEMENT_THEORY = SGP/SGP4\n\
         EPOCH = 2024-01-01T00:00:00\n\
         SEMI_MAJOR_AXIS = 6860.0 [km]\n\
         ECCENTRICITY = 0.001\n\
         INCLINATION = 45.0 [deg]\n\
         RA_OF_ASC_NODE = 0.0 [deg]\n\
         ARG_OF_PERICENTER = 0.0 [deg]\n\
         MEAN_ANOMALY = 0.0 [deg]\n"
            .to_string()
    }

    #[test]
    fn read_missing_object_name_returns_error() {
        let kvn = "CCSDS_OMM_VERS = 3.0\n\
                   CREATION_DATE = 2024-01-01T00:00:00\n\
                   ORIGINATOR = TEST\n\
                   OBJECT_ID = 2024-000A\n\
                   CENTER_NAME = EARTH\n\
                   REF_FRAME = TEME\n\
                   TIME_SYSTEM = TAI\n\
                   MEAN_ELEMENT_THEORY = SGP/SGP4\n\
                   EPOCH = 2024-01-01T00:00:00\n\
                   SEMI_MAJOR_AXIS = 6860.0 [km]\n\
                   ECCENTRICITY = 0.001\n\
                   INCLINATION = 45.0 [deg]\n\
                   RA_OF_ASC_NODE = 0.0 [deg]\n\
                   ARG_OF_PERICENTER = 0.0 [deg]\n\
                   MEAN_ANOMALY = 0.0 [deg]\n";

        let err = read_omm(kvn).expect_err("should fail on missing OBJECT_NAME");
        assert!(
            matches!(err.kind, KvnErrorKind::MissingRequiredField(ref k) if k == "OBJECT_NAME"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_unknown_keyword_returns_error() {
        let mut kvn = minimal_omm_kvn();
        kvn.push_str("FOO = bar\n");

        let err = read_omm(&kvn).expect_err("should fail on unknown keyword FOO");
        assert!(
            matches!(err.kind, KvnErrorKind::UnexpectedKeyword(ref k) if k == "FOO"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_user_defined_preserved() {
        let mut kvn = minimal_omm_kvn();
        kvn.push_str("USER_DEFINED_OPERATOR = GSOC\n");

        let parsed = read_omm(&kvn).expect("parse failed");
        assert_eq!(
            parsed.user_defined.get("OPERATOR"),
            Some(&"GSOC".to_string()),
            "USER_DEFINED_OPERATOR not preserved"
        );
    }

    #[test]
    fn write_omm_with_spacecraft_parameters() {
        let mut omm = sample_omm();
        omm.spacecraft = Some(SpacecraftParameters {
            comments: Vec::new(),
            mass: Some(Mass::kilograms(120.0)),
            solar_rad_area: Some(Area::square_meters(2.0)),
            solar_rad_coeff: Some(1.2),
            drag_area: Some(Area::square_meters(1.5)),
            drag_coeff: Some(2.2),
        });

        let output = write_omm(&omm);

        assert!(
            output.contains("MASS = 120 [kg]"),
            "missing MASS; got:\n{output}"
        );
        assert!(
            output.contains("SOLAR_RAD_AREA = 2 [m**2]"),
            "missing SOLAR_RAD_AREA; got:\n{output}"
        );
        assert!(
            output.contains("DRAG_COEFF = 2.2"),
            "missing DRAG_COEFF; got:\n{output}"
        );
    }

    // -----------------------------------------------------------------------
    // Additional error-path and branch tests
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
        let err = read_omm(kvn).expect_err("should fail on wrong message kind");
        assert!(
            matches!(err.kind, KvnErrorKind::UnexpectedKeyword(_)),
            "unexpected error kind: {err}"
        );
    }

    #[test]
    fn read_omm_unknown_keyword_returns_error() {
        let mut kvn = minimal_omm_kvn();
        kvn.push_str("UNKNOWN_FIELD = foo\n");

        let err = read_omm(&kvn).expect_err("should fail on unknown keyword");
        assert!(
            matches!(err.kind, KvnErrorKind::UnexpectedKeyword(ref k) if k == "UNKNOWN_FIELD"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_omm_duplicate_eccentricity_returns_error() {
        let kvn = "CCSDS_OMM_VERS = 3.0\n\
                   CREATION_DATE = 2024-01-01T00:00:00\n\
                   ORIGINATOR = TEST\n\
                   OBJECT_NAME = TEST-SAT\n\
                   OBJECT_ID = 2024-000A\n\
                   CENTER_NAME = EARTH\n\
                   REF_FRAME = TEME\n\
                   TIME_SYSTEM = TAI\n\
                   MEAN_ELEMENT_THEORY = SGP/SGP4\n\
                   EPOCH = 2024-01-01T00:00:00\n\
                   SEMI_MAJOR_AXIS = 6860.0 [km]\n\
                   ECCENTRICITY = 0.001\n\
                   ECCENTRICITY = 0.002\n\
                   INCLINATION = 45.0 [deg]\n\
                   RA_OF_ASC_NODE = 0.0 [deg]\n\
                   ARG_OF_PERICENTER = 0.0 [deg]\n\
                   MEAN_ANOMALY = 0.0 [deg]\n";

        let err = read_omm(kvn).expect_err("should fail on duplicate ECCENTRICITY");
        assert!(
            matches!(err.kind, KvnErrorKind::DuplicateField(ref k) if k == "ECCENTRICITY"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_omm_missing_both_sma_and_mean_motion_returns_error() {
        let kvn = "CCSDS_OMM_VERS = 3.0\n\
                   CREATION_DATE = 2024-01-01T00:00:00\n\
                   ORIGINATOR = TEST\n\
                   OBJECT_NAME = TEST-SAT\n\
                   OBJECT_ID = 2024-000A\n\
                   CENTER_NAME = EARTH\n\
                   REF_FRAME = TEME\n\
                   TIME_SYSTEM = TAI\n\
                   MEAN_ELEMENT_THEORY = SGP/SGP4\n\
                   EPOCH = 2024-01-01T00:00:00\n\
                   ECCENTRICITY = 0.001\n\
                   INCLINATION = 45.0 [deg]\n\
                   RA_OF_ASC_NODE = 0.0 [deg]\n\
                   ARG_OF_PERICENTER = 0.0 [deg]\n\
                   MEAN_ANOMALY = 0.0 [deg]\n";

        let err = read_omm(kvn).expect_err("should fail without SMA or MEAN_MOTION");
        assert!(
            matches!(err.kind, KvnErrorKind::MissingRequiredField(ref k) if k == "SEMI_MAJOR_AXIS"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_omm_tle_invalid_integer_returns_error() {
        let mut kvn = minimal_omm_kvn();
        kvn.push_str("EPHEMERIS_TYPE = not-a-number\n");

        let err = read_omm(&kvn).expect_err("should fail on invalid EPHEMERIS_TYPE");
        assert!(
            matches!(err.kind, KvnErrorKind::InvalidValue { ref keyword, .. } if keyword == "EPHEMERIS_TYPE"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_omm_tle_invalid_i64_returns_error() {
        let mut kvn = minimal_omm_kvn();
        kvn.push_str("ELEMENT_SET_NO = not-an-integer\n");

        let err = read_omm(&kvn).expect_err("should fail on invalid ELEMENT_SET_NO");
        assert!(
            matches!(err.kind, KvnErrorKind::InvalidValue { ref keyword, .. } if keyword == "ELEMENT_SET_NO"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_omm_tle_invalid_u64_returns_error() {
        let mut kvn = minimal_omm_kvn();
        kvn.push_str("REV_AT_EPOCH = not-a-u64\n");

        let err = read_omm(&kvn).expect_err("should fail on invalid REV_AT_EPOCH");
        assert!(
            matches!(err.kind, KvnErrorKind::InvalidValue { ref keyword, .. } if keyword == "REV_AT_EPOCH"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn round_trip_omm_with_spacecraft_and_user_defined() {
        let mut omm = sample_omm();
        omm.spacecraft = Some(SpacecraftParameters {
            comments: Vec::new(),
            mass: Some(Mass::kilograms(150.0)),
            solar_rad_area: None,
            solar_rad_coeff: None,
            drag_area: None,
            drag_coeff: None,
        });
        omm.user_defined
            .insert("OPERATOR".to_string(), "GSOC".to_string());

        let written = write_omm(&omm);
        let parsed = read_omm(&written).expect("parse failed");
        let sp = parsed.spacecraft.expect("spacecraft block should survive");
        assert!((sp.mass.unwrap().to_kilograms() - 150.0).abs() < 1e-9);
        assert_eq!(
            parsed.user_defined.get("OPERATOR"),
            Some(&"GSOC".to_string())
        );
    }

    #[test]
    fn read_omm_comments_in_tle_block_routed_correctly() {
        let kvn = "CCSDS_OMM_VERS = 3.0\n\
                   CREATION_DATE = 2024-01-01T00:00:00\n\
                   ORIGINATOR = TEST\n\
                   OBJECT_NAME = TEST-SAT\n\
                   OBJECT_ID = 2024-000A\n\
                   CENTER_NAME = EARTH\n\
                   REF_FRAME = TEME\n\
                   TIME_SYSTEM = TAI\n\
                   MEAN_ELEMENT_THEORY = SGP/SGP4\n\
                   EPOCH = 2024-01-01T00:00:00\n\
                   SEMI_MAJOR_AXIS = 6860.0 [km]\n\
                   ECCENTRICITY = 0.001\n\
                   INCLINATION = 45.0 [deg]\n\
                   RA_OF_ASC_NODE = 0.0 [deg]\n\
                   ARG_OF_PERICENTER = 0.0 [deg]\n\
                   MEAN_ANOMALY = 0.0 [deg]\n\
                   COMMENT TLE data comment\n\
                   EPHEMERIS_TYPE = 0\n";

        let parsed = read_omm(kvn).expect("parse failed");
        let tle = parsed
            .tle_parameters
            .expect("TLE parameters should be present");
        assert_eq!(tle.ephemeris_type, Some(0));
        // The comment should be routed to tle_comments
        assert_eq!(tle.comments.len(), 1, "TLE comment count mismatch");
    }

    #[test]
    fn read_omm_missing_mean_element_theory_returns_error() {
        let kvn = "CCSDS_OMM_VERS = 3.0\n\
                   CREATION_DATE = 2024-01-01T00:00:00\n\
                   ORIGINATOR = TEST\n\
                   OBJECT_NAME = TEST-SAT\n\
                   OBJECT_ID = 2024-000A\n\
                   CENTER_NAME = EARTH\n\
                   REF_FRAME = TEME\n\
                   TIME_SYSTEM = TAI\n\
                   EPOCH = 2024-01-01T00:00:00\n\
                   SEMI_MAJOR_AXIS = 6860.0 [km]\n\
                   ECCENTRICITY = 0.001\n\
                   INCLINATION = 45.0 [deg]\n\
                   RA_OF_ASC_NODE = 0.0 [deg]\n\
                   ARG_OF_PERICENTER = 0.0 [deg]\n\
                   MEAN_ANOMALY = 0.0 [deg]\n";

        let err = read_omm(kvn).expect_err("should fail on missing MEAN_ELEMENT_THEORY");
        assert!(
            matches!(err.kind, KvnErrorKind::MissingRequiredField(ref k) if k == "MEAN_ELEMENT_THEORY"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn write_omm_header_message_id_and_classification() {
        let mut omm = sample_omm();
        omm.header.message_id = Some("MSG-001".to_string());
        omm.header.classification = Some("UNCLASSIFIED".to_string());

        let output = write_omm(&omm);
        assert!(
            output.contains("MESSAGE_ID = MSG-001"),
            "missing MESSAGE_ID; got:\n{output}"
        );
        assert!(
            output.contains("CLASSIFICATION = UNCLASSIFIED"),
            "missing CLASSIFICATION; got:\n{output}"
        );
    }

    #[test]
    fn write_omm_metadata_frame_epoch() {
        let mut omm = sample_omm();
        omm.metadata.frame_epoch = Some(sample_epoch());

        let output = write_omm(&omm);
        assert!(
            output.contains("REF_FRAME_EPOCH ="),
            "missing REF_FRAME_EPOCH; got:\n{output}"
        );
    }
}
