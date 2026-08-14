// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Compares the eager and lazy ground-to-space visibility detection paths.

use std::collections::HashMap;
use std::sync::LazyLock;

use lox_analysis::assets::{AssetId, GroundStation, Scenario, Spacecraft};
use lox_analysis::visibility::{ElevationMask, VisibilityAnalysis};
use lox_bodies::Origin;
use lox_core::coords::LonLatAlt;
use lox_core::units::Angle;
use lox_frames::Frame;
use lox_orbits::ground::EllipsoidLocation;
use lox_orbits::orbits::{Ensemble, Trajectory};
use lox_orbits::propagators::OrbitSource;
use lox_test_utils::read_data_file;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::TimeScale;

type Fixture = (Scenario<Origin, Frame>, Ensemble<AssetId, Origin, Frame>);

static FIXTURE: LazyLock<Fixture> = LazyLock::new(|| {
    let sc_traj = Trajectory::from_csv_dynamic(
        &read_data_file("trajectory_lunar.csv"),
        Origin::Earth,
        Frame::Icrf,
    )
    .unwrap();
    let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
    let gs_loc = EllipsoidLocation::try_new(coords, Frame::Iau(Origin::Earth)).unwrap();
    let mask = ElevationMask::with_fixed_elevation(Angle::ZERO);
    let gs = GroundStation::new("cebreros", gs_loc, mask);
    let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(sc_traj.clone()));

    let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
    let scenario = Scenario::with_interval(interval, Origin::Earth, Frame::Icrf)
        .with_ground_stations(&[gs])
        .with_spacecraft(std::slice::from_ref(&sc));

    let (epoch, origin, frame, data) = sc_traj.into_parts();
    let typed = Trajectory::from_parts(epoch.with_scale(TimeScale::Tai), origin, frame, data);
    let ensemble = Ensemble::new(HashMap::from([(sc.id().clone(), typed)]));
    (scenario, ensemble)
});

const STEPS: [i64; 3] = [10, 60, 300];

#[divan::bench(args = STEPS)]
fn uniform(bencher: divan::Bencher, step: i64) {
    let (scenario, ensemble) = &*FIXTURE;
    bencher.bench(|| {
        VisibilityAnalysis::new(scenario, ensemble)
            .with_step(TimeDelta::from_seconds(step))
            .compute()
            .unwrap()
    });
}

#[divan::bench(args = STEPS)]
fn adaptive(bencher: divan::Bencher, step: i64) {
    let (scenario, ensemble) = &*FIXTURE;
    bencher.bench(|| {
        VisibilityAnalysis::new(scenario, ensemble)
            .with_step(TimeDelta::from_seconds(step))
            .with_adaptive_detection()
            .compute()
            .unwrap()
    });
}

fn main() {
    LazyLock::force(&FIXTURE);
    divan::main();
}
