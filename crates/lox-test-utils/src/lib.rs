use std::path::PathBuf;

/// Returns a [PathBuf] to the test fixture directory.
pub fn data_dir() -> PathBuf {
    PathBuf::from(format!("{}/../../data", env!("CARGO_MANIFEST_DIR")))
}
