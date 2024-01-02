mod schema_opm
{
    pub mod xml_schema_types
    {
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OpmType
        {
            #[serde(rename = "header")] pub header : schema_common ::
            xml_schema_types :: OdmHeader, #[serde(rename = "body")] pub body
            : schema_common :: xml_schema_types :: OpmBody,
            #[serde(rename = "@id")] pub id : String,
            #[serde(rename = "@version")] pub version : String,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OpmBody
        {
            #[serde(rename = "segment")] pub segment : schema_common ::
            xml_schema_types :: OpmSegment,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OpmSegment
        {
            #[serde(rename = "metadata")] pub metadata : schema_common ::
            xml_schema_types :: OpmMetadata, #[serde(rename = "data")] pub
            data : schema_common :: xml_schema_types :: OpmData,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OpmMetadata
        {
            #[serde(rename = "COMMENT")] pub comment_list : Vec < String >,
            #[serde(rename = "OBJECT_NAME")] pub object_name : String,
            #[serde(rename = "OBJECT_ID")] pub object_id : String,
            #[serde(rename = "CENTER_NAME")] pub center_name : String,
            #[serde(rename = "REF_FRAME")] pub ref_frame : String,
            #[serde(rename = "REF_FRAME_EPOCH")] pub ref_frame_epoch : Option
            < schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "TIME_SYSTEM")] pub time_system : String,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OpmData
        {
            #[serde(rename = "COMMENT")] pub comment_list : Vec < String >,
            #[serde(rename = "stateVector")] pub state_vector : schema_common
            :: xml_schema_types :: StateVectorType,
            #[serde(rename = "keplerianElements")] pub keplerian_elements :
            Option < schema_common :: xml_schema_types ::
            KeplerianElementsType >, #[serde(rename = "spacecraftParameters")]
            pub spacecraft_parameters : Option < schema_common ::
            xml_schema_types :: SpacecraftParametersType >,
            #[serde(rename = "covarianceMatrix")] pub covariance_matrix :
            Option < schema_common :: xml_schema_types ::
            OpmCovarianceMatrixType >, #[serde(rename = "maneuverParameters")]
            pub maneuver_parameters_list : Vec < schema_common ::
            xml_schema_types :: ManeuverParametersType >,
            #[serde(rename = "userDefinedParameters")] pub
            user_defined_parameters : Option < schema_common ::
            xml_schema_types :: UserDefinedType >,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct KeplerianElementsType
        {
            #[serde(rename = "COMMENT")] pub comment_list : Vec < String >,
            #[serde(rename = "SEMI_MAJOR_AXIS")] pub semi_major_axis :
            schema_common :: xml_schema_types :: DistanceType,
            #[serde(rename = "ECCENTRICITY")] pub eccentricity : schema_common
            :: xml_schema_types :: NonNegativeDouble,
            #[serde(rename = "INCLINATION")] pub inclination : schema_common
            :: xml_schema_types :: InclinationType,
            #[serde(rename = "RA_OF_ASC_NODE")] pub ra_of_asc_node :
            schema_common :: xml_schema_types :: AngleType,
            #[serde(rename = "ARG_OF_PERICENTER")] pub arg_of_pericenter :
            schema_common :: xml_schema_types :: AngleType,
            #[serde(rename = "GM")] pub gm : schema_common :: xml_schema_types
            :: GmType,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct ManeuverParametersType
        {
            #[serde(rename = "COMMENT")] pub comment_list : Vec < String >,
            #[serde(rename = "MAN_EPOCH_IGNITION")] pub man_epoch_ignition :
            schema_common :: xml_schema_types :: EpochType,
            #[serde(rename = "MAN_DURATION")] pub man_duration : schema_common
            :: xml_schema_types :: DurationType,
            #[serde(rename = "MAN_DELTA_MASS")] pub man_delta_mass :
            schema_common :: xml_schema_types :: DeltamassType,
            #[serde(rename = "MAN_REF_FRAME")] pub man_ref_frame : String,
            #[serde(rename = "MAN_DV_1")] pub man_dv_1 : schema_common ::
            xml_schema_types :: VelocityType, #[serde(rename = "MAN_DV_2")]
            pub man_dv_2 : schema_common :: xml_schema_types :: VelocityType,
            #[serde(rename = "MAN_DV_3")] pub man_dv_3 : schema_common ::
            xml_schema_types :: VelocityType,
        }
    }
} pub use schema_opm :: * ;