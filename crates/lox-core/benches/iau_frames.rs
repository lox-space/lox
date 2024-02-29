/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use divan::Bencher;
use glam::DVec3;

use lox_coords::frames::{FromFrame, Icrf, Rotation};
use lox_coords::frames::iau::BodyFixed;
use lox_core::bodies::{Jupiter, RotationalElements};

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn right_ascension() {
    Jupiter::right_ascension(divan::black_box(0.0));
}

#[divan::bench]
fn right_ascension_dot() {
    Jupiter::right_ascension_dot(divan::black_box(0.0));
}

#[divan::bench]
fn declination() {
    Jupiter::declination(divan::black_box(0.0));
}

#[divan::bench]
fn declination_dot() {
    Jupiter::declination_dot(divan::black_box(0.0));
}

#[divan::bench]
fn prime_meridian() {
    Jupiter::prime_meridian(divan::black_box(0.0));
}

#[divan::bench]
fn prime_meridian_dot() {
    Jupiter::prime_meridian_dot(divan::black_box(0.0));
}

#[divan::bench]
fn rotation() -> Rotation {
    BodyFixed(Jupiter).rotation_from(Icrf, divan::black_box(0.0))
}

#[divan::bench]
fn transform(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            (
                DVec3::new(6068279.27e-3, -1692843.94e-3, -2516619.18e-3),
                DVec3::new(-660.415582e-3, 5495.938726e-3, -5303.093233e-3),
            )
        })
        .bench_values(|rv| {
            BodyFixed(Jupiter).transform_from(Icrf, divan::black_box(0.0), divan::black_box(rv))
        })
}
