// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Comparator constellation library: loads bundled TLE snapshots into
//! SGP4 propagators keyed by well-known id (e.g. "iceye").

use lox_orbits::propagators::sgp4::Sgp4;
use sgp4::Elements;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComparatorError {
    #[error("comparator file {file:?} could not be read: {source}")]
    Read {
        file: String,
        source: std::io::Error,
    },
    #[error("comparator file {file:?} has a malformed TLE near line {line}: {msg}")]
    Tle {
        file: String,
        line: usize,
        msg: String,
    },
}

/// A named comparator constellation: its satellites as SGP4 propagators.
pub struct Comparator {
    pub id: String,
    pub satellites: Vec<(String, Sgp4)>,
}

pub struct ComparatorLibrary {
    by_id: HashMap<String, Comparator>,
}

impl ComparatorLibrary {
    /// Load bundled comparator TLE files from `dir`. Currently loads
    /// `iceye.tle`.
    pub fn load_from_dir(dir: &Path) -> Result<Self, ComparatorError> {
        let mut by_id = HashMap::new();
        let iceye = load_tle_file("iceye", &dir.join("iceye.tle"))?;
        by_id.insert("iceye".to_string(), iceye);
        Ok(Self { by_id })
    }

    pub fn get(&self, id: &str) -> Option<&Comparator> {
        self.by_id.get(id)
    }
}

fn load_tle_file(id: &str, path: &Path) -> Result<Comparator, ComparatorError> {
    let body = std::fs::read_to_string(path).map_err(|e| ComparatorError::Read {
        file: id.to_string(),
        source: e,
    })?;
    let lines: Vec<&str> = body.lines().collect();
    let mut satellites = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim_end();
        if line.starts_with("1 ") && i + 1 < lines.len() && lines[i + 1].starts_with("2 ") {
            let name =
                if i > 0 && !lines[i - 1].starts_with("1 ") && !lines[i - 1].starts_with("2 ") {
                    lines[i - 1].trim().to_string()
                } else {
                    format!("{id}-{i}")
                };
            let elements = Elements::from_tle(
                Some(name.clone()),
                lines[i].as_bytes(),
                lines[i + 1].as_bytes(),
            )
            .map_err(|e| ComparatorError::Tle {
                file: id.to_string(),
                line: i,
                msg: e.to_string(),
            })?;
            let sgp4 = Sgp4::new(elements).map_err(|e| ComparatorError::Tle {
                file: id.to_string(),
                line: i,
                msg: e.to_string(),
            })?;
            satellites.push((name, sgp4));
            i += 2;
        } else {
            i += 1;
        }
    }
    Ok(Comparator {
        id: id.to_string(),
        satellites,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("comparators")
    }

    #[test]
    fn loads_iceye_with_many_sats() {
        let lib = ComparatorLibrary::load_from_dir(&data_dir()).unwrap();
        let iceye = lib.get("iceye").expect("iceye present");
        assert!(
            iceye.satellites.len() >= 20,
            "got {} sats",
            iceye.satellites.len()
        );
    }

    #[test]
    fn unknown_comparator_is_none() {
        let lib = ComparatorLibrary::load_from_dir(&data_dir()).unwrap();
        assert!(lib.get("capella").is_none());
    }
}
