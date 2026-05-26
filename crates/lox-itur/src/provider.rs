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
use crate::npz::{self, NpyError};

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

    /// Returns the grid for the data entry `val_key`, built from companion
    /// lat/lon entries `lat_key` / `lon_key`. Cached after first load by `val_key`.
    pub(crate) fn grid_xyz(
        &self,
        lat_key: &str,
        lon_key: &str,
        val_key: &str,
    ) -> Result<Arc<RegularGrid2D>, ItuProviderError> {
        if let Some(g) = self.grids.read().unwrap().get(val_key) {
            return Ok(g.clone());
        }
        let (lat_bytes, lon_bytes, val_bytes) = {
            let mut archive = self.archive.lock().unwrap();
            (
                read_entry_bytes(&mut archive, lat_key)?,
                read_entry_bytes(&mut archive, lon_key)?,
                read_entry_bytes(&mut archive, val_key)?,
            )
        };
        let parsed = npz::grid_from_npy(&lat_bytes, &lon_bytes, &val_bytes).map_err(|source| {
            ItuProviderError::Npy {
                entry: val_key.to_owned(),
                source,
            }
        })?;
        let arc = Arc::new(parsed);
        Ok(self
            .grids
            .write()
            .unwrap()
            .entry(val_key.to_owned())
            .or_insert_with(|| arc.clone())
            .clone())
    }

    /// Topographic altitude above mean sea level (ITU-R P.1511-2).
    pub fn topographic_altitude(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
    ) -> Result<lox_core::units::Distance, ItuProviderError> {
        let g = self.grid_xyz("1511/v2_lat.npy", "1511/v2_lon.npy", "1511/v2_topo.npy")?;
        Ok(lox_core::units::Distance::meters(
            g.bilinear(lat.to_degrees(), lon.to_degrees()),
        ))
    }

    /// Annual surface mean temperature (ITU-R P.1510).
    pub fn surface_mean_temperature(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
    ) -> Result<lox_core::units::Temperature, ItuProviderError> {
        let g = self.grid_xyz("1510/v1_lat.npy", "1510/v1_lon.npy", "1510/v1_t_annual.npy")?;
        Ok(lox_core::units::Temperature::kelvin(
            g.bilinear(lat.to_degrees(), lon.to_degrees()),
        ))
    }

    /// Monthly surface mean temperature for `month` ∈ 1..=12 (ITU-R P.1510).
    ///
    /// # Panics
    ///
    /// Panics if `month` is not in 1..=12.
    pub fn surface_month_mean_temperature(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
        month: u8,
    ) -> Result<lox_core::units::Temperature, ItuProviderError> {
        assert!(
            (1..=12).contains(&month),
            "month must be 1..=12, got {month}"
        );
        let key = format!("1510/v1_t_month{month:02}.npy");
        let g = self.grid_xyz("1510/v1_lat.npy", "1510/v1_lon.npy", &key)?;
        Ok(lox_core::units::Temperature::kelvin(
            g.bilinear(lat.to_degrees(), lon.to_degrees()),
        ))
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

    fn synth_3x4_bundle() -> NamedTempFile {
        // matches grid_from_npy_no_flip test from npz module
        let lat = npz::tests_synth_npy_2d(
            3,
            4,
            &[
                -10.0, -10.0, -10.0, -10.0, 0.0, 0.0, 0.0, 0.0, 10.0, 10.0, 10.0, 10.0,
            ],
        );
        let lon = npz::tests_synth_npy_2d(
            3,
            4,
            &[
                0.0, 10.0, 20.0, 30.0, 0.0, 10.0, 20.0, 30.0, 0.0, 10.0, 20.0, 30.0,
            ],
        );
        let val = npz::tests_synth_npy_2d(
            3,
            4,
            &[
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
            ],
        );
        let f = NamedTempFile::new().unwrap();
        let mut writer = zip::ZipWriter::new(f.reopen().unwrap());
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        writer.start_file(MANIFEST_ENTRY, opts).unwrap();
        writer
            .write_all(
                br#"{"version":"1","upstream":"itur-0.4.0","grids":["t/lat.npy","t/lon.npy","t/val.npy"]}"#,
            )
            .unwrap();
        for (name, bytes) in [
            ("t/lat.npy", &lat),
            ("t/lon.npy", &lon),
            ("t/val.npy", &val),
        ] {
            writer.start_file(name, opts).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap();
        f
    }

    #[test]
    fn grid_xyz_loads_and_caches() {
        let f = synth_3x4_bundle();
        let p = ItuProvider::open(f.path()).unwrap();
        let g1 = p.grid_xyz("t/lat.npy", "t/lon.npy", "t/val.npy").unwrap();
        assert!((g1.bilinear(0.0, 10.0) - 6.0).abs() < 1e-12);
        // cache hit returns same Arc
        let g2 = p.grid_xyz("t/lat.npy", "t/lon.npy", "t/val.npy").unwrap();
        assert!(Arc::ptr_eq(&g1, &g2));
    }

    #[test]
    fn grid_xyz_missing_entry() {
        let f = synth_3x4_bundle();
        let p = ItuProvider::open(f.path()).unwrap();
        let err = p
            .grid_xyz("t/lat.npy", "t/lon.npy", "t/nope.npy")
            .unwrap_err();
        assert!(matches!(err, ItuProviderError::MissingEntry(s) if s == "t/nope.npy"));
    }
}
