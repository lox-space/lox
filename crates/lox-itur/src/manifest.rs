// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Manifest schema for the lox-itur-data.npz bundle.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Supported manifest format version.
pub const FORMAT_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub upstream: String,
    pub grids: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("malformed manifest JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported manifest version {0:?} (this build expects {FORMAT_VERSION:?})")]
    UnsupportedVersion(String),
}

impl Manifest {
    pub fn parse(bytes: &[u8]) -> Result<Self, ManifestError> {
        let m: Manifest = serde_json::from_slice(bytes)?;
        if m.version != FORMAT_VERSION {
            return Err(ManifestError::UnsupportedVersion(m.version));
        }
        Ok(m)
    }

    pub fn to_json_bytes(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self).expect("Manifest serialization is infallible")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_manifest() {
        let json = br#"{"version":"1","upstream":"itur-0.4.0","grids":["1511/v2_topo.npy"]}"#;
        let m = Manifest::parse(json).unwrap();
        assert_eq!(m.upstream, "itur-0.4.0");
        assert_eq!(m.grids, vec!["1511/v2_topo.npy"]);
    }

    #[test]
    fn rejects_version_mismatch() {
        let json = br#"{"version":"99","upstream":"x","grids":[]}"#;
        let err = Manifest::parse(json).unwrap_err();
        assert!(matches!(err, ManifestError::UnsupportedVersion(v) if v == "99"));
    }

    #[test]
    fn rejects_malformed_json() {
        let json = b"not json";
        assert!(matches!(Manifest::parse(json), Err(ManifestError::Json(_))));
    }

    #[test]
    fn round_trip() {
        let m = Manifest {
            version: FORMAT_VERSION.to_owned(),
            upstream: "itur-0.4.0".to_owned(),
            grids: vec!["a/b.npy".to_owned()],
        };
        let bytes = m.to_json_bytes();
        let m2 = Manifest::parse(&bytes).unwrap();
        assert_eq!(m.upstream, m2.upstream);
    }
}
