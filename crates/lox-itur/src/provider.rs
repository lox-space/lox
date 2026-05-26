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

const PROB_LEVELS_18: [f64; 18] = [
    0.1, 0.2, 0.3, 0.5, 1.0, 2.0, 3.0, 5.0, 10.0, 20.0, 30.0, 50.0, 60.0, 70.0, 80.0, 90.0, 95.0,
    99.0,
];

const PROB_KEYS_18: [&str; 18] = [
    "01", "02", "03", "05", "1", "2", "3", "5", "10", "20", "30", "50", "60", "70", "80", "90",
    "95", "99",
];

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

    /// Log-probability interpolation across the 18 standard probability levels.
    ///
    /// `prefix` is the grid-key prefix (e.g. `"836/v6_rho"`); the level suffix
    /// and `.npy` extension are appended. Mirrors the legacy `interpolate_probability`.
    fn interpolate_prob_18(
        &self,
        prefix: &str,
        lat_key: &str,
        lon_key: &str,
        lat_deg: f64,
        lon_deg: f64,
        p: f64,
    ) -> Result<f64, ItuProviderError> {
        let idx = PROB_LEVELS_18
            .iter()
            .position(|&pl| pl >= p)
            .unwrap_or(PROB_LEVELS_18.len() - 1);
        let key_at = |i: usize| format!("{prefix}_{}.npy", PROB_KEYS_18[i]);

        if (PROB_LEVELS_18[idx] - p).abs() < 1e-10 {
            let g = self.grid_xyz(lat_key, lon_key, &key_at(idx))?;
            return Ok(g.bilinear(lat_deg, lon_deg));
        }
        if idx == 0 {
            let g = self.grid_xyz(lat_key, lon_key, &key_at(0))?;
            return Ok(g.bilinear(lat_deg, lon_deg));
        }
        let p_below = PROB_LEVELS_18[idx - 1];
        let p_above = PROB_LEVELS_18[idx];
        let v_below = self
            .grid_xyz(lat_key, lon_key, &key_at(idx - 1))?
            .bilinear(lat_deg, lon_deg);
        let v_above = self
            .grid_xyz(lat_key, lon_key, &key_at(idx))?
            .bilinear(lat_deg, lon_deg);
        let t = (p.ln() - p_below.ln()) / (p_above.ln() - p_below.ln());
        Ok(v_below + (v_above - v_below) * t)
    }

    /// Surface water vapour density [g/m³] exceeded for `p`% of the average year (P.836-6).
    pub fn surface_water_vapour_density(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
        p: f64,
    ) -> Result<f64, ItuProviderError> {
        self.interpolate_prob_18(
            "836/v6_rho",
            "836/v6_lat.npy",
            "836/v6_lon.npy",
            lat.to_degrees(),
            lon.to_degrees(),
            p,
        )
    }

    /// Total columnar water vapour content [kg/m²] exceeded for `p`% (P.836-6).
    pub fn total_water_vapour_content(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
        p: f64,
    ) -> Result<f64, ItuProviderError> {
        self.interpolate_prob_18(
            "836/v6_v",
            "836/v6_lat.npy",
            "836/v6_lon.npy",
            lat.to_degrees(),
            lon.to_degrees(),
            p,
        )
    }

    /// Wet-term radio refractivity (Nwet) exceeded for `p`% (P.453, grid).
    pub fn map_wet_term_radio_refractivity(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
        p: f64,
    ) -> Result<f64, ItuProviderError> {
        self.interpolate_prob_18(
            "453/v13_nwet_annual",
            "453/v13_lat_n.npy",
            "453/v13_lon_n.npy",
            lat.to_degrees(),
            lon.to_degrees(),
            p,
        )
    }

    /// Mean annual 0°C isotherm height (P.839, grid). Data is in kilometres.
    pub fn isotherm_0c_height(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
    ) -> Result<lox_core::units::Distance, ItuProviderError> {
        let g = self.grid_xyz(
            "839/v4_esalat.npy",
            "839/v4_esalon.npy",
            "839/v4_esa0height.npy",
        )?;
        Ok(lox_core::units::Distance::kilometers(
            g.bilinear(lat.to_degrees(), lon.to_degrees()),
        ))
    }

    /// Mean annual rain height = isotherm 0°C height + 0.36 km (P.839-4 Eq. 1).
    pub fn rain_height(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
    ) -> Result<lox_core::units::Distance, ItuProviderError> {
        let g = self.grid_xyz(
            "839/v4_esalat.npy",
            "839/v4_esalon.npy",
            "839/v4_esa0height.npy",
        )?;
        let h0_km = g.bilinear(lat.to_degrees(), lon.to_degrees());
        Ok(lox_core::units::Distance::kilometers(h0_km + 0.36))
    }

    /// Total columnar content of reduced cloud liquid water [kg/m²] exceeded for `p`% (P.840).
    pub fn columnar_content_reduced_liquid(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
        p: f64,
    ) -> Result<f64, ItuProviderError> {
        self.interpolate_prob_18(
            "840/v7_lred",
            "840/v7_lat.npy",
            "840/v7_lon.npy",
            lat.to_degrees(),
            lon.to_degrees(),
            p,
        )
    }

    /// Cloud attenuation [dB] on a slant path (P.840-9).
    pub fn cloud_attenuation(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
        elevation: lox_core::units::Angle,
        frequency: lox_core::units::Frequency,
        p: f64,
    ) -> Result<f64, ItuProviderError> {
        let lred = self.columnar_content_reduced_liquid(lat, lon, p)?;
        let kl = crate::p840::cloud_liquid_mass_absorption_coefficient(frequency.to_gigahertz());
        let sin_el = elevation.to_degrees().max(5.0).to_radians().sin();
        Ok(lred * kl / sin_el)
    }

    /// Log-normal approximation coefficients (m, σ, P_clw) for cloud attenuation (P.840).
    pub fn lognormal_approximation_coefficient(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
    ) -> Result<crate::p840::LognormalCoefficients, ItuProviderError> {
        let lat_deg = lat.to_degrees();
        let lon_deg = lon.to_degrees();
        let m = self
            .grid_xyz("840/v7_lat.npy", "840/v7_lon.npy", "840/v7_m.npy")?
            .bilinear(lat_deg, lon_deg);
        let sigma = self
            .grid_xyz("840/v7_lat.npy", "840/v7_lon.npy", "840/v7_sigma.npy")?
            .bilinear(lat_deg, lon_deg);
        let pclw = self
            .grid_xyz("840/v7_lat.npy", "840/v7_lon.npy", "840/v7_pclw.npy")?
            .bilinear(lat_deg, lon_deg);
        Ok(crate::p840::LognormalCoefficients { m, sigma, pclw })
    }

    /// Rainfall rate exceeded 0.01% of the year [mm/h] (P.837).
    pub fn rainfall_rate_r001(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
    ) -> Result<f64, ItuProviderError> {
        let g = self.grid_xyz(
            "837/v7_lat_r001.npy",
            "837/v7_lon_r001.npy",
            "837/v7_r001.npy",
        )?;
        Ok(g.bilinear(lat.to_degrees(), lon.to_degrees()))
    }

    /// Per-month (r, p0) parameters used by both rainfall_probability and rainfall_rate.
    fn monthly_rain_params(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
    ) -> Result<([f64; 12], [f64; 12]), ItuProviderError> {
        let lat_deg = lat.to_degrees();
        let lon_deg = lon.to_degrees();
        let mut mt = [0.0_f64; 12];
        let mut t_k = [0.0_f64; 12];
        for month in 0..12 {
            let key = format!("837/v7_mt_month{:02}.npy", month + 1);
            mt[month] = self
                .grid_xyz("837/v7_lat_mt.npy", "837/v7_lon_mt.npy", &key)?
                .bilinear(lat_deg, lon_deg);
            t_k[month] = self
                .surface_month_mean_temperature(lat, lon, (month + 1) as u8)?
                .to_kelvin();
        }
        Ok(crate::p837::monthly_rain_params_from(&mt, &t_k))
    }

    /// Annual probability of rain [%] (P.837).
    pub fn rainfall_probability(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
    ) -> Result<f64, ItuProviderError> {
        let (_r, p0) = self.monthly_rain_params(lat, lon)?;
        Ok(crate::p837::rainfall_probability_from(&p0))
    }

    /// Rainfall rate exceeded for `p`% [mm/h] (P.837).
    pub fn rainfall_rate(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
        p: f64,
    ) -> Result<f64, ItuProviderError> {
        if (p - 0.01).abs() < 1e-10 {
            return self.rainfall_rate_r001(lat, lon);
        }
        let (r, p0) = self.monthly_rain_params(lat, lon)?;
        Ok(crate::p837::bisect_rainfall_rate(&r, &p0, p))
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
