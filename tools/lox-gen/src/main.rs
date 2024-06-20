/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::path::PathBuf;

use lox_io::spice::Kernel;

use crate::frames::generate_frames;
use crate::modules::generate_modules;

mod bodies;
mod common;
mod frames;
mod generators;
mod modules;
mod rotational_elements;

fn crates_dir() -> PathBuf {
    PathBuf::from(format!("{}/../../crates", env!("CARGO_MANIFEST_DIR")))
}

pub fn main() {
    let pck = Kernel::from_string(include_str!("../../../data/pck00011.tpc"))
        .expect("parsing should succeed");
    let gm = Kernel::from_string(include_str!("../../../data/gm_de440.tpc"))
        .expect("parsing should succeed");
    let bodies_target_dir = crates_dir().join("lox-bodies/src/generated/");
    generate_modules(&bodies_target_dir, &pck, &gm);
    let frames_target_dir = crates_dir().join("lox-orbits/src/python/");
    generate_frames(&frames_target_dir);
}
