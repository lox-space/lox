// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=proto");
    prost_build::compile_protos(
        &["proto/kerolox/v1/kerolox.proto"],
        &["proto"],
    )
}
