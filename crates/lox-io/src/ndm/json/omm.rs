/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Deserializers for JSON CCSDS Orbit Mean Elements Message
//!
//! To deserialize a JSON message:
//!
//! ```
//! # use lox_io::ndm::json::omm::OmmType;
//!
//! # let json = r#"{
//! #     "CCSDS_OMM_VERS": "2.0",
//! #     "COMMENT": "GENERATED VIA SPACE-TRACK.ORG API",
//! #     "CREATION_DATE": "2020-12-29T06:26:10",
//! #     "ORIGINATOR": "18 SPCS",
//! #     "OBJECT_NAME": "NUSAT-8 (MARIE)",
//! #     "OBJECT_ID": "2020-003C",
//! #     "CENTER_NAME": "EARTH",
//! #     "REF_FRAME": "TEME",
//! #     "TIME_SYSTEM": "UTC",
//! #     "MEAN_ELEMENT_THEORY": "SGP4",
//! #     "EPOCH": "2020-12-29T03:57:59.406624",
//! #     "MEAN_MOTION": "15.27989249",
//! #     "ECCENTRICITY": "0.00133560",
//! #     "INCLINATION": "97.2970",
//! #     "RA_OF_ASC_NODE": "66.4161",
//! #     "ARG_OF_PERICENTER": "110.6345",
//! #     "MEAN_ANOMALY": "334.7107",
//! #     "EPHEMERIS_TYPE": "0",
//! #     "CLASSIFICATION_TYPE": "U",
//! #     "NORAD_CAT_ID": "45018",
//! #     "ELEMENT_SET_NO": "999",
//! #     "REV_AT_EPOCH": "5327",
//! #     "BSTAR": "0.00008455300000",
//! #     "MEAN_MOTION_DOT": "0.00002241",
//! #     "MEAN_MOTION_DDOT": "0.0000000000000",
//! #     "SEMIMAJOR_AXIS": "6859.961",
//! #     "PERIOD": "94.242",
//! #     "APOAPSIS": "490.988",
//! #     "PERIAPSIS": "472.664",
//! #     "OBJECT_TYPE": "PAYLOAD",
//! #     "RCS_SIZE": "MEDIUM",
//! #     "COUNTRY_CODE": "ARGN",
//! #     "LAUNCH_DATE": "2020-01-15",
//! #     "SITE": "TSC",
//! #     "DECAY_DATE": null,
//! #     "FILE": "2911831",
//! #     "GP_ID": "168552672",
//! #     "TLE_LINE0": "0 NUSAT-8 (MARIE)",
//! #     "TLE_LINE1": "1 45018U 20003C   20364.16527091  .00002241  00000-0  84553-4 0  9997",
//! #     "TLE_LINE2": "2 45018  97.2970  66.4161 0013356 110.6345 334.7107 15.27989249 53274"
//! # }"#;
//! #
//! let message: OmmType = serde_json::de::from_str(json).unwrap();
//! ```
//!
//! To deserialize a list of JSON messages:
//!
//! ```
//! # use lox_io::ndm::json::omm::OmmType;
//! # let json = r#"[{
//! #     "CCSDS_OMM_VERS": "2.0",
//! #     "COMMENT": "GENERATED VIA SPACE-TRACK.ORG API",
//! #     "CREATION_DATE": "2020-12-29T06:26:10",
//! #     "ORIGINATOR": "18 SPCS",
//! #     "OBJECT_NAME": "NUSAT-8 (MARIE)",
//! #     "OBJECT_ID": "2020-003C",
//! #     "CENTER_NAME": "EARTH",
//! #     "REF_FRAME": "TEME",
//! #     "TIME_SYSTEM": "UTC",
//! #     "MEAN_ELEMENT_THEORY": "SGP4",
//! #     "EPOCH": "2020-12-29T03:57:59.406624",
//! #     "MEAN_MOTION": "15.27989249",
//! #     "ECCENTRICITY": "0.00133560",
//! #     "INCLINATION": "97.2970",
//! #     "RA_OF_ASC_NODE": "66.4161",
//! #     "ARG_OF_PERICENTER": "110.6345",
//! #     "MEAN_ANOMALY": "334.7107",
//! #     "EPHEMERIS_TYPE": "0",
//! #     "CLASSIFICATION_TYPE": "U",
//! #     "NORAD_CAT_ID": "45018",
//! #     "ELEMENT_SET_NO": "999",
//! #     "REV_AT_EPOCH": "5327",
//! #     "BSTAR": "0.00008455300000",
//! #     "MEAN_MOTION_DOT": "0.00002241",
//! #     "MEAN_MOTION_DDOT": "0.0000000000000",
//! #     "SEMIMAJOR_AXIS": "6859.961",
//! #     "PERIOD": "94.242",
//! #     "APOAPSIS": "490.988",
//! #     "PERIAPSIS": "472.664",
//! #     "OBJECT_TYPE": "PAYLOAD",
//! #     "RCS_SIZE": "MEDIUM",
//! #     "COUNTRY_CODE": "ARGN",
//! #     "LAUNCH_DATE": "2020-01-15",
//! #     "SITE": "TSC",
//! #     "DECAY_DATE": null,
//! #     "FILE": "2911831",
//! #     "GP_ID": "168552672",
//! #     "TLE_LINE0": "0 NUSAT-8 (MARIE)",
//! #     "TLE_LINE1": "1 45018U 20003C   20364.16527091  .00002241  00000-0  84553-4 0  9997",
//! #     "TLE_LINE2": "2 45018  97.2970  66.4161 0013356 110.6345 334.7107 15.27989249 53274"
//! # }]"#;
//! #
//! let message: Vec<OmmType> = serde_json::de::from_str(json).unwrap();
//! ```

use serde_aux::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SpacecraftParametersType {
    #[serde(
        rename = "MASS",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub mass: Option<f64>,
    #[serde(
        rename = "SOLAR_RAD_AREA",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub solar_rad_area: Option<f64>,
    #[serde(
        rename = "SOLAR_RAD_COEFF",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub solar_rad_coeff: Option<f64>,
    #[serde(
        rename = "DRAG_AREA",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub drag_area: Option<f64>,
    #[serde(
        rename = "DRAG_COEFF",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub drag_coeff: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MeanElementsType {
    #[serde(rename = "EPOCH")]
    pub epoch: String,
    #[serde(rename = "SEMI_MAJOR_AXIS")]
    pub semi_major_axis: Option<f64>,
    #[serde(
        rename = "MEAN_MOTION",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub mean_motion: f64,
    #[serde(
        rename = "ECCENTRICITY",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub eccentricity: f64,
    #[serde(
        rename = "INCLINATION",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub inclination: f64,
    #[serde(
        rename = "RA_OF_ASC_NODE",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub ra_of_asc_node: f64,
    #[serde(
        rename = "ARG_OF_PERICENTER",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub arg_of_pericenter: f64,
    #[serde(
        rename = "MEAN_ANOMALY",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub mean_anomaly: f64,
    #[serde(
        rename = "GM",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub gm: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TleParametersType {
    #[serde(
        rename = "EPHEMERIS_TYPE",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub ephemeris_type: Option<i32>,
    #[serde(rename = "CLASSIFICATION_TYPE")]
    pub classification_type: Option<String>,
    #[serde(
        rename = "NORAD_CAT_ID",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub norad_cat_id: Option<i32>,
    #[serde(
        rename = "ELEMENT_SET_NO",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub element_set_no: Option<i64>,
    #[serde(
        rename = "REV_AT_EPOCH",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub rev_at_epoch: Option<u64>,
    #[serde(
        rename = "BSTAR",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub bstar: Option<f64>,
    #[serde(
        rename = "BTERM",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub bterm: Option<f64>,
    #[serde(
        rename = "MEAN_MOTION_DOT",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub mean_motion_dot: f64,
    #[serde(
        rename = "MEAN_MOTION_DDOT",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub mean_motion_ddot: Option<f64>,
    #[serde(
        rename = "AGOM",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub agom: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OpmCovarianceMatrixType {
    #[serde(rename = "COV_REF_FRAME")]
    pub cov_ref_frame: Option<String>,
    #[serde(rename = "CX_X", deserialize_with = "deserialize_number_from_string")]
    pub cx_x: f64,
    #[serde(rename = "CY_X", deserialize_with = "deserialize_number_from_string")]
    pub cy_x: f64,
    #[serde(rename = "CY_Y", deserialize_with = "deserialize_number_from_string")]
    pub cy_y: f64,
    #[serde(rename = "CZ_X", deserialize_with = "deserialize_number_from_string")]
    pub cz_x: f64,
    #[serde(rename = "CZ_Y", deserialize_with = "deserialize_number_from_string")]
    pub cz_y: f64,
    #[serde(rename = "CZ_Z", deserialize_with = "deserialize_number_from_string")]
    pub cz_z: f64,
    #[serde(
        rename = "CX_DOT_X",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub cx_dot_x: f64,
    #[serde(
        rename = "CX_DOT_Y",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub cx_dot_y: f64,
    #[serde(
        rename = "CX_DOT_Z",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub cx_dot_z: f64,
    #[serde(
        rename = "CX_DOT_X_DOT",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub cx_dot_x_dot: f64,
    #[serde(
        rename = "CY_DOT_X",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub cy_dot_x: f64,
    #[serde(
        rename = "CY_DOT_Y",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub cy_dot_y: f64,
    #[serde(
        rename = "CY_DOT_Z",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub cy_dot_z: f64,
    #[serde(
        rename = "CY_DOT_X_DOT",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub cy_dot_x_dot: f64,
    #[serde(
        rename = "CY_DOT_Y_DOT",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub cy_dot_y_dot: f64,
    #[serde(
        rename = "CZ_DOT_X",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub cz_dot_x: f64,
    #[serde(
        rename = "CZ_DOT_Y",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub cz_dot_y: f64,
    #[serde(
        rename = "CZ_DOT_Z",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub cz_dot_z: f64,
    #[serde(
        rename = "CZ_DOT_X_DOT",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub cz_dot_x_dot: f64,
    #[serde(
        rename = "CZ_DOT_Y_DOT",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub cz_dot_y_dot: f64,
    #[serde(
        rename = "CZ_DOT_Z_DOT",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub cz_dot_z_dot: f64,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OmmMetadata {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "OBJECT_NAME")]
    pub object_name: String,
    #[serde(rename = "OBJECT_ID")]
    pub object_id: String,
    #[serde(rename = "CENTER_NAME")]
    pub center_name: String,
    #[serde(rename = "REF_FRAME")]
    pub ref_frame: String,
    #[serde(rename = "REF_FRAME_EPOCH")]
    pub ref_frame_epoch: Option<String>,
    #[serde(rename = "TIME_SYSTEM")]
    pub time_system: String,
    #[serde(rename = "MEAN_ELEMENT_THEORY")]
    pub mean_element_theory: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OdmHeader {
    #[serde(rename = "COMMENT")]
    pub comment_list: String,
    #[serde(rename = "CLASSIFICATION")]
    pub classification_list: Vec<String>,
    #[serde(rename = "CREATION_DATE")]
    pub creation_date: String,
    #[serde(rename = "ORIGINATOR")]
    pub originator: String,
    #[serde(rename = "MESSAGE_ID")]
    pub message_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OmmType {
    #[serde(flatten)]
    pub header: OdmHeader,

    #[serde(flatten)]
    pub metadata: OmmMetadata,

    #[serde(flatten)]
    pub mean_elements: MeanElementsType,
    #[serde(flatten)]
    pub spacecraft_parameters: Option<SpacecraftParametersType>,
    #[serde(flatten)]
    pub tle_parameters: Option<TleParametersType>,
    #[serde(flatten)]
    pub covariance_matrix: Option<OpmCovarianceMatrixType>,

    #[serde(flatten)]
    pub user_defined_parameters: HashMap<String, Option<String>>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_json() {
        let json = r#"[
    {
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
    }
]"#;

        let message: Vec<OmmType> = serde_json::de::from_str(json).unwrap();

        assert_eq!(
            message,
            [OmmType {
                header: OdmHeader {
                    comment_list: "GENERATED VIA SPACE-TRACK.ORG API".to_string(),
                    classification_list: vec![],
                    creation_date: "2020-12-29T06:26:10".to_string(),
                    originator: "18 SPCS".to_string(),
                    message_id: None,
                },
                metadata: OmmMetadata {
                    comment_list: vec![],
                    object_name: "NUSAT-8 (MARIE)".to_string(),
                    object_id: "2020-003C".to_string(),
                    center_name: "EARTH".to_string(),
                    ref_frame: "TEME".to_string(),
                    ref_frame_epoch: None,
                    time_system: "UTC".to_string(),
                    mean_element_theory: "SGP4".to_string(),
                },
                mean_elements: MeanElementsType {
                    epoch: "2020-12-29T03:57:59.406624".to_string(),
                    semi_major_axis: None,
                    mean_motion: 15.27989249,
                    eccentricity: 0.0013356,
                    inclination: 97.297,
                    ra_of_asc_node: 66.4161,
                    arg_of_pericenter: 110.6345,
                    mean_anomaly: 334.7107,
                    gm: None,
                },
                spacecraft_parameters: Some(SpacecraftParametersType {
                    mass: None,
                    solar_rad_area: None,
                    solar_rad_coeff: None,
                    drag_area: None,
                    drag_coeff: None,
                },),
                tle_parameters: Some(TleParametersType {
                    ephemeris_type: Some(0,),
                    classification_type: Some("U".to_string()),
                    norad_cat_id: Some(45018,),
                    element_set_no: Some(999,),
                    rev_at_epoch: Some(5327,),
                    bstar: Some(8.4553e-5,),
                    bterm: None,
                    mean_motion_dot: 2.241e-5,
                    mean_motion_ddot: Some(0.0,),
                    agom: None,
                },),
                covariance_matrix: Some(OpmCovarianceMatrixType {
                    cov_ref_frame: None,
                    cx_x: 0.0,
                    cy_x: 0.0,
                    cy_y: 0.0,
                    cz_x: 0.0,
                    cz_y: 0.0,
                    cz_z: 0.0,
                    cx_dot_x: 0.0,
                    cx_dot_y: 0.0,
                    cx_dot_z: 0.0,
                    cx_dot_x_dot: 0.0,
                    cy_dot_x: 0.0,
                    cy_dot_y: 0.0,
                    cy_dot_z: 0.0,
                    cy_dot_x_dot: 0.0,
                    cy_dot_y_dot: 0.0,
                    cz_dot_x: 0.0,
                    cz_dot_y: 0.0,
                    cz_dot_z: 0.0,
                    cz_dot_x_dot: 0.0,
                    cz_dot_y_dot: 0.0,
                    cz_dot_z_dot: 0.0,
                },),
                user_defined_parameters: HashMap::from([
                    ("RCS_SIZE".to_string(), Some("MEDIUM".to_string(),)),
                    (
                        "TLE_LINE2".to_string(),
                        Some(
                            "2 45018  97.2970  66.4161 0013356 110.6345 334.7107 15.27989249 53274"
                                .to_string(),
                        )
                    ),
                    ("SITE".to_string(), Some("TSC".to_string(),)),
                    ("LAUNCH_DATE".to_string(), Some("2020-01-15".to_string(),)),
                    ("CCSDS_OMM_VERS".to_string(), Some("2.0".to_string(),)),
                    ("DECAY_DATE".to_string(), None),
                    ("FILE".to_string(), Some("2911831".to_string(),)),
                    ("GP_ID".to_string(), Some("168552672".to_string(),)),
                    ("OBJECT_TYPE".to_string(), Some("PAYLOAD".to_string(),)),
                    ("APOAPSIS".to_string(), Some("490.988".to_string(),)),
                    (
                        "TLE_LINE0".to_string(),
                        Some("0 NUSAT-8 (MARIE)".to_string(),)
                    ),
                    ("PERIOD".to_string(), Some("94.242".to_string(),)),
                    ("SEMIMAJOR_AXIS".to_string(), Some("6859.961".to_string(),)),
                    ("COUNTRY_CODE".to_string(), Some("ARGN".to_string(),)),
                    (
                        "TLE_LINE1".to_string(),
                        Some(
                            "1 45018U 20003C   20364.16527091  .00002241  00000-0  84553-4 0  9997"
                                .to_string(),
                        )
                    ),
                    ("PERIAPSIS".to_string(), Some("472.664".to_string(),)),
                ]),
            },]
        );
    }
}
