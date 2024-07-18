/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Deserializers for XML and KVN CCSDS Orbit Mean Elements Message
//!
//! To deserialize an XML message:
//!
//! ```
//! # let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
//! # <omm id="CCSDS_OMM_VERS" version="2.0">
//! # <header>
//! #     <CREATION_DATE>2021-03-24T23:00:00.000</CREATION_DATE>
//! #     <ORIGINATOR>CelesTrak</ORIGINATOR>
//! # </header>
//! # <body>
//! # <segment>
//! #     <metadata>
//! #         <OBJECT_NAME>STARLETTE</OBJECT_NAME>
//! #         <OBJECT_ID>1975-010A</OBJECT_ID>
//! #         <CENTER_NAME>EARTH</CENTER_NAME>
//! #         <REF_FRAME>TEME</REF_FRAME>
//! #         <TIME_SYSTEM>UTC</TIME_SYSTEM>
//! #         <MEAN_ELEMENT_THEORY>SGP4</MEAN_ELEMENT_THEORY>
//! #     </metadata>
//! #     <data>
//! #         <meanElements>
//! #             <EPOCH>2008-09-20T12:25:40.104192</EPOCH>
//! #             <MEAN_MOTION units="rev/day">15.72125391</MEAN_MOTION>
//! #             <ECCENTRICITY>0.0006703</ECCENTRICITY>
//! #             <INCLINATION units="deg">51.6416</INCLINATION>
//! #             <RA_OF_ASC_NODE units="deg">247.4627</RA_OF_ASC_NODE>
//! #             <ARG_OF_PERICENTER units="deg">130.5360</ARG_OF_PERICENTER>
//! #             <MEAN_ANOMALY units="deg">325.0288</MEAN_ANOMALY>
//! #             <GM units="km**3/s**2">398600.8</GM>
//! #         </meanElements>
//! #         <tleParameters>
//! #             <EPHEMERIS_TYPE>0</EPHEMERIS_TYPE>
//! #             <CLASSIFICATION_TYPE>U</CLASSIFICATION_TYPE>
//! #             <NORAD_CAT_ID>7646</NORAD_CAT_ID>
//! #             <ELEMENT_SET_NO>999</ELEMENT_SET_NO>
//! #             <REV_AT_EPOCH>32997</REV_AT_EPOCH>
//! #             <BSTAR>-.47102E-5</BSTAR>
//! #             <MEAN_MOTION_DOT>-.147E-5</MEAN_MOTION_DOT>
//! #             <MEAN_MOTION_DDOT>0</MEAN_MOTION_DDOT>
//! #         </tleParameters>
//! #         <userDefinedParameters>
//! #             <USER_DEFINED parameter="FOO">foo enters</USER_DEFINED>
//! #             <USER_DEFINED parameter="BAR">a bar</USER_DEFINED>
//! #         </userDefinedParameters>
//! #     </data>
//! # </segment>
//! # </body>
//! # </omm>"#;
//! #
//! # use lox_io::ndm::omm::OmmType;
//! use lox_io::ndm::xml::FromXmlStr;
//!
//! let message = OmmType::from_xml_str(xml).unwrap();
//! ```
//!
//! To deserialize a KVN message:
//! ```
//! # let kvn = r#"CCSDS_OMM_VERS = 3.0
//! # COMMENT this is a comment
//! # COMMENT here is another one
//! # CREATION_DATE = 2007-06-05T16:00:00
//! # ORIGINATOR = NOAA/USA
//! # COMMENT this comment doesn't say much
//! # OBJECT_NAME = GOES 9
//! # OBJECT_ID = 1995-025A
//! # CENTER_NAME = EARTH
//! # REF_FRAME = TOD
//! # REF_FRAME_EPOCH = 2000-01-03T10:34:00
//! # TIME_SYSTEM = MRT
//! # MEAN_ELEMENT_THEORY = SOME THEORY
//! # COMMENT the following data is what we're looking for
//! # EPOCH = 2000-01-05T10:00:00
//! # SEMI_MAJOR_AXIS = 6800
//! # ECCENTRICITY = 0.0005013
//! # INCLINATION = 3.0539
//! # RA_OF_ASC_NODE = 81.7939
//! # ARG_OF_PERICENTER = 249.2363
//! # MEAN_ANOMALY = 150.1602
//! # COMMENT spacecraft data
//! # MASS = 300
//! # SOLAR_RAD_AREA = 5
//! # SOLAR_RAD_COEFF = 0.001
//! # DRAG_AREA = 4
//! # DRAG_COEFF = 0.002
//! # COMMENT Covariance matrix
//! # COV_REF_FRAME = TNW
//! # CX_X = 3.331349476038534e-04
//! # CY_X = 4.618927349220216e-04
//! # CY_Y = 6.782421679971363e-04
//! # CZ_X = -3.070007847730449e-04
//! # CZ_Y = -4.221234189514228e-04
//! # CZ_Z = 3.231931992380369e-04
//! # CX_DOT_X = -3.349365033922630e-07
//! # CX_DOT_Y = -4.686084221046758e-07
//! # CX_DOT_Z = 2.484949578400095e-07
//! # CX_DOT_X_DOT = 4.296022805587290e-10
//! # CY_DOT_X = -2.211832501084875e-07
//! # CY_DOT_Y = -2.864186892102733e-07
//! # CY_DOT_Z = 1.798098699846038e-07
//! # CY_DOT_X_DOT = 2.608899201686016e-10
//! # CY_DOT_Y_DOT = 1.767514756338532e-10
//! # CZ_DOT_X = -3.041346050686871e-07
//! # CZ_DOT_Y = -4.989496988610662e-07
//! # CZ_DOT_Z = 3.540310904497689e-07
//! # CZ_DOT_X_DOT = 1.869263192954590e-10
//! # CZ_DOT_Y_DOT = 1.008862586240695e-10
//! # CZ_DOT_Z_DOT = 6.224444338635500e-10"#;
//! #
//! # use lox_io::ndm::omm::OmmType;
//! use lox_io::ndm::kvn::KvnDeserializer;
//!
//! let message: OmmType = KvnDeserializer::from_kvn_str(&kvn).unwrap();
//! ```

// This file is partially generated with xml-schema-derive from the XSD schema
// published by CCSDS. Adaptations have been made to simplify the types or
// allow to simplify the implementation of the KVN parser.

use serde;

use super::common;

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
pub struct BStarUnits(#[serde(rename = "$text")] pub String);

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
pub struct BTermUnits(#[serde(rename = "$text")] pub String);

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
pub struct AgomUnits(#[serde(rename = "$text")] pub String);

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
pub struct ElementSetNoType(#[serde(rename = "$text")] pub String);

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
pub struct RevUnits(#[serde(rename = "$text")] pub String);

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
pub struct DRevUnits(#[serde(rename = "$text")] pub String);

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
pub struct DdRevUnits(#[serde(rename = "$text")] pub String);

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
pub struct SpacewarnType(#[serde(rename = "$text")] pub String);

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
pub struct OmmType {
    #[serde(rename = "@id")]
    // Marked as option for the KVN deserializer
    pub id: Option<String>,
    #[serde(rename = "@version")]
    pub version: String,
    #[serde(rename = "header")]
    pub header: common::OdmHeader,
    #[serde(rename = "body")]
    pub body: OmmBody,
}

impl crate::ndm::xml::FromXmlStr<'_> for OmmType {}

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
pub struct OmmBody {
    #[serde(rename = "segment")]
    pub segment: OmmSegment,
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
pub struct OmmSegment {
    #[serde(rename = "metadata")]
    pub metadata: OmmMetadata,
    #[serde(rename = "data")]
    pub data: OmmData,
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
pub struct MeanElementsType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "EPOCH")]
    pub epoch: common::EpochType,
    #[serde(rename = "SEMI_MAJOR_AXIS")]
    pub semi_major_axis: Option<common::DistanceType>,
    #[serde(rename = "MEAN_MOTION")]
    pub mean_motion: Option<RevType>,
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
    pub bstar: Option<BStarType>,
    #[serde(rename = "BTERM")]
    pub bterm: Option<BTermType>,
    #[serde(rename = "MEAN_MOTION_DOT")]
    pub mean_motion_dot: DRevType,
    #[serde(rename = "MEAN_MOTION_DDOT")]
    pub mean_motion_ddot: Option<DRevType>,
    #[serde(rename = "AGOM")]
    pub agom: Option<AgomType>,
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
pub struct BStarType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<BStarUnits>,
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
pub struct BTermType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<BTermUnits>,
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
pub struct AgomType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<AgomUnits>,
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
pub struct RevType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<RevUnits>,
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
pub struct DRevType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<DRevUnits>,
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
pub struct DdRevType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<DdRevUnits>,
}

#[cfg(test)]
mod test {
    use crate::ndm::xml::FromXmlStr;

    use super::*;

    #[test]
    fn test_parse_omm_message_xml_1() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<omm xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="http://cwe.ccsds.org/moims/docs/MOIMS-NAV/Schemas/ndmxml-1.0-master.xsd"
        id="CCSDS_OMM_VERS" version="2.0">

    <header>
        <COMMENT>THIS EXAMPLE CONFORMS TO FIGURE 4-2 IN 502.0-B-2</COMMENT>
        <CREATION_DATE>2007-065T16:00:00</CREATION_DATE>
        <ORIGINATOR>NOAA/USA</ORIGINATOR>
    </header>
    <body>
        <segment>
            <metadata>
                <OBJECT_NAME>GOES-9</OBJECT_NAME>
                <OBJECT_ID>1995-025A</OBJECT_ID>
                <CENTER_NAME>EARTH</CENTER_NAME>
                <REF_FRAME>TEME</REF_FRAME>
                <TIME_SYSTEM>UTC</TIME_SYSTEM>
                <MEAN_ELEMENT_THEORY>TLE</MEAN_ELEMENT_THEORY>
            </metadata>
            <data>
                <COMMENT>USAF SGP4 IS THE ONLY PROPAGATOR THAT SHOULD BE USED FOR THIS DATA</COMMENT>
                <meanElements>
                    <EPOCH>2007-064T10:34:41.4264</EPOCH>
                    <MEAN_MOTION>1.00273272</MEAN_MOTION>
                    <ECCENTRICITY>0.0005013</ECCENTRICITY>
                    <INCLINATION>3.0539</INCLINATION>
                    <RA_OF_ASC_NODE>81.7939</RA_OF_ASC_NODE>
                    <ARG_OF_PERICENTER>249.2363</ARG_OF_PERICENTER>
                    <MEAN_ANOMALY>150.1602</MEAN_ANOMALY>
                    <GM>398600.8</GM>
                </meanElements>
                <tleParameters>
                    <NORAD_CAT_ID>23581</NORAD_CAT_ID>
                    <ELEMENT_SET_NO>0925</ELEMENT_SET_NO>
                    <REV_AT_EPOCH>4316</REV_AT_EPOCH>
                    <BSTAR>0.0001</BSTAR>
                    <MEAN_MOTION_DOT>-0.00000113</MEAN_MOTION_DOT>
                    <MEAN_MOTION_DDOT>0.0</MEAN_MOTION_DDOT>
                </tleParameters>
                <userDefinedParameters>
                    <USER_DEFINED parameter="ABC0">xyz</USER_DEFINED>
                    <USER_DEFINED parameter="ABC1">9</USER_DEFINED>
                    <USER_DEFINED parameter="ABC2">xyz</USER_DEFINED>
                    <USER_DEFINED parameter="ABC3">9</USER_DEFINED>
                    <USER_DEFINED parameter="ABC4">xyz</USER_DEFINED>
                    <USER_DEFINED parameter="ABC5">9</USER_DEFINED>
                    <USER_DEFINED parameter="ABC6">xyz</USER_DEFINED>
                    <USER_DEFINED parameter="ABC7">9</USER_DEFINED>
                    <USER_DEFINED parameter="ABC8">xyz</USER_DEFINED>
                    <USER_DEFINED parameter="ABC9">9</USER_DEFINED>
                </userDefinedParameters>
            </data>
        </segment>
    </body>
</omm>"#;

        let message = OmmType::from_xml_str(xml).unwrap();

        assert_eq!(message,
            OmmType {
                header: common::OdmHeader {
                    comment_list: vec![
                        "THIS EXAMPLE CONFORMS TO FIGURE 4-2 IN 502.0-B-2".to_string(),
                    ],
                    classification_list: vec![],
                    creation_date: common::EpochType(
                        "2007-065T16:00:00".to_string(),
                    ),
                    originator: "NOAA/USA".to_string(),
                    message_id: None,
                },
                body: OmmBody {
                    segment: OmmSegment {
                        metadata: OmmMetadata {
                            comment_list: vec![],
                            object_name: "GOES-9".to_string(),
                            object_id: "1995-025A".to_string(),
                            center_name: "EARTH".to_string(),
                            ref_frame: "TEME".to_string(),
                            ref_frame_epoch: None,
                            time_system: "UTC".to_string(),
                            mean_element_theory: "TLE".to_string(),
                        },
                        data: OmmData {
                            comment_list: vec![
                                "USAF SGP4 IS THE ONLY PROPAGATOR THAT SHOULD BE USED FOR THIS DATA".to_string(),
                            ],
                            mean_elements: MeanElementsType {
                                comment_list: vec![],
                                epoch: common::EpochType(
                                    "2007-064T10:34:41.4264".to_string(),
                                ),
                                semi_major_axis: None,
                                mean_motion: Some(
                                    RevType {
                                        base: 1.00273272,
                                        units: None,
                                    },
                                ),
                                eccentricity: common::NonNegativeDouble(
                                    0.0005013,
                                ),
                                inclination: common::InclinationType {
                                    base: 3.0539,
                                    units: None,
                                },
                                ra_of_asc_node: common::AngleType {
                                    base: 81.7939,
                                    units: None,
                                },
                                arg_of_pericenter: common::AngleType {
                                    base: 249.2363,
                                    units: None,
                                },
                                mean_anomaly: common::AngleType {
                                    base: 150.1602,
                                    units: None,
                                },
                                gm: Some(
                                    common::GmType {
                                        base: common::PositiveDouble(
                                            398600.8,
                                        ),
                                        units: None,
                                    },
                                ),
                            },
                            spacecraft_parameters: None,
                            tle_parameters: Some(
                                TleParametersType {
                                    comment_list: vec![],
                                    ephemeris_type: None,
                                    classification_type: None,
                                    norad_cat_id: Some(
                                        23581,
                                    ),
                                    element_set_no: Some(
                                        ElementSetNoType(
                                            "0925".to_string(),
                                        ),
                                    ),
                                    rev_at_epoch: Some(
                                        4316,
                                    ),
                                    bstar: Some(
                                        BStarType {
                                            base: 0.0001,
                                            units: None,
                                        },
                                    ),
                                    bterm: None,
                                    mean_motion_dot: DRevType {
                                        base: -1.13e-6,
                                        units: None,
                                    },
                                    mean_motion_ddot: Some(
                                        DRevType {
                                            base: 0.0,
                                            units: None,
                                        },
                                    ),
                                    agom: None,
                                },
                            ),
                            covariance_matrix: None,
                            user_defined_parameters: Some(
                                common::UserDefinedType {
                                    comment_list: vec![],
                                    user_defined_list: vec![
                                        common::UserDefinedParameterType {
                                            base: "xyz".to_string(),
                                            parameter: "ABC0".to_string(),
                                        },
                                        common::UserDefinedParameterType {
                                            base: "9".to_string(),
                                            parameter: "ABC1".to_string(),
                                        },
                                        common::UserDefinedParameterType {
                                            base: "xyz".to_string(),
                                            parameter: "ABC2".to_string(),
                                        },
                                        common::UserDefinedParameterType {
                                            base: "9".to_string(),
                                            parameter: "ABC3".to_string(),
                                        },
                                        common::UserDefinedParameterType {
                                            base: "xyz".to_string(),
                                            parameter: "ABC4".to_string(),
                                        },
                                        common::UserDefinedParameterType {
                                            base: "9".to_string(),
                                            parameter: "ABC5".to_string(),
                                        },
                                        common::UserDefinedParameterType {
                                            base: "xyz".to_string(),
                                            parameter: "ABC6".to_string(),
                                        },
                                        common::UserDefinedParameterType {
                                            base: "9".to_string(),
                                            parameter: "ABC7".to_string(),
                                        },
                                        common::UserDefinedParameterType {
                                            base: "xyz".to_string(),
                                            parameter: "ABC8".to_string(),
                                        },
                                        common::UserDefinedParameterType {
                                            base: "9".to_string(),
                                            parameter: "ABC9".to_string(),
                                        },
                                    ],
                                },
                            ),
                        },
                    },
                },
                id: Some("CCSDS_OMM_VERS".to_string()),
                version: "2.0".to_string(),
            });
    }

    #[test]
    fn test_parse_omm_message_xml_with_empty_object_id() {
        // According to Orekit this should fail to parse due to having an empty object id. However, the XSD type of
        // the object id is just xsd:string, which allows empty strings too.

        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<omm id="CCSDS_OMM_VERS" version="2.0">
    <header>
    <CREATION_DATE>2021-03-24T23:00:00.000</CREATION_DATE>
    <ORIGINATOR>CelesTrak</ORIGINATOR>
    </header>
    <body>
    <segment>
        <metadata>
        <OBJECT_NAME>STARLETTE</OBJECT_NAME>
        <OBJECT_ID></OBJECT_ID>
        <CENTER_NAME>EARTH</CENTER_NAME>
        <REF_FRAME>TEME</REF_FRAME>
        <TIME_SYSTEM>UTC</TIME_SYSTEM>
        <MEAN_ELEMENT_THEORY>SGP4</MEAN_ELEMENT_THEORY>
        </metadata>
        <data>
        <meanElements>
            <EPOCH>2021-03-22T13:21:09.224928</EPOCH>
            <MEAN_MOTION>13.82309053</MEAN_MOTION>
            <ECCENTRICITY>.0205751</ECCENTRICITY>
            <INCLINATION>49.8237</INCLINATION>
            <RA_OF_ASC_NODE>93.8140</RA_OF_ASC_NODE>
            <ARG_OF_PERICENTER>224.8348</ARG_OF_PERICENTER>
            <MEAN_ANOMALY>133.5761</MEAN_ANOMALY>
        </meanElements>
        <tleParameters>
            <EPHEMERIS_TYPE>0</EPHEMERIS_TYPE>
            <CLASSIFICATION_TYPE>U</CLASSIFICATION_TYPE>
            <NORAD_CAT_ID>7646</NORAD_CAT_ID>
            <ELEMENT_SET_NO>999</ELEMENT_SET_NO>
            <REV_AT_EPOCH>32997</REV_AT_EPOCH>
            <BSTAR>-.47102E-5</BSTAR>
            <MEAN_MOTION_DOT>-.147E-5</MEAN_MOTION_DOT>
            <MEAN_MOTION_DDOT>0</MEAN_MOTION_DDOT>
        </tleParameters>
        </data>
    </segment>
    </body>
</omm>"#;

        let message = OmmType::from_xml_str(xml).unwrap();

        assert_eq!(
            message,
            OmmType {
                header: common::OdmHeader {
                    comment_list: vec![],
                    classification_list: vec![],
                    creation_date: common::EpochType("2021-03-24T23:00:00.000".to_string()),
                    originator: "CelesTrak".to_string(),
                    message_id: None,
                },
                body: OmmBody {
                    segment: OmmSegment {
                        metadata: OmmMetadata {
                            comment_list: vec![],
                            object_name: "STARLETTE".to_string(),
                            object_id: "".to_string(),
                            center_name: "EARTH".to_string(),
                            ref_frame: "TEME".to_string(),
                            ref_frame_epoch: None,
                            time_system: "UTC".to_string(),
                            mean_element_theory: "SGP4".to_string(),
                        },
                        data: OmmData {
                            comment_list: vec![],
                            mean_elements: MeanElementsType {
                                comment_list: vec![],
                                epoch: common::EpochType("2021-03-22T13:21:09.224928".to_string()),
                                semi_major_axis: None,
                                mean_motion: Some(RevType {
                                    base: 13.82309053,
                                    units: None,
                                },),
                                eccentricity: common::NonNegativeDouble(0.0205751,),
                                inclination: common::InclinationType {
                                    base: 49.8237,
                                    units: None,
                                },
                                ra_of_asc_node: common::AngleType {
                                    base: 93.8140,

                                    units: None,
                                },
                                arg_of_pericenter: common::AngleType {
                                    base: 224.8348,
                                    units: None,
                                },
                                mean_anomaly: common::AngleType {
                                    base: 133.5761,
                                    units: None,
                                },
                                gm: None,
                            },
                            spacecraft_parameters: None,
                            tle_parameters: Some(TleParametersType {
                                comment_list: vec![],
                                ephemeris_type: Some(0,),
                                classification_type: Some("U".to_string()),
                                norad_cat_id: Some(7646,),
                                element_set_no: Some(ElementSetNoType("999".to_string()),),
                                rev_at_epoch: Some(32997,),
                                bstar: Some(BStarType {
                                    base: -4.7102e-6,
                                    units: None,
                                },),
                                bterm: None,
                                mean_motion_dot: DRevType {
                                    base: -1.47e-6,
                                    units: None,
                                },
                                mean_motion_ddot: Some(DRevType {
                                    base: 0.0,
                                    units: None,
                                },),
                                agom: None,
                            },),
                            covariance_matrix: None,
                            user_defined_parameters: None,
                        },
                    },
                },
                id: Some("CCSDS_OMM_VERS".to_string()),
                version: "2.0".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_omm_message_xml_2() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<omm  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="http://sanaregistry.org/r/ndmxml/ndmxml-1.0-master.xsd"
        id="CCSDS_OMM_VERS" version="3.0">
    <header>
    <COMMENT> THIS IS AN XML VERSION OF THE OMM </COMMENT>
    <CREATION_DATE>2007-065T16:00:00</CREATION_DATE>
    <ORIGINATOR>NOAA</ORIGINATOR>
    <MESSAGE_ID>OMM 201113719185</MESSAGE_ID>
    </header>

    <body>
    <segment>
        <metadata>
        <OBJECT_NAME>GOES-9</OBJECT_NAME>
        <OBJECT_ID>1995-025A</OBJECT_ID>
        <CENTER_NAME>EARTH</CENTER_NAME>
        <REF_FRAME>TEME</REF_FRAME>
        <TIME_SYSTEM>UTC</TIME_SYSTEM>
        <MEAN_ELEMENT_THEORY>SGP/SGP4</MEAN_ELEMENT_THEORY>
        </metadata>

        <data>
        <meanElements>
            <EPOCH>2007-064T10:34:41.4264</EPOCH>
            <MEAN_MOTION>1.00273272</MEAN_MOTION>
            <ECCENTRICITY>0.0005013</ECCENTRICITY>
            <INCLINATION>3.0539</INCLINATION>
            <RA_OF_ASC_NODE>81.7939</RA_OF_ASC_NODE>
            <ARG_OF_PERICENTER>249.2363</ARG_OF_PERICENTER>
            <MEAN_ANOMALY>150.1602</MEAN_ANOMALY>
            <GM>398600.8</GM>
        </meanElements>
        <tleParameters>
            <NORAD_CAT_ID>23581</NORAD_CAT_ID>
            <ELEMENT_SET_NO>0925</ELEMENT_SET_NO>
            <REV_AT_EPOCH>4316</REV_AT_EPOCH>
            <BSTAR>0.0001</BSTAR>
            <MEAN_MOTION_DOT>-0.00000113</MEAN_MOTION_DOT>
            <MEAN_MOTION_DDOT>0.0</MEAN_MOTION_DDOT>
        </tleParameters>
        <covarianceMatrix>
            <COV_REF_FRAME>TEME</COV_REF_FRAME>
            <CX_X>3.331349476038534e-04</CX_X>
            <CY_X>4.618927349220216e-04</CY_X>
            <CY_Y>6.782421679971363e-04</CY_Y>
            <CZ_X>-3.070007847730449e-04</CZ_X>
            <CZ_Y>-4.221234189514228e-04</CZ_Y>
            <CZ_Z>3.231931992380369e-04</CZ_Z>
            <CX_DOT_X>-3.349365033922630e-07</CX_DOT_X>
            <CX_DOT_Y>-4.686084221046758e-07</CX_DOT_Y>
            <CX_DOT_Z>2.484949578400095e-07</CX_DOT_Z>
            <CX_DOT_X_DOT>4.296022805587290e-10</CX_DOT_X_DOT>
            <CY_DOT_X>-2.211832501084875e-07</CY_DOT_X>
            <CY_DOT_Y>-2.864186892102733e-07</CY_DOT_Y>
            <CY_DOT_Z>1.798098699846038e-07</CY_DOT_Z>
            <CY_DOT_X_DOT>2.608899201686016e-10</CY_DOT_X_DOT>
            <CY_DOT_Y_DOT>1.767514756338532e-10</CY_DOT_Y_DOT>
            <CZ_DOT_X>-3.041346050686871e-07</CZ_DOT_X>
            <CZ_DOT_Y>-4.989496988610662e-07</CZ_DOT_Y>
            <CZ_DOT_Z>3.540310904497689e-07</CZ_DOT_Z>
            <CZ_DOT_X_DOT>1.869263192954590e-10</CZ_DOT_X_DOT>
            <CZ_DOT_Y_DOT>1.008862586240695e-10</CZ_DOT_Y_DOT>
            <CZ_DOT_Z_DOT>6.224444338635500e-10</CZ_DOT_Z_DOT>
        </covarianceMatrix>
        </data>
    </segment>
    </body>
</omm>"#;

        let message = OmmType::from_xml_str(xml).unwrap();

        assert_eq!(
            message,
            OmmType {
                header: common::OdmHeader {
                    comment_list: vec!["THIS IS AN XML VERSION OF THE OMM".to_string()],
                    classification_list: vec![],
                    creation_date: common::EpochType("2007-065T16:00:00".to_string()),
                    originator: "NOAA".to_string(),
                    message_id: Some("OMM 201113719185".to_string()),
                },
                body: OmmBody {
                    segment: OmmSegment {
                        metadata: OmmMetadata {
                            comment_list: vec![],
                            object_name: "GOES-9".to_string(),
                            object_id: "1995-025A".to_string(),
                            center_name: "EARTH".to_string(),
                            ref_frame: "TEME".to_string(),
                            ref_frame_epoch: None,
                            time_system: "UTC".to_string(),
                            mean_element_theory: "SGP/SGP4".to_string(),
                        },
                        data: OmmData {
                            comment_list: vec![],
                            mean_elements: MeanElementsType {
                                comment_list: vec![],
                                epoch: common::EpochType("2007-064T10:34:41.4264".to_string()),
                                semi_major_axis: None,
                                mean_motion: Some(RevType {
                                    base: 1.00273272,
                                    units: None,
                                },),
                                eccentricity: common::NonNegativeDouble(0.0005013,),
                                inclination: common::InclinationType {
                                    base: 3.0539,
                                    units: None,
                                },
                                ra_of_asc_node: common::AngleType {
                                    base: 81.7939,
                                    units: None,
                                },
                                arg_of_pericenter: common::AngleType {
                                    base: 249.2363,
                                    units: None,
                                },
                                mean_anomaly: common::AngleType {
                                    base: 150.1602,
                                    units: None,
                                },
                                gm: Some(common::GmType {
                                    base: common::PositiveDouble(398600.8,),
                                    units: None,
                                },),
                            },
                            spacecraft_parameters: None,
                            tle_parameters: Some(TleParametersType {
                                comment_list: vec![],
                                ephemeris_type: None,
                                classification_type: None,
                                norad_cat_id: Some(23581,),
                                element_set_no: Some(ElementSetNoType("0925".to_string()),),
                                rev_at_epoch: Some(4316,),
                                bstar: Some(BStarType {
                                    base: 0.0001,
                                    units: None,
                                },),
                                bterm: None,
                                mean_motion_dot: DRevType {
                                    base: -1.13e-6,
                                    units: None,
                                },
                                mean_motion_ddot: Some(DRevType {
                                    base: 0.0,
                                    units: None,
                                },),
                                agom: None,
                            },),
                            covariance_matrix: Some(common::OpmCovarianceMatrixType {
                                comment_list: vec![],
                                cov_ref_frame: Some("TEME".to_string()),
                                cx_x: common::PositionCovarianceType {
                                    base: 0.0003331349476038534,
                                    units: None,
                                },
                                cy_x: common::PositionCovarianceType {
                                    base: 0.0004618927349220216,
                                    units: None,
                                },
                                cy_y: common::PositionCovarianceType {
                                    base: 0.0006782421679971363,
                                    units: None,
                                },
                                cz_x: common::PositionCovarianceType {
                                    base: -0.0003070007847730449,
                                    units: None,
                                },
                                cz_y: common::PositionCovarianceType {
                                    base: -0.0004221234189514228,
                                    units: None,
                                },
                                cz_z: common::PositionCovarianceType {
                                    base: 0.0003231931992380369,
                                    units: None,
                                },
                                cx_dot_x: common::PositionVelocityCovarianceType {
                                    base: -3.34936503392263e-7,
                                    units: None,
                                },
                                cx_dot_y: common::PositionVelocityCovarianceType {
                                    base: -4.686084221046758e-7,
                                    units: None,
                                },
                                cx_dot_z: common::PositionVelocityCovarianceType {
                                    base: 2.484949578400095e-7,
                                    units: None,
                                },
                                cx_dot_x_dot: common::VelocityCovarianceType {
                                    base: 4.29602280558729e-10,
                                    units: None,
                                },
                                cy_dot_x: common::PositionVelocityCovarianceType {
                                    base: -2.211832501084875e-7,
                                    units: None,
                                },
                                cy_dot_y: common::PositionVelocityCovarianceType {
                                    base: -2.864186892102733e-7,
                                    units: None,
                                },
                                cy_dot_z: common::PositionVelocityCovarianceType {
                                    base: 1.798098699846038e-7,
                                    units: None,
                                },
                                cy_dot_x_dot: common::VelocityCovarianceType {
                                    base: 2.608899201686016e-10,
                                    units: None,
                                },
                                cy_dot_y_dot: common::VelocityCovarianceType {
                                    base: 1.767514756338532e-10,
                                    units: None,
                                },
                                cz_dot_x: common::PositionVelocityCovarianceType {
                                    base: -3.041346050686871e-7,
                                    units: None,
                                },
                                cz_dot_y: common::PositionVelocityCovarianceType {
                                    base: -4.989496988610662e-7,
                                    units: None,
                                },
                                cz_dot_z: common::PositionVelocityCovarianceType {
                                    base: 3.540310904497689e-7,
                                    units: None,
                                },
                                cz_dot_x_dot: common::VelocityCovarianceType {
                                    base: 1.86926319295459e-10,
                                    units: None,
                                },
                                cz_dot_y_dot: common::VelocityCovarianceType {
                                    base: 1.008862586240695e-10,
                                    units: None,
                                },
                                cz_dot_z_dot: common::VelocityCovarianceType {
                                    base: 6.2244443386355e-10,
                                    units: None,
                                },
                            },),
                            user_defined_parameters: None,
                        },
                    },
                },
                id: Some("CCSDS_OMM_VERS".to_string()),
                version: "3.0".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_omm_message_xml_with_units() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<omm  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="http://sanaregistry.org/r/ndmxml/ndmxml-1.0-master.xsd"
        id="CCSDS_OMM_VERS" version="3.0">
    <header>
    <COMMENT> THIS IS AN XML VERSION OF THE OMM </COMMENT>
    <CREATION_DATE>2007-065T16:00:00</CREATION_DATE>
    <ORIGINATOR>NOAA</ORIGINATOR>
    <MESSAGE_ID> OMM 201113719185</MESSAGE_ID>
    </header>

    <body>
    <segment>
        <metadata>
        <OBJECT_NAME>GOES-9</OBJECT_NAME>
        <OBJECT_ID>1995-025A</OBJECT_ID>
        <CENTER_NAME>EARTH</CENTER_NAME>
        <REF_FRAME>TEME</REF_FRAME>
        <TIME_SYSTEM>UTC</TIME_SYSTEM>
        <MEAN_ELEMENT_THEORY>SGP/SGP4</MEAN_ELEMENT_THEORY>
        </metadata>

        <data>
        <meanElements>
            <COMMENT>mean Elements</COMMENT>
            <EPOCH>2007-064T10:34:41.4264</EPOCH>
            <MEAN_MOTION units="rev/day">1.00273272</MEAN_MOTION>
            <ECCENTRICITY>0.0005013</ECCENTRICITY>
            <INCLINATION units="deg">3.0539</INCLINATION>
            <RA_OF_ASC_NODE units="deg">81.7939</RA_OF_ASC_NODE>
            <ARG_OF_PERICENTER units="deg">249.2363</ARG_OF_PERICENTER>
            <MEAN_ANOMALY units="deg">150.1602</MEAN_ANOMALY>
            <GM>398600.8</GM>
        </meanElements>
        <tleParameters>
            <COMMENT>tle Parameters</COMMENT>
            <NORAD_CAT_ID>23581</NORAD_CAT_ID>
            <ELEMENT_SET_NO>0925</ELEMENT_SET_NO>
            <REV_AT_EPOCH>4316</REV_AT_EPOCH>
            <BSTAR units="1/ER">0.0001</BSTAR>
            <MEAN_MOTION_DOT units="rev/day**2">-0.00000113</MEAN_MOTION_DOT>
            <MEAN_MOTION_DDOT units="rev/day**3">0.0</MEAN_MOTION_DDOT>
        </tleParameters>
        <covarianceMatrix>
            <COMMENT>covariance Matrix</COMMENT>
            <COV_REF_FRAME>TEME</COV_REF_FRAME>
            <CX_X>3.331349476038534e-04</CX_X>
            <CY_X>4.618927349220216e-04</CY_X>
            <CY_Y>6.782421679971363e-04</CY_Y>
            <CZ_X>-3.070007847730449e-04</CZ_X>
            <CZ_Y>-4.221234189514228e-04</CZ_Y>
            <CZ_Z>3.231931992380369e-04</CZ_Z>
            <CX_DOT_X>-3.349365033922630e-07</CX_DOT_X>
            <CX_DOT_Y>-4.686084221046758e-07</CX_DOT_Y>
            <CX_DOT_Z>2.484949578400095e-07</CX_DOT_Z>
            <CX_DOT_X_DOT>4.296022805587290e-10</CX_DOT_X_DOT>
            <CY_DOT_X>-2.211832501084875e-07</CY_DOT_X>
            <CY_DOT_Y>-2.864186892102733e-07</CY_DOT_Y>
            <CY_DOT_Z>1.798098699846038e-07</CY_DOT_Z>
            <CY_DOT_X_DOT>2.608899201686016e-10</CY_DOT_X_DOT>
            <CY_DOT_Y_DOT>1.767514756338532e-10</CY_DOT_Y_DOT>
            <CZ_DOT_X>-3.041346050686871e-07</CZ_DOT_X>
            <CZ_DOT_Y>-4.989496988610662e-07</CZ_DOT_Y>
            <CZ_DOT_Z>3.540310904497689e-07</CZ_DOT_Z>
            <CZ_DOT_X_DOT>1.869263192954590e-10</CZ_DOT_X_DOT>
            <CZ_DOT_Y_DOT>1.008862586240695e-10</CZ_DOT_Y_DOT>
            <CZ_DOT_Z_DOT>6.224444338635500e-10</CZ_DOT_Z_DOT>
        </covarianceMatrix>
        </data>
    </segment>
    </body>
</omm>"#;

        let message = OmmType::from_xml_str(xml).unwrap();

        assert_eq!(
            message,
            OmmType {
                header: common::OdmHeader {
                    comment_list: vec!["THIS IS AN XML VERSION OF THE OMM".to_string()],
                    classification_list: vec![],
                    creation_date: common::EpochType("2007-065T16:00:00".to_string()),
                    originator: "NOAA".to_string(),
                    message_id: Some("OMM 201113719185".to_string()),
                },
                body: OmmBody {
                    segment: OmmSegment {
                        metadata: OmmMetadata {
                            comment_list: vec![],
                            object_name: "GOES-9".to_string(),
                            object_id: "1995-025A".to_string(),
                            center_name: "EARTH".to_string(),
                            ref_frame: "TEME".to_string(),
                            ref_frame_epoch: None,
                            time_system: "UTC".to_string(),
                            mean_element_theory: "SGP/SGP4".to_string(),
                        },
                        data: OmmData {
                            comment_list: vec![],
                            mean_elements: MeanElementsType {
                                comment_list: vec!["mean Elements".to_string()],
                                epoch: common::EpochType("2007-064T10:34:41.4264".to_string()),
                                semi_major_axis: None,
                                mean_motion: Some(RevType {
                                    base: 1.00273272,
                                    units: Some(RevUnits("rev/day".to_string()),),
                                },),
                                eccentricity: common::NonNegativeDouble(0.0005013,),
                                inclination: common::InclinationType {
                                    base: 3.0539,
                                    units: Some(common::AngleUnits("deg".to_string()),),
                                },
                                ra_of_asc_node: common::AngleType {
                                    base: 81.7939,
                                    units: Some(common::AngleUnits("deg".to_string()),),
                                },
                                arg_of_pericenter: common::AngleType {
                                    base: 249.2363,
                                    units: Some(common::AngleUnits("deg".to_string()),),
                                },
                                mean_anomaly: common::AngleType {
                                    base: 150.1602,
                                    units: Some(common::AngleUnits("deg".to_string()),),
                                },
                                gm: Some(common::GmType {
                                    base: common::PositiveDouble(398600.8,),
                                    units: None,
                                },),
                            },
                            spacecraft_parameters: None,
                            tle_parameters: Some(TleParametersType {
                                comment_list: vec!["tle Parameters".to_string()],
                                ephemeris_type: None,
                                classification_type: None,
                                norad_cat_id: Some(23581,),
                                element_set_no: Some(ElementSetNoType("0925".to_string()),),
                                rev_at_epoch: Some(4316,),
                                bstar: Some(BStarType {
                                    base: 0.0001,
                                    units: Some(BStarUnits("1/ER".to_string()),),
                                },),
                                bterm: None,
                                mean_motion_dot: DRevType {
                                    base: -1.13e-6,
                                    units: Some(DRevUnits("rev/day**2".to_string()),),
                                },
                                mean_motion_ddot: Some(DRevType {
                                    base: 0.0,
                                    units: Some(DRevUnits("rev/day**3".to_string()),),
                                },),
                                agom: None,
                            },),
                            covariance_matrix: Some(common::OpmCovarianceMatrixType {
                                comment_list: vec!["covariance Matrix".to_string()],
                                cov_ref_frame: Some("TEME".to_string()),
                                cx_x: common::PositionCovarianceType {
                                    base: 0.0003331349476038534,
                                    units: None,
                                },
                                cy_x: common::PositionCovarianceType {
                                    base: 0.0004618927349220216,
                                    units: None,
                                },
                                cy_y: common::PositionCovarianceType {
                                    base: 0.0006782421679971363,
                                    units: None,
                                },
                                cz_x: common::PositionCovarianceType {
                                    base: -0.0003070007847730449,
                                    units: None,
                                },
                                cz_y: common::PositionCovarianceType {
                                    base: -0.0004221234189514228,
                                    units: None,
                                },
                                cz_z: common::PositionCovarianceType {
                                    base: 0.0003231931992380369,
                                    units: None,
                                },
                                cx_dot_x: common::PositionVelocityCovarianceType {
                                    base: -3.34936503392263e-7,
                                    units: None,
                                },
                                cx_dot_y: common::PositionVelocityCovarianceType {
                                    base: -4.686084221046758e-7,
                                    units: None,
                                },
                                cx_dot_z: common::PositionVelocityCovarianceType {
                                    base: 2.484949578400095e-7,
                                    units: None,
                                },
                                cx_dot_x_dot: common::VelocityCovarianceType {
                                    base: 4.29602280558729e-10,
                                    units: None,
                                },
                                cy_dot_x: common::PositionVelocityCovarianceType {
                                    base: -2.211832501084875e-7,
                                    units: None,
                                },
                                cy_dot_y: common::PositionVelocityCovarianceType {
                                    base: -2.864186892102733e-7,
                                    units: None,
                                },
                                cy_dot_z: common::PositionVelocityCovarianceType {
                                    base: 1.798098699846038e-7,
                                    units: None,
                                },
                                cy_dot_x_dot: common::VelocityCovarianceType {
                                    base: 2.608899201686016e-10,
                                    units: None,
                                },
                                cy_dot_y_dot: common::VelocityCovarianceType {
                                    base: 1.767514756338532e-10,
                                    units: None,
                                },
                                cz_dot_x: common::PositionVelocityCovarianceType {
                                    base: -3.041346050686871e-7,
                                    units: None,
                                },
                                cz_dot_y: common::PositionVelocityCovarianceType {
                                    base: -4.989496988610662e-7,
                                    units: None,
                                },
                                cz_dot_z: common::PositionVelocityCovarianceType {
                                    base: 3.540310904497689e-7,
                                    units: None,
                                },
                                cz_dot_x_dot: common::VelocityCovarianceType {
                                    base: 1.86926319295459e-10,
                                    units: None,
                                },
                                cz_dot_y_dot: common::VelocityCovarianceType {
                                    base: 1.008862586240695e-10,
                                    units: None,
                                },
                                cz_dot_z_dot: common::VelocityCovarianceType {
                                    base: 6.2244443386355e-10,
                                    units: None,
                                },
                            },),
                            user_defined_parameters: None,
                        },
                    },
                },
                id: Some("CCSDS_OMM_VERS".to_string()),
                version: "3.0".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_omm_message_xml_3() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<omm id="CCSDS_OMM_VERS" version="2.0">
    <header>
        <CREATION_DATE>2021-03-24T23:00:00.000</CREATION_DATE>
        <ORIGINATOR>CelesTrak</ORIGINATOR>
    </header>
    <body>
    <segment>
        <metadata>
            <OBJECT_NAME>STARLETTE</OBJECT_NAME>
            <OBJECT_ID>1975-010A</OBJECT_ID>
            <CENTER_NAME>EARTH</CENTER_NAME>
            <REF_FRAME>TEME</REF_FRAME>
            <TIME_SYSTEM>UTC</TIME_SYSTEM>
            <MEAN_ELEMENT_THEORY>SGP4</MEAN_ELEMENT_THEORY>
        </metadata>
        <data>
            <meanElements>
                <EPOCH>2008-09-20T12:25:40.104192</EPOCH>
                <MEAN_MOTION units="rev/day">15.72125391</MEAN_MOTION>
                <ECCENTRICITY>0.0006703</ECCENTRICITY>
                <INCLINATION units="deg">51.6416</INCLINATION>
                <RA_OF_ASC_NODE units="deg">247.4627</RA_OF_ASC_NODE>
                <ARG_OF_PERICENTER units="deg">130.5360</ARG_OF_PERICENTER>
                <MEAN_ANOMALY units="deg">325.0288</MEAN_ANOMALY>
                <GM units="km**3/s**2">398600.8</GM>
            </meanElements>
            <tleParameters>
                <EPHEMERIS_TYPE>0</EPHEMERIS_TYPE>
                <CLASSIFICATION_TYPE>U</CLASSIFICATION_TYPE>
                <NORAD_CAT_ID>7646</NORAD_CAT_ID>
                <ELEMENT_SET_NO>999</ELEMENT_SET_NO>
                <REV_AT_EPOCH>32997</REV_AT_EPOCH>
                <BSTAR>-.47102E-5</BSTAR>
                <MEAN_MOTION_DOT>-.147E-5</MEAN_MOTION_DOT>
                <MEAN_MOTION_DDOT>0</MEAN_MOTION_DDOT>
            </tleParameters>
            <userDefinedParameters>
                <USER_DEFINED parameter="FOO">foo enters</USER_DEFINED>
                <USER_DEFINED parameter="BAR">a bar</USER_DEFINED>
            </userDefinedParameters>
        </data>
    </segment>
    </body>
</omm>"#;

        let message = OmmType::from_xml_str(xml).unwrap();

        assert_eq!(
            message,
            OmmType {
                header: common::OdmHeader {
                    comment_list: vec![],
                    classification_list: vec![],
                    creation_date: common::EpochType("2021-03-24T23:00:00.000".to_string()),
                    originator: "CelesTrak".to_string(),
                    message_id: None,
                },
                body: OmmBody {
                    segment: OmmSegment {
                        metadata: OmmMetadata {
                            comment_list: vec![],
                            object_name: "STARLETTE".to_string(),
                            object_id: "1975-010A".to_string(),
                            center_name: "EARTH".to_string(),
                            ref_frame: "TEME".to_string(),
                            ref_frame_epoch: None,
                            time_system: "UTC".to_string(),
                            mean_element_theory: "SGP4".to_string(),
                        },
                        data: OmmData {
                            comment_list: vec![],
                            mean_elements: MeanElementsType {
                                comment_list: vec![],
                                epoch: common::EpochType("2008-09-20T12:25:40.104192".to_string()),
                                semi_major_axis: None,
                                mean_motion: Some(RevType {
                                    base: 15.72125391,
                                    units: Some(RevUnits("rev/day".to_string()),),
                                },),
                                eccentricity: common::NonNegativeDouble(0.0006703,),
                                inclination: common::InclinationType {
                                    base: 51.6416,
                                    units: Some(common::AngleUnits("deg".to_string()),),
                                },
                                ra_of_asc_node: common::AngleType {
                                    base: 247.4627,
                                    units: Some(common::AngleUnits("deg".to_string()),),
                                },
                                arg_of_pericenter: common::AngleType {
                                    base: 130.536,
                                    units: Some(common::AngleUnits("deg".to_string()),),
                                },
                                mean_anomaly: common::AngleType {
                                    base: 325.0288,
                                    units: Some(common::AngleUnits("deg".to_string()),),
                                },
                                gm: Some(common::GmType {
                                    base: common::PositiveDouble(398600.8,),
                                    units: Some(common::GmUnits("km**3/s**2".to_string()),),
                                },),
                            },
                            spacecraft_parameters: None,
                            tle_parameters: Some(TleParametersType {
                                comment_list: vec![],
                                ephemeris_type: Some(0,),
                                classification_type: Some("U".to_string()),
                                norad_cat_id: Some(7646,),
                                element_set_no: Some(ElementSetNoType("999".to_string()),),
                                rev_at_epoch: Some(32997,),
                                bstar: Some(BStarType {
                                    base: -4.7102e-6,
                                    units: None,
                                },),
                                bterm: None,
                                mean_motion_dot: DRevType {
                                    base: -1.47e-6,
                                    units: None,
                                },
                                mean_motion_ddot: Some(DRevType {
                                    base: 0.0,
                                    units: None,
                                },),
                                agom: None,
                            },),
                            covariance_matrix: None,
                            user_defined_parameters: Some(common::UserDefinedType {
                                comment_list: vec![],
                                user_defined_list: vec![
                                    common::UserDefinedParameterType {
                                        base: "foo enters".to_string(),
                                        parameter: "FOO".to_string(),
                                    },
                                    common::UserDefinedParameterType {
                                        base: "a bar".to_string(),
                                        parameter: "BAR".to_string(),
                                    },
                                ],
                            },),
                        },
                    },
                },
                id: Some("CCSDS_OMM_VERS".to_string()),
                version: "2.0".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_omm_message_spurious() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<omm id="CCSDS_OMM_VERS" version="2.0">
    <header>
    <CREATION_DATE>2021-03-24T23:00:00.000</CREATION_DATE>
    <ORIGINATOR>CelesTrak</ORIGINATOR>
    </header>
    <body>
    <segment>
        <metadata>
        <OBJECT_NAME>STARLETTE</OBJECT_NAME>
        <OBJECT_ID>1975-010A</OBJECT_ID>
        <CENTER_NAME>EARTH</CENTER_NAME>
        <REF_FRAME>TEME</REF_FRAME>
        <TIME_SYSTEM>UTC</TIME_SYSTEM>
        <MEAN_ELEMENT_THEORY>SGP4</MEAN_ELEMENT_THEORY>
        </metadata>
        <metadata>
        <COMMENT>second metadata is an error</COMMENT>
        </metadata>
    </segment>
    </body>
</omm>"#;

        let message = OmmType::from_xml_str(xml);

        assert!(message.is_err());
    }

    #[test]
    fn test_parse_omm_message_kvn() {
        //@TOOD add back user defined stuff
        let kvn = r#"CCSDS_OMM_VERS = 3.0
 COMMENT this is a comment
 COMMENT here is another one
 CREATION_DATE = 2007-06-05T16:00:00
 ORIGINATOR = NOAA/USA
 COMMENT this comment doesn't say much
 OBJECT_NAME = GOES 9
 OBJECT_ID = 1995-025A
 CENTER_NAME = EARTH
 REF_FRAME = TOD
 REF_FRAME_EPOCH = 2000-01-03T10:34:00
 TIME_SYSTEM = MRT
 MEAN_ELEMENT_THEORY = SOME THEORY
 COMMENT the following data is what we're looking for
 EPOCH = 2000-01-05T10:00:00
 SEMI_MAJOR_AXIS = 6800
 ECCENTRICITY = 0.0005013
 INCLINATION = 3.0539
 RA_OF_ASC_NODE = 81.7939
 ARG_OF_PERICENTER = 249.2363
 MEAN_ANOMALY = 150.1602
 COMMENT spacecraft data
 MASS = 300
 SOLAR_RAD_AREA = 5
 SOLAR_RAD_COEFF = 0.001
 DRAG_AREA = 4
 DRAG_COEFF = 0.002
 COMMENT Covariance matrix
 COV_REF_FRAME = TNW
 CX_X = 3.331349476038534e-04
 CY_X = 4.618927349220216e-04
 CY_Y = 6.782421679971363e-04
 CZ_X = -3.070007847730449e-04
 CZ_Y = -4.221234189514228e-04
 CZ_Z = 3.231931992380369e-04
 CX_DOT_X = -3.349365033922630e-07
 CX_DOT_Y = -4.686084221046758e-07
 CX_DOT_Z = 2.484949578400095e-07
 CX_DOT_X_DOT = 4.296022805587290e-10
 CY_DOT_X = -2.211832501084875e-07
 CY_DOT_Y = -2.864186892102733e-07
 CY_DOT_Z = 1.798098699846038e-07
 CY_DOT_X_DOT = 2.608899201686016e-10
 CY_DOT_Y_DOT = 1.767514756338532e-10
 CZ_DOT_X = -3.041346050686871e-07
 CZ_DOT_Y = -4.989496988610662e-07
 CZ_DOT_Z = 3.540310904497689e-07
 CZ_DOT_X_DOT = 1.869263192954590e-10
 CZ_DOT_Y_DOT = 1.008862586240695e-10
 CZ_DOT_Z_DOT = 6.224444338635500e-10"#;

        assert_eq!(
            crate::ndm::kvn::KvnDeserializer::from_kvn_str(kvn),
            Ok(OmmType {
                header: common::OdmHeader {
                    comment_list: vec![
                        "this is a comment".to_string(),
                        "here is another one".to_string(),
                    ],
                    classification_list: vec![],
                    creation_date: common::EpochType("2007-06-05T16:00:00".to_string()),
                    originator: "NOAA/USA".to_string(),
                    message_id: None,
                },
                body: OmmBody {
                    segment: OmmSegment {
                        metadata: OmmMetadata {
                            comment_list: vec!["this comment doesn't say much".to_string()],
                            object_name: "GOES 9".to_string(),
                            object_id: "1995-025A".to_string(),
                            center_name: "EARTH".to_string(),
                            ref_frame: "TOD".to_string(),
                            ref_frame_epoch: Some(common::EpochType(
                                "2000-01-03T10:34:00".to_string(),
                            )),
                            time_system: "MRT".to_string(),
                            mean_element_theory: "SOME THEORY".to_string(),
                        },
                        data: OmmData {
                            comment_list: vec![
                                "the following data is what we're looking for".to_string(),
                            ],
                            mean_elements: MeanElementsType {
                                comment_list: vec![],
                                epoch: common::EpochType("2000-01-05T10:00:00".to_string()),
                                semi_major_axis: Some(common::DistanceType {
                                    base: 6800.0,
                                    units: None,
                                },),
                                mean_motion: None,
                                eccentricity: common::NonNegativeDouble(0.0005013,),
                                inclination: common::InclinationType {
                                    base: 3.0539,
                                    units: None,
                                },
                                ra_of_asc_node: common::AngleType {
                                    base: 81.7939,
                                    units: None,
                                },
                                arg_of_pericenter: common::AngleType {
                                    base: 249.2363,
                                    units: None,
                                },
                                mean_anomaly: common::AngleType {
                                    base: 150.1602,
                                    units: None,
                                },
                                gm: None,
                            },
                            spacecraft_parameters: Some(common::SpacecraftParametersType {
                                comment_list: vec!["spacecraft data".to_string()],
                                mass: Some(common::MassType {
                                    base: common::NonNegativeDouble(300.0,),
                                    units: None,
                                },),
                                solar_rad_area: Some(common::AreaType {
                                    base: common::NonNegativeDouble(5.0,),
                                    units: None,
                                },),
                                solar_rad_coeff: Some(common::NonNegativeDouble(0.001,)),
                                drag_area: Some(common::AreaType {
                                    base: common::NonNegativeDouble(4.0,),
                                    units: None,
                                },),
                                drag_coeff: Some(common::NonNegativeDouble(0.002,)),
                            },),
                            tle_parameters: None,
                            covariance_matrix: Some(common::OpmCovarianceMatrixType {
                                comment_list: vec![],
                                cov_ref_frame: Some("TNW".to_string()),
                                cx_x: common::PositionCovarianceType {
                                    base: 0.0003331349476038534,
                                    units: None,
                                },
                                cy_x: common::PositionCovarianceType {
                                    base: 0.0004618927349220216,
                                    units: None,
                                },
                                cy_y: common::PositionCovarianceType {
                                    base: 0.0006782421679971363,
                                    units: None,
                                },
                                cz_x: common::PositionCovarianceType {
                                    base: -0.0003070007847730449,
                                    units: None,
                                },
                                cz_y: common::PositionCovarianceType {
                                    base: -0.0004221234189514228,
                                    units: None,
                                },
                                cz_z: common::PositionCovarianceType {
                                    base: 0.0003231931992380369,
                                    units: None,
                                },
                                cx_dot_x: common::PositionVelocityCovarianceType {
                                    base: -3.34936503392263e-7,
                                    units: None,
                                },
                                cx_dot_y: common::PositionVelocityCovarianceType {
                                    base: -4.686084221046758e-7,
                                    units: None,
                                },
                                cx_dot_z: common::PositionVelocityCovarianceType {
                                    base: 2.484949578400095e-7,
                                    units: None,
                                },
                                cx_dot_x_dot: common::VelocityCovarianceType {
                                    base: 4.29602280558729e-10,
                                    units: None,
                                },
                                cy_dot_x: common::PositionVelocityCovarianceType {
                                    base: -2.211832501084875e-7,
                                    units: None,
                                },
                                cy_dot_y: common::PositionVelocityCovarianceType {
                                    base: -2.864186892102733e-7,
                                    units: None,
                                },
                                cy_dot_z: common::PositionVelocityCovarianceType {
                                    base: 1.798098699846038e-7,
                                    units: None,
                                },
                                cy_dot_x_dot: common::VelocityCovarianceType {
                                    base: 2.608899201686016e-10,
                                    units: None,
                                },
                                cy_dot_y_dot: common::VelocityCovarianceType {
                                    base: 1.767514756338532e-10,
                                    units: None,
                                },
                                cz_dot_x: common::PositionVelocityCovarianceType {
                                    base: -3.041346050686871e-7,
                                    units: None,
                                },
                                cz_dot_y: common::PositionVelocityCovarianceType {
                                    base: -4.989496988610662e-7,
                                    units: None,
                                },
                                cz_dot_z: common::PositionVelocityCovarianceType {
                                    base: 3.540310904497689e-7,
                                    units: None,
                                },
                                cz_dot_x_dot: common::VelocityCovarianceType {
                                    base: 1.86926319295459e-10,
                                    units: None,
                                },
                                cz_dot_y_dot: common::VelocityCovarianceType {
                                    base: 1.008862586240695e-10,
                                    units: None,
                                },
                                cz_dot_z_dot: common::VelocityCovarianceType {
                                    base: 6.2244443386355e-10,
                                    units: None,
                                },
                            },),
                            user_defined_parameters: None,
                        },
                    },
                },
                id: None,
                version: "3.0".to_string(),
            },)
        );
    }
}
