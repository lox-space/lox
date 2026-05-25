// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Protobuf contract for the Kerolox compute engine.
//!
//! Phase 1 defines the schema only; no service is implemented yet.

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/kerolox.v1.rs"));
}

#[cfg(test)]
mod tests {
    use super::v1::*;

    #[test]
    fn access_request_can_be_constructed_with_defaults() {
        let req = AccessRequest::default();
        assert_eq!(req.start_time_iso, "");
        assert_eq!(req.duration_seconds, 0.0);
        assert!(req.satellites.is_empty());
        assert!(req.aoi_ids.is_empty());
        assert!(req.comparators.is_empty());
    }

    #[test]
    fn access_pair_result_has_source_field() {
        let r = AccessPairResult::default();
        assert_eq!(r.source(), ResultSource::Unspecified);
        assert!(r.comparator_id.is_empty());
    }

    #[test]
    fn satellite_orbital_elements_round_trip() {
        let s = SatelliteOrbitalElements {
            id: "sat-1".into(),
            sma_m: 7_000_000.0,
            ecc: 0.001,
            inc_rad: 0.9,
            raan_rad: 0.1,
            aop_rad: 0.0,
            true_anomaly_rad: 0.0,
            plane: 0,
            index_in_plane: 0,
        };
        let cloned = s.clone();
        assert_eq!(cloned.id, "sat-1");
        assert!((cloned.sma_m - 7_000_000.0).abs() < 1e-9);
    }
}
