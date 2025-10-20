use std::path::PathBuf;

pub mod approx_eq;

/// Returns a [PathBuf] to the test fixture directory.
pub fn data_dir() -> PathBuf {
    PathBuf::from(format!("{}/../../data", env!("CARGO_MANIFEST_DIR")))
}
