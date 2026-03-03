// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::sync::OnceLock;

use divan::Bencher;
use lox_space::analysis::assets::{AssetId, DynScenario, GroundStation, Scenario, Spacecraft};
use lox_space::analysis::visibility::{ElevationMask, VisibilityAnalysis};
use lox_space::bodies::{DynOrigin, Earth};
use lox_space::core::coords::LonLatAlt;
use lox_space::ephem::spk::parser::Spk;
use lox_space::frames::providers::DefaultRotationProvider;
use lox_space::frames::{DynFrame, Icrf};
use lox_space::orbits::ground::GroundLocation;
use lox_space::orbits::orbits::{DynTrajectory, Ensemble, Trajectory};
use lox_space::orbits::propagators::OrbitSource;
use lox_space::time::deltas::TimeDelta;
use lox_space::time::intervals::TimeInterval;
use lox_space::time::time_scales::Tai;

fn main() {
    divan::main();
}

fn ephemeris() -> &'static Spk {
    static EPHEMERIS: OnceLock<Spk> = OnceLock::new();
    EPHEMERIS.get_or_init(|| Spk::from_file(lox_test_utils::data_file("spice/de440s.bsp")).unwrap())
}

// ---------------------------------------------------------------------------
// Dyn (dynamic dispatch) setup
// ---------------------------------------------------------------------------

fn spacecraft_trajectory_dyn() -> DynTrajectory {
    DynTrajectory::from_csv_dyn(
        &lox_test_utils::read_data_file("trajectory_lunar.csv"),
        DynOrigin::Earth,
        DynFrame::Icrf,
    )
    .unwrap()
}

fn ground_location_dyn() -> GroundLocation<DynOrigin> {
    let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
    GroundLocation::try_new(coords, DynOrigin::Earth).unwrap()
}

fn setup_dyn() -> (DynScenario, Ensemble<AssetId, Tai, DynOrigin, DynFrame>) {
    let sc_traj = spacecraft_trajectory_dyn();
    let gs_loc = ground_location_dyn();
    let mask = ElevationMask::with_fixed_elevation(0.0);
    let gs = GroundStation::new("cebreros", gs_loc, mask);
    let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
    let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(sc_traj));
    let scenario = DynScenario::new(
        interval.start().to_scale(Tai),
        interval.end().to_scale(Tai),
        DynOrigin::Earth,
        DynFrame::Icrf,
    )
    .with_spacecraft(&[sc])
    .with_ground_stations(&[gs]);
    let ensemble = scenario.propagate(&DefaultRotationProvider).unwrap();
    (scenario, ensemble)
}

// ---------------------------------------------------------------------------
// Monomorphic setup
// ---------------------------------------------------------------------------

fn spacecraft_trajectory_mono() -> Trajectory<Tai, Earth, Icrf> {
    Trajectory::from_csv(
        &lox_test_utils::read_data_file("trajectory_lunar.csv"),
        Earth,
        Icrf,
    )
    .unwrap()
}

fn ground_location_mono() -> GroundLocation<Earth> {
    let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
    GroundLocation::try_new(coords, Earth).unwrap()
}

fn setup_mono() -> (Scenario<Earth, Icrf>, Ensemble<AssetId, Tai, Earth, Icrf>) {
    let traj = spacecraft_trajectory_mono();
    let gs_loc = ground_location_mono();
    let mask = ElevationMask::with_fixed_elevation(0.0);
    let interval = TimeInterval::new(traj.start_time(), traj.end_time());
    let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(traj.into_dyn()));
    let gs = GroundStation::new("cebreros", gs_loc.into_dyn(), mask);
    let scenario = Scenario::new(interval.start(), interval.end(), Earth, Icrf)
        .with_spacecraft(&[sc])
        .with_ground_stations(&[gs]);
    let ensemble = scenario.propagate(&DefaultRotationProvider).unwrap();
    (scenario, ensemble)
}

// ---------------------------------------------------------------------------
// Dyn benchmarks
// ---------------------------------------------------------------------------

#[divan::bench]
fn visibility_single_pair(bencher: Bencher) {
    let spk = ephemeris();
    let (scenario, ensemble) = setup_dyn();

    bencher.bench(|| {
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk);
        analysis.compute().unwrap()
    });
}

#[divan::bench]
fn visibility_single_pair_min_pass_5m(bencher: Bencher) {
    let spk = ephemeris();
    let (scenario, ensemble) = setup_dyn();

    bencher.bench(|| {
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk)
            .with_min_pass_duration(TimeDelta::from_seconds(300));
        analysis.compute().unwrap()
    });
}

#[divan::bench]
fn visibility_single_pair_with_los(bencher: Bencher) {
    let spk = ephemeris();
    let (scenario, ensemble) = setup_dyn();

    bencher.bench(|| {
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk)
            .with_occulting_bodies(vec![DynOrigin::Moon]);
        analysis.compute().unwrap()
    });
}

#[divan::bench]
fn visibility_single_pair_with_los_min_pass_5m(bencher: Bencher) {
    let spk = ephemeris();
    let (scenario, ensemble) = setup_dyn();

    bencher.bench(|| {
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk)
            .with_occulting_bodies(vec![DynOrigin::Moon])
            .with_min_pass_duration(TimeDelta::from_seconds(300));
        analysis.compute().unwrap()
    });
}

// ---------------------------------------------------------------------------
// Monomorphic benchmarks (concrete types, no DynFrame dispatch)
// ---------------------------------------------------------------------------

#[divan::bench]
fn visibility_single_pair_mono(bencher: Bencher) {
    let spk = ephemeris();
    let (scenario, ensemble) = setup_mono();

    bencher.bench(|| {
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk);
        analysis.compute().unwrap()
    });
}

#[divan::bench]
fn visibility_single_pair_mono_min_pass_5m(bencher: Bencher) {
    let spk = ephemeris();
    let (scenario, ensemble) = setup_mono();

    bencher.bench(|| {
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk)
            .with_min_pass_duration(TimeDelta::from_seconds(300));
        analysis.compute().unwrap()
    });
}

#[divan::bench]
fn visibility_single_pair_mono_with_los(bencher: Bencher) {
    let spk = ephemeris();
    let (scenario, ensemble) = setup_mono();

    bencher.bench(|| {
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk)
            .with_occulting_bodies(vec![DynOrigin::Moon]);
        analysis.compute().unwrap()
    });
}

#[divan::bench]
fn visibility_single_pair_mono_with_los_min_pass_5m(bencher: Bencher) {
    let spk = ephemeris();
    let (scenario, ensemble) = setup_mono();

    bencher.bench(|| {
        let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk)
            .with_occulting_bodies(vec![DynOrigin::Moon])
            .with_min_pass_duration(TimeDelta::from_seconds(300));
        analysis.compute().unwrap()
    });
}
