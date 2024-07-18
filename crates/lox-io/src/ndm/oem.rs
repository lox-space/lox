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
//! use lox_io::ndm::xml::FromXmlStr;
//!
//! let message = OemType::from_xml_str(xml).unwrap();
//! ```
//!
//! To deserialize a KVN message:
//! ```
//! # let kvn = r#"CCSDS_OEM_VERS = 3.0
//! # CREATION_DATE = 1996-11-04T17:22:31
//! # ORIGINATOR = NASA/JPL
//! # META_START
//! # OBJECT_NAME         = MARS GLOBAL SURVEYOR
//! # OBJECT_ID           = 1996-062A
//! # CENTER_NAME         = MARS BARYCENTER
//! # REF_FRAME           = J2000
//! # TIME_SYSTEM         = TAI
//! # START_TIME          = 1996-12-18T12:00:00.331
//! # USEABLE_START_TIME  = 1996-12-18T12:10:00.331
//! # USEABLE_STOP_TIME   = 1996-12-28T21:23:00.331
//! # STOP_TIME           = 1996-12-28T21:28:00.331
//! # INTERPOLATION       = HERMITE
//! # INTERPOLATION_DEGREE = 7
//! # META_STOP"#;
//! #
//! # use lox_io::ndm::oem::OemType;
//! use lox_io::ndm::kvn::KvnDeserializer;
//!
//! let message: OemType = KvnDeserializer::from_kvn_str(&kvn).unwrap();
//! ```

// This file is partially generated with xml-schema-derive from the XSD schema
// published by CCSDS. Adaptations have been made to simplify the types or
// allow to simplify the implementation of the KVN parser.

use serde;

use super::{common, kvn::parser::KvnStateVectorValue};

#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    lox_derive::KvnDeserialize,
)]
#[serde(default)]
pub struct OemType {
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "@version")]
    pub version: String,
    #[serde(rename = "header")]
    pub header: common::OdmHeader,
    #[serde(rename = "body")]
    pub body: OemBody,
}

impl crate::ndm::xml::FromXmlStr<'_> for OemType {}

#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    lox_derive::KvnDeserialize,
)]
#[serde(default)]
pub struct OemBody {
    #[serde(rename = "segment")]
    pub segment_list: Vec<OemSegment>,
}

#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    lox_derive::KvnDeserialize,
)]
#[serde(default)]
pub struct OemSegment {
    #[serde(rename = "metadata")]
    pub metadata: OemMetadata,
    #[serde(rename = "data")]
    pub data: OemData,
}

#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    lox_derive::KvnDeserialize,
)]
#[serde(default)]
#[kvn(prefix_and_postfix_keyword = "META")]
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

#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    lox_derive::KvnDeserialize,
)]
#[serde(default)]
pub struct OemData {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "stateVector")]
    pub state_vector_list: Vec<common::StateVectorAccType>,
    #[serde(rename = "covarianceMatrix")]
    #[kvn(prefix_and_postfix_keyword = "COVARIANCE")]
    pub covariance_matrix_list: Vec<common::OemCovarianceMatrixType>,
}

impl From<KvnStateVectorValue> for crate::ndm::common::StateVectorAccType {
    fn from(value: KvnStateVectorValue) -> Self {
        Self {
            epoch: crate::ndm::common::EpochType(value.epoch.full_value),
            x: crate::ndm::common::PositionType {
                base: value.x,
                units: None,
            },
            y: crate::ndm::common::PositionType {
                base: value.y,
                units: None,
            },
            z: crate::ndm::common::PositionType {
                base: value.z,
                units: None,
            },
            x_dot: crate::ndm::common::VelocityType {
                base: value.x_dot,
                units: None,
            },
            y_dot: crate::ndm::common::VelocityType {
                base: value.y_dot,
                units: None,
            },
            z_dot: crate::ndm::common::VelocityType {
                base: value.z_dot,
                units: None,
            },
            x_ddot: value.x_ddot.map(|x_ddot| crate::ndm::common::AccType {
                base: x_ddot,
                units: None,
            }),
            y_ddot: value.y_ddot.map(|y_ddot| crate::ndm::common::AccType {
                base: y_ddot,
                units: None,
            }),
            z_ddot: value.z_ddot.map(|z_ddot| crate::ndm::common::AccType {
                base: z_ddot,
                units: None,
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ndm::xml::FromXmlStr;

    use super::*;

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

        let message = OemType::from_xml_str(xml).unwrap();

        assert_eq!(
            message,
            OemType {
                header: common::OdmHeader {
                    comment_list: vec![],
                    classification_list: vec![],
                    creation_date: common::EpochType("2004-281T17:26:06".to_string()),
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
                            start_time: common::EpochType("2004-100T00:00:00.000000".to_string()),
                            useable_start_time: None,
                            useable_stop_time: None,
                            stop_time: common::EpochType("2004-100T01:00:00.000000".to_string()),
                            interpolation: Some("Hermite".to_string()),
                            interpolation_degree: Some(1,),
                        },
                        data: OemData {
                            comment_list: vec![],
                            state_vector_list: vec![
                                common::StateVectorAccType {
                                    epoch: common::EpochType("2004-100T00:00:00".to_string()),
                                    x: common::PositionType {
                                        base: 1.0,
                                        units: Some(common::PositionUnits("km".to_string()),),
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
                                        units: Some(common::VelocityUnits("km/s".to_string()),),
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
                                    epoch: common::EpochType("2004-100T00:00:00".to_string()),
                                    x: common::PositionType {
                                        base: 1.0,
                                        units: None,
                                    },
                                    y: common::PositionType {
                                        base: 1.0,
                                        units: Some(common::PositionUnits("km".to_string()),),
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
                                        units: Some(common::VelocityUnits("km/s".to_string()),),
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
                                    epoch: common::EpochType("2004-100T00:00:00".to_string()),
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
                                        units: Some(common::PositionUnits("km".to_string()),),
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
                                        units: Some(common::VelocityUnits("km/s".to_string()),),
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
                id: Some("CCSDS_OEM_VERS".to_string()),
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

        let message = OemType::from_xml_str(xml).unwrap();

        assert_eq!(
            message,
            OemType {
                header: common::OdmHeader {
                    comment_list: vec!["OEM WITH OPTIONAL ACCELERATIONS".to_string()],
                    classification_list: vec![],
                    creation_date: common::EpochType("1996-11-04T17:22:31".to_string()),
                    originator: "NASA/JPL".to_string(),
                    message_id: Some("OEM 201113719185".to_string()),
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
                            start_time: common::EpochType("1996-12-18T12:00:00.331".to_string()),
                            useable_start_time: Some(common::EpochType(
                                "1996-12-18T12:10:00.331".to_string(),
                            )),
                            useable_stop_time: Some(common::EpochType(
                                "1996-12-28T21:23:00.331".to_string(),
                            )),
                            stop_time: common::EpochType("1996-12-28T21:28:00.331".to_string()),
                            interpolation: Some("HERMITE".to_string()),
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
                                    epoch: common::EpochType("1996-12-18T12:00:00.331".to_string()),
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
                                    epoch: common::EpochType("1996-12-18T12:01:00.331".to_string()),
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
                                    epoch: common::EpochType("1996-12-18T12:02:00.331".to_string()),
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
                                    epoch: common::EpochType("1996-12-28T21:28:00.331".to_string()),
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
                                comment_list: vec!["blabla".to_string()],
                                epoch: common::EpochType("1996-12-28T22:28:00.331".to_string()),
                                cov_ref_frame: Some("ITRF1997".to_string()),
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
                id: Some("CCSDS_OEM_VERS".to_string()),
                version: "3.0".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_oem_message_kvn() {
        let kvn = r#"CCSDS_OEM_VERS = 3.0
CREATION_DATE = 1996-11-04T17:22:31
ORIGINATOR = NASA/JPL

META_START
OBJECT_NAME         = MARS GLOBAL SURVEYOR
OBJECT_ID           = 1996-062A
CENTER_NAME         = MARS BARYCENTER
REF_FRAME           = J2000
TIME_SYSTEM         = TAI
START_TIME          = 1996-12-18T12:00:00.331
USEABLE_START_TIME  = 1996-12-18T12:10:00.331
USEABLE_STOP_TIME   = 1996-12-28T21:23:00.331
STOP_TIME           = 1996-12-28T21:28:00.331
INTERPOLATION       = HERMITE
INTERPOLATION_DEGREE = 7
META_STOP


COMMENT This file was produced by M.R. Somebody, MSOO NAV/JPL, 1996NOV 04. It is
COMMENT to be used for DSN scheduling purposes only.


1996-12-18T12:00:00.331 2789.619 -280.045 -1746.755 4.73372 -2.49586 -1.04195
1996-12-18T12:01:00.331 2783.419 -308.143 -1877.071 5.18604 -2.42124 -1.99608
1996-12-18T12:02:00.331 2776.033 -336.859 -2008.682 5.63678 -2.33951 -1.94687
1996-12-28T21:28:00.331 -3881.024 563.959 -682.773 -3.28827 -3.66735 1.63861



META_START
OBJECT_NAME         = MARS GLOBAL SURVEYOR
OBJECT_ID           = 1996-062A
CENTER_NAME         = MARS BARYCENTER
REF_FRAME           = J2000
TIME_SYSTEM         = TAI
START_TIME          = 1996-12-28T21:29:07.267
USEABLE_START_TIME  = 1996-12-28T22:08:02.5
USEABLE_STOP_TIME   = 1996-12-30T01:18:02.5
STOP_TIME           = 1996-12-30T01:28:02.267
INTERPOLATION       = HERMITE
INTERPOLATION_DEGREE = 7
META_STOP


COMMENT This block begins after trajectory correction maneuver TCM-3.
1996-12-28T21:29:07.267 -2432.166 -063.042 1742.754 7.33702 -3.495867 -1.041945
1996-12-28T21:59:02.267 -2445.234 -878.141 1873.073 1.86043 -3.421256 -0.996366
1996-12-28T22:00:02.267 -2458.079 -683.858 2007.684 6.36786 -3.339563 -0.946654
1996-12-30T01:28:02.267 2164.375 1115.811 -688.131 -3.53328 -2.88452 0.88535


COVARIANCE_START
EPOCH = 1996-12-28T21:29:07.267
COV_REF_FRAME = EME2000
3.3313494e-04
4.6189273e-04 6.7824216e-04
-3.0700078e-04 -4.2212341e-04 3.2319319e-04
-3.3493650e-07 -4.6860842e-07 2.4849495e-07 4.2960228e-10
-2.2118325e-07 -2.8641868e-07 1.7980986e-07 2.6088992e-10 1.7675147e-10
-3.0413460e-07 -4.9894969e-07 3.5403109e-07 1.8692631e-10 1.0088625e-10 6.2244443e-10
COVARIANCE_STOP


META_START
COMMENT comment
OBJECT_NAME = MARS GLOBAL SURVEYOR
OBJECT_ID = 1996-062A
CENTER_NAME = MARS BARYCENTER
REF_FRAME = EME2000
TIME_SYSTEM = TAI
START_TIME = 1996-12-28T21:29:07.267
USEABLE_START_TIME = 1996-12-28T22:08:02.5
USEABLE_STOP_TIME = 1996-12-30T01:18:02.5
STOP_TIME = 1996-12-30T01:28:02.267
INTERPOLATION = HERMITE
INTERPOLATION_DEGREE = 7
META_STOP


COMMENT This block begins after trajectory correction maneuver TCM-3.
1996-12-28T21:29:07.267 -2432.166 -063.042 1742.754 7.33702 -3.495867 -1.041945
1996-12-28T21:59:02.267 -2445.234 -878.141 1873.073 1.86043 -3.421256 -0.996366
1996-12-28T22:00:02.267 -2458.079 -683.858 2007.684 6.36786 -3.339563 -0.946654
1996-12-30T01:28:02.267 2164.375 1115.811 -688.131 -3.53328 -2.88452 0.88535
1996-12-30T01:28:02.267 2164.375 1115.811 -688.131 -3.53328 -2.88452 0.88535


COVARIANCE_START
EPOCH = 1996-12-28T21:29:07.267
COV_REF_FRAME = RTN
3.3313494e-04
4.6189273e-04 6.7824216e-04
-3.0700078e-04 -4.2212341e-04 3.2319319e-04
-3.3493650e-07 -4.6860842e-07 2.4849495e-07 4.2960228e-10
-2.2118325e-07 -2.8641868e-07 1.7980986e-07 2.6088992e-10 1.7675147e-10
-3.0413460e-07 -4.9894969e-07 3.5403109e-07 1.8692631e-10 1.0088625e-10 6.2244443e-10
EPOCH = 1996-12-29T21:00:00


COV_REF_FRAME = EME2000
3.4424505e-04
4.5078162e-04 6.8935327e-04
-3.0600067e-04 -4.1101230e-04 3.3420420e-04
-3.2382549e-07 -4.5750731e-07 2.3738384e-07 4.3071339e-10
-2.1007214e-07 -2.7530757e-07 1.6870875e-07 2.5077881e-10 1.8786258e-10
-3.0302350e-07 -4.8783858e-07 3.4302008e-07 1.7581520e-10 1.0077514e-10 6.2244443e-10
COVARIANCE_STOP"#;

        assert_eq!(
            crate::ndm::kvn::KvnDeserializer::from_kvn_str(kvn),
            Ok(OemType {
                id: None,
                version: "3.0".to_string(),
                header: common::OdmHeader {
                    comment_list: vec![],
                    classification_list: vec![],
                    creation_date: common::EpochType("1996-11-04T17:22:31".to_string(),),
                    originator: "NASA/JPL".to_string(),
                    message_id: None,
                },
                body: OemBody {
                    segment_list: vec![
                        OemSegment {
                            metadata: OemMetadata {
                                comment_list: vec![],
                                object_name: "MARS GLOBAL SURVEYOR".to_string(),
                                object_id: "1996-062A".to_string(),
                                center_name: "MARS BARYCENTER".to_string(),
                                ref_frame: "J2000".to_string(),
                                ref_frame_epoch: None,
                                time_system: "TAI".to_string(),
                                start_time: common::EpochType(
                                    "1996-12-18T12:00:00.331".to_string(),
                                ),
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
                                    "This file was produced by M.R. Somebody, MSOO NAV/JPL, 1996NOV 04. It is"
                                    .to_string(),
                                    "to be used for DSN scheduling purposes only."
                                    .to_string(),
                                ],
                                state_vector_list: vec![
                                    common::StateVectorAccType {
                                        epoch: common::EpochType("1996-12-18T12:00:00.331".to_string(),),
                                        x: common::PositionType {
                                            base: 2789.619,
                                            units: None,
                                        },
                                        y: common::PositionType {
                                            base: -280.045,
                                            units: None,
                                        },
                                        z: common::PositionType {
                                            base: -1746.755,
                                            units: None,
                                        },
                                        x_dot: common::VelocityType {
                                            base: 4.73372,
                                            units: None,
                                        },
                                        y_dot: common::VelocityType {
                                            base: -2.49586,
                                            units: None,
                                        },
                                        z_dot: common::VelocityType {
                                            base: -1.04195,
                                            units: None,
                                        },
                                        x_ddot: None,
                                        y_ddot: None,
                                        z_ddot: None,
                                    },
                                    common::StateVectorAccType {
                                        epoch: common::EpochType("1996-12-18T12:01:00.331".to_string(),),
                                        x: common::PositionType {
                                            base: 2783.419,
                                            units: None,
                                        },
                                        y: common::PositionType {
                                            base: -308.143,
                                            units: None,
                                        },
                                        z: common::PositionType {
                                            base: -1877.071,
                                            units: None,
                                        },
                                        x_dot: common::VelocityType {
                                            base: 5.18604,
                                            units: None,
                                        },
                                        y_dot: common::VelocityType {
                                            base: -2.42124,
                                            units: None,
                                        },
                                        z_dot: common::VelocityType {
                                            base: -1.99608,
                                            units: None,
                                        },
                                        x_ddot: None,
                                        y_ddot: None,
                                        z_ddot: None,
                                    },
                                    common::StateVectorAccType {
                                        epoch: common::EpochType("1996-12-18T12:02:00.331".to_string(),),
                                        x: common::PositionType {
                                            base: 2776.033,
                                            units: None,
                                        },
                                        y: common::PositionType {
                                            base: -336.859,
                                            units: None,
                                        },
                                        z: common::PositionType {
                                            base: -2008.682,
                                            units: None,
                                        },
                                        x_dot: common::VelocityType {
                                            base: 5.63678,
                                            units: None,
                                        },
                                        y_dot: common::VelocityType {
                                            base: -2.33951,
                                            units: None,
                                        },
                                        z_dot: common::VelocityType {
                                            base: -1.94687,
                                            units: None,
                                        },
                                        x_ddot: None,
                                        y_ddot: None,
                                        z_ddot: None,
                                    },
                                    common::StateVectorAccType {
                                        epoch: common::EpochType("1996-12-28T21:28:00.331".to_string(),),
                                        x: common::PositionType {
                                            base: -3881.024,
                                            units: None,
                                        },
                                        y: common::PositionType {
                                            base: 563.959,
                                            units: None,
                                        },
                                        z: common::PositionType {
                                            base: -682.773,
                                            units: None,
                                        },
                                        x_dot: common::VelocityType {
                                            base: -3.28827,
                                            units: None,
                                        },
                                        y_dot: common::VelocityType {
                                            base: -3.66735,
                                            units: None,
                                        },
                                        z_dot: common::VelocityType {
                                            base: 1.63861,
                                            units: None,
                                        },
                                        x_ddot: None,
                                        y_ddot: None,
                                        z_ddot: None,
                                    },
                                ],
                                covariance_matrix_list: vec![],
                            }
                        },
                        OemSegment {
                            metadata: OemMetadata {
                                comment_list: vec![],
                                object_name: "MARS GLOBAL SURVEYOR".to_string(),
                                object_id: "1996-062A".to_string(),
                                center_name: "MARS BARYCENTER".to_string(),
                                ref_frame: "J2000".to_string(),
                                ref_frame_epoch: None,
                                time_system: "TAI".to_string(),
                                start_time: common::EpochType(
                                    "1996-12-28T21:29:07.267".to_string(),
                                ),
                                useable_start_time: Some(common::EpochType(
                                        "1996-12-28T22:08:02.5".to_string(),
                                ),),
                                useable_stop_time: Some(common::EpochType(
                                        "1996-12-30T01:18:02.5".to_string(),
                                ),),
                                stop_time: common::EpochType("1996-12-30T01:28:02.267".to_string(),),
                                interpolation: Some("HERMITE".to_string(),),
                                interpolation_degree: Some(7,),
                            },
                            data: OemData {
                                comment_list: vec![
                                    "This block begins after trajectory correction maneuver TCM-3."
                                    .to_string(),
                                ],
                                state_vector_list: vec![
                                    common::StateVectorAccType {
                                        epoch: common::EpochType(
                                            "1996-12-28T21:29:07.267".to_string(),
                                        ),
                                        x: common::PositionType {
                                            base: -2432.166,
                                            units: None,
                                        },
                                        y: common::PositionType {
                                            base: -63.042,
                                            units: None,
                                        },
                                        z: common::PositionType {
                                            base: 1742.754,
                                            units: None,
                                        },
                                        x_dot: common::VelocityType {
                                            base: 7.33702,
                                            units: None,
                                        },
                                        y_dot: common::VelocityType {
                                            base: -3.495867,
                                            units: None,
                                        },
                                        z_dot: common::VelocityType {
                                            base: -1.041945,
                                            units: None,
                                        },
                                        x_ddot: None,
                                        y_ddot: None,
                                        z_ddot: None,
                                    },
                                    common::StateVectorAccType {
                                        epoch: common::EpochType(
                                            "1996-12-28T21:59:02.267".to_string(),
                                        ),
                                        x: common::PositionType {
                                            base: -2445.234,
                                            units: None,
                                        },
                                        y: common::PositionType {
                                            base: -878.141,
                                            units: None,
                                        },
                                        z: common::PositionType {
                                            base: 1873.073,
                                            units: None,
                                        },
                                        x_dot: common::VelocityType {
                                            base: 1.86043,
                                            units: None,
                                        },
                                        y_dot: common::VelocityType {
                                            base: -3.421256,
                                            units: None,
                                        },
                                        z_dot: common::VelocityType {
                                            base: -0.996366,
                                            units: None,
                                        },
                                        x_ddot: None,
                                        y_ddot: None,
                                        z_ddot: None,
                                    },
                                    common::StateVectorAccType {
                                        epoch: common::EpochType(
                                            "1996-12-28T22:00:02.267".to_string(),
                                        ),
                                        x: common::PositionType {
                                            base: -2458.079,
                                            units: None,
                                        },
                                        y: common::PositionType {
                                            base: -683.858,
                                            units: None,
                                        },
                                        z: common::PositionType {
                                            base: 2007.684,
                                            units: None,
                                        },
                                        x_dot: common::VelocityType {
                                            base: 6.36786,
                                            units: None,
                                        },
                                        y_dot: common::VelocityType {
                                            base: -3.339563,
                                            units: None,
                                        },
                                        z_dot: common::VelocityType {
                                            base: -0.946654,
                                            units: None,
                                        },
                                        x_ddot: None,
                                        y_ddot: None,
                                        z_ddot: None,
                                    },
                                    common::StateVectorAccType {
                                        epoch: common::EpochType(
                                            "1996-12-30T01:28:02.267".to_string(),
                                        ),
                                        x: common::PositionType {
                                            base: 2164.375,
                                            units: None,
                                        },
                                        y: common::PositionType {
                                            base: 1115.811,
                                            units: None,
                                        },
                                        z: common::PositionType {
                                            base: -688.131,
                                            units: None,
                                        },
                                        x_dot: common::VelocityType {
                                            base: -3.53328,
                                            units: None,
                                        },
                                        y_dot: common::VelocityType {
                                            base: -2.88452,
                                            units: None,
                                        },
                                        z_dot: common::VelocityType {
                                            base: 0.88535,
                                            units: None,
                                        },
                                        x_ddot: None,
                                        y_ddot: None,
                                        z_ddot: None,
                                    },
                                ],
                                covariance_matrix_list: vec![common::OemCovarianceMatrixType {
                                        comment_list: vec![],
                                        epoch: common::EpochType("1996-12-28T21:29:07.267".to_string(),),
                                        cov_ref_frame: Some("EME2000".to_string(),),
                                        cx_x: common::PositionCovarianceType {
                                            base: 0.00033313494,
                                            units: None,
                                        },
                                        cy_x: common::PositionCovarianceType {
                                            base: 0.00046189273,
                                            units: None,
                                        },
                                        cy_y: common::PositionCovarianceType {
                                            base: 0.00067824216,
                                            units: None,
                                        },
                                        cz_x: common::PositionCovarianceType {
                                            base: -0.00030700078,
                                            units: None,
                                        },
                                        cz_y: common::PositionCovarianceType {
                                            base: -0.00042212341,
                                            units: None,
                                        },
                                        cz_z: common::PositionCovarianceType {
                                            base: 0.00032319319,
                                            units: None,
                                        },
                                        cx_dot_x: common::PositionVelocityCovarianceType {
                                            base: -3.349365e-7,
                                            units: None,
                                        },
                                        cx_dot_y: common::PositionVelocityCovarianceType {
                                            base: -4.6860842e-7,
                                            units: None,
                                        },
                                        cx_dot_z: common::PositionVelocityCovarianceType {
                                            base: 2.4849495e-7,
                                            units: None,
                                        },
                                        cx_dot_x_dot: common::VelocityCovarianceType {
                                            base: 4.2960228e-10,
                                            units: None,
                                        },
                                        cy_dot_x: common::PositionVelocityCovarianceType {
                                            base: -2.2118325e-7,
                                            units: None,
                                        },
                                        cy_dot_y: common::PositionVelocityCovarianceType {
                                            base: -2.8641868e-7,
                                            units: None,
                                        },
                                        cy_dot_z: common::PositionVelocityCovarianceType {
                                            base: 1.7980986e-7,
                                            units: None,
                                        },
                                        cy_dot_x_dot: common::VelocityCovarianceType {
                                            base: 2.6088992e-10,
                                            units: None,
                                        },
                                        cy_dot_y_dot: common::VelocityCovarianceType {
                                            base: 1.7675147e-10,
                                            units: None,
                                        },
                                        cz_dot_x: common::PositionVelocityCovarianceType {
                                            base: -3.041346e-7,
                                            units: None,
                                        },
                                        cz_dot_y: common::PositionVelocityCovarianceType {
                                            base: -4.9894969e-7,
                                            units: None,
                                        },
                                        cz_dot_z: common::PositionVelocityCovarianceType {
                                            base: 3.5403109e-7,
                                            units: None,
                                        },
                                        cz_dot_x_dot: common::VelocityCovarianceType {
                                            base: 1.8692631e-10,
                                            units: None,
                                        },
                                        cz_dot_y_dot: common::VelocityCovarianceType {
                                            base: 1.0088625e-10,
                                            units: None,
                                        },
                                        cz_dot_z_dot: common::VelocityCovarianceType {
                                            base: 6.2244443e-10,
                                            units: None,
                                        },
                                    },],
                            },
                        },
                        OemSegment {
                            metadata: OemMetadata {
                                comment_list: vec!["comment".to_string(),],
                                object_name: "MARS GLOBAL SURVEYOR".to_string(),
                                object_id: "1996-062A".to_string(),
                                center_name: "MARS BARYCENTER".to_string(),
                                ref_frame: "EME2000".to_string(),
                                ref_frame_epoch: None,
                                time_system: "TAI".to_string(),
                                start_time: common::EpochType(
                                    "1996-12-28T21:29:07.267".to_string(),
                                ),
                                useable_start_time: Some(common::EpochType(
                                        "1996-12-28T22:08:02.5".to_string(),
                                ),),
                                useable_stop_time: Some(common::EpochType(
                                        "1996-12-30T01:18:02.5".to_string(),
                                ),),
                                stop_time: common::EpochType("1996-12-30T01:28:02.267".to_string(),),
                                interpolation: Some("HERMITE".to_string(),),
                                interpolation_degree: Some(7,),
                            },
                            data: OemData {
                                comment_list: vec![
                                    "This block begins after trajectory correction maneuver TCM-3."
                                    .to_string(),
                                ],
                                state_vector_list: vec![
                                    common::StateVectorAccType {
                                        epoch: common::EpochType(
                                            "1996-12-28T21:29:07.267".to_string(),
                                        ),
                                        x: common::PositionType {
                                            base: -2432.166,
                                            units: None,
                                        },
                                        y: common::PositionType {
                                            base: -63.042,
                                            units: None,
                                        },
                                        z: common::PositionType {
                                            base: 1742.754,
                                            units: None,
                                        },
                                        x_dot: common::VelocityType {
                                            base: 7.33702,
                                            units: None,
                                        },
                                        y_dot: common::VelocityType {
                                            base: -3.495867,
                                            units: None,
                                        },
                                        z_dot: common::VelocityType {
                                            base: -1.041945,
                                            units: None,
                                        },
                                        x_ddot: None,
                                        y_ddot: None,
                                        z_ddot: None,
                                    },
                                    common::StateVectorAccType {
                                        epoch: common::EpochType(
                                            "1996-12-28T21:59:02.267".to_string(),
                                        ),
                                        x: common::PositionType {
                                            base: -2445.234,
                                            units: None,
                                        },
                                        y: common::PositionType {
                                            base: -878.141,
                                            units: None,
                                        },
                                        z: common::PositionType {
                                            base: 1873.073,
                                            units: None,
                                        },
                                        x_dot: common::VelocityType {
                                            base: 1.86043,
                                            units: None,
                                        },
                                        y_dot: common::VelocityType {
                                            base: -3.421256,
                                            units: None,
                                        },
                                        z_dot: common::VelocityType {
                                            base: -0.996366,
                                            units: None,
                                        },
                                        x_ddot: None,
                                        y_ddot: None,
                                        z_ddot: None,
                                    },
                                    common::StateVectorAccType {
                                        epoch: common::EpochType(
                                            "1996-12-28T22:00:02.267".to_string(),
                                        ),
                                        x: common::PositionType {
                                            base: -2458.079,
                                            units: None,
                                        },
                                        y: common::PositionType {
                                            base: -683.858,
                                            units: None,
                                        },
                                        z: common::PositionType {
                                            base: 2007.684,
                                            units: None,
                                        },
                                        x_dot: common::VelocityType {
                                            base: 6.36786,
                                            units: None,
                                        },
                                        y_dot: common::VelocityType {
                                            base: -3.339563,
                                            units: None,
                                        },
                                        z_dot: common::VelocityType {
                                            base: -0.946654,
                                            units: None,
                                        },
                                        x_ddot: None,
                                        y_ddot: None,
                                        z_ddot: None,
                                    },
                                    common::StateVectorAccType {
                                        epoch: common::EpochType(
                                            "1996-12-30T01:28:02.267".to_string(),
                                        ),
                                        x: common::PositionType {
                                            base: 2164.375,
                                            units: None,
                                        },
                                        y: common::PositionType {
                                            base: 1115.811,
                                            units: None,
                                        },
                                        z: common::PositionType {
                                            base: -688.131,
                                            units: None,
                                        },
                                        x_dot: common::VelocityType {
                                            base: -3.53328,
                                            units: None,
                                        },
                                        y_dot: common::VelocityType {
                                            base: -2.88452,
                                            units: None,
                                        },
                                        z_dot: common::VelocityType {
                                            base: 0.88535,
                                            units: None,
                                        },
                                        x_ddot: None,
                                        y_ddot: None,
                                        z_ddot: None,
                                    },
                                    common::StateVectorAccType {
                                        epoch: common::EpochType(
                                            "1996-12-30T01:28:02.267".to_string(),
                                        ),
                                        x: common::PositionType {
                                            base: 2164.375,
                                            units: None,
                                        },
                                        y: common::PositionType {
                                            base: 1115.811,
                                            units: None,
                                        },
                                        z: common::PositionType {
                                            base: -688.131,
                                            units: None,
                                        },
                                        x_dot: common::VelocityType {
                                            base: -3.53328,
                                            units: None,
                                        },
                                        y_dot: common::VelocityType {
                                            base: -2.88452,
                                            units: None,
                                        },
                                        z_dot: common::VelocityType {
                                            base: 0.88535,
                                            units: None,
                                        },
                                        x_ddot: None,
                                        y_ddot: None,
                                        z_ddot: None,
                                    },
                                ],
                                covariance_matrix_list: vec![
                                    common::OemCovarianceMatrixType {
                                        comment_list: vec![],
                                        epoch: common::EpochType(
                                            "1996-12-28T21:29:07.267".to_string(),
                                        ),
                                        cov_ref_frame: Some("RTN".to_string(),),
                                        cx_x: common::PositionCovarianceType {
                                            base: 0.00033313494,
                                            units: None,
                                        },
                                        cy_x: common::PositionCovarianceType {
                                            base: 0.00046189273,
                                            units: None,
                                        },
                                        cy_y: common::PositionCovarianceType {
                                            base: 0.00067824216,
                                            units: None,
                                        },
                                        cz_x: common::PositionCovarianceType {
                                            base: -0.00030700078,
                                            units: None,
                                        },
                                        cz_y: common::PositionCovarianceType {
                                            base: -0.00042212341,
                                            units: None,
                                        },
                                        cz_z: common::PositionCovarianceType {
                                            base: 0.00032319319,
                                            units: None,
                                        },
                                        cx_dot_x: common::PositionVelocityCovarianceType {
                                            base: -3.349365e-7,
                                            units: None,
                                        },
                                        cx_dot_y: common::PositionVelocityCovarianceType {
                                            base: -4.6860842e-7,
                                            units: None,
                                        },
                                        cx_dot_z: common::PositionVelocityCovarianceType {
                                            base: 2.4849495e-7,
                                            units: None,
                                        },
                                        cx_dot_x_dot: common::VelocityCovarianceType {
                                            base: 4.2960228e-10,
                                            units: None,
                                        },
                                        cy_dot_x: common::PositionVelocityCovarianceType {
                                            base: -2.2118325e-7,
                                            units: None,
                                        },
                                        cy_dot_y: common::PositionVelocityCovarianceType {
                                            base: -2.8641868e-7,
                                            units: None,
                                        },
                                        cy_dot_z: common::PositionVelocityCovarianceType {
                                            base: 1.7980986e-7,
                                            units: None,
                                        },
                                        cy_dot_x_dot: common::VelocityCovarianceType {
                                            base: 2.6088992e-10,
                                            units: None,
                                        },
                                        cy_dot_y_dot: common::VelocityCovarianceType {
                                            base: 1.7675147e-10,
                                            units: None,
                                        },
                                        cz_dot_x: common::PositionVelocityCovarianceType {
                                            base: -3.041346e-7,
                                            units: None,
                                        },
                                        cz_dot_y: common::PositionVelocityCovarianceType {
                                            base: -4.9894969e-7,
                                            units: None,
                                        },
                                        cz_dot_z: common::PositionVelocityCovarianceType {
                                            base: 3.5403109e-7,
                                            units: None,
                                        },
                                        cz_dot_x_dot: common::VelocityCovarianceType {
                                            base: 1.8692631e-10,
                                            units: None,
                                        },
                                        cz_dot_y_dot: common::VelocityCovarianceType {
                                            base: 1.0088625e-10,
                                            units: None,
                                        },
                                        cz_dot_z_dot: common::VelocityCovarianceType {
                                            base: 6.2244443e-10,
                                            units: None,
                                        },
                                    },
                                    common::OemCovarianceMatrixType {
                                        comment_list: vec![],
                                        epoch: common::EpochType("1996-12-29T21:00:00".to_string(),),
                                        cov_ref_frame: Some("EME2000".to_string(),),
                                        cx_x: common::PositionCovarianceType {
                                            base: 0.00034424505,
                                            units: None,
                                        },
                                        cy_x: common::PositionCovarianceType {
                                            base: 0.00045078162,
                                            units: None,
                                        },
                                        cy_y: common::PositionCovarianceType {
                                            base: 0.00068935327,
                                            units: None,
                                        },
                                        cz_x: common::PositionCovarianceType {
                                            base: -0.00030600067,
                                            units: None,
                                        },
                                        cz_y: common::PositionCovarianceType {
                                            base: -0.0004110123,
                                            units: None,
                                        },
                                        cz_z: common::PositionCovarianceType {
                                            base: 0.0003342042,
                                            units: None,
                                        },
                                        cx_dot_x: common::PositionVelocityCovarianceType {
                                            base: -3.2382549e-7,
                                            units: None,
                                        },
                                        cx_dot_y: common::PositionVelocityCovarianceType {
                                            base: -4.5750731e-7,
                                            units: None,
                                        },
                                        cx_dot_z: common::PositionVelocityCovarianceType {
                                            base: 2.3738384e-7,
                                            units: None,
                                        },
                                        cx_dot_x_dot: common::VelocityCovarianceType {
                                            base: 4.3071339e-10,
                                            units: None,
                                        },
                                        cy_dot_x: common::PositionVelocityCovarianceType {
                                            base: -2.1007214e-7,
                                            units: None,
                                        },
                                        cy_dot_y: common::PositionVelocityCovarianceType {
                                            base: -2.7530757e-7,
                                            units: None,
                                        },
                                        cy_dot_z: common::PositionVelocityCovarianceType {
                                            base: 1.6870875e-7,
                                            units: None,
                                        },
                                        cy_dot_x_dot: common::VelocityCovarianceType {
                                            base: 2.5077881e-10,
                                            units: None,
                                        },
                                        cy_dot_y_dot: common::VelocityCovarianceType {
                                            base: 1.8786258e-10,
                                            units: None,
                                        },
                                        cz_dot_x: common::PositionVelocityCovarianceType {
                                            base: -3.030235e-7,
                                            units: None,
                                        },
                                        cz_dot_y: common::PositionVelocityCovarianceType {
                                            base: -4.8783858e-7,
                                            units: None,
                                        },
                                        cz_dot_z: common::PositionVelocityCovarianceType {
                                            base: 3.4302008e-7,
                                            units: None,
                                        },
                                        cz_dot_x_dot: common::VelocityCovarianceType {
                                            base: 1.758152e-10,
                                            units: None,
                                        },
                                        cz_dot_y_dot: common::VelocityCovarianceType {
                                            base: 1.0077514e-10,
                                            units: None,
                                        },
                                        cz_dot_z_dot: common::VelocityCovarianceType {
                                            base: 6.2244443e-10,
                                            units: None,
                                        },
                                    },
                                ],
                            },
                        },
                    ],
                },
            })
        );
    }
}
