// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![warn(missing_docs)]

//! ITU-R P-series atmospheric propagation models.
//!
//! This crate implements ITU-R recommendations for computing atmospheric attenuation
//! on Earth-to-space and terrestrial radio paths. It is a Rust port of the Python
//! [ITU-Rpy](https://github.com/iportillo/ITU-Rpy) library.
//!
//! # Implemented Recommendations
//!
//! - **P.618-14** — Propagation data for Earth-space telecommunication systems
//! - **P.676-13** — Attenuation by atmospheric gases
//! - **P.835** — Reference standard atmospheres
//! - **P.836** — Water vapour surface density and total columnar content
//! - **P.837** — Characteristics of precipitation for propagation modelling
//! - **P.838** — Specific attenuation model for rain
//! - **P.839** — Rain height model for prediction methods
//! - **P.840-9** — Attenuation due to clouds and fog
//! - **P.1510** — Annual mean surface temperature
//! - **P.1511** — Topography for Earth-to-space propagation modelling
//!
//! # Bundled Data
//!
//! Grid-based models require reference data from the ITU, distributed via the
//! upstream `itur` Python package. Build a `lox-itur-data.npz` bundle once:
//!
//! ```text
//! pip download --no-deps itur==0.4.0
//! cargo run -p lox-itur --bin pack -- itur-0.4.0-py2.py3-none-any.whl lox-itur-data.npz
//! ```
//!
//! Then open it via [`ItuProvider`]:
//!
//! ```ignore
//! let provider = lox_itur::ItuProvider::open("lox-itur-data.npz")?;
//! let alt = provider.topographic_altitude(lat, lon)?;
//! ```
//!
//! Grid-bearing recommendations expose their data as methods on [`ItuProvider`].
//! Pure formulae (P.835, P.676 line tables, P.838 rain-specific attenuation, etc.)
//! remain free functions in the `pXXX` modules.

pub(crate) mod grid;
pub(crate) mod manifest;
pub(crate) mod npz;
pub mod p1510;
pub mod p1511;
pub mod p453;
pub mod p618;
pub mod p676;
pub mod p835;
pub mod p836;
pub mod p837;
pub mod p838;
pub mod p839;
pub mod p840;
pub mod provider;

pub use provider::{ItuProvider, ItuProviderError};

use lox_core::units::{Angle, Decibel, Distance, Frequency};

/// Builds a serialized manifest for the packager.
///
/// Kept on the library side (not in `bin/pack.rs`) so the schema stays in
/// one place and changes flow through `Manifest`'s serde impl.
#[doc(hidden)]
pub fn manifest_for_packager(upstream: &str, grids: Vec<String>) -> Vec<u8> {
    crate::manifest::Manifest {
        version: crate::manifest::FORMAT_VERSION.to_owned(),
        upstream: upstream.to_owned(),
        grids,
    }
    .to_json_bytes()
}

/// Environmental losses (rain, atmospheric, etc.) computed from ITU-R models.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EnvironmentalLosses {
    /// Rain attenuation.
    pub rain: Decibel,
    /// Gaseous absorption.
    pub gaseous: Decibel,
    /// Scintillation loss.
    pub scintillation: Decibel,
    /// General atmospheric loss.
    pub atmospheric: Decibel,
    /// Cloud attenuation.
    pub cloud: Decibel,
    /// Depolarization loss.
    pub depolarization: Decibel,
}

impl EnvironmentalLosses {
    /// Returns zero environmental losses.
    pub fn none() -> Self {
        Self {
            rain: Decibel::new(0.0),
            gaseous: Decibel::new(0.0),
            scintillation: Decibel::new(0.0),
            atmospheric: Decibel::new(0.0),
            cloud: Decibel::new(0.0),
            depolarization: Decibel::new(0.0),
        }
    }

    /// Returns the total environmental loss in dB.
    pub fn total(&self) -> Decibel {
        self.rain
            + self.gaseous
            + self.scintillation
            + self.atmospheric
            + self.cloud
            + self.depolarization
    }

    /// Computes atmospheric attenuation on a slant path from ITU-R models.
    ///
    /// Combines rain (P.618), gaseous (P.676), cloud (P.840), and
    /// scintillation (P.618) attenuation.
    ///
    /// # Arguments
    ///
    /// * `provider` — Open [`ItuProvider`] supplying the gridded reference data
    /// * `lat` — Latitude
    /// * `lon` — Longitude
    /// * `frequency` — Frequency
    /// * `elevation` — Elevation angle (clamped to ≥ 5°)
    /// * `p` — Exceedance probability (% of average year)
    /// * `diameter` — Physical antenna diameter
    /// * `polarisation_tilt` — Polarisation tilt angle (45° for circular)
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider: &ItuProvider,
        lat: Angle,
        lon: Angle,
        frequency: Frequency,
        elevation: Angle,
        p: f64,
        diameter: Distance,
        polarisation_tilt: Angle,
    ) -> Result<EnvironmentalLosses, ItuProviderError> {
        let f_ghz = frequency.to_gigahertz();
        // ITU-R P.618/P.676 approximate methods are only valid for el ≥ 5°.
        let el_deg = elevation.to_degrees().max(5.0);
        let tau_deg = polarisation_tilt.to_degrees();

        let hs_km = provider.topographic_altitude(lat, lon)?.to_kilometers();

        // Gaseous attenuation (P.676 approximate method)
        let p_hpa = p835::standard_pressure(Distance::kilometers(hs_km)).to_hpa();
        let t_k = provider.surface_mean_temperature(lat, lon)?.to_kelvin();
        let rho = provider.surface_water_vapour_density(lat, lon, p.max(0.1))?;
        let (a_o, a_w) = p676::gaseous_attenuation_slant_path_raw(f_ghz, el_deg, p_hpa, rho, t_k);
        let a_gas = a_o + a_w;

        // Rain attenuation (P.618)
        let a_rain = provider
            .rain_attenuation(
                lat,
                lon,
                frequency,
                elevation,
                p,
                polarisation_tilt,
                Some(Distance::kilometers(hs_km)),
            )?
            .as_f64();

        // Cloud attenuation (P.840)
        let a_cloud = provider.cloud_attenuation(lat, lon, elevation, frequency, p.max(0.1))?;

        // Scintillation (P.618 + P.453 for N_wet)
        let a_scint = provider
            .scintillation_attenuation(
                frequency,
                elevation,
                p.max(0.01),
                diameter,
                0.5,
                None,
                lat,
                lon,
            )?
            .as_f64();

        // Cross-polarization discrimination (P.618)
        let a_depol = if a_rain > 0.0 && (4.0..=55.0).contains(&f_ghz) {
            let xpd =
                p618::rain_cross_polarization_discrimination_raw(a_rain, f_ghz, el_deg, p, tau_deg);
            -xpd
        } else {
            0.0
        };

        // ITU-R combined total: A = Ag + sqrt((Ar + Ac)² + As²)
        let a_total = a_gas + ((a_rain + a_cloud).powi(2) + a_scint.powi(2)).sqrt();

        Ok(EnvironmentalLosses {
            rain: Decibel::new(a_rain),
            gaseous: Decibel::new(a_gas),
            scintillation: Decibel::new(a_scint),
            atmospheric: Decibel::new(a_total),
            cloud: Decibel::new(a_cloud),
            depolarization: Decibel::new(a_depol),
        })
    }
}
