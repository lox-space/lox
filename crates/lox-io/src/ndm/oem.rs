/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Deserializers for XML and KVN CCSDS Orbit Ephemeris Message
//!
//! To deserialize an XML message:
//!
//! ```
//! # let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
//! # <oem xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
//! # xsi:noNamespaceSchemaLocation="http://cwe.ccsds.org/moims/docs/MOIMS-NAV/Schemas/ndmxml-1.0-master.xsd"
//! # id="CCSDS_OEM_VERS" version="2.0">
//! #
//! # <header>
//! # <CREATION_DATE>2004-281T17:26:06</CREATION_DATE>
//! # <ORIGINATOR>me</ORIGINATOR>
//! # </header>
//! # <body>
//! # <segment>
//! #     <metadata>
//! #         <OBJECT_NAME>Cassini</OBJECT_NAME>
//! #         <OBJECT_ID>1997-061A</OBJECT_ID>
//! #         <CENTER_NAME>Saturn</CENTER_NAME>
//! #         <REF_FRAME>IAU-Saturn</REF_FRAME>
//! #         <TIME_SYSTEM>UTC</TIME_SYSTEM>
//! #         <START_TIME>2004-100T00:00:00.000000</START_TIME>
//! #         <STOP_TIME>2004-100T01:00:00.000000</STOP_TIME>
//! #         <INTERPOLATION>Hermite</INTERPOLATION>
//! #         <INTERPOLATION_DEGREE>1</INTERPOLATION_DEGREE>
//! #     </metadata>
//! #     <data>
//! #         <stateVector>
//! #             <EPOCH>2004-100T00:00:00</EPOCH>
//! #             <X units="km">1</X>
//! #             <Y>1</Y>
//! #             <Z>1</Z>
//! #             <X_DOT units="km/s">1</X_DOT>
//! #             <Y_DOT>1</Y_DOT>
//! #             <Z_DOT>1</Z_DOT>
//! #         </stateVector>
//! #         <stateVector>
//! #             <EPOCH>2004-100T00:00:00</EPOCH>
//! #             <X>1</X>
//! #             <Y units="km">1</Y>
//! #             <Z>1</Z>
//! #             <X_DOT>1</X_DOT>
//! #             <Y_DOT units="km/s">1</Y_DOT>
//! #             <Z_DOT>1</Z_DOT>
//! #         </stateVector>
//! #         <stateVector>
//! #             <EPOCH>2004-100T00:00:00</EPOCH>
//! #             <X>1</X>
//! #             <Y>1</Y>
//! #             <Z units="km">1</Z>
//! #             <X_DOT>1</X_DOT>
//! #             <Y_DOT>1</Y_DOT>
//! #             <Z_DOT units="km/s">1</Z_DOT>
//! #         </stateVector>
//! #     </data>
//! # </segment>
//! # </body>
//! # </oem>"#;
//! #
//! # use lox_io::ndm::oem::OemType;
//! #
//! let message: OemType = quick_xml::de::from_str(xml).unwrap();
//! ```

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
    fn test_parse_oem_message1() {
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

    #[test]
    fn test_parse_oem_message2() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<oem  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="http://sanaregistry.org/r/ndmxml/ndmxml-1.0-master.xsd"
        id="CCSDS_OEM_VERS" version="3.0">

    <header>
        <COMMENT>OEM WITH OPTIONAL ACCELERATIONS</COMMENT>
        <CREATION_DATE>1996-11-04T17:22:31</CREATION_DATE>
        <ORIGINATOR>NASA/JPL</ORIGINATOR>
        <MESSAGE_ID>OEM 201113719185</MESSAGE_ID>
    </header>
    <body>
        <segment>
            <metadata>
                <OBJECT_NAME>MARS GLOBAL SURVEYOR</OBJECT_NAME>
                <OBJECT_ID>2000-028A</OBJECT_ID>
                <CENTER_NAME>MARS BARYCENTER</CENTER_NAME>
                <REF_FRAME>J2000</REF_FRAME>
                <TIME_SYSTEM>UTC</TIME_SYSTEM>
                <START_TIME>1996-12-18T12:00:00.331</START_TIME>
                <USEABLE_START_TIME>1996-12-18T12:10:00.331</USEABLE_START_TIME>
                <USEABLE_STOP_TIME>1996-12-28T21:23:00.331</USEABLE_STOP_TIME>
                <STOP_TIME>1996-12-28T21:28:00.331</STOP_TIME>
                <INTERPOLATION>HERMITE</INTERPOLATION>
                <INTERPOLATION_DEGREE>7</INTERPOLATION_DEGREE>
            </metadata>
            <data>
                <COMMENT>Produced by M.R. Sombedody, MSOO NAV/JPL, 1996 OCT 11.  It is</COMMENT>
                <COMMENT>to be used for DSN scheduling purposes only.</COMMENT>
                <stateVector>
                    <EPOCH>1996-12-18T12:00:00.331</EPOCH>
                    <X>2789.6</X>
                    <Y>-280.0</Y>
                    <Z>-1746.8</Z>
                    <X_DOT>4.73</X_DOT>
                    <Y_DOT>-2.50</Y_DOT>
                    <Z_DOT>-1.04</Z_DOT>
                    <X_DDOT>0.008</X_DDOT>
                    <Y_DDOT>0.001</Y_DDOT>
                    <Z_DDOT>-0.159</Z_DDOT>
                </stateVector>
                <stateVector>
                    <EPOCH>1996-12-18T12:01:00.331</EPOCH>
                    <X>2783.4</X>
                    <Y>-308.1</Y>
                    <Z>-1877.1</Z>
                    <X_DOT>5.19</X_DOT>
                    <Y_DOT>-2.42</Y_DOT>
                    <Z_DOT>-2.00</Z_DOT>
                    <X_DDOT>0.008</X_DDOT>
                    <Y_DDOT>0.001</Y_DDOT>
                    <Z_DDOT>0.001</Z_DDOT>
                </stateVector>
                <stateVector>
                    <EPOCH>1996-12-18T12:02:00.331</EPOCH>
                    <X>2776.0</X>
                    <Y>-336.9</Y>
                    <Z>-2008.7</Z>
                    <X_DOT>5.64</X_DOT>
                    <Y_DOT>-2.34</Y_DOT>
                    <Z_DOT>-1.95</Z_DOT>
                    <X_DDOT>0.008</X_DDOT>
                    <Y_DDOT>0.001</Y_DDOT>
                    <Z_DDOT>0.159</Z_DDOT>
                </stateVector>
                <stateVector>
                    <EPOCH>1996-12-28T21:28:00.331</EPOCH>
                    <X>-3881.0</X>
                    <Y>564.0</Y>
                    <Z>-682.8</Z>
                    <X_DOT>-3.29</X_DOT>
                    <Y_DOT>-3.67</Y_DOT>
                    <Z_DOT>1.64</Z_DOT>
                    <X_DDOT>-0.003</X_DDOT>
                    <Y_DDOT>0.000</Y_DDOT>
                    <Z_DDOT>0.000</Z_DDOT>
                </stateVector>
                <covarianceMatrix>
                    <COMMENT>blabla</COMMENT>
                    <EPOCH>1996-12-28T22:28:00.331</EPOCH>
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
            </data>
        </segment>
    </body>
</oem>"#;

        let message: OemType = from_str(xml).unwrap();

        assert_eq!(
            message,
            OemType {
                header: common::OdmHeader {
                    comment_list: vec!["OEM WITH OPTIONAL ACCELERATIONS".to_string(),],
                    classification_list: vec![],
                    creation_date: common::EpochType("1996-11-04T17:22:31".to_string(),),
                    originator: "NASA/JPL".to_string(),
                    message_id: Some("OEM 201113719185".to_string(),),
                },
                body: OemBody {
                    segment_list: vec![OemSegment {
                        metadata: OemMetadata {
                            comment_list: vec![],
                            object_name: "MARS GLOBAL SURVEYOR".to_string(),
                            object_id: "2000-028A".to_string(),
                            center_name: "MARS BARYCENTER".to_string(),
                            ref_frame: "J2000".to_string(),
                            ref_frame_epoch: None,
                            time_system: "UTC".to_string(),
                            start_time: common::EpochType("1996-12-18T12:00:00.331".to_string(),),
                            useable_start_time: Some(common::EpochType(
                                "1996-12-18T12:10:00.331".to_string(),
                            ),),
                            useable_stop_time: Some(common::EpochType(
                                "1996-12-28T21:23:00.331".to_string(),
                            ),),
                            stop_time: common::EpochType("1996-12-28T21:28:00.331".to_string(),),
                            interpolation: Some("HERMITE".to_string(),),
                            interpolation_degree: Some(7,),
                        },
                        data: OemData {
                            comment_list: vec![
                                "Produced by M.R. Sombedody, MSOO NAV/JPL, 1996 OCT 11.  It is"
                                    .to_string(),
                                "to be used for DSN scheduling purposes only.".to_string(),
                            ],
                            state_vector_list: vec![
                                common::StateVectorAccType {
                                    epoch: common::EpochType("1996-12-18T12:00:00.331".to_string(),),
                                    x: common::PositionType {
                                        base: 2789.6,
                                        units: None,
                                    },
                                    y: common::PositionType {
                                        base: -280.0,
                                        units: None,
                                    },
                                    z: common::PositionType {
                                        base: -1746.8,
                                        units: None,
                                    },
                                    x_dot: common::VelocityType {
                                        base: 4.73,
                                        units: None,
                                    },
                                    y_dot: common::VelocityType {
                                        base: -2.5,
                                        units: None,
                                    },
                                    z_dot: common::VelocityType {
                                        base: -1.04,
                                        units: None,
                                    },
                                    x_ddot: Some(common::AccType {
                                        base: 0.008,
                                        units: None,
                                    },),
                                    y_ddot: Some(common::AccType {
                                        base: 0.001,
                                        units: None,
                                    },),
                                    z_ddot: Some(common::AccType {
                                        base: -0.159,
                                        units: None,
                                    },),
                                },
                                common::StateVectorAccType {
                                    epoch: common::EpochType("1996-12-18T12:01:00.331".to_string(),),
                                    x: common::PositionType {
                                        base: 2783.4,
                                        units: None,
                                    },
                                    y: common::PositionType {
                                        base: -308.1,
                                        units: None,
                                    },
                                    z: common::PositionType {
                                        base: -1877.1,
                                        units: None,
                                    },
                                    x_dot: common::VelocityType {
                                        base: 5.19,
                                        units: None,
                                    },
                                    y_dot: common::VelocityType {
                                        base: -2.42,
                                        units: None,
                                    },
                                    z_dot: common::VelocityType {
                                        base: -2.0,
                                        units: None,
                                    },
                                    x_ddot: Some(common::AccType {
                                        base: 0.008,
                                        units: None,
                                    },),
                                    y_ddot: Some(common::AccType {
                                        base: 0.001,
                                        units: None,
                                    },),
                                    z_ddot: Some(common::AccType {
                                        base: 0.001,
                                        units: None,
                                    },),
                                },
                                common::StateVectorAccType {
                                    epoch: common::EpochType("1996-12-18T12:02:00.331".to_string(),),
                                    x: common::PositionType {
                                        base: 2776.0,
                                        units: None,
                                    },
                                    y: common::PositionType {
                                        base: -336.9,
                                        units: None,
                                    },
                                    z: common::PositionType {
                                        base: -2008.7,
                                        units: None,
                                    },
                                    x_dot: common::VelocityType {
                                        base: 5.64,
                                        units: None,
                                    },
                                    y_dot: common::VelocityType {
                                        base: -2.34,
                                        units: None,
                                    },
                                    z_dot: common::VelocityType {
                                        base: -1.95,
                                        units: None,
                                    },
                                    x_ddot: Some(common::AccType {
                                        base: 0.008,
                                        units: None,
                                    },),
                                    y_ddot: Some(common::AccType {
                                        base: 0.001,
                                        units: None,
                                    },),
                                    z_ddot: Some(common::AccType {
                                        base: 0.159,
                                        units: None,
                                    },),
                                },
                                common::StateVectorAccType {
                                    epoch: common::EpochType("1996-12-28T21:28:00.331".to_string(),),
                                    x: common::PositionType {
                                        base: -3881.0,
                                        units: None,
                                    },
                                    y: common::PositionType {
                                        base: 564.0,
                                        units: None,
                                    },
                                    z: common::PositionType {
                                        base: -682.8,
                                        units: None,
                                    },
                                    x_dot: common::VelocityType {
                                        base: -3.29,
                                        units: None,
                                    },
                                    y_dot: common::VelocityType {
                                        base: -3.67,
                                        units: None,
                                    },
                                    z_dot: common::VelocityType {
                                        base: 1.64,
                                        units: None,
                                    },
                                    x_ddot: Some(common::AccType {
                                        base: -0.003,
                                        units: None,
                                    },),
                                    y_ddot: Some(common::AccType {
                                        base: 0.0,
                                        units: None,
                                    },),
                                    z_ddot: Some(common::AccType {
                                        base: 0.0,
                                        units: None,
                                    },),
                                },
                            ],
                            covariance_matrix_list: vec![common::OemCovarianceMatrixType {
                                comment_list: vec!["blabla".to_string(),],
                                epoch: common::EpochType("1996-12-28T22:28:00.331".to_string(),),
                                cov_ref_frame: Some("ITRF1997".to_string(),),
                                cx_x: common::PositionCovarianceType {
                                    base: 0.316,
                                    units: None,
                                },
                                cy_x: common::PositionCovarianceType {
                                    base: 0.722,
                                    units: None,
                                },
                                cy_y: common::PositionCovarianceType {
                                    base: 0.518,
                                    units: None,
                                },
                                cz_x: common::PositionCovarianceType {
                                    base: 0.202,
                                    units: None,
                                },
                                cz_y: common::PositionCovarianceType {
                                    base: 0.715,
                                    units: None,
                                },
                                cz_z: common::PositionCovarianceType {
                                    base: 0.002,
                                    units: None,
                                },
                                cx_dot_x: common::PositionVelocityCovarianceType {
                                    base: 0.912,
                                    units: None,
                                },
                                cx_dot_y: common::PositionVelocityCovarianceType {
                                    base: 0.306,
                                    units: None,
                                },
                                cx_dot_z: common::PositionVelocityCovarianceType {
                                    base: 0.276,
                                    units: None,
                                },
                                cx_dot_x_dot: common::VelocityCovarianceType {
                                    base: 0.797,
                                    units: None,
                                },
                                cy_dot_x: common::PositionVelocityCovarianceType {
                                    base: 0.562,
                                    units: None,
                                },
                                cy_dot_y: common::PositionVelocityCovarianceType {
                                    base: 0.899,
                                    units: None,
                                },
                                cy_dot_z: common::PositionVelocityCovarianceType {
                                    base: 0.022,
                                    units: None,
                                },
                                cy_dot_x_dot: common::VelocityCovarianceType {
                                    base: 0.079,
                                    units: None,
                                },
                                cy_dot_y_dot: common::VelocityCovarianceType {
                                    base: 0.415,
                                    units: None,
                                },
                                cz_dot_x: common::PositionVelocityCovarianceType {
                                    base: 0.245,
                                    units: None,
                                },
                                cz_dot_y: common::PositionVelocityCovarianceType {
                                    base: 0.965,
                                    units: None,
                                },
                                cz_dot_z: common::PositionVelocityCovarianceType {
                                    base: 0.95,
                                    units: None,
                                },
                                cz_dot_x_dot: common::VelocityCovarianceType {
                                    base: 0.435,
                                    units: None,
                                },
                                cz_dot_y_dot: common::VelocityCovarianceType {
                                    base: 0.621,
                                    units: None,
                                },
                                cz_dot_z_dot: common::VelocityCovarianceType {
                                    base: 0.991,
                                    units: None,
                                },
                            },],
                        },
                    },],
                },
                id: "CCSDS_OEM_VERS".to_string(),
                version: "3.0".to_string(),
            }
        );
    }
}
