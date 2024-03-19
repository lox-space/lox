/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;

use serde;

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SpacecraftParametersType {
    #[serde(rename = "MASS")]
    pub mass: Option<f64>,
    #[serde(rename = "SOLAR_RAD_AREA")]
    pub solar_rad_area: Option<f64>,
    #[serde(rename = "SOLAR_RAD_COEFF")]
    pub solar_rad_coeff: Option<f64>,
    #[serde(rename = "DRAG_AREA")]
    pub drag_area: Option<f64>,
    #[serde(rename = "DRAG_COEFF")]
    pub drag_coeff: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MeanElementsType {
    #[serde(rename = "EPOCH")]
    pub epoch: String,
    #[serde(rename = "SEMI_MAJOR_AXIS")]
    pub semi_major_axis: Option<f64>,
    #[serde(rename = "MEAN_MOTION")]
    pub mean_motion: f64,
    #[serde(rename = "ECCENTRICITY")]
    pub eccentricity: f64,
    #[serde(rename = "INCLINATION")]
    pub inclination: f64,
    #[serde(rename = "RA_OF_ASC_NODE")]
    pub ra_of_asc_node: f64,
    #[serde(rename = "ARG_OF_PERICENTER")]
    pub arg_of_pericenter: f64,
    #[serde(rename = "MEAN_ANOMALY")]
    pub mean_anomaly: f64,
    #[serde(rename = "GM")]
    pub gm: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TleParametersType {
    #[serde(rename = "EPHEMERIS_TYPE")]
    pub ephemeris_type: Option<i32>,
    #[serde(rename = "CLASSIFICATION_TYPE")]
    pub classification_type: Option<String>,
    #[serde(rename = "NORAD_CAT_ID")]
    pub norad_cat_id: Option<i32>,
    #[serde(rename = "ELEMENT_SET_NO")]
    pub element_set_no: Option<i64>,
    #[serde(rename = "REV_AT_EPOCH")]
    pub rev_at_epoch: Option<u64>,
    #[serde(rename = "BSTAR")]
    pub bstar: Option<f64>,
    #[serde(rename = "BTERM")]
    pub bterm: Option<f64>,
    #[serde(rename = "MEAN_MOTION_DOT")]
    pub mean_motion_dot: f64,
    #[serde(rename = "MEAN_MOTION_DDOT")]
    pub mean_motion_ddot: Option<f64>,
    #[serde(rename = "AGOM")]
    pub agom: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OpmCovarianceMatrixType {
    #[serde(rename = "COV_REF_FRAME")]
    pub cov_ref_frame: Option<String>,
    #[serde(rename = "CX_X")]
    pub cx_x: f64,
    #[serde(rename = "CY_X")]
    pub cy_x: f64,
    #[serde(rename = "CY_Y")]
    pub cy_y: f64,
    #[serde(rename = "CZ_X")]
    pub cz_x: f64,
    #[serde(rename = "CZ_Y")]
    pub cz_y: f64,
    #[serde(rename = "CZ_Z")]
    pub cz_z: f64,
    #[serde(rename = "CX_DOT_X")]
    pub cx_dot_x: f64,
    #[serde(rename = "CX_DOT_Y")]
    pub cx_dot_y: f64,
    #[serde(rename = "CX_DOT_Z")]
    pub cx_dot_z: f64,
    #[serde(rename = "CX_DOT_X_DOT")]
    pub cx_dot_x_dot: f64,
    #[serde(rename = "CY_DOT_X")]
    pub cy_dot_x: f64,
    #[serde(rename = "CY_DOT_Y")]
    pub cy_dot_y: f64,
    #[serde(rename = "CY_DOT_Z")]
    pub cy_dot_z: f64,
    #[serde(rename = "CY_DOT_X_DOT")]
    pub cy_dot_x_dot: f64,
    #[serde(rename = "CY_DOT_Y_DOT")]
    pub cy_dot_y_dot: f64,
    #[serde(rename = "CZ_DOT_X")]
    pub cz_dot_x: f64,
    #[serde(rename = "CZ_DOT_Y")]
    pub cz_dot_y: f64,
    #[serde(rename = "CZ_DOT_Z")]
    pub cz_dot_z: f64,
    #[serde(rename = "CZ_DOT_X_DOT")]
    pub cz_dot_x_dot: f64,
    #[serde(rename = "CZ_DOT_Y_DOT")]
    pub cz_dot_y_dot: f64,
    #[serde(rename = "CZ_DOT_Z_DOT")]
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
    #[serde(rename = "COMMENT")] //@TODO
    pub comment_list: Vec<String>,
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
    pub user_defined_parameters: HashMap<String, String>,
}

mod test {
    use super::*;

    #[test]
    fn test_parse_json() {
        let json = r#"
     {
         "OBJECT_NAME": "SORAYA",
         "OBJECT_ID": "2024-015A",
         "EPOCH": "2024-02-18T10:40:46.668576",
         "MEAN_MOTION": 14.41921171,
         "ECCENTRICITY": 0.0010889,
         "INCLINATION": 64.5147,
         "RA_OF_ASC_NODE": 28.0601,
         "ARG_OF_PERICENTER": 260.5688,
         "MEAN_ANOMALY": 99.4167,
         "EPHEMERIS_TYPE": 0,
         "CLASSIFICATION_TYPE": "U",
         "NORAD_CAT_ID": 58817,
         "ELEMENT_SET_NO": 999,
         "REV_AT_EPOCH": 421,
         "BSTAR": 0.00015073,
         "MEAN_MOTION_DOT": 0.00000388,
         "MEAN_MOTION_DDOT": 0,
         "USER_DEFINED_FOO": "asd"
     }"#;

        let message: OmmType = serde_json::de::from_str(json).unwrap();

        assert_eq!(
            message,
            OmmType {
                header: OdmHeader {
                    comment_list: vec![],
                    classification_list: vec![],
                    creation_date: "".to_string(),
                    originator: "".to_string(),
                    message_id: None,
                },
                metadata: OmmMetadata {
                    comment_list: vec![],
                    object_name: "SORAYA".to_string(),
                    object_id: "2024-015A".to_string(),
                    center_name: "".to_string(),
                    ref_frame: "".to_string(),
                    ref_frame_epoch: None,
                    time_system: "".to_string(),
                    mean_element_theory: "".to_string(),
                },
                mean_elements: MeanElementsType {
                    epoch: "2024-02-18T10:40:46.668576".to_string(),
                    semi_major_axis: None,
                    mean_motion: 14.41921171,
                    eccentricity: 0.0010889,
                    inclination: 64.5147,
                    ra_of_asc_node: 28.0601,
                    arg_of_pericenter: 260.5688,
                    mean_anomaly: 99.4167,
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
                    classification_type: Some("U".to_string(),),
                    norad_cat_id: Some(58817,),
                    element_set_no: Some(999,),
                    rev_at_epoch: Some(421,),
                    bstar: Some(0.00015073,),
                    bterm: None,
                    mean_motion_dot: 3.88e-6,
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
                user_defined_parameters: HashMap::from([(
                    "USER_DEFINED_FOO".to_string(),
                    "asd".to_string()
                )])
            }
        );
    }
}
