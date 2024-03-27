/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_time::deltas::TimeDelta;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn from_f64_seconds() {
    TimeDelta::from_decimal_seconds(divan::black_box(60.3)).unwrap();
}
