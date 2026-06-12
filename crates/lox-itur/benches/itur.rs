// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Performance benchmarks for lox-itur, comparing the `ItuProvider` design
//! (RwLock + HashMap + Arc::clone) against the pre-refactor `LazyGrid`
//! (OnceLock<RegularGrid2D>) baseline captured in commit 530472c.
//!
//! Two scopes:
//!   * `environmental_losses_madrid` — full orchestrator call; the
//!     real-world workload (hits ~12 grids, dominated by math).
//!   * `topographic_altitude_madrid` — single grid query; amplified
//!     view of per-call lookup overhead.

use std::path::PathBuf;
use std::sync::OnceLock;

use divan::Bencher;
use lox_core::units::{Angle, Distance, Frequency};
use lox_itur::ItuProvider;

fn main() {
    divan::main();
}

// Madrid — also used as the canonical fixture in the existing reference-value tests.
const LAT_DEG: f64 = 40.4;
const LON_DEG: f64 = -3.7;

fn provider() -> &'static ItuProvider {
    static P: OnceLock<ItuProvider> = OnceLock::new();
    P.get_or_init(|| {
        let path = std::env::var("LOX_ITUR_BUNDLE")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                let m = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                let ws = m.parent()?.parent()?;
                Some(ws.join("target").join("lox-itur-data.npz"))
            })
            .filter(|p| p.exists())
            .expect(
                "lox-itur benches need lox-itur-data.npz. \
                 Run `just lox-itur-pack <wheel>` or set LOX_ITUR_BUNDLE.",
            );
        ItuProvider::open(path).expect("failed to open bundle")
    })
}

// Warm the grid caches once so we measure steady-state (cached lookup +
// bilinear interpolation), not first-load (decompress + parse).
fn warmup() {
    let _ = provider().propagation_losses(
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
fn propagation_losses_madrid(bencher: Bencher) {
    let p = provider();
    warmup();
    bencher.bench(|| {
        p.propagation_losses(
            Angle::degrees(LAT_DEG),
            Angle::degrees(LON_DEG),
            Frequency::gigahertz(20.0),
            Angle::degrees(30.0),
            1.0,
            Distance::meters(1.2),
            Angle::degrees(45.0),
        )
        .unwrap()
    });
}

#[divan::bench]
fn topographic_altitude_madrid(bencher: Bencher) {
    let p = provider();
    warmup();
    let lat = Angle::degrees(LAT_DEG);
    let lon = Angle::degrees(LON_DEG);
    bencher.bench(|| p.topographic_altitude(lat, lon).unwrap());
}
