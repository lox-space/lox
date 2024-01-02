mod schema_omm {
    pub mod xml_schema_types {
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
            pub header: schema_common::xml_schema_types::OdmHeader,
            #[serde(rename = "body")]
            pub body: schema_common::xml_schema_types::OmmBody,
            #[serde(rename = "@id")]
            pub id: String,
            #[serde(rename = "@version")]
            pub version: String,
        }

        #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(default)]
        pub struct OmmBody {
            #[serde(rename = "segment")]
            pub segment: schema_common::xml_schema_types::OmmSegment,
        }

        #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(default)]
        pub struct OmmSegment {
            #[serde(rename = "metadata")]
            pub metadata: schema_common::xml_schema_types::OmmMetadata,
            #[serde(rename = "data")]
            pub data: schema_common::xml_schema_types::OmmData,
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
            pub ref_frame_epoch: Option<schema_common::xml_schema_types::EpochType>,
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
            pub mean_elements: schema_common::xml_schema_types::MeanElementsType,
            #[serde(rename = "spacecraftParameters")]
            pub spacecraft_parameters:
                Option<schema_common::xml_schema_types::SpacecraftParametersType>,
            #[serde(rename = "tleParameters")]
            pub tle_parameters: Option<schema_common::xml_schema_types::TleParametersType>,
            #[serde(rename = "covarianceMatrix")]
            pub covariance_matrix: Option<schema_common::xml_schema_types::OpmCovarianceMatrixType>,
            #[serde(rename = "userDefinedParameters")]
            pub user_defined_parameters: Option<schema_common::xml_schema_types::UserDefinedType>,
        }

        #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(default)]
        pub struct MeanElementsType {
            #[serde(rename = "COMMENT")]
            pub comment_list: Vec<String>,
            #[serde(rename = "EPOCH")]
            pub epoch: schema_common::xml_schema_types::EpochType,
            #[serde(rename = "ECCENTRICITY")]
            pub eccentricity: schema_common::xml_schema_types::NonNegativeDouble,
            #[serde(rename = "INCLINATION")]
            pub inclination: schema_common::xml_schema_types::InclinationType,
            #[serde(rename = "RA_OF_ASC_NODE")]
            pub ra_of_asc_node: schema_common::xml_schema_types::AngleType,
            #[serde(rename = "ARG_OF_PERICENTER")]
            pub arg_of_pericenter: schema_common::xml_schema_types::AngleType,
            #[serde(rename = "MEAN_ANOMALY")]
            pub mean_anomaly: schema_common::xml_schema_types::AngleType,
            #[serde(rename = "GM")]
            pub gm: Option<schema_common::xml_schema_types::GmType>,
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
            pub element_set_no: Option<schema_common::xml_schema_types::ElementSetNoType>,
            #[serde(rename = "REV_AT_EPOCH")]
            pub rev_at_epoch: Option<u64>,
            #[serde(rename = "MEAN_MOTION_DOT")]
            pub mean_motion_dot: schema_common::xml_schema_types::DRevType,
        }

        #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(default)]
        pub struct BStarType {
            #[serde(rename = "$text")]
            pub base: f64,
            #[serde(rename = "@units")]
            pub units: Option<schema_common::xml_schema_types::BStarUnits>,
        }

        #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(default)]
        pub struct BTermType {
            #[serde(rename = "$text")]
            pub base: f64,
            #[serde(rename = "@units")]
            pub units: Option<schema_common::xml_schema_types::BTermUnits>,
        }

        #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(default)]
        pub struct AgomType {
            #[serde(rename = "$text")]
            pub base: f64,
            #[serde(rename = "@units")]
            pub units: Option<schema_common::xml_schema_types::AgomUnits>,
        }

        #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(default)]
        pub struct RevType {
            #[serde(rename = "$text")]
            pub base: f64,
            #[serde(rename = "@units")]
            pub units: Option<schema_common::xml_schema_types::RevUnits>,
        }

        #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(default)]
        pub struct DRevType {
            #[serde(rename = "$text")]
            pub base: f64,
            #[serde(rename = "@units")]
            pub units: Option<schema_common::xml_schema_types::DRevUnits>,
        }

        #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(default)]
        pub struct DdRevType {
            #[serde(rename = "$text")]
            pub base: f64,
            #[serde(rename = "@units")]
            pub units: Option<schema_common::xml_schema_types::DdRevUnits>,
        }
    }
}
pub use schema_omm::*;
