// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-License-Identifier: MPL-2.0

use std::path::PathBuf;

use crate::origins::generate_bodies;
use lox_io::spice::Kernel;

mod common;
mod origins;

fn crates_dir() -> PathBuf {
    PathBuf::from(format!("{}/../../crates", env!("CARGO_MANIFEST_DIR")))
}

pub fn main() {
    let pck = Kernel::from_string(include_str!("../../../data/pck00011_n0066.tpc"))
        .expect("parsing should succeed");
    let gm = Kernel::from_string(include_str!("../../../data/gm_de440.tpc"))
        .expect("parsing should succeed");
    let bodies_target_dir = crates_dir().join("lox-bodies/src/");
    generate_bodies(&bodies_target_dir, &pck, &gm);
}
