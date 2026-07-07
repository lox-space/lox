// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Microbenchmarks for the pure-function geometry kernels that sit at the
//! bottom of every visibility and power computation: line-of-sight angle
//! (spherical and spheroidal), beta angle, and solar flux. These are
//! sub-microsecond calls, so each runs with a large sample size and
//! `black_box`-wrapped inputs/outputs to defeat constant folding.
//!
//! Run with `cargo bench -p lox-space --bench geometry`.

use divan::black_box;
use lox_space::analysis::power::{beta_angle, solar_flux};
use lox_space::analysis::visibility::{line_of_sight, line_of_sight_spheroid};
use lox_space::orbits::DVec3;

fn main() {
    divan::main();
}

// Representative LEO geometry (kilometres): a spacecraft and a target on
// opposite sides of Earth, plus Earth's radii.
const R1: DVec3 = DVec3::new(0.0, -4464.696, -5102.509);
const R2: DVec3 = DVec3::new(0.0, 5740.323, 3189.068);
const EARTH_EQ_KM: f64 = 6378.137;
const EARTH_POLAR_KM: f64 = 6356.752;
const EARTH_MEAN_KM: f64 = 6371.0084;

#[divan::bench(sample_size = 1000, sample_count = 1000)]
fn line_of_sight_spherical(bencher: divan::Bencher) {
    bencher.bench(|| line_of_sight(black_box(EARTH_MEAN_KM), black_box(R1), black_box(R2)));
}

#[divan::bench(sample_size = 1000, sample_count = 1000)]
fn line_of_sight_spheroidal(bencher: divan::Bencher) {
    bencher.bench(|| {
        line_of_sight_spheroid(
            black_box(EARTH_MEAN_KM),
            black_box(EARTH_EQ_KM),
            black_box(EARTH_POLAR_KM),
            black_box(R1),
            black_box(R2),
        )
    });
}

#[divan::bench(sample_size = 1000, sample_count = 1000)]
fn beta_angle_kernel(bencher: divan::Bencher) {
    let orbit_normal = DVec3::new(0.0, 0.0, 1.0);
    let sun = DVec3::new(1.0, 0.0, 1.0).normalize();
    bencher.bench(|| beta_angle(black_box(orbit_normal), black_box(sun)));
}

#[divan::bench(sample_size = 1000, sample_count = 1000)]
fn solar_flux_kernel(bencher: divan::Bencher) {
    let distance_m = 1.495_978_707e11_f64;
    bencher.bench(|| solar_flux(black_box(distance_m)));
}
