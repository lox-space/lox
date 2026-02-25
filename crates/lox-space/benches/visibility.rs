// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::sync::OnceLock;

use divan::Bencher;
use lox_space::bodies::DynOrigin;
use lox_space::core::coords::LonLatAlt;
use lox_space::ephem::spk::parser::Spk;
use lox_space::frames::DynFrame;
use lox_space::orbits::analysis::{ElevationMask, VisibilityAnalysis};
use lox_space::orbits::assets::{GroundAsset, SpaceAsset};
use lox_space::orbits::ground::GroundLocation;
use lox_space::orbits::orbits::DynTrajectory;
use lox_space::time::deltas::TimeDelta;
use lox_space::time::intervals::TimeInterval;

fn main() {
    divan::main();
}

fn ephemeris() -> &'static Spk {
    static EPHEMERIS: OnceLock<Spk> = OnceLock::new();
    EPHEMERIS.get_or_init(|| Spk::from_file(lox_test_utils::data_file("spice/de440s.bsp")).unwrap())
}

fn spacecraft_trajectory() -> DynTrajectory {
    DynTrajectory::from_csv_dyn(
        &lox_test_utils::read_data_file("trajectory_lunar.csv"),
        DynOrigin::Earth,
        DynFrame::Icrf,
    )
    .unwrap()
}

fn ground_location() -> GroundLocation<DynOrigin> {
    let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
    GroundLocation::try_new(coords, DynOrigin::Earth).unwrap()
}

fn setup() -> (
    [GroundAsset; 1],
    [SpaceAsset; 1],
    TimeInterval<lox_space::time::time_scales::DynTimeScale>,
) {
    let sc_traj = spacecraft_trajectory();
    let gs_loc = ground_location();
    let mask = ElevationMask::with_fixed_elevation(0.0);
    let gs = GroundAsset::new("cebreros", gs_loc, mask);
    let sc = SpaceAsset::new("lunar", sc_traj.clone());
    let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
    ([gs], [sc], interval)
}

#[divan::bench]
fn visibility_single_pair(bencher: Bencher) {
    let spk = ephemeris();
    let (ground_assets, space_assets, interval) = setup();

    bencher.bench(|| {
        let analysis = VisibilityAnalysis::new(&ground_assets, &space_assets, spk);
        analysis.compute(interval).unwrap()
    });
}

#[divan::bench]
fn visibility_single_pair_min_pass_5m(bencher: Bencher) {
    let spk = ephemeris();
    let (ground_assets, space_assets, interval) = setup();

    bencher.bench(|| {
        let analysis = VisibilityAnalysis::new(&ground_assets, &space_assets, spk)
            .with_min_pass_duration(TimeDelta::from_seconds(300));
        analysis.compute(interval).unwrap()
    });
}

#[divan::bench]
fn visibility_single_pair_with_los(bencher: Bencher) {
    let spk = ephemeris();
    let (ground_assets, space_assets, interval) = setup();

    bencher.bench(|| {
        let analysis = VisibilityAnalysis::new(&ground_assets, &space_assets, spk)
            .with_occulting_bodies(vec![DynOrigin::Moon]);
        analysis.compute(interval).unwrap()
    });
}

#[divan::bench]
fn visibility_single_pair_with_los_min_pass_5m(bencher: Bencher) {
    let spk = ephemeris();
    let (ground_assets, space_assets, interval) = setup();

    bencher.bench(|| {
        let analysis = VisibilityAnalysis::new(&ground_assets, &space_assets, spk)
            .with_occulting_bodies(vec![DynOrigin::Moon])
            .with_min_pass_duration(TimeDelta::from_seconds(300));
        analysis.compute(interval).unwrap()
    });
}
