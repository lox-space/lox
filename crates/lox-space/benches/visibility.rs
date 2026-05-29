// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Visibility-analysis benchmarks.
//!
//! Ground-space single-pair benchmarks (dyn vs. monomorphic) establish the
//! baseline; inter-satellite, filter, and many-asset scaling benchmarks cover
//! the rest of the [`VisibilityAnalysis`] surface — including the rayon-parallel
//! fan-out that activates above 100 asset pairs.
//!
//! Run with `cargo bench -p lox-space --bench visibility`.

use divan::{Bencher, black_box};
use lox_space::analysis::visibility::VisibilityAnalysis;
use lox_space::bodies::DynOrigin;
use lox_space::core::units::{AngularRate, Distance};
use lox_space::time::deltas::TimeDelta;

#[path = "common/mod.rs"]
mod common;

fn main() {
    divan::main();
}

// ---------------------------------------------------------------------------
// Ground-space single pair — dyn dispatch (baseline)
// ---------------------------------------------------------------------------

#[divan::bench]
fn visibility_single_pair(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_dyn();
    bencher.bench(|| {
        VisibilityAnalysis::new(&scenario, &ensemble)
            .compute()
            .unwrap()
    });
}

#[divan::bench]
fn visibility_single_pair_min_pass_5m(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_dyn();
    bencher.bench(|| {
        VisibilityAnalysis::new(&scenario, &ensemble)
            .with_min_pass_duration(TimeDelta::from_seconds(300))
            .compute()
            .unwrap()
    });
}

#[divan::bench]
fn visibility_single_pair_with_los(bencher: Bencher) {
    let spk = common::ephemeris();
    let (scenario, ensemble) = common::setup_dyn();
    bencher.bench(|| {
        VisibilityAnalysis::new(&scenario, &ensemble)
            .with_occulting_bodies(spk, vec![DynOrigin::Moon])
            .compute()
            .unwrap()
    });
}

#[divan::bench]
fn visibility_single_pair_with_los_min_pass_5m(bencher: Bencher) {
    let spk = common::ephemeris();
    let (scenario, ensemble) = common::setup_dyn();
    bencher.bench(|| {
        VisibilityAnalysis::new(&scenario, &ensemble)
            .with_occulting_bodies(spk, vec![DynOrigin::Moon])
            .with_min_pass_duration(TimeDelta::from_seconds(300))
            .compute()
            .unwrap()
    });
}

// ---------------------------------------------------------------------------
// Ground-space single pair — monomorphic (concrete types, no DynFrame dispatch)
// ---------------------------------------------------------------------------

#[divan::bench]
fn visibility_single_pair_mono(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_mono();
    bencher.bench(|| {
        VisibilityAnalysis::new(&scenario, &ensemble)
            .compute()
            .unwrap()
    });
}

#[divan::bench]
fn visibility_single_pair_mono_min_pass_5m(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_mono();
    bencher.bench(|| {
        VisibilityAnalysis::new(&scenario, &ensemble)
            .with_min_pass_duration(TimeDelta::from_seconds(300))
            .compute()
            .unwrap()
    });
}

#[divan::bench]
fn visibility_single_pair_mono_with_los(bencher: Bencher) {
    let spk = common::ephemeris();
    let (scenario, ensemble) = common::setup_mono();
    bencher.bench(|| {
        VisibilityAnalysis::new(&scenario, &ensemble)
            .with_occulting_bodies(spk, vec![DynOrigin::Moon])
            .compute()
            .unwrap()
    });
}

#[divan::bench]
fn visibility_single_pair_mono_with_los_min_pass_5m(bencher: Bencher) {
    let spk = common::ephemeris();
    let (scenario, ensemble) = common::setup_mono();
    bencher.bench(|| {
        VisibilityAnalysis::new(&scenario, &ensemble)
            .with_occulting_bodies(spk, vec![DynOrigin::Moon])
            .with_min_pass_duration(TimeDelta::from_seconds(300))
            .compute()
            .unwrap()
    });
}

// ---------------------------------------------------------------------------
// Inter-satellite single pair (two crossing-orbit OneWeb spacecraft)
// ---------------------------------------------------------------------------

#[divan::bench]
fn intersat_pair(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_intersat_pair(None);
    bencher.bench(|| {
        VisibilityAnalysis::new(&scenario, &ensemble)
            .with_inter_satellite()
            .compute()
            .unwrap()
    });
}

#[divan::bench]
fn intersat_pair_max_range(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_intersat_pair(None);
    bencher.bench(|| {
        VisibilityAnalysis::new(&scenario, &ensemble)
            .with_inter_satellite()
            .with_max_range(Distance::kilometers(5000.0))
            .compute()
            .unwrap()
    });
}

#[divan::bench]
fn intersat_pair_min_max_range(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_intersat_pair(None);
    bencher.bench(|| {
        VisibilityAnalysis::new(&scenario, &ensemble)
            .with_inter_satellite()
            .with_min_range(Distance::kilometers(100.0))
            .with_max_range(Distance::kilometers(5000.0))
            .compute()
            .unwrap()
    });
}

#[divan::bench]
fn intersat_pair_slew_rate(bencher: Bencher) {
    let (scenario, ensemble) =
        common::setup_intersat_pair(Some(AngularRate::degrees_per_second(0.5)));
    bencher.bench(|| {
        VisibilityAnalysis::new(&scenario, &ensemble)
            .with_inter_satellite()
            .compute()
            .unwrap()
    });
}

#[divan::bench]
fn intersat_pair_with_los(bencher: Bencher) {
    let spk = common::ephemeris();
    let (scenario, ensemble) = common::setup_intersat_pair(None);
    bencher.bench(|| {
        VisibilityAnalysis::new(&scenario, &ensemble)
            .with_inter_satellite()
            .with_occulting_bodies(spk, vec![DynOrigin::Moon])
            .compute()
            .unwrap()
    });
}

#[divan::bench]
fn intersat_pair_filter(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_intersat_pair(None);
    bencher.bench(|| {
        VisibilityAnalysis::new(&scenario, &ensemble)
            .with_inter_satellite_filter(|a, b| a.id() != b.id())
            .compute()
            .unwrap()
    });
}

// ---------------------------------------------------------------------------
// Many-asset scaling — exercises the rayon parallel fan-out.
//
// Inter-satellite pair count is n*(n-1)/2: n=10 -> 45 (serial),
// n=50 -> 1225, n=120 -> 7140, n=250 -> 31125. The parallel path engages
// above 100 pairs, so the 10 -> 50 step exposes the serial->parallel transition.
// ---------------------------------------------------------------------------

#[divan::bench(args = [10, 50, 120, 250])]
fn intersat_scaling(bencher: Bencher, n: usize) {
    bencher
        .with_inputs(|| common::propagate_oneweb(n, 2))
        .bench_values(|(scenario, ensemble)| {
            VisibilityAnalysis::new(&scenario, &ensemble)
                .with_inter_satellite()
                .compute()
                .unwrap()
        });
}

// Ground-space pair count is 5*n (five ground stations); crosses 100 at n=20.
#[divan::bench(args = [10, 50, 120])]
fn groundspace_scaling(bencher: Bencher, n: usize) {
    bencher
        .with_inputs(|| common::build_groundspace_scenario(n))
        .bench_values(|(scenario, ensemble)| {
            VisibilityAnalysis::new(&scenario, &ensemble)
                .compute()
                .unwrap()
        });
}

// ---------------------------------------------------------------------------
// Pass construction and interpolation
// ---------------------------------------------------------------------------

// `VisibilityAnalysis` holds boxed filter closures and is therefore not `Sync`,
// so it cannot be captured by a divan bench closure — rebuild it inside (cheap;
// `results` is computed once, outside the timed region).
#[divan::bench]
fn to_passes(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_dyn();
    let results = VisibilityAnalysis::new(&scenario, &ensemble)
        .compute()
        .unwrap();
    bencher.bench(|| VisibilityAnalysis::new(&scenario, &ensemble).to_passes(&results));
}

#[divan::bench(sample_size = 1000)]
fn pass_interpolate(bencher: Bencher) {
    let (scenario, ensemble) = common::setup_dyn();
    let analysis = VisibilityAnalysis::new(&scenario, &ensemble);
    let results = analysis.compute().unwrap();
    let passes = analysis.to_passes(&results);
    let pass = passes.values().flatten().next().expect("at least one pass");
    // Interpolate at the middle sample time of the pass.
    let times = pass.times();
    let mid = times[times.len() / 2];
    bencher.bench(|| pass.interpolate(black_box(mid)));
}
