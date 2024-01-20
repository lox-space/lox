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
pub struct OemType {
    #[serde(rename = "header")]
    pub header: common::OdmHeader,
    #[serde(rename = "body")]
    pub body: OemBody,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@version")]
    pub version: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OemBody {
    #[serde(rename = "segment")]
    pub segment_list: Vec<OemSegment>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OemSegment {
    #[serde(rename = "metadata")]
    pub metadata: OemMetadata,
    #[serde(rename = "data")]
    pub data: OemData,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OemMetadata {
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
    #[serde(rename = "START_TIME")]
    pub start_time: common::EpochType,
    #[serde(rename = "USEABLE_START_TIME")]
    pub useable_start_time: Option<common::EpochType>,
    #[serde(rename = "USEABLE_STOP_TIME")]
    pub useable_stop_time: Option<common::EpochType>,
    #[serde(rename = "STOP_TIME")]
    pub stop_time: common::EpochType,
    #[serde(rename = "INTERPOLATION")]
    pub interpolation: Option<String>,
    #[serde(rename = "INTERPOLATION_DEGREE")]
    pub interpolation_degree: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OemData {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "stateVector")]
    pub state_vector_list: Vec<common::StateVectorAccType>,
    #[serde(rename = "covarianceMatrix")]
    pub covariance_matrix_list: Vec<common::OemCovarianceMatrixType>,
}
