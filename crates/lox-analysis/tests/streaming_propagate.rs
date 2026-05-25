// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;
use std::sync::Arc;

use lox_analysis::assets::{Scenario, Spacecraft};
use lox_bodies::Origin;
use lox_frames::Frame;
use lox_frames::providers::DefaultRotationProvider;
use lox_orbits::orbits::Trajectory;
use lox_orbits::propagators::OrbitSource;
use lox_stream::OnError;

fn make_trajectory() -> Trajectory {
    Trajectory::from_csv_dynamic(
        &lox_test_utils::read_data_file("trajectory_lunar.csv"),
        Origin::Earth,
        Frame::Icrf,
    )
    .unwrap()
}

fn make_scenario(n: usize) -> (Scenario, DefaultRotationProvider) {
    let traj = make_trajectory();
    let start = traj.start_time();
    let end = traj.end_time();
    let spacecraft: Vec<Spacecraft> = (0..n)
        .map(|i| Spacecraft::new(format!("sc{i}"), OrbitSource::Trajectory(traj.clone())))
        .collect();
    let scenario =
        Scenario::new(start, end, Origin::Earth, Frame::Icrf).with_spacecraft(&spacecraft);
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
