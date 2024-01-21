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

#[cfg(test)]
mod test {
    use super::*;

    use quick_xml::de::from_str;

    #[test]
    fn test_parse_oem_message() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<oem xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="http://cwe.ccsds.org/moims/docs/MOIMS-NAV/Schemas/ndmxml-1.0-master.xsd"
        id="CCSDS_OEM_VERS" version="2.0">

    <header>
        <CREATION_DATE>2004-281T17:26:06</CREATION_DATE>
        <ORIGINATOR>me</ORIGINATOR>
    </header>
    <body>
        <segment>
            <metadata>
                <OBJECT_NAME>Cassini</OBJECT_NAME>
                <OBJECT_ID>1997-061A</OBJECT_ID>
                <CENTER_NAME>Saturn</CENTER_NAME>
                <REF_FRAME>IAU-Saturn</REF_FRAME>
                <TIME_SYSTEM>UTC</TIME_SYSTEM>
                <START_TIME>2004-100T00:00:00.000000</START_TIME>
                <STOP_TIME>2004-100T01:00:00.000000</STOP_TIME>
                <INTERPOLATION>Hermite</INTERPOLATION>
                <INTERPOLATION_DEGREE>1</INTERPOLATION_DEGREE>
            </metadata>
            <data>
                <stateVector>
                    <EPOCH>2004-100T00:00:00</EPOCH>
                    <X units="km">1</X>
                    <Y>1</Y>
                    <Z>1</Z>
                    <X_DOT units="km/s">1</X_DOT>
                    <Y_DOT>1</Y_DOT>
                    <Z_DOT>1</Z_DOT>
                </stateVector>
                <stateVector>
                    <EPOCH>2004-100T00:00:00</EPOCH>
                    <X>1</X>
                    <Y units="km">1</Y>
                    <Z>1</Z>
                    <X_DOT>1</X_DOT>
                    <Y_DOT units="km/s">1</Y_DOT>
                    <Z_DOT>1</Z_DOT>
                </stateVector>
                <stateVector>
                    <EPOCH>2004-100T00:00:00</EPOCH>
                    <X>1</X>
                    <Y>1</Y>
                    <Z units="km">1</Z>
                    <X_DOT>1</X_DOT>
                    <Y_DOT>1</Y_DOT>
                    <Z_DOT units="km/s">1</Z_DOT>
                </stateVector>
            </data>
        </segment>
    </body>
</oem>"#;

        let message: OemType = from_str(xml).unwrap();

        assert_eq!(
            message,
            OemType {
                header: common::OdmHeader {
                    comment_list: vec![],
                    classification_list: vec![],
                    creation_date: common::EpochType("2004-281T17:26:06".to_string(),),
                    originator: "me".to_string(),
                    message_id: None,
                },
                body: OemBody {
                    segment_list: vec![OemSegment {
                        metadata: OemMetadata {
                            comment_list: vec![],
                            object_name: "Cassini".to_string(),
                            object_id: "1997-061A".to_string(),
                            center_name: "Saturn".to_string(),
                            ref_frame: "IAU-Saturn".to_string(),
                            ref_frame_epoch: None,
                            time_system: "UTC".to_string(),
                            start_time: common::EpochType("2004-100T00:00:00.000000".to_string(),),
                            useable_start_time: None,
                            useable_stop_time: None,
                            stop_time: common::EpochType("2004-100T01:00:00.000000".to_string(),),
                            interpolation: Some("Hermite".to_string(),),
                            interpolation_degree: Some(1,),
                        },
                        data: OemData {
                            comment_list: vec![],
                            state_vector_list: vec![
                                common::StateVectorAccType {
                                    epoch: common::EpochType("2004-100T00:00:00".to_string(),),
                                    x: common::PositionType {
                                        base: 1.0,
                                        units: Some(common::PositionUnits("km".to_string(),),),
                                    },
                                    y: common::PositionType {
                                        base: 1.0,
                                        units: None,
                                    },
                                    z: common::PositionType {
                                        base: 1.0,
                                        units: None,
                                    },
                                    x_dot: common::VelocityType {
                                        base: 1.0,
                                        units: Some(common::VelocityUnits("km/s".to_string(),),),
                                    },
                                    y_dot: common::VelocityType {
                                        base: 1.0,
                                        units: None,
                                    },
                                    z_dot: common::VelocityType {
                                        base: 1.0,
                                        units: None,
                                    },
                                    x_ddot: None,
                                    y_ddot: None,
                                    z_ddot: None,
                                },
                                common::StateVectorAccType {
                                    epoch: common::EpochType("2004-100T00:00:00".to_string(),),
                                    x: common::PositionType {
                                        base: 1.0,
                                        units: None,
                                    },
                                    y: common::PositionType {
                                        base: 1.0,
                                        units: Some(common::PositionUnits("km".to_string(),),),
                                    },
                                    z: common::PositionType {
                                        base: 1.0,
                                        units: None,
                                    },
                                    x_dot: common::VelocityType {
                                        base: 1.0,
                                        units: None,
                                    },
                                    y_dot: common::VelocityType {
                                        base: 1.0,
                                        units: Some(common::VelocityUnits("km/s".to_string(),),),
                                    },
                                    z_dot: common::VelocityType {
                                        base: 1.0,
                                        units: None,
                                    },
                                    x_ddot: None,
                                    y_ddot: None,
                                    z_ddot: None,
                                },
                                common::StateVectorAccType {
                                    epoch: common::EpochType("2004-100T00:00:00".to_string(),),
                                    x: common::PositionType {
                                        base: 1.0,
                                        units: None,
                                    },
                                    y: common::PositionType {
                                        base: 1.0,
                                        units: None,
                                    },
                                    z: common::PositionType {
                                        base: 1.0,
                                        units: Some(common::PositionUnits("km".to_string(),),),
                                    },
                                    x_dot: common::VelocityType {
                                        base: 1.0,
                                        units: None,
                                    },
                                    y_dot: common::VelocityType {
                                        base: 1.0,
                                        units: None,
                                    },
                                    z_dot: common::VelocityType {
                                        base: 1.0,
                                        units: Some(common::VelocityUnits("km/s".to_string(),),),
                                    },
                                    x_ddot: None,
                                    y_ddot: None,
                                    z_ddot: None,
                                },
                            ],
                            covariance_matrix_list: vec![],
                        },
                    },],
                },
                id: "CCSDS_OEM_VERS".to_string(),
                version: "2.0".to_string(),
            }
        );
    }
}
