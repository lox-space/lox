// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![cfg(feature = "stream")]

use std::collections::HashMap;
use std::sync::Arc;

use lox_analysis::assets::{AssetId, GroundStation, Scenario, Spacecraft};
use lox_analysis::visibility::{ElevationMask, PairResult, PairType, VisibilityAnalysis};
use lox_bodies::DynOrigin;
use lox_core::coords::LonLatAlt;
use lox_ephem::spk::parser::Spk;
use lox_frames::DynFrame;
use lox_orbits::ground::GroundLocation;
use lox_orbits::orbits::{DynTrajectory, Ensemble, Trajectory};
use lox_orbits::propagators::OrbitSource;
use lox_stream::OnError;
use lox_test_utils::{data_file, read_data_file};
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::Tai;

fn make_trajectory() -> DynTrajectory {
    DynTrajectory::from_csv_dyn(
        &read_data_file("trajectory_lunar.csv"),
        DynOrigin::Earth,
        DynFrame::Icrf,
    )
    .unwrap()
}

fn make_ground_station(id: &str) -> GroundStation {
    let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
    let loc = GroundLocation::try_new(coords, DynOrigin::Earth).unwrap();
    let mask = ElevationMask::with_fixed_elevation(0.0);
    GroundStation::new(id, loc, mask)
}

fn make_spacecraft(id: &str, traj: DynTrajectory) -> Spacecraft {
    Spacecraft::new(id, OrbitSource::Trajectory(traj))
}

fn make_ensemble(spacecraft: &[Spacecraft]) -> Ensemble<AssetId, Tai, DynOrigin, DynFrame> {
    let mut map = HashMap::new();
    for sc in spacecraft {
        if let OrbitSource::Trajectory(traj) = sc.orbit() {
            let (epoch, origin, frame, data) = traj.clone().into_parts();
            let typed = Trajectory::from_parts(epoch.with_scale(Tai), origin, frame, data);
            map.insert(sc.id().clone(), typed);
        }
    }
    Ensemble::new(map)
}

fn make_scenario(
    ground_stations: &[GroundStation],
    spacecraft: &[Spacecraft],
    traj: &DynTrajectory,
) -> Scenario<DynOrigin, DynFrame> {
    let start = traj.start_time().to_scale(Tai);
    let end = traj.end_time().to_scale(Tai);
    Scenario::new(start, end, DynOrigin::Earth, DynFrame::Icrf)
        .with_ground_stations(ground_stations)
        .with_spacecraft(spacecraft)
}

fn ephemeris() -> Spk {
    Spk::from_file(data_file("spice/de440s.bsp")).unwrap()
}

#[test]
fn streamed_results_equal_batch() {
    let traj = make_trajectory();
    let gs = make_ground_station("cebreros");
    let sc = make_spacecraft("lunar", traj.clone());
    let ground_assets = [gs];
    let space_assets = [sc];
    let scenario = make_scenario(&ground_assets, &space_assets, &traj);
    let ensemble = make_ensemble(&space_assets);
    let spk = ephemeris();

    let arc_scenario = Arc::new(scenario);
    let arc_ensemble = Arc::new(ensemble);
    let arc_ephemeris = Arc::new(spk);

    let analysis_batch = VisibilityAnalysis::new(arc_scenario.as_ref(), arc_ensemble.as_ref())
        .with_occulting_bodies(arc_ephemeris.as_ref(), vec![]);
    let batch = analysis_batch.compute().unwrap();

    let analysis_stream = VisibilityAnalysis::new(arc_scenario.as_ref(), arc_ensemble.as_ref())
        .with_occulting_bodies(arc_ephemeris.as_ref(), vec![]);
    let mut s = analysis_stream.compute_stream(
        arc_scenario.clone(),
        arc_ensemble.clone(),
        arc_ephemeris.clone(),
        8,
        OnError::Continue,
    );

    let mut streamed: HashMap<(AssetId, AssetId), Vec<TimeInterval<Tai>>> = HashMap::new();
    while let Some(item) = s.blocking_next() {
        let PairResult {
            id1,
            id2,
            intervals,
            ..
        } = item.unwrap();
        streamed.insert((id1, id2), intervals);
    }

    assert_eq!(streamed.len(), batch.num_pairs());
    for ((id1, id2), intervals) in streamed {
        let batch_iv = batch.intervals_for(&id1, &id2).expect("missing in batch");
        assert_eq!(intervals.len(), batch_iv.len());
    }
}

#[test]
fn filter_excludes_unwanted_pairs() {
    // 1 ground station + 4 spacecraft. Filter allows only "sc1" and "sc2".
    let traj = make_trajectory();
    let gs = make_ground_station("gs1");
    let sc1 = make_spacecraft("sc1", traj.clone());
    let sc2 = make_spacecraft("sc2", traj.clone());
    let sc3 = make_spacecraft("sc3", traj.clone());
    let sc4 = make_spacecraft("sc4", traj.clone());
    let ground_assets = [gs];
    let space_assets = [sc1, sc2, sc3, sc4];
    let scenario = make_scenario(&ground_assets, &space_assets, &traj);
    let ensemble = make_ensemble(&space_assets);
    let spk = ephemeris();

    let arc_scenario = Arc::new(scenario);
    let arc_ensemble = Arc::new(ensemble);
    let arc_ephemeris = Arc::new(spk);

    let analysis = VisibilityAnalysis::new(arc_scenario.as_ref(), arc_ensemble.as_ref())
        .with_occulting_bodies(arc_ephemeris.as_ref(), vec![])
        .with_ground_space_filter(|_gs, sc| matches!(sc.id().as_str(), "sc1" | "sc2"));

    let mut s = analysis.compute_stream(
        arc_scenario.clone(),
        arc_ensemble.clone(),
        arc_ephemeris.clone(),
        8,
        OnError::Continue,
    );
    let mut count = 0usize;
    while let Some(item) = s.blocking_next() {
        let _ = item.unwrap();
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn mixed_pair_types_interleave() {
    // Scenario with both ground stations AND inter-satellite enabled.
    let traj = make_trajectory();
    let gs = make_ground_station("cebreros");
    let sc1 = make_spacecraft("sc1", traj.clone());
    let sc2 = make_spacecraft("sc2", traj.clone());
    let ground_assets = [gs];
    let space_assets = [sc1, sc2];
    let scenario = make_scenario(&ground_assets, &space_assets, &traj);
    let ensemble = make_ensemble(&space_assets);
    let spk = ephemeris();

    let arc_scenario = Arc::new(scenario);
    let arc_ensemble = Arc::new(ensemble);
    let arc_ephemeris = Arc::new(spk);

    let analysis = VisibilityAnalysis::new(arc_scenario.as_ref(), arc_ensemble.as_ref())
        .with_occulting_bodies(arc_ephemeris.as_ref(), vec![])
        .with_inter_satellite();

    let mut s = analysis.compute_stream(
        arc_scenario.clone(),
        arc_ensemble.clone(),
        arc_ephemeris.clone(),
        8,
        OnError::Continue,
    );

    let mut saw_gs = false;
    let mut saw_is = false;
    while let Some(item) = s.blocking_next() {
        let pr = item.unwrap();
        match pr.pair_type {
            PairType::GroundSpace => saw_gs = true,
            PairType::InterSatellite => saw_is = true,
        }
    }
    assert!(saw_gs);
    assert!(saw_is);
}

#[test]
fn drop_stops_workers() {
    use std::time::Duration;

    // Use a scenario with 2 spacecraft and an inter-satellite check enabled so
    // there is at least something to compute; we drop the stream immediately.
    let traj = make_trajectory();
    let gs = make_ground_station("cebreros");
    let sc1 = make_spacecraft("sc1", traj.clone());
    let sc2 = make_spacecraft("sc2", traj.clone());
    let ground_assets = [gs];
    let space_assets = [sc1, sc2];
    let scenario = make_scenario(&ground_assets, &space_assets, &traj);
    let ensemble = make_ensemble(&space_assets);
    let spk = ephemeris();

    let arc_scenario = Arc::new(scenario);
    let arc_ensemble = Arc::new(ensemble);
    let arc_ephemeris = Arc::new(spk);

    let analysis = VisibilityAnalysis::new(arc_scenario.as_ref(), arc_ensemble.as_ref())
        .with_occulting_bodies(arc_ephemeris.as_ref(), vec![])
        .with_inter_satellite();

    let s = analysis.compute_stream(
        arc_scenario.clone(),
        arc_ensemble.clone(),
        arc_ephemeris.clone(),
        4,
        OnError::Continue,
    );
    let token = s.token();
    drop(s);
    std::thread::sleep(Duration::from_millis(100));
    assert!(token.is_cancelled());
}

// A test-only ephemeris implementation that panics on every state lookup.
struct PanickingEphemeris;

impl lox_ephem::Ephemeris for PanickingEphemeris {
    type Error = std::convert::Infallible;

    fn state<O1: lox_bodies::Origin, O2: lox_bodies::Origin>(
        &self,
        _time: lox_time::Time<lox_time::time_scales::Tdb>,
        _origin: O1,
        _target: O2,
    ) -> Result<lox_core::coords::Cartesian, Self::Error> {
        panic!("injected ephemeris panic");
    }
}

#[test]
fn panic_in_detector_surfaces_as_worker_panicked() {
    use lox_analysis::visibility::VisibilityError;

    // Build a scenario with one gs + one spacecraft, using a PanickingEphemeris
    // with an occulting body so the ephemeris is actually invoked during detection.
    let traj = make_trajectory();
    let gs = make_ground_station("cebreros");
    let sc = make_spacecraft("lunar", traj.clone());
    let ground_assets = [gs];
    let space_assets = [sc];
    let scenario = make_scenario(&ground_assets, &space_assets, &traj);
    let ensemble = make_ensemble(&space_assets);

    let arc_scenario = Arc::new(scenario);
    let arc_ensemble = Arc::new(ensemble);
    let arc_ephemeris = Arc::new(PanickingEphemeris);

    // With Moon as occulting body, the ephemeris is queried during LOS detection.
    let analysis = VisibilityAnalysis::new(arc_scenario.as_ref(), arc_ensemble.as_ref())
        .with_occulting_bodies(arc_ephemeris.as_ref(), vec![DynOrigin::Moon]);

    let mut s = analysis.compute_stream(
        arc_scenario.clone(),
        arc_ensemble.clone(),
        arc_ephemeris.clone(),
        8,
        OnError::Continue,
    );

    let mut found_panic = false;
    while let Some(item) = s.blocking_next() {
        if let Err(VisibilityError::WorkerPanicked { .. }) = item {
            found_panic = true;
        }
    }
    assert!(
        found_panic,
        "expected WorkerPanicked error from panicking ephemeris"
    );
}
