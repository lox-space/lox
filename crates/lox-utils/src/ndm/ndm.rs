use serde;

use super::{ocm, oem, omm, opm};

/// Combined instantiation type. Currently does not support AEM, APM, CDM, RDM, TDM
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct NdmType {
    #[serde(rename = "MESSAGE_ID")]
    pub message_id: Option<String>,
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,

    #[serde(rename = "ocm")]
    pub ocm_list: Vec<ocm::OcmType>,
    #[serde(rename = "oem")]
    pub oem_list: Vec<oem::OemType>,
    #[serde(rename = "omm")]
    pub omm_list: Vec<omm::OmmType>,
    #[serde(rename = "opm")]
    pub opm_list: Vec<opm::OpmType>,
}

mod test {
    use super::super::common;
    use super::*;

    use quick_xml::de::from_str;

    #[test]
    fn test_parse_combined_ndm() {
        let xml = r#"<ndm xsi:noNamespaceSchemaLocation="https://sanaregistry.org/r/ndmxml/ndmxml-1.0-master.xsd">
<MESSAGE_ID>bla</MESSAGE_ID>
<COMMENT>asdfg</COMMENT>
<omm id="CCSDS_OMM_VERS" version="2.0">
    <header>
        <CREATION_DATE/>
        <ORIGINATOR/>
    </header>
    <body>
        <segment>
        </segment>
    </body>
</omm>
<omm id="CCSDS_OMM_VERS" version="2.0">
    <header>
        <CREATION_DATE/>
        <ORIGINATOR/>
    </header>
    <body>
        <segment>
            <metadata>
                <OBJECT_NAME>NUSAT-2 (BATATA)</OBJECT_NAME>
                <OBJECT_ID>2016-033C</OBJECT_ID>
                <CENTER_NAME>EARTH</CENTER_NAME>
                <REF_FRAME>TEME</REF_FRAME>
                <TIME_SYSTEM>UTC</TIME_SYSTEM>
                <MEAN_ELEMENT_THEORY>SGP4</MEAN_ELEMENT_THEORY>
            </metadata>
            <data>
                <meanElements>
                    <EPOCH>2020-12-04T15:27:01.975104</EPOCH>
                    <MEAN_MOTION>15.30610336</MEAN_MOTION>
                    <ECCENTRICITY>.0011780</ECCENTRICITY>
                    <INCLINATION>97.4090</INCLINATION>
                    <RA_OF_ASC_NODE>71.7453</RA_OF_ASC_NODE>
                    <ARG_OF_PERICENTER>193.9419</ARG_OF_PERICENTER>
                    <MEAN_ANOMALY>272.1492</MEAN_ANOMALY>
                </meanElements>
                <tleParameters>
                    <EPHEMERIS_TYPE>0</EPHEMERIS_TYPE>
                    <CLASSIFICATION_TYPE>U</CLASSIFICATION_TYPE>
                    <NORAD_CAT_ID>41558</NORAD_CAT_ID>
                    <ELEMENT_SET_NO>999</ELEMENT_SET_NO>
                    <REV_AT_EPOCH>25191</REV_AT_EPOCH>
                    <BSTAR>.15913E-3</BSTAR>
                    <MEAN_MOTION_DOT>4.64E-5</MEAN_MOTION_DOT>
                    <MEAN_MOTION_DDOT>0</MEAN_MOTION_DDOT>
                </tleParameters>
            </data>
        </segment>
    </body>
</omm>
<omm id="CCSDS_OMM_VERS" version="2.0">
    <header>
        <CREATION_DATE/>
        <ORIGINATOR/>
    </header>
    <body>
        <segment>
            <metadata>
                <OBJECT_NAME>NUSAT-13 (EMMY)</OBJECT_NAME>
                <OBJECT_ID>2020-079G</OBJECT_ID>
                <CENTER_NAME>EARTH</CENTER_NAME>
                <REF_FRAME>TEME</REF_FRAME>
                <TIME_SYSTEM>UTC</TIME_SYSTEM>
                <MEAN_ELEMENT_THEORY>SGP4</MEAN_ELEMENT_THEORY>
            </metadata>
            <data>
                <meanElements>
                    <EPOCH>2020-12-04T13:30:01.539648</EPOCH>
                    <MEAN_MOTION>15.31433655</MEAN_MOTION>
                    <ECCENTRICITY>.0009574</ECCENTRICITY>
                    <INCLINATION>97.2663</INCLINATION>
                    <RA_OF_ASC_NODE>51.2167</RA_OF_ASC_NODE>
                    <ARG_OF_PERICENTER>149.8567</ARG_OF_PERICENTER>
                    <MEAN_ANOMALY>322.5146</MEAN_ANOMALY>
                </meanElements>
                <tleParameters>
                    <EPHEMERIS_TYPE>0</EPHEMERIS_TYPE>
                    <CLASSIFICATION_TYPE>U</CLASSIFICATION_TYPE>
                    <NORAD_CAT_ID>46833</NORAD_CAT_ID>
                    <ELEMENT_SET_NO>999</ELEMENT_SET_NO>
                    <REV_AT_EPOCH>434</REV_AT_EPOCH>
                    <BSTAR>.14401E-3</BSTAR>
                    <MEAN_MOTION_DOT>4.301E-5</MEAN_MOTION_DOT>
                    <MEAN_MOTION_DDOT>0</MEAN_MOTION_DDOT>
                </tleParameters>
            </data>
        </segment>
    </body>
</omm>
<omm id="CCSDS_OMM_VERS" version="2.0">
    <header>
        <CREATION_DATE/>
        <ORIGINATOR/>
    </header>
    <body>
        <segment>
            <metadata>
                <OBJECT_NAME>NUSAT-17 (MARY)</OBJECT_NAME>
                <OBJECT_ID>2020-079J</OBJECT_ID>
                <CENTER_NAME>EARTH</CENTER_NAME>
                <REF_FRAME>TEME</REF_FRAME>
                <TIME_SYSTEM>UTC</TIME_SYSTEM>
                <MEAN_ELEMENT_THEORY>SGP4</MEAN_ELEMENT_THEORY>
            </metadata>
            <data>
                <meanElements>
                    <EPOCH>2020-12-04T16:27:08.698176</EPOCH>
                    <MEAN_MOTION>15.31317097</MEAN_MOTION>
                    <ECCENTRICITY>.0009674</ECCENTRICITY>
                    <INCLINATION>97.2671</INCLINATION>
                    <RA_OF_ASC_NODE>51.3486</RA_OF_ASC_NODE>
                    <ARG_OF_PERICENTER>160.8608</ARG_OF_PERICENTER>
                    <MEAN_ANOMALY>302.2789</MEAN_ANOMALY>
                </meanElements>
                <tleParameters>
                    <EPHEMERIS_TYPE>0</EPHEMERIS_TYPE>
                    <CLASSIFICATION_TYPE>U</CLASSIFICATION_TYPE>
                    <NORAD_CAT_ID>46835</NORAD_CAT_ID>
                    <ELEMENT_SET_NO>999</ELEMENT_SET_NO>
                    <REV_AT_EPOCH>436</REV_AT_EPOCH>
                    <BSTAR>-.13273E-3</BSTAR>
                    <MEAN_MOTION_DOT>-4.087E-5</MEAN_MOTION_DOT>
                    <MEAN_MOTION_DDOT>0</MEAN_MOTION_DDOT>
                </tleParameters>
            </data>
        </segment>
    </body>
</omm>
<omm id="CCSDS_OMM_VERS" version="2.0">
    <header>
        <CREATION_DATE/>
        <ORIGINATOR/>
    </header>
    <body>
        <segment>
            <metadata>
                <OBJECT_NAME>NUSAT-18 (VERA)</OBJECT_NAME>
                <OBJECT_ID>2020-079K</OBJECT_ID>
                <CENTER_NAME>EARTH</CENTER_NAME>
                <REF_FRAME>TEME</REF_FRAME>
                <TIME_SYSTEM>UTC</TIME_SYSTEM>
                <MEAN_ELEMENT_THEORY>SGP4</MEAN_ELEMENT_THEORY>
            </metadata>
            <data>
                <meanElements>
                    <EPOCH>2020-12-04T13:13:33.140064</EPOCH>
                    <MEAN_MOTION>15.32037173</MEAN_MOTION>
                    <ECCENTRICITY>.0009024</ECCENTRICITY>
                    <INCLINATION>97.2666</INCLINATION>
                    <RA_OF_ASC_NODE>51.2301</RA_OF_ASC_NODE>
                    <ARG_OF_PERICENTER>167.2057</ARG_OF_PERICENTER>
                    <MEAN_ANOMALY>304.5569</MEAN_ANOMALY>
                </meanElements>
                <tleParameters>
                    <EPHEMERIS_TYPE>0</EPHEMERIS_TYPE>
                    <CLASSIFICATION_TYPE>U</CLASSIFICATION_TYPE>
                    <NORAD_CAT_ID>46836</NORAD_CAT_ID>
                    <ELEMENT_SET_NO>999</ELEMENT_SET_NO>
                    <REV_AT_EPOCH>434</REV_AT_EPOCH>
                    <BSTAR>.13328E-3</BSTAR>
                    <MEAN_MOTION_DOT>4.05E-5</MEAN_MOTION_DOT>
                    <MEAN_MOTION_DDOT>0</MEAN_MOTION_DDOT>
                </tleParameters>
            </data>
        </segment>
    </body>
</omm>
</ndm>"#;

        let message: NdmType = from_str(xml).unwrap();

        assert_eq!(
            message,
            NdmType {
                message_id: Some("bla".to_string()),
                comment_list: vec!["asdfg".to_string()],
                ocm_list: vec![],
                oem_list: vec![],
                omm_list: vec![
                    omm::OmmType {
                        header: common::OdmHeader {
                            comment_list: vec![],
                            classification_list: vec![],
                            creation_date: common::EpochType("".to_string()),
                            originator: "".to_string(),
                            message_id: None,
                        },
                        body: omm::OmmBody {
                            segment: omm::OmmSegment {
                                metadata: omm::OmmMetadata {
                                    comment_list: vec![],
                                    object_name: "".to_string(),
                                    object_id: "".to_string(),
                                    center_name: "".to_string(),
                                    ref_frame: "".to_string(),
                                    ref_frame_epoch: None,
                                    time_system: "".to_string(),
                                    mean_element_theory: "".to_string(),
                                },
                                data: omm::OmmData {
                                    comment_list: vec![],
                                    mean_elements: omm::MeanElementsType {
                                        comment_list: vec![],
                                        epoch: common::EpochType("".to_string()),
                                        semi_major_axis: None,
                                        mean_motion: None,
                                        eccentricity: common::NonNegativeDouble(0.0),
                                        inclination: common::InclinationType {
                                            base: common::InclinationRange(0.0),
                                            units: None,
                                        },
                                        ra_of_asc_node: common::AngleType {
                                            base: common::AngleRange(0.0),
                                            units: None,
                                        },
                                        arg_of_pericenter: common::AngleType {
                                            base: common::AngleRange(0.0),
                                            units: None,
                                        },
                                        mean_anomaly: common::AngleType {
                                            base: common::AngleRange(0.0),
                                            units: None,
                                        },
                                        gm: None,
                                    },
                                    spacecraft_parameters: None,
                                    tle_parameters: None,
                                    covariance_matrix: None,
                                    user_defined_parameters: None,
                                },
                            },
                        },
                        id: "CCSDS_OMM_VERS".to_string(),
                        version: "2.0".to_string(),
                    },
                    omm::OmmType {
                        header: common::OdmHeader {
                            comment_list: vec![],
                            classification_list: vec![],
                            creation_date: common::EpochType("".to_string()),
                            originator: "".to_string(),
                            message_id: None,
                        },
                        body: omm::OmmBody {
                            segment: omm::OmmSegment {
                                metadata: omm::OmmMetadata {
                                    comment_list: vec![],
                                    object_name: "NUSAT-2 (BATATA)".to_string(),
                                    object_id: "2016-033C".to_string(),
                                    center_name: "EARTH".to_string(),
                                    ref_frame: "TEME".to_string(),
                                    ref_frame_epoch: None,
                                    time_system: "UTC".to_string(),
                                    mean_element_theory: "SGP4".to_string(),
                                },
                                data: omm::OmmData {
                                    comment_list: vec![],
                                    mean_elements: omm::MeanElementsType {
                                        comment_list: vec![],
                                        epoch: common::EpochType(
                                            "2020-12-04T15:27:01.975104".to_string()
                                        ),
                                        semi_major_axis: None,
                                        mean_motion: Some(omm::RevType {
                                            base: 15.30610336,
                                            units: None,
                                        }),
                                        eccentricity: common::NonNegativeDouble(0.001178),
                                        inclination: common::InclinationType {
                                            base: common::InclinationRange(97.409),
                                            units: None,
                                        },
                                        ra_of_asc_node: common::AngleType {
                                            base: common::AngleRange(71.7453),
                                            units: None,
                                        },
                                        arg_of_pericenter: common::AngleType {
                                            base: common::AngleRange(193.9419),
                                            units: None,
                                        },
                                        mean_anomaly: common::AngleType {
                                            base: common::AngleRange(272.1492),
                                            units: None,
                                        },
                                        gm: None,
                                    },
                                    spacecraft_parameters: None,
                                    tle_parameters: Some(omm::TleParametersType {
                                        comment_list: vec![],
                                        ephemeris_type: Some(0),
                                        classification_type: Some("U".to_string()),
                                        norad_cat_id: Some(41558),
                                        element_set_no: Some(omm::ElementSetNoType(
                                            "999".to_string()
                                        )),
                                        rev_at_epoch: Some(25191),
                                        bstar: Some(omm::BStarType {
                                            base: 0.00015913,
                                            units: None,
                                        }),
                                        bterm: None,
                                        mean_motion_dot: omm::DRevType {
                                            base: 4.64e-5,
                                            units: None,
                                        },
                                        mean_motion_ddot: Some(omm::DRevType {
                                            base: 0.0,
                                            units: None,
                                        }),
                                        agom: None,
                                    }),
                                    covariance_matrix: None,
                                    user_defined_parameters: None,
                                },
                            },
                        },
                        id: "CCSDS_OMM_VERS".to_string(),
                        version: "2.0".to_string(),
                    },
                    omm::OmmType {
                        header: common::OdmHeader {
                            comment_list: vec![],
                            classification_list: vec![],
                            creation_date: common::EpochType("".to_string()),
                            originator: "".to_string(),
                            message_id: None,
                        },
                        body: omm::OmmBody {
                            segment: omm::OmmSegment {
                                metadata: omm::OmmMetadata {
                                    comment_list: vec![],
                                    object_name: "NUSAT-13 (EMMY)".to_string(),
                                    object_id: "2020-079G".to_string(),
                                    center_name: "EARTH".to_string(),
                                    ref_frame: "TEME".to_string(),
                                    ref_frame_epoch: None,
                                    time_system: "UTC".to_string(),
                                    mean_element_theory: "SGP4".to_string(),
                                },
                                data: omm::OmmData {
                                    comment_list: vec![],
                                    mean_elements: omm::MeanElementsType {
                                        comment_list: vec![],
                                        epoch: common::EpochType(
                                            "2020-12-04T13:30:01.539648".to_string()
                                        ),
                                        semi_major_axis: None,
                                        mean_motion: Some(omm::RevType {
                                            base: 15.31433655,
                                            units: None,
                                        }),
                                        eccentricity: common::NonNegativeDouble(0.0009574),
                                        inclination: common::InclinationType {
                                            base: common::InclinationRange(97.2663),
                                            units: None,
                                        },
                                        ra_of_asc_node: common::AngleType {
                                            base: common::AngleRange(51.2167),
                                            units: None,
                                        },
                                        arg_of_pericenter: common::AngleType {
                                            base: common::AngleRange(149.8567),
                                            units: None,
                                        },
                                        mean_anomaly: common::AngleType {
                                            base: common::AngleRange(322.5146),
                                            units: None,
                                        },
                                        gm: None,
                                    },
                                    spacecraft_parameters: None,
                                    tle_parameters: Some(omm::TleParametersType {
                                        comment_list: vec![],
                                        ephemeris_type: Some(0),
                                        classification_type: Some("U".to_string()),
                                        norad_cat_id: Some(46833),
                                        element_set_no: Some(omm::ElementSetNoType(
                                            "999".to_string()
                                        )),
                                        rev_at_epoch: Some(434),
                                        bstar: Some(omm::BStarType {
                                            base: 0.00014401,
                                            units: None,
                                        }),
                                        bterm: None,
                                        mean_motion_dot: omm::DRevType {
                                            base: 4.301e-5,
                                            units: None,
                                        },
                                        mean_motion_ddot: Some(omm::DRevType {
                                            base: 0.0,
                                            units: None,
                                        }),
                                        agom: None,
                                    }),
                                    covariance_matrix: None,
                                    user_defined_parameters: None,
                                },
                            },
                        },
                        id: "CCSDS_OMM_VERS".to_string(),
                        version: "2.0".to_string(),
                    },
                    omm::OmmType {
                        header: common::OdmHeader {
                            comment_list: vec![],
                            classification_list: vec![],
                            creation_date: common::EpochType("".to_string()),
                            originator: "".to_string(),
                            message_id: None,
                        },
                        body: omm::OmmBody {
                            segment: omm::OmmSegment {
                                metadata: omm::OmmMetadata {
                                    comment_list: vec![],
                                    object_name: "NUSAT-17 (MARY)".to_string(),
                                    object_id: "2020-079J".to_string(),
                                    center_name: "EARTH".to_string(),
                                    ref_frame: "TEME".to_string(),
                                    ref_frame_epoch: None,
                                    time_system: "UTC".to_string(),
                                    mean_element_theory: "SGP4".to_string(),
                                },
                                data: omm::OmmData {
                                    comment_list: vec![],
                                    mean_elements: omm::MeanElementsType {
                                        comment_list: vec![],
                                        epoch: common::EpochType(
                                            "2020-12-04T16:27:08.698176".to_string()
                                        ),
                                        semi_major_axis: None,
                                        mean_motion: Some(omm::RevType {
                                            base: 15.31317097,
                                            units: None,
                                        }),
                                        eccentricity: common::NonNegativeDouble(0.0009674),
                                        inclination: common::InclinationType {
                                            base: common::InclinationRange(97.2671),
                                            units: None,
                                        },
                                        ra_of_asc_node: common::AngleType {
                                            base: common::AngleRange(51.3486),
                                            units: None,
                                        },
                                        arg_of_pericenter: common::AngleType {
                                            base: common::AngleRange(160.8608),
                                            units: None,
                                        },
                                        mean_anomaly: common::AngleType {
                                            base: common::AngleRange(302.2789),
                                            units: None,
                                        },
                                        gm: None,
                                    },
                                    spacecraft_parameters: None,
                                    tle_parameters: Some(omm::TleParametersType {
                                        comment_list: vec![],
                                        ephemeris_type: Some(0),
                                        classification_type: Some("U".to_string()),
                                        norad_cat_id: Some(46835),
                                        element_set_no: Some(omm::ElementSetNoType(
                                            "999".to_string()
                                        )),
                                        rev_at_epoch: Some(436),
                                        bstar: Some(omm::BStarType {
                                            base: -0.00013273,
                                            units: None,
                                        }),
                                        bterm: None,
                                        mean_motion_dot: omm::DRevType {
                                            base: -4.087e-5,
                                            units: None,
                                        },
                                        mean_motion_ddot: Some(omm::DRevType {
                                            base: 0.0,
                                            units: None,
                                        }),
                                        agom: None,
                                    }),
                                    covariance_matrix: None,
                                    user_defined_parameters: None,
                                },
                            },
                        },
                        id: "CCSDS_OMM_VERS".to_string(),
                        version: "2.0".to_string(),
                    },
                    omm::OmmType {
                        header: common::OdmHeader {
                            comment_list: vec![],
                            classification_list: vec![],
                            creation_date: common::EpochType("".to_string()),
                            originator: "".to_string(),
                            message_id: None,
                        },
                        body: omm::OmmBody {
                            segment: omm::OmmSegment {
                                metadata: omm::OmmMetadata {
                                    comment_list: vec![],
                                    object_name: "NUSAT-18 (VERA)".to_string(),
                                    object_id: "2020-079K".to_string(),
                                    center_name: "EARTH".to_string(),
                                    ref_frame: "TEME".to_string(),
                                    ref_frame_epoch: None,
                                    time_system: "UTC".to_string(),
                                    mean_element_theory: "SGP4".to_string(),
                                },
                                data: omm::OmmData {
                                    comment_list: vec![],
                                    mean_elements: omm::MeanElementsType {
                                        comment_list: vec![],
                                        epoch: common::EpochType(
                                            "2020-12-04T13:13:33.140064".to_string()
                                        ),
                                        semi_major_axis: None,
                                        mean_motion: Some(omm::RevType {
                                            base: 15.32037173,
                                            units: None,
                                        }),
                                        eccentricity: common::NonNegativeDouble(0.0009024),
                                        inclination: common::InclinationType {
                                            base: common::InclinationRange(97.2666),
                                            units: None,
                                        },
                                        ra_of_asc_node: common::AngleType {
                                            base: common::AngleRange(51.2301),
                                            units: None,
                                        },
                                        arg_of_pericenter: common::AngleType {
                                            base: common::AngleRange(167.2057),
                                            units: None,
                                        },
                                        mean_anomaly: common::AngleType {
                                            base: common::AngleRange(304.5569),
                                            units: None,
                                        },
                                        gm: None,
                                    },
                                    spacecraft_parameters: None,
                                    tle_parameters: Some(omm::TleParametersType {
                                        comment_list: vec![],
                                        ephemeris_type: Some(0),
                                        classification_type: Some("U".to_string()),
                                        norad_cat_id: Some(46836),
                                        element_set_no: Some(omm::ElementSetNoType(
                                            "999".to_string()
                                        )),
                                        rev_at_epoch: Some(434),
                                        bstar: Some(omm::BStarType {
                                            base: 0.00013328,
                                            units: None,
                                        }),
                                        bterm: None,
                                        mean_motion_dot: omm::DRevType {
                                            base: 4.05e-5,
                                            units: None,
                                        },
                                        mean_motion_ddot: Some(omm::DRevType {
                                            base: 0.0,
                                            units: None,
                                        }),
                                        agom: None,
                                    }),
                                    covariance_matrix: None,
                                    user_defined_parameters: None,
                                },
                            },
                        },
                        id: "CCSDS_OMM_VERS".to_string(),
                        version: "2.0".to_string(),
                    },
                ],
                opm_list: vec![],
            },
        );
    }
}
