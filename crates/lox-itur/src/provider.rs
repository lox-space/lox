// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! ITU-R data provider — opens a bundled `.npz` and serves interpolated values.
//!
//! Bundle layout (produced by `cargo run -p lox-itur --bin pack`):
//!   * `manifest.json` — `{"version":"1","upstream":"itur-0.4.0","grids":[...]}`
//!   * `<recommendation>/<filename>.npy` — one entry per grid
//!
//! Each grid the provider can serve is assigned a compile-time slot id (see the
//! `ids` module). [`ItuProvider`] holds a fixed array of per-slot [`OnceLock`]s,
//! so a cached lookup is a bounds-checked index plus an atomic load — no hashing,
//! no allocation, and no reference-count traffic on the hot path.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

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

/// Slot ids into the grid cache. Singles first, then the four 18-level
/// probability series. **Append-only**: never renumber or insert in the middle,
/// or [`grid_specs`] and these ids drift apart (the `slot_ids_match_specs` test
/// guards against exactly that). New grids get the next free id at the end.
mod ids {
    pub(super) const TOPO: usize = 0; // P.1511 topographic altitude
    pub(super) const T_ANNUAL: usize = 1; // P.1510 annual mean temperature
    pub(super) const T_MONTH: usize = 2; // P.1510 monthly, + (month - 1) → 2..=13
    pub(super) const ESA0HEIGHT: usize = 14; // P.839 0°C isotherm height
    pub(super) const R001: usize = 15; // P.837 rainfall rate at 0.01%
    pub(super) const MT_MONTH: usize = 16; // P.837 monthly MT, + (month - 1) → 16..=27
    pub(super) const M_840: usize = 28; // P.840 lognormal μ
    pub(super) const SIGMA_840: usize = 29; // P.840 lognormal σ
    pub(super) const PCLW_840: usize = 30; // P.840 lognormal P_clw
    pub(super) const RHO_836: usize = 31; // P.836 ρ, + level → 31..=48
    pub(super) const V_836: usize = 49; // P.836 V, + level → 49..=66
    pub(super) const NWET_453: usize = 67; // P.453 Nwet, + level → 67..=84
    pub(super) const LRED_840: usize = 85; // P.840 Lred, + level → 85..=102
    pub(super) const N_GRIDS: usize = 103;
}
use ids::*;

/// Load recipe for one cached grid: the `.npz` entry names for its latitude and
/// longitude meshgrids and its value array. Only touched on a slot's first load.
struct GridSpec {
    lat: &'static str,
    lon: &'static str,
    val: String,
}

/// Builds the slot table in id order. Must stay in lock-step with [`ids`];
/// the `slot_ids_match_specs` test pins the mapping.
fn grid_specs() -> Box<[GridSpec]> {
    fn spec(lat: &'static str, lon: &'static str, val: impl Into<String>) -> GridSpec {
        GridSpec {
            lat,
            lon,
            val: val.into(),
        }
    }

    let mut v: Vec<GridSpec> = Vec::with_capacity(N_GRIDS);

    // 0: P.1511 topography
    v.push(spec(
        "1511/v2_lat.npy",
        "1511/v2_lon.npy",
        "1511/v2_topo.npy",
    ));
    // 1: P.1510 annual + 2..=13: monthly
    v.push(spec(
        "1510/v1_lat.npy",
        "1510/v1_lon.npy",
        "1510/v1_t_annual.npy",
    ));
    for m in 1..=12u8 {
        v.push(spec(
            "1510/v1_lat.npy",
            "1510/v1_lon.npy",
            format!("1510/v1_t_month{m:02}.npy"),
        ));
    }
    // 14: P.839 isotherm height
    v.push(spec(
        "839/v4_esalat.npy",
        "839/v4_esalon.npy",
        "839/v4_esa0height.npy",
    ));
    // 15: P.837 r001
    v.push(spec(
        "837/v7_lat_r001.npy",
        "837/v7_lon_r001.npy",
        "837/v7_r001.npy",
    ));
    // 16..=27: P.837 monthly MT
    for m in 1..=12u8 {
        v.push(spec(
            "837/v7_lat_mt.npy",
            "837/v7_lon_mt.npy",
            format!("837/v7_mt_month{m:02}.npy"),
        ));
    }
    // 28,29,30: P.840 lognormal coefficients
    for name in ["m", "sigma", "pclw"] {
        v.push(spec(
            "840/v7_lat.npy",
            "840/v7_lon.npy",
            format!("840/v7_{name}.npy"),
        ));
    }
    // 31..=48: P.836 ρ
    for k in PROB_KEYS_18 {
        v.push(spec(
            "836/v6_lat.npy",
            "836/v6_lon.npy",
            format!("836/v6_rho_{k}.npy"),
        ));
    }
    // 49..=66: P.836 V
    for k in PROB_KEYS_18 {
        v.push(spec(
            "836/v6_lat.npy",
            "836/v6_lon.npy",
            format!("836/v6_v_{k}.npy"),
        ));
    }
    // 67..=84: P.453 Nwet
    for k in PROB_KEYS_18 {
        v.push(spec(
            "453/v13_lat_n.npy",
            "453/v13_lon_n.npy",
            format!("453/v13_nwet_annual_{k}.npy"),
        ));
    }
    // 85..=102: P.840 Lred
    for k in PROB_KEYS_18 {
        v.push(spec(
            "840/v7_lat.npy",
            "840/v7_lon.npy",
            format!("840/v7_lred_{k}.npy"),
        ));
    }

    debug_assert_eq!(v.len(), N_GRIDS, "grid_specs() out of sync with ids");
    v.into_boxed_slice()
}

/// Error returned when opening an ITU data bundle or loading a grid from it.
#[derive(Debug, Error)]
pub enum ItuProviderError {
    /// A computed attenuation was non-physical.
    #[error(transparent)]
    InvalidLoss(#[from] lox_core::comms::PropagationLossError),
    /// The bundle file could not be opened.
    #[error("opening bundle at {path}: {source}")]
    Open {
        /// The bundle path that failed to open.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The bundle's ZIP archive could not be read.
    #[error("bundle archive: {0}")]
    Zip(#[from] zip::result::ZipError),
    /// The bundle is missing an expected entry.
    #[error("bundle missing entry {0:?}")]
    MissingEntry(String),
    /// The bundle manifest could not be parsed.
    #[error("manifest: {0}")]
    Manifest(#[from] ManifestError),
    /// An archive entry could not be read.
    #[error("reading entry {entry:?}: {source}")]
    Read {
        /// The name of the entry being read.
        entry: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// An NPY-encoded grid entry could not be parsed.
    #[error("parsing NPY entry {entry:?}: {source}")]
    Npy {
        /// The name of the NPY entry being parsed.
        entry: String,
        /// The underlying NPY parse error.
        #[source]
        source: NpyError,
    },
}

/// Lazily loads ITU-R atmospheric data grids from a packaged bundle file.
pub struct ItuProvider {
    archive: Mutex<ZipArchive<File>>,
    specs: Box<[GridSpec]>,
    slots: Box<[OnceLock<RegularGrid2D>]>,
    upstream: String,
}

impl std::fmt::Debug for ItuProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cached = self.slots.iter().filter(|s| s.get().is_some()).count();
        f.debug_struct("ItuProvider")
            .field("upstream", &self.upstream)
            .field("cached_grids", &cached)
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
        let slots = (0..N_GRIDS).map(|_| OnceLock::new()).collect::<Vec<_>>();
        Ok(Self {
            archive: Mutex::new(archive),
            specs: grid_specs(),
            slots: slots.into_boxed_slice(),
            upstream: manifest.upstream,
        })
    }

    /// Returns the upstream data version recorded in the bundle manifest.
    pub fn upstream_version(&self) -> &str {
        &self.upstream
    }

    /// Reads and parses the grid for slot `id` from the archive. Cold path.
    fn load(&self, id: usize) -> Result<RegularGrid2D, ItuProviderError> {
        let spec = &self.specs[id];
        let (lat_bytes, lon_bytes, val_bytes) = {
            let mut archive = self.archive.lock().unwrap();
            (
                read_entry_bytes(&mut archive, spec.lat)?,
                read_entry_bytes(&mut archive, spec.lon)?,
                read_entry_bytes(&mut archive, &spec.val)?,
            )
        };
        npz::grid_from_npy(&lat_bytes, &lon_bytes, &val_bytes).map_err(|source| {
            ItuProviderError::Npy {
                entry: spec.val.clone(),
                source,
            }
        })
    }

    /// Returns the grid for slot `id`, loading + caching it on first access.
    ///
    /// Cached lookups are lock-free: an index into `slots` and an atomic load.
    fn grid(&self, id: usize) -> Result<&RegularGrid2D, ItuProviderError> {
        if let Some(g) = self.slots[id].get() {
            return Ok(g);
        }
        let grid = self.load(id)?;
        // A concurrent loader may have won the race; either way `get` then
        // returns an initialised value (ours or theirs).
        let _ = self.slots[id].set(grid);
        Ok(self.slots[id].get().expect("slot initialised above"))
    }

    /// Log-probability interpolation across the 18 standard probability levels,
    /// where `base_id` is the slot id of the lowest-probability grid in the series.
    fn interpolate_prob_18(
        &self,
        base_id: usize,
        lat_deg: f64,
        lon_deg: f64,
        p: f64,
    ) -> Result<f64, ItuProviderError> {
        let idx = PROB_LEVELS_18
            .iter()
            .position(|&pl| pl >= p)
            .unwrap_or(PROB_LEVELS_18.len() - 1);

        if (PROB_LEVELS_18[idx] - p).abs() < 1e-10 {
            return Ok(self.grid(base_id + idx)?.bilinear(lat_deg, lon_deg));
        }
        if idx == 0 {
            return Ok(self.grid(base_id)?.bilinear(lat_deg, lon_deg));
        }
        let p_below = PROB_LEVELS_18[idx - 1];
        let p_above = PROB_LEVELS_18[idx];
        let v_below = self.grid(base_id + idx - 1)?.bilinear(lat_deg, lon_deg);
        let v_above = self.grid(base_id + idx)?.bilinear(lat_deg, lon_deg);
        let t = (p.ln() - p_below.ln()) / (p_above.ln() - p_below.ln());
        Ok(v_below + (v_above - v_below) * t)
    }

    /// Topographic altitude above mean sea level (ITU-R P.1511-2).
    pub fn topographic_altitude(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
    ) -> Result<lox_core::units::Distance, ItuProviderError> {
        let g = self.grid(TOPO)?;
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
        let g = self.grid(T_ANNUAL)?;
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
        let g = self.grid(T_MONTH + (month - 1) as usize)?;
        Ok(lox_core::units::Temperature::kelvin(
            g.bilinear(lat.to_degrees(), lon.to_degrees()),
        ))
    }

    /// Surface water vapour density [g/m³] exceeded for `p`% of the average year (P.836-6).
    pub fn surface_water_vapour_density(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
        p: f64,
    ) -> Result<f64, ItuProviderError> {
        self.interpolate_prob_18(RHO_836, lat.to_degrees(), lon.to_degrees(), p)
    }

    /// Total columnar water vapour content [kg/m²] exceeded for `p`% (P.836-6).
    pub fn total_water_vapour_content(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
        p: f64,
    ) -> Result<f64, ItuProviderError> {
        self.interpolate_prob_18(V_836, lat.to_degrees(), lon.to_degrees(), p)
    }

    /// Wet-term radio refractivity (Nwet) exceeded for `p`% (P.453, grid).
    pub fn map_wet_term_radio_refractivity(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
        p: f64,
    ) -> Result<f64, ItuProviderError> {
        self.interpolate_prob_18(NWET_453, lat.to_degrees(), lon.to_degrees(), p)
    }

    /// Mean annual 0°C isotherm height (P.839, grid). Data is in kilometres.
    pub fn isotherm_0c_height(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
    ) -> Result<lox_core::units::Distance, ItuProviderError> {
        let g = self.grid(ESA0HEIGHT)?;
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
        let h0_km = self
            .grid(ESA0HEIGHT)?
            .bilinear(lat.to_degrees(), lon.to_degrees());
        Ok(lox_core::units::Distance::kilometers(h0_km + 0.36))
    }

    /// Total columnar content of reduced cloud liquid water [kg/m²] exceeded for `p`% (P.840).
    pub fn columnar_content_reduced_liquid(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
        p: f64,
    ) -> Result<f64, ItuProviderError> {
        self.interpolate_prob_18(LRED_840, lat.to_degrees(), lon.to_degrees(), p)
    }

    /// Cloud attenuation in dB on a slant path (P.840-9).
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
        let m = self.grid(M_840)?.bilinear(lat_deg, lon_deg);
        let sigma = self.grid(SIGMA_840)?.bilinear(lat_deg, lon_deg);
        let pclw = self.grid(PCLW_840)?.bilinear(lat_deg, lon_deg);
        Ok(crate::p840::LognormalCoefficients { m, sigma, pclw })
    }

    /// Rainfall rate exceeded 0.01% of the year [mm/h] (P.837).
    pub fn rainfall_rate_r001(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
    ) -> Result<f64, ItuProviderError> {
        let g = self.grid(R001)?;
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
            mt[month] = self.grid(MT_MONTH + month)?.bilinear(lat_deg, lon_deg);
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

    /// Rain attenuation in dB exceeded for `p`% of the average year (P.618).
    #[allow(clippy::too_many_arguments)]
    pub fn rain_attenuation(
        &self,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
        frequency: lox_core::units::Frequency,
        elevation: lox_core::units::Angle,
        p: f64,
        polarisation_tilt: lox_core::units::Angle,
        station_altitude: Option<lox_core::units::Distance>,
    ) -> Result<lox_core::units::Decibel, ItuProviderError> {
        let lat_deg = lat.to_degrees();
        let f_ghz = frequency.to_gigahertz();
        let el_deg = elevation.to_degrees().max(5.0);
        let tau_deg = polarisation_tilt.to_degrees();
        let hs_km = match station_altitude {
            Some(d) => d.to_kilometers(),
            None => self.topographic_altitude(lat, lon)?.to_kilometers(),
        };
        let hr_km = self.rain_height(lat, lon)?.to_kilometers();
        let r001 = self.rainfall_rate_r001(lat, lon)?;
        let a = crate::p618::rain_attenuation_core(
            lat_deg, f_ghz, el_deg, p, tau_deg, hs_km, hr_km, r001,
        );
        Ok(lox_core::units::Decibel::new(a))
    }

    /// Tropospheric scintillation attenuation in dB exceeded for `p`% (P.618).
    ///
    /// Parameter order mirrors the legacy `p618::scintillation_attenuation` free fn.
    #[allow(clippy::too_many_arguments)]
    pub fn scintillation_attenuation(
        &self,
        frequency: lox_core::units::Frequency,
        elevation: lox_core::units::Angle,
        p: f64,
        diameter: lox_core::units::Distance,
        eta: f64,
        n_wet: Option<f64>,
        lat: lox_core::units::Angle,
        lon: lox_core::units::Angle,
    ) -> Result<lox_core::units::Decibel, ItuProviderError> {
        let f_ghz = frequency.to_gigahertz();
        let el_deg = elevation.to_degrees().max(5.0);
        let d_m = diameter.to_meters();
        let n_wet = match n_wet {
            Some(n) => n,
            None => self.map_wet_term_radio_refractivity(lat, lon, 50.0)?,
        };
        let sigma =
            crate::p618::scintillation_attenuation_sigma_raw(f_ghz, el_deg, d_m, eta, n_wet);
        let log_p = p.log10();
        let a_p = -0.061 * log_p.powi(3) + 0.072 * log_p.powi(2) - 1.71 * log_p + 3.0;
        Ok(lox_core::units::Decibel::new(a_p * sigma))
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

    fn write_bundle(manifest_json: &str, entries: &[(&str, &[u8])]) -> NamedTempFile {
        let f = NamedTempFile::new().unwrap();
        let mut writer = zip::ZipWriter::new(f.reopen().unwrap());
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        writer.start_file(MANIFEST_ENTRY, opts).unwrap();
        writer.write_all(manifest_json.as_bytes()).unwrap();
        for (name, bytes) in entries {
            writer.start_file(*name, opts).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap();
        f
    }

    fn synth_bundle_with_manifest(manifest_json: &str) -> NamedTempFile {
        write_bundle(manifest_json, &[])
    }

    const VALID_MANIFEST: &str = r#"{"version":"1","upstream":"itur-0.4.0","grids":[]}"#;

    /// Bundle carrying a real 3×4 P.1511 topography grid (and its lat/lon
    /// companions), so `grid(TOPO)` resolves. Value at (lat=0, lon=10) is 6.0.
    fn topo_bundle() -> NamedTempFile {
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
        write_bundle(
            VALID_MANIFEST,
            &[
                ("1511/v2_lat.npy", &lat),
                ("1511/v2_lon.npy", &lon),
                ("1511/v2_topo.npy", &val),
            ],
        )
    }

    #[test]
    fn open_reads_manifest_upstream() {
        let f = synth_bundle_with_manifest(VALID_MANIFEST);
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
        let writer = zip::ZipWriter::new(f.reopen().unwrap());
        writer.finish().unwrap();
        let err = ItuProvider::open(f.path()).unwrap_err();
        assert!(matches!(err, ItuProviderError::MissingEntry(s) if s == "manifest.json"));
    }

    #[test]
    fn open_rejects_nonexistent_file() {
        let err = ItuProvider::open("/nonexistent/path/lox-itur-data.npz").unwrap_err();
        assert!(matches!(err, ItuProviderError::Open { .. }));
    }

    #[test]
    fn slot_ids_match_specs() {
        // Drift guard: the id constants must line up with grid_specs() order.
        let s = grid_specs();
        assert_eq!(s.len(), N_GRIDS);
        assert_eq!(s[TOPO].val, "1511/v2_topo.npy");
        assert_eq!(s[T_ANNUAL].val, "1510/v1_t_annual.npy");
        assert_eq!(s[T_MONTH + 6].val, "1510/v1_t_month07.npy");
        assert_eq!(s[ESA0HEIGHT].val, "839/v4_esa0height.npy");
        assert_eq!(s[R001].val, "837/v7_r001.npy");
        assert_eq!(s[MT_MONTH + 6].val, "837/v7_mt_month07.npy");
        assert_eq!(s[M_840].val, "840/v7_m.npy");
        assert_eq!(s[SIGMA_840].val, "840/v7_sigma.npy");
        assert_eq!(s[PCLW_840].val, "840/v7_pclw.npy");
        assert_eq!(s[RHO_836].val, "836/v6_rho_01.npy");
        assert_eq!(s[RHO_836 + 4].val, "836/v6_rho_1.npy");
        assert_eq!(s[V_836].val, "836/v6_v_01.npy");
        assert_eq!(s[NWET_453].val, "453/v13_nwet_annual_01.npy");
        assert_eq!(s[LRED_840 + 17].val, "840/v7_lred_99.npy");
    }

    #[test]
    fn grid_loads_and_caches() {
        let f = topo_bundle();
        let p = ItuProvider::open(f.path()).unwrap();
        let g1 = p.grid(TOPO).unwrap();
        assert!((g1.bilinear(0.0, 10.0) - 6.0).abs() < 1e-12);
        // Cache hit returns the very same stored grid (no reload, no copy).
        let g2 = p.grid(TOPO).unwrap();
        assert!(std::ptr::eq(g1, g2));
    }

    #[test]
    fn grid_missing_entry() {
        // Manifest present but no P.1511 entries → first companion read fails.
        let f = synth_bundle_with_manifest(VALID_MANIFEST);
        let p = ItuProvider::open(f.path()).unwrap();
        let err = p.grid(TOPO).unwrap_err();
        assert!(matches!(err, ItuProviderError::MissingEntry(s) if s == "1511/v2_lat.npy"));
    }
}

#[cfg(test)]
pub(crate) mod test_fixture {
    use std::path::PathBuf;
    use std::sync::OnceLock;

    use super::ItuProvider;

    /// Shared provider for in-module unit tests. Bundle resolution:
    /// `LOX_ITUR_BUNDLE` env var, else `<workspace>/target/lox-itur-data.npz`.
    pub fn provider() -> &'static ItuProvider {
        static P: OnceLock<ItuProvider> = OnceLock::new();
        P.get_or_init(|| {
            let path = std::env::var("LOX_ITUR_BUNDLE")
                .ok()
                .map(PathBuf::from)
                .or_else(|| {
                    let m = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                    let ws = m.parent()?.parent()?;
                    Some(ws.join("target").join("lox-itur-data.npz"))
                })
                .filter(|p| p.exists())
                .expect(
                    "tests need target/lox-itur-data.npz; \
                     run `just lox-itur-pack <wheel>` or set LOX_ITUR_BUNDLE.",
                );
            ItuProvider::open(path).expect("open bundle")
        })
    }
}
