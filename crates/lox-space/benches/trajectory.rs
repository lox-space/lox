// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Trajectory interpolation benchmarks.
//!
//! Interpolation is the innermost call of every analysis, so these isolate it
//! from the rest of the pipeline. Each benchmark evaluates a batch of
//! [`QUERIES`] precomputed epochs against one of three fixtures:
//!
//! - `leo`: one OneWeb spacecraft, SGP4, 24 h at 10 s (~8.6k knots, uniform).
//! - `lunar`: the lunar CSV trajectory (~40k knots, uniform 60 s).
//! - `lunar_irregular`: the lunar trajectory with a deterministic pseudo-random
//!   third of its knots dropped, standing in for adaptive-step output.
//!
//! and one of two access patterns:
//!
//! - `sequential`: monotonically increasing epochs, as in event scans.
//! - `random`: uniformly distributed epochs in random order.
//!
//! Run with `cargo bench -p lox-space --bench trajectory`.

use std::sync::OnceLock;

use divan::counter::ItemsCount;
use divan::{Bencher, black_box};
use lox_space::bodies::Origin;
use lox_space::frames::Frame;
use lox_space::orbits::Trajectory;
use lox_space::time::Time;
use lox_space::time::deltas::TimeDelta;

#[path = "common/mod.rs"]
mod common;

fn main() {
    divan::main();
}

/// Number of epochs evaluated per benchmark iteration.
const QUERIES: usize = 10_000;

const FIXTURES: &[&str] = &["leo", "lunar", "lunar_irregular"];

fn lunar() -> &'static Trajectory {
    static LUNAR: OnceLock<Trajectory> = OnceLock::new();
    LUNAR.get_or_init(|| {
        Trajectory::from_csv_dynamic(
            &lox_test_utils::read_data_file("trajectory_lunar.csv"),
            Origin::Earth,
            Frame::Icrf,
        )
        .unwrap()
    })
}

fn lunar_irregular() -> &'static Trajectory {
    static LUNAR_IRREGULAR: OnceLock<Trajectory> = OnceLock::new();
    LUNAR_IRREGULAR.get_or_init(|| {
        let states = lunar().states();
        let last = states.len() - 1;
        let mut rng = Lcg::new(0x5eed);
        // Keep both endpoints so the span matches `lunar`.
        let kept = states
            .into_iter()
            .enumerate()
            .filter(|(i, _)| *i == 0 || *i == last || rng.next_f64() >= 1.0 / 3.0)
            .map(|(_, s)| s);
        Trajectory::try_new(kept).unwrap()
    })
}

fn leo() -> &'static Trajectory {
    static LEO: OnceLock<Trajectory> = OnceLock::new();
    LEO.get_or_init(|| {
        common::propagate_oneweb_trajectories(1, 24)
            .into_iter()
            .next()
            .unwrap()
            .1
    })
}

fn fixture(name: &str) -> &'static Trajectory {
    match name {
        "leo" => leo(),
        "lunar" => lunar(),
        "lunar_irregular" => lunar_irregular(),
        _ => unreachable!("unknown fixture: {name}"),
    }
}

/// Minimal deterministic PRNG so the query set is identical across runs.
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_f64(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn span_seconds(traj: &Trajectory) -> f64 {
    (traj.end_time() - traj.start_time()).as_seconds_f64()
}

/// Evenly spaced epochs across the span, offset so they rarely hit a knot.
fn sequential_epochs(traj: &Trajectory) -> Vec<Time> {
    let start = traj.start_time();
    let step = span_seconds(traj) / (QUERIES as f64 + 1.0);
    (0..QUERIES)
        .map(|i| start + TimeDelta::from_seconds_f64((i as f64 + 0.5) * step))
        .collect()
}

/// Uniformly distributed epochs across the span, in random order.
fn random_epochs(traj: &Trajectory) -> Vec<Time> {
    let start = traj.start_time();
    let span = span_seconds(traj);
    let mut rng = Lcg::new(0xdecade);
    (0..QUERIES)
        .map(|_| start + TimeDelta::from_seconds_f64(rng.next_f64() * span))
        .collect()
}

#[divan::bench(args = FIXTURES)]
fn at_sequential(bencher: Bencher, name: &str) {
    let traj = fixture(name);
    let epochs = sequential_epochs(traj);
    bencher.counter(ItemsCount::new(QUERIES)).bench(|| {
        for &t in &epochs {
            black_box(traj.at(black_box(t)));
        }
    });
}

#[divan::bench(args = FIXTURES)]
fn at_random(bencher: Bencher, name: &str) {
    let traj = fixture(name);
    let epochs = random_epochs(traj);
    bencher.counter(ItemsCount::new(QUERIES)).bench(|| {
        for &t in &epochs {
            black_box(traj.at(black_box(t)));
        }
    });
}

/// Position-only consumers (line of sight, range) dominate the analysis layer.
#[divan::bench(args = FIXTURES)]
fn position_sequential(bencher: Bencher, name: &str) {
    let traj = fixture(name);
    let epochs = sequential_epochs(traj);
    bencher.counter(ItemsCount::new(QUERIES)).bench(|| {
        for &t in &epochs {
            black_box(traj.at(black_box(t)).position());
        }
    });
}
