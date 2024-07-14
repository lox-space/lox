/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Deserializers for XML CCSDS Navigation Data Message Combined Instantiation
//!
//! To deserialize an XML message:
//!
//! ```
//! # let xml = r#"<ndm xsi:noNamespaceSchemaLocation="https://sanaregistry.org/r/ndmxml/ndmxml-1.0-master.xsd">
//! # <MESSAGE_ID>bla</MESSAGE_ID>
//! # <COMMENT>asdfg</COMMENT>
//! # <omm id="CCSDS_OMM_VERS" version="2.0">
//! #     <header>
//! #         <CREATION_DATE/>
//! #         <ORIGINATOR/>
//! #     </header>
//! #     <body>
//! #         <segment>
//! #         </segment>
//! #     </body>
//! # </omm>
//! # <opm id="CCSDS_OPM_VERS" version="2.0">
//! # <header>
//! #    <CREATION_DATE>2004-281T17:26:06</CREATION_DATE>
//! #    <ORIGINATOR>me</ORIGINATOR>
//! # </header>
//! # <body>
//! #    <segment>
//! #       <metadata>
//! #          <OBJECT_NAME>Cassini</OBJECT_NAME>
//! #          <OBJECT_ID>1997-061A</OBJECT_ID>
//! #          <CENTER_NAME>Saturn</CENTER_NAME>
//! #          <REF_FRAME>IAU-Saturn</REF_FRAME>
//! #          <TIME_SYSTEM>UTC</TIME_SYSTEM>
//! #       </metadata>
//! #       <data>
//! #          <stateVector>
//! #             <COMMENT>this is a comment</COMMENT>
//! #             <EPOCH>2004-100T00:00:00</EPOCH>
//! #             <X>1</X>
//! #             <Y>1</Y>
//! #             <Z>1</Z>
//! #             <X_DOT>1</X_DOT>
//! #             <Y_DOT>1</Y_DOT>
//! #             <Z_DOT>1</Z_DOT>
//! #          </stateVector>
//! #          <spacecraftParameters>
//! #             <COMMENT>This is a COMMENT</COMMENT>
//! #             <MASS>100</MASS>
//! #             <SOLAR_RAD_AREA>2</SOLAR_RAD_AREA>
//! #             <SOLAR_RAD_COEFF>1</SOLAR_RAD_COEFF>
//! #             <DRAG_AREA units="m**2">2</DRAG_AREA>
//! #             <DRAG_COEFF>2.0</DRAG_COEFF>
//! #          </spacecraftParameters>
//! #          <maneuverParameters>
//! #             <COMMENT>This is a COMMENT</COMMENT>
//! #             <MAN_EPOCH_IGNITION>2004-125T00:00:00</MAN_EPOCH_IGNITION>
//! #             <MAN_DURATION units="s">0</MAN_DURATION>
//! #             <MAN_DELTA_MASS units="kg">-1</MAN_DELTA_MASS>
//! #             <MAN_REF_FRAME>GRC</MAN_REF_FRAME>
//! #             <MAN_DV_1>1</MAN_DV_1>
//! #             <MAN_DV_2>1</MAN_DV_2>
//! #             <MAN_DV_3>1</MAN_DV_3>
//! #          </maneuverParameters>
//! #       </data>
//! #    </segment>
//! # </body>
//! # </opm>
//! #
//! #
//! # <oem id="CCSDS_OEM_VERS" version="2.0">
//! # <header>
//! #    <CREATION_DATE>2004-281T17:26:06</CREATION_DATE>
//! #    <ORIGINATOR>me</ORIGINATOR>
//! # </header>
//! # <body>
//! #    <segment>
//! #       <metadata>
//! #          <OBJECT_NAME>Cassini</OBJECT_NAME>
//! #          <OBJECT_ID>1997-061A</OBJECT_ID>
//! #          <CENTER_NAME>Saturn</CENTER_NAME>
//! #          <REF_FRAME>IAU-Saturn</REF_FRAME>
//! #          <TIME_SYSTEM>UTC</TIME_SYSTEM>
//! #          <START_TIME>2004-100T00:00:00.000000</START_TIME>
//! #          <STOP_TIME>2004-100T01:00:00.000000</STOP_TIME>
//! #          <INTERPOLATION>Hermite</INTERPOLATION>
//! #          <INTERPOLATION_DEGREE>1</INTERPOLATION_DEGREE>
//! #       </metadata>
//! #       <data>
//! #          <stateVector>
//! #             <EPOCH>2004-100T00:00:00</EPOCH>
//! #             <X units="km">1</X>
//! #             <Y>1</Y>
//! #             <Z>1</Z>
//! #             <X_DOT units="km/s">1</X_DOT>
//! #             <Y_DOT>1</Y_DOT>
//! #             <Z_DOT>1</Z_DOT>
//! #          </stateVector>
//! #          <stateVector>
//! #             <EPOCH>2004-100T00:00:00</EPOCH>
//! #             <X>1</X>
//! #             <Y units="km">1</Y>
//! #             <Z>1</Z>
//! #             <X_DOT>1</X_DOT>
//! #             <Y_DOT units="km/s">1</Y_DOT>
//! #             <Z_DOT>1</Z_DOT>
//! #          </stateVector>
//! #          <stateVector>
//! #             <EPOCH>2004-100T00:00:00</EPOCH>
//! #             <X>1</X>
//! #             <Y>1</Y>
//! #             <Z units="km">1</Z>
//! #             <X_DOT>1</X_DOT>
//! #             <Y_DOT>1</Y_DOT>
//! #             <Z_DOT units="km/s">1</Z_DOT>
//! #          </stateVector>
//! #       </data>
//! #    </segment>
//! # </body>
//! # </oem>
//! # <omm id="CCSDS_OMM_VERS" version="2.0">
//! #     <header>
//! #         <CREATION_DATE/>
//! #         <ORIGINATOR/>
//! #     </header>
//! #     <body>
//! #         <segment>
//! #             <metadata>
//! #                 <OBJECT_NAME>NUSAT-13 (EMMY)</OBJECT_NAME>
//! #                 <OBJECT_ID>2020-079G</OBJECT_ID>
//! #                 <CENTER_NAME>EARTH</CENTER_NAME>
//! #                 <REF_FRAME>TEME</REF_FRAME>
//! #                 <TIME_SYSTEM>UTC</TIME_SYSTEM>
//! #                 <MEAN_ELEMENT_THEORY>SGP4</MEAN_ELEMENT_THEORY>
//! #             </metadata>
//! #             <data>
//! #                 <meanElements>
//! #                     <EPOCH>2020-12-04T13:30:01.539648</EPOCH>
//! #                     <MEAN_MOTION>15.31433655</MEAN_MOTION>
//! #                     <ECCENTRICITY>.0009574</ECCENTRICITY>
//! #                     <INCLINATION>97.2663</INCLINATION>
//! #                     <RA_OF_ASC_NODE>51.2167</RA_OF_ASC_NODE>
//! #                     <ARG_OF_PERICENTER>149.8567</ARG_OF_PERICENTER>
//! #                     <MEAN_ANOMALY>322.5146</MEAN_ANOMALY>
//! #                 </meanElements>
//! #                 <tleParameters>
//! #                     <EPHEMERIS_TYPE>0</EPHEMERIS_TYPE>
//! #                     <CLASSIFICATION_TYPE>U</CLASSIFICATION_TYPE>
//! #                     <NORAD_CAT_ID>46833</NORAD_CAT_ID>
//! #                     <ELEMENT_SET_NO>999</ELEMENT_SET_NO>
//! #                     <REV_AT_EPOCH>434</REV_AT_EPOCH>
//! #                     <BSTAR>.14401E-3</BSTAR>
//! #                     <MEAN_MOTION_DOT>4.301E-5</MEAN_MOTION_DOT>
//! #                     <MEAN_MOTION_DDOT>0</MEAN_MOTION_DDOT>
//! #                 </tleParameters>
//! #             </data>
//! #         </segment>
//! #     </body>
//! # </omm>
//! #
//! # </ndm>"#;
//! #
//! # use lox_io::ndm::ndm_ci::NdmType;
//! use lox_io::ndm::xml::FromXmlStr;
//!
//! let message = NdmType::from_xml_str(xml).unwrap();
//! ```

// This file is partially generated with xml-schema-derive from the XSD schema
// published by CCSDS. Adaptations have been made to simplify the types or
// allow to simplify the implementation of the KVN parser.

use serde;

use super::{ocm, oem, omm, opm};

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde()]
#[allow(clippy::large_enum_variant)]
pub enum NdmChildChoice {
    #[serde(rename = "ocm")]
    Ocm(ocm::OcmType),

    #[serde(rename = "oem")]
    Oem(oem::OemType),

    #[serde(rename = "omm")]
    Omm(omm::OmmType),

    #[serde(rename = "opm")]
    Opm(opm::OpmType),
}

/// Combined instantiation type. Currently does not support AEM, APM, CDM,
/// RDM, TDM
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct NdmType {
    #[serde(rename = "MESSAGE_ID")]
    pub message_id: Option<String>,
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,

    #[serde(rename = "$value")]
    pub child_list: Vec<NdmChildChoice>,
}

impl crate::ndm::xml::FromXmlStr<'_> for NdmType {}

#[cfg(test)]
mod test {
    use crate::ndm::xml::FromXmlStr;

    use super::super::common;
    use super::*;

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
<opm id="CCSDS_OPM_VERS" version="2.0">
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
      </metadata>
      <data>
         <COMMENT>final COMMENT, I think</COMMENT>
         <stateVector>
            <EPOCH>2004-100T00:00:00Z</EPOCH>
            <X>1</X>
            <Y>1</Y>
            <Z>1</Z>
            <X_DOT>1</X_DOT>
            <Y_DOT>1</Y_DOT>
            <Z_DOT>1</Z_DOT>
         </stateVector>
         <keplerianElements>
            <SEMI_MAJOR_AXIS units="km">1</SEMI_MAJOR_AXIS>
            <ECCENTRICITY>0</ECCENTRICITY>
            <INCLINATION units="deg">45</INCLINATION>
            <RA_OF_ASC_NODE units="deg">0</RA_OF_ASC_NODE>
            <ARG_OF_PERICENTER units="deg">15</ARG_OF_PERICENTER>
            <TRUE_ANOMALY units="deg">15</TRUE_ANOMALY>
            <GM units="km**3/s**2">398644</GM>
         </keplerianElements>
         <spacecraftParameters>
            <MASS units="kg">100</MASS>
            <SOLAR_RAD_AREA units="m**2">2</SOLAR_RAD_AREA>
            <SOLAR_RAD_COEFF>1</SOLAR_RAD_COEFF>
            <DRAG_AREA units="m**2">2</DRAG_AREA>
            <DRAG_COEFF>2.0</DRAG_COEFF>
         </spacecraftParameters>
         <maneuverParameters>
            <MAN_EPOCH_IGNITION>2004-125T00:00:00Z</MAN_EPOCH_IGNITION>
            <MAN_DURATION>0</MAN_DURATION>
            <MAN_DELTA_MASS>-1</MAN_DELTA_MASS>
            <MAN_REF_FRAME>GRC</MAN_REF_FRAME>
            <MAN_DV_1 units="km/s">1</MAN_DV_1>
            <MAN_DV_2 units="km/s">1</MAN_DV_2>
            <MAN_DV_3 units="km/s">1</MAN_DV_3>
         </maneuverParameters>
      </data>
   </segment>
</body>
</opm>

<opm id="CCSDS_OPM_VERS" version="2.0">
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
      </metadata>
      <data>
         <stateVector>
            <COMMENT>this is a comment</COMMENT>
            <EPOCH>2004-100T00:00:00</EPOCH>
            <X>1</X>
            <Y>1</Y>
            <Z>1</Z>
            <X_DOT>1</X_DOT>
            <Y_DOT>1</Y_DOT>
            <Z_DOT>1</Z_DOT>
         </stateVector>
         <spacecraftParameters>
            <COMMENT>This is a COMMENT</COMMENT>
            <MASS>100</MASS>
            <SOLAR_RAD_AREA>2</SOLAR_RAD_AREA>
            <SOLAR_RAD_COEFF>1</SOLAR_RAD_COEFF>
            <DRAG_AREA units="m**2">2</DRAG_AREA>
            <DRAG_COEFF>2.0</DRAG_COEFF>
         </spacecraftParameters>
         <maneuverParameters>
            <COMMENT>This is a COMMENT</COMMENT>
            <MAN_EPOCH_IGNITION>2004-125T00:00:00</MAN_EPOCH_IGNITION>
            <MAN_DURATION units="s">0</MAN_DURATION>
            <MAN_DELTA_MASS units="kg">-1</MAN_DELTA_MASS>
            <MAN_REF_FRAME>GRC</MAN_REF_FRAME>
            <MAN_DV_1>1</MAN_DV_1>
            <MAN_DV_2>1</MAN_DV_2>
            <MAN_DV_3>1</MAN_DV_3>
         </maneuverParameters>
      </data>
   </segment>
</body>
</opm>


<oem id="CCSDS_OEM_VERS" version="2.0">
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
</oem>
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

        let message = NdmType::from_xml_str(xml).unwrap();

        assert_eq!(
            message,
            NdmType {
                message_id: Some("bla".to_string()),
                comment_list: vec!["asdfg".to_string()],
                child_list: vec![
                    NdmChildChoice::Omm(omm::OmmType {
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
                                            base: 0.0,
                                            units: None,
                                        },
                                        ra_of_asc_node: common::AngleType {
                                            base: 0.0,
                                            units: None,
                                        },
                                        arg_of_pericenter: common::AngleType {
                                            base: 0.0,
                                            units: None,
                                        },
                                        mean_anomaly: common::AngleType {
                                            base: 0.0,
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
                        id: Some("CCSDS_OMM_VERS".to_string()),
                        version: "2.0".to_string(),
                    }),
                    NdmChildChoice::Omm(omm::OmmType {
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
                                            base: 97.409,
                                            units: None,
                                        },
                                        ra_of_asc_node: common::AngleType {
                                            base: 71.7453,
                                            units: None,
                                        },
                                        arg_of_pericenter: common::AngleType {
                                            base: 193.9419,
                                            units: None,
                                        },
                                        mean_anomaly: common::AngleType {
                                            base: 272.1492,
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
                        id: Some("CCSDS_OMM_VERS".to_string()),
                        version: "2.0".to_string(),
                    }),
                    NdmChildChoice::Opm(opm::OpmType {
                        header: common::OdmHeader {
                            comment_list: vec![],
                            classification_list: vec![],
                            creation_date: common::EpochType("2004-281T17:26:06".to_string()),
                            originator: "me".to_string(),
                            message_id: None,
                        },
                        body: opm::OpmBody {
                            segment: opm::OpmSegment {
                                metadata: opm::OpmMetadata {
                                    comment_list: vec![],
                                    object_name: "Cassini".to_string(),
                                    object_id: "1997-061A".to_string(),
                                    center_name: "Saturn".to_string(),
                                    ref_frame: "IAU-Saturn".to_string(),
                                    ref_frame_epoch: None,
                                    time_system: "UTC".to_string(),
                                },
                                data: opm::OpmData {
                                    comment_list: vec!["final COMMENT, I think".to_string()],
                                    state_vector: common::StateVectorType {
                                        comment_list: vec![],
                                        epoch: common::EpochType("2004-100T00:00:00Z".to_string()),
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
                                            units: None,
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
                                            units: None,
                                        },
                                    },
                                    keplerian_elements: Some(opm::KeplerianElementsType {
                                        comment_list: vec![],
                                        semi_major_axis: common::DistanceType {
                                            base: 1.0,
                                            units: Some(common::PositionUnits("km".to_string())),
                                        },
                                        eccentricity: common::NonNegativeDouble(0.0,),
                                        inclination: common::InclinationType {
                                            base: 45.0,
                                            units: Some(common::AngleUnits("deg".to_string())),
                                        },
                                        ra_of_asc_node: common::AngleType {
                                            base: 0.0,
                                            units: Some(common::AngleUnits("deg".to_string())),
                                        },
                                        arg_of_pericenter: common::AngleType {
                                            base: 15.0,
                                            units: Some(common::AngleUnits("deg".to_string())),
                                        },
                                        true_anomaly: Some(common::AngleType {
                                            base: 15.0,
                                            units: Some(common::AngleUnits("deg".to_string())),
                                        },),
                                        mean_anomaly: None,
                                        gm: common::GmType {
                                            base: common::PositiveDouble(398644.0,),
                                            units: Some(common::GmUnits("km**3/s**2".to_string())),
                                        },
                                    },),
                                    spacecraft_parameters: Some(common::SpacecraftParametersType {
                                        comment_list: vec![],
                                        mass: Some(common::MassType {
                                            base: common::NonNegativeDouble(100.0,),
                                            units: Some(common::MassUnits("kg".to_string())),
                                        },),
                                        solar_rad_area: Some(common::AreaType {
                                            base: common::NonNegativeDouble(2.0,),
                                            units: Some(common::AreaUnits("m**2".to_string())),
                                        },),
                                        solar_rad_coeff: Some(common::NonNegativeDouble(1.0,)),
                                        drag_area: Some(common::AreaType {
                                            base: common::NonNegativeDouble(2.0,),
                                            units: Some(common::AreaUnits("m**2".to_string())),
                                        },),
                                        drag_coeff: Some(common::NonNegativeDouble(2.0,)),
                                    },),
                                    covariance_matrix: None,
                                    maneuver_parameters_list: vec![opm::ManeuverParametersType {
                                        comment_list: vec![],
                                        man_epoch_ignition: common::EpochType(
                                            "2004-125T00:00:00Z".to_string(),
                                        ),
                                        man_duration: common::DurationType {
                                            base: common::NonNegativeDouble(0.0,),
                                            units: None,
                                        },
                                        man_delta_mass: common::DeltamassType {
                                            base: common::NegativeDouble(-1.0,),
                                            units: None,
                                        },
                                        man_ref_frame: "GRC".to_string(),
                                        man_dv_1: common::VelocityType {
                                            base: 1.0,
                                            units: Some(common::VelocityUnits("km/s".to_string())),
                                        },
                                        man_dv_2: common::VelocityType {
                                            base: 1.0,
                                            units: Some(common::VelocityUnits("km/s".to_string())),
                                        },
                                        man_dv_3: common::VelocityType {
                                            base: 1.0,
                                            units: Some(common::VelocityUnits("km/s".to_string())),
                                        },
                                    },],
                                    user_defined_parameters: None,
                                },
                            },
                        },
                        id: Some("CCSDS_OPM_VERS".to_string()),
                        version: "2.0".to_string(),
                    },),
                    NdmChildChoice::Opm(opm::OpmType {
                        header: common::OdmHeader {
                            comment_list: vec![],
                            classification_list: vec![],
                            creation_date: common::EpochType("2004-281T17:26:06".to_string()),
                            originator: "me".to_string(),
                            message_id: None,
                        },
                        body: opm::OpmBody {
                            segment: opm::OpmSegment {
                                metadata: opm::OpmMetadata {
                                    comment_list: vec![],
                                    object_name: "Cassini".to_string(),
                                    object_id: "1997-061A".to_string(),
                                    center_name: "Saturn".to_string(),
                                    ref_frame: "IAU-Saturn".to_string(),
                                    ref_frame_epoch: None,
                                    time_system: "UTC".to_string(),
                                },
                                data: opm::OpmData {
                                    comment_list: vec![],
                                    state_vector: common::StateVectorType {
                                        comment_list: vec!["this is a comment".to_string()],
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
                                            units: None,
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
                                            units: None,
                                        },
                                    },
                                    keplerian_elements: None,
                                    spacecraft_parameters: Some(common::SpacecraftParametersType {
                                        comment_list: vec!["This is a COMMENT".to_string()],
                                        mass: Some(common::MassType {
                                            base: common::NonNegativeDouble(100.0,),
                                            units: None,
                                        },),
                                        solar_rad_area: Some(common::AreaType {
                                            base: common::NonNegativeDouble(2.0,),
                                            units: None,
                                        },),
                                        solar_rad_coeff: Some(common::NonNegativeDouble(1.0,)),
                                        drag_area: Some(common::AreaType {
                                            base: common::NonNegativeDouble(2.0,),
                                            units: Some(common::AreaUnits("m**2".to_string())),
                                        },),
                                        drag_coeff: Some(common::NonNegativeDouble(2.0,)),
                                    },),
                                    covariance_matrix: None,
                                    maneuver_parameters_list: vec![opm::ManeuverParametersType {
                                        comment_list: vec!["This is a COMMENT".to_string()],
                                        man_epoch_ignition: common::EpochType(
                                            "2004-125T00:00:00".to_string(),
                                        ),
                                        man_duration: common::DurationType {
                                            base: common::NonNegativeDouble(0.0,),
                                            units: Some(common::TimeUnits("s".to_string())),
                                        },
                                        man_delta_mass: common::DeltamassType {
                                            base: common::NegativeDouble(-1.0,),
                                            units: Some(common::MassUnits("kg".to_string())),
                                        },
                                        man_ref_frame: "GRC".to_string(),
                                        man_dv_1: common::VelocityType {
                                            base: 1.0,
                                            units: None,
                                        },
                                        man_dv_2: common::VelocityType {
                                            base: 1.0,
                                            units: None,
                                        },
                                        man_dv_3: common::VelocityType {
                                            base: 1.0,
                                            units: None,
                                        },
                                    },],
                                    user_defined_parameters: None,
                                },
                            },
                        },
                        id: Some("CCSDS_OPM_VERS".to_string()),
                        version: "2.0".to_string(),
                    },),
                    NdmChildChoice::Oem(oem::OemType {
                        header: common::OdmHeader {
                            comment_list: vec![],
                            classification_list: vec![],
                            creation_date: common::EpochType("2004-281T17:26:06".to_string()),
                            originator: "me".to_string(),
                            message_id: None,
                        },
                        body: oem::OemBody {
                            segment_list: vec![oem::OemSegment {
                                metadata: oem::OemMetadata {
                                    comment_list: vec![],
                                    object_name: "Cassini".to_string(),
                                    object_id: "1997-061A".to_string(),
                                    center_name: "Saturn".to_string(),
                                    ref_frame: "IAU-Saturn".to_string(),
                                    ref_frame_epoch: None,
                                    time_system: "UTC".to_string(),
                                    start_time: common::EpochType(
                                        "2004-100T00:00:00.000000".to_string(),
                                    ),
                                    useable_start_time: None,
                                    useable_stop_time: None,
                                    stop_time: common::EpochType(
                                        "2004-100T01:00:00.000000".to_string(),
                                    ),
                                    interpolation: Some("Hermite".to_string()),
                                    interpolation_degree: Some(1,),
                                },
                                data: oem::OemData {
                                    comment_list: vec![],
                                    state_vector_list: vec![
                                        common::StateVectorAccType {
                                            epoch: common::EpochType(
                                                "2004-100T00:00:00".to_string(),
                                            ),
                                            x: common::PositionType {
                                                base: 1.0,
                                                units: Some(common::PositionUnits(
                                                    "km".to_string(),
                                                )),
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
                                                units: Some(common::VelocityUnits(
                                                    "km/s".to_string(),
                                                )),
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
                                            epoch: common::EpochType(
                                                "2004-100T00:00:00".to_string(),
                                            ),
                                            x: common::PositionType {
                                                base: 1.0,
                                                units: None,
                                            },
                                            y: common::PositionType {
                                                base: 1.0,
                                                units: Some(common::PositionUnits(
                                                    "km".to_string(),
                                                )),
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
                                                units: Some(common::VelocityUnits(
                                                    "km/s".to_string(),
                                                )),
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
                                            epoch: common::EpochType(
                                                "2004-100T00:00:00".to_string(),
                                            ),
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
                                                units: Some(common::PositionUnits(
                                                    "km".to_string(),
                                                )),
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
                                                units: Some(common::VelocityUnits(
                                                    "km/s".to_string(),
                                                )),
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
                    },),
                    NdmChildChoice::Omm(omm::OmmType {
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
                                            base: 97.2663,
                                            units: None,
                                        },
                                        ra_of_asc_node: common::AngleType {
                                            base: 51.2167,
                                            units: None,
                                        },
                                        arg_of_pericenter: common::AngleType {
                                            base: 149.8567,
                                            units: None,
                                        },
                                        mean_anomaly: common::AngleType {
                                            base: 322.5146,
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
                        id: Some("CCSDS_OMM_VERS".to_string()),
                        version: "2.0".to_string(),
                    }),
                    NdmChildChoice::Omm(omm::OmmType {
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
                                            base: 97.2671,
                                            units: None,
                                        },
                                        ra_of_asc_node: common::AngleType {
                                            base: 51.3486,
                                            units: None,
                                        },
                                        arg_of_pericenter: common::AngleType {
                                            base: 160.8608,
                                            units: None,
                                        },
                                        mean_anomaly: common::AngleType {
                                            base: 302.2789,
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
                        id: Some("CCSDS_OMM_VERS".to_string()),
                        version: "2.0".to_string(),
                    }),
                    NdmChildChoice::Omm(omm::OmmType {
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
                                            base: 97.2666,
                                            units: None,
                                        },
                                        ra_of_asc_node: common::AngleType {
                                            base: 51.2301,
                                            units: None,
                                        },
                                        arg_of_pericenter: common::AngleType {
                                            base: 167.2057,
                                            units: None,
                                        },
                                        mean_anomaly: common::AngleType {
                                            base: 304.5569,
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
                        id: Some("CCSDS_OMM_VERS".to_string()),
                        version: "2.0".to_string(),
                    }),
                ],
            },
        );
    }
}
