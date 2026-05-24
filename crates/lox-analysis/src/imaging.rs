// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! AOI access analysis: sensor payloads, AOI geometry, and per-(spacecraft, AOI) access windows.

pub mod analysis;
pub mod aoi;
pub mod optical;
/// Access analysis result types.
pub mod results;

pub use analysis::{
    AccessAnalysis, AccessError, AccessPayload, OpticalAccessAnalysis, PayloadAccessor,
};
#[cfg(feature = "geojson")]
pub use aoi::AoiError;
pub use aoi::{Aoi, AoiId};
pub use optical::OpticalPayload;
pub use results::AccessResults;

#[cfg(test)]
mod tests {
    use super::*;
    use geo::{LineString, Polygon};
    use lox_bodies::DynOrigin;
    use lox_core::units::{Angle, Distance};
    use lox_frames::DynFrame;
    use lox_orbits::orbits::{DynTrajectory, Ensemble};
    use lox_orbits::propagators::OrbitSource;
    use lox_time::deltas::TimeDelta;
    use lox_time::intervals::TimeInterval;
    use lox_time::time_scales::{DynTimeScale, Tai};

    use crate::assets::{AssetId, Scenario};

    // -----------------------------------------------------------------------
    // Integration tests — full OpticalAccessAnalysis pipeline with Sentinel-2 TLEs
    // -----------------------------------------------------------------------

    /// Propagate a Sentinel-2 TLE into a DynTrajectory over a 6-hour window.
    fn sentinel2_trajectory(name: &str, line1: &[u8], line2: &[u8]) -> DynTrajectory {
        use lox_orbits::propagators::Propagator;
        use lox_orbits::propagators::sgp4::{Elements, Sgp4};
        use lox_time::intervals::Interval;

        let tle = Elements::from_tle(Some(name.to_string()), line1, line2).unwrap();
        let sgp4 = Sgp4::new(tle).unwrap();
        let t0 = sgp4.time();
        let t1 = t0 + TimeDelta::from_hours(6);
        sgp4.with_step(TimeDelta::from_seconds(10))
            .propagate(Interval::new(t0, t1))
            .unwrap()
            .into_dyn()
    }

    fn sentinel2a_trajectory() -> DynTrajectory {
        sentinel2_trajectory(
            "SENTINEL-2A",
            b"1 40697U 15028A   26079.19377485 -.00000072  00000+0 -11026-4 0  9994",
            b"2 40697  98.5642 155.3327 0001269  98.1407 261.9920 14.30816376561005",
        )
    }

    fn sentinel2b_trajectory() -> DynTrajectory {
        sentinel2_trajectory(
            "SENTINEL-2B",
            b"1 42063U 17013A   26079.18648189  .00000015  00000+0  22231-4 0  9995",
            b"2 42063  98.5694 155.2271 0001161  93.5553 266.5763 14.30810963471912",
        )
    }

    /// Build a Scenario + Ensemble from spacecraft with pre-computed trajectories.
    fn make_imaging_scenario(
        space_assets: &[crate::assets::Spacecraft],
        interval: TimeInterval<DynTimeScale>,
    ) -> (
        Scenario<DynOrigin, DynFrame>,
        Ensemble<AssetId, Tai, DynOrigin, DynFrame>,
    ) {
        let tai_interval =
            TimeInterval::new(interval.start().to_scale(Tai), interval.end().to_scale(Tai));
        let scenario = Scenario::with_interval(tai_interval, DynOrigin::Earth, DynFrame::Icrf)
            .with_spacecraft(space_assets);
        let mut map = std::collections::HashMap::new();
        for sc in space_assets {
            if let OrbitSource::Trajectory(traj) = sc.orbit() {
                let (epoch, origin, frame, data) = traj.clone().into_parts();
                let typed = lox_orbits::orbits::Trajectory::from_parts(
                    epoch.with_scale(Tai),
                    origin,
                    frame,
                    data,
                );
                map.insert(sc.id().clone(), typed);
            }
        }
        (scenario, Ensemble::new(map))
    }

    /// A large AOI covering most of Western Europe — Sentinel-2 SSO orbits
    /// will definitely overfly this within 6 hours.
    fn western_europe_aoi() -> Aoi {
        Aoi::new(Polygon::new(
            LineString::from(vec![
                (-10.0, 35.0),
                (20.0, 35.0),
                (20.0, 60.0),
                (-10.0, 60.0),
                (-10.0, 35.0),
            ]),
            vec![],
        ))
    }

    /// A small AOI in the middle of the Pacific — unlikely to be imaged
    /// from a sun-synchronous orbit in a short window.
    fn pacific_aoi() -> Aoi {
        Aoi::new(Polygon::new(
            LineString::from(vec![
                (-175.0, -5.0),
                (-174.0, -5.0),
                (-174.0, -4.0),
                (-175.0, -4.0),
                (-175.0, -5.0),
            ]),
            vec![],
        ))
    }

    #[test]
    fn test_imaging_params_nadir_only() {
        let params = OpticalPayload::nadir_only(Distance::kilometers(20.0));
        // Nadir-only: range is just half swath
        let range = params.max_accessible_ground_range(500_000.0, 6_371_000.0);
        assert!((range - 10_000.0).abs() < 1e-6);
    }

    #[test]
    fn test_imaging_params_off_nadir() {
        let params = OpticalPayload::off_nadir(Distance::kilometers(20.0), Angle::degrees(30.0));
        let range = params.max_accessible_ground_range(500_000.0, 6_371_000.0);
        // Should be > half swath (10 km) due to off-nadir contribution
        assert!(range > 10_000.0);
        // For 30° off-nadir at 500 km altitude, expect ~300-400 km total
        assert!(range > 200_000.0);
    }

    #[test]
    fn test_imaging_analysis_sentinel2_over_europe() {
        let traj = sentinel2a_trajectory();
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());

        // Sentinel-2: 290 km swath, nadir-only
        let payload = OpticalPayload::nadir_only(Distance::kilometers(290.0));
        let sc = crate::assets::Spacecraft::new("s2a", OrbitSource::Trajectory(traj))
            .with_imaging_payload(payload);

        let (scenario, ensemble) = make_imaging_scenario(std::slice::from_ref(&sc), interval);

        let aois = vec![(AoiId::new("europe"), western_europe_aoi())];

        let analysis = OpticalAccessAnalysis::new(&scenario, &ensemble, aois)
            .with_step(TimeDelta::from_seconds(30));
        let results = analysis.compute().expect("access analysis failed");

        let intervals = results.intervals(&AssetId::new("s2a"), &AoiId::new("europe"));
        // A sun-synchronous LEO satellite should overfly Western Europe
        // at least once in 6 hours.
        assert!(
            !intervals.is_empty(),
            "expected at least one access window over Western Europe"
        );

        // Each window should be short (a few minutes at most for a LEO pass).
        for iv in intervals {
            let duration_s = (iv.end() - iv.start()).to_seconds().to_f64();
            assert!(duration_s > 0.0, "zero-length interval");
            assert!(
                duration_s < 600.0,
                "access window too long ({duration_s:.0}s) — expected < 600s for a LEO pass"
            );
        }
    }

    #[test]
    fn test_imaging_analysis_no_payload_skips_spacecraft() {
        let traj = sentinel2a_trajectory();
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());

        // Spacecraft without an imaging payload
        let sc = crate::assets::Spacecraft::new("s2a", OrbitSource::Trajectory(traj));

        let (scenario, ensemble) = make_imaging_scenario(&[sc], interval);

        let aois = vec![(AoiId::new("europe"), western_europe_aoi())];

        let analysis = OpticalAccessAnalysis::new(&scenario, &ensemble, aois);
        let results = analysis.compute().expect("access analysis failed");

        assert!(
            results.is_empty(),
            "expected no results when spacecraft has no payload"
        );
    }

    #[test]
    fn test_imaging_analysis_multiple_spacecraft_and_aois() {
        let traj_a = sentinel2a_trajectory();
        let traj_b = sentinel2b_trajectory();
        let interval = TimeInterval::new(traj_a.start_time(), traj_a.end_time());

        let payload = OpticalPayload::nadir_only(Distance::kilometers(290.0));

        let sc_a = crate::assets::Spacecraft::new("s2a", OrbitSource::Trajectory(traj_a))
            .with_imaging_payload(payload);
        let sc_b = crate::assets::Spacecraft::new("s2b", OrbitSource::Trajectory(traj_b))
            .with_imaging_payload(payload);

        let (scenario, ensemble) = make_imaging_scenario(&[sc_a.clone(), sc_b.clone()], interval);

        let aois = vec![
            (AoiId::new("europe"), western_europe_aoi()),
            (AoiId::new("pacific"), pacific_aoi()),
        ];

        let analysis = OpticalAccessAnalysis::new(&scenario, &ensemble, aois)
            .with_step(TimeDelta::from_seconds(30));
        let results = analysis.compute().expect("access analysis failed");

        // Both spacecraft should have windows over the large European AOI.
        let s2a_europe = results.intervals(&AssetId::new("s2a"), &AoiId::new("europe"));
        let s2b_europe = results.intervals(&AssetId::new("s2b"), &AoiId::new("europe"));
        assert!(!s2a_europe.is_empty(), "S2A should image Europe");
        assert!(!s2b_europe.is_empty(), "S2B should image Europe");

        // The small Pacific AOI may or may not be hit. But the total result
        // count should cover all 4 pairs.
        assert_eq!(results.num_pairs(), 4);
    }

    #[test]
    fn test_imaging_off_nadir_wider_than_nadir() {
        let traj = sentinel2a_trajectory();
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());

        let nadir_payload = OpticalPayload::nadir_only(Distance::kilometers(290.0));
        let off_nadir_payload =
            OpticalPayload::off_nadir(Distance::kilometers(290.0), Angle::degrees(30.0));

        // Same trajectory, different payloads
        let sc_nadir =
            crate::assets::Spacecraft::new("nadir", OrbitSource::Trajectory(traj.clone()))
                .with_imaging_payload(nadir_payload);
        let sc_off_nadir =
            crate::assets::Spacecraft::new("off_nadir", OrbitSource::Trajectory(traj))
                .with_imaging_payload(off_nadir_payload);

        let (scenario, ensemble) = make_imaging_scenario(&[sc_nadir, sc_off_nadir], interval);

        let aois = vec![(AoiId::new("europe"), western_europe_aoi())];

        let analysis = OpticalAccessAnalysis::new(&scenario, &ensemble, aois)
            .with_step(TimeDelta::from_seconds(30));
        let results = analysis.compute().expect("access analysis failed");

        let nadir_intervals = results.intervals(&AssetId::new("nadir"), &AoiId::new("europe"));
        let off_nadir_intervals =
            results.intervals(&AssetId::new("off_nadir"), &AoiId::new("europe"));

        // Agile satellite should have at least as much total coverage time.
        let nadir_total: f64 = nadir_intervals
            .iter()
            .map(|iv| (iv.end() - iv.start()).to_seconds().to_f64())
            .sum();
        let off_nadir_total: f64 = off_nadir_intervals
            .iter()
            .map(|iv| (iv.end() - iv.start()).to_seconds().to_f64())
            .sum();
        assert!(
            off_nadir_total >= nadir_total - 1.0,
            "off-nadir ({off_nadir_total:.0}s) should have >= nadir ({nadir_total:.0}s) coverage"
        );
    }
}
