/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Data types shared between different NDM message types

use serde;

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AccUnits(#[serde(rename = "$text")] pub String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AngleUnits(#[serde(rename = "$text")] pub String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AngleRateUnits(#[serde(rename = "$text")] pub String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AreaUnits(#[serde(rename = "$text")] pub String);

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
pub struct DayIntervalUnits(#[serde(rename = "$text")] pub String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct GmUnits(#[serde(rename = "$text")] pub String);

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
pub struct LengthUnits(#[serde(rename = "$text")] pub String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MassUnits(#[serde(rename = "$text")] pub String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MomentUnits(#[serde(rename = "$text")] pub String);

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
pub struct WkgUnits(#[serde(rename = "$text")] pub String);

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
pub struct ObjectDescriptionType(#[serde(rename = "$text")] pub String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PositionUnits(#[serde(rename = "$text")] pub String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct VelocityUnits(#[serde(rename = "$text")] pub String);

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
pub struct Vec3Double(#[serde(rename = "$text")] pub f64);

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
pub struct EpochType(#[serde(rename = "$text")] pub String);

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
pub struct TimeUnits(#[serde(rename = "$text")] pub String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct NegativeDouble(#[serde(rename = "$text")] pub f64);

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
pub struct NonNegativeDouble(#[serde(rename = "$text")] pub f64);

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
pub struct PositiveDouble(#[serde(rename = "$text")] pub f64);

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
pub struct Range100Type(#[serde(rename = "$text")] pub String);

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
pub struct ProbabilityType(#[serde(rename = "$text")] pub String);

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
pub struct PercentageUnits(#[serde(rename = "$text")] pub String);

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
pub struct TrajBasisType(#[serde(rename = "$text")] pub String);

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
pub struct RevNumBasisType(#[serde(rename = "$text")] pub String);

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
pub struct CovBasisType(#[serde(rename = "$text")] pub String);

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
pub struct ManBasisType(#[serde(rename = "$text")] pub String);

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
pub struct ManDcType(#[serde(rename = "$text")] pub String);

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
pub struct NumPerYearUnits(#[serde(rename = "$text")] pub String);

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
pub struct ThrustUnits(#[serde(rename = "$text")] pub String);

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
pub struct CovOrderType(#[serde(rename = "$text")] pub String);

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
pub struct GeomagUnits(#[serde(rename = "$text")] pub String);

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
pub struct SolarFluxUnits(#[serde(rename = "$text")] pub String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PositionCovarianceUnits(#[serde(rename = "$text")] pub String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct VelocityCovarianceUnits(#[serde(rename = "$text")] pub String);

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PositionVelocityCovarianceUnits(#[serde(rename = "$text")] pub String);

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
#[kvn(value_unit_struct)]
pub struct AccType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<AccUnits>,
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
#[kvn(value_unit_struct)]
pub struct AngleType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<AngleUnits>,
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
#[kvn(value_unit_struct)]
pub struct AngleRateType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<AngleRateUnits>,
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
#[kvn(value_unit_struct)]
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
pub struct OcmDayIntervalType {
    #[serde(rename = "$text")]
    pub base: NonNegativeDouble,
    #[serde(rename = "@units")]
    pub units: Option<DayIntervalUnits>,
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
#[kvn(value_unit_struct)]
pub struct DeltamassType {
    #[serde(rename = "$text")]
    pub base: NegativeDouble,
    #[serde(rename = "@units")]
    pub units: Option<MassUnits>,
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
#[kvn(value_unit_struct)]
pub struct GmType {
    #[serde(rename = "$text")]
    pub base: PositiveDouble,
    #[serde(rename = "@units")]
    pub units: Option<GmUnits>,
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
#[kvn(value_unit_struct)]
pub struct InclinationType {
    #[serde(rename = "$text")]
    pub base: f64,
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
pub struct OcmLengthType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<LengthUnits>,
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
#[kvn(value_unit_struct)]
pub struct MassType {
    #[serde(rename = "$text")]
    pub base: NonNegativeDouble,
    #[serde(rename = "@units")]
    pub units: Option<MassUnits>,
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
#[kvn(value_unit_struct)]
pub struct MomentType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<MomentUnits>,
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
#[kvn(value_unit_struct)]
pub struct DistanceType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<PositionUnits>,
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
#[kvn(value_unit_struct)]
pub struct PositionType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<PositionUnits>,
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
#[kvn(value_unit_struct)]
pub struct VelocityType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<VelocityUnits>,
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
#[kvn(value_unit_struct)]
pub struct DurationType {
    #[serde(rename = "$text")]
    pub base: NonNegativeDouble,
    #[serde(rename = "@units")]
    pub units: Option<TimeUnits>,
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
pub struct TimeOffsetType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<TimeUnits>,
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
pub struct PercentageType {
    #[serde(rename = "$text")]
    pub base: Range100Type,
    #[serde(rename = "@units")]
    pub units: Option<PercentageUnits>,
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
pub struct ManeuverFreqType {
    #[serde(rename = "$text")]
    pub base: NonNegativeDouble,
    #[serde(rename = "@units")]
    pub units: Option<NumPerYearUnits>,
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
pub struct ThrustType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<ThrustUnits>,
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
pub struct GeomagType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<GeomagUnits>,
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
pub struct SolarFluxType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<SolarFluxUnits>,
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
#[kvn(value_unit_struct)]
pub struct PositionCovarianceType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<PositionCovarianceUnits>,
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
#[kvn(value_unit_struct)]
pub struct VelocityCovarianceType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<VelocityCovarianceUnits>,
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
#[kvn(value_unit_struct)]
pub struct PositionVelocityCovarianceType {
    #[serde(rename = "$text")]
    pub base: f64,
    #[serde(rename = "@units")]
    pub units: Option<PositionVelocityCovarianceUnits>,
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
pub struct UserDefinedType {
    #[serde(rename = "COMMENT")]
    pub comment_list: Vec<String>,
    #[serde(rename = "USER_DEFINED")]
    pub user_defined_list: Vec<UserDefinedParameterType>,
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
pub struct UserDefinedParameterType {
    #[serde(rename = "$text")]
    pub base: String,
    #[serde(rename = "@parameter")]
    pub parameter: String,
}
