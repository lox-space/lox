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

use lox_core::comms::PropagationLosses;
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

impl ItuProvider {
    /// Computes excess propagation losses on a slant path from ITU-R models.
    ///
    /// Combines rain (P.618), gaseous (P.676), cloud (P.840), and
    /// scintillation (P.618) attenuation. The total follows the ITU-R P.618
    /// §2.5 combination `A_T = A_g + √((A_r + A_c)² + A_s²)`: the rain,
    /// gaseous, and cloud lines carry the raw model outputs and the
    /// combination residual is attributed to the scintillation line, so
    /// [`PropagationLosses::total`] equals `A_T` and
    /// [`PropagationLosses::absorptive`] equals `A_r + A_g + A_c` — the
    /// attenuation excluding scintillation, per P.618 §8.2.
    ///
    /// # Arguments
    ///
    /// * `lat` — Latitude
    /// * `lon` — Longitude
    /// * `frequency` — Frequency
    /// * `elevation` — Elevation angle (clamped to ≥ 5°)
    /// * `p` — Exceedance probability (% of average year)
    /// * `diameter` — Physical antenna diameter
    /// * `polarisation_tilt` — Polarisation tilt angle (45° for circular)
    #[allow(clippy::too_many_arguments)]
    pub fn propagation_losses(
        &self,
        lat: Angle,
        lon: Angle,
        frequency: Frequency,
        elevation: Angle,
        p: f64,
        diameter: Distance,
        polarisation_tilt: Angle,
    ) -> Result<PropagationLosses, ItuProviderError> {
        let f_ghz = frequency.to_gigahertz();
        // ITU-R P.618/P.676 approximate methods are only valid for el ≥ 5°.
        let el_deg = elevation.to_degrees().max(5.0);

        let hs_km = self.topographic_altitude(lat, lon)?.to_kilometers();

        // Gaseous attenuation (P.676 approximate method)
        let p_hpa = p835::standard_pressure(Distance::kilometers(hs_km)).to_hpa();
        let t_k = self.surface_mean_temperature(lat, lon)?.to_kelvin();
        let rho = self.surface_water_vapour_density(lat, lon, p.max(0.1))?;
        let (a_o, a_w) = p676::gaseous_attenuation_slant_path_raw(f_ghz, el_deg, p_hpa, rho, t_k);
        let a_gas = a_o + a_w;

        // Rain attenuation (P.618)
        let a_rain = self
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
        let a_cloud = self.cloud_attenuation(lat, lon, elevation, frequency, p.max(0.1))?;

        // Scintillation (P.618 + P.453 for N_wet)
        let a_scint = self
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

        // ITU-R combined total: A_T = A_g + √((A_r + A_c)² + A_s²). The
        // combination residual is booked on the scintillation line (clamped
        // against float rounding when A_s ≈ 0) so the lines sum to A_T.
        let a_total = a_gas + ((a_rain + a_cloud).powi(2) + a_scint.powi(2)).sqrt();
        let a_scint_eff = (a_total - a_gas - a_rain - a_cloud).max(0.0);

        Ok(PropagationLosses::builder()
            .rain(Decibel::new(a_rain))
            .gaseous(Decibel::new(a_gas))
            .cloud(Decibel::new(a_cloud))
            .scintillation(Decibel::new(a_scint_eff))
            .build()?)
    }
}

#[cfg(test)]
mod tests {
    use lox_core::comms::LossKind;
    use lox_test_utils::assert_approx_eq;

    use crate::provider::test_fixture::provider;

    use super::*;

    /// Madrid fixture: the emitted lines must match the raw sub-model
    /// outputs, with the P.618 §2.5 combination residual booked on the
    /// scintillation line so that the total equals the combined attenuation
    /// and the absorptive part excludes scintillation entirely.
    #[test]
    fn test_propagation_losses_madrid_matches_p618_combination() {
        let p = provider();
        let lat = Angle::degrees(40.4);
        let lon = Angle::degrees(-3.7);
        let frequency = Frequency::gigahertz(20.0);
        let elevation = Angle::degrees(30.0);
        let exceedance = 1.0;
        let diameter = Distance::meters(1.2);
        let tilt = Angle::degrees(45.0);

        let losses = p
            .propagation_losses(lat, lon, frequency, elevation, exceedance, diameter, tilt)
            .unwrap();

        // Recompute the raw sub-model outputs independently.
        let hs = p.topographic_altitude(lat, lon).unwrap();
        let p_hpa = p835::standard_pressure(hs).to_hpa();
        let t_k = p.surface_mean_temperature(lat, lon).unwrap().to_kelvin();
        let rho = p
            .surface_water_vapour_density(lat, lon, exceedance)
            .unwrap();
        let (a_o, a_w) = p676::gaseous_attenuation_slant_path_raw(
            frequency.to_gigahertz(),
            elevation.to_degrees(),
            p_hpa,
            rho,
            t_k,
        );
        let a_gas = a_o + a_w;
        let a_rain = p
            .rain_attenuation(lat, lon, frequency, elevation, exceedance, tilt, Some(hs))
            .unwrap()
            .as_f64();
        let a_cloud = p
            .cloud_attenuation(lat, lon, elevation, frequency, exceedance)
            .unwrap();
        let a_scint = p
            .scintillation_attenuation(
                frequency, elevation, exceedance, diameter, 0.5, None, lat, lon,
            )
            .unwrap()
            .as_f64();
        let a_total = a_gas + ((a_rain + a_cloud).powi(2) + a_scint.powi(2)).sqrt();

        // Sanity: the fixture exercises every term.
        assert!(a_rain > 0.0 && a_gas > 0.0 && a_cloud > 0.0 && a_scint > 0.0);

        let lines = losses.lines();
        assert_eq!(lines.len(), 4);
        assert_eq!(*lines[0].kind(), LossKind::Rain);
        assert_approx_eq!(lines[0].value().as_f64(), a_rain, rtol <= 1e-12);
        assert_eq!(*lines[1].kind(), LossKind::Gaseous);
        assert_approx_eq!(lines[1].value().as_f64(), a_gas, rtol <= 1e-12);
        assert_eq!(*lines[2].kind(), LossKind::Cloud);
        assert_approx_eq!(lines[2].value().as_f64(), a_cloud, rtol <= 1e-12);
        // The scintillation line carries the combination residual, which the
        // RSS combination keeps at or below the raw scintillation fade.
        assert_eq!(*lines[3].kind(), LossKind::Scintillation);
        assert!(lines[3].value().as_f64() >= 0.0);
        assert!(lines[3].value().as_f64() <= a_scint);

        // total() is the ITU-exact combined attenuation — counted once.
        assert_approx_eq!(losses.total().as_f64(), a_total, rtol <= 1e-12);
        // absorptive() excludes scintillation per P.618 §8.2.
        assert_approx_eq!(
            losses.absorptive().as_f64(),
            a_rain + a_gas + a_cloud,
            rtol <= 1e-12
        );
    }
}
