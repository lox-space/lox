// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Microbenchmarks for the solution of Kepler's equation, through both the anomaly and the
//! equinoctial longitude interfaces. These are sub-microsecond calls, so each runs with a large
//! sample size and `black_box`-wrapped inputs/outputs to defeat constant folding. Both unwrap their
//! results, because returning a `Result` to the harness can add tens of nanoseconds that depend on
//! how it is compiled rather than on the conversion.
//!
//! Run with `cargo bench -p lox-space --bench kepler`.

use divan::black_box;
use lox_space::core::anomalies::MeanAnomaly;
use lox_space::core::elements::{Eccentricity, Equinoctial, RetrogradeFactor};
use lox_space::units::{Angle, Distance};

fn main() {
    divan::main();
}

const ECCENTRICITIES: [f64; 3] = [0.01, 0.5, 0.95];

#[divan::bench(args = ECCENTRICITIES, sample_size = 1000, sample_count = 1000)]
fn mean_to_eccentric_anomaly(bencher: divan::Bencher, e: f64) {
    let e = Eccentricity::try_new(e).unwrap();
    let mean = MeanAnomaly::new(Angle::radians(2.0));
    bencher.bench(|| black_box(black_box(mean).to_eccentric(black_box(e)).unwrap()));
}

#[divan::bench(args = ECCENTRICITIES, sample_size = 1000, sample_count = 1000)]
fn equinoctial_eccentric_longitude(bencher: divan::Bencher, e: f64) {
    let elements = Equinoctial::try_new(
        Distance::kilometers(7000.0),
        0.6 * e,
        0.8 * e,
        0.1,
        0.2,
        Angle::radians(2.0),
        RetrogradeFactor::Prograde,
    )
    .unwrap();
    bencher.bench(|| black_box(black_box(elements).eccentric_longitude().unwrap()));
}
