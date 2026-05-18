// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Test utilities for the Lox ecosystem.
//!
//! Provides approximate equality testing (`approx_eq`), test data helpers, and the
//! `#[derive(ApproxEq)]` macro (behind the `derive` feature).

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

extern crate alloc;

pub mod approx_eq;

#[cfg(feature = "std")]
mod fixtures {
    use std::{
        fs::read_to_string,
        path::{Path, PathBuf},
    };

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
}

#[cfg(feature = "std")]
pub use fixtures::{data_dir, data_file, read_data_file};

#[cfg(feature = "derive")]
#[doc(inline)]
pub use lox_derive::ApproxEq;
