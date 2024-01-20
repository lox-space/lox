/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde;

use super::common;

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BStarUnits(#[serde(rename = "$text")] std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BTermUnits(#[serde(rename = "$text")] std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AgomUnits(#[serde(rename = "$text")] std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ElementSetNoType(#[serde(rename = "$text")] std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RevUnits(#[serde(rename = "$text")] std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct DRevUnits(#[serde(rename = "$text")] std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct DdRevUnits(#[serde(rename = "$text")] std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SpacewarnType(#[serde(rename = "$text")] std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OmmType {
    #[serde(rename = "header")]
    pub header: common::OdmHeader,
    #[serde(rename = "body")]
    pub body: OmmBody,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@version")]
    pub version: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OmmBody {
    #[serde(rename = "segment")]
    pub segment: OmmSegment,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OmmSegment {
    #[serde(rename = "metadata")]
    pub metadata: OmmMetadata,
    #[serde(rename = "data")]
    pub data: OmmData,
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
    pub ref_frame_epoch: Option<common::EpochType>,
    #[serde(rename = "TIME_SYSTEM")]
    pub time_system: String,
    #[serde(rename = "MEAN_ELEMENT_THEORY")]
    pub mean_element_theory: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OmmData {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "meanElements")]
    pub mean_elements: MeanElementsType,
    #[serde(rename = "spacecraftParameters")]
    pub spacecraft_parameters: Option<common::SpacecraftParametersType>,
    #[serde(rename = "tleParameters")]
    pub tle_parameters: Option<TleParametersType>,
    #[serde(rename = "covarianceMatrix")]
    pub covariance_matrix: Option<common::OpmCovarianceMatrixType>,
    #[serde(rename = "userDefinedParameters")]
    pub user_defined_parameters: Option<common::UserDefinedType>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MeanElementsType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "EPOCH")]
    pub epoch: common::EpochType,
    #[serde(rename = "SEMI_MAJOR_AXIS")]
    pub semi_major_axis: Option<common::DistanceType>,
    #[serde(rename = "MEAN_MOTION")]
    pub mean_motion: Option<common::RevType>,
    #[serde(rename = "ECCENTRICITY")]
    pub eccentricity: common::NonNegativeDouble,
    #[serde(rename = "INCLINATION")]
    pub inclination: common::InclinationType,
    #[serde(rename = "RA_OF_ASC_NODE")]
    pub ra_of_asc_node: common::AngleType,
    #[serde(rename = "ARG_OF_PERICENTER")]
    pub arg_of_pericenter: common::AngleType,
    #[serde(rename = "MEAN_ANOMALY")]
    pub mean_anomaly: common::AngleType,
    #[serde(rename = "GM")]
    pub gm: Option<common::GmType>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TleParametersType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "EPHEMERIS_TYPE")]
    pub ephemeris_type: Option<i32>,
    #[serde(rename = "CLASSIFICATION_TYPE")]
    pub classification_type: Option<String>,
    #[serde(rename = "NORAD_CAT_ID")]
    pub norad_cat_id: Option<i32>,
    #[serde(rename = "ELEMENT_SET_NO")]
    pub element_set_no: Option<ElementSetNoType>,
    #[serde(rename = "REV_AT_EPOCH")]
    pub rev_at_epoch: Option<u64>,
    #[serde(rename = "BSTAR")]
    pub bstar: Option<common::BStarType>,
    #[serde(rename = "BTERM")]
    pub bterm: Option<common::BTermType>,
    #[serde(rename = "MEAN_MOTION_DOT")]
    pub mean_motion_dot: DRevType,
    #[serde(rename = "MEAN_MOTION_DDOT")]
    pub mean_motion_ddot: Option<common::DRevType>,
    #[serde(rename = "AGOM")]
    pub agom: Option<common::AgomType>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BStarType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<BStarUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BTermType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<BTermUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AgomType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<AgomUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RevType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<RevUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct DRevType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<DRevUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct DdRevType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<DdRevUnits>,
}
