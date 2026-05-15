// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! XML ↔ typed [`Oem`] projection.
//!
//! The private `*Xml` mirror structs follow the XSD shape used by the CCSDS
//! official XML schema.  They map 1-to-1 onto the wire-format element/attribute
//! names via serde rename annotations.
//!
//! - [`read_oem`] — parse an XML string → [`Oem`]
//! - [`write_oem`] — serialise [`Oem`] → XML string

use std::collections::BTreeMap;

use lox_core::coords::Cartesian;
use lox_core::units::{Distance, Velocity};
use nalgebra::Matrix6;
use serde::{Deserialize, Serialize};

use crate::types::common::{OdmCenter, OdmFrame, OdmHeader, OdmTime};
use crate::types::oem::{Oem, OemCovariance, OemMetadata, OemSegment};
use crate::xml::error::XmlError;

// ---------------------------------------------------------------------------
// Helper — value with an optional `units` attribute
// ---------------------------------------------------------------------------

/// An XML leaf element that may carry a `units` attribute, e.g.
/// `<X units="km">7000.0</X>`.
///
/// The unit string is accepted on read for interop but is not validated.
/// On write we always emit the CCSDS-canonical fixed unit for the field.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct ValueWithUnits {
    #[serde(rename = "@units", skip_serializing_if = "Option::is_none")]
    units: Option<String>,
    #[serde(rename = "$text")]
    value: f64,
}

impl ValueWithUnits {
    fn new(value: f64, units: &str) -> Self {
        ValueWithUnits {
            units: Some(units.to_string()),
            value,
        }
    }
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct OdmHeaderXml {
    #[serde(rename = "COMMENT", skip_serializing_if = "Vec::is_empty")]
    comments: Vec<String>,
    #[serde(rename = "CLASSIFICATION", skip_serializing_if = "Option::is_none")]
    classification: Option<String>,
    #[serde(rename = "CREATION_DATE")]
    creation_date: String,
    #[serde(rename = "ORIGINATOR")]
    originator: String,
    #[serde(rename = "MESSAGE_ID", skip_serializing_if = "Option::is_none")]
    message_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct OemMetadataXml {
    #[serde(rename = "COMMENT", skip_serializing_if = "Vec::is_empty")]
    comments: Vec<String>,
    #[serde(rename = "OBJECT_NAME")]
    object_name: String,
    #[serde(rename = "OBJECT_ID")]
    object_id: String,
    #[serde(rename = "CENTER_NAME")]
    center_name: String,
    #[serde(rename = "REF_FRAME")]
    ref_frame: String,
    #[serde(rename = "REF_FRAME_EPOCH", skip_serializing_if = "Option::is_none")]
    ref_frame_epoch: Option<String>,
    #[serde(rename = "TIME_SYSTEM")]
    time_system: String,
    #[serde(rename = "START_TIME")]
    start_time: String,
    #[serde(rename = "USEABLE_START_TIME", skip_serializing_if = "Option::is_none")]
    useable_start_time: Option<String>,
    #[serde(rename = "USEABLE_STOP_TIME", skip_serializing_if = "Option::is_none")]
    useable_stop_time: Option<String>,
    #[serde(rename = "STOP_TIME")]
    stop_time: String,
    #[serde(rename = "INTERPOLATION", skip_serializing_if = "Option::is_none")]
    interpolation: Option<String>,
    #[serde(
        rename = "INTERPOLATION_DEGREE",
        skip_serializing_if = "Option::is_none"
    )]
    interpolation_degree: Option<u64>,
}

// ---------------------------------------------------------------------------
// State vector
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct StateVectorXml {
    #[serde(rename = "EPOCH")]
    epoch: String,
    #[serde(rename = "X")]
    x: ValueWithUnits,
    #[serde(rename = "Y")]
    y: ValueWithUnits,
    #[serde(rename = "Z")]
    z: ValueWithUnits,
    #[serde(rename = "X_DOT")]
    x_dot: ValueWithUnits,
    #[serde(rename = "Y_DOT")]
    y_dot: ValueWithUnits,
    #[serde(rename = "Z_DOT")]
    z_dot: ValueWithUnits,
}

// ---------------------------------------------------------------------------
// Covariance matrix
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct CovarianceMatrixXml {
    #[serde(rename = "COMMENT", skip_serializing_if = "Vec::is_empty")]
    comments: Vec<String>,
    #[serde(rename = "EPOCH")]
    epoch: String,
    #[serde(rename = "COV_REF_FRAME", skip_serializing_if = "Option::is_none")]
    cov_ref_frame: Option<String>,
    #[serde(rename = "CX_X")]
    cx_x: f64,
    #[serde(rename = "CY_X")]
    cy_x: f64,
    #[serde(rename = "CY_Y")]
    cy_y: f64,
    #[serde(rename = "CZ_X")]
    cz_x: f64,
    #[serde(rename = "CZ_Y")]
    cz_y: f64,
    #[serde(rename = "CZ_Z")]
    cz_z: f64,
    #[serde(rename = "CX_DOT_X")]
    cx_dot_x: f64,
    #[serde(rename = "CX_DOT_Y")]
    cx_dot_y: f64,
    #[serde(rename = "CX_DOT_Z")]
    cx_dot_z: f64,
    #[serde(rename = "CX_DOT_X_DOT")]
    cx_dot_x_dot: f64,
    #[serde(rename = "CY_DOT_X")]
    cy_dot_x: f64,
    #[serde(rename = "CY_DOT_Y")]
    cy_dot_y: f64,
    #[serde(rename = "CY_DOT_Z")]
    cy_dot_z: f64,
    #[serde(rename = "CY_DOT_X_DOT")]
    cy_dot_x_dot: f64,
    #[serde(rename = "CY_DOT_Y_DOT")]
    cy_dot_y_dot: f64,
    #[serde(rename = "CZ_DOT_X")]
    cz_dot_x: f64,
    #[serde(rename = "CZ_DOT_Y")]
    cz_dot_y: f64,
    #[serde(rename = "CZ_DOT_Z")]
    cz_dot_z: f64,
    #[serde(rename = "CZ_DOT_X_DOT")]
    cz_dot_x_dot: f64,
    #[serde(rename = "CZ_DOT_Y_DOT")]
    cz_dot_y_dot: f64,
    #[serde(rename = "CZ_DOT_Z_DOT")]
    cz_dot_z_dot: f64,
}

// ---------------------------------------------------------------------------
// Data block
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct OemDataXml {
    #[serde(rename = "COMMENT", skip_serializing_if = "Vec::is_empty")]
    comments: Vec<String>,
    #[serde(rename = "stateVector")]
    state_vector: Vec<StateVectorXml>,
    #[serde(rename = "covarianceMatrix", skip_serializing_if = "Vec::is_empty")]
    covariance_matrix: Vec<CovarianceMatrixXml>,
}

// ---------------------------------------------------------------------------
// Segment / Body / Root
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct OemSegmentXml {
    #[serde(rename = "metadata")]
    metadata: OemMetadataXml,
    #[serde(rename = "data")]
    data: OemDataXml,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct OemBodyXml {
    #[serde(rename = "segment")]
    segment: Vec<OemSegmentXml>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename = "oem")]
pub(crate) struct OemXml {
    #[serde(rename = "@xmlns:xsi", skip_serializing_if = "Option::is_none")]
    xmlns_xsi: Option<String>,
    #[serde(
        rename = "@xsi:noNamespaceSchemaLocation",
        skip_serializing_if = "Option::is_none"
    )]
    schema_location: Option<String>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(rename = "@version")]
    version: String,
    #[serde(rename = "header")]
    header: OdmHeaderXml,
    #[serde(rename = "body")]
    body: OemBodyXml,
}

// ---------------------------------------------------------------------------
// Typed → Wire (Oem → OemXml)
// ---------------------------------------------------------------------------

impl From<&Oem> for OemXml {
    fn from(oem: &Oem) -> Self {
        let header = OdmHeaderXml {
            comments: oem.header.comments.clone(),
            classification: oem.header.classification.clone(),
            creation_date: oem.header.creation_date.iso(),
            originator: oem.header.originator.clone(),
            message_id: oem.header.message_id.clone(),
        };

        let segments = oem
            .segments
            .iter()
            .map(|seg| {
                let meta = &seg.metadata;
                let time_system = meta.start_time.time_system();

                let metadata = OemMetadataXml {
                    comments: meta.comments.clone(),
                    object_name: meta.object_name.clone(),
                    object_id: meta.object_id.clone(),
                    center_name: meta.center.name().into_owned(),
                    ref_frame: meta.frame.name().into_owned(),
                    ref_frame_epoch: meta.frame_epoch.map(|e| e.iso()),
                    time_system: time_system.to_string(),
                    start_time: meta.start_time.iso(),
                    useable_start_time: meta.useable_start_time.map(|t| t.iso()),
                    useable_stop_time: meta.useable_stop_time.map(|t| t.iso()),
                    stop_time: meta.stop_time.iso(),
                    interpolation: meta.interpolation.clone(),
                    interpolation_degree: meta.interpolation_degree,
                };

                let state_vector = seg
                    .states
                    .iter()
                    .map(|(epoch, state)| {
                        let pos = state.position();
                        let vel = state.velocity();
                        StateVectorXml {
                            epoch: epoch.iso(),
                            x: ValueWithUnits::new(pos.x / 1000.0, "km"),
                            y: ValueWithUnits::new(pos.y / 1000.0, "km"),
                            z: ValueWithUnits::new(pos.z / 1000.0, "km"),
                            x_dot: ValueWithUnits::new(vel.x / 1000.0, "km/s"),
                            y_dot: ValueWithUnits::new(vel.y / 1000.0, "km/s"),
                            z_dot: ValueWithUnits::new(vel.z / 1000.0, "km/s"),
                        }
                    })
                    .collect();

                let covariance_matrix = seg
                    .covariance_history
                    .iter()
                    .map(|cov| {
                        let m = &cov.matrix;
                        CovarianceMatrixXml {
                            comments: cov.comments.clone(),
                            epoch: cov.epoch.iso(),
                            cov_ref_frame: cov.frame.as_ref().map(|f| f.name().into_owned()),
                            cx_x: m[(0, 0)],
                            cy_x: m[(1, 0)],
                            cy_y: m[(1, 1)],
                            cz_x: m[(2, 0)],
                            cz_y: m[(2, 1)],
                            cz_z: m[(2, 2)],
                            cx_dot_x: m[(3, 0)],
                            cx_dot_y: m[(3, 1)],
                            cx_dot_z: m[(3, 2)],
                            cx_dot_x_dot: m[(3, 3)],
                            cy_dot_x: m[(4, 0)],
                            cy_dot_y: m[(4, 1)],
                            cy_dot_z: m[(4, 2)],
                            cy_dot_x_dot: m[(4, 3)],
                            cy_dot_y_dot: m[(4, 4)],
                            cz_dot_x: m[(5, 0)],
                            cz_dot_y: m[(5, 1)],
                            cz_dot_z: m[(5, 2)],
                            cz_dot_x_dot: m[(5, 3)],
                            cz_dot_y_dot: m[(5, 4)],
                            cz_dot_z_dot: m[(5, 5)],
                        }
                    })
                    .collect();

                OemSegmentXml {
                    metadata,
                    data: OemDataXml {
                        comments: seg.data_comments.clone(),
                        state_vector,
                        covariance_matrix,
                    },
                }
            })
            .collect();

        OemXml {
            xmlns_xsi: Some("http://www.w3.org/2001/XMLSchema-instance".to_string()),
            schema_location: Some(
                "http://sanaregistry.org/r/ndmxml/ndmxml-1.0-master.xsd".to_string(),
            ),
            id: Some("CCSDS_OEM_VERS".to_string()),
            version: "3.0".to_string(),
            header,
            body: OemBodyXml { segment: segments },
        }
    }
}

// ---------------------------------------------------------------------------
// Wire → Typed (OemXml → Oem)
// ---------------------------------------------------------------------------

/// Parse an epoch string under the given time_system, returning an XmlError.
fn parse_epoch(time_system: &str, iso: &str) -> Result<OdmTime, XmlError> {
    OdmTime::from_wire(time_system, iso.trim()).map_err(|e| XmlError::InvalidEpoch {
        value: iso.to_string(),
        time_system: time_system.to_string(),
        reason: e.to_string(),
    })
}

/// Parse a covariance matrix XML block into an [`OemCovariance`].
fn parse_covariance(time_system: &str, cm: CovarianceMatrixXml) -> Result<OemCovariance, XmlError> {
    let epoch = parse_epoch(time_system, &cm.epoch)?;
    let mut matrix = Matrix6::<f64>::zeros();
    let lower: &[(f64, usize, usize)] = &[
        (cm.cx_x, 0, 0),
        (cm.cy_x, 1, 0),
        (cm.cy_y, 1, 1),
        (cm.cz_x, 2, 0),
        (cm.cz_y, 2, 1),
        (cm.cz_z, 2, 2),
        (cm.cx_dot_x, 3, 0),
        (cm.cx_dot_y, 3, 1),
        (cm.cx_dot_z, 3, 2),
        (cm.cx_dot_x_dot, 3, 3),
        (cm.cy_dot_x, 4, 0),
        (cm.cy_dot_y, 4, 1),
        (cm.cy_dot_z, 4, 2),
        (cm.cy_dot_x_dot, 4, 3),
        (cm.cy_dot_y_dot, 4, 4),
        (cm.cz_dot_x, 5, 0),
        (cm.cz_dot_y, 5, 1),
        (cm.cz_dot_z, 5, 2),
        (cm.cz_dot_x_dot, 5, 3),
        (cm.cz_dot_y_dot, 5, 4),
        (cm.cz_dot_z_dot, 5, 5),
    ];
    for &(v, row, col) in lower {
        matrix[(row, col)] = v;
        if row != col {
            matrix[(col, row)] = v;
        }
    }
    Ok(OemCovariance {
        comments: cm.comments,
        epoch,
        frame: cm.cov_ref_frame.map(|s| OdmFrame::from_wire(s.trim())),
        matrix,
    })
}

impl TryFrom<OemXml> for Oem {
    type Error = XmlError;

    fn try_from(xml: OemXml) -> Result<Self, Self::Error> {
        // ---- header ----
        // Use the first segment's time_system for the header epoch, or fall
        // back to "TAI" if there are no segments (degenerate case).
        let header_ts = xml
            .body
            .segment
            .first()
            .map(|s| s.metadata.time_system.as_str())
            .unwrap_or("TAI");

        let creation_date = parse_epoch(header_ts, &xml.header.creation_date)?;
        let header = OdmHeader {
            comments: xml.header.comments,
            classification: xml.header.classification,
            creation_date,
            originator: xml.header.originator.trim().to_string(),
            message_id: xml.header.message_id,
        };

        // ---- segments ----
        let mut segments: Vec<OemSegment> = Vec::with_capacity(xml.body.segment.len());
        for seg_xml in xml.body.segment {
            let meta_xml = seg_xml.metadata;
            let data_xml = seg_xml.data;
            let time_system = meta_xml.time_system.trim();

            // metadata
            let frame_epoch = meta_xml
                .ref_frame_epoch
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(|s| parse_epoch(time_system, s))
                .transpose()?;

            let start_time = parse_epoch(time_system, &meta_xml.start_time)?;
            let useable_start_time = meta_xml
                .useable_start_time
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(|s| parse_epoch(time_system, s))
                .transpose()?;
            let useable_stop_time = meta_xml
                .useable_stop_time
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(|s| parse_epoch(time_system, s))
                .transpose()?;
            let stop_time = parse_epoch(time_system, &meta_xml.stop_time)?;

            let metadata = OemMetadata {
                comments: meta_xml.comments,
                object_name: meta_xml.object_name.trim().to_string(),
                object_id: meta_xml.object_id.trim().to_string(),
                center: OdmCenter::from_wire(meta_xml.center_name.trim()),
                frame: OdmFrame::from_wire(meta_xml.ref_frame.trim()),
                frame_epoch,
                start_time,
                useable_start_time,
                useable_stop_time,
                stop_time,
                interpolation: meta_xml
                    .interpolation
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty()),
                interpolation_degree: meta_xml.interpolation_degree,
            };

            // state vectors
            let mut states: Vec<(OdmTime, Cartesian)> =
                Vec::with_capacity(data_xml.state_vector.len());
            for sv in data_xml.state_vector {
                let epoch = parse_epoch(time_system, &sv.epoch)?;
                let state = Cartesian::new(
                    Distance::kilometers(sv.x.value),
                    Distance::kilometers(sv.y.value),
                    Distance::kilometers(sv.z.value),
                    Velocity::kilometers_per_second(sv.x_dot.value),
                    Velocity::kilometers_per_second(sv.y_dot.value),
                    Velocity::kilometers_per_second(sv.z_dot.value),
                );
                states.push((epoch, state));
            }

            // covariance history
            let mut covariance_history: Vec<OemCovariance> =
                Vec::with_capacity(data_xml.covariance_matrix.len());
            for cm in data_xml.covariance_matrix {
                covariance_history.push(parse_covariance(time_system, cm)?);
            }

            segments.push(OemSegment {
                metadata,
                data_comments: data_xml.comments,
                states,
                covariance_history,
            });
        }

        Ok(Oem {
            header,
            segments,
            user_defined: BTreeMap::new(),
        })
    }
}

// ---------------------------------------------------------------------------
// Public free functions
// ---------------------------------------------------------------------------

/// Parse an OEM XML document into a typed [`Oem`].
pub fn read_oem(input: &str) -> Result<Oem, XmlError> {
    let xml: OemXml = quick_xml::de::from_str(input)?;
    Oem::try_from(xml)
}

/// Serialise a typed [`Oem`] to an XML string.
///
/// Returns [`XmlError::XmlSer`] if `quick-xml` rejects any field — most
/// realistically a non-finite `f64` (NaN/Infinity) in a numeric slot.
pub fn write_oem(oem: &Oem) -> Result<String, XmlError> {
    let xml = OemXml::from(oem);
    Ok(quick_xml::se::to_string(&xml)?)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use lox_bodies::DynOrigin;
    use lox_core::coords::Cartesian;
    use lox_core::units::{Distance, Velocity};
    use lox_frames::DynFrame;
    use nalgebra::Matrix6;

    use crate::types::common::{OdmCenter, OdmFrame, OdmHeader, OdmTime};
    use crate::types::oem::{Oem, OemCovariance, OemMetadata, OemSegment};

    use super::{read_oem, write_oem};

    fn tai_epoch() -> OdmTime {
        OdmTime::Time(lox_time::time::Time::j2000(
            lox_time::time_scales::DynTimeScale::Tai,
        ))
    }

    fn tai_epoch_plus(seconds: i64) -> OdmTime {
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

    fn sample_header() -> OdmHeader {
        OdmHeader {
            comments: Vec::new(),
            classification: None,
            creation_date: tai_epoch(),
            originator: "TEST".to_string(),
            message_id: None,
        }
    }

    fn sample_metadata() -> OemMetadata {
        OemMetadata {
            comments: Vec::new(),
            object_name: "TEST-SAT".to_string(),
            object_id: "2024-000A".to_string(),
            center: OdmCenter::Known(DynOrigin::Earth),
            frame: OdmFrame::Known(DynFrame::Icrf),
            frame_epoch: None,
            start_time: tai_epoch(),
            useable_start_time: None,
            useable_stop_time: None,
            stop_time: tai_epoch_plus(3600),
            interpolation: None,
            interpolation_degree: None,
        }
    }

    fn sample_segment() -> OemSegment {
        OemSegment {
            metadata: sample_metadata(),
            data_comments: Vec::new(),
            states: vec![
                (tai_epoch(), sample_state(0.0)),
                (tai_epoch_plus(60), sample_state(1.0)),
            ],
            covariance_history: Vec::new(),
        }
    }

    fn sample_oem() -> Oem {
        Oem {
            header: sample_header(),
            segments: vec![sample_segment()],
            user_defined: BTreeMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // 1. Minimal round-trip (one segment, 2 states, no covariance)
    // -----------------------------------------------------------------------

    #[test]
    fn minimal_oem_round_trip() {
        let oem = sample_oem();
        let xml_str = write_oem(&oem).unwrap();
        let parsed = read_oem(&xml_str).expect("round-trip parse failed");
        assert_eq!(oem, parsed, "minimal round-trip mismatch");
    }

    // -----------------------------------------------------------------------
    // 2. Multi-segment round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn multi_segment_round_trip() {
        let mut oem = sample_oem();
        // Second segment with a different center and frame
        let mut seg2 = sample_segment();
        seg2.metadata.object_name = "TEST-SAT-2".to_string();
        seg2.metadata.center = OdmCenter::Known(DynOrigin::Moon);
        seg2.states = vec![
            (tai_epoch_plus(3600), sample_state(5.0)),
            (tai_epoch_plus(7200), sample_state(10.0)),
            (tai_epoch_plus(10800), sample_state(15.0)),
        ];
        oem.segments.push(seg2);

        let xml_str = write_oem(&oem).unwrap();
        let parsed = read_oem(&xml_str).expect("multi-segment round-trip parse failed");
        assert_eq!(oem, parsed, "multi-segment round-trip mismatch");
        assert_eq!(parsed.segments.len(), 2);
        assert_eq!(parsed.segments[1].states.len(), 3);
    }

    // -----------------------------------------------------------------------
    // 3. Round-trip with one covariance entry
    // -----------------------------------------------------------------------

    #[test]
    fn covariance_round_trip() {
        let mut oem = sample_oem();
        let mut matrix = Matrix6::<f64>::zeros();
        // Fill lower triangle with distinct values
        for row in 0..6usize {
            for col in 0..=row {
                let v = (row * 10 + col) as f64 * 0.01 + 0.01;
                matrix[(row, col)] = v;
                matrix[(col, row)] = v;
            }
        }
        oem.segments[0].covariance_history.push(OemCovariance {
            comments: vec!["cov comment".to_string()],
            epoch: tai_epoch_plus(30),
            frame: Some(OdmFrame::Known(DynFrame::J2000)),
            matrix,
        });

        let xml_str = write_oem(&oem).unwrap();
        let parsed = read_oem(&xml_str).expect("covariance round-trip parse failed");
        assert_eq!(oem, parsed, "covariance round-trip mismatch");
        assert_eq!(parsed.segments[0].covariance_history.len(), 1);
        let cov = &parsed.segments[0].covariance_history[0];
        assert_eq!(cov.comments, vec!["cov comment".to_string()]);
        // Matrix symmetry preserved
        assert_eq!(cov.matrix[(1, 0)], cov.matrix[(0, 1)]);
        assert_eq!(cov.matrix[(5, 0)], cov.matrix[(0, 5)]);
    }

    // -----------------------------------------------------------------------
    // 4. Comments preserved in metadata and data blocks
    // -----------------------------------------------------------------------

    #[test]
    fn comments_preserved_in_metadata_and_data() {
        let mut oem = sample_oem();
        oem.header.comments.push("Header comment".to_string());
        oem.segments[0]
            .metadata
            .comments
            .push("Metadata comment".to_string());
        oem.segments[0]
            .data_comments
            .push("Data block comment".to_string());

        let xml_str = write_oem(&oem).unwrap();
        let parsed = read_oem(&xml_str).expect("comments round-trip parse failed");
        assert_eq!(parsed.header.comments, vec!["Header comment".to_string()]);
        assert_eq!(
            parsed.segments[0].metadata.comments,
            vec!["Metadata comment".to_string()]
        );
        assert_eq!(
            parsed.segments[0].data_comments,
            vec!["Data block comment".to_string()]
        );
    }

    // -----------------------------------------------------------------------
    // 5. Optional metadata fields (interpolation, useable times, ref_frame_epoch)
    // -----------------------------------------------------------------------

    #[test]
    fn optional_metadata_fields_round_trip() {
        let mut oem = sample_oem();
        oem.segments[0].metadata.interpolation = Some("HERMITE".to_string());
        oem.segments[0].metadata.interpolation_degree = Some(7);
        oem.segments[0].metadata.useable_start_time = Some(tai_epoch_plus(10));
        oem.segments[0].metadata.useable_stop_time = Some(tai_epoch_plus(3590));
        oem.segments[0].metadata.frame_epoch = Some(tai_epoch_plus(0));

        let xml_str = write_oem(&oem).unwrap();
        let parsed = read_oem(&xml_str).expect("optional metadata round-trip parse failed");
        assert_eq!(oem, parsed, "optional metadata round-trip mismatch");
        assert_eq!(
            parsed.segments[0].metadata.interpolation.as_deref(),
            Some("HERMITE")
        );
        assert_eq!(parsed.segments[0].metadata.interpolation_degree, Some(7));
        assert!(parsed.segments[0].metadata.useable_start_time.is_some());
        assert!(parsed.segments[0].metadata.useable_stop_time.is_some());
    }

    // -----------------------------------------------------------------------
    // 6. Multiple covariance entries in one segment
    // -----------------------------------------------------------------------

    #[test]
    fn multiple_covariance_entries_preserved() {
        let mut oem = sample_oem();
        oem.segments[0]
            .states
            .push((tai_epoch_plus(120), sample_state(2.0)));
        for i in 0..3usize {
            oem.segments[0].covariance_history.push(OemCovariance {
                comments: Vec::new(),
                epoch: tai_epoch_plus((i as i64) * 60),
                frame: None,
                matrix: Matrix6::identity() * ((i + 1) as f64),
            });
        }

        let xml_str = write_oem(&oem).unwrap();
        let parsed = read_oem(&xml_str).expect("multiple covariance parse failed");
        assert_eq!(parsed.segments[0].covariance_history.len(), 3);
        assert_eq!(parsed.segments[0].covariance_history[2].matrix[(0, 0)], 3.0);
    }
}
