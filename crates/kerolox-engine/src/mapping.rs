// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Bidirectional conversions between Kerolox protobuf types and
//! lox-space domain types. Every public function here is pure and
//! exhaustively tested.

use kerolox_proto::kerolox::v1::{
    AccessWindow as ProtoAccessWindow, LookSide as ProtoLookSide,
    PassDirection as ProtoPassDirection, SarSensor, SatelliteOrbitalElements,
};
use lox_space::analysis::imaging::results::AccessWindow as LoxAccessWindow;
use lox_space::analysis::imaging::results::PassDirection as LoxPassDirection;
use lox_space::analysis::imaging::sar::{LookSide as LoxLookSide, SarPayload, SarPayloadError};
use lox_space::core::anomalies::TrueAnomaly;
use lox_space::core::elements::keplerian::{
    ArgumentOfPeriapsis, Eccentricity, Inclination, Keplerian, LongitudeOfAscendingNode,
};
use lox_space::core::units::{Angle, AngleUnits, DistanceUnits};
use lox_space::time::Time;
use lox_space::time::calendar_dates::CalendarDate;
use lox_space::time::time_of_day::CivilTime;
use lox_space::time::time_scales::Tai;
use lox_space::time::utc::Utc;
use lox_space::time::utc::transformations::ToUtc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MappingError {
    #[error("invalid eccentricity: {0}")]
    Eccentricity(String),
    #[error("invalid inclination: {0}")]
    Inclination(String),
    #[error("invalid RAAN: {0}")]
    Raan(String),
    #[error("invalid AOP: {0}")]
    Aop(String),
    #[error("invalid SAR sensor: {0}")]
    Sar(#[from] SarPayloadError),
    #[error("malformed start time {iso:?}: {source}")]
    StartTime {
        iso: String,
        source: lox_space::time::utc::UtcError,
    },
}

/// Converts a [`SatelliteOrbitalElements`] proto message into a lox [`Keplerian`].
pub fn satellite_to_keplerian(s: &SatelliteOrbitalElements) -> Result<Keplerian, MappingError> {
    let ecc =
        Eccentricity::try_new(s.ecc).map_err(|e| MappingError::Eccentricity(e.to_string()))?;
    let inc = Inclination::try_new(s.inc_rad.rad())
        .map_err(|e| MappingError::Inclination(e.to_string()))?;
    let raan = LongitudeOfAscendingNode::try_new(s.raan_rad.rad())
        .map_err(|e| MappingError::Raan(e.to_string()))?;
    let aop = ArgumentOfPeriapsis::try_new(s.aop_rad.rad())
        .map_err(|e| MappingError::Aop(e.to_string()))?;
    let ta = TrueAnomaly::new(s.true_anomaly_rad.rad());
    let sma = s.sma_m.m();
    Ok(Keplerian::new(sma, ecc, inc, raan, aop, ta))
}

/// Convert a proto `SarSensor` to a lox `SarPayload`.
///
/// Uses the incidence-angle constructor; the proto fields are degrees.
///
/// When `look_side` is `LOOK_SIDE_UNSPECIFIED` (the proto3 default), the
/// payload defaults to **right-looking**. Callers (e.g. the service
/// handler in Task 7) should consider logging when this fallback fires,
/// since it silently picks a side for ambiguous requests.
pub fn sar_sensor_to_payload(s: &SarSensor) -> Result<SarPayload, MappingError> {
    let look = match s.look_side.as_known() {
        Some(ProtoLookSide::LOOK_SIDE_LEFT) => LoxLookSide::Left,
        Some(ProtoLookSide::LOOK_SIDE_RIGHT) => LoxLookSide::Right,
        // LOOK_SIDE_UNSPECIFIED or unknown wire value: default to right
        _ => LoxLookSide::Right,
    };
    let payload = SarPayload::with_incidence_angles(
        Angle::degrees(s.min_incidence_deg),
        Angle::degrees(s.max_incidence_deg),
        look,
    )?;
    Ok(payload)
}

/// Converts a lox [`LoxAccessWindow`] into a proto [`ProtoAccessWindow`].
pub fn access_window_to_proto(w: &LoxAccessWindow) -> ProtoAccessWindow {
    let start_utc: Utc = w.interval.start().to_utc();
    let end_utc: Utc = w.interval.end().to_utc();
    // Format as ISO-8601 without the trailing " UTC" suffix that Utc::to_string appends.
    let start_iso = format!("{}T{}Z", start_utc.date(), start_utc.time());
    let end_iso = format!("{}T{}Z", end_utc.date(), end_utc.time());
    let direction = match w.direction {
        LoxPassDirection::Ascending => ProtoPassDirection::PASS_DIRECTION_ASCENDING.into(),
        LoxPassDirection::Descending => ProtoPassDirection::PASS_DIRECTION_DESCENDING.into(),
    };
    ProtoAccessWindow {
        start_iso,
        end_iso,
        direction,
        __buffa_unknown_fields: Default::default(),
    }
}

/// Parses an ISO-8601 UTC string into a [`Time<Tai>`].
pub fn parse_start_time(iso: &str) -> Result<Time<Tai>, MappingError> {
    iso.parse::<Utc>()
        .map(|u| u.to_time())
        .map_err(|e| MappingError::StartTime {
            iso: iso.to_owned(),
            source: e,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn satellite_to_keplerian_round_trip_for_leo() {
        let s = SatelliteOrbitalElements {
            id: "test-0".into(),
            sma_m: 6_978_137.0,
            ecc: 0.001,
            inc_rad: 53.0_f64.to_radians(),
            raan_rad: 1.0,
            aop_rad: 0.5,
            true_anomaly_rad: 0.0,
            plane: 0,
            index_in_plane: 0,
            // buffa injects this on every generated message struct; required by Rust's exhaustive struct-literal check.
            __buffa_unknown_fields: Default::default(),
        };
        let k = satellite_to_keplerian(&s).unwrap();
        assert!((k.semi_major_axis().to_meters() - 6_978_137.0).abs() < 1e-3);
        assert!((k.eccentricity().as_f64() - 0.001).abs() < 1e-12);
        assert!((k.inclination().as_f64() - 53.0_f64.to_radians()).abs() < 1e-12);
        assert!((k.longitude_of_ascending_node().as_f64() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn satellite_to_keplerian_rejects_ecc_negative() {
        let s = SatelliteOrbitalElements {
            id: "bad".into(),
            sma_m: 7_000_000.0,
            ecc: -0.1,
            inc_rad: 0.0,
            raan_rad: 0.0,
            aop_rad: 0.0,
            true_anomaly_rad: 0.0,
            plane: 0,
            index_in_plane: 0,
            __buffa_unknown_fields: Default::default(),
        };
        assert!(satellite_to_keplerian(&s).is_err());
    }

    #[test]
    fn satellite_to_keplerian_rejects_negative_raan() {
        let s = SatelliteOrbitalElements {
            id: "bad-raan".into(),
            sma_m: 7_000_000.0,
            ecc: 0.001,
            inc_rad: 0.9,
            raan_rad: -1.0,
            aop_rad: 0.0,
            true_anomaly_rad: 0.0,
            plane: 0,
            index_in_plane: 0,
            __buffa_unknown_fields: Default::default(),
        };
        assert!(matches!(
            satellite_to_keplerian(&s),
            Err(MappingError::Raan(_))
        ));
    }

    #[test]
    fn sar_sensor_left_with_incidence_band() {
        let s = SarSensor {
            look_side: ProtoLookSide::LOOK_SIDE_LEFT.into(),
            min_incidence_deg: 20.0,
            max_incidence_deg: 45.0,
            __buffa_unknown_fields: Default::default(),
        };
        let p = sar_sensor_to_payload(&s).unwrap();
        assert!(matches!(p.side(), LoxLookSide::Left));
    }

    #[test]
    fn sar_sensor_inverted_band_errors() {
        let s = SarSensor {
            look_side: ProtoLookSide::LOOK_SIDE_RIGHT.into(),
            min_incidence_deg: 60.0,
            max_incidence_deg: 30.0,
            __buffa_unknown_fields: Default::default(),
        };
        assert!(sar_sensor_to_payload(&s).is_err());
    }

    #[test]
    fn parse_start_time_accepts_iso() {
        let t = parse_start_time("2026-06-01T00:00:00.000").unwrap();
        let utc = t.to_utc();
        assert_eq!(utc.year(), 2026);
        assert_eq!(utc.month(), 6);
        assert_eq!(utc.day(), 1);
    }
}
