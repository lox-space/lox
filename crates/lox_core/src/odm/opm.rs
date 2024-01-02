use serde;

use super::common;

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OpmType {
    #[serde(rename = "header")]
    pub header: common::OdmHeader,
    #[serde(rename = "body")]
    pub body: common::OpmBody,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@version")]
    pub version: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OpmBody {
    #[serde(rename = "segment")]
    pub segment: common::OpmSegment,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OpmSegment {
    #[serde(rename = "metadata")]
    pub metadata: common::OpmMetadata,
    #[serde(rename = "data")]
    pub data: common::OpmData,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OpmMetadata {
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
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OpmData {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "stateVector")]
    pub state_vector: common::StateVectorType,
    #[serde(rename = "keplerianElements")]
    pub keplerian_elements: Option<common::KeplerianElementsType>,
    #[serde(rename = "spacecraftParameters")]
    pub spacecraft_parameters: Option<common::SpacecraftParametersType>,
    #[serde(rename = "covarianceMatrix")]
    pub covariance_matrix: Option<common::OpmCovarianceMatrixType>,
    #[serde(rename = "maneuverParameters")]
    pub maneuver_parameters_list: Vec<common::ManeuverParametersType>,
    #[serde(rename = "userDefinedParameters")]
    pub user_defined_parameters: Option<common::UserDefinedType>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct KeplerianElementsType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "SEMI_MAJOR_AXIS")]
    pub semi_major_axis: common::DistanceType,
    #[serde(rename = "ECCENTRICITY")]
    pub eccentricity: common::NonNegativeDouble,
    #[serde(rename = "INCLINATION")]
    pub inclination: common::InclinationType,
    #[serde(rename = "RA_OF_ASC_NODE")]
    pub ra_of_asc_node: common::AngleType,
    #[serde(rename = "ARG_OF_PERICENTER")]
    pub arg_of_pericenter: common::AngleType,
    #[serde(rename = "GM")]
    pub gm: common::GmType,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ManeuverParametersType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "MAN_EPOCH_IGNITION")]
    pub man_epoch_ignition: common::EpochType,
    #[serde(rename = "MAN_DURATION")]
    pub man_duration: common::DurationType,
    #[serde(rename = "MAN_DELTA_MASS")]
    pub man_delta_mass: common::DeltamassType,
    #[serde(rename = "MAN_REF_FRAME")]
    pub man_ref_frame: String,
    #[serde(rename = "MAN_DV_1")]
    pub man_dv_1: common::VelocityType,
    #[serde(rename = "MAN_DV_2")]
    pub man_dv_2: common::VelocityType,
    #[serde(rename = "MAN_DV_3")]
    pub man_dv_3: common::VelocityType,
}
