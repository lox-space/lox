// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! XML ↔ typed [`Omm`] projection.
//!
//! The private `*Xml` mirror structs follow the XSD shape used by the CCSDS
//! official XML schema.  They map 1-to-1 onto the wire-format element/attribute
//! names via serde rename annotations.
//!
//! - [`read_omm`] — parse an XML string → [`Omm`]
//! - [`write_omm`] — serialise [`Omm`] → XML string

use std::collections::BTreeMap;
use std::f64::consts::PI;

use lox_bodies::TryPointMass;
use lox_core::elements::{GravitationalParameter, MeanElements};
use lox_core::units::{Area, AreaToMass, Mass};
use nalgebra::Matrix6;
use serde::{Deserialize, Serialize};

use crate::types::common::{
    Covariance, OdmCenter, OdmFrame, OdmHeader, OdmTime, SpacecraftParameters,
};
use crate::types::omm::{Omm, OmmMeanElements, OmmMetadata, TleParameters};
use crate::xml::error::XmlError;

// ---------------------------------------------------------------------------
// Helper — value with an optional `units` attribute
// ---------------------------------------------------------------------------

/// An XML leaf element that may carry a `units` attribute, e.g.
/// `<SEMI_MAJOR_AXIS units="km">6860.0</SEMI_MAJOR_AXIS>`.
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
// User-defined parameter
// ---------------------------------------------------------------------------

/// `<USER_DEFINED parameter="KEY">VALUE</USER_DEFINED>`
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct UserDefinedParameterXml {
    #[serde(rename = "@parameter")]
    parameter: String,
    #[serde(rename = "$text")]
    value: String,
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
pub(crate) struct OmmMetadataXml {
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
    #[serde(rename = "MEAN_ELEMENT_THEORY")]
    mean_element_theory: String,
}

// ---------------------------------------------------------------------------
// Mean elements block
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct MeanElementsXml {
    #[serde(rename = "COMMENT", skip_serializing_if = "Vec::is_empty")]
    comments: Vec<String>,
    #[serde(rename = "EPOCH")]
    epoch: String,
    /// Mutually exclusive with `mean_motion`.  On write always emit SMA.
    #[serde(rename = "SEMI_MAJOR_AXIS", skip_serializing_if = "Option::is_none")]
    semi_major_axis: Option<ValueWithUnits>,
    /// Present on read when the source provides `MEAN_MOTION` instead of SMA.
    #[serde(rename = "MEAN_MOTION", skip_serializing_if = "Option::is_none")]
    mean_motion: Option<ValueWithUnits>,
    #[serde(rename = "ECCENTRICITY")]
    eccentricity: f64,
    #[serde(rename = "INCLINATION")]
    inclination: ValueWithUnits,
    #[serde(rename = "RA_OF_ASC_NODE")]
    ra_of_asc_node: ValueWithUnits,
    #[serde(rename = "ARG_OF_PERICENTER")]
    arg_of_pericenter: ValueWithUnits,
    #[serde(rename = "MEAN_ANOMALY")]
    mean_anomaly: ValueWithUnits,
    #[serde(rename = "GM", skip_serializing_if = "Option::is_none")]
    gm: Option<ValueWithUnits>,
}

// ---------------------------------------------------------------------------
// TLE parameters block
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct TleParametersXml {
    #[serde(rename = "COMMENT", skip_serializing_if = "Vec::is_empty")]
    comments: Vec<String>,
    #[serde(rename = "EPHEMERIS_TYPE", skip_serializing_if = "Option::is_none")]
    ephemeris_type: Option<i32>,
    #[serde(
        rename = "CLASSIFICATION_TYPE",
        skip_serializing_if = "Option::is_none"
    )]
    classification_type: Option<String>,
    #[serde(rename = "NORAD_CAT_ID", skip_serializing_if = "Option::is_none")]
    norad_cat_id: Option<i32>,
    #[serde(rename = "ELEMENT_SET_NO", skip_serializing_if = "Option::is_none")]
    element_set_no: Option<i64>,
    #[serde(rename = "REV_AT_EPOCH", skip_serializing_if = "Option::is_none")]
    rev_at_epoch: Option<u64>,
    #[serde(rename = "BSTAR", skip_serializing_if = "Option::is_none")]
    bstar: Option<f64>,
    #[serde(rename = "BTERM", skip_serializing_if = "Option::is_none")]
    bterm: Option<ValueWithUnits>,
    #[serde(rename = "MEAN_MOTION_DOT", skip_serializing_if = "Option::is_none")]
    mean_motion_dot: Option<f64>,
    #[serde(rename = "MEAN_MOTION_DDOT", skip_serializing_if = "Option::is_none")]
    mean_motion_ddot: Option<f64>,
    #[serde(rename = "AGOM", skip_serializing_if = "Option::is_none")]
    agom: Option<ValueWithUnits>,
}

// ---------------------------------------------------------------------------
// Spacecraft parameters
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct SpacecraftParametersXml {
    #[serde(rename = "COMMENT", skip_serializing_if = "Vec::is_empty")]
    comments: Vec<String>,
    #[serde(rename = "MASS", skip_serializing_if = "Option::is_none")]
    mass: Option<ValueWithUnits>,
    #[serde(rename = "SOLAR_RAD_AREA", skip_serializing_if = "Option::is_none")]
    solar_rad_area: Option<ValueWithUnits>,
    #[serde(rename = "SOLAR_RAD_COEFF", skip_serializing_if = "Option::is_none")]
    solar_rad_coeff: Option<f64>,
    #[serde(rename = "DRAG_AREA", skip_serializing_if = "Option::is_none")]
    drag_area: Option<ValueWithUnits>,
    #[serde(rename = "DRAG_COEFF", skip_serializing_if = "Option::is_none")]
    drag_coeff: Option<f64>,
}

// ---------------------------------------------------------------------------
// Covariance matrix
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct CovarianceMatrixXml {
    #[serde(rename = "COMMENT", skip_serializing_if = "Vec::is_empty")]
    comments: Vec<String>,
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
// User-defined parameters container
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct UserDefinedParametersXml {
    #[serde(rename = "USER_DEFINED", skip_serializing_if = "Vec::is_empty")]
    user_defined: Vec<UserDefinedParameterXml>,
}

// ---------------------------------------------------------------------------
// Data block
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct OmmDataXml {
    #[serde(rename = "meanElements")]
    mean_elements: MeanElementsXml,
    #[serde(rename = "tleParameters", skip_serializing_if = "Option::is_none")]
    tle_parameters: Option<TleParametersXml>,
    #[serde(
        rename = "spacecraftParameters",
        skip_serializing_if = "Option::is_none"
    )]
    spacecraft_parameters: Option<SpacecraftParametersXml>,
    #[serde(rename = "covarianceMatrix", skip_serializing_if = "Option::is_none")]
    covariance_matrix: Option<CovarianceMatrixXml>,
    #[serde(
        rename = "userDefinedParameters",
        skip_serializing_if = "Option::is_none"
    )]
    user_defined_parameters: Option<UserDefinedParametersXml>,
}

// ---------------------------------------------------------------------------
// Segment / Body / Root
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct OmmSegmentXml {
    #[serde(rename = "metadata")]
    metadata: OmmMetadataXml,
    #[serde(rename = "data")]
    data: OmmDataXml,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct OmmBodyXml {
    #[serde(rename = "segment")]
    segment: OmmSegmentXml,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename = "omm")]
pub(crate) struct OmmXml {
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
    body: OmmBodyXml,
}

// ---------------------------------------------------------------------------
// Typed → Wire (Omm → OmmXml)
// ---------------------------------------------------------------------------

impl From<&Omm> for OmmXml {
    fn from(omm: &Omm) -> Self {
        let header = OdmHeaderXml {
            comments: omm.header.comments.clone(),
            classification: omm.header.classification.clone(),
            creation_date: omm.header.creation_date.iso(),
            originator: omm.header.originator.clone(),
            message_id: omm.header.message_id.clone(),
        };

        let metadata = OmmMetadataXml {
            comments: omm.metadata.comments.clone(),
            object_name: omm.metadata.object_name.clone(),
            object_id: omm.metadata.object_id.clone(),
            center_name: omm.metadata.center.name().into_owned(),
            ref_frame: omm.metadata.frame.name().into_owned(),
            ref_frame_epoch: omm.metadata.frame_epoch.map(|e| e.iso()),
            time_system: omm.epoch.time_system().to_string(),
            mean_element_theory: omm.metadata.mean_element_theory.clone(),
        };

        let el = &omm.mean_elements.elements;
        let mean_elements = MeanElementsXml {
            comments: omm.mean_elements.comments.clone(),
            epoch: omm.epoch.iso(),
            semi_major_axis: Some(ValueWithUnits::new(el.a / 1000.0, "km")),
            mean_motion: None,
            eccentricity: el.e,
            inclination: ValueWithUnits::new(el.i.to_degrees(), "deg"),
            ra_of_asc_node: ValueWithUnits::new(el.raan.to_degrees(), "deg"),
            arg_of_pericenter: ValueWithUnits::new(el.aop.to_degrees(), "deg"),
            mean_anomaly: ValueWithUnits::new(el.m.to_degrees(), "deg"),
            gm: omm
                .mean_elements
                .gm
                .map(|g| ValueWithUnits::new(g.as_f64() / 1e9, "km**3/s**2")),
        };

        let tle_parameters = omm.tle_parameters.as_ref().map(|tle| TleParametersXml {
            comments: tle.comments.clone(),
            ephemeris_type: tle.ephemeris_type,
            classification_type: tle.classification_type.clone(),
            norad_cat_id: tle.norad_cat_id,
            element_set_no: tle.element_set_no,
            rev_at_epoch: tle.rev_at_epoch,
            bstar: tle.bstar,
            bterm: tle
                .bterm
                .map(|b| ValueWithUnits::new(b.to_square_meters_per_kilogram(), "m**2/kg")),
            mean_motion_dot: tle.mean_motion_dot,
            mean_motion_ddot: tle.mean_motion_ddot,
            agom: tle
                .agom
                .map(|a| ValueWithUnits::new(a.to_square_meters_per_kilogram(), "m**2/kg")),
        });

        let spacecraft_parameters = omm.spacecraft.as_ref().map(|sp| SpacecraftParametersXml {
            comments: sp.comments.clone(),
            mass: sp.mass.map(|m| ValueWithUnits::new(m.to_kilograms(), "kg")),
            solar_rad_area: sp
                .solar_rad_area
                .map(|a| ValueWithUnits::new(a.to_square_meters(), "m**2")),
            solar_rad_coeff: sp.solar_rad_coeff,
            drag_area: sp
                .drag_area
                .map(|a| ValueWithUnits::new(a.to_square_meters(), "m**2")),
            drag_coeff: sp.drag_coeff,
        });

        let covariance_matrix = omm.covariance.as_ref().map(|cov| {
            let m = &cov.matrix;
            CovarianceMatrixXml {
                comments: cov.comments.clone(),
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
        });

        let user_defined_parameters = if omm.user_defined.is_empty() {
            None
        } else {
            Some(UserDefinedParametersXml {
                user_defined: omm
                    .user_defined
                    .iter()
                    .map(|(k, v)| UserDefinedParameterXml {
                        parameter: k.clone(),
                        value: v.clone(),
                    })
                    .collect(),
            })
        };

        OmmXml {
            xmlns_xsi: Some("http://www.w3.org/2001/XMLSchema-instance".to_string()),
            schema_location: Some(
                "http://sanaregistry.org/r/ndmxml/ndmxml-1.0-master.xsd".to_string(),
            ),
            id: Some("CCSDS_OMM_VERS".to_string()),
            version: "3.0".to_string(),
            header,
            body: OmmBodyXml {
                segment: OmmSegmentXml {
                    metadata,
                    data: OmmDataXml {
                        mean_elements,
                        tle_parameters,
                        spacecraft_parameters,
                        covariance_matrix,
                        user_defined_parameters,
                    },
                },
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Wire → Typed (OmmXml → Omm)
// ---------------------------------------------------------------------------

/// Parse an epoch string under the given time_system, returning an XmlError.
fn parse_epoch(time_system: &str, iso: &str) -> Result<OdmTime, XmlError> {
    OdmTime::from_wire(time_system, iso.trim()).map_err(|e| XmlError::InvalidEpoch {
        value: iso.to_string(),
        time_system: time_system.to_string(),
        reason: e.to_string(),
    })
}

impl TryFrom<OmmXml> for Omm {
    type Error = XmlError;

    fn try_from(xml: OmmXml) -> Result<Self, Self::Error> {
        let seg = xml.body.segment;
        let meta = seg.metadata;
        let data = seg.data;

        // ---- time system (needed to parse all epochs) ----
        let time_system = meta.time_system.trim();

        // ---- header ----
        let creation_date = parse_epoch(time_system, &xml.header.creation_date)?;
        let header = OdmHeader {
            comments: xml.header.comments,
            classification: xml.header.classification,
            creation_date,
            originator: xml.header.originator.trim().to_string(),
            message_id: xml.header.message_id,
        };

        // ---- metadata ----
        let frame_epoch = meta
            .ref_frame_epoch
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(|s| parse_epoch(time_system, s))
            .transpose()?;

        let metadata = OmmMetadata {
            comments: meta.comments,
            object_name: meta.object_name.trim().to_string(),
            object_id: meta.object_id.trim().to_string(),
            center: OdmCenter::from_wire(meta.center_name.trim()),
            frame: OdmFrame::from_wire(meta.ref_frame.trim()),
            frame_epoch,
            mean_element_theory: meta.mean_element_theory.trim().to_string(),
        };

        // ---- mean elements ----
        let me = &data.mean_elements;
        let epoch = parse_epoch(time_system, &me.epoch)?;

        // Wire GM (optional) — needed for MEAN_MOTION → SMA conversion.
        let wire_gm = me
            .gm
            .as_ref()
            .map(|v| GravitationalParameter::km3_per_s2(v.value));

        // Resolve GM for MEAN_MOTION conversion:
        //  1. Wire GM (preferred)
        //  2. Canonical body GM from center
        //  3. Error if neither available
        let resolve_gm_for_mean_motion = || -> Result<f64, XmlError> {
            if let Some(gm) = wire_gm {
                return Ok(gm.as_f64());
            }
            metadata
                .center
                .known()
                .and_then(|o| o.try_gravitational_parameter().ok())
                .map(|gm| gm.as_f64())
                .ok_or_else(|| XmlError::MissingRequiredField("GM".to_string()))
        };

        // Prefer SEMI_MAJOR_AXIS; fall back to MEAN_MOTION with GM conversion.
        let a_m = if let Some(sma) = &me.semi_major_axis {
            sma.value * 1000.0
        } else if let Some(mm) = &me.mean_motion {
            // MEAN_MOTION in rev/day → SMA in meters via Kepler's third law.
            let mu = resolve_gm_for_mean_motion()?;
            let n = mm.value * 2.0 * PI / 86400.0; // rad/s
            (mu / (n * n)).cbrt()
        } else {
            return Err(XmlError::MissingRequiredField(
                "SEMI_MAJOR_AXIS".to_string(),
            ));
        };

        let elements = MeanElements {
            a: a_m,
            e: me.eccentricity,
            i: me.inclination.value.to_radians(),
            raan: me.ra_of_asc_node.value.to_radians(),
            aop: me.arg_of_pericenter.value.to_radians(),
            m: me.mean_anomaly.value.to_radians(),
        };

        let mean_elements = OmmMeanElements {
            comments: me.comments.clone(),
            elements,
            gm: wire_gm,
        };

        // ---- TLE parameters (optional) ----
        let tle_parameters = data.tle_parameters.map(|tle| TleParameters {
            comments: tle.comments,
            ephemeris_type: tle.ephemeris_type,
            classification_type: tle.classification_type,
            norad_cat_id: tle.norad_cat_id,
            element_set_no: tle.element_set_no,
            rev_at_epoch: tle.rev_at_epoch,
            bstar: tle.bstar,
            bterm: tle
                .bterm
                .map(|v| AreaToMass::square_meters_per_kilogram(v.value)),
            mean_motion_dot: tle.mean_motion_dot,
            mean_motion_ddot: tle.mean_motion_ddot,
            agom: tle
                .agom
                .map(|v| AreaToMass::square_meters_per_kilogram(v.value)),
        });

        // ---- spacecraft parameters (optional) ----
        let spacecraft = data.spacecraft_parameters.map(|sp| SpacecraftParameters {
            comments: sp.comments,
            mass: sp.mass.map(|v| Mass::kilograms(v.value)),
            solar_rad_area: sp.solar_rad_area.map(|v| Area::square_meters(v.value)),
            solar_rad_coeff: sp.solar_rad_coeff,
            drag_area: sp.drag_area.map(|v| Area::square_meters(v.value)),
            drag_coeff: sp.drag_coeff,
        });

        // ---- covariance matrix (optional) ----
        let covariance = data.covariance_matrix.map(|cm| {
            let mut matrix = Matrix6::<f64>::zeros();
            let lower: &[(&str, f64, usize, usize)] = &[
                ("CX_X", cm.cx_x, 0, 0),
                ("CY_X", cm.cy_x, 1, 0),
                ("CY_Y", cm.cy_y, 1, 1),
                ("CZ_X", cm.cz_x, 2, 0),
                ("CZ_Y", cm.cz_y, 2, 1),
                ("CZ_Z", cm.cz_z, 2, 2),
                ("CX_DOT_X", cm.cx_dot_x, 3, 0),
                ("CX_DOT_Y", cm.cx_dot_y, 3, 1),
                ("CX_DOT_Z", cm.cx_dot_z, 3, 2),
                ("CX_DOT_X_DOT", cm.cx_dot_x_dot, 3, 3),
                ("CY_DOT_X", cm.cy_dot_x, 4, 0),
                ("CY_DOT_Y", cm.cy_dot_y, 4, 1),
                ("CY_DOT_Z", cm.cy_dot_z, 4, 2),
                ("CY_DOT_X_DOT", cm.cy_dot_x_dot, 4, 3),
                ("CY_DOT_Y_DOT", cm.cy_dot_y_dot, 4, 4),
                ("CZ_DOT_X", cm.cz_dot_x, 5, 0),
                ("CZ_DOT_Y", cm.cz_dot_y, 5, 1),
                ("CZ_DOT_Z", cm.cz_dot_z, 5, 2),
                ("CZ_DOT_X_DOT", cm.cz_dot_x_dot, 5, 3),
                ("CZ_DOT_Y_DOT", cm.cz_dot_y_dot, 5, 4),
                ("CZ_DOT_Z_DOT", cm.cz_dot_z_dot, 5, 5),
            ];
            for &(_, v, row, col) in lower {
                matrix[(row, col)] = v;
                if row != col {
                    matrix[(col, row)] = v;
                }
            }
            Covariance {
                comments: cm.comments,
                frame: cm.cov_ref_frame.map(|s| OdmFrame::from_wire(s.trim())),
                matrix,
            }
        });

        // ---- user-defined parameters ----
        let mut user_defined: BTreeMap<String, String> = BTreeMap::new();
        if let Some(udp) = data.user_defined_parameters {
            for p in udp.user_defined {
                user_defined.insert(p.parameter, p.value);
            }
        }

        Ok(Omm {
            header,
            metadata,
            epoch,
            mean_elements,
            tle_parameters,
            spacecraft,
            covariance,
            user_defined,
            provider_extras: BTreeMap::new(),
        })
    }
}

// ---------------------------------------------------------------------------
// Public free functions
// ---------------------------------------------------------------------------

/// Parse an OMM XML document into a typed [`Omm`].
pub fn read_omm(input: &str) -> Result<Omm, XmlError> {
    let xml: OmmXml = quick_xml::de::from_str(input)?;
    Omm::try_from(xml)
}

/// Serialise a typed [`Omm`] to an XML string.
///
/// Returns [`XmlError::XmlSer`] if `quick-xml` rejects any field — most
/// realistically a non-finite `f64` (NaN/Infinity) in a numeric slot.
pub fn write_omm(omm: &Omm) -> Result<String, XmlError> {
    let xml = OmmXml::from(omm);
    Ok(quick_xml::se::to_string(&xml)?)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::f64::consts::PI;

    use lox_approx::assert_approx_eq;
    use lox_bodies::DynOrigin;
    use lox_core::elements::{GravitationalParameter, MeanElements};
    use lox_core::units::{Area, AreaToMass, Mass};
    use lox_frames::DynFrame;
    use nalgebra::Matrix6;

    use crate::types::common::{Covariance, OdmCenter, OdmFrame, OdmHeader, OdmTime};
    use crate::types::omm::{Omm, OmmMeanElements, OmmMetadata, TleParameters};
    use crate::xml::error::XmlError;

    use super::{read_omm, write_omm};

    fn sample_epoch() -> OdmTime {
        OdmTime::Time(lox_time::time::Time::j2000(
            lox_time::time_scales::DynTimeScale::Tai,
        ))
    }

    fn sample_mean_elements() -> OmmMeanElements {
        OmmMeanElements {
            comments: Vec::new(),
            elements: MeanElements {
                a: 6_859_961.0, // m (~482 km altitude)
                e: 0.001_335_6,
                i: 1.697_775,    // rad (~97.3 deg)
                raan: 1.159_523, // rad
                aop: 1.931_018,  // rad
                m: 5.842_034,    // rad
            },
            gm: None,
        }
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
            mean_elements: sample_mean_elements(),
            tle_parameters: None,
            spacecraft: None,
            covariance: None,
            user_defined: BTreeMap::new(),
            provider_extras: BTreeMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // 1. Minimal OMM round-trip (no TLE, no spacecraft, no covariance)
    // -----------------------------------------------------------------------

    #[test]
    fn minimal_omm_round_trip() {
        let omm = sample_omm();
        let xml_str = write_omm(&omm).unwrap();
        let parsed = read_omm(&xml_str).expect("round-trip parse failed");

        assert_eq!(parsed.header, omm.header, "header mismatch");
        assert_eq!(parsed.metadata, omm.metadata, "metadata mismatch");
        assert_eq!(parsed.mean_elements.gm, omm.mean_elements.gm, "gm mismatch");
        assert!(
            (parsed.mean_elements.elements.a - omm.mean_elements.elements.a).abs() < 1.0,
            "SMA round-trip error too large"
        );
        let eps = 1e-9_f64;
        assert!(
            (parsed.mean_elements.elements.e - omm.mean_elements.elements.e).abs() < eps,
            "eccentricity round-trip error"
        );
        assert!(parsed.tle_parameters.is_none());
        assert!(parsed.spacecraft.is_none());
        assert!(parsed.covariance.is_none());
    }

    // -----------------------------------------------------------------------
    // 2. Round-trip with TLE parameters populated
    // -----------------------------------------------------------------------

    #[test]
    fn round_trip_with_tle_parameters() {
        let mut omm = sample_omm();
        omm.tle_parameters = Some(TleParameters {
            comments: vec!["TLE block comment".to_string()],
            ephemeris_type: Some(0),
            classification_type: Some("U".to_string()),
            norad_cat_id: Some(45018),
            element_set_no: Some(999),
            rev_at_epoch: Some(5327),
            bstar: Some(8.4553e-5),
            bterm: Some(AreaToMass::square_meters_per_kilogram(0.05)),
            mean_motion_dot: Some(2.241e-5),
            mean_motion_ddot: Some(0.0),
            agom: Some(AreaToMass::square_meters_per_kilogram(0.03)),
        });

        let xml_str = write_omm(&omm).unwrap();
        let parsed = read_omm(&xml_str).expect("TLE round-trip parse failed");

        let tle = parsed.tle_parameters.expect("TLE parameters missing");
        assert_eq!(tle.ephemeris_type, Some(0));
        assert_eq!(tle.classification_type.as_deref(), Some("U"));
        assert_eq!(tle.norad_cat_id, Some(45018));
        assert_eq!(tle.element_set_no, Some(999));
        assert_eq!(tle.rev_at_epoch, Some(5327));
        assert_approx_eq!(tle.bstar, Some(8.4553e-5), atol <= 1e-12);
        assert_approx_eq!(
            tle.bterm.map(|v| v.to_square_meters_per_kilogram()),
            Some(0.05),
            atol <= 1e-12
        );
        assert_approx_eq!(tle.mean_motion_dot, Some(2.241e-5), atol <= 1e-12);
        assert_eq!(tle.mean_motion_ddot, Some(0.0));
        assert_approx_eq!(
            tle.agom.map(|v| v.to_square_meters_per_kilogram()),
            Some(0.03),
            atol <= 1e-12
        );
    }

    // -----------------------------------------------------------------------
    // 3. Round-trip with GM preserved
    // -----------------------------------------------------------------------

    #[test]
    fn round_trip_with_gm_preserved() {
        let wire_gm = GravitationalParameter::km3_per_s2(398600.4415);
        let mut omm = sample_omm();
        omm.mean_elements.gm = Some(wire_gm);

        let xml_str = write_omm(&omm).unwrap();
        let parsed = read_omm(&xml_str).expect("GM round-trip parse failed");

        let parsed_gm = parsed.mean_elements.gm.expect("wire GM not preserved");
        let diff = (parsed_gm.as_f64() - wire_gm.as_f64()).abs();
        assert!(
            diff < 1.0,
            "GM round-trip error too large: {diff} m³/s²; parsed={parsed_gm}, original={wire_gm}"
        );
    }

    // -----------------------------------------------------------------------
    // 4. MEAN_MOTION input (Known Earth center, no wire GM) — succeeds, SMA computed
    // -----------------------------------------------------------------------

    #[test]
    fn mean_motion_input_known_earth_center_succeeds() {
        let xml = r#"<omm version="3.0">
            <header>
                <CREATION_DATE>2024-01-01T00:00:00</CREATION_DATE>
                <ORIGINATOR>TEST</ORIGINATOR>
            </header>
            <body>
                <segment>
                    <metadata>
                        <OBJECT_NAME>TEST-SAT</OBJECT_NAME>
                        <OBJECT_ID>2024-001A</OBJECT_ID>
                        <CENTER_NAME>EARTH</CENTER_NAME>
                        <REF_FRAME>TEME</REF_FRAME>
                        <TIME_SYSTEM>TAI</TIME_SYSTEM>
                        <MEAN_ELEMENT_THEORY>SGP/SGP4</MEAN_ELEMENT_THEORY>
                    </metadata>
                    <data>
                        <meanElements>
                            <EPOCH>2024-01-01T00:00:00</EPOCH>
                            <MEAN_MOTION units="rev/day">15.5</MEAN_MOTION>
                            <ECCENTRICITY>0.001</ECCENTRICITY>
                            <INCLINATION units="deg">45.0</INCLINATION>
                            <RA_OF_ASC_NODE units="deg">0.0</RA_OF_ASC_NODE>
                            <ARG_OF_PERICENTER units="deg">0.0</ARG_OF_PERICENTER>
                            <MEAN_ANOMALY units="deg">0.0</MEAN_ANOMALY>
                        </meanElements>
                    </data>
                </segment>
            </body>
        </omm>"#;

        let parsed = read_omm(xml).expect("MEAN_MOTION parse should succeed for Earth center");

        // No wire GM stored
        assert!(parsed.mean_elements.gm.is_none());

        // SMA should be computed: 15.5 rev/day ≈ reasonable LEO/MEO regime
        let a_km = parsed.mean_elements.elements.a / 1000.0;
        assert!(
            a_km > 5000.0 && a_km < 20000.0,
            "computed SMA = {a_km} km, expected 5000–20000 km"
        );

        // Verify via explicit formula
        use lox_bodies::TryPointMass;
        let mu = DynOrigin::Earth
            .try_gravitational_parameter()
            .unwrap()
            .as_f64();
        let n = 15.5 * 2.0 * PI / 86400.0;
        let expected_a_m = (mu / (n * n)).cbrt();
        assert!(
            (parsed.mean_elements.elements.a - expected_a_m).abs() < 1.0,
            "SMA mismatch vs expected"
        );
    }

    // -----------------------------------------------------------------------
    // 5. MEAN_MOTION input + Custom center + no wire GM → error
    // -----------------------------------------------------------------------

    #[test]
    fn mean_motion_custom_center_no_gm_returns_error() {
        let xml = r#"<omm version="3.0">
            <header>
                <CREATION_DATE>2024-01-01T00:00:00</CREATION_DATE>
                <ORIGINATOR>TEST</ORIGINATOR>
            </header>
            <body>
                <segment>
                    <metadata>
                        <OBJECT_NAME>TEST-SAT</OBJECT_NAME>
                        <OBJECT_ID>2024-001A</OBJECT_ID>
                        <CENTER_NAME>APOPHIS</CENTER_NAME>
                        <REF_FRAME>TEME</REF_FRAME>
                        <TIME_SYSTEM>TAI</TIME_SYSTEM>
                        <MEAN_ELEMENT_THEORY>SGP/SGP4</MEAN_ELEMENT_THEORY>
                    </metadata>
                    <data>
                        <meanElements>
                            <EPOCH>2024-01-01T00:00:00</EPOCH>
                            <MEAN_MOTION units="rev/day">15.5</MEAN_MOTION>
                            <ECCENTRICITY>0.001</ECCENTRICITY>
                            <INCLINATION units="deg">45.0</INCLINATION>
                            <RA_OF_ASC_NODE units="deg">0.0</RA_OF_ASC_NODE>
                            <ARG_OF_PERICENTER units="deg">0.0</ARG_OF_PERICENTER>
                            <MEAN_ANOMALY units="deg">0.0</MEAN_ANOMALY>
                        </meanElements>
                    </data>
                </segment>
            </body>
        </omm>"#;

        let err = read_omm(xml).expect_err("should fail: custom center + no GM");
        assert!(
            matches!(&err, XmlError::MissingRequiredField(k) if k == "GM"),
            "unexpected error kind: {err}"
        );
    }

    // -----------------------------------------------------------------------
    // 6. Comments preserved through round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn comments_preserved_through_round_trip() {
        let mut omm = sample_omm();
        omm.header.comments.push("Header comment one".to_string());
        omm.header.comments.push("Header comment two".to_string());
        omm.metadata.comments.push("Metadata comment".to_string());
        omm.mean_elements
            .comments
            .push("Mean elements comment".to_string());

        let xml_str = write_omm(&omm).unwrap();
        let parsed = read_omm(&xml_str).expect("comments round-trip parse failed");

        assert_eq!(
            parsed.header.comments,
            vec![
                "Header comment one".to_string(),
                "Header comment two".to_string()
            ]
        );
        assert_eq!(
            parsed.metadata.comments,
            vec!["Metadata comment".to_string()]
        );
        assert_eq!(
            parsed.mean_elements.comments,
            vec!["Mean elements comment".to_string()]
        );
    }

    // -----------------------------------------------------------------------
    // 7. User-defined parameters preserved
    // -----------------------------------------------------------------------

    #[test]
    fn user_defined_parameters_preserved() {
        let mut omm = sample_omm();
        omm.user_defined
            .insert("OPERATOR".to_string(), "GSOC".to_string());
        omm.user_defined
            .insert("CUSTOM_KEY".to_string(), "custom_value".to_string());

        let xml_str = write_omm(&omm).unwrap();
        let parsed = read_omm(&xml_str).expect("user-defined round-trip parse failed");

        assert_eq!(
            parsed.user_defined.get("OPERATOR"),
            Some(&"GSOC".to_string())
        );
        assert_eq!(
            parsed.user_defined.get("CUSTOM_KEY"),
            Some(&"custom_value".to_string())
        );
    }

    // -----------------------------------------------------------------------
    // 8. Spacecraft parameters + covariance round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn spacecraft_and_covariance_round_trip() {
        let mut omm = sample_omm();
        omm.spacecraft = Some(crate::types::common::SpacecraftParameters {
            comments: vec!["Spacecraft comment".to_string()],
            mass: Some(Mass::kilograms(500.0)),
            solar_rad_area: Some(Area::square_meters(5.0)),
            solar_rad_coeff: Some(1.3),
            drag_area: Some(Area::square_meters(4.0)),
            drag_coeff: Some(2.2),
        });
        omm.covariance = Some(Covariance {
            comments: Vec::new(),
            frame: Some(OdmFrame::Custom("RSW".to_string())),
            matrix: Matrix6::identity(),
        });

        let xml_str = write_omm(&omm).unwrap();
        let parsed = read_omm(&xml_str).expect("spacecraft+covariance parse failed");

        let sp = parsed.spacecraft.expect("spacecraft missing");
        assert_approx_eq!(
            sp.mass.map(|v| v.to_kilograms()),
            Some(500.0),
            atol <= 1e-10
        );
        assert_approx_eq!(sp.drag_coeff, Some(2.2), atol <= 1e-12);

        let cov = parsed.covariance.expect("covariance missing");
        assert_eq!(cov.matrix, Matrix6::identity());
        assert!(matches!(&cov.frame, Some(OdmFrame::Custom(s)) if s == "RSW"));
    }
}
