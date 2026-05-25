// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

fn main() {
    println!("cargo:rerun-if-changed=proto");

    // Point connectrpc-build (and prost-build under it) at a protoc binary
    // bundled with the build, so contributors don't need to install
    // protobuf system-wide on their host.
    let protoc = protoc_bin_vendored::protoc_bin_path()
        .expect("protoc-bin-vendored did not ship a binary for this platform");
    // SAFETY: build.rs runs single-threaded before any user code; setting
    // an env var here is safe and is the documented integration point for
    // prost/connectrpc codegen.
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    connectrpc_build::Config::new()
        .files(&["proto/kerolox/v1/kerolox.proto"])
        .includes(&["proto"])
        .include_file("_connectrpc.rs")
        .compile()
        .expect("connectrpc-build codegen failed");
}
