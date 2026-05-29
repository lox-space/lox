// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Power-budget benchmarks: eclipse detection plus beta-angle and solar-flux
//! sampling. Every configuration requires the SPK ephemeris (loaded once) for
//! the Sun position. The scaling benchmark exercises the per-spacecraft
//! parallel fan-out over a OneWeb constellation.
//!
//! Run with `cargo bench -p lox-space --bench power`.

use divan::Bencher;
use lox_space::analysis::power::PowerBudgetAnalysis;
use lox_space::time::deltas::TimeDelta;

#[path = "common/mod.rs"]
mod common;

fn main() {
    divan::main();
}

#[divan::bench]
fn power_single(bencher: Bencher) {
    let spk = common::ephemeris();
    let (scenario, ensemble) = common::build_power_scenario(1);
    bencher.bench(|| {
        PowerBudgetAnalysis::new(&scenario, &ensemble, spk)
            .with_step(TimeDelta::from_seconds(30))
            .compute()
            .unwrap()
    });
}

// Step sensitivity over a single spacecraft (scenario built once).
#[divan::bench(args = [30, 60, 120])]
fn power_step(bencher: Bencher, step_s: i64) {
    let spk = common::ephemeris();
    let (scenario, ensemble) = common::build_power_scenario(1);
    bencher.bench(|| {
        PowerBudgetAnalysis::new(&scenario, &ensemble, spk)
            .with_step(TimeDelta::from_seconds(step_s))
            .compute()
            .unwrap()
    });
}

// Per-spacecraft parallel fan-out across a OneWeb constellation.
#[divan::bench(args = [1, 10, 50, 100])]
fn power_scaling(bencher: Bencher, n: usize) {
    let spk = common::ephemeris();
    bencher
        .with_inputs(|| common::build_power_scenario(n))
        .bench_values(|(scenario, ensemble)| {
            PowerBudgetAnalysis::new(&scenario, &ensemble, spk)
                .compute()
                .unwrap()
        });
}
