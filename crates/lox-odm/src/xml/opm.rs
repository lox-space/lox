// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! XML ↔ typed [`Opm`] projection.
//!
//! The private `*Xml` mirror structs follow the XSD shape used by the CCSDS
//! official XML schema.  They map 1-to-1 onto the wire-format element/attribute
//! names via serde rename annotations.
//!
//! - [`read_opm`] — parse an XML string → [`Opm`]
//! - [`write_opm`] — serialise [`Opm`] → XML string

use std::collections::BTreeMap;

use lox_core::coords::Cartesian;
use lox_core::elements::{GravitationalParameter, Keplerian};
use lox_core::time::deltas::TimeDelta;
use lox_core::units::{Angle, Area, Distance, Mass, Velocity};
use nalgebra::Matrix6;
use serde::{Deserialize, Serialize};

use crate::types::common::{
    Covariance, OdmCenter, OdmFrame, OdmHeader, OdmTime, SpacecraftParameters,
};
use crate::types::opm::{Maneuver, Opm, OpmKeplerian, OpmMetadata};
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
pub(crate) struct OpmMetadataXml {
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
}

// ---------------------------------------------------------------------------
// State vector
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct StateVectorXml {
    #[serde(rename = "COMMENT", skip_serializing_if = "Vec::is_empty")]
    comments: Vec<String>,
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
// Keplerian elements
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct KeplerianElementsXml {
    #[serde(rename = "COMMENT", skip_serializing_if = "Vec::is_empty")]
    comments: Vec<String>,
    #[serde(rename = "SEMI_MAJOR_AXIS")]
    semi_major_axis: ValueWithUnits,
    #[serde(rename = "ECCENTRICITY")]
    eccentricity: f64,
    #[serde(rename = "INCLINATION")]
    inclination: ValueWithUnits,
    #[serde(rename = "RA_OF_ASC_NODE")]
    ra_of_asc_node: ValueWithUnits,
    #[serde(rename = "ARG_OF_PERICENTER")]
    arg_of_pericenter: ValueWithUnits,
    #[serde(rename = "TRUE_ANOMALY", skip_serializing_if = "Option::is_none")]
    true_anomaly: Option<ValueWithUnits>,
    #[serde(rename = "MEAN_ANOMALY", skip_serializing_if = "Option::is_none")]
    mean_anomaly: Option<ValueWithUnits>,
    #[serde(rename = "GM", skip_serializing_if = "Option::is_none")]
    gm: Option<ValueWithUnits>,
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
// Maneuver parameters
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct ManeuverParametersXml {
    #[serde(rename = "COMMENT", skip_serializing_if = "Vec::is_empty")]
    comments: Vec<String>,
    #[serde(rename = "MAN_EPOCH_IGNITION")]
    man_epoch_ignition: String,
    #[serde(rename = "MAN_DURATION")]
    man_duration: ValueWithUnits,
    #[serde(rename = "MAN_DELTA_MASS")]
    man_delta_mass: ValueWithUnits,
    #[serde(rename = "MAN_REF_FRAME", skip_serializing_if = "Option::is_none")]
    man_ref_frame: Option<String>,
    #[serde(rename = "MAN_DV_1")]
    man_dv_1: ValueWithUnits,
    #[serde(rename = "MAN_DV_2")]
    man_dv_2: ValueWithUnits,
    #[serde(rename = "MAN_DV_3")]
    man_dv_3: ValueWithUnits,
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
pub(crate) struct OpmDataXml {
    #[serde(rename = "COMMENT", skip_serializing_if = "Vec::is_empty")]
    comments: Vec<String>,
    #[serde(rename = "stateVector")]
    state_vector: StateVectorXml,
    #[serde(rename = "keplerianElements", skip_serializing_if = "Option::is_none")]
    keplerian_elements: Option<KeplerianElementsXml>,
    #[serde(
        rename = "spacecraftParameters",
        skip_serializing_if = "Option::is_none"
    )]
    spacecraft_parameters: Option<SpacecraftParametersXml>,
    #[serde(rename = "covarianceMatrix", skip_serializing_if = "Option::is_none")]
    covariance_matrix: Option<CovarianceMatrixXml>,
    #[serde(rename = "maneuverParameters", skip_serializing_if = "Vec::is_empty")]
    maneuver_parameters: Vec<ManeuverParametersXml>,
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
pub(crate) struct OpmSegmentXml {
    #[serde(rename = "metadata")]
    metadata: OpmMetadataXml,
    #[serde(rename = "data")]
    data: OpmDataXml,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct OpmBodyXml {
    #[serde(rename = "segment")]
    segment: OpmSegmentXml,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename = "opm")]
pub(crate) struct OpmXml {
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
    body: OpmBodyXml,
}

// ---------------------------------------------------------------------------
// Typed → Wire (Opm → OpmXml)
// ---------------------------------------------------------------------------

impl From<&Opm> for OpmXml {
    fn from(opm: &Opm) -> Self {
        let header = OdmHeaderXml {
            comments: opm.header.comments.clone(),
            classification: opm.header.classification.clone(),
            creation_date: opm.header.creation_date.iso(),
            originator: opm.header.originator.clone(),
            message_id: opm.header.message_id.clone(),
        };

        let metadata = OpmMetadataXml {
            comments: opm.metadata.comments.clone(),
            object_name: opm.metadata.object_name.clone(),
            object_id: opm.metadata.object_id.clone(),
            center_name: opm.metadata.center.name().into_owned(),
            ref_frame: opm.metadata.frame.name().into_owned(),
            ref_frame_epoch: opm.metadata.frame_epoch.map(|e| e.iso()),
            time_system: opm.epoch.time_system().to_string(),
        };

        let pos = opm.state.position();
        let vel = opm.state.velocity();

        let state_vector = StateVectorXml {
            comments: opm.state_comments.clone(),
            epoch: opm.epoch.iso(),
            x: ValueWithUnits::new(pos.x / 1000.0, "km"),
            y: ValueWithUnits::new(pos.y / 1000.0, "km"),
            z: ValueWithUnits::new(pos.z / 1000.0, "km"),
            x_dot: ValueWithUnits::new(vel.x / 1000.0, "km/s"),
            y_dot: ValueWithUnits::new(vel.y / 1000.0, "km/s"),
            z_dot: ValueWithUnits::new(vel.z / 1000.0, "km/s"),
        };

        let keplerian_elements = opm.keplerian.as_ref().map(|kep| KeplerianElementsXml {
            comments: kep.comments.clone(),
            semi_major_axis: ValueWithUnits::new(
                kep.elements.semi_major_axis().to_kilometers(),
                "km",
            ),
            eccentricity: kep.elements.eccentricity().as_f64(),
            inclination: ValueWithUnits::new(
                kep.elements.inclination().as_f64().to_degrees(),
                "deg",
            ),
            ra_of_asc_node: ValueWithUnits::new(
                kep.elements
                    .longitude_of_ascending_node()
                    .as_f64()
                    .to_degrees(),
                "deg",
            ),
            arg_of_pericenter: ValueWithUnits::new(
                kep.elements.argument_of_periapsis().as_f64().to_degrees(),
                "deg",
            ),
            true_anomaly: Some(ValueWithUnits::new(
                kep.elements.true_anomaly().as_angle().to_degrees(),
                "deg",
            )),
            mean_anomaly: None,
            gm: kep
                .gm
                .map(|g| ValueWithUnits::new(g.as_f64() / 1e9, "km**3/s**2")),
        });

        let spacecraft_parameters = opm.spacecraft.as_ref().map(|sp| SpacecraftParametersXml {
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

        let covariance_matrix = opm.covariance.as_ref().map(|cov| {
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

        let maneuver_parameters: Vec<ManeuverParametersXml> = opm
            .maneuvers
            .iter()
            .map(|man| ManeuverParametersXml {
                comments: man.comments.clone(),
                man_epoch_ignition: man.ignition_epoch.iso(),
                man_duration: ValueWithUnits::new(man.duration.to_seconds().to_f64(), "s"),
                man_delta_mass: ValueWithUnits::new(man.delta_mass.to_kilograms(), "kg"),
                man_ref_frame: man.frame.as_ref().map(|f| f.name().into_owned()),
                man_dv_1: ValueWithUnits::new(man.delta_v[0].to_kilometers_per_second(), "km/s"),
                man_dv_2: ValueWithUnits::new(man.delta_v[1].to_kilometers_per_second(), "km/s"),
                man_dv_3: ValueWithUnits::new(man.delta_v[2].to_kilometers_per_second(), "km/s"),
            })
            .collect();

        let user_defined_parameters = if opm.user_defined.is_empty() {
            None
        } else {
            Some(UserDefinedParametersXml {
                user_defined: opm
                    .user_defined
                    .iter()
                    .map(|(k, v)| UserDefinedParameterXml {
                        parameter: k.clone(),
                        value: v.clone(),
                    })
                    .collect(),
            })
        };

        OpmXml {
            xmlns_xsi: Some("http://www.w3.org/2001/XMLSchema-instance".to_string()),
            schema_location: Some(
                "http://sanaregistry.org/r/ndmxml/ndmxml-1.0-master.xsd".to_string(),
            ),
            id: Some("CCSDS_OPM_VERS".to_string()),
            version: "3.0".to_string(),
            header,
            body: OpmBodyXml {
                segment: OpmSegmentXml {
                    metadata,
                    data: OpmDataXml {
                        comments: Vec::new(),
                        state_vector,
                        keplerian_elements,
                        spacecraft_parameters,
                        covariance_matrix,
                        maneuver_parameters,
                        user_defined_parameters,
                    },
                },
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Wire → Typed (OpmXml → Opm)
// ---------------------------------------------------------------------------

/// Parse an epoch string under the given time_system, returning an XmlError.
fn parse_epoch(time_system: &str, iso: &str) -> Result<OdmTime, XmlError> {
    OdmTime::from_wire(time_system, iso.trim()).map_err(|e| XmlError::InvalidEpoch {
        value: iso.to_string(),
        time_system: time_system.to_string(),
        reason: e.to_string(),
    })
}

impl TryFrom<OpmXml> for Opm {
    type Error = XmlError;

    fn try_from(xml: OpmXml) -> Result<Self, Self::Error> {
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

        let metadata = OpmMetadata {
            comments: meta.comments,
            object_name: meta.object_name.trim().to_string(),
            object_id: meta.object_id.trim().to_string(),
            center: OdmCenter::from_wire(meta.center_name.trim()),
            frame: OdmFrame::from_wire(meta.ref_frame.trim()),
            frame_epoch,
        };

        // ---- state vector ----
        let sv = &data.state_vector;
        let epoch = parse_epoch(time_system, &sv.epoch)?;

        let state = Cartesian::new(
            Distance::kilometers(sv.x.value),
            Distance::kilometers(sv.y.value),
            Distance::kilometers(sv.z.value),
            Velocity::kilometers_per_second(sv.x_dot.value),
            Velocity::kilometers_per_second(sv.y_dot.value),
            Velocity::kilometers_per_second(sv.z_dot.value),
        );
        let state_comments = sv.comments.clone();

        // ---- Keplerian elements (optional) ----
        let keplerian = match data.keplerian_elements {
            None => None,
            Some(ke) => {
                let true_anomaly = ke
                    .true_anomaly
                    .map(|v| Angle::degrees(v.value))
                    .ok_or_else(|| XmlError::MissingRequiredField("TRUE_ANOMALY".to_string()))?;

                let elements = Keplerian::builder()
                    .with_semi_major_axis(
                        Distance::kilometers(ke.semi_major_axis.value),
                        ke.eccentricity,
                    )
                    .with_inclination(Angle::degrees(ke.inclination.value))
                    .with_longitude_of_ascending_node(Angle::degrees(ke.ra_of_asc_node.value))
                    .with_argument_of_periapsis(Angle::degrees(ke.arg_of_pericenter.value))
                    .with_true_anomaly(true_anomaly)
                    .build()
                    .map_err(|e| XmlError::InvalidValue {
                        keyword: "keplerianElements".to_string(),
                        reason: e.to_string(),
                    })?;

                let gm = ke.gm.map(|v| GravitationalParameter::km3_per_s2(v.value));

                Some(OpmKeplerian {
                    comments: ke.comments,
                    elements,
                    gm,
                })
            }
        };

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

        // ---- maneuvers ----
        let mut maneuvers: Vec<Maneuver> = Vec::new();
        for mp in data.maneuver_parameters {
            let ignition_epoch = parse_epoch(time_system, &mp.man_epoch_ignition)?;
            let duration = TimeDelta::from_seconds_f64(mp.man_duration.value);
            let delta_mass = Mass::kilograms(mp.man_delta_mass.value);
            let frame = mp.man_ref_frame.map(|s| OdmFrame::from_wire(s.trim()));
            let dv1 = Velocity::kilometers_per_second(mp.man_dv_1.value);
            let dv2 = Velocity::kilometers_per_second(mp.man_dv_2.value);
            let dv3 = Velocity::kilometers_per_second(mp.man_dv_3.value);
            maneuvers.push(Maneuver {
                comments: mp.comments,
                ignition_epoch,
                duration,
                delta_mass,
                frame,
                delta_v: [dv1, dv2, dv3],
            });
        }

        // ---- user-defined parameters ----
        let mut user_defined: BTreeMap<String, String> = BTreeMap::new();
        if let Some(udp) = data.user_defined_parameters {
            for p in udp.user_defined {
                user_defined.insert(p.parameter, p.value);
            }
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

// ---------------------------------------------------------------------------
// Public free functions
// ---------------------------------------------------------------------------

/// Parse an OPM XML document into a typed [`Opm`].
pub fn read_opm(input: &str) -> Result<Opm, XmlError> {
    let xml: OpmXml = quick_xml::de::from_str(input)?;
    Opm::try_from(xml)
}

/// Serialise a typed [`Opm`] to an XML string.
///
/// Returns [`XmlError::XmlSer`] if `quick-xml` rejects any field — most
/// realistically a non-finite `f64` (NaN/Infinity) in a numeric slot.
pub fn write_opm(opm: &Opm) -> Result<String, XmlError> {
    let xml = OpmXml::from(opm);
    Ok(quick_xml::se::to_string(&xml)?)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use lox_bodies::DynOrigin;
    use lox_core::elements::{GravitationalParameter, Keplerian};
    use lox_core::time::deltas::TimeDelta;
    use lox_core::units::{Angle, Area, Distance, Mass, Velocity};
    use lox_frames::DynFrame;
    use nalgebra::Matrix6;

    use crate::types::common::{Covariance, OdmCenter, OdmFrame, OdmHeader, OdmTime};
    use crate::types::opm::{Maneuver, Opm, OpmKeplerian, OpmMetadata};

    use super::{read_opm, write_opm};

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

    fn keplerian_elements_safe() -> Keplerian {
        Keplerian::builder()
            .with_semi_major_axis(Distance::kilometers(7000.0), 0.001)
            .with_inclination(Angle::degrees(45.0))
            .with_longitude_of_ascending_node(Angle::ZERO)
            .with_argument_of_periapsis(Angle::ZERO)
            .with_true_anomaly(Angle::ZERO)
            .build()
            .expect("valid elements")
    }

    // -----------------------------------------------------------------------
    // 1. Minimal OPM round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn minimal_opm_round_trip() {
        let opm = sample_opm();
        let xml_str = write_opm(&opm).unwrap();
        let parsed = read_opm(&xml_str).expect("round-trip parse failed");
        assert_eq!(opm, parsed, "minimal round-trip mismatch");
    }

    // -----------------------------------------------------------------------
    // 2. Full OPM round-trip (Keplerian + spacecraft + covariance + 2 maneuvers)
    // -----------------------------------------------------------------------

    #[test]
    fn full_opm_round_trip() {
        let mut opm = sample_opm();
        opm.header.comments.push("Full OPM".to_string());
        opm.metadata.comments.push("Metadata comment".to_string());

        opm.keplerian = Some(OpmKeplerian {
            comments: vec!["Keplerian block".to_string()],
            elements: keplerian_elements_safe(),
            gm: Some(GravitationalParameter::km3_per_s2(398600.4415)),
        });

        opm.spacecraft = Some(crate::types::common::SpacecraftParameters {
            comments: Vec::new(),
            mass: Some(Mass::kilograms(1500.0)),
            solar_rad_area: Some(Area::square_meters(10.0)),
            solar_rad_coeff: Some(1.2),
            drag_area: Some(Area::square_meters(8.0)),
            drag_coeff: Some(2.2),
        });

        opm.covariance = Some(Covariance {
            comments: Vec::new(),
            frame: None,
            matrix: Matrix6::identity(),
        });

        opm.maneuvers.push(Maneuver {
            comments: vec!["First burn".to_string()],
            ignition_epoch: sample_epoch(),
            duration: TimeDelta::from_seconds(60),
            delta_mass: Mass::kilograms(-1.0),
            frame: Some(OdmFrame::Custom("RSW".to_string())),
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

        let xml_str = write_opm(&opm).unwrap();
        let parsed = read_opm(&xml_str).expect("full round-trip parse failed");
        assert_eq!(opm, parsed, "full round-trip mismatch");
    }

    // -----------------------------------------------------------------------
    // 3. GM round-trip preserved in OpmKeplerian
    // -----------------------------------------------------------------------

    #[test]
    fn gm_round_trip_preserved() {
        let wire_gm = GravitationalParameter::km3_per_s2(398600.4415);
        let mut opm = sample_opm();
        opm.keplerian = Some(OpmKeplerian {
            comments: Vec::new(),
            elements: keplerian_elements_safe(),
            gm: Some(wire_gm),
        });

        let xml_str = write_opm(&opm).unwrap();
        let parsed = read_opm(&xml_str).expect("GM round-trip parse failed");
        let parsed_gm = parsed.keplerian.unwrap().gm.unwrap();
        let diff = (parsed_gm.as_f64() - wire_gm.as_f64()).abs();
        assert!(
            diff < 1.0,
            "GM round-trip error too large: {diff} m³/s²; parsed={parsed_gm}, original={wire_gm}"
        );
    }

    // -----------------------------------------------------------------------
    // 4. Comments preserved in header and metadata
    // -----------------------------------------------------------------------

    #[test]
    fn comments_preserved_in_header_and_metadata() {
        let mut opm = sample_opm();
        opm.header.comments.push("Header comment one".to_string());
        opm.header.comments.push("Header comment two".to_string());
        opm.metadata.comments.push("Metadata comment".to_string());

        let xml_str = write_opm(&opm).unwrap();
        let parsed = read_opm(&xml_str).expect("comments round-trip parse failed");
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
    }

    // -----------------------------------------------------------------------
    // 5. read_opm on the legacy fixture
    // -----------------------------------------------------------------------

    const LEGACY_FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<opm  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="http://sanaregistry.org/r/ndmxml/ndmxml-1.0-master.xsd"
        id="CCSDS_OPM_VERS" version="3.0">

    <header>
    <COMMENT>THIS IS AN XML VERSION OF THE OPM</COMMENT>
    <CREATION_DATE>2001-11-06T09:23:57</CREATION_DATE>
    <ORIGINATOR>JAXA</ORIGINATOR>
    <MESSAGE_ID>OPM 201113719185</MESSAGE_ID>
    </header>
    <body>
    <segment>
        <metadata>
            <COMMENT>GEOCENTRIC, CARTESIAN, EARTH FIXED</COMMENT>
            <OBJECT_NAME>OSPREY 5</OBJECT_NAME>
            <OBJECT_ID>1998-999A</OBJECT_ID>
            <CENTER_NAME>EARTH</CENTER_NAME>
            <REF_FRAME>TOD</REF_FRAME>
            <REF_FRAME_EPOCH>1998-12-18T14:28:15.1172</REF_FRAME_EPOCH>
            <TIME_SYSTEM>UTC</TIME_SYSTEM>
        </metadata>
        <data>
            <stateVector>
                <EPOCH>2008-09-20T12:25:40.104192</EPOCH>
                <X units="km">4086.147180</X>
                <Y units="km">-994.936814</Y>
                <Z units="km">5250.678791</Z>
                <X_DOT units="km/s">2.511071</X_DOT>
                <Y_DOT units="km/s">7.255240</Y_DOT>
                <Z_DOT units="km/s">-0.583165</Z_DOT>
            </stateVector>
            <keplerianElements>
                <SEMI_MAJOR_AXIS units="km">6730.96</SEMI_MAJOR_AXIS>
                <ECCENTRICITY>0.0006703</ECCENTRICITY>
                <INCLINATION units="deg">51.6416</INCLINATION>
                <RA_OF_ASC_NODE units="deg">247.463</RA_OF_ASC_NODE>
                <ARG_OF_PERICENTER units="deg">130.536</ARG_OF_PERICENTER>
                <TRUE_ANOMALY units="deg">324.985</TRUE_ANOMALY>
                <GM units="km**3/s**2">398600.9368</GM>
            </keplerianElements>
            <spacecraftParameters>
                <MASS>3000.000000</MASS>
                <SOLAR_RAD_AREA>18.770000</SOLAR_RAD_AREA>
                <SOLAR_RAD_COEFF>1.000000</SOLAR_RAD_COEFF>
                <DRAG_AREA>18.770000</DRAG_AREA>
                <DRAG_COEFF>2.500000</DRAG_COEFF>
            </spacecraftParameters>
            <covarianceMatrix>
                <COV_REF_FRAME>ITRF1997</COV_REF_FRAME>
                <CX_X>0.316</CX_X>
                <CY_X>0.722</CY_X>
                <CY_Y>0.518</CY_Y>
                <CZ_X>0.202</CZ_X>
                <CZ_Y>0.715</CZ_Y>
                <CZ_Z>0.002</CZ_Z>
                <CX_DOT_X>0.912</CX_DOT_X>
                <CX_DOT_Y>0.306</CX_DOT_Y>
                <CX_DOT_Z>0.276</CX_DOT_Z>
                <CX_DOT_X_DOT>0.797</CX_DOT_X_DOT>
                <CY_DOT_X>0.562</CY_DOT_X>
                <CY_DOT_Y>0.899</CY_DOT_Y>
                <CY_DOT_Z>0.022</CY_DOT_Z>
                <CY_DOT_X_DOT>0.079</CY_DOT_X_DOT>
                <CY_DOT_Y_DOT>0.415</CY_DOT_Y_DOT>
                <CZ_DOT_X>0.245</CZ_DOT_X>
                <CZ_DOT_Y>0.965</CZ_DOT_Y>
                <CZ_DOT_Z>0.950</CZ_DOT_Z>
                <CZ_DOT_X_DOT>0.435</CZ_DOT_X_DOT>
                <CZ_DOT_Y_DOT>0.621</CZ_DOT_Y_DOT>
                <CZ_DOT_Z_DOT>0.991</CZ_DOT_Z_DOT>
            </covarianceMatrix>
            <maneuverParameters>
                <COMMENT>Maneuver 1</COMMENT>
                <MAN_EPOCH_IGNITION>2008-09-20T12:41:09.984493</MAN_EPOCH_IGNITION>
                <MAN_DURATION units="s">180.000</MAN_DURATION>
                <MAN_DELTA_MASS units="kg">-0.001</MAN_DELTA_MASS>
                <MAN_REF_FRAME>RSW</MAN_REF_FRAME>
                <MAN_DV_1 units="km/s">0.000000</MAN_DV_1>
                <MAN_DV_2 units="km/s">0.280000</MAN_DV_2>
                <MAN_DV_3 units="km/s">0.000000</MAN_DV_3>
            </maneuverParameters>
            <maneuverParameters>
                <MAN_EPOCH_IGNITION>2008-09-20T13:33:11.374985</MAN_EPOCH_IGNITION>
                <MAN_DURATION units="s">180.000</MAN_DURATION>
                <MAN_DELTA_MASS units="kg">-0.001</MAN_DELTA_MASS>
                <MAN_REF_FRAME>RSW</MAN_REF_FRAME>
                <MAN_DV_1 units="km/s">0.000000</MAN_DV_1>
                <MAN_DV_2 units="km/s">0.270000</MAN_DV_2>
                <MAN_DV_3 units="km/s">0.000000</MAN_DV_3>
            </maneuverParameters>
        </data>
    </segment>
    </body>
</opm>"#;

    #[test]
    fn read_opm_legacy_fixture_succeeds() {
        let opm = read_opm(LEGACY_FIXTURE).expect("failed to parse legacy fixture");
        assert_eq!(opm.metadata.object_name, "OSPREY 5");
        assert_eq!(opm.metadata.object_id, "1998-999A");
        assert_eq!(opm.maneuvers.len(), 2);
        assert!(opm.keplerian.is_some());
        assert!(opm.spacecraft.is_some());
        assert!(opm.covariance.is_some());
        // First maneuver comment preserved
        assert_eq!(opm.maneuvers[0].comments, vec!["Maneuver 1".to_string()]);
        // GM round-trip in km³/s²: 398600.9368 km³/s² = 3.986009368e14 m³/s²
        let gm_m3s2 = opm.keplerian.unwrap().gm.unwrap().as_f64();
        let diff = (gm_m3s2 - 398600.9368e9).abs();
        assert!(diff < 1.0, "GM value unexpected: {gm_m3s2}");
    }
}
