mod schema_oem
{
    pub mod xml_schema_types
    {
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OemType
        {
            #[serde(rename = "header")] pub header : schema_common ::
            xml_schema_types :: OdmHeader, #[serde(rename = "body")] pub body
            : schema_common :: xml_schema_types :: OemBody,
            #[serde(rename = "@id")] pub id : String,
            #[serde(rename = "@version")] pub version : String,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OemBody
        {
            #[serde(rename = "segment")] pub segment_list : Vec <
            schema_common :: xml_schema_types :: OemSegment >,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OemSegment
        {
            #[serde(rename = "metadata")] pub metadata : schema_common ::
            xml_schema_types :: OemMetadata, #[serde(rename = "data")] pub
            data : schema_common :: xml_schema_types :: OemData,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OemMetadata
        {
            #[serde(rename = "COMMENT")] pub comment_list : Vec < String >,
            #[serde(rename = "OBJECT_NAME")] pub object_name : String,
            #[serde(rename = "OBJECT_ID")] pub object_id : String,
            #[serde(rename = "CENTER_NAME")] pub center_name : String,
            #[serde(rename = "REF_FRAME")] pub ref_frame : String,
            #[serde(rename = "REF_FRAME_EPOCH")] pub ref_frame_epoch : Option
            < schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "TIME_SYSTEM")] pub time_system : String,
            #[serde(rename = "START_TIME")] pub start_time : schema_common ::
            xml_schema_types :: EpochType,
            #[serde(rename = "USEABLE_START_TIME")] pub useable_start_time :
            Option < schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "USEABLE_STOP_TIME")] pub useable_stop_time :
            Option < schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "STOP_TIME")] pub stop_time : schema_common ::
            xml_schema_types :: EpochType, #[serde(rename = "INTERPOLATION")]
            pub interpolation : Option < String >,
            #[serde(rename = "INTERPOLATION_DEGREE")] pub interpolation_degree
            : Option < u64 >,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OemData
        {
            #[serde(rename = "COMMENT")] pub comment_list : Vec < String >,
            #[serde(rename = "stateVector")] pub state_vector_list : Vec <
            schema_common :: xml_schema_types :: StateVectorAccType >,
            #[serde(rename = "covarianceMatrix")] pub covariance_matrix_list :
            Vec < schema_common :: xml_schema_types :: OemCovarianceMatrixType
            >,
        }
    }
} pub use schema_oem :: * ;