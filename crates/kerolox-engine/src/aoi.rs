// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! AOI library: loads bundled GeoJSON polygons at startup and exposes
//! them by well-known ID.

use lox_space::analysis::imaging::{Aoi, AoiError};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AoiLibraryError {
    #[error("AOI file {0:?} could not be read: {1}")]
    Read(String, std::io::Error),
    #[error("AOI file {0:?} is not valid GeoJSON: {1}")]
    Parse(String, String),
    #[error("AOI file {0:?} produced an invalid polygon: {1}")]
    Invalid(String, AoiError),
    #[error("AOI file {0:?} has no `id` property")]
    MissingId(String),
}

/// In-memory AOI library indexed by well-known ID.
pub struct AoiLibrary {
    by_id: HashMap<String, Aoi>,
}

impl AoiLibrary {
    /// Load the bundled AOI files into a library.
    ///
    /// Currently loads `hormuz.geojson` and `black_sea.geojson`; both files
    /// must be present and parseable.
    pub fn load_from_dir(dir: &Path) -> Result<Self, AoiLibraryError> {
        let mut by_id = HashMap::new();
        for path_str in &["hormuz.geojson", "black_sea.geojson"] {
            let path = dir.join(path_str);
            let body = std::fs::read_to_string(&path)
                .map_err(|e| AoiLibraryError::Read(path_str.to_string(), e))?;
            let (id, aoi) = load_one(path_str, &body)?;
            by_id.insert(id, aoi);
        }
        Ok(Self { by_id })
    }

    pub fn get(&self, id: &str) -> Option<&Aoi> {
        self.by_id.get(id)
    }

    pub fn ids(&self) -> impl Iterator<Item = &String> {
        self.by_id.keys()
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

fn load_one(label: &str, body: &str) -> Result<(String, Aoi), AoiLibraryError> {
    let fc: geojson::FeatureCollection = body
        .parse()
        .map_err(|e: geojson::Error| AoiLibraryError::Parse(label.to_string(), e.to_string()))?;
    let feature = fc
        .features
        .first()
        .ok_or_else(|| AoiLibraryError::Parse(label.to_string(), "no features".into()))?;
    let id = feature
        .properties
        .as_ref()
        .and_then(|p| p.get("id"))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .ok_or_else(|| AoiLibraryError::MissingId(label.to_string()))?;
    let aoi = Aoi::from_geojson(body)
        .map_err(|e| AoiLibraryError::Invalid(label.to_string(), e))?;
    Ok((id, aoi))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("aois")
    }

    #[test]
    fn loads_both_aois() {
        let lib = AoiLibrary::load_from_dir(&data_dir()).unwrap();
        assert_eq!(lib.len(), 2);
        assert!(lib.get("hormuz").is_some());
        assert!(lib.get("black_sea").is_some());
    }

    #[test]
    fn aoi_polygons_are_closed() {
        let lib = AoiLibrary::load_from_dir(&data_dir()).unwrap();
        for id in ["hormuz", "black_sea"] {
            let aoi = lib.get(id).unwrap();
            let poly = aoi.polygon();
            let first = poly.exterior().0.first().unwrap();
            let last = poly.exterior().0.last().unwrap();
            assert_eq!(first, last, "polygon {id} not closed");
            assert!(poly.exterior().0.len() >= 4, "polygon {id} too small");
        }
    }
}
