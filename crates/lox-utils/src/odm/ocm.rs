mod schema_ocm
{
    pub mod xml_schema_types
    {
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OcmType
        {
            #[serde(rename = "header")] pub header : schema_common ::
            xml_schema_types :: OdmHeader, #[serde(rename = "body")] pub body
            : schema_common :: xml_schema_types :: OcmBody,
            #[serde(rename = "@id")] pub id : String,
            #[serde(rename = "@version")] pub version : String,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OcmBody
        {
            #[serde(rename = "segment")] pub segment : schema_common ::
            xml_schema_types :: OcmSegment,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OcmSegment
        {
            #[serde(rename = "metadata")] pub metadata : schema_common ::
            xml_schema_types :: OcmMetadata, #[serde(rename = "data")] pub
            data : schema_common :: xml_schema_types :: OcmData,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OcmMetadata
        {
            #[serde(rename = "COMMENT")] pub comment_list : Vec < String >,
            #[serde(rename = "OBJECT_NAME")] pub object_name : Option < String
            >, #[serde(rename = "INTERNATIONAL_DESIGNATOR")] pub
            international_designator : Option < String >,
            #[serde(rename = "CATALOG_NAME")] pub catalog_name : Option <
            String >, #[serde(rename = "OBJECT_DESIGNATOR")] pub
            object_designator : Option < String >,
            #[serde(rename = "ALTERNATE_NAMES")] pub alternate_names : Option
            < String >, #[serde(rename = "ORIGINATOR_POC")] pub originator_poc
            : Option < String >, #[serde(rename = "ORIGINATOR_POSITION")] pub
            originator_position : Option < String >,
            #[serde(rename = "ORIGINATOR_PHONE")] pub originator_phone :
            Option < String >, #[serde(rename = "ORIGINATOR_EMAIL")] pub
            originator_email : Option < String >,
            #[serde(rename = "ORIGINATOR_ADDRESS")] pub originator_address :
            Option < String >, #[serde(rename = "TECH_ORG")] pub tech_org :
            Option < String >, #[serde(rename = "TECH_POC")] pub tech_poc :
            Option < String >, #[serde(rename = "TECH_POSITION")] pub
            tech_position : Option < String >, #[serde(rename = "TECH_PHONE")]
            pub tech_phone : Option < String >,
            #[serde(rename = "TECH_EMAIL")] pub tech_email : Option < String
            >, #[serde(rename = "TECH_ADDRESS")] pub tech_address : Option <
            String >, #[serde(rename = "PREVIOUS_MESSAGE_ID")] pub
            previous_message_id : Option < String >,
            #[serde(rename = "NEXT_MESSAGE_ID")] pub next_message_id : Option
            < String >, #[serde(rename = "ADM_MSG_LINK")] pub adm_msg_link :
            Option < String >, #[serde(rename = "CDM_MSG_LINK")] pub
            cdm_msg_link : Option < String >,
            #[serde(rename = "PRM_MSG_LINK")] pub prm_msg_link : Option <
            String >, #[serde(rename = "RDM_MSG_LINK")] pub rdm_msg_link :
            Option < String >, #[serde(rename = "TDM_MSG_LINK")] pub
            tdm_msg_link : Option < String >, #[serde(rename = "OPERATOR")]
            pub operator : Option < String >, #[serde(rename = "OWNER")] pub
            owner : Option < String >, #[serde(rename = "COUNTRY")] pub
            country : Option < String >, #[serde(rename = "CONSTELLATION")]
            pub constellation : Option < String >,
            #[serde(rename = "OBJECT_TYPE")] pub object_type : Option <
            schema_common :: xml_schema_types :: ObjectDescriptionType >,
            #[serde(rename = "TIME_SYSTEM")] pub time_system : String,
            #[serde(rename = "EPOCH_TZERO")] pub epoch_tzero : schema_common
            :: xml_schema_types :: EpochType, #[serde(rename = "OPS_STATUS")]
            pub ops_status : Option < String >,
            #[serde(rename = "ORBIT_CATEGORY")] pub orbit_category : Option <
            String >, #[serde(rename = "OCM_DATA_ELEMENTS")] pub
            ocm_data_elements : Option < String >,
            #[serde(rename = "SCLK_OFFSET_AT_EPOCH")] pub sclk_offset_at_epoch
            : Option < schema_common :: xml_schema_types :: TimeOffsetType >,
            #[serde(rename = "SCLK_SEC_PER_SI_SEC")] pub sclk_sec_per_si_sec :
            Option < schema_common :: xml_schema_types :: DurationType >,
            #[serde(rename = "PREVIOUS_MESSAGE_EPOCH")] pub
            previous_message_epoch : Option < schema_common ::
            xml_schema_types :: EpochType >,
            #[serde(rename = "NEXT_MESSAGE_EPOCH")] pub next_message_epoch :
            Option < schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "START_TIME")] pub start_time : Option <
            schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "STOP_TIME")] pub stop_time : Option <
            schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "TIME_SPAN")] pub time_span : Option <
            schema_common :: xml_schema_types :: OcmDayIntervalType >,
            #[serde(rename = "TAIMUTC_AT_TZERO")] pub taimutc_at_tzero :
            Option < schema_common :: xml_schema_types :: TimeOffsetType >,
            #[serde(rename = "NEXT_LEAP_EPOCH")] pub next_leap_epoch : Option
            < schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "NEXT_LEAP_TAIMUTC")] pub next_leap_taimutc :
            Option < schema_common :: xml_schema_types :: TimeOffsetType >,
            #[serde(rename = "UT1MUTC_AT_TZERO")] pub ut1mutc_at_tzero :
            Option < schema_common :: xml_schema_types :: TimeOffsetType >,
            #[serde(rename = "EOP_SOURCE")] pub eop_source : Option < String
            >, #[serde(rename = "INTERP_METHOD_EOP")] pub interp_method_eop :
            Option < String >, #[serde(rename = "CELESTIAL_SOURCE")] pub
            celestial_source : Option < String >,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OcmData
        {
            #[serde(rename = "traj")] pub traj_list : Vec < schema_common ::
            xml_schema_types :: OcmTrajStateType >, #[serde(rename = "phys")]
            pub phys : Option < schema_common :: xml_schema_types ::
            OcmPhysicalDescriptionType >, #[serde(rename = "cov")] pub
            cov_list : Vec < schema_common :: xml_schema_types ::
            OcmCovarianceMatrixType >, #[serde(rename = "man")] pub man_list :
            Vec < schema_common :: xml_schema_types ::
            OcmManeuverParametersType >, #[serde(rename = "pert")] pub pert :
            Option < schema_common :: xml_schema_types :: OcmPerturbationsType
            >, #[serde(rename = "od")] pub od : Option < schema_common ::
            xml_schema_types :: OcmOdParametersType >,
            #[serde(rename = "user")] pub user : Option < schema_common ::
            xml_schema_types :: UserDefinedType >,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OcmTrajStateType
        {
            #[serde(rename = "COMMENT")] pub comment_list : Vec < String >,
            #[serde(rename = "TRAJ_ID")] pub traj_id : Option < String >,
            #[serde(rename = "TRAJ_PREV_ID")] pub traj_prev_id : Option <
            String >, #[serde(rename = "TRAJ_NEXT_ID")] pub traj_next_id :
            Option < String >, #[serde(rename = "TRAJ_BASIS")] pub traj_basis
            : Option < schema_common :: xml_schema_types :: TrajBasisType >,
            #[serde(rename = "TRAJ_BASIS_ID")] pub traj_basis_id : Option <
            String >, #[serde(rename = "INTERPOLATION")] pub interpolation :
            Option < String >, #[serde(rename = "INTERPOLATION_DEGREE")] pub
            interpolation_degree : Option < u64 >,
            #[serde(rename = "PROPAGATOR")] pub propagator : Option < String
            >, #[serde(rename = "CENTER_NAME")] pub center_name : String,
            #[serde(rename = "TRAJ_REF_FRAME")] pub traj_ref_frame : String,
            #[serde(rename = "TRAJ_FRAME_EPOCH")] pub traj_frame_epoch :
            Option < schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "USEABLE_START_TIME")] pub useable_start_time :
            Option < schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "USEABLE_STOP_TIME")] pub useable_stop_time :
            Option < schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "ORB_REVNUM")] pub orb_revnum : Option <
            schema_common :: xml_schema_types :: NonNegativeDouble >,
            #[serde(rename = "ORB_REVNUM_BASIS")] pub orb_revnum_basis :
            Option < schema_common :: xml_schema_types :: RevNumBasisType >,
            #[serde(rename = "TRAJ_TYPE")] pub traj_type : String,
            #[serde(rename = "ORB_AVERAGING")] pub orb_averaging : Option <
            String >, #[serde(rename = "TRAJ_UNITS")] pub traj_units : Option
            < String >, #[serde(rename = "trajLine")] pub traj_line_list : Vec
            < String >,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OcmPhysicalDescriptionType
        {
            #[serde(rename = "COMMENT")] pub comment_list : Vec < String >,
            #[serde(rename = "MANUFACTURER")] pub manufacturer : Option <
            String >, #[serde(rename = "BUS_MODEL")] pub bus_model : Option <
            String >, #[serde(rename = "DOCKED_WITH")] pub docked_with :
            Option < String >, #[serde(rename = "DRAG_CONST_AREA")] pub
            drag_const_area : Option < schema_common :: xml_schema_types ::
            AreaType >, #[serde(rename = "DRAG_COEFF_NOM")] pub drag_coeff_nom
            : Option < schema_common :: xml_schema_types :: PositiveDouble >,
            #[serde(rename = "DRAG_UNCERTAINTY")] pub drag_uncertainty :
            Option < schema_common :: xml_schema_types :: PercentageType >,
            #[serde(rename = "INITIAL_WET_MASS")] pub initial_wet_mass :
            Option < schema_common :: xml_schema_types :: MassType >,
            #[serde(rename = "WET_MASS")] pub wet_mass : Option <
            schema_common :: xml_schema_types :: MassType >,
            #[serde(rename = "DRY_MASS")] pub dry_mass : Option <
            schema_common :: xml_schema_types :: MassType >,
            #[serde(rename = "OEB_PARENT_FRAME")] pub oeb_parent_frame :
            Option < String >, #[serde(rename = "OEB_PARENT_FRAME_EPOCH")] pub
            oeb_parent_frame_epoch : Option < schema_common ::
            xml_schema_types :: EpochType >, #[serde(rename = "OEB_Q1")] pub
            oeb_q1 : Option < f64 >, #[serde(rename = "OEB_Q2")] pub oeb_q2 :
            Option < f64 >, #[serde(rename = "OEB_Q3")] pub oeb_q3 : Option <
            f64 >, #[serde(rename = "OEB_QC")] pub oeb_qc : Option < f64 >,
            #[serde(rename = "OEB_MAX")] pub oeb_max : Option < schema_common
            :: xml_schema_types :: OcmLengthType >,
            #[serde(rename = "OEB_INT")] pub oeb_int : Option < schema_common
            :: xml_schema_types :: OcmLengthType >,
            #[serde(rename = "OEB_MIN")] pub oeb_min : Option < schema_common
            :: xml_schema_types :: OcmLengthType >,
            #[serde(rename = "AREA_ALONG_OEB_MAX")] pub area_along_oeb_max :
            Option < schema_common :: xml_schema_types :: AreaType >,
            #[serde(rename = "AREA_ALONG_OEB_INT")] pub area_along_oeb_int :
            Option < schema_common :: xml_schema_types :: AreaType >,
            #[serde(rename = "AREA_ALONG_OEB_MIN")] pub area_along_oeb_min :
            Option < schema_common :: xml_schema_types :: AreaType >,
            #[serde(rename = "AREA_MIN_FOR_PC")] pub area_min_for_pc : Option
            < schema_common :: xml_schema_types :: AreaType >,
            #[serde(rename = "AREA_MAX_FOR_PC")] pub area_max_for_pc : Option
            < schema_common :: xml_schema_types :: AreaType >,
            #[serde(rename = "AREA_TYP_FOR_PC")] pub area_typ_for_pc : Option
            < schema_common :: xml_schema_types :: AreaType >,
            #[serde(rename = "RCS")] pub rcs : Option < schema_common ::
            xml_schema_types :: AreaType >, #[serde(rename = "RCS_MIN")] pub
            rcs_min : Option < schema_common :: xml_schema_types :: AreaType
            >, #[serde(rename = "RCS_MAX")] pub rcs_max : Option <
            schema_common :: xml_schema_types :: AreaType >,
            #[serde(rename = "SRP_CONST_AREA")] pub srp_const_area : Option <
            schema_common :: xml_schema_types :: AreaType >,
            #[serde(rename = "SOLAR_RAD_COEFF")] pub solar_rad_coeff : Option
            < f64 >, #[serde(rename = "SOLAR_RAD_UNCERTAINTY")] pub
            solar_rad_uncertainty : Option < schema_common :: xml_schema_types
            :: PercentageType >, #[serde(rename = "VM_ABSOLUTE")] pub
            vm_absolute : Option < f64 >, #[serde(rename = "VM_APPARENT_MIN")]
            pub vm_apparent_min : Option < f64 >,
            #[serde(rename = "VM_APPARENT")] pub vm_apparent : Option < f64 >,
            #[serde(rename = "VM_APPARENT_MAX")] pub vm_apparent_max : Option
            < f64 >, #[serde(rename = "REFLECTANCE")] pub reflectance : Option
            < schema_common :: xml_schema_types :: ProbabilityType >,
            #[serde(rename = "ATT_CONTROL_MODE")] pub att_control_mode :
            Option < String >, #[serde(rename = "ATT_ACTUATOR_TYPE")] pub
            att_actuator_type : Option < String >,
            #[serde(rename = "ATT_KNOWLEDGE")] pub att_knowledge : Option <
            schema_common :: xml_schema_types :: AngleType >,
            #[serde(rename = "ATT_CONTROL")] pub att_control : Option <
            schema_common :: xml_schema_types :: AngleType >,
            #[serde(rename = "ATT_POINTING")] pub att_pointing : Option <
            schema_common :: xml_schema_types :: AngleType >,
            #[serde(rename = "AVG_MANEUVER_FREQ")] pub avg_maneuver_freq :
            Option < schema_common :: xml_schema_types :: ManeuverFreqType >,
            #[serde(rename = "MAX_THRUST")] pub max_thrust : Option <
            schema_common :: xml_schema_types :: ThrustType >,
            #[serde(rename = "DV_BOL")] pub dv_bol : Option < schema_common ::
            xml_schema_types :: VelocityType >,
            #[serde(rename = "DV_REMAINING")] pub dv_remaining : Option <
            schema_common :: xml_schema_types :: VelocityType >,
            #[serde(rename = "IXX")] pub ixx : Option < schema_common ::
            xml_schema_types :: MomentType >, #[serde(rename = "IYY")] pub iyy
            : Option < schema_common :: xml_schema_types :: MomentType >,
            #[serde(rename = "IZZ")] pub izz : Option < schema_common ::
            xml_schema_types :: MomentType >, #[serde(rename = "IXY")] pub ixy
            : Option < schema_common :: xml_schema_types :: MomentType >,
            #[serde(rename = "IXZ")] pub ixz : Option < schema_common ::
            xml_schema_types :: MomentType >, #[serde(rename = "IYZ")] pub iyz
            : Option < schema_common :: xml_schema_types :: MomentType >,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OcmCovarianceMatrixType
        {
            #[serde(rename = "COMMENT")] pub comment_list : Vec < String >,
            #[serde(rename = "COV_ID")] pub cov_id : Option < String >,
            #[serde(rename = "COV_PREV_ID")] pub cov_prev_id : Option < String
            >, #[serde(rename = "COV_NEXT_ID")] pub cov_next_id : Option <
            String >, #[serde(rename = "COV_BASIS")] pub cov_basis : Option <
            schema_common :: xml_schema_types :: CovBasisType >,
            #[serde(rename = "COV_BASIS_ID")] pub cov_basis_id : Option <
            String >, #[serde(rename = "COV_REF_FRAME")] pub cov_ref_frame :
            String, #[serde(rename = "COV_FRAME_EPOCH")] pub cov_frame_epoch :
            Option < schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "COV_SCALE_MIN")] pub cov_scale_min : Option <
            f64 >, #[serde(rename = "COV_SCALE_MAX")] pub cov_scale_max :
            Option < f64 >, #[serde(rename = "COV_CONFIDENCE")] pub
            cov_confidence : Option < schema_common :: xml_schema_types ::
            PercentageType >, #[serde(rename = "COV_TYPE")] pub cov_type :
            String, #[serde(rename = "COV_ORDERING")] pub cov_ordering :
            schema_common :: xml_schema_types :: CovOrderType,
            #[serde(rename = "COV_UNITS")] pub cov_units : Option < String >,
            #[serde(rename = "covLine")] pub cov_line_list : Vec < String >,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OcmManeuverParametersType
        {
            #[serde(rename = "COMMENT")] pub comment_list : Vec < String >,
            #[serde(rename = "MAN_ID")] pub man_id : String,
            #[serde(rename = "MAN_PREV_ID")] pub man_prev_id : Option < String
            >, #[serde(rename = "MAN_NEXT_ID")] pub man_next_id : Option <
            String >, #[serde(rename = "MAN_BASIS")] pub man_basis : Option <
            schema_common :: xml_schema_types :: ManBasisType >,
            #[serde(rename = "MAN_BASIS_ID")] pub man_basis_id : Option <
            String >, #[serde(rename = "MAN_DEVICE_ID")] pub man_device_id :
            String, #[serde(rename = "MAN_PREV_EPOCH")] pub man_prev_epoch :
            Option < schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "MAN_NEXT_EPOCH")] pub man_next_epoch : Option <
            schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "MAN_PURPOSE")] pub man_purpose : Option < String
            >, #[serde(rename = "MAN_PRED_SOURCE")] pub man_pred_source :
            Option < String >, #[serde(rename = "MAN_REF_FRAME")] pub
            man_ref_frame : String, #[serde(rename = "MAN_FRAME_EPOCH")] pub
            man_frame_epoch : Option < schema_common :: xml_schema_types ::
            EpochType >, #[serde(rename = "GRAV_ASSIST_NAME")] pub
            grav_assist_name : Option < String >, #[serde(rename = "DC_TYPE")]
            pub dc_type : schema_common :: xml_schema_types :: ManDcType,
            #[serde(rename = "DC_WIN_OPEN")] pub dc_win_open : Option <
            schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "DC_WIN_CLOSE")] pub dc_win_close : Option <
            schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "DC_MIN_CYCLES")] pub dc_min_cycles : Option <
            u64 >, #[serde(rename = "DC_MAX_CYCLES")] pub dc_max_cycles :
            Option < u64 >, #[serde(rename = "DC_EXEC_START")] pub
            dc_exec_start : Option < schema_common :: xml_schema_types ::
            EpochType >, #[serde(rename = "DC_EXEC_STOP")] pub dc_exec_stop :
            Option < schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "DC_REF_TIME")] pub dc_ref_time : Option <
            schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "DC_TIME_PULSE_DURATION")] pub
            dc_time_pulse_duration : Option < schema_common ::
            xml_schema_types :: DurationType >,
            #[serde(rename = "DC_TIME_PULSE_PERIOD")] pub dc_time_pulse_period
            : Option < schema_common :: xml_schema_types :: DurationType >,
            #[serde(rename = "DC_REF_DIR")] pub dc_ref_dir : Option <
            schema_common :: xml_schema_types :: Vec3Double >,
            #[serde(rename = "DC_BODY_FRAME")] pub dc_body_frame : Option <
            String >, #[serde(rename = "DC_BODY_TRIGGER")] pub dc_body_trigger
            : Option < schema_common :: xml_schema_types :: Vec3Double >,
            #[serde(rename = "DC_PA_START_ANGLE")] pub dc_pa_start_angle :
            Option < schema_common :: xml_schema_types :: AngleType >,
            #[serde(rename = "DC_PA_STOP_ANGLE")] pub dc_pa_stop_angle :
            Option < schema_common :: xml_schema_types :: AngleType >,
            #[serde(rename = "MAN_COMPOSITION")] pub man_composition : String,
            #[serde(rename = "MAN_UNITS")] pub man_units : Option < String >,
            #[serde(rename = "manLine")] pub man_line_list : Vec < String >,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OcmPerturbationsType
        {
            #[serde(rename = "COMMENT")] pub comment_list : Vec < String >,
            #[serde(rename = "ATMOSPHERIC_MODEL")] pub atmospheric_model :
            Option < String >, #[serde(rename = "GRAVITY_MODEL")] pub
            gravity_model : Option < String >,
            #[serde(rename = "EQUATORIAL_RADIUS")] pub equatorial_radius :
            Option < schema_common :: xml_schema_types :: PositionType >,
            #[serde(rename = "GM")] pub gm : Option < schema_common ::
            xml_schema_types :: GmType >,
            #[serde(rename = "N_BODY_PERTURBATIONS")] pub n_body_perturbations
            : Option < String >, #[serde(rename = "CENTRAL_BODY_ROTATION")]
            pub central_body_rotation : Option < schema_common ::
            xml_schema_types :: AngleRateType >,
            #[serde(rename = "OBLATE_FLATTENING")] pub oblate_flattening :
            Option < schema_common :: xml_schema_types :: PositiveDouble >,
            #[serde(rename = "OCEAN_TIDES_MODEL")] pub ocean_tides_model :
            Option < String >, #[serde(rename = "SOLID_TIDES_MODEL")] pub
            solid_tides_model : Option < String >,
            #[serde(rename = "REDUCTION_THEORY")] pub reduction_theory :
            Option < String >, #[serde(rename = "ALBEDO_MODEL")] pub
            albedo_model : Option < String >,
            #[serde(rename = "ALBEDO_GRID_SIZE")] pub albedo_grid_size :
            Option < u64 >, #[serde(rename = "SHADOW_MODEL")] pub shadow_model
            : Option < String >, #[serde(rename = "SHADOW_BODIES")] pub
            shadow_bodies : Option < String >, #[serde(rename = "SRP_MODEL")]
            pub srp_model : Option < String >,
            #[serde(rename = "SW_DATA_SOURCE")] pub sw_data_source : Option <
            String >, #[serde(rename = "SW_DATA_EPOCH")] pub sw_data_epoch :
            Option < schema_common :: xml_schema_types :: EpochType >,
            #[serde(rename = "SW_INTERP_METHOD")] pub sw_interp_method :
            Option < String >, #[serde(rename = "FIXED_GEOMAG_KP")] pub
            fixed_geomag_kp : Option < schema_common :: xml_schema_types ::
            GeomagType >, #[serde(rename = "FIXED_GEOMAG_AP")] pub
            fixed_geomag_ap : Option < schema_common :: xml_schema_types ::
            GeomagType >, #[serde(rename = "FIXED_GEOMAG_DST")] pub
            fixed_geomag_dst : Option < schema_common :: xml_schema_types ::
            GeomagType >, #[serde(rename = "FIXED_F10P7")] pub fixed_f10p7 :
            Option < schema_common :: xml_schema_types :: SolarFluxType >,
            #[serde(rename = "FIXED_F10P7_MEAN")] pub fixed_f10p7_mean :
            Option < schema_common :: xml_schema_types :: SolarFluxType >,
            #[serde(rename = "FIXED_M10P7")] pub fixed_m10p7 : Option <
            schema_common :: xml_schema_types :: SolarFluxType >,
            #[serde(rename = "FIXED_M10P7_MEAN")] pub fixed_m10p7_mean :
            Option < schema_common :: xml_schema_types :: SolarFluxType >,
            #[serde(rename = "FIXED_S10P7")] pub fixed_s10p7 : Option <
            schema_common :: xml_schema_types :: SolarFluxType >,
            #[serde(rename = "FIXED_S10P7_MEAN")] pub fixed_s10p7_mean :
            Option < schema_common :: xml_schema_types :: SolarFluxType >,
            #[serde(rename = "FIXED_Y10P7")] pub fixed_y10p7 : Option <
            schema_common :: xml_schema_types :: SolarFluxType >,
            #[serde(rename = "FIXED_Y10P7_MEAN")] pub fixed_y10p7_mean :
            Option < schema_common :: xml_schema_types :: SolarFluxType >,
        }
        #[derive(Clone, Debug, Default, PartialEq, serde :: Deserialize, serde
        :: Serialize)] #[serde(default)] pub struct OcmOdParametersType
        {
            #[serde(rename = "COMMENT")] pub comment_list : Vec < String >,
            #[serde(rename = "OD_ID")] pub od_id : String,
            #[serde(rename = "OD_PREV_ID")] pub od_prev_id : Option < String
            >, #[serde(rename = "OD_METHOD")] pub od_method : String,
            #[serde(rename = "OD_EPOCH")] pub od_epoch : schema_common ::
            xml_schema_types :: EpochType,
            #[serde(rename = "DAYS_SINCE_FIRST_OBS")] pub days_since_first_obs
            : Option < schema_common :: xml_schema_types :: OcmDayIntervalType
            >, #[serde(rename = "DAYS_SINCE_LAST_OBS")] pub
            days_since_last_obs : Option < schema_common :: xml_schema_types
            :: OcmDayIntervalType >, #[serde(rename = "RECOMMENDED_OD_SPAN")]
            pub recommended_od_span : Option < schema_common ::
            xml_schema_types :: OcmDayIntervalType >,
            #[serde(rename = "ACTUAL_OD_SPAN")] pub actual_od_span : Option <
            schema_common :: xml_schema_types :: OcmDayIntervalType >,
            #[serde(rename = "OBS_AVAILABLE")] pub obs_available : Option <
            u64 >, #[serde(rename = "OBS_USED")] pub obs_used : Option < u64
            >, #[serde(rename = "TRACKS_AVAILABLE")] pub tracks_available :
            Option < u64 >, #[serde(rename = "TRACKS_USED")] pub tracks_used :
            Option < u64 >, #[serde(rename = "MAXIMUM_OBS_GAP")] pub
            maximum_obs_gap : Option < schema_common :: xml_schema_types ::
            OcmDayIntervalType >, #[serde(rename = "OD_EPOCH_EIGMAJ")] pub
            od_epoch_eigmaj : Option < schema_common :: xml_schema_types ::
            OcmLengthType >, #[serde(rename = "OD_EPOCH_EIGINT")] pub
            od_epoch_eigint : Option < schema_common :: xml_schema_types ::
            OcmLengthType >, #[serde(rename = "OD_EPOCH_EIGMIN")] pub
            od_epoch_eigmin : Option < schema_common :: xml_schema_types ::
            OcmLengthType >, #[serde(rename = "OD_MAX_PRED_EIGMAJ")] pub
            od_max_pred_eigmaj : Option < schema_common :: xml_schema_types ::
            OcmLengthType >, #[serde(rename = "OD_MIN_PRED_EIGMIN")] pub
            od_min_pred_eigmin : Option < schema_common :: xml_schema_types ::
            OcmLengthType >, #[serde(rename = "OD_CONFIDENCE")] pub
            od_confidence : Option < schema_common :: xml_schema_types ::
            PercentageType >, #[serde(rename = "GDOP")] pub gdop : Option <
            schema_common :: xml_schema_types :: NonNegativeDouble >,
            #[serde(rename = "SOLVE_N")] pub solve_n : Option < u64 >,
            #[serde(rename = "SOLVE_STATES")] pub solve_states : Option <
            String >, #[serde(rename = "CONSIDER_N")] pub consider_n : Option
            < u64 >, #[serde(rename = "CONSIDER_PARAMS")] pub consider_params
            : Option < String >, #[serde(rename = "SEDR")] pub sedr : Option <
            schema_common :: xml_schema_types :: WkgType >,
            #[serde(rename = "SENSORS_N")] pub sensors_n : Option < u64 >,
            #[serde(rename = "SENSORS")] pub sensors : Option < String >,
            #[serde(rename = "WEIGHTED_RMS")] pub weighted_rms : Option <
            schema_common :: xml_schema_types :: NonNegativeDouble >,
            #[serde(rename = "DATA_TYPES")] pub data_types : Option < String
            >,
        }
    }
} pub use schema_ocm :: * ;