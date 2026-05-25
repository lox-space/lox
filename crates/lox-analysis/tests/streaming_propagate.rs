// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![cfg(feature = "stream")]

use std::collections::HashMap;
use std::sync::Arc;

use lox_analysis::assets::{DynScenario, ScenarioPropagateError, Spacecraft};
use lox_frames::DynFrame;
use lox_frames::providers::DefaultRotationProvider;
use lox_orbits::orbits::DynTrajectory;
use lox_orbits::propagators::OrbitSource;
use lox_stream::OnError;
use lox_time::time_scales::Tai;

fn make_trajectory() -> DynTrajectory {
    use lox_bodies::DynOrigin;
    DynTrajectory::from_csv_dyn(
        &lox_test_utils::read_data_file("trajectory_lunar.csv"),
        DynOrigin::Earth,
        DynFrame::Icrf,
    )
    .unwrap()
}

fn make_scenario(n: usize) -> (DynScenario, DefaultRotationProvider) {
    use lox_bodies::DynOrigin;
    let traj = make_trajectory();
    let start = traj.start_time().to_scale(Tai);
    let end = traj.end_time().to_scale(Tai);
    let spacecraft: Vec<Spacecraft> = (0..n)
        .map(|i| Spacecraft::new(format!("sc{i}"), OrbitSource::Trajectory(traj.clone())))
        .collect();
    let scenario =
        DynScenario::new(start, end, DynOrigin::Earth, DynFrame::Icrf).with_spacecraft(&spacecraft);
    (scenario, DefaultRotationProvider)
}

/// Build a scenario with `n_good` healthy spacecraft plus one spacecraft using
/// `bad_orbit` as its orbit source. The bad spacecraft has id `"bad"`.
fn make_mixed_scenario(
    n_good: usize,
    bad_orbit: OrbitSource,
) -> (DynScenario, DefaultRotationProvider) {
    use lox_bodies::DynOrigin;
    let traj = make_trajectory();
    let start = traj.start_time().to_scale(Tai);
    let end = traj.end_time().to_scale(Tai);
    let mut spacecraft: Vec<Spacecraft> = (0..n_good)
        .map(|i| Spacecraft::new(format!("sc{i}"), OrbitSource::Trajectory(traj.clone())))
        .collect();
    spacecraft.push(Spacecraft::new("bad", bad_orbit));
    let scenario =
        DynScenario::new(start, end, DynOrigin::Earth, DynFrame::Icrf).with_spacecraft(&spacecraft);
    (scenario, DefaultRotationProvider)
}

#[test]
fn streamed_results_equal_batch() {
    let (scenario, provider) = make_scenario(4);
    let arc_scenario = Arc::new(scenario);
    let arc_provider = Arc::new(provider);

    let batch = arc_scenario
        .clone()
        .as_ref()
        .propagate(&*arc_provider)
        .unwrap();
    let mut streamed =
        arc_scenario
            .clone()
            .propagate_stream(arc_provider.clone(), 16, OnError::Continue);

    let mut collected: HashMap<_, _> = HashMap::new();
    while let Some(item) = streamed.blocking_next() {
        let (id, traj) = item.unwrap();
        collected.insert(id, traj);
    }

    assert_eq!(collected.len(), batch.len());
    for (id, traj) in collected {
        let bt = batch.get(&id).expect("missing in batch");
        assert_eq!(traj.states().len(), bt.states().len());
    }
}

#[test]
fn drop_stops_workers() {
    use std::time::Duration;

    // A large scenario so not every spacecraft can be propagated before we drop.
    let (scenario, provider) = make_scenario(100);
    let arc_scenario = Arc::new(scenario);
    let arc_provider = Arc::new(provider);

    let stream = arc_scenario
        .clone()
        .propagate_stream(arc_provider.clone(), 4, OnError::Continue);

    let token = stream.token();
    drop(stream);
    // Give rayon a moment to observe cancellation.
    std::thread::sleep(Duration::from_millis(100));
    assert!(token.is_cancelled());
}

#[test]
fn continue_yields_per_spacecraft_errors() {
    // One spacecraft has a deliberately failing orbit source; N-1 are healthy.
    // Under OnError::Continue, we expect exactly one Err and N-1 Ok results.
    let n_good = 3;
    let bad_orbit = OrbitSource::TestError("injected error".to_string());
    let (scenario, provider) = make_mixed_scenario(n_good, bad_orbit);
    let arc_scenario = Arc::new(scenario);
    let arc_provider = Arc::new(provider);

    let mut s = arc_scenario.propagate_stream(arc_provider, 8, OnError::Continue);
    let mut oks = 0usize;
    let mut errs = 0usize;
    while let Some(item) = s.blocking_next() {
        match item {
            Ok(_) => oks += 1,
            Err(_) => errs += 1,
        }
    }
    assert_eq!(errs, 1);
    assert_eq!(oks, n_good);
}

#[test]
fn abort_terminates_after_first_error() {
    // One spacecraft fails; rest are healthy. Under OnError::Abort the stream
    // should terminate early — fewer than total OK results will be emitted.
    let total_spacecraft = 10_000;
    let n_good = total_spacecraft - 1;
    let bad_orbit = OrbitSource::TestError("abort trigger".to_string());
    let (scenario, provider) = make_mixed_scenario(n_good, bad_orbit);
    let arc_scenario = Arc::new(scenario);
    let arc_provider = Arc::new(provider);

    let mut s = arc_scenario.propagate_stream(arc_provider, 8, OnError::Abort);
    let mut errs = 0usize;
    let mut oks = 0usize;
    while let Some(item) = s.blocking_next() {
        match item {
            Ok(_) => oks += 1,
            Err(_) => errs += 1,
        }
    }
    assert!(errs >= 1);
    assert!(oks < n_good);
}

#[test]
fn panic_in_propagator_surfaces_as_worker_panicked() {
    // One spacecraft's orbit source panics during propagation. Verify the
    // panic is converted to ScenarioPropagateError::WorkerPanicked.
    let panicking_orbit = OrbitSource::TestPanic("test panic".to_string());
    let (scenario, provider) = make_mixed_scenario(2, panicking_orbit);
    let arc_scenario = Arc::new(scenario);
    let arc_provider = Arc::new(provider);

    let mut s = arc_scenario.propagate_stream(arc_provider, 8, OnError::Continue);
    let mut found_panic = false;
    while let Some(item) = s.blocking_next() {
        if let Err(ScenarioPropagateError::WorkerPanicked { id, message: _ }) = item {
            assert_eq!(id.as_str(), "bad");
            found_panic = true;
        }
    }
    assert!(found_panic, "expected one WorkerPanicked error");
}
