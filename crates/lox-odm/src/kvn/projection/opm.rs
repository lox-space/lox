// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! KVN ↔ typed [`Opm`] projection.
//!
//! - [`From<&Opm> for KvnDocument`] emits a canonical AST.
//! - [`TryFrom<KvnDocument> for Opm`] (Task 5) validates and projects
//!   an AST that the [`crate::kvn::parser`] produced.
//! - [`read_opm`] / [`write_opm`] (Task 5/6) are the public free
//!   functions re-exported from [`crate::kvn`].

use std::collections::BTreeMap;
use std::convert::TryFrom;

use lox_core::coords::Cartesian;
use lox_core::elements::{GravitationalParameter, Keplerian};
use lox_core::time::deltas::TimeDelta;
use lox_core::units::{Angle, Area, Distance, Mass, Velocity};

use crate::kvn::ast::{KvnDocument, KvnEntry, KvnField, KvnSection};
use crate::kvn::error::{KvnError, KvnErrorKind, Span};
use crate::kvn::parser::parse;
use crate::kvn::projection::{find_field, parse_epoch, parse_f64, require_field};
use crate::types::common::{
    Covariance, MessageKind, OdmCenter, OdmFrame, OdmHeader, SpacecraftParameters,
};
use crate::types::opm::{Maneuver, Opm, OpmKeplerian, OpmMetadata};

/// Build a single `KEY = VALUE [unit]` entry.
fn fld(key: &str, value: impl ToString, unit: Option<&str>) -> KvnEntry {
    KvnEntry::Field(KvnField {
        key: key.to_string(),
        value: value.to_string(),
        unit: unit.map(|u| u.to_string()),
    })
}

fn build_header_section(opm: &Opm) -> KvnSection {
    let mut entries = Vec::new();

    for comment in &opm.header.comments {
        entries.push(KvnEntry::Comment(comment.clone()));
    }

    if let Some(cls) = &opm.header.classification {
        entries.push(fld("CLASSIFICATION", cls, None));
    }

    entries.push(fld("CREATION_DATE", opm.header.creation_date.iso(), None));
    entries.push(fld("ORIGINATOR", &opm.header.originator, None));

    if let Some(mid) = &opm.header.message_id {
        entries.push(fld("MESSAGE_ID", mid, None));
    }

    KvnSection {
        keyword: "HEADER".to_string(),
        bracketed: false,
        entries,
    }
}

fn build_metadata_section(opm: &Opm) -> KvnSection {
    let mut entries = Vec::new();

    for comment in &opm.metadata.comments {
        entries.push(KvnEntry::Comment(comment.clone()));
    }

    entries.push(fld("OBJECT_NAME", &opm.metadata.object_name, None));
    entries.push(fld("OBJECT_ID", &opm.metadata.object_id, None));
    entries.push(fld("CENTER_NAME", opm.metadata.center.name(), None));
    entries.push(fld("REF_FRAME", opm.metadata.frame.name(), None));

    if let Some(epoch) = &opm.metadata.frame_epoch {
        entries.push(fld("REF_FRAME_EPOCH", epoch.iso(), None));
    }

    entries.push(fld("TIME_SYSTEM", opm.epoch.time_system(), None));

    KvnSection {
        keyword: "METADATA".to_string(),
        bracketed: false,
        entries,
    }
}

fn build_data_section(opm: &Opm) -> KvnSection {
    let mut entries = Vec::new();

    // State-vector comments
    for comment in &opm.state_comments {
        entries.push(KvnEntry::Comment(comment.clone()));
    }

    // State vector fields
    let pos = opm.state.position();
    let vel = opm.state.velocity();

    entries.push(fld("EPOCH", opm.epoch.iso(), None));
    entries.push(fld("X", format!("{}", pos.x / 1000.0), Some("km")));
    entries.push(fld("Y", format!("{}", pos.y / 1000.0), Some("km")));
    entries.push(fld("Z", format!("{}", pos.z / 1000.0), Some("km")));
    entries.push(fld("X_DOT", format!("{}", vel.x / 1000.0), Some("km/s")));
    entries.push(fld("Y_DOT", format!("{}", vel.y / 1000.0), Some("km/s")));
    entries.push(fld("Z_DOT", format!("{}", vel.z / 1000.0), Some("km/s")));

    // Optional Keplerian block
    if let Some(kep) = &opm.keplerian {
        for comment in &kep.comments {
            entries.push(KvnEntry::Comment(comment.clone()));
        }

        entries.push(fld(
            "SEMI_MAJOR_AXIS",
            format!("{}", kep.elements.semi_major_axis().to_kilometers()),
            Some("km"),
        ));
        entries.push(fld(
            "ECCENTRICITY",
            format!("{}", kep.elements.eccentricity().as_f64()),
            None,
        ));
        entries.push(fld(
            "INCLINATION",
            format!("{}", kep.elements.inclination().as_f64().to_degrees()),
            Some("deg"),
        ));
        entries.push(fld(
            "RA_OF_ASC_NODE",
            format!(
                "{}",
                kep.elements
                    .longitude_of_ascending_node()
                    .as_f64()
                    .to_degrees()
            ),
            Some("deg"),
        ));
        entries.push(fld(
            "ARG_OF_PERICENTER",
            format!(
                "{}",
                kep.elements.argument_of_periapsis().as_f64().to_degrees()
            ),
            Some("deg"),
        ));
        entries.push(fld(
            "TRUE_ANOMALY",
            format!("{}", kep.elements.true_anomaly().as_angle().to_degrees()),
            Some("deg"),
        ));

        // Emit GM only when explicitly stored on the wire block
        if let Some(gm) = kep.gm {
            // GravitationalParameter stores m³/s²; wire format is km³/s²
            entries.push(fld(
                "GM",
                format!("{}", gm.as_f64() / 1e9),
                Some("km**3/s**2"),
            ));
        }
    }

    // Optional spacecraft parameters
    if let Some(sp) = &opm.spacecraft {
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
    if let Some(cov) = &opm.covariance {
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

    // Maneuvers
    for maneuver in &opm.maneuvers {
        for comment in &maneuver.comments {
            entries.push(KvnEntry::Comment(comment.clone()));
        }

        entries.push(fld(
            "MAN_EPOCH_IGNITION",
            maneuver.ignition_epoch.iso(),
            None,
        ));
        entries.push(fld(
            "MAN_DURATION",
            format!("{}", maneuver.duration.to_seconds().to_f64()),
            Some("s"),
        ));
        entries.push(fld(
            "MAN_DELTA_MASS",
            format!("{}", maneuver.delta_mass.to_kilograms()),
            Some("kg"),
        ));

        if let Some(frame) = &maneuver.frame {
            entries.push(fld("MAN_REF_FRAME", frame.name(), None));
        }

        entries.push(fld(
            "MAN_DV_1",
            format!("{}", maneuver.delta_v[0].to_kilometers_per_second()),
            Some("km/s"),
        ));
        entries.push(fld(
            "MAN_DV_2",
            format!("{}", maneuver.delta_v[1].to_kilometers_per_second()),
            Some("km/s"),
        ));
        entries.push(fld(
            "MAN_DV_3",
            format!("{}", maneuver.delta_v[2].to_kilometers_per_second()),
            Some("km/s"),
        ));
    }

    // User-defined parameters (BTreeMap iteration is sorted by key)
    for (key, value) in &opm.user_defined {
        entries.push(fld(&format!("USER_DEFINED_{key}"), value, None));
    }

    KvnSection {
        keyword: "DATA".to_string(),
        bracketed: false,
        entries,
    }
}

impl From<&Opm> for KvnDocument {
    fn from(opm: &Opm) -> Self {
        let header_section = build_header_section(opm);
        let metadata_section = build_metadata_section(opm);
        let data_section = build_data_section(opm);

        KvnDocument {
            message_kind: MessageKind::Opm,
            version: "3.0".to_string(),
            preamble: Vec::new(),
            sections: vec![header_section, metadata_section, data_section],
        }
    }
}

/// Serialises an [`Opm`] to its canonical KVN text form.
pub fn write_opm(opm: &Opm) -> String {
    let doc: KvnDocument = opm.into();
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

/// Classify a keyword into a broad role used by the read state-machine.
enum KeywordRole {
    Header,
    Metadata,
    State,
    Keplerian,
    Spacecraft,
    ManeuverStart,
    ManeuverField,
    UserDefined(String),
    Unknown,
}

fn classify_keyword(key: &str) -> KeywordRole {
    if let Some(suffix) = key.strip_prefix("USER_DEFINED_") {
        return KeywordRole::UserDefined(suffix.to_string());
    }
    match key {
        "CLASSIFICATION" | "CREATION_DATE" | "ORIGINATOR" | "MESSAGE_ID" => KeywordRole::Header,
        "OBJECT_NAME" | "OBJECT_ID" | "CENTER_NAME" | "REF_FRAME" | "REF_FRAME_EPOCH"
        | "TIME_SYSTEM" => KeywordRole::Metadata,
        "EPOCH" | "X" | "Y" | "Z" | "X_DOT" | "Y_DOT" | "Z_DOT" => KeywordRole::State,
        "SEMI_MAJOR_AXIS" | "ECCENTRICITY" | "INCLINATION" | "RA_OF_ASC_NODE"
        | "ARG_OF_PERICENTER" | "TRUE_ANOMALY" | "MEAN_ANOMALY" | "GM" => KeywordRole::Keplerian,
        "MASS" | "SOLAR_RAD_AREA" | "SOLAR_RAD_COEFF" | "DRAG_AREA" | "DRAG_COEFF" => {
            KeywordRole::Spacecraft
        }
        "MAN_EPOCH_IGNITION" => KeywordRole::ManeuverStart,
        "MAN_DURATION" | "MAN_DELTA_MASS" | "MAN_REF_FRAME" | "MAN_DV_1" | "MAN_DV_2"
        | "MAN_DV_3" => KeywordRole::ManeuverField,
        _ => KeywordRole::Unknown,
    }
}

/// A partially-assembled maneuver during the read pass.
struct ManeuverBuilder {
    comments: Vec<String>,
    ignition_epoch: Option<KvnField>,
    duration: Option<KvnField>,
    delta_mass: Option<KvnField>,
    frame: Option<String>,
    dv1: Option<KvnField>,
    dv2: Option<KvnField>,
    dv3: Option<KvnField>,
}

impl ManeuverBuilder {
    fn new(pending_comments: Vec<String>, ignition_field: KvnField) -> Self {
        ManeuverBuilder {
            comments: pending_comments,
            ignition_epoch: Some(ignition_field),
            duration: None,
            delta_mass: None,
            frame: None,
            dv1: None,
            dv2: None,
            dv3: None,
        }
    }

    fn set_field(&mut self, key: &str, field: KvnField) -> Result<(), KvnError> {
        match key {
            "MAN_DURATION" => {
                if self.duration.is_some() {
                    return Err(KvnError {
                        span: Span::default(),
                        kind: KvnErrorKind::DuplicateField(key.to_string()),
                    });
                }
                self.duration = Some(field);
            }
            "MAN_DELTA_MASS" => {
                if self.delta_mass.is_some() {
                    return Err(KvnError {
                        span: Span::default(),
                        kind: KvnErrorKind::DuplicateField(key.to_string()),
                    });
                }
                self.delta_mass = Some(field);
            }
            "MAN_REF_FRAME" => {
                if self.frame.is_some() {
                    return Err(KvnError {
                        span: Span::default(),
                        kind: KvnErrorKind::DuplicateField(key.to_string()),
                    });
                }
                self.frame = Some(field.value.trim().to_string());
            }
            "MAN_DV_1" => {
                if self.dv1.is_some() {
                    return Err(KvnError {
                        span: Span::default(),
                        kind: KvnErrorKind::DuplicateField(key.to_string()),
                    });
                }
                self.dv1 = Some(field);
            }
            "MAN_DV_2" => {
                if self.dv2.is_some() {
                    return Err(KvnError {
                        span: Span::default(),
                        kind: KvnErrorKind::DuplicateField(key.to_string()),
                    });
                }
                self.dv2 = Some(field);
            }
            "MAN_DV_3" => {
                if self.dv3.is_some() {
                    return Err(KvnError {
                        span: Span::default(),
                        kind: KvnErrorKind::DuplicateField(key.to_string()),
                    });
                }
                self.dv3 = Some(field);
            }
            _ => {}
        }
        Ok(())
    }

    fn build(self, time_system: &str) -> Result<Maneuver, KvnError> {
        let ign_field = self.ignition_epoch.ok_or_else(|| KvnError {
            span: Span::default(),
            kind: KvnErrorKind::MissingRequiredField("MAN_EPOCH_IGNITION".to_string()),
        })?;
        let dur_field = self.duration.ok_or_else(|| KvnError {
            span: Span::default(),
            kind: KvnErrorKind::MissingRequiredField("MAN_DURATION".to_string()),
        })?;
        let dm_field = self.delta_mass.ok_or_else(|| KvnError {
            span: Span::default(),
            kind: KvnErrorKind::MissingRequiredField("MAN_DELTA_MASS".to_string()),
        })?;
        let dv1_field = self.dv1.ok_or_else(|| KvnError {
            span: Span::default(),
            kind: KvnErrorKind::MissingRequiredField("MAN_DV_1".to_string()),
        })?;
        let dv2_field = self.dv2.ok_or_else(|| KvnError {
            span: Span::default(),
            kind: KvnErrorKind::MissingRequiredField("MAN_DV_2".to_string()),
        })?;
        let dv3_field = self.dv3.ok_or_else(|| KvnError {
            span: Span::default(),
            kind: KvnErrorKind::MissingRequiredField("MAN_DV_3".to_string()),
        })?;

        let ignition_epoch = parse_epoch(&ign_field, time_system)?;
        let duration = TimeDelta::from_seconds_f64(parse_f64(&dur_field)?);
        let delta_mass = Mass::kilograms(parse_f64(&dm_field)?);
        let frame = self.frame.map(|s| OdmFrame::from_wire(&s));
        let dv1 = Velocity::kilometers_per_second(parse_f64(&dv1_field)?);
        let dv2 = Velocity::kilometers_per_second(parse_f64(&dv2_field)?);
        let dv3 = Velocity::kilometers_per_second(parse_f64(&dv3_field)?);

        Ok(Maneuver {
            comments: self.comments,
            ignition_epoch,
            duration,
            delta_mass,
            frame,
            delta_v: [dv1, dv2, dv3],
        })
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
            // Other bracketed sections ignored for OPM
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
                    // Other subsections ignored for OPM
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

impl TryFrom<KvnDocument> for Opm {
    type Error = KvnError;

    fn try_from(doc: KvnDocument) -> Result<Self, Self::Error> {
        // 1. Validate message kind.
        if doc.message_kind != MessageKind::Opm {
            return Err(KvnError {
                span: Span::default(),
                kind: KvnErrorKind::UnexpectedKeyword(format!("{}", doc.message_kind)),
            });
        }

        // 2. Flatten entries and extract COVARIANCE subsection.
        let (flat, covariance_section) = flatten_entries(&doc);

        // 3. Walk the flat list with a state machine to accumulate fields by role.
        //    We use KvnEntry references; collect fields owned into per-role Vecs.
        let mut header_entries: Vec<KvnEntry> = Vec::new();
        let mut metadata_entries: Vec<KvnEntry> = Vec::new();
        let mut state_entries: Vec<KvnEntry> = Vec::new();
        let mut keplerian_entries: Vec<KvnEntry> = Vec::new();
        let mut spacecraft_entries: Vec<KvnEntry> = Vec::new();
        let mut user_defined: BTreeMap<String, String> = BTreeMap::new();

        // Comment routing: pending comments go to the first field role that fires after them.
        let mut pending_comments: Vec<String> = Vec::new();
        let mut header_comments: Vec<String> = Vec::new();
        let mut metadata_comments: Vec<String> = Vec::new();
        let mut state_comments: Vec<String> = Vec::new();
        let mut keplerian_comments: Vec<String> = Vec::new();
        let mut spacecraft_comments: Vec<String> = Vec::new();

        // Maneuver state-machine
        let mut maneuver_builders: Vec<ManeuverBuilder> = Vec::new();
        let mut current_maneuver: Option<ManeuverBuilder> = None;

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
                        KeywordRole::State => {
                            state_comments.append(&mut pending_comments);
                            state_entries.push(KvnEntry::Field(field.clone()));
                        }
                        KeywordRole::Keplerian => {
                            keplerian_comments.append(&mut pending_comments);
                            keplerian_entries.push(KvnEntry::Field(field.clone()));
                        }
                        KeywordRole::Spacecraft => {
                            spacecraft_comments.append(&mut pending_comments);
                            spacecraft_entries.push(KvnEntry::Field(field.clone()));
                        }
                        KeywordRole::ManeuverStart => {
                            // Close out the previous maneuver builder if any.
                            if let Some(builder) = current_maneuver.take() {
                                maneuver_builders.push(builder);
                            }
                            current_maneuver = Some(ManeuverBuilder::new(
                                std::mem::take(&mut pending_comments),
                                field.clone(),
                            ));
                        }
                        KeywordRole::ManeuverField => {
                            pending_comments.clear(); // comments within maneuver body are dropped
                            let builder = current_maneuver.as_mut().ok_or_else(|| KvnError {
                                span: Span::default(),
                                kind: KvnErrorKind::UnexpectedKeyword(field.key.clone()),
                            })?;
                            builder.set_field(&field.key, field.clone())?;
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

        // Close the last maneuver builder.
        if let Some(builder) = current_maneuver.take() {
            maneuver_builders.push(builder);
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

        let metadata = OpmMetadata {
            comments: metadata_comments,
            object_name,
            object_id,
            center,
            frame,
            frame_epoch,
        };

        // 6. Parse state vector.
        let epoch_field = require_field(&state_entries, "EPOCH")?;
        let epoch = parse_epoch(epoch_field, &time_system)?;

        let x =
            Distance::kilometers(parse_f64_optional(&state_entries, "X")?.ok_or_else(|| {
                KvnError {
                    span: Span::default(),
                    kind: KvnErrorKind::MissingRequiredField("X".to_string()),
                }
            })?);
        let y =
            Distance::kilometers(parse_f64_optional(&state_entries, "Y")?.ok_or_else(|| {
                KvnError {
                    span: Span::default(),
                    kind: KvnErrorKind::MissingRequiredField("Y".to_string()),
                }
            })?);
        let z =
            Distance::kilometers(parse_f64_optional(&state_entries, "Z")?.ok_or_else(|| {
                KvnError {
                    span: Span::default(),
                    kind: KvnErrorKind::MissingRequiredField("Z".to_string()),
                }
            })?);
        let xd = Velocity::kilometers_per_second(
            parse_f64_optional(&state_entries, "X_DOT")?.ok_or_else(|| KvnError {
                span: Span::default(),
                kind: KvnErrorKind::MissingRequiredField("X_DOT".to_string()),
            })?,
        );
        let yd = Velocity::kilometers_per_second(
            parse_f64_optional(&state_entries, "Y_DOT")?.ok_or_else(|| KvnError {
                span: Span::default(),
                kind: KvnErrorKind::MissingRequiredField("Y_DOT".to_string()),
            })?,
        );
        let zd = Velocity::kilometers_per_second(
            parse_f64_optional(&state_entries, "Z_DOT")?.ok_or_else(|| KvnError {
                span: Span::default(),
                kind: KvnErrorKind::MissingRequiredField("Z_DOT".to_string()),
            })?,
        );
        let state = Cartesian::new(x, y, z, xd, yd, zd);

        // 7. Parse optional Keplerian block.
        let keplerian = if keplerian_entries.is_empty() {
            None
        } else {
            let sma = Distance::kilometers(
                parse_f64_optional(&keplerian_entries, "SEMI_MAJOR_AXIS")?.ok_or_else(|| {
                    KvnError {
                        span: Span::default(),
                        kind: KvnErrorKind::MissingRequiredField("SEMI_MAJOR_AXIS".to_string()),
                    }
                })?,
            );
            let ecc = parse_f64_optional(&keplerian_entries, "ECCENTRICITY")?.ok_or_else(|| {
                KvnError {
                    span: Span::default(),
                    kind: KvnErrorKind::MissingRequiredField("ECCENTRICITY".to_string()),
                }
            })?;
            let inc = Angle::degrees(
                parse_f64_optional(&keplerian_entries, "INCLINATION")?.ok_or_else(|| KvnError {
                    span: Span::default(),
                    kind: KvnErrorKind::MissingRequiredField("INCLINATION".to_string()),
                })?,
            );
            let raan = Angle::degrees(
                parse_f64_optional(&keplerian_entries, "RA_OF_ASC_NODE")?.ok_or_else(|| {
                    KvnError {
                        span: Span::default(),
                        kind: KvnErrorKind::MissingRequiredField("RA_OF_ASC_NODE".to_string()),
                    }
                })?,
            );
            let aop = Angle::degrees(
                parse_f64_optional(&keplerian_entries, "ARG_OF_PERICENTER")?.ok_or_else(|| {
                    KvnError {
                        span: Span::default(),
                        kind: KvnErrorKind::MissingRequiredField("ARG_OF_PERICENTER".to_string()),
                    }
                })?,
            );
            // TRUE_ANOMALY is the supported anomaly type; MEAN_ANOMALY present
            // without TRUE_ANOMALY is not supported in v1.
            let ta_opt = parse_f64_optional(&keplerian_entries, "TRUE_ANOMALY")?;
            let ta = match ta_opt {
                Some(deg) => Angle::degrees(deg),
                None => {
                    return Err(KvnError {
                        span: Span::default(),
                        kind: KvnErrorKind::MissingRequiredField("TRUE_ANOMALY".to_string()),
                    });
                }
            };

            let elements = Keplerian::builder()
                .with_semi_major_axis(sma, ecc)
                .with_inclination(inc)
                .with_longitude_of_ascending_node(raan)
                .with_argument_of_periapsis(aop)
                .with_true_anomaly(ta)
                .build()
                .map_err(|e| KvnError {
                    span: Span::default(),
                    kind: KvnErrorKind::InvalidValue {
                        keyword: "KEPLERIAN".to_string(),
                        reason: e.to_string(),
                    },
                })?;

            let gm = parse_f64_optional(&keplerian_entries, "GM")?
                .map(GravitationalParameter::km3_per_s2);

            Some(OpmKeplerian {
                comments: keplerian_comments,
                elements,
                gm,
            })
        };

        // 8. Parse optional spacecraft parameters.
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

        // 9. Parse optional covariance.
        let covariance = match covariance_section {
            Some(sec) => Some(parse_covariance(sec)?),
            None => None,
        };

        // 10. Build maneuvers.
        let mut maneuvers: Vec<Maneuver> = Vec::new();
        for builder in maneuver_builders {
            maneuvers.push(builder.build(&time_system)?);
        }

        Ok(Opm {
            header,
            metadata,
            epoch,
            state,
            state_comments,
            keplerian,
            spacecraft,
            covariance,
            maneuvers,
            user_defined,
        })
    }
}

/// Parses a KVN-formatted string into a typed [`Opm`].
pub fn read_opm(input: &str) -> Result<Opm, KvnError> {
    let doc = parse(input)?;
    Opm::try_from(doc)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use lox_bodies::DynOrigin;
    use lox_core::elements::{GravitationalParameter, Keplerian};
    use lox_core::units::{Angle, Distance, Mass, Velocity};
    use lox_frames::DynFrame;
    use nalgebra::Matrix6;

    use crate::types::common::{Covariance, OdmCenter, OdmFrame, OdmHeader, OdmTime};
    use crate::types::opm::{Maneuver, Opm, OpmKeplerian, OpmMetadata};

    use super::*;

    fn sample_epoch() -> OdmTime {
        OdmTime::Time(lox_time::time::Time::j2000(
            lox_time::time_scales::DynTimeScale::Tai,
        ))
    }

    fn sample_opm() -> Opm {
        let epoch = sample_epoch();
        Opm {
            header: OdmHeader {
                comments: Vec::new(),
                classification: None,
                creation_date: epoch,
                originator: "TEST".to_string(),
                message_id: None,
            },
            metadata: OpmMetadata {
                comments: Vec::new(),
                object_name: "TEST-SAT".to_string(),
                object_id: "2024-000A".to_string(),
                center: OdmCenter::Known(DynOrigin::Earth),
                frame: OdmFrame::Known(DynFrame::Icrf),
                frame_epoch: None,
            },
            epoch,
            state: lox_core::coords::Cartesian::new(
                Distance::kilometers(7000.0),
                Distance::kilometers(0.0),
                Distance::kilometers(0.0),
                Velocity::kilometers_per_second(0.0),
                Velocity::kilometers_per_second(7.5),
                Velocity::kilometers_per_second(0.0),
            ),
            state_comments: Vec::new(),
            keplerian: None,
            spacecraft: None,
            covariance: None,
            maneuvers: Vec::new(),
            user_defined: BTreeMap::new(),
        }
    }

    #[test]
    fn write_minimal_opm_contains_expected_fields() {
        let opm = sample_opm();
        let output = write_opm(&opm);

        assert!(
            output.contains("CCSDS_OPM_VERS = 3.0"),
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
            output.contains("REF_FRAME = ICRF"),
            "missing REF_FRAME; got:\n{output}"
        );
        assert!(
            output.contains("TIME_SYSTEM = TAI"),
            "missing TIME_SYSTEM; got:\n{output}"
        );
        assert!(output.contains("EPOCH = "), "missing EPOCH; got:\n{output}");
        assert!(
            output.contains("X = 7000 [km]"),
            "missing X; got:\n{output}"
        );
        assert!(output.contains("Y = 0 [km]"), "missing Y; got:\n{output}");
        assert!(output.contains("Z = 0 [km]"), "missing Z; got:\n{output}");
        assert!(
            output.contains("X_DOT = 0 [km/s]"),
            "missing X_DOT; got:\n{output}"
        );
        assert!(
            output.contains("Y_DOT = 7.5 [km/s]"),
            "missing Y_DOT; got:\n{output}"
        );
        assert!(
            output.contains("Z_DOT = 0 [km/s]"),
            "missing Z_DOT; got:\n{output}"
        );
    }

    #[test]
    fn write_opm_with_keplerian_block() {
        let mut opm = sample_opm();
        let elements = Keplerian::builder()
            .with_semi_major_axis(Distance::kilometers(7000.0), 0.001)
            .with_inclination(Angle::radians(0.9))
            .with_longitude_of_ascending_node(Angle::ZERO)
            .with_argument_of_periapsis(Angle::ZERO)
            .with_true_anomaly(Angle::ZERO)
            .build()
            .expect("valid Keplerian elements");

        let wire_gm = GravitationalParameter::km3_per_s2(398600.4415);
        opm.keplerian = Some(OpmKeplerian {
            comments: Vec::new(),
            elements,
            gm: Some(wire_gm),
        });

        let output = write_opm(&opm);

        assert!(
            output.contains("SEMI_MAJOR_AXIS = 7000 [km]"),
            "missing SEMI_MAJOR_AXIS; got:\n{output}"
        );
        assert!(
            output.contains("ECCENTRICITY = 0.001"),
            "missing ECCENTRICITY; got:\n{output}"
        );
        assert!(
            output.contains("INCLINATION ="),
            "missing INCLINATION; got:\n{output}"
        );
        assert!(
            output.contains("GM =") && output.contains("[km**3/s**2]"),
            "missing GM; got:\n{output}"
        );
        assert!(
            output.contains("398600.4415"),
            "wrong GM value; got:\n{output}"
        );
    }

    #[test]
    fn write_opm_keplerian_without_gm_omits_gm_field() {
        let mut opm = sample_opm();
        let elements = Keplerian::builder()
            .with_semi_major_axis(Distance::kilometers(7000.0), 0.001)
            .with_inclination(Angle::radians(0.9))
            .with_longitude_of_ascending_node(Angle::ZERO)
            .with_argument_of_periapsis(Angle::ZERO)
            .with_true_anomaly(Angle::ZERO)
            .build()
            .expect("valid Keplerian elements");

        opm.keplerian = Some(OpmKeplerian {
            comments: Vec::new(),
            elements,
            // No wire GM → should not appear in output
            gm: None,
        });

        let output = write_opm(&opm);

        assert!(
            output.contains("SEMI_MAJOR_AXIS ="),
            "missing SEMI_MAJOR_AXIS; got:\n{output}"
        );
        assert!(
            !output.contains("GM ="),
            "unexpected GM field; got:\n{output}"
        );
    }

    #[test]
    fn write_opm_with_maneuver() {
        use lox_core::time::deltas::TimeDelta;

        let mut opm = sample_opm();
        let man = Maneuver {
            comments: Vec::new(),
            ignition_epoch: sample_epoch(),
            duration: TimeDelta::from_seconds(60),
            delta_mass: Mass::kilograms(-1.0),
            frame: None,
            delta_v: [
                Velocity::kilometers_per_second(0.1),
                Velocity::kilometers_per_second(0.0),
                Velocity::kilometers_per_second(0.0),
            ],
        };
        opm.maneuvers.push(man);

        let output = write_opm(&opm);

        assert!(
            output.contains("MAN_EPOCH_IGNITION ="),
            "missing MAN_EPOCH_IGNITION; got:\n{output}"
        );
        assert!(
            output.contains("MAN_DURATION = 60 [s]"),
            "missing MAN_DURATION; got:\n{output}"
        );
        assert!(
            output.contains("MAN_DELTA_MASS = -1 [kg]"),
            "missing MAN_DELTA_MASS; got:\n{output}"
        );
        assert!(
            output.contains("MAN_DV_1 = 0.1 [km/s]"),
            "missing MAN_DV_1; got:\n{output}"
        );
    }

    #[test]
    fn write_opm_with_covariance() {
        let mut opm = sample_opm();
        opm.covariance = Some(Covariance {
            comments: Vec::new(),
            frame: None,
            matrix: Matrix6::identity(),
        });

        let output = write_opm(&opm);

        assert!(
            output.contains("COVARIANCE_START"),
            "missing COVARIANCE_START; got:\n{output}"
        );
        assert!(
            output.contains("COVARIANCE_STOP"),
            "missing COVARIANCE_STOP; got:\n{output}"
        );
        assert!(output.contains("CX_X = 1"), "missing CX_X; got:\n{output}");
        // Off-diagonal should be zero
        assert!(output.contains("CY_X = 0"), "missing CY_X; got:\n{output}");
    }

    #[test]
    fn write_opm_with_user_defined() {
        let mut opm = sample_opm();
        opm.user_defined
            .insert("OPERATOR".to_string(), "GSOC".to_string());

        let output = write_opm(&opm);

        assert!(
            output.contains("USER_DEFINED_OPERATOR = GSOC"),
            "missing USER_DEFINED_OPERATOR; got:\n{output}"
        );
    }

    #[test]
    fn write_opm_metadata_comments_appear_before_fields() {
        let mut opm = sample_opm();
        opm.metadata.comments.push("A metadata comment".to_string());

        let output = write_opm(&opm);

        // Verify the comment appears before OBJECT_NAME in the output string
        let comment_pos = output
            .find("COMMENT A metadata comment")
            .expect("comment not found");
        let object_pos = output.find("OBJECT_NAME =").expect("OBJECT_NAME not found");
        assert!(
            comment_pos < object_pos,
            "comment should appear before OBJECT_NAME"
        );
    }

    // -----------------------------------------------------------------------
    // Read direction tests
    // -----------------------------------------------------------------------

    #[test]
    fn round_trip_minimal_opm() {
        let opm = sample_opm();
        let written = write_opm(&opm);
        let parsed = read_opm(&written).expect("parse failed");
        assert_eq!(opm, parsed, "round-trip mismatch");
    }

    #[test]
    fn round_trip_opm_with_two_maneuvers() {
        use lox_core::time::deltas::TimeDelta;

        let mut opm = sample_opm();
        opm.maneuvers.push(Maneuver {
            comments: Vec::new(),
            ignition_epoch: sample_epoch(),
            duration: TimeDelta::from_seconds(60),
            delta_mass: Mass::kilograms(-1.0),
            frame: None,
            delta_v: [
                Velocity::kilometers_per_second(0.1),
                Velocity::kilometers_per_second(0.0),
                Velocity::kilometers_per_second(0.0),
            ],
        });
        opm.maneuvers.push(Maneuver {
            comments: Vec::new(),
            ignition_epoch: sample_epoch(),
            duration: TimeDelta::from_seconds(120),
            delta_mass: Mass::kilograms(-2.0),
            frame: None,
            delta_v: [
                Velocity::kilometers_per_second(0.0),
                Velocity::kilometers_per_second(0.2),
                Velocity::kilometers_per_second(0.0),
            ],
        });

        let written = write_opm(&opm);
        let parsed = read_opm(&written).expect("parse failed");
        assert_eq!(opm, parsed, "maneuver round-trip mismatch");
    }

    #[test]
    fn round_trip_opm_with_covariance() {
        let mut opm = sample_opm();
        opm.covariance = Some(Covariance {
            comments: Vec::new(),
            frame: None,
            matrix: Matrix6::identity(),
        });

        let written = write_opm(&opm);
        let parsed = read_opm(&written).expect("parse failed");
        assert_eq!(opm, parsed, "covariance round-trip mismatch");
    }

    /// Build Keplerian elements with angles that survive `{}` degree
    /// formatting exactly. Using multiples of `π/180 * N` degrees where N
    /// is representable exactly: 0°, 30°, 45°, 60°, 90°, 180°, 270°, 360°
    /// are exact. Here we use all-zero angles for simplicity.
    fn keplerian_elements_roundtrip_safe() -> Keplerian {
        Keplerian::builder()
            .with_semi_major_axis(Distance::kilometers(7000.0), 0.001)
            .with_inclination(Angle::degrees(45.0))
            .with_longitude_of_ascending_node(Angle::ZERO)
            .with_argument_of_periapsis(Angle::ZERO)
            .with_true_anomaly(Angle::ZERO)
            .build()
            .expect("valid elements")
    }

    #[test]
    fn round_trip_keplerian_no_gm() {
        let mut opm = sample_opm();
        opm.keplerian = Some(OpmKeplerian {
            comments: Vec::new(),
            elements: keplerian_elements_roundtrip_safe(),
            gm: None,
        });

        let written = write_opm(&opm);
        let parsed = read_opm(&written).expect("parse failed");
        assert_eq!(opm, parsed, "keplerian-no-gm round-trip mismatch");
        assert!(parsed.keplerian.unwrap().gm.is_none());
    }

    #[test]
    fn round_trip_keplerian_with_gm() {
        let wire_gm = GravitationalParameter::km3_per_s2(398600.4415);
        let mut opm = sample_opm();
        opm.keplerian = Some(OpmKeplerian {
            comments: Vec::new(),
            elements: keplerian_elements_roundtrip_safe(),
            gm: Some(wire_gm),
        });

        let written = write_opm(&opm);
        let parsed = read_opm(&written).expect("parse failed");
        assert_eq!(opm, parsed, "keplerian-with-gm round-trip mismatch");
        let parsed_gm = parsed.keplerian.unwrap().gm.unwrap();
        // Values should be within floating-point round-trip tolerance.
        let diff = (parsed_gm.as_f64() - wire_gm.as_f64()).abs();
        assert!(
            diff < 1.0,
            "GM round-trip error too large: {diff} m³/s²; parsed={parsed_gm}, original={wire_gm}"
        );
    }

    #[test]
    fn read_missing_epoch_returns_error() {
        let kvn = "\
CCSDS_OPM_VERS = 3.0
CREATION_DATE = 2024-01-01T00:00:00
ORIGINATOR = TEST
OBJECT_NAME = TEST-SAT
OBJECT_ID = 2024-000A
CENTER_NAME = EARTH
REF_FRAME = ICRF
TIME_SYSTEM = TAI
X = 7000.0 [km]
Y = 0.0 [km]
Z = 0.0 [km]
X_DOT = 0.0 [km/s]
Y_DOT = 7.5 [km/s]
Z_DOT = 0.0 [km/s]
";
        let err = read_opm(kvn).expect_err("should fail on missing EPOCH");
        assert!(
            matches!(err.kind, KvnErrorKind::MissingRequiredField(ref k) if k == "EPOCH"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_duplicate_field_returns_error() {
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
X = 8000.0 [km]
Y = 0.0 [km]
Z = 0.0 [km]
X_DOT = 0.0 [km/s]
Y_DOT = 7.5 [km/s]
Z_DOT = 0.0 [km/s]
";
        let err = read_opm(kvn).expect_err("should fail on duplicate X");
        assert!(
            matches!(err.kind, KvnErrorKind::DuplicateField(ref k) if k == "X"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_invalid_value_returns_error() {
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
X = not-a-number [km]
Y = 0.0 [km]
Z = 0.0 [km]
X_DOT = 0.0 [km/s]
Y_DOT = 7.5 [km/s]
Z_DOT = 0.0 [km/s]
";
        let err = read_opm(kvn).expect_err("should fail on non-numeric X");
        assert!(
            matches!(err.kind, KvnErrorKind::InvalidValue { ref keyword, .. } if keyword == "X"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_unknown_keyword_returns_error() {
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
FOO = bar
";
        let err = read_opm(kvn).expect_err("should fail on unknown keyword FOO");
        assert!(
            matches!(err.kind, KvnErrorKind::UnexpectedKeyword(ref k) if k == "FOO"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_user_defined_preserved() {
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
USER_DEFINED_OPERATOR = GSOC
";
        let parsed = read_opm(kvn).expect("parse failed");
        assert_eq!(
            parsed.user_defined.get("OPERATOR"),
            Some(&"GSOC".to_string()),
            "USER_DEFINED_OPERATOR not preserved"
        );
    }

    #[test]
    fn read_multiple_maneuvers_split_correctly() {
        use lox_core::time::deltas::TimeDelta;

        let mut opm = sample_opm();
        opm.maneuvers.push(Maneuver {
            comments: Vec::new(),
            ignition_epoch: sample_epoch(),
            duration: TimeDelta::from_seconds(60),
            delta_mass: Mass::kilograms(-1.0),
            frame: None,
            delta_v: [
                Velocity::kilometers_per_second(0.1),
                Velocity::kilometers_per_second(0.0),
                Velocity::kilometers_per_second(0.0),
            ],
        });
        opm.maneuvers.push(Maneuver {
            comments: Vec::new(),
            ignition_epoch: sample_epoch(),
            duration: TimeDelta::from_seconds(120),
            delta_mass: Mass::kilograms(-0.5),
            frame: None,
            delta_v: [
                Velocity::kilometers_per_second(0.0),
                Velocity::kilometers_per_second(0.05),
                Velocity::kilometers_per_second(0.0),
            ],
        });

        let written = write_opm(&opm);
        let parsed = read_opm(&written).expect("parse failed");
        assert_eq!(
            parsed.maneuvers.len(),
            2,
            "expected 2 maneuvers, got {}",
            parsed.maneuvers.len()
        );
        assert!((parsed.maneuvers[0].duration.to_seconds().to_f64() - 60.0).abs() < 0.001);
        assert!((parsed.maneuvers[1].duration.to_seconds().to_f64() - 120.0).abs() < 0.001);
    }

    #[test]
    fn read_comments_routed_to_correct_buckets() {
        // Comments before header fields → header.comments
        // Comments before metadata fields → metadata.comments
        let kvn = "\
CCSDS_OPM_VERS = 3.0
COMMENT Header comment
CREATION_DATE = 2024-01-01T00:00:00
ORIGINATOR = TEST
COMMENT Metadata comment
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
        let parsed = read_opm(kvn).expect("parse failed");
        assert_eq!(
            parsed.header.comments.len(),
            1,
            "header comments count mismatch"
        );
        assert_eq!(
            parsed.metadata.comments.len(),
            1,
            "metadata comments count mismatch"
        );
    }

    // -----------------------------------------------------------------------
    // Additional read direction tests for error paths and optional blocks
    // -----------------------------------------------------------------------

    #[test]
    fn read_wrong_message_kind_returns_error() {
        // An OMM document fed to read_opm (which calls OPM TryFrom) must fail.
        let kvn = "\
CCSDS_OMM_VERS = 3.0
CREATION_DATE = 2024-01-01T00:00:00
ORIGINATOR = TEST
OBJECT_NAME = TEST-SAT
OBJECT_ID = 2024-000A
CENTER_NAME = EARTH
REF_FRAME = TEME
TIME_SYSTEM = TAI
MEAN_ELEMENT_THEORY = SGP4
EPOCH = 2024-01-01T00:00:00
SEMI_MAJOR_AXIS = 6860.0 [km]
ECCENTRICITY = 0.001
INCLINATION = 45.0 [deg]
RA_OF_ASC_NODE = 0.0 [deg]
ARG_OF_PERICENTER = 0.0 [deg]
MEAN_ANOMALY = 0.0 [deg]
";
        let err = read_opm(kvn).expect_err("should fail on wrong message kind");
        assert!(
            matches!(err.kind, KvnErrorKind::UnexpectedKeyword(_)),
            "unexpected error kind: {err}"
        );
    }

    #[test]
    fn read_opm_with_keplerian_block() {
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
SEMI_MAJOR_AXIS = 7000.0 [km]
ECCENTRICITY = 0.001
INCLINATION = 45.0 [deg]
RA_OF_ASC_NODE = 0.0 [deg]
ARG_OF_PERICENTER = 0.0 [deg]
TRUE_ANOMALY = 0.0 [deg]
GM = 398600.4415 [km**3/s**2]
";
        let parsed = read_opm(kvn).expect("parse failed");
        let kep = parsed.keplerian.expect("keplerian block should be present");
        assert!(kep.gm.is_some(), "GM should be parsed");
    }

    #[test]
    fn read_opm_missing_true_anomaly_returns_error() {
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
SEMI_MAJOR_AXIS = 7000.0 [km]
ECCENTRICITY = 0.001
INCLINATION = 45.0 [deg]
RA_OF_ASC_NODE = 0.0 [deg]
ARG_OF_PERICENTER = 0.0 [deg]
MEAN_ANOMALY = 0.0 [deg]
";
        let err = read_opm(kvn).expect_err("should fail on missing TRUE_ANOMALY");
        assert!(
            matches!(err.kind, KvnErrorKind::MissingRequiredField(ref k) if k == "TRUE_ANOMALY"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_opm_with_spacecraft_parameters() {
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
MASS = 500.0 [kg]
SOLAR_RAD_AREA = 2.0 [m**2]
SOLAR_RAD_COEFF = 1.2
DRAG_AREA = 1.5 [m**2]
DRAG_COEFF = 2.2
";
        let parsed = read_opm(kvn).expect("parse failed");
        let sp = parsed
            .spacecraft
            .expect("spacecraft block should be present");
        assert!((sp.mass.unwrap().to_kilograms() - 500.0).abs() < 1e-9);
        assert_eq!(sp.solar_rad_coeff, Some(1.2));
        assert_eq!(sp.drag_coeff, Some(2.2));
    }

    #[test]
    fn read_opm_with_covariance_with_frame() {
        let mut opm = sample_opm();
        opm.covariance = Some(Covariance {
            comments: Vec::new(),
            frame: Some(OdmFrame::Known(lox_frames::DynFrame::Icrf)),
            matrix: Matrix6::identity(),
        });
        let written = write_opm(&opm);
        let parsed = read_opm(&written).expect("parse failed");
        let cov = parsed.covariance.expect("covariance should be present");
        assert!(cov.frame.is_some(), "covariance frame should be preserved");
    }

    #[test]
    fn read_opm_maneuver_with_ref_frame() {
        use lox_core::time::deltas::TimeDelta;

        let mut opm = sample_opm();
        opm.maneuvers.push(Maneuver {
            comments: Vec::new(),
            ignition_epoch: sample_epoch(),
            duration: TimeDelta::from_seconds(30),
            delta_mass: Mass::kilograms(-0.5),
            frame: Some(OdmFrame::Known(lox_frames::DynFrame::Icrf)),
            delta_v: [
                Velocity::kilometers_per_second(0.01),
                Velocity::kilometers_per_second(0.0),
                Velocity::kilometers_per_second(0.0),
            ],
        });
        let written = write_opm(&opm);
        let parsed = read_opm(&written).expect("parse failed");
        assert_eq!(parsed.maneuvers.len(), 1);
        assert!(
            parsed.maneuvers[0].frame.is_some(),
            "maneuver frame should be preserved"
        );
    }

    #[test]
    fn read_opm_maneuver_duplicate_dv1_returns_error() {
        let base = "\
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
MAN_EPOCH_IGNITION = 2024-01-01T00:00:00
MAN_DURATION = 60 [s]
MAN_DELTA_MASS = -1.0 [kg]
MAN_DV_1 = 0.1 [km/s]
MAN_DV_1 = 0.2 [km/s]
MAN_DV_2 = 0.0 [km/s]
MAN_DV_3 = 0.0 [km/s]
";
        let err = read_opm(base).expect_err("should fail on duplicate MAN_DV_1");
        assert!(
            matches!(err.kind, KvnErrorKind::DuplicateField(ref k) if k == "MAN_DV_1"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_opm_maneuver_field_before_epoch_returns_error() {
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
MAN_DURATION = 60 [s]
";
        let err = read_opm(kvn).expect_err("should fail on maneuver field before ignition epoch");
        assert!(
            matches!(err.kind, KvnErrorKind::UnexpectedKeyword(_)),
            "unexpected error kind: {err}"
        );
    }

    #[test]
    fn read_opm_missing_maneuver_dv3_returns_error() {
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
MAN_EPOCH_IGNITION = 2024-01-01T00:00:00
MAN_DURATION = 60 [s]
MAN_DELTA_MASS = -1.0 [kg]
MAN_DV_1 = 0.1 [km/s]
MAN_DV_2 = 0.0 [km/s]
";
        let err = read_opm(kvn).expect_err("should fail on missing MAN_DV_3");
        assert!(
            matches!(err.kind, KvnErrorKind::MissingRequiredField(ref k) if k == "MAN_DV_3"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn write_opm_with_spacecraft_all_fields() {
        use crate::types::common::SpacecraftParameters;
        use lox_core::units::Area;

        let mut opm = sample_opm();
        opm.spacecraft = Some(SpacecraftParameters {
            comments: Vec::new(),
            mass: Some(Mass::kilograms(500.0)),
            solar_rad_area: Some(Area::square_meters(2.0)),
            solar_rad_coeff: Some(1.2),
            drag_area: Some(Area::square_meters(1.5)),
            drag_coeff: Some(2.2),
        });

        let output = write_opm(&opm);
        assert!(
            output.contains("MASS = 500 [kg]"),
            "missing MASS; got:\n{output}"
        );
        assert!(
            output.contains("DRAG_COEFF = 2.2"),
            "missing DRAG_COEFF; got:\n{output}"
        );
    }

    #[test]
    fn round_trip_opm_with_keplerian_and_spacecraft() {
        use crate::types::common::SpacecraftParameters;
        use lox_core::units::Area;

        let mut opm = sample_opm();
        opm.keplerian = Some(OpmKeplerian {
            comments: Vec::new(),
            elements: keplerian_elements_roundtrip_safe(),
            gm: None,
        });
        opm.spacecraft = Some(SpacecraftParameters {
            comments: vec!["Spacecraft comment".to_string()],
            mass: Some(Mass::kilograms(200.0)),
            solar_rad_area: Some(Area::square_meters(3.0)),
            solar_rad_coeff: Some(1.5),
            drag_area: Some(Area::square_meters(2.0)),
            drag_coeff: Some(2.5),
        });

        let written = write_opm(&opm);
        let parsed = read_opm(&written).expect("parse failed");
        assert!(
            parsed.keplerian.is_some(),
            "keplerian block should survive round-trip"
        );
        let sp = parsed
            .spacecraft
            .expect("spacecraft block should survive round-trip");
        assert!((sp.mass.unwrap().to_kilograms() - 200.0).abs() < 1e-9);
    }

    #[test]
    fn write_opm_with_maneuver_with_frame() {
        use lox_core::time::deltas::TimeDelta;

        let mut opm = sample_opm();
        opm.maneuvers.push(Maneuver {
            comments: Vec::new(),
            ignition_epoch: sample_epoch(),
            duration: TimeDelta::from_seconds(60),
            delta_mass: Mass::kilograms(-1.0),
            frame: Some(OdmFrame::Known(lox_frames::DynFrame::Icrf)),
            delta_v: [
                Velocity::kilometers_per_second(0.1),
                Velocity::kilometers_per_second(0.0),
                Velocity::kilometers_per_second(0.0),
            ],
        });

        let output = write_opm(&opm);
        assert!(
            output.contains("MAN_REF_FRAME = ICRF"),
            "missing MAN_REF_FRAME; got:\n{output}"
        );
    }

    // -----------------------------------------------------------------
    // Missing-required-field coverage matrix
    //
    // Each `.ok_or_else(|| KvnError { ... MissingRequiredField })` site
    // is a distinct closure in LLVM coverage. The tests below take the
    // path missing exactly one required field, exercising every such
    // closure on the read path.
    // -----------------------------------------------------------------

    /// Builds a full OPM KVN string with header + metadata + state +
    /// Keplerian + a single maneuver, then strips the named field by
    /// removing the matching line.
    fn opm_kvn_without_field(missing: &str) -> String {
        let full = "\
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
SEMI_MAJOR_AXIS = 7000.0 [km]
ECCENTRICITY = 0.001
INCLINATION = 45.0 [deg]
RA_OF_ASC_NODE = 0.0 [deg]
ARG_OF_PERICENTER = 0.0 [deg]
TRUE_ANOMALY = 0.0 [deg]
MAN_EPOCH_IGNITION = 2024-01-01T00:01:00
MAN_DURATION = 60 [s]
MAN_DELTA_MASS = -1.0 [kg]
MAN_DV_1 = 0.1 [km/s]
MAN_DV_2 = 0.0 [km/s]
MAN_DV_3 = 0.0 [km/s]
";
        full.lines()
            .filter(|line| !line.trim_start().starts_with(&format!("{missing} =")))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }

    fn assert_missing_field(input: &str, expected: &str) {
        let err = read_opm(input).expect_err(&format!("expected missing {expected}"));
        let KvnErrorKind::MissingRequiredField(ref k) = err.kind else {
            panic!("expected MissingRequiredField({expected}), got: {err:?}");
        };
        assert_eq!(k, expected, "wrong missing-field name");
    }

    #[test]
    fn missing_x_returns_error() {
        assert_missing_field(&opm_kvn_without_field("X"), "X");
    }
    #[test]
    fn missing_y_returns_error() {
        assert_missing_field(&opm_kvn_without_field("Y"), "Y");
    }
    #[test]
    fn missing_z_returns_error() {
        assert_missing_field(&opm_kvn_without_field("Z"), "Z");
    }
    #[test]
    fn missing_x_dot_returns_error() {
        assert_missing_field(&opm_kvn_without_field("X_DOT"), "X_DOT");
    }
    #[test]
    fn missing_y_dot_returns_error() {
        assert_missing_field(&opm_kvn_without_field("Y_DOT"), "Y_DOT");
    }
    #[test]
    fn missing_z_dot_returns_error() {
        assert_missing_field(&opm_kvn_without_field("Z_DOT"), "Z_DOT");
    }
    #[test]
    fn missing_semi_major_axis_returns_error() {
        assert_missing_field(&opm_kvn_without_field("SEMI_MAJOR_AXIS"), "SEMI_MAJOR_AXIS");
    }
    #[test]
    fn missing_eccentricity_returns_error() {
        assert_missing_field(&opm_kvn_without_field("ECCENTRICITY"), "ECCENTRICITY");
    }
    #[test]
    fn missing_inclination_returns_error() {
        assert_missing_field(&opm_kvn_without_field("INCLINATION"), "INCLINATION");
    }
    #[test]
    fn missing_ra_of_asc_node_returns_error() {
        assert_missing_field(&opm_kvn_without_field("RA_OF_ASC_NODE"), "RA_OF_ASC_NODE");
    }
    #[test]
    fn missing_arg_of_pericenter_returns_error() {
        assert_missing_field(
            &opm_kvn_without_field("ARG_OF_PERICENTER"),
            "ARG_OF_PERICENTER",
        );
    }
    #[test]
    fn missing_man_duration_returns_error() {
        assert_missing_field(&opm_kvn_without_field("MAN_DURATION"), "MAN_DURATION");
    }
    #[test]
    fn missing_man_delta_mass_returns_error() {
        assert_missing_field(&opm_kvn_without_field("MAN_DELTA_MASS"), "MAN_DELTA_MASS");
    }
    #[test]
    fn missing_man_dv_1_returns_error() {
        assert_missing_field(&opm_kvn_without_field("MAN_DV_1"), "MAN_DV_1");
    }
    #[test]
    fn missing_man_dv_2_returns_error() {
        assert_missing_field(&opm_kvn_without_field("MAN_DV_2"), "MAN_DV_2");
    }
}
