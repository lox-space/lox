// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! ITU-R data provider — opens a bundled `.npz` and serves interpolated values.
//!
//! Bundle layout (produced by `cargo run -p lox-itur --bin pack`):
//!   * `manifest.json` — `{"version":"1","upstream":"itur-0.4.0","grids":[...]}`
//!   * `<recommendation>/<filename>.npy` — one entry per grid

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use thiserror::Error;
use zip::ZipArchive;

use crate::grid::RegularGrid2D;
use crate::manifest::{Manifest, ManifestError};
use crate::npz::NpyError;

const MANIFEST_ENTRY: &str = "manifest.json";

#[derive(Debug, Error)]
pub enum ItuProviderError {
    #[error("opening bundle at {path}: {source}")]
    Open {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("bundle archive: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("bundle missing entry {0:?}")]
    MissingEntry(String),
    #[error("manifest: {0}")]
    Manifest(#[from] ManifestError),
    #[error("reading entry {entry:?}: {source}")]
    Read {
        entry: String,
        #[source]
        source: std::io::Error,
    },
    #[error("parsing NPY entry {entry:?}: {source}")]
    Npy {
        entry: String,
        #[source]
        source: NpyError,
    },
}

pub struct ItuProvider {
    archive: Mutex<ZipArchive<File>>,
    grids: RwLock<HashMap<String, Arc<RegularGrid2D>>>,
    upstream: String,
}

impl std::fmt::Debug for ItuProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ItuProvider")
            .field("upstream", &self.upstream)
            .field(
                "cached_grids",
                &self.grids.read().map(|g| g.len()).unwrap_or(0),
            )
            .finish_non_exhaustive()
    }
}

impl ItuProvider {
    /// Opens a bundle file. Reads + validates the manifest; does not parse any grids.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ItuProviderError> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|source| ItuProviderError::Open {
            path: path.to_owned(),
            source,
        })?;
        let mut archive = ZipArchive::new(file)?;
        let manifest_bytes = read_entry_bytes(&mut archive, MANIFEST_ENTRY)?;
        let manifest = Manifest::parse(&manifest_bytes)?;
        Ok(Self {
            archive: Mutex::new(archive),
            grids: RwLock::new(HashMap::new()),
            upstream: manifest.upstream,
        })
    }

    pub fn upstream_version(&self) -> &str {
        &self.upstream
    }
}

fn read_entry_bytes(
    archive: &mut ZipArchive<File>,
    name: &str,
) -> Result<Vec<u8>, ItuProviderError> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| ItuProviderError::MissingEntry(name.to_owned()))?;
    let mut buf = Vec::with_capacity(entry.size() as usize);
    entry
        .read_to_end(&mut buf)
        .map_err(|source| ItuProviderError::Read {
            entry: name.to_owned(),
            source,
        })?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn synth_bundle_with_manifest(manifest_json: &str) -> NamedTempFile {
        let f = NamedTempFile::new().unwrap();
        let mut writer = zip::ZipWriter::new(f.reopen().unwrap());
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        writer.start_file(MANIFEST_ENTRY, opts).unwrap();
        writer.write_all(manifest_json.as_bytes()).unwrap();
        writer.finish().unwrap();
        f
    }

    #[test]
    fn open_reads_manifest_upstream() {
        let f = synth_bundle_with_manifest(r#"{"version":"1","upstream":"itur-0.4.0","grids":[]}"#);
        let p = ItuProvider::open(f.path()).unwrap();
        assert_eq!(p.upstream_version(), "itur-0.4.0");
    }

    #[test]
    fn open_rejects_bad_manifest_version() {
        let f = synth_bundle_with_manifest(r#"{"version":"99","upstream":"x","grids":[]}"#);
        let err = ItuProvider::open(f.path()).unwrap_err();
        assert!(matches!(err, ItuProviderError::Manifest(_)), "{err:?}");
    }

    #[test]
    fn open_rejects_missing_manifest() {
        let f = NamedTempFile::new().unwrap();
        let mut writer = zip::ZipWriter::new(f.reopen().unwrap());
        writer.finish().unwrap();
        let err = ItuProvider::open(f.path()).unwrap_err();
        assert!(matches!(err, ItuProviderError::MissingEntry(s) if s == "manifest.json"));
    }

    #[test]
    fn open_rejects_nonexistent_file() {
        let err = ItuProvider::open("/nonexistent/path/lox-itur-data.npz").unwrap_err();
        assert!(matches!(err, ItuProviderError::Open { .. }));
    }
}
