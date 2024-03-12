/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::path::{Path, PathBuf};

use lazy_static::lazy_static;

use lox_io::spice::Kernel;
use modules::generate_modules;

mod bodies;
mod generators;
mod modules;
mod rotational_elements;

lazy_static! {
    static ref TARGET_DIR: PathBuf = {
        let parent = Path::new(file!()).parent().unwrap();
        parent.join(Path::new("../../../crates/lox-bodies/src/generated/"))
    };
}

pub fn main() {
    let pck = Kernel::from_string(include_str!("../../../data/pck00011.tpc"))
        .expect("parsing should succeed");
    let gm = Kernel::from_string(include_str!("../../../data/gm_de440.tpc"))
        .expect("parsing should succeed");
    generate_modules(&TARGET_DIR, &pck, &gm);
}
