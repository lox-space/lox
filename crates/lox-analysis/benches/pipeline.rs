// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! What the pipeline's shape costs and saves.
//!
//! Written to compare against the eager implementation before it was deleted;
//! those arms are gone, and what remains measures the two things the eager path
//! could not express at all:
//!
//! - `windows` versus `passes` — observables were roughly half again the cost of
//!   the windows alone, which is why a windows-only entry point exists.
//! - `first_pass_only` — stopping after one item, which an eager scan cannot do.
//! - `sequential` versus `rayon` — parallelism as the caller's explicit choice
//!   rather than a hard-coded pair-count threshold.
//!
//! Run with `cargo bench -p lox-analysis --bench pipeline`.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use divan::{Bencher, black_box};
use lox_analysis::assets::{AssetId, GroundStation, Scenario, Spacecraft};
use lox_analysis::pipeline::Parallelism;
use lox_analysis::visibility::ElevationMask;
use lox_analysis::visibility::VisibilityAnalysis;
use lox_bodies::Origin;
use lox_core::coords::LonLatAlt;
use lox_ephem::spk::parser::Spk;
use lox_frames::Frame;
use lox_orbits::ground::GroundLocation;
use lox_orbits::orbits::{Ensemble, Trajectory};
use lox_orbits::propagators::sgp4::{Elements, Sgp4};
use lox_orbits::propagators::{OrbitSource, Propagator};
use lox_test_utils::{data_file, read_data_file};
use lox_time::deltas::TimeDelta;
use lox_time::intervals::{Interval, TimeInterval};
use lox_time::time_scales::TimeScale;

fn main() {
    LazyLock::force(&SINGLE_PAIR);
    LazyLock::force(&EPHEMERIS);
    divan::main();
}

type Fixture = (
    Scenario<Origin, Frame>,
    Ensemble<AssetId, Origin, Frame>,
    Vec<GroundStation>,
    Vec<Spacecraft>,
    TimeInterval,
);

static EPHEMERIS: LazyLock<Arc<Spk>> =
    LazyLock::new(|| Arc::new(Spk::from_file(data_file("spice/de440s.bsp")).unwrap()));

fn station(name: &str, lon: f64, lat: f64) -> GroundStation {
    let coords = LonLatAlt::from_degrees(lon, lat, 0.0).unwrap();
    let location = GroundLocation::try_new(coords, Origin::Earth).unwrap();
    GroundStation::new(name, location, ElevationMask::with_fixed_elevation(0.0))
}

fn assemble(stations: Vec<GroundStation>, trajectories: Vec<(String, Trajectory)>) -> Fixture {
    let spacecraft: Vec<Spacecraft> = trajectories
        .iter()
        .map(|(id, traj)| Spacecraft::new(id.clone(), OrbitSource::Trajectory(traj.clone())))
        .collect();
    let interval = TimeInterval::new(trajectories[0].1.start_time(), trajectories[0].1.end_time());
    let scenario = Scenario::with_interval(interval, Origin::Earth, Frame::Icrf)
        .with_ground_stations(&stations)
        .with_spacecraft(&spacecraft);
    let ensemble = Ensemble::new(
        trajectories
            .into_iter()
            .map(|(id, traj)| {
                let (epoch, origin, frame, data) = traj.into_parts();
                (
                    AssetId::new(id),
                    Trajectory::from_parts(epoch.with_scale(TimeScale::Tai), origin, frame, data),
                )
            })
            .collect::<HashMap<_, _>>(),
    );
    (scenario, ensemble, stations, spacecraft, interval)
}

/// The same lunar arc the existing visibility bench uses, so the numbers here
/// sit alongside that baseline.
static SINGLE_PAIR: LazyLock<Fixture> = LazyLock::new(|| {
    let traj = Trajectory::from_csv_dynamic(
        &read_data_file("trajectory_lunar.csv"),
        Origin::Earth,
        Frame::Icrf,
    )
    .unwrap();
    assemble(
        vec![station("cebreros", -4.3676, 40.4527)],
        vec![("lunar".to_string(), traj)],
    )
});

/// Six stations by six LEO spacecraft: 36 pairs, enough that fan-out dominates
/// per-pair cost.
static MANY_PAIRS: LazyLock<Fixture> = LazyLock::new(|| {
    let tle = Elements::from_tle(
        Some("ISS (ZARYA)".to_string()),
        b"1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996",
        b"2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731",
    )
    .unwrap();
    let sgp4 = Sgp4::new(tle).unwrap();
    let t0 = sgp4.time();
    let base = sgp4
        .with_step(TimeDelta::from_seconds(30))
        .propagate(Interval::new(t0, t0 + TimeDelta::from_hours(12)).into_dynamic())
        .unwrap()
        .into_dynamic();

    let stations = vec![
        station("cebreros", -4.3676, 40.4527),
        station("kiruna", 20.9647, 67.8574),
        station("santiago", -70.6667, -33.1500),
        station("dongara", 115.3489, -29.0455),
        station("malindi", 40.1944, -2.9956),
        station("kourou", -52.8047, 5.2517),
    ];
    // Six copies of one orbit: the point is fan-out width, not orbital variety,
    // and identical inputs keep per-pair cost constant across the batch.
    let trajectories = (0..6)
        .map(|i| (format!("sc{i}"), base.clone()))
        .collect::<Vec<_>>();
    assemble(stations, trajectories)
});

const STEP: TimeDelta = TimeDelta::from_seconds(60);

// ---------------------------------------------------------------------------
// Single pair: intervals versus passes
// ---------------------------------------------------------------------------

/// The pipeline equivalent of `eager_passes`.
#[divan::bench]
fn pipeline_passes(bencher: Bencher) {
    let (scenario, ensemble, stations, spacecraft, interval) = &*SINGLE_PAIR;
    let analysis = VisibilityAnalysis::new(scenario, ensemble, EPHEMERIS.as_ref()).with_step(STEP);
    bencher.bench(|| {
        black_box(
            analysis
                .single(&stations[0], &spacecraft[0], *interval)
                .unwrap(),
        )
    });
}

/// The pipeline equivalent of `eager_intervals`.
#[divan::bench]
fn pipeline_windows(bencher: Bencher) {
    let (scenario, ensemble, stations, spacecraft, interval) = &*SINGLE_PAIR;
    let analysis = VisibilityAnalysis::new(scenario, ensemble, EPHEMERIS.as_ref()).with_step(STEP);
    bencher.bench(|| {
        black_box(
            analysis
                .windows(&stations[0], &spacecraft[0], *interval)
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
        )
    });
}

/// Laziness that the eager path structurally cannot express: stop after the
/// first pass. Should cost a fraction of the full scan.
#[divan::bench]
fn pipeline_first_pass_only(bencher: Bencher) {
    let (scenario, ensemble, stations, spacecraft, interval) = &*SINGLE_PAIR;
    let analysis = VisibilityAnalysis::new(scenario, ensemble, EPHEMERIS.as_ref()).with_step(STEP);
    bencher.bench(|| {
        black_box(
            analysis
                .passes(&stations[0], &spacecraft[0], *interval)
                .next(),
        )
    });
}

// ---------------------------------------------------------------------------
// Fan-out: 36 pairs
// ---------------------------------------------------------------------------

#[divan::bench]
fn pipeline_many_pairs_sequential(bencher: Bencher) {
    let (scenario, ensemble, _, _, interval) = &*MANY_PAIRS;
    let analysis = VisibilityAnalysis::new(scenario, ensemble, EPHEMERIS.as_ref()).with_step(STEP);
    bencher.bench(|| black_box(analysis.run(*interval, Parallelism::Sequential)));
}

#[divan::bench]
fn pipeline_many_pairs_rayon(bencher: Bencher) {
    let (scenario, ensemble, _, _, interval) = &*MANY_PAIRS;
    let analysis = VisibilityAnalysis::new(scenario, ensemble, EPHEMERIS.as_ref()).with_step(STEP);
    bencher.bench(|| black_box(analysis.run(*interval, Parallelism::Rayon(None))));
}
