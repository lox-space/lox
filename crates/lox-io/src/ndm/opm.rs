/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Deserializers for XML and KVN CCSDS Orbit Parameter Message
//!
//! To deserialize an XML message:
//!
//! ```
//! # let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
//! # <opm  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
//! # xsi:noNamespaceSchemaLocation="http://sanaregistry.org/r/ndmxml/ndmxml-1.0-master.xsd"
//! # id="CCSDS_OPM_VERS" version="3.0">
//! #
//! # <header>
//! # <COMMENT>THIS IS AN XML VERSION OF THE OPM</COMMENT>
//! # <CREATION_DATE>2001-11-06T09:23:57</CREATION_DATE>
//! # <ORIGINATOR>JAXA</ORIGINATOR>
//! # <MESSAGE_ID>OPM 201113719185</MESSAGE_ID>
//! # </header>
//! # <body>
//! # <segment>
//! # <metadata>
//! #     <COMMENT>GEOCENTRIC, CARTESIAN, EARTH FIXED</COMMENT>
//! #     <OBJECT_NAME>OSPREY 5</OBJECT_NAME>
//! #     <OBJECT_ID>1998-999A</OBJECT_ID>
//! #     <CENTER_NAME>EARTH</CENTER_NAME>
//! #     <REF_FRAME>TOD</REF_FRAME>
//! #     <REF_FRAME_EPOCH>1998-12-18T14:28:15.1172</REF_FRAME_EPOCH>
//! #     <TIME_SYSTEM>UTC</TIME_SYSTEM>
//! # </metadata>
//! # <data>
//! #     <stateVector>
//! #         <EPOCH>2008-09-20T12:25:40.104192</EPOCH>
//! #         <X units="km">4086.147180</X>
//! #         <Y units="km">-994.936814</Y>
//! #         <Z units="km">5250.678791</Z>
//! #         <X_DOT units="km/s">2.511071</X_DOT>
//! #         <Y_DOT units="km/s">7.255240</Y_DOT>
//! #         <Z_DOT units="km/s">-0.583165</Z_DOT>
//! #     </stateVector>
//! #     <keplerianElements>
//! #         <SEMI_MAJOR_AXIS units="km">6730.96</SEMI_MAJOR_AXIS>
//! #         <ECCENTRICITY>0.0006703</ECCENTRICITY>
//! #         <INCLINATION units="deg">51.6416</INCLINATION>
//! #         <RA_OF_ASC_NODE units="deg">247.463</RA_OF_ASC_NODE>
//! #         <ARG_OF_PERICENTER units="deg">130.536</ARG_OF_PERICENTER>
//! #         <TRUE_ANOMALY units="deg">324.985</TRUE_ANOMALY>
//! #         <GM units="km**3/s**2">398600.9368</GM>
//! #     </keplerianElements>
//! #     <spacecraftParameters>
//! #         <MASS>3000.000000</MASS>
//! #         <SOLAR_RAD_AREA>18.770000</SOLAR_RAD_AREA>
//! #         <SOLAR_RAD_COEFF>1.000000</SOLAR_RAD_COEFF>
//! #         <DRAG_AREA>18.770000</DRAG_AREA>
//! #         <DRAG_COEFF>2.500000</DRAG_COEFF>
//! #     </spacecraftParameters>
//! #     <covarianceMatrix>
//! #         <COV_REF_FRAME>ITRF1997</COV_REF_FRAME>
//! #         <CX_X>0.316</CX_X>
//! #         <CY_X>0.722</CY_X>
//! #         <CY_Y>0.518</CY_Y>
//! #         <CZ_X>0.202</CZ_X>
//! #         <CZ_Y>0.715</CZ_Y>
//! #         <CZ_Z>0.002</CZ_Z>
//! #         <CX_DOT_X>0.912</CX_DOT_X>
//! #         <CX_DOT_Y>0.306</CX_DOT_Y>
//! #         <CX_DOT_Z>0.276</CX_DOT_Z>
//! #         <CX_DOT_X_DOT>0.797</CX_DOT_X_DOT>
//! #         <CY_DOT_X>0.562</CY_DOT_X>
//! #         <CY_DOT_Y>0.899</CY_DOT_Y>
//! #         <CY_DOT_Z>0.022</CY_DOT_Z>
//! #         <CY_DOT_X_DOT>0.079</CY_DOT_X_DOT>
//! #         <CY_DOT_Y_DOT>0.415</CY_DOT_Y_DOT>
//! #         <CZ_DOT_X>0.245</CZ_DOT_X>
//! #         <CZ_DOT_Y>0.965</CZ_DOT_Y>
//! #         <CZ_DOT_Z>0.950</CZ_DOT_Z>
//! #         <CZ_DOT_X_DOT>0.435</CZ_DOT_X_DOT>
//! #         <CZ_DOT_Y_DOT>0.621</CZ_DOT_Y_DOT>
//! #         <CZ_DOT_Z_DOT>0.991</CZ_DOT_Z_DOT>
//! #     </covarianceMatrix>
//! #     <maneuverParameters>
//! #         <COMMENT>Maneuver 1</COMMENT>
//! #         <MAN_EPOCH_IGNITION>2008-09-20T12:41:09.984493</MAN_EPOCH_IGNITION>
//! #         <MAN_DURATION units="s">180.000</MAN_DURATION>
//! #         <MAN_DELTA_MASS units="kg">-0.001</MAN_DELTA_MASS>
//! #         <MAN_REF_FRAME>RSW</MAN_REF_FRAME>
//! #         <MAN_DV_1 units="km/s">0.000000</MAN_DV_1>
//! #         <MAN_DV_2 units="km/s">0.280000</MAN_DV_2>
//! #         <MAN_DV_3 units="km/s">0.000000</MAN_DV_3>
//! #     </maneuverParameters>
//! #     <maneuverParameters>
//! #         <MAN_EPOCH_IGNITION>2008-09-20T13:33:11.374985</MAN_EPOCH_IGNITION>
//! #         <MAN_DURATION units="s">180.000</MAN_DURATION>
//! #         <MAN_DELTA_MASS units="kg">-0.001</MAN_DELTA_MASS>
//! #         <MAN_REF_FRAME>RSW</MAN_REF_FRAME>
//! #         <MAN_DV_1 units="km/s">0.000000</MAN_DV_1>
//! #         <MAN_DV_2 units="km/s">0.270000</MAN_DV_2>
//! #         <MAN_DV_3 units="km/s">0.000000</MAN_DV_3>
//! #     </maneuverParameters>
//! # </data>
//! # </segment>
//! # </body>
//! # </opm>"#;
//! #
//! # use lox_io::ndm::opm::OpmType;
//! #
//! let message: OpmType = quick_xml::de::from_str(xml).unwrap();
//! ```

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
pub struct OpmType {
    #[serde(rename = "header")]
    pub header: common::OdmHeader,
    #[serde(rename = "body")]
    pub body: OpmBody,
    #[serde(rename = "@id")]
    // Marked as option for the KVN deserializer
    pub id: Option<String>,
    #[serde(rename = "@version")]
    pub version: String,
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
pub struct OpmBody {
    #[serde(rename = "segment")]
    pub segment: OpmSegment,
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
pub struct OpmSegment {
    #[serde(rename = "metadata")]
    pub metadata: OpmMetadata,
    #[serde(rename = "data")]
    pub data: OpmData,
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
pub struct OpmData {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "stateVector")]
    pub state_vector: common::StateVectorType,
    #[serde(rename = "keplerianElements")]
    pub keplerian_elements: Option<KeplerianElementsType>,
    #[serde(rename = "spacecraftParameters")]
    pub spacecraft_parameters: Option<common::SpacecraftParametersType>,
    #[serde(rename = "covarianceMatrix")]
    pub covariance_matrix: Option<common::OpmCovarianceMatrixType>,
    #[serde(rename = "maneuverParameters")]
    pub maneuver_parameters_list: Vec<ManeuverParametersType>,
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
    #[serde(rename = "TRUE_ANOMALY")]
    pub true_anomaly: Option<common::AngleType>,
    #[serde(rename = "MEAN_ANOMALY")]
    pub mean_anomaly: Option<common::AngleType>,
    #[serde(rename = "GM")]
    pub gm: common::GmType,
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

#[cfg(test)]
mod test {
    use super::*;

    use quick_xml::de::from_str;

    #[test]
    fn test_parse_opm_message_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<opm  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="http://sanaregistry.org/r/ndmxml/ndmxml-1.0-master.xsd"
        id="CCSDS_OPM_VERS" version="3.0">

    <header>
    <COMMENT>THIS IS AN XML VERSION OF THE OPM</COMMENT>
    <CREATION_DATE>2001-11-06T09:23:57</CREATION_DATE>
    <ORIGINATOR>JAXA</ORIGINATOR>
    <MESSAGE_ID>OPM 201113719185</MESSAGE_ID>
    </header>
    <body>
    <segment>
        <metadata>
            <COMMENT>GEOCENTRIC, CARTESIAN, EARTH FIXED</COMMENT>
            <OBJECT_NAME>OSPREY 5</OBJECT_NAME>
            <OBJECT_ID>1998-999A</OBJECT_ID>
            <CENTER_NAME>EARTH</CENTER_NAME>
            <REF_FRAME>TOD</REF_FRAME>
            <REF_FRAME_EPOCH>1998-12-18T14:28:15.1172</REF_FRAME_EPOCH>
            <TIME_SYSTEM>UTC</TIME_SYSTEM>
        </metadata>
        <data>
            <stateVector>
                <EPOCH>2008-09-20T12:25:40.104192</EPOCH>
                <X units="km">4086.147180</X>
                <Y units="km">-994.936814</Y>
                <Z units="km">5250.678791</Z>
                <X_DOT units="km/s">2.511071</X_DOT>
                <Y_DOT units="km/s">7.255240</Y_DOT>
                <Z_DOT units="km/s">-0.583165</Z_DOT>
            </stateVector>
            <keplerianElements>
                <SEMI_MAJOR_AXIS units="km">6730.96</SEMI_MAJOR_AXIS>
                <ECCENTRICITY>0.0006703</ECCENTRICITY>
                <INCLINATION units="deg">51.6416</INCLINATION>
                <RA_OF_ASC_NODE units="deg">247.463</RA_OF_ASC_NODE>
                <ARG_OF_PERICENTER units="deg">130.536</ARG_OF_PERICENTER>
                <TRUE_ANOMALY units="deg">324.985</TRUE_ANOMALY>
                <GM units="km**3/s**2">398600.9368</GM>
            </keplerianElements>
            <spacecraftParameters>
                <MASS>3000.000000</MASS>
                <SOLAR_RAD_AREA>18.770000</SOLAR_RAD_AREA>
                <SOLAR_RAD_COEFF>1.000000</SOLAR_RAD_COEFF>
                <DRAG_AREA>18.770000</DRAG_AREA>
                <DRAG_COEFF>2.500000</DRAG_COEFF>
            </spacecraftParameters>
            <covarianceMatrix>
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
            <maneuverParameters>
                <COMMENT>Maneuver 1</COMMENT>
                <MAN_EPOCH_IGNITION>2008-09-20T12:41:09.984493</MAN_EPOCH_IGNITION>
                <MAN_DURATION units="s">180.000</MAN_DURATION>
                <MAN_DELTA_MASS units="kg">-0.001</MAN_DELTA_MASS>
                <MAN_REF_FRAME>RSW</MAN_REF_FRAME>
                <MAN_DV_1 units="km/s">0.000000</MAN_DV_1>
                <MAN_DV_2 units="km/s">0.280000</MAN_DV_2>
                <MAN_DV_3 units="km/s">0.000000</MAN_DV_3>
            </maneuverParameters>
            <maneuverParameters>
                <MAN_EPOCH_IGNITION>2008-09-20T13:33:11.374985</MAN_EPOCH_IGNITION>
                <MAN_DURATION units="s">180.000</MAN_DURATION>
                <MAN_DELTA_MASS units="kg">-0.001</MAN_DELTA_MASS>
                <MAN_REF_FRAME>RSW</MAN_REF_FRAME>
                <MAN_DV_1 units="km/s">0.000000</MAN_DV_1>
                <MAN_DV_2 units="km/s">0.270000</MAN_DV_2>
                <MAN_DV_3 units="km/s">0.000000</MAN_DV_3>
            </maneuverParameters>
        </data>
    </segment>
    </body>
</opm>"#;

        let message: OpmType = from_str(xml).unwrap();

        assert_eq!(
            message,
            OpmType {
                header: common::OdmHeader {
                    comment_list: vec!["THIS IS AN XML VERSION OF THE OPM".to_string(),],
                    classification_list: vec![],
                    creation_date: common::EpochType("2001-11-06T09:23:57".to_string(),),
                    originator: "JAXA".to_string(),
                    message_id: Some("OPM 201113719185".to_string(),),
                },
                body: OpmBody {
                    segment: OpmSegment {
                        metadata: OpmMetadata {
                            comment_list: vec!["GEOCENTRIC, CARTESIAN, EARTH FIXED".to_string(),],
                            object_name: "OSPREY 5".to_string(),
                            object_id: "1998-999A".to_string(),
                            center_name: "EARTH".to_string(),
                            ref_frame: "TOD".to_string(),
                            ref_frame_epoch: Some(common::EpochType(
                                "1998-12-18T14:28:15.1172".to_string(),
                            ),),
                            time_system: "UTC".to_string(),
                        },
                        data: OpmData {
                            comment_list: vec![],
                            state_vector: common::StateVectorType {
                                comment_list: vec![],
                                epoch: common::EpochType("2008-09-20T12:25:40.104192".to_string(),),
                                x: common::PositionType {
                                    base: 4086.14718,
                                    units: Some(common::PositionUnits("km".to_string(),),),
                                },
                                y: common::PositionType {
                                    base: -994.936814,
                                    units: Some(common::PositionUnits("km".to_string(),),),
                                },
                                z: common::PositionType {
                                    base: 5250.678791,
                                    units: Some(common::PositionUnits("km".to_string(),),),
                                },
                                x_dot: common::VelocityType {
                                    base: 2.511071,
                                    units: Some(common::VelocityUnits("km/s".to_string(),),),
                                },
                                y_dot: common::VelocityType {
                                    base: 7.25524,
                                    units: Some(common::VelocityUnits("km/s".to_string(),),),
                                },
                                z_dot: common::VelocityType {
                                    base: -0.583165,
                                    units: Some(common::VelocityUnits("km/s".to_string(),),),
                                },
                            },
                            keplerian_elements: Some(KeplerianElementsType {
                                comment_list: vec![],
                                semi_major_axis: common::DistanceType {
                                    base: 6730.96,
                                    units: Some(common::PositionUnits("km".to_string(),),),
                                },
                                eccentricity: common::NonNegativeDouble(0.0006703,),
                                inclination: common::InclinationType {
                                    base: 51.6416,
                                    units: Some(common::AngleUnits("deg".to_string(),),),
                                },
                                ra_of_asc_node: common::AngleType {
                                    base: 247.463,
                                    units: Some(common::AngleUnits("deg".to_string(),),),
                                },
                                arg_of_pericenter: common::AngleType {
                                    base: 130.536,
                                    units: Some(common::AngleUnits("deg".to_string(),),),
                                },
                                true_anomaly: Some(common::AngleType {
                                    base: 324.985,
                                    units: Some(common::AngleUnits("deg".to_string(),),),
                                },),
                                mean_anomaly: None,
                                gm: common::GmType {
                                    base: common::PositiveDouble(398600.9368,),
                                    units: Some(common::GmUnits("km**3/s**2".to_string(),),),
                                },
                            },),
                            spacecraft_parameters: Some(common::SpacecraftParametersType {
                                comment_list: vec![],
                                mass: Some(common::MassType {
                                    base: common::NonNegativeDouble(3000.0,),
                                    units: None,
                                },),
                                solar_rad_area: Some(common::AreaType {
                                    base: common::NonNegativeDouble(18.77,),
                                    units: None,
                                },),
                                solar_rad_coeff: Some(common::NonNegativeDouble(1.0,),),
                                drag_area: Some(common::AreaType {
                                    base: common::NonNegativeDouble(18.77,),
                                    units: None,
                                },),
                                drag_coeff: Some(common::NonNegativeDouble(2.5,),),
                            },),
                            covariance_matrix: Some(common::OpmCovarianceMatrixType {
                                comment_list: vec![],
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
                            },),
                            maneuver_parameters_list: vec![
                                ManeuverParametersType {
                                    comment_list: vec!["Maneuver 1".to_string(),],
                                    man_epoch_ignition: common::EpochType(
                                        "2008-09-20T12:41:09.984493".to_string(),
                                    ),
                                    man_duration: common::DurationType {
                                        base: common::NonNegativeDouble(180.0,),
                                        units: Some(common::TimeUnits("s".to_string(),),),
                                    },
                                    man_delta_mass: common::DeltamassType {
                                        base: common::NegativeDouble(-0.001,),
                                        units: Some(common::MassUnits("kg".to_string(),),),
                                    },
                                    man_ref_frame: "RSW".to_string(),
                                    man_dv_1: common::VelocityType {
                                        base: 0.0,
                                        units: Some(common::VelocityUnits("km/s".to_string(),),),
                                    },
                                    man_dv_2: common::VelocityType {
                                        base: 0.28,
                                        units: Some(common::VelocityUnits("km/s".to_string(),),),
                                    },
                                    man_dv_3: common::VelocityType {
                                        base: 0.0,
                                        units: Some(common::VelocityUnits("km/s".to_string(),),),
                                    },
                                },
                                ManeuverParametersType {
                                    comment_list: vec![],
                                    man_epoch_ignition: common::EpochType(
                                        "2008-09-20T13:33:11.374985".to_string(),
                                    ),
                                    man_duration: common::DurationType {
                                        base: common::NonNegativeDouble(180.0,),
                                        units: Some(common::TimeUnits("s".to_string(),),),
                                    },
                                    man_delta_mass: common::DeltamassType {
                                        base: common::NegativeDouble(-0.001,),
                                        units: Some(common::MassUnits("kg".to_string(),),),
                                    },
                                    man_ref_frame: "RSW".to_string(),
                                    man_dv_1: common::VelocityType {
                                        base: 0.0,
                                        units: Some(common::VelocityUnits("km/s".to_string(),),),
                                    },
                                    man_dv_2: common::VelocityType {
                                        base: 0.27,
                                        units: Some(common::VelocityUnits("km/s".to_string(),),),
                                    },
                                    man_dv_3: common::VelocityType {
                                        base: 0.0,
                                        units: Some(common::VelocityUnits("km/s".to_string(),),),
                                    },
                                },
                            ],
                            user_defined_parameters: None,
                        },
                    },
                },
                id: Some("CCSDS_OPM_VERS".to_string()),
                version: "3.0".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_opm_message_spurious() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<opm  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="http://sanaregistry.org/r/ndmxml/ndmxml-1.0-master.xsd"
        id="CCSDS_OPM_VERS" version="3.0">

    <header>
    <COMMENT>THIS IS AN XML VERSION OF THE OPM</COMMENT>
    <CREATION_DATE>2001-11-06T09:23:57</CREATION_DATE>
    <ORIGINATOR>JAXA</ORIGINATOR>
    <MESSAGE_ID>OPM 201113719185</MESSAGE_ID>
    </header>
    <body>
    <segment>
        <metadata>
        <COMMENT>GEOCENTRIC, CARTESIAN, EARTH FIXED</COMMENT>
        <OBJECT_NAME>OSPREY 5</OBJECT_NAME>
        <OBJECT_ID>1998-999A</OBJECT_ID>
        <CENTER_NAME>EARTH</CENTER_NAME>
        <REF_FRAME>TOD</REF_FRAME>
        <REF_FRAME_EPOCH>1998-12-18T14:28:15.1172</REF_FRAME_EPOCH>
        <TIME_SYSTEM>UTC</TIME_SYSTEM>
        </metadata>
        <metadata>
        <COMMENT>second metadata is an error</COMMENT>
        </metadata>
    </segment>
    </body>
</opm>"#;

        let message: Result<OpmType, _> = from_str(xml);

        assert!(message.is_err());
    }

    #[test]
    fn test_parse_opm_message_kvn() {
        //@TOOD add user defined stuff
        let kvn = r#"CCSDS_OPM_VERS = 3.0
COMMENT Generated by GSOC, R. Kiehling
COMMENT Current intermediate orbit IO2 and maneuver planning data
CREATION_DATE = 2021-06-03T05:33:00.123
ORIGINATOR = GSOC
OBJECT_NAME = EUTELSAT W4
OBJECT_ID = 2021-028A
CENTER_NAME = EARTH
REF_FRAME = TOD
TIME_SYSTEM = UTC
COMMENT State Vector
EPOCH = 2021-06-03T00:00:00.000
X = 6655.9942 [km]
Y = -40218.5751 [km]
Z = -82.9177 [km]
X_DOT = 3.11548208 [km/s]
Y_DOT = 0.47042605 [km/s]
Z_DOT = -0.00101495 [km/s]
COMMENT Keplerian elements
SEMI_MAJOR_AXIS = 41399.5123 [km]
ECCENTRICITY = 0.020842611
INCLINATION = 0.117746 [deg]
RA_OF_ASC_NODE = 17.604721 [deg]
ARG_OF_PERICENTER = 218.242943 [deg]
TRUE_ANOMALY = 41.922339 [deg]
GM = 398600.4415 [km**3/s**2]
COMMENT Spacecraft parameters
MASS = 1913.000 [kg]
SOLAR_RAD_AREA = 10.000 [m**2]
SOLAR_RAD_COEFF = 1.300
DRAG_AREA = 10.000 [m**2]
DRAG_COEFF = 2.300
COMMENT 2 planned maneuvers
COMMENT First maneuver: AMF-3
COMMENT Non-impulsive, thrust direction fixed in inertial frame
MAN_EPOCH_IGNITION = 2021-06-03T09:00:34.1
MAN_DURATION = 132.60 [s]
MAN_DELTA_MASS = -18.418 [kg]
MAN_REF_FRAME = EME2000
MAN_DV_1 = -0.02325700 [km/s]
MAN_DV_2 = 0.01683160 [km/s]
MAN_DV_3 = -0.00893444 [km/s]
COMMENT Second maneuver: first station acquisition maneuver
COMMENT impulsive, thrust direction fixed in RTN frame
MAN_EPOCH_IGNITION = 2021-06-05T18:59:21.0
MAN_DURATION = 0.00 [s]
MAN_DELTA_MASS = -1.469 [kg]
MAN_REF_FRAME = RTN
MAN_DV_1 = 0.00101500 [km/s]
MAN_DV_2 = -0.00187300 [km/s]
MAN_DV_3 = 0.00000000 [km/s]"#;

        assert_eq!(
            crate::ndm::kvn::KvnDeserializer::deserialize(&mut kvn.lines().peekable()),
            Ok(OpmType {
                header: common::OdmHeader {
                    comment_list: vec![
                        "Generated by GSOC, R. Kiehling".to_string(),
                        "Current intermediate orbit IO2 and maneuver planning data".to_string(),
                    ],
                    classification_list: vec![],
                    creation_date: common::EpochType("2021-06-03T05:33:00.123".to_string(),),
                    originator: "GSOC".to_string(),
                    message_id: None,
                },
                body: OpmBody {
                    segment: OpmSegment {
                        metadata: OpmMetadata {
                            comment_list: vec![],
                            object_name: "EUTELSAT W4".to_string(),
                            object_id: "2021-028A".to_string(),
                            center_name: "EARTH".to_string(),
                            ref_frame: "TOD".to_string(),
                            ref_frame_epoch: None,
                            time_system: "UTC".to_string(),
                        },
                        data: OpmData {
                            comment_list: vec!["State Vector".to_string(),],
                            state_vector: common::StateVectorType {
                                comment_list: vec![],
                                epoch: common::EpochType("2021-06-03T00:00:00.000".to_string(),),
                                x: common::PositionType {
                                    base: 6655.9942,
                                    units: Some(common::PositionUnits("km".to_string(),),),
                                },
                                y: common::PositionType {
                                    base: -40218.5751,
                                    units: Some(common::PositionUnits("km".to_string(),),),
                                },
                                z: common::PositionType {
                                    base: -82.9177,
                                    units: Some(common::PositionUnits("km".to_string(),),),
                                },
                                x_dot: common::VelocityType {
                                    base: 3.11548208,
                                    units: Some(common::VelocityUnits("km/s".to_string(),),),
                                },
                                y_dot: common::VelocityType {
                                    base: 0.47042605,
                                    units: Some(common::VelocityUnits("km/s".to_string(),),),
                                },
                                z_dot: common::VelocityType {
                                    base: -0.00101495,
                                    units: Some(common::VelocityUnits("km/s".to_string(),),),
                                },
                            },
                            keplerian_elements: Some(KeplerianElementsType {
                                comment_list: vec!["Keplerian elements".to_string(),],
                                semi_major_axis: common::DistanceType {
                                    base: 41399.5123,
                                    units: Some(common::PositionUnits("km".to_string(),),),
                                },
                                eccentricity: common::NonNegativeDouble(0.020842611,),
                                inclination: common::InclinationType {
                                    base: 0.117746,
                                    units: Some(common::AngleUnits("deg".to_string(),),),
                                },
                                ra_of_asc_node: common::AngleType {
                                    base: 17.604721,
                                    units: Some(common::AngleUnits("deg".to_string(),),),
                                },
                                arg_of_pericenter: common::AngleType {
                                    base: 218.242943,
                                    units: Some(common::AngleUnits("deg".to_string(),),),
                                },
                                true_anomaly: Some(common::AngleType {
                                    base: 41.922339,
                                    units: Some(common::AngleUnits("deg".to_string(),),),
                                },),
                                mean_anomaly: None,
                                gm: common::GmType {
                                    base: common::PositiveDouble(398600.4415,),
                                    units: Some(common::GmUnits("km**3/s**2".to_string(),),),
                                },
                            },),
                            spacecraft_parameters: Some(common::SpacecraftParametersType {
                                comment_list: vec!["Spacecraft parameters".to_string(),],
                                mass: Some(common::MassType {
                                    base: common::NonNegativeDouble(1913.0,),
                                    units: Some(common::MassUnits("kg".to_string(),),),
                                },),
                                solar_rad_area: Some(common::AreaType {
                                    base: common::NonNegativeDouble(10.0,),
                                    units: Some(common::AreaUnits("m**2".to_string(),),),
                                },),
                                solar_rad_coeff: Some(common::NonNegativeDouble(1.3,),),
                                drag_area: Some(common::AreaType {
                                    base: common::NonNegativeDouble(10.0,),
                                    units: Some(common::AreaUnits("m**2".to_string(),),),
                                },),
                                drag_coeff: Some(common::NonNegativeDouble(2.3,),),
                            },),
                            covariance_matrix: None,
                            maneuver_parameters_list: vec![
                                ManeuverParametersType {
                                    comment_list: vec![],
                                    man_epoch_ignition: common::EpochType(
                                        "2021-06-03T09:00:34.1".to_string(),
                                    ),
                                    man_duration: common::DurationType {
                                        base: common::NonNegativeDouble(132.6,),
                                        units: Some(common::TimeUnits("s".to_string(),),),
                                    },
                                    man_delta_mass: common::DeltamassType {
                                        base: common::NegativeDouble(-18.418,),
                                        units: Some(common::MassUnits("kg".to_string(),),),
                                    },
                                    man_ref_frame: "EME2000".to_string(),
                                    man_dv_1: common::VelocityType {
                                        base: -0.023257,
                                        units: Some(common::VelocityUnits("km/s".to_string(),),),
                                    },
                                    man_dv_2: common::VelocityType {
                                        base: 0.0168316,
                                        units: Some(common::VelocityUnits("km/s".to_string(),),),
                                    },
                                    man_dv_3: common::VelocityType {
                                        base: -0.00893444,
                                        units: Some(common::VelocityUnits("km/s".to_string(),),),
                                    },
                                },
                                ManeuverParametersType {
                                    comment_list: vec![
                                        "Second maneuver: first station acquisition maneuver"
                                            .to_string(),
                                        "impulsive, thrust direction fixed in RTN frame"
                                            .to_string(),
                                    ],
                                    man_epoch_ignition: common::EpochType(
                                        "2021-06-05T18:59:21.0".to_string(),
                                    ),
                                    man_duration: common::DurationType {
                                        base: common::NonNegativeDouble(0.0,),
                                        units: Some(common::TimeUnits("s".to_string(),),),
                                    },
                                    man_delta_mass: common::DeltamassType {
                                        base: common::NegativeDouble(-1.469,),
                                        units: Some(common::MassUnits("kg".to_string(),),),
                                    },
                                    man_ref_frame: "RTN".to_string(),
                                    man_dv_1: common::VelocityType {
                                        base: 0.001015,
                                        units: Some(common::VelocityUnits("km/s".to_string(),),),
                                    },
                                    man_dv_2: common::VelocityType {
                                        base: -0.001873,
                                        units: Some(common::VelocityUnits("km/s".to_string(),),),
                                    },
                                    man_dv_3: common::VelocityType {
                                        base: 0.0,
                                        units: Some(common::VelocityUnits("km/s".to_string(),),),
                                    },
                                },
                            ],
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
