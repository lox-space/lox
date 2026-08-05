// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Visibility-analysis benchmarks.
//!
//! Ground-space single-pair cases establish the baseline; inter-satellite and
//! many-asset scaling cover the rest of the surface, including the rayon fan-out
//! that `Parallelism::Rayon` selects.
//!
//! Run with `cargo bench -p lox-space --bench visibility`.

use divan::{Bencher, black_box};
use lox_space::analysis::pipeline::{NoEphemeris, Parallelism};
use lox_space::analysis::visibility::{InterSatelliteAnalysis, VisibilityAnalysis};
use lox_space::bodies::Origin;
use lox_space::core::units::{AngularRate, Distance};
use lox_space::time::deltas::TimeDelta;

#[path = "common/mod.rs"]
mod common;

fn main() {
    divan::main();
}

// ---------------------------------------------------------------------------
// Ground-space single pair
// ---------------------------------------------------------------------------

/// Windows only — no observables. The cheap half of the analysis.
#[divan::bench]
fn visibility_single_pair_windows(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_dynamic();
    let gs = &scenario.ground_stations()[0];
    let sc = &scenario.spacecraft()[0];
    let interval = *scenario.interval();
    let analysis = VisibilityAnalysis::new(&scenario, &ensemble, &NoEphemeris);
    bencher.bench(|| {
        analysis
            .windows(gs, sc, interval)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    });
}

/// Passes — windows plus observables sampled across each one.
#[divan::bench]
fn visibility_single_pair(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_dynamic();
    let gs = &scenario.ground_stations()[0];
    let sc = &scenario.spacecraft()[0];
    let interval = *scenario.interval();
    let analysis = VisibilityAnalysis::new(&scenario, &ensemble, &NoEphemeris);
    bencher.bench(|| analysis.single(gs, sc, interval).unwrap());
}

/// Only the first pass: what laziness buys over an eager scan.
#[divan::bench]
fn visibility_first_pass_only(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_dynamic();
    let gs = &scenario.ground_stations()[0];
    let sc = &scenario.spacecraft()[0];
    let interval = *scenario.interval();
    let analysis = VisibilityAnalysis::new(&scenario, &ensemble, &NoEphemeris);
    bencher.bench(|| analysis.passes(gs, sc, interval).next());
}

#[divan::bench]
fn visibility_single_pair_min_pass_5m(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_dynamic();
    let gs = &scenario.ground_stations()[0];
    let sc = &scenario.spacecraft()[0];
    let interval = *scenario.interval();
    let analysis = VisibilityAnalysis::new(&scenario, &ensemble, &NoEphemeris)
        .with_min_pass_duration(TimeDelta::from_seconds(300));
    bencher.bench(|| analysis.single(gs, sc, interval).unwrap());
}

#[divan::bench]
fn visibility_single_pair_with_los(bencher: Bencher) {
    let spk = common::ephemeris();
    let (scenario, ensemble) = common::setup_dynamic();
    let gs = &scenario.ground_stations()[0];
    let sc = &scenario.spacecraft()[0];
    let interval = *scenario.interval();
    let analysis = VisibilityAnalysis::new(&scenario, &ensemble, spk)
        .with_occulting_bodies(vec![Origin::Moon]);
    bencher.bench(|| analysis.single(gs, sc, interval).unwrap());
}

// ---------------------------------------------------------------------------
// Inter-satellite single pair
// ---------------------------------------------------------------------------

fn intersat(
    scenario: &lox_space::analysis::assets::Scenario,
) -> (
    &lox_space::analysis::assets::Spacecraft,
    &lox_space::analysis::assets::Spacecraft,
) {
    let sc = scenario.spacecraft();
    (&sc[0], &sc[1])
}

#[divan::bench]
fn intersat_pair(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_intersat_pair(None);
    let (a, b) = intersat(&scenario);
    let interval = *scenario.interval();
    let analysis = InterSatelliteAnalysis::new(&scenario, &ensemble, &NoEphemeris);
    bencher.bench(|| analysis.single(a, b, interval).unwrap());
}

#[divan::bench]
fn intersat_pair_max_range(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_intersat_pair(None);
    let (a, b) = intersat(&scenario);
    let interval = *scenario.interval();
    let analysis = InterSatelliteAnalysis::new(&scenario, &ensemble, &NoEphemeris)
        .with_range_limits(None, Some(Distance::kilometers(5000.0)));
    bencher.bench(|| analysis.single(a, b, interval).unwrap());
}

#[divan::bench]
fn intersat_pair_min_max_range(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_intersat_pair(None);
    let (a, b) = intersat(&scenario);
    let interval = *scenario.interval();
    let analysis = InterSatelliteAnalysis::new(&scenario, &ensemble, &NoEphemeris)
        .with_range_limits(
            Some(Distance::kilometers(100.0)),
            Some(Distance::kilometers(5000.0)),
        );
    bencher.bench(|| analysis.single(a, b, interval).unwrap());
}

#[divan::bench]
fn intersat_pair_slew_rate(bencher: Bencher) {
    let (scenario, ensemble) =
        common::setup_intersat_pair(Some(AngularRate::degrees_per_second(0.5)));
    let (a, b) = intersat(&scenario);
    let interval = *scenario.interval();
    let analysis = InterSatelliteAnalysis::new(&scenario, &ensemble, &NoEphemeris);
    bencher.bench(|| analysis.single(a, b, interval).unwrap());
}

#[divan::bench]
fn intersat_pair_with_los(bencher: Bencher) {
    let spk = common::ephemeris();
    let (scenario, ensemble) = common::setup_intersat_pair(None);
    let (a, b) = intersat(&scenario);
    let interval = *scenario.interval();
    let analysis = InterSatelliteAnalysis::new(&scenario, &ensemble, spk)
        .with_occulting_bodies(vec![Origin::Moon]);
    bencher.bench(|| analysis.single(a, b, interval).unwrap());
}

// ---------------------------------------------------------------------------
// Many-asset scaling — the rayon fan-out
//
// Inter-satellite pair count is n*(n-1)/2: n=10 -> 45, n=50 -> 1225,
// n=120 -> 7140, n=250 -> 31125. Unlike the old implementation's hard-coded
// 100-pair threshold, parallelism is now the caller's explicit choice, so both
// modes are measured at every size.
// ---------------------------------------------------------------------------

const MODES: [(&str, Parallelism); 2] = [
    ("sequential", Parallelism::Sequential),
    ("rayon", Parallelism::Rayon(None)),
];

#[divan::bench(args = [10, 50, 120, 250], consts = [0, 1])]
fn intersat_scaling<const MODE: usize>(bencher: Bencher, n: usize) {
    let (_, mode) = MODES[MODE];
    bencher
        .with_inputs(|| common::propagate_oneweb(n, 2))
        .bench_values(|(scenario, ensemble)| {
            let interval = *scenario.interval();
            InterSatelliteAnalysis::new(&scenario, &ensemble, &NoEphemeris).run(interval, mode)
        });
}

// Ground-space pair count is 5*n (five ground stations).
#[divan::bench(args = [10, 50, 120], consts = [0, 1])]
fn groundspace_scaling<const MODE: usize>(bencher: Bencher, n: usize) {
    let (_, mode) = MODES[MODE];
    bencher
        .with_inputs(|| common::build_groundspace_scenario(n))
        .bench_values(|(scenario, ensemble)| {
            let interval = *scenario.interval();
            VisibilityAnalysis::new(&scenario, &ensemble, &NoEphemeris).run(interval, mode)
        });
}

// ---------------------------------------------------------------------------
// Pass interpolation
// ---------------------------------------------------------------------------

#[divan::bench(sample_size = 1000)]
fn pass_interpolate(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_dynamic();
    let gs = &scenario.ground_stations()[0];
    let sc = &scenario.spacecraft()[0];
    let passes = VisibilityAnalysis::new(&scenario, &ensemble, &NoEphemeris)
        .single(gs, sc, *scenario.interval())
        .unwrap();
    let pass = passes.first().expect("at least one pass");
    let times = pass.times();
    let mid = times[times.len() / 2];
    bencher.bench(|| pass.interpolate(black_box(mid)));
}
