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

mod test {
    use super::*;

    use quick_xml::de::from_str;

    #[test]
    fn test_parse_omm_message1() {
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

        let message: OmmType = from_str(xml).unwrap();

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
                                    "0.0005013".to_string(),
                                ),
                                inclination: common::InclinationType {
                                    base: common::InclinationRange(
                                        "3.0539".to_string(),
                                    ),
                                    units: None,
                                },
                                ra_of_asc_node: common::AngleType {
                                    base: common::AngleRange(
                                        "81.7939".to_string(),
                                    ),
                                    units: None,
                                },
                                arg_of_pericenter: common::AngleType {
                                    base: common::AngleRange(
                                        "249.2363".to_string(),
                                    ),
                                    units: None,
                                },
                                mean_anomaly: common::AngleType {
                                    base: common::AngleRange(
                                        "150.1602".to_string(),
                                    ),
                                    units: None,
                                },
                                gm: Some(
                                    common::GmType {
                                        base: common::PositiveDouble(
                                            "398600.8".to_string(),
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
                id: "CCSDS_OMM_VERS".to_string(),
                version: "2.0".to_string(),
            });
    }

    #[test]
    fn test_parse_omm_message2() {
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

        let message: OmmType = from_str(xml).unwrap();

        assert_eq!(message, 
            OmmType {
                header: common::OdmHeader {
                    comment_list: vec![],
                    classification_list: vec![],
                    creation_date: common::EpochType(
                        "2021-03-24T23:00:00.000".to_string(),
                    ),
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
                                epoch: common::EpochType(
                                    "2021-03-22T13:21:09.224928".to_string(),
                                ),
                                semi_major_axis: None,
                                mean_motion: Some(
                                    RevType {
                                        base: 13.82309053,
                                        units: None,
                                    },
                                ),
                                eccentricity: common::NonNegativeDouble(
                                    ".0205751".to_string(),
                                ),
                                inclination: common::InclinationType {
                                    base: common::InclinationRange(
                                        "49.8237".to_string(),
                                    ),
                                    units: None,
                                },
                                ra_of_asc_node: common::AngleType {
                                    base: common::AngleRange(
                                        "93.8140".to_string(),
                                    ),
                                    units: None,
                                },
                                arg_of_pericenter: common::AngleType {
                                    base: common::AngleRange(
                                        "224.8348".to_string(),
                                    ),
                                    units: None,
                                },
                                mean_anomaly: common::AngleType {
                                    base: common::AngleRange(
                                        "133.5761".to_string(),
                                    ),
                                    units: None,
                                },
                                gm: None,
                            },
                            spacecraft_parameters: None,
                            tle_parameters: Some(
                                TleParametersType {
                                    comment_list: vec![],
                                    ephemeris_type: Some(
                                        0,
                                    ),
                                    classification_type: Some(
                                        "U".to_string(),
                                    ),
                                    norad_cat_id: Some(
                                        7646,
                                    ),
                                    element_set_no: Some(
                                        ElementSetNoType(
                                            "999".to_string(),
                                        ),
                                    ),
                                    rev_at_epoch: Some(
                                        32997,
                                    ),
                                    bstar: Some(
                                        BStarType {
                                            base: -4.7102e-6,
                                            units: None,
                                        },
                                    ),
                                    bterm: None,
                                    mean_motion_dot: DRevType {
                                        base: -1.47e-6,
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
                            user_defined_parameters: None,
                        },
                    },
                },
                id: "CCSDS_OMM_VERS".to_string(),
                version: "2.0".to_string(),
            });
    }
}
