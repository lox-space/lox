// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! JSON ↔ typed [`Omm`] projection.
//!
//! Implements the Space-Track / Celestrak flat-object JSON wire format for OMM.
//! Every CCSDS keyword appears as a top-level key (`OBJECT_NAME`, `MEAN_MOTION`,
//! `BSTAR`, etc.).  Numbers on the wire may be strings or native numbers; we
//! accept both on read and emit native numbers on write.
//!
//! ## Comment handling
//!
//! The wire carries a single `COMMENT` string.  On write, `header.comments` is
//! joined with `\n`.  On read, the value is split on `\n` and stored in
//! `header.comments`.  Sub-block comments (`metadata.comments`,
//! `mean_elements.comments`, etc.) are **not** preserved through JSON — they
//! are empty on read and dropped on write.
//!
//! ## Non-CCSDS extras
//!
//! Space-Track / Celestrak sends additional non-CCSDS fields
//! (`TLE_LINE0`, `TLE_LINE1`, `TLE_LINE2`, `OBJECT_TYPE`, `RCS_SIZE`,
//! `COUNTRY_CODE`, `LAUNCH_DATE`, `SITE`, `DECAY_DATE`, `FILE`, `GP_ID`,
//! `PERIOD`, `APOAPSIS`, `PERIAPSIS`, `SEMIMAJOR_AXIS`).  These are
//! captured via `#[serde(flatten)]` into [`Omm::provider_extras`] and
//! re-emitted verbatim on write, so a JSON OMM round-trip preserves
//! everything the operator sent.

use std::collections::BTreeMap;
use std::f64::consts::PI;

use lox_bodies::TryPointMass;
use lox_core::elements::{GravitationalParameter, MeanElements};
use lox_core::units::{Area, AreaToMass, Mass};
use nalgebra::Matrix6;
use serde::de::{self, Unexpected};
use serde::{Deserialize, Deserializer, Serialize};

use crate::json::error::JsonError;
use crate::types::common::{
    Covariance, OdmCenter, OdmFrame, OdmHeader, OdmTime, SpacecraftParameters,
};
use crate::types::omm::{Omm, OmmMeanElements, OmmMetadata, TleParameters};

// ---------------------------------------------------------------------------
// Lax number-from-string deserializers
// ---------------------------------------------------------------------------

/// Deserialise a required `f64` that may arrive as either a JSON string
/// (`"15.27989249"`) or a native JSON number (`15.27989249`).
fn de_f64_lax<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    struct F64Visitor;

    impl<'de> de::Visitor<'de> for F64Visitor {
        type Value = f64;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a float or a string containing a float")
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<f64, E> {
            Ok(v)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<f64, E> {
            Ok(v as f64)
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<f64, E> {
            Ok(v as f64)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<f64, E> {
            v.trim()
                .parse::<f64>()
                .map_err(|_| de::Error::invalid_value(Unexpected::Str(v), &"a float-valued string"))
        }
    }

    deserializer.deserialize_any(F64Visitor)
}

/// Deserialise an optional `f64` that may arrive as a JSON string, number, or
/// `null` / absent key.
fn de_f64_lax_opt<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptF64Visitor;

    impl<'de> de::Visitor<'de> for OptF64Visitor {
        type Value = Option<f64>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a float, a string containing a float, or null")
        }

        fn visit_unit<E: de::Error>(self) -> Result<Option<f64>, E> {
            Ok(None)
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<Option<f64>, E> {
            Ok(Some(v))
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Option<f64>, E> {
            Ok(Some(v as f64))
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Option<f64>, E> {
            Ok(Some(v as f64))
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Option<f64>, E> {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            trimmed
                .parse::<f64>()
                .map(Some)
                .map_err(|_| de::Error::invalid_value(Unexpected::Str(v), &"a float-valued string"))
        }
    }

    deserializer.deserialize_any(OptF64Visitor)
}

/// Deserialise an optional `i32` from a JSON string, number, or null.
fn de_i32_lax_opt<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptI32Visitor;

    impl<'de> de::Visitor<'de> for OptI32Visitor {
        type Value = Option<i32>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("an integer, a string containing an integer, or null")
        }

        fn visit_unit<E: de::Error>(self) -> Result<Option<i32>, E> {
            Ok(None)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Option<i32>, E> {
            i32::try_from(v).map(Some).map_err(de::Error::custom)
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Option<i32>, E> {
            i32::try_from(v).map(Some).map_err(de::Error::custom)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Option<i32>, E> {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            trimmed
                .parse::<i32>()
                .map(Some)
                .map_err(|_| de::Error::invalid_value(Unexpected::Str(v), &"an i32 string"))
        }
    }

    deserializer.deserialize_any(OptI32Visitor)
}

/// Deserialise an optional `i64` from a JSON string, number, or null.
fn de_i64_lax_opt<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptI64Visitor;

    impl<'de> de::Visitor<'de> for OptI64Visitor {
        type Value = Option<i64>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("an integer, a string containing an integer, or null")
        }

        fn visit_unit<E: de::Error>(self) -> Result<Option<i64>, E> {
            Ok(None)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Option<i64>, E> {
            Ok(Some(v))
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Option<i64>, E> {
            i64::try_from(v).map(Some).map_err(de::Error::custom)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Option<i64>, E> {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            trimmed
                .parse::<i64>()
                .map(Some)
                .map_err(|_| de::Error::invalid_value(Unexpected::Str(v), &"an i64 string"))
        }
    }

    deserializer.deserialize_any(OptI64Visitor)
}

/// Deserialise an optional `u64` from a JSON string, number, or null.
fn de_u64_lax_opt<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptU64Visitor;

    impl<'de> de::Visitor<'de> for OptU64Visitor {
        type Value = Option<u64>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a non-negative integer, a string containing one, or null")
        }

        fn visit_unit<E: de::Error>(self) -> Result<Option<u64>, E> {
            Ok(None)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Option<u64>, E> {
            u64::try_from(v).map(Some).map_err(de::Error::custom)
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Option<u64>, E> {
            Ok(Some(v))
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Option<u64>, E> {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            trimmed
                .parse::<u64>()
                .map(Some)
                .map_err(|_| de::Error::invalid_value(Unexpected::Str(v), &"a u64 string"))
        }
    }

    deserializer.deserialize_any(OptU64Visitor)
}

// ---------------------------------------------------------------------------
// Private wire-format struct
// ---------------------------------------------------------------------------

/// Private flat JSON representation of a CCSDS OMM.
///
/// All fields use CCSDS wire-spelling as the JSON key.  Required string fields
/// default to an empty string; numeric fields default to `None`.  The
/// `extras` catch-all preserves Space-Track non-CCSDS fields across
/// read→write via `#[serde(flatten)]`.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct OmmJson {
    // --- Header ---
    #[serde(rename = "CCSDS_OMM_VERS")]
    vers: String,
    #[serde(rename = "CREATION_DATE")]
    creation_date: String,
    #[serde(rename = "ORIGINATOR")]
    originator: String,
    #[serde(rename = "MESSAGE_ID", skip_serializing_if = "Option::is_none")]
    message_id: Option<String>,
    #[serde(rename = "CLASSIFICATION", skip_serializing_if = "Option::is_none")]
    classification: Option<String>,
    /// Single comment string.  On write: `header.comments` joined by `\n`.
    /// On read: split by `\n` → `header.comments`.
    #[serde(rename = "COMMENT", skip_serializing_if = "Option::is_none")]
    comment: Option<String>,

    // --- Metadata ---
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

    // --- Mean elements ---
    #[serde(rename = "EPOCH")]
    epoch: String,
    #[serde(
        rename = "SEMI_MAJOR_AXIS",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    semi_major_axis: Option<f64>,
    #[serde(
        rename = "MEAN_MOTION",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    mean_motion: Option<f64>,
    #[serde(rename = "ECCENTRICITY", deserialize_with = "de_f64_lax")]
    eccentricity: f64,
    #[serde(rename = "INCLINATION", deserialize_with = "de_f64_lax")]
    inclination: f64,
    #[serde(rename = "RA_OF_ASC_NODE", deserialize_with = "de_f64_lax")]
    ra_of_asc_node: f64,
    #[serde(rename = "ARG_OF_PERICENTER", deserialize_with = "de_f64_lax")]
    arg_of_pericenter: f64,
    #[serde(rename = "MEAN_ANOMALY", deserialize_with = "de_f64_lax")]
    mean_anomaly: f64,
    #[serde(
        rename = "GM",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    gm: Option<f64>,

    // --- TLE-related parameters (all optional) ---
    #[serde(
        rename = "EPHEMERIS_TYPE",
        deserialize_with = "de_i32_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    ephemeris_type: Option<i32>,
    #[serde(
        rename = "CLASSIFICATION_TYPE",
        skip_serializing_if = "Option::is_none"
    )]
    classification_type: Option<String>,
    #[serde(
        rename = "NORAD_CAT_ID",
        deserialize_with = "de_i32_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    norad_cat_id: Option<i32>,
    #[serde(
        rename = "ELEMENT_SET_NO",
        deserialize_with = "de_i64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    element_set_no: Option<i64>,
    #[serde(
        rename = "REV_AT_EPOCH",
        deserialize_with = "de_u64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    rev_at_epoch: Option<u64>,
    #[serde(
        rename = "BSTAR",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    bstar: Option<f64>,
    #[serde(
        rename = "BTERM",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    bterm: Option<f64>,
    #[serde(
        rename = "MEAN_MOTION_DOT",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    mean_motion_dot: Option<f64>,
    #[serde(
        rename = "MEAN_MOTION_DDOT",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    mean_motion_ddot: Option<f64>,
    #[serde(
        rename = "AGOM",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    agom: Option<f64>,

    // --- Spacecraft parameters (all optional) ---
    #[serde(
        rename = "MASS",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    mass: Option<f64>,
    #[serde(
        rename = "SOLAR_RAD_AREA",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    solar_rad_area: Option<f64>,
    #[serde(
        rename = "SOLAR_RAD_COEFF",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    solar_rad_coeff: Option<f64>,
    #[serde(
        rename = "DRAG_AREA",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    drag_area: Option<f64>,
    #[serde(
        rename = "DRAG_COEFF",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    drag_coeff: Option<f64>,

    // --- Covariance (21 lower-triangle fields, all optional) ---
    #[serde(rename = "COV_REF_FRAME", skip_serializing_if = "Option::is_none")]
    cov_ref_frame: Option<String>,
    #[serde(
        rename = "CX_X",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cx_x: Option<f64>,
    #[serde(
        rename = "CY_X",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cy_x: Option<f64>,
    #[serde(
        rename = "CY_Y",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cy_y: Option<f64>,
    #[serde(
        rename = "CZ_X",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cz_x: Option<f64>,
    #[serde(
        rename = "CZ_Y",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cz_y: Option<f64>,
    #[serde(
        rename = "CZ_Z",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cz_z: Option<f64>,
    #[serde(
        rename = "CX_DOT_X",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cx_dot_x: Option<f64>,
    #[serde(
        rename = "CX_DOT_Y",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cx_dot_y: Option<f64>,
    #[serde(
        rename = "CX_DOT_Z",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cx_dot_z: Option<f64>,
    #[serde(
        rename = "CX_DOT_X_DOT",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cx_dot_x_dot: Option<f64>,
    #[serde(
        rename = "CY_DOT_X",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cy_dot_x: Option<f64>,
    #[serde(
        rename = "CY_DOT_Y",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cy_dot_y: Option<f64>,
    #[serde(
        rename = "CY_DOT_Z",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cy_dot_z: Option<f64>,
    #[serde(
        rename = "CY_DOT_X_DOT",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cy_dot_x_dot: Option<f64>,
    #[serde(
        rename = "CY_DOT_Y_DOT",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cy_dot_y_dot: Option<f64>,
    #[serde(
        rename = "CZ_DOT_X",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cz_dot_x: Option<f64>,
    #[serde(
        rename = "CZ_DOT_Y",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cz_dot_y: Option<f64>,
    #[serde(
        rename = "CZ_DOT_Z",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cz_dot_z: Option<f64>,
    #[serde(
        rename = "CZ_DOT_X_DOT",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cz_dot_x_dot: Option<f64>,
    #[serde(
        rename = "CZ_DOT_Y_DOT",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cz_dot_y_dot: Option<f64>,
    #[serde(
        rename = "CZ_DOT_Z_DOT",
        deserialize_with = "de_f64_lax_opt",
        skip_serializing_if = "Option::is_none"
    )]
    cz_dot_z_dot: Option<f64>,

    /// Catches all Space-Track / Celestrak non-CCSDS extras and rolls
    /// them through `Omm::provider_extras` so they survive read→write.
    #[serde(flatten)]
    extras: BTreeMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Helper: require a non-empty string field
// ---------------------------------------------------------------------------

fn require_str(value: &str, keyword: &str) -> Result<String, JsonError> {
    if value.is_empty() {
        Err(JsonError::MissingRequiredField(keyword.to_string()))
    } else {
        Ok(value.to_string())
    }
}

// ---------------------------------------------------------------------------
// Typed → OmmJson (write direction)
// ---------------------------------------------------------------------------

impl From<&Omm> for OmmJson {
    fn from(omm: &Omm) -> Self {
        // NOTE: sub-block comments (metadata, mean_elements, TLE, spacecraft,
        // covariance) are not preserved through the JSON format — the wire
        // carries only a single top-level COMMENT.
        let comment = if omm.header.comments.is_empty() {
            None
        } else {
            Some(omm.header.comments.join("\n"))
        };

        // SMA: meters → km for wire
        let semi_major_axis = Some(omm.mean_elements.elements.a / 1000.0);

        // Optional wire GM: m³/s² → km³/s²
        let gm = omm.mean_elements.gm.map(|g| g.as_f64() / 1e9);

        // TLE fields
        let (
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
        ) = match &omm.tle_parameters {
            Some(t) => (
                t.ephemeris_type,
                t.classification_type.clone(),
                t.norad_cat_id,
                t.element_set_no,
                t.rev_at_epoch,
                t.bstar,
                t.bterm.map(|v| v.to_square_meters_per_kilogram()),
                t.mean_motion_dot,
                t.mean_motion_ddot,
                t.agom.map(|v| v.to_square_meters_per_kilogram()),
            ),
            None => (None, None, None, None, None, None, None, None, None, None),
        };

        // Spacecraft fields
        let (mass, solar_rad_area, solar_rad_coeff, drag_area, drag_coeff) = match &omm.spacecraft {
            Some(sp) => (
                sp.mass.map(|m| m.to_kilograms()),
                sp.solar_rad_area.map(|a| a.to_square_meters()),
                sp.solar_rad_coeff,
                sp.drag_area.map(|a| a.to_square_meters()),
                sp.drag_coeff,
            ),
            None => (None, None, None, None, None),
        };

        // Covariance fields
        let (
            cov_ref_frame,
            cx_x,
            cy_x,
            cy_y,
            cz_x,
            cz_y,
            cz_z,
            cx_dot_x,
            cx_dot_y,
            cx_dot_z,
            cx_dot_x_dot,
            cy_dot_x,
            cy_dot_y,
            cy_dot_z,
            cy_dot_x_dot,
            cy_dot_y_dot,
            cz_dot_x,
            cz_dot_y,
            cz_dot_z,
            cz_dot_x_dot,
            cz_dot_y_dot,
            cz_dot_z_dot,
        ) = match &omm.covariance {
            Some(cov) => {
                let m = &cov.matrix;
                (
                    cov.frame.as_ref().map(|f| f.name().into_owned()),
                    Some(m[(0, 0)]),
                    Some(m[(1, 0)]),
                    Some(m[(1, 1)]),
                    Some(m[(2, 0)]),
                    Some(m[(2, 1)]),
                    Some(m[(2, 2)]),
                    Some(m[(3, 0)]),
                    Some(m[(3, 1)]),
                    Some(m[(3, 2)]),
                    Some(m[(3, 3)]),
                    Some(m[(4, 0)]),
                    Some(m[(4, 1)]),
                    Some(m[(4, 2)]),
                    Some(m[(4, 3)]),
                    Some(m[(4, 4)]),
                    Some(m[(5, 0)]),
                    Some(m[(5, 1)]),
                    Some(m[(5, 2)]),
                    Some(m[(5, 3)]),
                    Some(m[(5, 4)]),
                    Some(m[(5, 5)]),
                )
            }
            None => (
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
            ),
        };

        let ref_frame_epoch = omm.metadata.frame_epoch.map(|e| e.iso());

        OmmJson {
            vers: "2.0".to_string(),
            creation_date: omm.header.creation_date.iso(),
            originator: omm.header.originator.clone(),
            message_id: omm.header.message_id.clone(),
            classification: omm.header.classification.clone(),
            comment,
            object_name: omm.metadata.object_name.clone(),
            object_id: omm.metadata.object_id.clone(),
            center_name: omm.metadata.center.name().into_owned(),
            ref_frame: omm.metadata.frame.name().into_owned(),
            ref_frame_epoch,
            time_system: omm.epoch.time_system().to_string(),
            mean_element_theory: omm.metadata.mean_element_theory.clone(),
            epoch: omm.epoch.iso(),
            semi_major_axis,
            mean_motion: None,
            eccentricity: omm.mean_elements.elements.e,
            inclination: omm.mean_elements.elements.i.to_degrees(),
            ra_of_asc_node: omm.mean_elements.elements.raan.to_degrees(),
            arg_of_pericenter: omm.mean_elements.elements.aop.to_degrees(),
            mean_anomaly: omm.mean_elements.elements.m.to_degrees(),
            gm,
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
            mass,
            solar_rad_area,
            solar_rad_coeff,
            drag_area,
            drag_coeff,
            cov_ref_frame,
            cx_x,
            cy_x,
            cy_y,
            cz_x,
            cz_y,
            cz_z,
            cx_dot_x,
            cx_dot_y,
            cx_dot_z,
            cx_dot_x_dot,
            cy_dot_x,
            cy_dot_y,
            cy_dot_z,
            cy_dot_x_dot,
            cy_dot_y_dot,
            cz_dot_x,
            cz_dot_y,
            cz_dot_z,
            cz_dot_x_dot,
            cz_dot_y_dot,
            cz_dot_z_dot,
            extras: omm.provider_extras.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// OmmJson → Typed (read direction)
// ---------------------------------------------------------------------------

impl TryFrom<OmmJson> for Omm {
    type Error = JsonError;

    fn try_from(j: OmmJson) -> Result<Self, Self::Error> {
        // --- Required string fields ---
        let originator = require_str(&j.originator, "ORIGINATOR")?;
        let creation_date_str = require_str(&j.creation_date, "CREATION_DATE")?;
        let object_name = require_str(&j.object_name, "OBJECT_NAME")?;
        let object_id = require_str(&j.object_id, "OBJECT_ID")?;
        let center_name = require_str(&j.center_name, "CENTER_NAME")?;
        let ref_frame_str = require_str(&j.ref_frame, "REF_FRAME")?;
        let time_system = require_str(&j.time_system, "TIME_SYSTEM")?;
        let mean_element_theory = require_str(&j.mean_element_theory, "MEAN_ELEMENT_THEORY")?;
        let epoch_str = require_str(&j.epoch, "EPOCH")?;

        // --- Parse creation date ---
        let creation_date = OdmTime::from_wire(&time_system, &creation_date_str).map_err(|e| {
            JsonError::InvalidEpoch {
                value: creation_date_str.clone(),
                time_system: time_system.clone(),
                reason: e.to_string(),
            }
        })?;

        // --- Parse epoch ---
        let epoch =
            OdmTime::from_wire(&time_system, &epoch_str).map_err(|e| JsonError::InvalidEpoch {
                value: epoch_str.clone(),
                time_system: time_system.clone(),
                reason: e.to_string(),
            })?;

        // --- Optional REF_FRAME_EPOCH ---
        let frame_epoch = match j.ref_frame_epoch {
            Some(ref s) if !s.is_empty() => Some(OdmTime::from_wire(&time_system, s).map_err(
                |e| JsonError::InvalidEpoch {
                    value: s.clone(),
                    time_system: time_system.clone(),
                    reason: e.to_string(),
                },
            )?),
            _ => None,
        };

        // --- Header ---
        // NOTE: sub-block comments are not preserved in JSON; only header
        // comments survive via the single top-level COMMENT field.
        let header_comments = j
            .comment
            .map(|s| s.split('\n').map(|x| x.to_string()).collect::<Vec<_>>())
            .unwrap_or_default();

        let header = OdmHeader {
            comments: header_comments,
            classification: j.classification,
            creation_date,
            originator,
            message_id: j.message_id,
        };

        // --- Metadata ---
        let center = OdmCenter::from_wire(&center_name);
        let frame = OdmFrame::from_wire(&ref_frame_str);

        let metadata = OmmMetadata {
            comments: Vec::new(),
            object_name,
            object_id,
            center,
            frame,
            frame_epoch,
            mean_element_theory,
        };

        // --- Wire GM (optional) ---
        let wire_gm = j.gm.map(GravitationalParameter::km3_per_s2);

        // --- MEAN_MOTION → SMA helper ---
        // Kepler's third law: a = (μ / n²)^(1/3)
        // n in rad/s = (rev/day) * 2π / 86400
        let resolve_gm = || -> Result<f64, JsonError> {
            if let Some(gm) = wire_gm {
                return Ok(gm.as_f64());
            }
            metadata
                .center
                .known()
                .and_then(|o| o.try_gravitational_parameter().ok())
                .map(|gm| gm.as_f64())
                .ok_or_else(|| JsonError::MissingRequiredField("GM".to_string()))
        };

        // --- Semi-major axis (required; accept SMA or MEAN_MOTION) ---
        let a_m = if let Some(sma_km) = j.semi_major_axis {
            sma_km * 1000.0
        } else if let Some(mm_rev_day) = j.mean_motion {
            let mu = resolve_gm()?;
            let n = mm_rev_day * 2.0 * PI / 86400.0;
            (mu / (n * n)).cbrt()
        } else {
            return Err(JsonError::MissingRequiredField(
                "SEMI_MAJOR_AXIS".to_string(),
            ));
        };

        // --- Remaining mean elements (all required) ---
        let elements = MeanElements {
            a: a_m,
            e: j.eccentricity,
            i: j.inclination.to_radians(),
            raan: j.ra_of_asc_node.to_radians(),
            aop: j.arg_of_pericenter.to_radians(),
            m: j.mean_anomaly.to_radians(),
        };

        let mean_elements = OmmMeanElements {
            comments: Vec::new(),
            elements,
            gm: wire_gm,
        };

        // --- Optional TLE parameters ---
        let has_tle = j.ephemeris_type.is_some()
            || j.classification_type.is_some()
            || j.norad_cat_id.is_some()
            || j.element_set_no.is_some()
            || j.rev_at_epoch.is_some()
            || j.bstar.is_some()
            || j.bterm.is_some()
            || j.mean_motion_dot.is_some()
            || j.mean_motion_ddot.is_some()
            || j.agom.is_some();

        let tle_parameters = if has_tle {
            Some(TleParameters {
                comments: Vec::new(),
                ephemeris_type: j.ephemeris_type,
                classification_type: j.classification_type,
                norad_cat_id: j.norad_cat_id,
                element_set_no: j.element_set_no,
                rev_at_epoch: j.rev_at_epoch,
                bstar: j.bstar,
                bterm: j.bterm.map(AreaToMass::square_meters_per_kilogram),
                mean_motion_dot: j.mean_motion_dot,
                mean_motion_ddot: j.mean_motion_ddot,
                agom: j.agom.map(AreaToMass::square_meters_per_kilogram),
            })
        } else {
            None
        };

        // --- Optional spacecraft parameters ---
        let has_spacecraft = j.mass.is_some()
            || j.solar_rad_area.is_some()
            || j.solar_rad_coeff.is_some()
            || j.drag_area.is_some()
            || j.drag_coeff.is_some();

        let spacecraft = if has_spacecraft {
            Some(SpacecraftParameters {
                comments: Vec::new(),
                mass: j.mass.map(Mass::kilograms),
                solar_rad_area: j.solar_rad_area.map(Area::square_meters),
                solar_rad_coeff: j.solar_rad_coeff,
                drag_area: j.drag_area.map(Area::square_meters),
                drag_coeff: j.drag_coeff,
            })
        } else {
            None
        };

        // --- Optional covariance ---
        // All 21 lower-triangle fields must be present for us to parse a
        // covariance block; if none are present there is no covariance.
        let covariance = if j.cx_x.is_some() {
            let get = |v: Option<f64>, name: &str| -> Result<f64, JsonError> {
                v.ok_or_else(|| JsonError::MissingRequiredField(name.to_string()))
            };
            let cx_x = get(j.cx_x, "CX_X")?;
            let cy_x = get(j.cy_x, "CY_X")?;
            let cy_y = get(j.cy_y, "CY_Y")?;
            let cz_x = get(j.cz_x, "CZ_X")?;
            let cz_y = get(j.cz_y, "CZ_Y")?;
            let cz_z = get(j.cz_z, "CZ_Z")?;
            let cx_dot_x = get(j.cx_dot_x, "CX_DOT_X")?;
            let cx_dot_y = get(j.cx_dot_y, "CX_DOT_Y")?;
            let cx_dot_z = get(j.cx_dot_z, "CX_DOT_Z")?;
            let cx_dot_x_dot = get(j.cx_dot_x_dot, "CX_DOT_X_DOT")?;
            let cy_dot_x = get(j.cy_dot_x, "CY_DOT_X")?;
            let cy_dot_y = get(j.cy_dot_y, "CY_DOT_Y")?;
            let cy_dot_z = get(j.cy_dot_z, "CY_DOT_Z")?;
            let cy_dot_x_dot = get(j.cy_dot_x_dot, "CY_DOT_X_DOT")?;
            let cy_dot_y_dot = get(j.cy_dot_y_dot, "CY_DOT_Y_DOT")?;
            let cz_dot_x = get(j.cz_dot_x, "CZ_DOT_X")?;
            let cz_dot_y = get(j.cz_dot_y, "CZ_DOT_Y")?;
            let cz_dot_z = get(j.cz_dot_z, "CZ_DOT_Z")?;
            let cz_dot_x_dot = get(j.cz_dot_x_dot, "CZ_DOT_X_DOT")?;
            let cz_dot_y_dot = get(j.cz_dot_y_dot, "CZ_DOT_Y_DOT")?;
            let cz_dot_z_dot = get(j.cz_dot_z_dot, "CZ_DOT_Z_DOT")?;

            let mut matrix = Matrix6::<f64>::zeros();
            // Lower triangle
            matrix[(0, 0)] = cx_x;
            matrix[(1, 0)] = cy_x;
            matrix[(1, 1)] = cy_y;
            matrix[(2, 0)] = cz_x;
            matrix[(2, 1)] = cz_y;
            matrix[(2, 2)] = cz_z;
            matrix[(3, 0)] = cx_dot_x;
            matrix[(3, 1)] = cx_dot_y;
            matrix[(3, 2)] = cx_dot_z;
            matrix[(3, 3)] = cx_dot_x_dot;
            matrix[(4, 0)] = cy_dot_x;
            matrix[(4, 1)] = cy_dot_y;
            matrix[(4, 2)] = cy_dot_z;
            matrix[(4, 3)] = cy_dot_x_dot;
            matrix[(4, 4)] = cy_dot_y_dot;
            matrix[(5, 0)] = cz_dot_x;
            matrix[(5, 1)] = cz_dot_y;
            matrix[(5, 2)] = cz_dot_z;
            matrix[(5, 3)] = cz_dot_x_dot;
            matrix[(5, 4)] = cz_dot_y_dot;
            matrix[(5, 5)] = cz_dot_z_dot;
            // Mirror to upper triangle
            for row in 0..6 {
                for col in (row + 1)..6 {
                    matrix[(row, col)] = matrix[(col, row)];
                }
            }

            let frame = j.cov_ref_frame.as_deref().map(OdmFrame::from_wire);

            Some(Covariance {
                comments: Vec::new(),
                frame,
                matrix,
            })
        } else {
            None
        };

        Ok(Omm {
            header,
            metadata,
            epoch,
            mean_elements,
            tle_parameters,
            spacecraft,
            covariance,
            user_defined: BTreeMap::new(),
            provider_extras: j.extras,
        })
    }
}

// ---------------------------------------------------------------------------
// Public free functions
// ---------------------------------------------------------------------------

/// Parse a single OMM from JSON (Space-Track / Celestrak shape).
pub fn read_omm(input: &str) -> Result<Omm, JsonError> {
    let json: OmmJson = serde_json::from_str(input)?;
    Omm::try_from(json)
}

/// Parse a JSON array of OMMs.
pub fn read_omm_list(input: &str) -> Result<Vec<Omm>, JsonError> {
    let jsons: Vec<OmmJson> = serde_json::from_str(input)?;
    jsons.into_iter().map(Omm::try_from).collect()
}

/// Serialise a single OMM to pretty-printed JSON.
///
/// Returns [`JsonError::Json`] if `serde_json` rejects any value — most
/// realistically a non-finite `f64` (NaN/Infinity) in a numeric slot.
pub fn write_omm(omm: &Omm) -> Result<String, JsonError> {
    let json: OmmJson = omm.into();
    Ok(serde_json::to_string_pretty(&json)?)
}

/// Serialise an array of OMMs to pretty-printed JSON.
///
/// See [`write_omm`] for the failure mode.
pub fn write_omm_list(omms: &[Omm]) -> Result<String, JsonError> {
    let jsons: Vec<OmmJson> = omms.iter().map(OmmJson::from).collect();
    Ok(serde_json::to_string_pretty(&jsons)?)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::f64::consts::PI;

    use lox_bodies::DynOrigin;
    use lox_core::elements::{GravitationalParameter, MeanElements};
    use lox_core::units::{Area, AreaToMass, Mass};
    use lox_frames::DynFrame;
    use nalgebra::Matrix6;

    use crate::json::error::JsonError;
    use crate::types::common::{Covariance, OdmCenter, OdmFrame, OdmHeader, OdmTime};
    use crate::types::omm::{Omm, OmmMeanElements, OmmMetadata, TleParameters};

    use super::*;

    // -----------------------------------------------------------------------
    // Fixtures
    // -----------------------------------------------------------------------

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
                mean_element_theory: "SGP4".to_string(),
            },
            epoch,
            mean_elements: OmmMeanElements {
                comments: Vec::new(),
                elements: MeanElements {
                    a: 6_859_961.0, // m
                    e: 0.001_335_6,
                    i: 1.697_775,    // rad
                    raan: 1.159_523, // rad
                    aop: 1.931_018,  // rad
                    m: 5.842_034,    // rad
                },
                gm: None,
            },
            tle_parameters: None,
            spacecraft: None,
            covariance: None,
            user_defined: BTreeMap::new(),
            provider_extras: BTreeMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // 1. Round-trip minimal
    // -----------------------------------------------------------------------

    #[test]
    fn round_trip_minimal() {
        let omm = sample_omm();
        let json = write_omm(&omm).unwrap();
        let parsed = read_omm(&json).expect("round-trip parse failed");

        assert_eq!(parsed.header.originator, omm.header.originator);
        assert_eq!(parsed.metadata.object_name, omm.metadata.object_name);
        assert_eq!(parsed.metadata.object_id, omm.metadata.object_id);
        assert_eq!(parsed.metadata.center, omm.metadata.center);
        assert_eq!(parsed.metadata.frame, omm.metadata.frame);
        assert_eq!(
            parsed.metadata.mean_element_theory,
            omm.metadata.mean_element_theory
        );
        let eps = 1e-9_f64;
        assert!(
            (parsed.mean_elements.elements.a - omm.mean_elements.elements.a).abs() < 1.0,
            "SMA round-trip error"
        );
        assert!((parsed.mean_elements.elements.e - omm.mean_elements.elements.e).abs() < eps);
        assert!((parsed.mean_elements.elements.i - omm.mean_elements.elements.i).abs() < eps);
    }

    // -----------------------------------------------------------------------
    // 2. Round-trip with TLE parameters
    // -----------------------------------------------------------------------

    #[test]
    fn round_trip_with_tle_parameters() {
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

        let json = write_omm(&omm).unwrap();
        let parsed = read_omm(&json).expect("round-trip parse failed");

        let tle = parsed.tle_parameters.expect("missing TLE parameters");
        assert_eq!(tle.norad_cat_id, Some(45018));
        assert!((tle.bstar.unwrap() - 8.4553e-5).abs() < 1e-10);
        assert_eq!(tle.classification_type.as_deref(), Some("U"));
    }

    // -----------------------------------------------------------------------
    // 3. Read with string numbers
    // -----------------------------------------------------------------------

    #[test]
    fn read_with_string_numbers() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "MEAN_MOTION": "15.5",
            "ECCENTRICITY": "0.001",
            "INCLINATION": "45.0",
            "RA_OF_ASC_NODE": "0.0",
            "ARG_OF_PERICENTER": "0.0",
            "MEAN_ANOMALY": "0.0"
        }"#;
        let omm = read_omm(json).expect("parse failed");
        assert!(omm.mean_elements.elements.a > 6_000_000.0);
    }

    // -----------------------------------------------------------------------
    // 4. Read with native numbers
    // -----------------------------------------------------------------------

    #[test]
    fn read_with_native_numbers() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "MEAN_MOTION": 15.5,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0
        }"#;
        let omm = read_omm(json).expect("parse failed with native numbers");
        assert!(omm.mean_elements.elements.a > 6_000_000.0);
    }

    // -----------------------------------------------------------------------
    // 5. MEAN_MOTION + Known center → computed SMA
    // -----------------------------------------------------------------------

    #[test]
    fn mean_motion_with_known_center_computes_sma() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "MEAN_MOTION": 15.5,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0
        }"#;
        let omm = read_omm(json).expect("parse failed");
        let mu = DynOrigin::Earth
            .try_gravitational_parameter()
            .unwrap()
            .as_f64();
        let n = 15.5 * 2.0 * PI / 86400.0;
        let expected_a = (mu / (n * n)).cbrt();
        assert!((omm.mean_elements.elements.a - expected_a).abs() < 1.0);
    }

    // -----------------------------------------------------------------------
    // 6. MEAN_MOTION + Custom center + no GM → error
    // -----------------------------------------------------------------------

    #[test]
    fn mean_motion_custom_center_no_gm_errors() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "APOPHIS",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "MEAN_MOTION": 15.5,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0
        }"#;
        let err = read_omm(json).expect_err("expected error for custom center without GM");
        assert!(
            matches!(err, JsonError::MissingRequiredField(ref k) if k == "GM"),
            "unexpected error: {err}"
        );
    }

    // -----------------------------------------------------------------------
    // 7. MEAN_MOTION + Custom center + wire GM → succeeds
    // -----------------------------------------------------------------------

    #[test]
    fn mean_motion_custom_center_with_wire_gm_succeeds() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "APOPHIS",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "GM": 398600.4415,
            "MEAN_MOTION": 15.5,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0
        }"#;
        let omm = read_omm(json).expect("parse failed");
        assert!(omm.mean_elements.elements.a > 6_000_000.0);
    }

    // -----------------------------------------------------------------------
    // 8. read_omm_list: 2-element array
    // -----------------------------------------------------------------------

    #[test]
    fn read_omm_list_parses_two_elements() {
        let omm = sample_omm();
        let list = vec![omm.clone(), omm];
        let json = write_omm_list(&list).unwrap();
        let parsed = read_omm_list(&json).expect("list parse failed");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].metadata.object_name, "TEST-SAT");
    }

    // -----------------------------------------------------------------------
    // 9. write_omm_list emits JSON array
    // -----------------------------------------------------------------------

    #[test]
    fn write_omm_list_emits_array() {
        let omm = sample_omm();
        let json = write_omm_list(&[omm]).unwrap();
        assert!(json.trim_start().starts_with('['), "expected JSON array");
        assert!(json.trim_end().ends_with(']'), "expected JSON array end");
    }

    // -----------------------------------------------------------------------
    // 10. Missing required field → MissingRequiredField
    // -----------------------------------------------------------------------

    #[test]
    fn missing_object_name_returns_error() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 6860.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0
        }"#;
        let err = read_omm(json).expect_err("expected MissingRequiredField for OBJECT_NAME");
        assert!(
            matches!(err, JsonError::MissingRequiredField(ref k) if k == "OBJECT_NAME"),
            "unexpected error: {err}"
        );
    }

    // -----------------------------------------------------------------------
    // 11. Header comments preserved (single line)
    // -----------------------------------------------------------------------

    #[test]
    fn header_comment_single_line_round_trips() {
        let mut omm = sample_omm();
        omm.header.comments = vec!["A single comment".to_string()];

        let json = write_omm(&omm).unwrap();
        let parsed = read_omm(&json).expect("parse failed");

        assert_eq!(parsed.header.comments, vec!["A single comment"]);
    }

    // -----------------------------------------------------------------------
    // 12. Header comments preserved (multi-line, \n-joined)
    // -----------------------------------------------------------------------

    #[test]
    fn header_comments_multi_line_round_trips() {
        let mut omm = sample_omm();
        omm.header.comments = vec!["Line one".to_string(), "Line two".to_string()];

        let json = write_omm(&omm).unwrap();
        let parsed = read_omm(&json).expect("parse failed");

        assert_eq!(parsed.header.comments, vec!["Line one", "Line two"]);
    }

    // -----------------------------------------------------------------------
    // 13. Space-Track extras round-trip via provider_extras
    // -----------------------------------------------------------------------

    #[test]
    fn space_track_extras_preserved_on_read() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "COMMENT": "GENERATED VIA SPACE-TRACK.ORG API",
            "CREATION_DATE": "2020-12-29T06:26:10",
            "ORIGINATOR": "18 SPCS",
            "OBJECT_NAME": "NUSAT-8 (MARIE)",
            "OBJECT_ID": "2020-003C",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2020-12-29T03:57:59.406624",
            "MEAN_MOTION": "15.27989249",
            "ECCENTRICITY": "0.00133560",
            "INCLINATION": "97.2970",
            "RA_OF_ASC_NODE": "66.4161",
            "ARG_OF_PERICENTER": "110.6345",
            "MEAN_ANOMALY": "334.7107",
            "EPHEMERIS_TYPE": "0",
            "CLASSIFICATION_TYPE": "U",
            "NORAD_CAT_ID": "45018",
            "ELEMENT_SET_NO": "999",
            "REV_AT_EPOCH": "5327",
            "BSTAR": "0.00008455300000",
            "MEAN_MOTION_DOT": "0.00002241",
            "MEAN_MOTION_DDOT": "0.0000000000000",
            "SEMIMAJOR_AXIS": "6859.961",
            "PERIOD": "94.242",
            "APOAPSIS": "490.988",
            "PERIAPSIS": "472.664",
            "OBJECT_TYPE": "PAYLOAD",
            "RCS_SIZE": "MEDIUM",
            "COUNTRY_CODE": "ARGN",
            "LAUNCH_DATE": "2020-01-15",
            "SITE": "TSC",
            "DECAY_DATE": null,
            "FILE": "2911831",
            "GP_ID": "168552672",
            "TLE_LINE0": "0 NUSAT-8 (MARIE)",
            "TLE_LINE1": "1 45018U 20003C   20364.16527091  .00002241  00000-0  84553-4 0  9997",
            "TLE_LINE2": "2 45018  97.2970  66.4161 0013356 110.6345 334.7107 15.27989249 53274"
        }"#;

        let omm = read_omm(json).expect("Space-Track fixture parse failed");
        assert_eq!(omm.metadata.object_name, "NUSAT-8 (MARIE)");
        assert_eq!(omm.metadata.object_id, "2020-003C");
        let tle = omm.tle_parameters.as_ref().expect("missing TLE parameters");
        assert_eq!(tle.norad_cat_id, Some(45018));
        assert!((tle.bstar.unwrap() - 8.4553e-5).abs() < 1e-9);

        // Provider extras (non-CCSDS keys) must be captured on read.
        for key in [
            "TLE_LINE0",
            "TLE_LINE1",
            "TLE_LINE2",
            "OBJECT_TYPE",
            "RCS_SIZE",
            "COUNTRY_CODE",
            "LAUNCH_DATE",
            "SITE",
            "DECAY_DATE",
            "FILE",
            "GP_ID",
            "PERIOD",
            "APOAPSIS",
            "PERIAPSIS",
            "SEMIMAJOR_AXIS",
        ] {
            assert!(
                omm.provider_extras.contains_key(key),
                "expected provider extra `{key}` to be captured"
            );
        }
    }

    // -----------------------------------------------------------------------
    // 14. Wire GM preserved across round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn wire_gm_round_trips() {
        let mut omm = sample_omm();
        let wire_gm = GravitationalParameter::km3_per_s2(398600.4415);
        omm.mean_elements.gm = Some(wire_gm);

        let json = write_omm(&omm).unwrap();
        let parsed = read_omm(&json).expect("parse failed");

        let stored_gm = parsed.mean_elements.gm.expect("GM not preserved");
        let diff = (stored_gm.as_f64() - wire_gm.as_f64()).abs();
        assert!(diff < 1.0, "wire GM round-trip error: diff = {diff}");
    }

    // -----------------------------------------------------------------------
    // Bonus: covariance round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn covariance_round_trips() {
        let mut omm = sample_omm();
        omm.covariance = Some(Covariance {
            comments: Vec::new(),
            frame: None,
            matrix: Matrix6::identity(),
        });

        let json = write_omm(&omm).unwrap();
        let parsed = read_omm(&json).expect("parse failed");

        let cov = parsed.covariance.expect("covariance not preserved");
        assert_eq!(cov.matrix, Matrix6::identity());
    }

    // -----------------------------------------------------------------------
    // Bonus: spacecraft parameters round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn spacecraft_parameters_round_trip() {
        let mut omm = sample_omm();
        omm.spacecraft = Some(SpacecraftParameters {
            comments: Vec::new(),
            mass: Some(Mass::kilograms(120.5)),
            solar_rad_area: Some(Area::square_meters(2.5)),
            solar_rad_coeff: Some(1.2),
            drag_area: Some(Area::square_meters(2.0)),
            drag_coeff: Some(2.2),
        });

        let json = write_omm(&omm).unwrap();
        let parsed = read_omm(&json).expect("parse failed");

        let sp = parsed.spacecraft.expect("spacecraft not preserved");
        assert!((sp.mass.unwrap().to_kilograms() - 120.5).abs() < 1e-9);
        assert_eq!(sp.solar_rad_coeff, Some(1.2));
        assert_eq!(sp.drag_coeff, Some(2.2));
    }

    // -----------------------------------------------------------------------
    // Bonus: bterm and agom round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn bterm_agom_round_trip() {
        let mut omm = sample_omm();
        omm.tle_parameters = Some(TleParameters {
            bterm: Some(AreaToMass::square_meters_per_kilogram(0.05)),
            agom: Some(AreaToMass::square_meters_per_kilogram(0.03)),
            ..TleParameters::default()
        });

        let json = write_omm(&omm).unwrap();
        let parsed = read_omm(&json).expect("parse failed");

        let tle = parsed.tle_parameters.expect("missing TLE parameters");
        assert!((tle.bterm.unwrap().to_square_meters_per_kilogram() - 0.05).abs() < 1e-9);
        assert!((tle.agom.unwrap().to_square_meters_per_kilogram() - 0.03).abs() < 1e-9);
    }

    // -----------------------------------------------------------------------
    // de_f64_lax_opt: null / empty-string / whitespace / malformed
    // -----------------------------------------------------------------------

    #[test]
    fn de_f64_lax_opt_null_returns_none() {
        // JSON null for an optional float field → None
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 6860.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0,
            "GM": null
        }"#;
        let omm = read_omm(json).expect("parse failed");
        assert!(
            omm.mean_elements.gm.is_none(),
            "null GM should produce None"
        );
    }

    #[test]
    fn de_f64_lax_opt_empty_string_returns_none() {
        // Empty-string value for optional f64 → None
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 6860.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0,
            "GM": ""
        }"#;
        let omm = read_omm(json).expect("parse failed");
        assert!(
            omm.mean_elements.gm.is_none(),
            "empty-string GM should produce None"
        );
    }

    #[test]
    fn de_f64_lax_opt_whitespace_string_returns_none() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 6860.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0,
            "GM": "   "
        }"#;
        let omm = read_omm(json).expect("parse failed");
        assert!(
            omm.mean_elements.gm.is_none(),
            "whitespace-only GM should produce None"
        );
    }

    #[test]
    fn de_f64_lax_opt_malformed_string_returns_error() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 6860.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0,
            "GM": "not-a-float"
        }"#;
        let err = read_omm(json).expect_err("should fail on malformed float string");
        // The error should come from serde_json
        assert!(
            matches!(err, JsonError::Json(_)),
            "unexpected error type: {err}"
        );
    }

    // -----------------------------------------------------------------------
    // de_i32_lax_opt: string int / null / malformed / overflow
    // -----------------------------------------------------------------------

    #[test]
    fn de_i32_lax_opt_string_int_works() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 6860.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0,
            "EPHEMERIS_TYPE": "0"
        }"#;
        let omm = read_omm(json).expect("parse failed");
        let tle = omm.tle_parameters.expect("TLE params should be present");
        assert_eq!(tle.ephemeris_type, Some(0));
    }

    #[test]
    fn de_i32_lax_opt_null_returns_none() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 6860.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0,
            "NORAD_CAT_ID": null
        }"#;
        let omm = read_omm(json).expect("parse failed");
        // null NORAD_CAT_ID → None → no tle_parameters (no other TLE fields)
        assert!(
            omm.tle_parameters.is_none(),
            "null NORAD_CAT_ID should produce None TLE block"
        );
    }

    #[test]
    fn de_i32_lax_opt_empty_string_returns_none() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 6860.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0,
            "EPHEMERIS_TYPE": ""
        }"#;
        let omm = read_omm(json).expect("parse failed with empty EPHEMERIS_TYPE");
        assert!(
            omm.tle_parameters.is_none(),
            "empty-string EPHEMERIS_TYPE should produce None"
        );
    }

    // -----------------------------------------------------------------------
    // de_i64_lax_opt / de_u64_lax_opt: string variants
    // -----------------------------------------------------------------------

    #[test]
    fn de_i64_lax_opt_string_int_works() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 6860.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0,
            "ELEMENT_SET_NO": "999"
        }"#;
        let omm = read_omm(json).expect("parse failed");
        let tle = omm.tle_parameters.expect("TLE params should be present");
        assert_eq!(tle.element_set_no, Some(999));
    }

    #[test]
    fn de_u64_lax_opt_string_int_works() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 6860.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0,
            "REV_AT_EPOCH": "5327"
        }"#;
        let omm = read_omm(json).expect("parse failed");
        let tle = omm.tle_parameters.expect("TLE params should be present");
        assert_eq!(tle.rev_at_epoch, Some(5327));
    }

    #[test]
    fn de_u64_lax_opt_empty_string_returns_none() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 6860.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0,
            "REV_AT_EPOCH": ""
        }"#;
        let omm = read_omm(json).expect("parse failed");
        assert!(
            omm.tle_parameters.is_none(),
            "empty-string REV_AT_EPOCH should produce None"
        );
    }

    #[test]
    fn de_i64_lax_opt_empty_string_returns_none() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 6860.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0,
            "ELEMENT_SET_NO": ""
        }"#;
        let omm = read_omm(json).expect("parse failed");
        assert!(
            omm.tle_parameters.is_none(),
            "empty-string ELEMENT_SET_NO should produce None"
        );
    }

    // -----------------------------------------------------------------------
    // Missing SEMI_MAJOR_AXIS and no MEAN_MOTION → error
    // -----------------------------------------------------------------------

    #[test]
    fn missing_sma_and_mean_motion_returns_error() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0
        }"#;
        let err = read_omm(json).expect_err("should fail without SMA or MEAN_MOTION");
        assert!(
            matches!(err, JsonError::MissingRequiredField(ref k) if k == "SEMI_MAJOR_AXIS"),
            "unexpected error: {err}"
        );
    }

    // -----------------------------------------------------------------------
    // Covariance partial → missing required field error
    // -----------------------------------------------------------------------

    #[test]
    fn partial_covariance_missing_cy_x_returns_error() {
        // Provide CX_X but omit CY_X → MissingRequiredField for CY_X
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 6860.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0,
            "CX_X": 1.0
        }"#;
        let err = read_omm(json).expect_err("should fail on missing CY_X");
        assert!(
            matches!(err, JsonError::MissingRequiredField(ref k) if k == "CY_X"),
            "unexpected error: {err}"
        );
    }

    // -----------------------------------------------------------------------
    // de_f64_lax: integer-in-JSON-number parsed as f64
    // -----------------------------------------------------------------------

    #[test]
    fn de_f64_lax_integer_json_number_parsed() {
        // INCLINATION as integer JSON number → f64
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 6860,
            "ECCENTRICITY": 0,
            "INCLINATION": 45,
            "RA_OF_ASC_NODE": 0,
            "ARG_OF_PERICENTER": 0,
            "MEAN_ANOMALY": 0
        }"#;
        let omm = read_omm(json).expect("parse failed with integer JSON numbers");
        assert!(
            omm.mean_elements.elements.e.abs() < 1e-10,
            "eccentricity should be 0"
        );
    }

    // -----------------------------------------------------------------------
    // Invalid epoch strings
    // -----------------------------------------------------------------------

    #[test]
    fn invalid_creation_date_returns_error() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "not-a-date",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 6860.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0
        }"#;
        let err = read_omm(json).expect_err("should fail on invalid CREATION_DATE");
        assert!(
            matches!(err, JsonError::InvalidEpoch { .. }),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn invalid_epoch_string_returns_error() {
        let json = r#"{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "TEST",
            "OBJECT_NAME": "SAT",
            "OBJECT_ID": "2024-000A",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "definitely-not-a-date",
            "SEMI_MAJOR_AXIS": 6860.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 45.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0
        }"#;
        let err = read_omm(json).expect_err("should fail on invalid EPOCH");
        assert!(
            matches!(err, JsonError::InvalidEpoch { .. }),
            "unexpected error: {err}"
        );
    }

    // -----------------------------------------------------------------
    // Lax deserializer visitor-method coverage
    //
    // Each `de_*_lax[_opt]` deserializer has a `Visitor` with multiple
    // `visit_*` methods, one per JSON value type. The tests below give
    // serde each shape of value (int, float, string, null, empty) so
    // every visitor branch is exercised.
    // -----------------------------------------------------------------

    /// Wraps a single OMM field value in an otherwise-valid skeleton.
    /// If `field` already appears in the skeleton, the value is replaced;
    /// otherwise it's appended.
    fn omm_with_value(field: &str, value: &str) -> String {
        let mut fields = vec![
            ("CCSDS_OMM_VERS", "\"2.0\"".to_string()),
            ("CREATION_DATE", "\"2024-01-01T00:00:00\"".to_string()),
            ("ORIGINATOR", "\"TEST\"".to_string()),
            ("OBJECT_NAME", "\"X\"".to_string()),
            ("OBJECT_ID", "\"Y\"".to_string()),
            ("CENTER_NAME", "\"EARTH\"".to_string()),
            ("REF_FRAME", "\"TEME\"".to_string()),
            ("TIME_SYSTEM", "\"UTC\"".to_string()),
            ("MEAN_ELEMENT_THEORY", "\"SGP4\"".to_string()),
            ("EPOCH", "\"2024-01-01T00:00:00\"".to_string()),
            ("SEMI_MAJOR_AXIS", "7000.0".to_string()),
            ("ECCENTRICITY", "0.001".to_string()),
            ("INCLINATION", "0.0".to_string()),
            ("RA_OF_ASC_NODE", "0.0".to_string()),
            ("ARG_OF_PERICENTER", "0.0".to_string()),
            ("MEAN_ANOMALY", "0.0".to_string()),
        ];
        if let Some(entry) = fields.iter_mut().find(|(k, _)| *k == field) {
            entry.1 = value.to_string();
        } else {
            fields.push((field, value.to_string()));
        }
        let body = fields
            .iter()
            .map(|(k, v)| format!("\"{k}\": {v}"))
            .collect::<Vec<_>>()
            .join(",\n");
        format!("{{\n{body}\n}}")
    }

    // --- f64 visitor branches via GM field (Option<f64> stored on the wire) ---

    #[test]
    fn de_f64_lax_opt_accepts_native_float() {
        let omm = read_omm(&omm_with_value("GM", "398600.4")).expect("read");
        let gm = omm.mean_elements.gm.expect("gm set");
        assert!((gm.as_f64() - 3.986004e14).abs() < 1e8);
    }

    #[test]
    fn de_f64_lax_opt_accepts_native_integer() {
        // JSON integer literals (no decimal point) trigger visit_i64 / visit_u64.
        let omm = read_omm(&omm_with_value("GM", "398600")).expect("read");
        let gm = omm.mean_elements.gm.expect("gm set");
        assert!((gm.as_f64() - 3.986e14).abs() < 1e9);
    }

    #[test]
    fn de_f64_lax_opt_accepts_native_negative_integer() {
        let omm = read_omm(&omm_with_value("GM", "-1")).expect("read");
        let gm = omm.mean_elements.gm.expect("gm set");
        // -1 km^3/s^2 = -1e9 m^3/s^2
        assert!((gm.as_f64() - (-1e9)).abs() < 1.0);
    }

    #[test]
    fn de_f64_lax_opt_accepts_quoted_string() {
        let omm = read_omm(&omm_with_value("GM", "\"398600.4\"")).expect("read");
        let gm = omm.mean_elements.gm.expect("gm set");
        assert!((gm.as_f64() - 3.986004e14).abs() < 1e8);
    }

    #[test]
    fn de_f64_lax_opt_treats_null_as_none() {
        let omm = read_omm(&omm_with_value("GM", "null")).expect("read");
        assert!(omm.mean_elements.gm.is_none());
    }

    #[test]
    fn de_f64_lax_opt_treats_empty_string_as_none() {
        let omm = read_omm(&omm_with_value("GM", "\"\"")).expect("read");
        assert!(omm.mean_elements.gm.is_none());
    }

    #[test]
    fn de_f64_lax_opt_treats_whitespace_string_as_none() {
        let omm = read_omm(&omm_with_value("GM", "\"   \"")).expect("read");
        assert!(omm.mean_elements.gm.is_none());
    }

    #[test]
    fn de_f64_lax_opt_rejects_garbage_string() {
        let json = omm_with_value("GM", "\"not-a-number\"");
        let err = read_omm(&json).expect_err("should reject");
        assert!(matches!(err, JsonError::Json(_)));
    }

    // --- i32 visitor branches via NORAD_CAT_ID ---

    #[test]
    fn de_i32_lax_opt_accepts_native_integer() {
        let omm = read_omm(&omm_with_value("NORAD_CAT_ID", "25544")).expect("read");
        let tle = omm.tle_parameters.expect("tle");
        assert_eq!(tle.norad_cat_id, Some(25544));
    }

    #[test]
    fn de_i32_lax_opt_accepts_negative_native_integer() {
        let omm = read_omm(&omm_with_value("NORAD_CAT_ID", "-1")).expect("read");
        let tle = omm.tle_parameters.expect("tle");
        assert_eq!(tle.norad_cat_id, Some(-1));
    }

    #[test]
    fn de_i32_lax_opt_accepts_quoted_string() {
        let omm = read_omm(&omm_with_value("NORAD_CAT_ID", "\"25544\"")).expect("read");
        let tle = omm.tle_parameters.expect("tle");
        assert_eq!(tle.norad_cat_id, Some(25544));
    }

    #[test]
    fn de_i32_lax_opt_treats_null_as_none() {
        let omm = read_omm(&omm_with_value("NORAD_CAT_ID", "null")).expect("read");
        assert!(omm.tle_parameters.is_none() || omm.tle_parameters.unwrap().norad_cat_id.is_none());
    }

    #[test]
    fn de_i32_lax_opt_treats_empty_string_as_none() {
        let omm = read_omm(&omm_with_value("NORAD_CAT_ID", "\"\"")).expect("read");
        assert!(omm.tle_parameters.is_none() || omm.tle_parameters.unwrap().norad_cat_id.is_none());
    }

    #[test]
    fn de_i32_lax_opt_rejects_overflow_native() {
        // i64 value larger than i32::MAX → try_from fails
        let json = omm_with_value("NORAD_CAT_ID", "9999999999");
        let err = read_omm(&json).expect_err("should overflow");
        assert!(matches!(err, JsonError::Json(_)));
    }

    #[test]
    fn de_i32_lax_opt_rejects_overflow_string() {
        let json = omm_with_value("NORAD_CAT_ID", "\"9999999999\"");
        let err = read_omm(&json).expect_err("should overflow");
        assert!(matches!(err, JsonError::Json(_)));
    }

    // --- i64 visitor branches via ELEMENT_SET_NO ---

    #[test]
    fn de_i64_lax_opt_accepts_native_integer() {
        let omm = read_omm(&omm_with_value("ELEMENT_SET_NO", "999")).expect("read");
        let tle = omm.tle_parameters.expect("tle");
        assert_eq!(tle.element_set_no, Some(999));
    }

    #[test]
    fn de_i64_lax_opt_accepts_negative_native_integer() {
        let omm = read_omm(&omm_with_value("ELEMENT_SET_NO", "-42")).expect("read");
        let tle = omm.tle_parameters.expect("tle");
        assert_eq!(tle.element_set_no, Some(-42));
    }

    #[test]
    fn de_i64_lax_opt_accepts_quoted_string() {
        let omm = read_omm(&omm_with_value("ELEMENT_SET_NO", "\"999\"")).expect("read");
        let tle = omm.tle_parameters.expect("tle");
        assert_eq!(tle.element_set_no, Some(999));
    }

    #[test]
    fn de_i64_lax_opt_treats_null_as_none() {
        let omm = read_omm(&omm_with_value("ELEMENT_SET_NO", "null")).expect("read");
        let _ = omm; // accepted
    }

    #[test]
    fn de_i64_lax_opt_treats_empty_string_as_none() {
        let omm = read_omm(&omm_with_value("ELEMENT_SET_NO", "\"\"")).expect("read");
        let _ = omm;
    }

    #[test]
    fn de_i64_lax_opt_rejects_garbage_string() {
        let json = omm_with_value("ELEMENT_SET_NO", "\"xyz\"");
        let err = read_omm(&json).expect_err("should reject");
        assert!(matches!(err, JsonError::Json(_)));
    }

    // --- u64 visitor branches via REV_AT_EPOCH ---

    #[test]
    fn de_u64_lax_opt_accepts_native_integer() {
        let omm = read_omm(&omm_with_value("REV_AT_EPOCH", "5327")).expect("read");
        let tle = omm.tle_parameters.expect("tle");
        assert_eq!(tle.rev_at_epoch, Some(5327));
    }

    #[test]
    fn de_u64_lax_opt_rejects_negative_native_integer() {
        let json = omm_with_value("REV_AT_EPOCH", "-1");
        let err = read_omm(&json).expect_err("u64 cannot be negative");
        assert!(matches!(err, JsonError::Json(_)));
    }

    #[test]
    fn de_u64_lax_opt_accepts_quoted_string() {
        let omm = read_omm(&omm_with_value("REV_AT_EPOCH", "\"5327\"")).expect("read");
        let tle = omm.tle_parameters.expect("tle");
        assert_eq!(tle.rev_at_epoch, Some(5327));
    }

    #[test]
    fn de_u64_lax_opt_treats_null_as_none() {
        let omm = read_omm(&omm_with_value("REV_AT_EPOCH", "null")).expect("read");
        let _ = omm;
    }

    #[test]
    fn de_u64_lax_opt_treats_empty_string_as_none() {
        let omm = read_omm(&omm_with_value("REV_AT_EPOCH", "\"\"")).expect("read");
        let _ = omm;
    }

    #[test]
    fn de_u64_lax_opt_rejects_garbage_string() {
        let json = omm_with_value("REV_AT_EPOCH", "\"xyz\"");
        let err = read_omm(&json).expect_err("should reject");
        assert!(matches!(err, JsonError::Json(_)));
    }

    // --- de_f64_lax (required, non-optional) visitor branches via ECCENTRICITY ---

    #[test]
    fn de_f64_lax_accepts_native_float() {
        // ECCENTRICITY is f64 (required). Native-float input.
        let omm = read_omm(&omm_with_value("INCLINATION", "51.6")).expect("read");
        assert!((omm.mean_elements.elements.i - 51.6_f64.to_radians()).abs() < 1e-12);
    }

    #[test]
    fn de_f64_lax_accepts_native_integer() {
        let omm = read_omm(&omm_with_value("INCLINATION", "0")).expect("read");
        assert!(omm.mean_elements.elements.i.abs() < 1e-12);
    }

    #[test]
    fn de_f64_lax_accepts_quoted_string() {
        let omm = read_omm(&omm_with_value("INCLINATION", "\"51.6\"")).expect("read");
        assert!((omm.mean_elements.elements.i - 51.6_f64.to_radians()).abs() < 1e-12);
    }

    #[test]
    fn de_f64_lax_rejects_garbage_string() {
        let json = omm_with_value("INCLINATION", "\"not-a-number\"");
        let err = read_omm(&json).expect_err("should reject");
        assert!(matches!(err, JsonError::Json(_)));
    }

    // --- visit_some branches via Option<T> wrapping ---
    //
    // serde encounters a `#[serde(deserialize_with = "...")]` field of type
    // `Option<f64>` and calls `deserialize_option` first, which routes
    // through our visitor's `visit_some`. The inner-visitor branches there
    // also need coverage. Adding a value-bearing field exercises the
    // `visit_some` → inner-visitor path.

    #[test]
    fn de_i32_lax_opt_visit_some_inner_path_via_native() {
        // serde-options that route through Option<T> can hit visit_some
        // and the inner visitor's visit_i64/visit_u64.
        let omm = read_omm(&omm_with_value("EPHEMERIS_TYPE", "0")).expect("read");
        let tle = omm.tle_parameters.expect("tle");
        assert_eq!(tle.ephemeris_type, Some(0));
    }

    // -----------------------------------------------------------------
    // Per-field missing-required-string coverage
    //
    // Each `require_str` call site is a separate `?` early-exit. Each
    // test below omits one required string and asserts the matching
    // MissingRequiredField error.
    // -----------------------------------------------------------------

    /// Build a JSON OMM with the named required string replaced by `""`
    /// (which `require_str` treats as missing).
    fn omm_json_empty(field: &str) -> String {
        omm_with_value(field, "\"\"")
    }

    fn assert_missing_str(input: &str, expected: &str) {
        let err = read_omm(input).expect_err(&format!("expected missing {expected}"));
        let JsonError::MissingRequiredField(ref k) = err else {
            panic!("expected MissingRequiredField({expected}), got: {err:?}");
        };
        assert_eq!(k, expected);
    }

    #[test]
    fn missing_originator_returns_error() {
        assert_missing_str(&omm_json_empty("ORIGINATOR"), "ORIGINATOR");
    }
    #[test]
    fn missing_creation_date_returns_error() {
        assert_missing_str(&omm_json_empty("CREATION_DATE"), "CREATION_DATE");
    }
    #[test]
    fn missing_object_id_returns_error() {
        assert_missing_str(&omm_json_empty("OBJECT_ID"), "OBJECT_ID");
    }
    #[test]
    fn missing_center_name_returns_error() {
        assert_missing_str(&omm_json_empty("CENTER_NAME"), "CENTER_NAME");
    }
    #[test]
    fn missing_ref_frame_returns_error() {
        assert_missing_str(&omm_json_empty("REF_FRAME"), "REF_FRAME");
    }
    #[test]
    fn missing_time_system_returns_error() {
        assert_missing_str(&omm_json_empty("TIME_SYSTEM"), "TIME_SYSTEM");
    }
    #[test]
    fn missing_mean_element_theory_returns_error() {
        assert_missing_str(
            &omm_json_empty("MEAN_ELEMENT_THEORY"),
            "MEAN_ELEMENT_THEORY",
        );
    }
    #[test]
    fn missing_epoch_returns_error() {
        assert_missing_str(&omm_json_empty("EPOCH"), "EPOCH");
    }

    // -----------------------------------------------------------------
    // Partial covariance missing-field coverage
    //
    // The covariance block has 21 entries, each requiring all-or-nothing.
    // Once CX_X is present, every other entry must also be present.
    // Each `get(...)` closure call is a distinct `?` path.
    // -----------------------------------------------------------------

    /// Build an OMM JSON with all 21 covariance fields set to 1.0,
    /// then remove the named one to trigger a MissingRequiredField.
    /// CX_X must be present (it's the gate condition); pass any other
    /// field name to omit it.
    fn omm_with_covariance_missing(missing: &str) -> String {
        let all = [
            "CX_X",
            "CY_X",
            "CY_Y",
            "CZ_X",
            "CZ_Y",
            "CZ_Z",
            "CX_DOT_X",
            "CX_DOT_Y",
            "CX_DOT_Z",
            "CX_DOT_X_DOT",
            "CY_DOT_X",
            "CY_DOT_Y",
            "CY_DOT_Z",
            "CY_DOT_X_DOT",
            "CY_DOT_Y_DOT",
            "CZ_DOT_X",
            "CZ_DOT_Y",
            "CZ_DOT_Z",
            "CZ_DOT_X_DOT",
            "CZ_DOT_Y_DOT",
            "CZ_DOT_Z_DOT",
        ];
        let fields = all
            .iter()
            .filter(|k| **k != missing)
            .map(|k| format!(r#""{k}": 1.0"#))
            .collect::<Vec<_>>()
            .join(",\n  ");
        format!(
            r#"{{
            "CCSDS_OMM_VERS": "2.0",
            "CREATION_DATE": "2024-01-01T00:00:00",
            "ORIGINATOR": "T",
            "OBJECT_NAME": "X",
            "OBJECT_ID": "Y",
            "CENTER_NAME": "EARTH",
            "REF_FRAME": "TEME",
            "TIME_SYSTEM": "UTC",
            "MEAN_ELEMENT_THEORY": "SGP4",
            "EPOCH": "2024-01-01T00:00:00",
            "SEMI_MAJOR_AXIS": 7000.0,
            "ECCENTRICITY": 0.001,
            "INCLINATION": 0.0,
            "RA_OF_ASC_NODE": 0.0,
            "ARG_OF_PERICENTER": 0.0,
            "MEAN_ANOMALY": 0.0,
            {fields}
        }}"#
        )
    }

    fn assert_missing_covariance_field(field: &str) {
        let input = omm_with_covariance_missing(field);
        let err = read_omm(&input).expect_err(&format!("expected missing {field}"));
        // Note: `CX_X` missing is the gate condition; without it the
        // covariance block isn't attempted, so omitting it yields no
        // covariance rather than an error. Test the others.
        let JsonError::MissingRequiredField(ref k) = err else {
            panic!("expected MissingRequiredField({field}), got: {err:?}");
        };
        assert_eq!(k, field);
    }

    #[test]
    fn missing_cy_x_in_covariance_returns_error() {
        assert_missing_covariance_field("CY_X");
    }
    #[test]
    fn missing_cy_y_in_covariance_returns_error() {
        assert_missing_covariance_field("CY_Y");
    }
    #[test]
    fn missing_cz_x_in_covariance_returns_error() {
        assert_missing_covariance_field("CZ_X");
    }
    #[test]
    fn missing_cz_y_in_covariance_returns_error() {
        assert_missing_covariance_field("CZ_Y");
    }
    #[test]
    fn missing_cz_z_in_covariance_returns_error() {
        assert_missing_covariance_field("CZ_Z");
    }
    #[test]
    fn missing_cx_dot_x_in_covariance_returns_error() {
        assert_missing_covariance_field("CX_DOT_X");
    }
    #[test]
    fn missing_cx_dot_y_in_covariance_returns_error() {
        assert_missing_covariance_field("CX_DOT_Y");
    }
    #[test]
    fn missing_cx_dot_z_in_covariance_returns_error() {
        assert_missing_covariance_field("CX_DOT_Z");
    }
    #[test]
    fn missing_cx_dot_x_dot_in_covariance_returns_error() {
        assert_missing_covariance_field("CX_DOT_X_DOT");
    }
    #[test]
    fn missing_cy_dot_x_in_covariance_returns_error() {
        assert_missing_covariance_field("CY_DOT_X");
    }
    #[test]
    fn missing_cy_dot_y_in_covariance_returns_error() {
        assert_missing_covariance_field("CY_DOT_Y");
    }
    #[test]
    fn missing_cy_dot_z_in_covariance_returns_error() {
        assert_missing_covariance_field("CY_DOT_Z");
    }
    #[test]
    fn missing_cy_dot_x_dot_in_covariance_returns_error() {
        assert_missing_covariance_field("CY_DOT_X_DOT");
    }
    #[test]
    fn missing_cy_dot_y_dot_in_covariance_returns_error() {
        assert_missing_covariance_field("CY_DOT_Y_DOT");
    }
    #[test]
    fn missing_cz_dot_x_in_covariance_returns_error() {
        assert_missing_covariance_field("CZ_DOT_X");
    }
    #[test]
    fn missing_cz_dot_y_in_covariance_returns_error() {
        assert_missing_covariance_field("CZ_DOT_Y");
    }
    #[test]
    fn missing_cz_dot_z_in_covariance_returns_error() {
        assert_missing_covariance_field("CZ_DOT_Z");
    }
    #[test]
    fn missing_cz_dot_x_dot_in_covariance_returns_error() {
        assert_missing_covariance_field("CZ_DOT_X_DOT");
    }
    #[test]
    fn missing_cz_dot_y_dot_in_covariance_returns_error() {
        assert_missing_covariance_field("CZ_DOT_Y_DOT");
    }
    #[test]
    fn missing_cz_dot_z_dot_in_covariance_returns_error() {
        assert_missing_covariance_field("CZ_DOT_Z_DOT");
    }

    // -----------------------------------------------------------------
    // REF_FRAME_EPOCH invalid → InvalidEpoch
    // -----------------------------------------------------------------

    #[test]
    fn invalid_ref_frame_epoch_returns_error() {
        let json = omm_with_value("REF_FRAME_EPOCH", "\"not-a-date\"");
        let err = read_omm(&json).expect_err("expected InvalidEpoch for REF_FRAME_EPOCH");
        assert!(
            matches!(err, JsonError::InvalidEpoch { ref value, .. } if value == "not-a-date"),
            "wrong error: {err:?}"
        );
    }

    // -----------------------------------------------------------------
    // Lax-deserializer branches not reached by the realistic
    // string/native-number fixtures above.
    // -----------------------------------------------------------------

    // The *required* `de_f64_lax` Visitor (used for ECCENTRICITY etc.)
    // sees JSON integer literals via `visit_i64` / `visit_u64`. The
    // happy-path fixtures all use decimal literals, so add explicit
    // integer-literal coverage.
    #[test]
    fn de_f64_lax_accepts_native_positive_integer_literal() {
        // INCLINATION as a bare integer (no decimal point) routes through
        // visit_u64 → f64 coercion in the required F64 visitor.
        let omm = read_omm(&omm_with_value("INCLINATION", "45")).expect("read");
        assert!((omm.mean_elements.elements.i - 45.0_f64.to_radians()).abs() < 1e-12);
    }

    #[test]
    fn de_f64_lax_accepts_native_negative_integer_literal() {
        // ECCENTRICITY < 0 is unphysical but the lax deserializer still
        // accepts it; the value lands in visit_i64.
        let omm = read_omm(&omm_with_value("ECCENTRICITY", "-1")).expect("read");
        assert_eq!(omm.mean_elements.elements.e, -1.0);
    }

    // Each lax Visitor has an `expecting()` method that serde invokes
    // only when synthesising an `invalid_type` error message. Feeding a
    // JSON array into each field reaches that method.
    fn assert_invalid_type_error(input: &str) {
        let err = read_omm(input).expect_err("expected JSON type error");
        assert!(
            matches!(err, JsonError::Json(_)),
            "expected JsonError::Json wrapping invalid_type, got: {err:?}"
        );
    }

    #[test]
    fn de_f64_lax_required_rejects_array_via_expecting() {
        assert_invalid_type_error(&omm_with_value("ECCENTRICITY", "[1.0]"));
    }

    #[test]
    fn de_f64_lax_opt_rejects_array_via_expecting() {
        assert_invalid_type_error(&omm_with_value("GM", "[1.0]"));
    }

    #[test]
    fn de_i32_lax_opt_rejects_array_via_expecting() {
        assert_invalid_type_error(&omm_with_value("NORAD_CAT_ID", "[1]"));
    }

    #[test]
    fn de_i64_lax_opt_rejects_array_via_expecting() {
        assert_invalid_type_error(&omm_with_value("ELEMENT_SET_NO", "[1]"));
    }

    #[test]
    fn de_u64_lax_opt_rejects_array_via_expecting() {
        assert_invalid_type_error(&omm_with_value("REV_AT_EPOCH", "[1]"));
    }
}
