// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::path::PathBuf;

pub mod approx_eq;

/// Returns a [PathBuf] to the test fixture directory.
pub fn data_dir() -> PathBuf {
    PathBuf::from(format!("{}/../../data", env!("CARGO_MANIFEST_DIR")))
}

#[cfg(feature = "derive")]
#[doc(inline)]
pub use lox_derive::ApproxEq;
