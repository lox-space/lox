// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Baseline performance benchmarks for lox-itur, captured before the
//! `ItuProvider` refactor so we can quantify any regression introduced
//! by the new indirection (RwLock + HashMap + Arc::clone) compared to
//! the current per-grid `LazyGrid` (OnceLock<RegularGrid2D>) statics.
//!
//! Two scopes:
//!   * `environmental_losses_madrid` — full orchestrator call; the
//!     real-world workload (hits ~12 grids, dominated by math).
//!   * `topographic_altitude_madrid` — single grid query; amplified
//!     view of per-call lookup overhead.

use divan::Bencher;
use lox_core::units::{Angle, Distance, Frequency};
use lox_itur::{EnvironmentalLosses, p1511};

fn main() {
    divan::main();
}

// Madrid — also used as the canonical fixture in the existing reference-value tests.
const LAT_DEG: f64 = 40.4;
const LON_DEG: f64 = -3.7;

// Warm the grid caches once so we measure steady-state (cached lookup +
// bilinear interpolation), not first-load (zstd decompress + parse).
fn warmup() {
    let _ = EnvironmentalLosses::new(
        Angle::degrees(LAT_DEG),
        Angle::degrees(LON_DEG),
        Frequency::gigahertz(20.0),
        Angle::degrees(30.0),
        1.0,
        Distance::meters(1.2),
        Angle::degrees(45.0),
    );
}

#[divan::bench]
fn environmental_losses_madrid(bencher: Bencher) {
    warmup();
    bencher.bench(|| {
        EnvironmentalLosses::new(
            Angle::degrees(LAT_DEG),
            Angle::degrees(LON_DEG),
            Frequency::gigahertz(20.0),
            Angle::degrees(30.0),
            1.0,
            Distance::meters(1.2),
            Angle::degrees(45.0),
        )
    });
}

#[divan::bench]
fn topographic_altitude_madrid(bencher: Bencher) {
    warmup();
    let lat = Angle::degrees(LAT_DEG);
    let lon = Angle::degrees(LON_DEG);
    bencher.bench(|| p1511::topographic_altitude(lat, lon));
}
