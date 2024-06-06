/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde;

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AccUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AngleUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AngleRange(#[serde(rename = "$text")] pub f64);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AngleRateUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AngMomentumUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AngVelFrameType(#[serde(rename = "$text")] pub f64);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AreaUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct DayIntervalUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FrequencyUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct GmUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct InclinationRange(#[serde(rename = "$text")] pub f64);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LengthUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MassUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MomentUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct WkgUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ObjectDescriptionType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Ms2Units(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Km2Units(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Km2sUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Km2s2Units(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PositionUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct VelocityUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VecDouble {
    pub items: Vec<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Vec3Double(#[serde(rename = "$text")] pub f64);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Vec6Double(#[serde(rename = "$text")] pub f64);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Vec9Double(#[serde(rename = "$text")] pub f64);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct EpochType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TimeUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TimeSystemType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct NegativeDouble(#[serde(rename = "$text")] pub f64);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct NonNegativeDouble(#[serde(rename = "$text")] pub f64);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct NonPositiveDouble(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PercentType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PositiveDouble(#[serde(rename = "$text")] pub f64);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Range100Type(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ProbabilityType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PercentageUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct YesNoType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TrajBasisType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RevNumBasisType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CovBasisType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ManBasisType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ManDcType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct NumPerYearUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ThrustUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CovOrderType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct GeomagUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SolarFluxUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PositionCovarianceUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct VelocityCovarianceUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PositionVelocityCovarianceUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BallisticCoeffUnitsType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LatRange(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AltRange(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LonRange(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LatLonUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ControlledType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct DisintegrationType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ImpactUncertaintyType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ReentryUncertaintyMethodType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct QuaternionComponentType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct QuaternionDotUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RotDirectionType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RotseqType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AngleKeywordType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AngleRateKeywordType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ApmRateFrameType(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TorqueUnits(#[serde(rename = "$text")] pub std::string::String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct NdmHeader {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "CREATION_DATE")]
    pub creation_date: EpochType,
    #[serde(rename = "ORIGINATOR")]
    pub originator: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AdmHeader {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "CREATION_DATE")]
    pub creation_date: EpochType,
    #[serde(rename = "ORIGINATOR")]
    pub originator: String,
    #[serde(rename = "MESSAGE_ID")]
    pub message_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OdmHeader {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "CLASSIFICATION")]
    pub classification_list: Vec<String>,
    #[serde(rename = "CREATION_DATE")]
    pub creation_date: EpochType,
    #[serde(rename = "ORIGINATOR")]
    pub originator: String,
    #[serde(rename = "MESSAGE_ID")]
    pub message_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AccType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<AccUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AngleType {
    #[serde(rename = "$text")]
    pub base: AngleRange,
    #[serde(rename = "@units")]
    pub units: Option<AngleUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AngleRateType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<AngleRateUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AngMomentumType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: AngMomentumUnits,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AngVelComponentType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<AngleRateUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AngVelStateType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "REF_FRAME_A")]
    pub ref_frame_a: String,
    #[serde(rename = "REF_FRAME_B")]
    pub ref_frame_b: String,
    #[serde(rename = "ANGVEL_FRAME")]
    pub angvel_frame: AngVelFrameType,
    #[serde(rename = "ANGVEL_X")]
    pub angvel_x: AngVelComponentType,
    #[serde(rename = "ANGVEL_Y")]
    pub angvel_y: AngVelComponentType,
    #[serde(rename = "ANGVEL_Z")]
    pub angvel_z: AngVelComponentType,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AngVelType {
    #[serde(rename = "ANGVEL_X")]
    pub angvel_x: AngVelComponentType,
    #[serde(rename = "ANGVEL_Y")]
    pub angvel_y: AngVelComponentType,
    #[serde(rename = "ANGVEL_Z")]
    pub angvel_z: AngVelComponentType,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AreaType {
    #[serde(rename = "$text")]
    pub base: NonNegativeDouble,
    #[serde(rename = "@units")]
    pub units: Option<AreaUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct DayIntervalType {
    #[serde(rename = "$text")]
    pub base: NonNegativeDouble,
    #[serde(rename = "@units")]
    pub units: DayIntervalUnits,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OcmDayIntervalType {
    #[serde(rename = "$text")]
    pub base: NonNegativeDouble,
    #[serde(rename = "@units")]
    pub units: Option<DayIntervalUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct DeltamassType {
    #[serde(rename = "$text")]
    pub base: NegativeDouble,
    #[serde(rename = "@units")]
    pub units: Option<MassUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct DeltamassTypeZ {
    #[serde(rename = "$text")]
    pub base: NonPositiveDouble,
    #[serde(rename = "@units")]
    pub units: Option<MassUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FrequencyType {
    #[serde(rename = "$text")]
    pub base: PositiveDouble,
    #[serde(rename = "@units")]
    pub units: Option<FrequencyUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct GmType {
    #[serde(rename = "$text")]
    pub base: PositiveDouble,
    #[serde(rename = "@units")]
    pub units: Option<GmUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct InclinationType {
    #[serde(rename = "$text")]
    pub base: InclinationRange,
    #[serde(rename = "@units")]
    pub units: Option<AngleUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LengthType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: LengthUnits,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OcmLengthType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<LengthUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MassType {
    #[serde(rename = "$text")]
    pub base: NonNegativeDouble,
    #[serde(rename = "@units")]
    pub units: Option<MassUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MomentType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<MomentUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct WkgType {
    #[serde(rename = "$text")]
    pub base: NonNegativeDouble,
    #[serde(rename = "@units")]
    pub units: WkgUnits,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OdParametersType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "TIME_LASTOB_START")]
    pub time_lastob_start: Option<EpochType>,
    #[serde(rename = "TIME_LASTOB_END")]
    pub time_lastob_end: Option<EpochType>,
    #[serde(rename = "RECOMMENDED_OD_SPAN")]
    pub recommended_od_span: Option<DayIntervalType>,
    #[serde(rename = "ACTUAL_OD_SPAN")]
    pub actual_od_span: Option<DayIntervalType>,
    #[serde(rename = "OBS_AVAILABLE")]
    pub obs_available: Option<u64>,
    #[serde(rename = "OBS_USED")]
    pub obs_used: Option<u64>,
    #[serde(rename = "TRACKS_AVAILABLE")]
    pub tracks_available: Option<u64>,
    #[serde(rename = "TRACKS_USED")]
    pub tracks_used: Option<u64>,
    #[serde(rename = "RESIDUALS_ACCEPTED")]
    pub residuals_accepted: Option<PercentageType>,
    #[serde(rename = "WEIGHTED_RMS")]
    pub weighted_rms: Option<NonNegativeDouble>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SpacecraftParametersType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "MASS")]
    pub mass: Option<MassType>,
    #[serde(rename = "SOLAR_RAD_AREA")]
    pub solar_rad_area: Option<AreaType>,
    #[serde(rename = "SOLAR_RAD_COEFF")]
    pub solar_rad_coeff: Option<NonNegativeDouble>,
    #[serde(rename = "DRAG_AREA")]
    pub drag_area: Option<AreaType>,
    #[serde(rename = "DRAG_COEFF")]
    pub drag_coeff: Option<NonNegativeDouble>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct StateVectorType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "EPOCH")]
    pub epoch: EpochType,
    #[serde(rename = "X")]
    pub x: PositionType,
    #[serde(rename = "Y")]
    pub y: PositionType,
    #[serde(rename = "Z")]
    pub z: PositionType,
    #[serde(rename = "X_DOT")]
    pub x_dot: VelocityType,
    #[serde(rename = "Y_DOT")]
    pub y_dot: VelocityType,
    #[serde(rename = "Z_DOT")]
    pub z_dot: VelocityType,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct StateVectorAccType {
    #[serde(rename = "EPOCH")]
    pub epoch: EpochType,
    #[serde(rename = "X")]
    pub x: PositionType,
    #[serde(rename = "Y")]
    pub y: PositionType,
    #[serde(rename = "Z")]
    pub z: PositionType,
    #[serde(rename = "X_DOT")]
    pub x_dot: VelocityType,
    #[serde(rename = "Y_DOT")]
    pub y_dot: VelocityType,
    #[serde(rename = "Z_DOT")]
    pub z_dot: VelocityType,
    #[serde(rename = "X_DDOT")]
    pub x_ddot: Option<AccType>,
    #[serde(rename = "Y_DDOT")]
    pub y_ddot: Option<AccType>,
    #[serde(rename = "Z_DDOT")]
    pub z_ddot: Option<AccType>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Ms2Type {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Ms2Units,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Km2Type {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<Km2Units>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Km2sType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<Km2sUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Km2s2Type {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<Km2s2Units>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct DistanceType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<PositionUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PositionType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<PositionUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RdmPositionType {
    #[serde(rename = "$text")]
    pub base: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct VelocityType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<VelocityUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RdmVelocityType {
    #[serde(rename = "$text")]
    pub base: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct DurationType {
    #[serde(rename = "$text")]
    pub base: NonNegativeDouble,
    #[serde(rename = "@units")]
    pub units: Option<TimeUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RelTimeType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<TimeUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TimeOffsetType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<TimeUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PercentageType {
    #[serde(rename = "$text")]
    pub base: Range100Type,
    #[serde(rename = "@units")]
    pub units: Option<PercentageUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ManeuverFreqType {
    #[serde(rename = "$text")]
    pub base: NonNegativeDouble,
    #[serde(rename = "@units")]
    pub units: Option<NumPerYearUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ThrustType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<ThrustUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct GeomagType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<GeomagUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SolarFluxType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<SolarFluxUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OemCovarianceMatrixType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "EPOCH")]
    pub epoch: EpochType,
    #[serde(rename = "COV_REF_FRAME")]
    pub cov_ref_frame: Option<String>,
    #[serde(rename = "CX_X")]
    pub cx_x: PositionCovarianceType,
    #[serde(rename = "CY_X")]
    pub cy_x: PositionCovarianceType,
    #[serde(rename = "CY_Y")]
    pub cy_y: PositionCovarianceType,
    #[serde(rename = "CZ_X")]
    pub cz_x: PositionCovarianceType,
    #[serde(rename = "CZ_Y")]
    pub cz_y: PositionCovarianceType,
    #[serde(rename = "CZ_Z")]
    pub cz_z: PositionCovarianceType,
    #[serde(rename = "CX_DOT_X")]
    pub cx_dot_x: PositionVelocityCovarianceType,
    #[serde(rename = "CX_DOT_Y")]
    pub cx_dot_y: PositionVelocityCovarianceType,
    #[serde(rename = "CX_DOT_Z")]
    pub cx_dot_z: PositionVelocityCovarianceType,
    #[serde(rename = "CX_DOT_X_DOT")]
    pub cx_dot_x_dot: VelocityCovarianceType,
    #[serde(rename = "CY_DOT_X")]
    pub cy_dot_x: PositionVelocityCovarianceType,
    #[serde(rename = "CY_DOT_Y")]
    pub cy_dot_y: PositionVelocityCovarianceType,
    #[serde(rename = "CY_DOT_Z")]
    pub cy_dot_z: PositionVelocityCovarianceType,
    #[serde(rename = "CY_DOT_X_DOT")]
    pub cy_dot_x_dot: VelocityCovarianceType,
    #[serde(rename = "CY_DOT_Y_DOT")]
    pub cy_dot_y_dot: VelocityCovarianceType,
    #[serde(rename = "CZ_DOT_X")]
    pub cz_dot_x: PositionVelocityCovarianceType,
    #[serde(rename = "CZ_DOT_Y")]
    pub cz_dot_y: PositionVelocityCovarianceType,
    #[serde(rename = "CZ_DOT_Z")]
    pub cz_dot_z: PositionVelocityCovarianceType,
    #[serde(rename = "CZ_DOT_X_DOT")]
    pub cz_dot_x_dot: VelocityCovarianceType,
    #[serde(rename = "CZ_DOT_Y_DOT")]
    pub cz_dot_y_dot: VelocityCovarianceType,
    #[serde(rename = "CZ_DOT_Z_DOT")]
    pub cz_dot_z_dot: VelocityCovarianceType,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OpmCovarianceMatrixType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "COV_REF_FRAME")]
    pub cov_ref_frame: Option<String>,
    #[serde(rename = "CX_X")]
    pub cx_x: PositionCovarianceType,
    #[serde(rename = "CY_X")]
    pub cy_x: PositionCovarianceType,
    #[serde(rename = "CY_Y")]
    pub cy_y: PositionCovarianceType,
    #[serde(rename = "CZ_X")]
    pub cz_x: PositionCovarianceType,
    #[serde(rename = "CZ_Y")]
    pub cz_y: PositionCovarianceType,
    #[serde(rename = "CZ_Z")]
    pub cz_z: PositionCovarianceType,
    #[serde(rename = "CX_DOT_X")]
    pub cx_dot_x: PositionVelocityCovarianceType,
    #[serde(rename = "CX_DOT_Y")]
    pub cx_dot_y: PositionVelocityCovarianceType,
    #[serde(rename = "CX_DOT_Z")]
    pub cx_dot_z: PositionVelocityCovarianceType,
    #[serde(rename = "CX_DOT_X_DOT")]
    pub cx_dot_x_dot: VelocityCovarianceType,
    #[serde(rename = "CY_DOT_X")]
    pub cy_dot_x: PositionVelocityCovarianceType,
    #[serde(rename = "CY_DOT_Y")]
    pub cy_dot_y: PositionVelocityCovarianceType,
    #[serde(rename = "CY_DOT_Z")]
    pub cy_dot_z: PositionVelocityCovarianceType,
    #[serde(rename = "CY_DOT_X_DOT")]
    pub cy_dot_x_dot: VelocityCovarianceType,
    #[serde(rename = "CY_DOT_Y_DOT")]
    pub cy_dot_y_dot: VelocityCovarianceType,
    #[serde(rename = "CZ_DOT_X")]
    pub cz_dot_x: PositionVelocityCovarianceType,
    #[serde(rename = "CZ_DOT_Y")]
    pub cz_dot_y: PositionVelocityCovarianceType,
    #[serde(rename = "CZ_DOT_Z")]
    pub cz_dot_z: PositionVelocityCovarianceType,
    #[serde(rename = "CZ_DOT_X_DOT")]
    pub cz_dot_x_dot: VelocityCovarianceType,
    #[serde(rename = "CZ_DOT_Y_DOT")]
    pub cz_dot_y_dot: VelocityCovarianceType,
    #[serde(rename = "CZ_DOT_Z_DOT")]
    pub cz_dot_z_dot: VelocityCovarianceType,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PositionCovarianceType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<PositionCovarianceUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct VelocityCovarianceType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<VelocityCovarianceUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PositionVelocityCovarianceType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<PositionVelocityCovarianceUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AtmosphericReentryParametersType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "ORBIT_LIFETIME")]
    pub orbit_lifetime: DayIntervalType,
    #[serde(rename = "REENTRY_ALTITUDE")]
    pub reentry_altitude: PositionType,
    #[serde(rename = "ORBIT_LIFETIME_WINDOW_START")]
    pub orbit_lifetime_window_start: Option<DayIntervalType>,
    #[serde(rename = "ORBIT_LIFETIME_WINDOW_END")]
    pub orbit_lifetime_window_end: Option<DayIntervalType>,
    #[serde(rename = "NOMINAL_REENTRY_EPOCH")]
    pub nominal_reentry_epoch: Option<EpochType>,
    #[serde(rename = "REENTRY_WINDOW_START")]
    pub reentry_window_start: Option<EpochType>,
    #[serde(rename = "REENTRY_WINDOW_END")]
    pub reentry_window_end: Option<EpochType>,
    #[serde(rename = "ORBIT_LIFETIME_CONFIDENCE_LEVEL")]
    pub orbit_lifetime_confidence_level: Option<PercentageType>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct GroundImpactParametersType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "PROBABILITY_OF_IMPACT")]
    pub probability_of_impact: Option<ProbabilityType>,
    #[serde(rename = "PROBABILITY_OF_BURN_UP")]
    pub probability_of_burn_up: Option<ProbabilityType>,
    #[serde(rename = "PROBABILITY_OF_BREAK_UP")]
    pub probability_of_break_up: Option<ProbabilityType>,
    #[serde(rename = "PROBABILITY_OF_LAND_IMPACT")]
    pub probability_of_land_impact: Option<ProbabilityType>,
    #[serde(rename = "PROBABILITY_OF_CASUALTY")]
    pub probability_of_casualty: Option<ProbabilityType>,
    #[serde(rename = "NOMINAL_IMPACT_EPOCH")]
    pub nominal_impact_epoch: Option<EpochType>,
    #[serde(rename = "IMPACT_WINDOW_START")]
    pub impact_window_start: Option<EpochType>,
    #[serde(rename = "IMPACT_WINDOW_END")]
    pub impact_window_end: Option<EpochType>,
    #[serde(rename = "IMPACT_REF_FRAME")]
    pub impact_ref_frame: Option<String>,
    #[serde(rename = "NOMINAL_IMPACT_LON")]
    pub nominal_impact_lon: Option<LonType>,
    #[serde(rename = "NOMINAL_IMPACT_LAT")]
    pub nominal_impact_lat: Option<LatType>,
    #[serde(rename = "NOMINAL_IMPACT_ALT")]
    pub nominal_impact_alt: Option<AltType>,
    #[serde(rename = "IMPACT_1_CONFIDENCE")]
    pub impact_1_confidence: Option<PercentageType>,
    #[serde(rename = "IMPACT_1_START_LON")]
    pub impact_1_start_lon: Option<LonType>,
    #[serde(rename = "IMPACT_1_START_LAT")]
    pub impact_1_start_lat: Option<LatType>,
    #[serde(rename = "IMPACT_1_STOP_LON")]
    pub impact_1_stop_lon: Option<LonType>,
    #[serde(rename = "IMPACT_1_STOP_LAT")]
    pub impact_1_stop_lat: Option<LatType>,
    #[serde(rename = "IMPACT_1_CROSS_TRACK")]
    pub impact_1_cross_track: Option<DistanceType>,
    #[serde(rename = "IMPACT_2_CONFIDENCE")]
    pub impact_2_confidence: Option<PercentageType>,
    #[serde(rename = "IMPACT_2_START_LON")]
    pub impact_2_start_lon: Option<LonType>,
    #[serde(rename = "IMPACT_2_START_LAT")]
    pub impact_2_start_lat: Option<LatType>,
    #[serde(rename = "IMPACT_2_STOP_LON")]
    pub impact_2_stop_lon: Option<LonType>,
    #[serde(rename = "IMPACT_2_STOP_LAT")]
    pub impact_2_stop_lat: Option<LatType>,
    #[serde(rename = "IMPACT_2_CROSS_TRACK")]
    pub impact_2_cross_track: Option<DistanceType>,
    #[serde(rename = "IMPACT_3_CONFIDENCE")]
    pub impact_3_confidence: Option<PercentageType>,
    #[serde(rename = "IMPACT_3_START_LON")]
    pub impact_3_start_lon: Option<LonType>,
    #[serde(rename = "IMPACT_3_START_LAT")]
    pub impact_3_start_lat: Option<LatType>,
    #[serde(rename = "IMPACT_3_STOP_LON")]
    pub impact_3_stop_lon: Option<LonType>,
    #[serde(rename = "IMPACT_3_STOP_LAT")]
    pub impact_3_stop_lat: Option<LatType>,
    #[serde(rename = "IMPACT_3_CROSS_TRACK")]
    pub impact_3_cross_track: Option<DistanceType>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RdmSpacecraftParametersType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "WET_MASS")]
    pub wet_mass: Option<MassType>,
    #[serde(rename = "DRY_MASS")]
    pub dry_mass: Option<MassType>,
    #[serde(rename = "HAZARDOUS_SUBSTANCES")]
    pub hazardous_substances: Option<String>,
    #[serde(rename = "SOLAR_RAD_AREA")]
    pub solar_rad_area: Option<AreaType>,
    #[serde(rename = "SOLAR_RAD_COEFF")]
    pub solar_rad_coeff: Option<NonNegativeDouble>,
    #[serde(rename = "DRAG_AREA")]
    pub drag_area: Option<AreaType>,
    #[serde(rename = "DRAG_COEFF")]
    pub drag_coeff: Option<NonNegativeDouble>,
    #[serde(rename = "RCS")]
    pub rcs: Option<AreaType>,
    #[serde(rename = "BALLISTIC_COEFF")]
    pub ballistic_coeff: Option<BallisticCoeffType>,
    #[serde(rename = "THRUST_ACCELERATION")]
    pub thrust_acceleration: Option<Ms2Type>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AltType {
    #[serde(rename = "$text")]
    pub base: AltRange,
    #[serde(rename = "@units")]
    pub units: Option<LengthUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BallisticCoeffType {
    #[serde(rename = "$text")]
    pub base: NonNegativeDouble,
    #[serde(rename = "@units")]
    pub units: Option<BallisticCoeffUnitsType>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LatType {
    #[serde(rename = "$text")]
    pub base: LatRange,
    #[serde(rename = "@units")]
    pub units: LatLonUnits,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LonType {
    #[serde(rename = "$text")]
    pub base: LonRange,
    #[serde(rename = "@units")]
    pub units: LatLonUnits,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct UserDefinedType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "USER_DEFINED")]
    pub user_defined_list: Vec<UserDefinedParameterType>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct UserDefinedParameterType {
    #[serde(rename = "$text")]
    pub base: String,
    #[serde(rename = "@parameter")]
    pub parameter: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct QuaternionType {}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct QuaternionRateType {}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct QuaternionDotType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<QuaternionDotUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RotationAngleType {
    #[serde(rename = "rotation1")]
    pub rotation1: RotationAngleComponentType,
    #[serde(rename = "rotation2")]
    pub rotation2: RotationAngleComponentType,
    #[serde(rename = "rotation3")]
    pub rotation3: RotationAngleComponentType,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RotationAngleComponentTypeold {
    #[serde(rename = "@units")]
    pub units: Option<AngleUnits>,
    #[serde(rename = "@angle")]
    pub angle: AngleKeywordType,
    #[serde(rename = "@value")]
    pub value: AngleRange,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RotationAngleComponentType {
    #[serde(rename = "$text")]
    pub base: AngleRange,
    #[serde(rename = "@angle")]
    pub angle: AngleKeywordType,
    #[serde(rename = "@units")]
    pub units: Option<AngleUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RotationRateType {
    #[serde(rename = "rotation1")]
    pub rotation1: RotationRateComponentType,
    #[serde(rename = "rotation2")]
    pub rotation2: RotationRateComponentType,
    #[serde(rename = "rotation3")]
    pub rotation3: RotationRateComponentType,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RotationRateComponentTypeOld {
    #[serde(rename = "@units")]
    pub units: Option<AngleRateUnits>,
    #[serde(rename = "@rate")]
    pub rate: AngleRateKeywordType,
    #[serde(rename = "@value")]
    pub value: f64,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RotationRateComponentType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@rate")]
    pub rate: AngleRateKeywordType,
    #[serde(rename = "@units")]
    pub units: Option<AngleRateUnits>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TorqueType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<TorqueUnits>,
}