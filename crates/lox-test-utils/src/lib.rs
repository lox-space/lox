// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

pub mod approx_eq;

/// Returns a [PathBuf] to the test fixture directory.
pub fn data_dir() -> PathBuf {
    PathBuf::from(format!("{}/../../data", env!("CARGO_MANIFEST_DIR")))
}

/// Returns a [PathBuf] to a file in the test fixture directory.
pub fn data_file(path: impl AsRef<Path>) -> PathBuf {
    data_dir().join(path)
}

/// Returns the contents of the data file at `path`.
///
/// # Panics
/// This function will panic if the file does not exist or is otherwise unreadable.
pub fn read_data_file(path: impl AsRef<Path>) -> String {
    read_to_string(data_file(path)).expect("data file should be readable")
}

#[cfg(feature = "derive")]
#[doc(inline)]
pub use lox_derive::ApproxEq;
