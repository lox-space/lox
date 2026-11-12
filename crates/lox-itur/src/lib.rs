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

use lox_comms::{ElevationProfile, PropagationLosses, SitePropagation};
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

/// Default elevation grid, in degrees, of the profiles built by
/// [`ItuProvider::site_propagation`].
///
/// Denser towards the horizon, where the loss changes fastest; it starts at
/// 5°, below which the ITU-R models are not valid.
pub const DEFAULT_PROFILE_ELEVATIONS_DEG: [f64; 14] = [
    5.0, 6.0, 7.5, 10.0, 12.5, 15.0, 20.0, 25.0, 30.0, 40.0, 50.0, 65.0, 80.0, 90.0,
];

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
        let p_hpa = p835::standard_pressure(Distance::kilometers(hs_km)).to_hectopascals();
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
            .to_decibels();

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
            .to_decibels();

        // ITU-R combined total: A_T = A_g + √((A_r + A_c)² + A_s²). The
        // combination residual is booked on the scintillation line (clamped
        // against float rounding when A_s ≈ 0) so the lines sum to A_T.
        let a_total = a_gas + ((a_rain + a_cloud).powi(2) + a_scint.powi(2)).sqrt();
        let a_scint_eff = (a_total - a_gas - a_rain - a_cloud).max(0.0);

        Ok(PropagationLosses::builder()
            .rain(Decibel::decibels(a_rain))
            .gaseous(Decibel::decibels(a_gas))
            .cloud(Decibel::decibels(a_cloud))
            .scintillation(Decibel::decibels(a_scint_eff))
            .build()?)
    }

    /// Tabulates [`Self::propagation_losses`] over a grid of elevations.
    ///
    /// The resulting [`ElevationProfile`] lets a link budget follow the loss
    /// along a pass instead of applying one fixed figure. `elevations` must be
    /// strictly increasing and lie in `(0°, 90°]`;
    /// [`DEFAULT_PROFILE_ELEVATIONS_DEG`] is a grid on which interpolation
    /// stays within about 1.5 % of the direct computation up to 80°, and
    /// within 4 % from there to zenith, where P.618 rain attenuation is itself
    /// not monotonic.
    ///
    /// The underlying models are only valid from 5° upwards and hold their 5°
    /// value below it, and so does the profile: for a station tracking below
    /// 5° the loss near the horizon is underestimated.
    ///
    /// The remaining arguments are those of [`Self::propagation_losses`].
    #[allow(clippy::too_many_arguments)]
    pub fn elevation_profile(
        &self,
        lat: Angle,
        lon: Angle,
        frequency: Frequency,
        p: f64,
        diameter: Distance,
        polarisation_tilt: Angle,
        elevations: &[Angle],
    ) -> Result<ElevationProfile, ItuProviderError> {
        let samples = elevations
            .iter()
            .map(|&elevation| {
                self.propagation_losses(
                    lat,
                    lon,
                    frequency,
                    elevation,
                    p,
                    diameter,
                    polarisation_tilt,
                )
                .map(|losses| (elevation, losses))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ElevationProfile::new(frequency, samples)?)
    }

    /// Computes the elevation-dependent propagation losses of a site: one
    /// [`ElevationProfile`] per carrier, on the
    /// [`DEFAULT_PROFILE_ELEVATIONS_DEG`] grid.
    ///
    /// Pass the downlink and the uplink carrier to cover both directions of a
    /// ground station in one call. The remaining arguments are those of
    /// [`Self::propagation_losses`].
    pub fn site_propagation(
        &self,
        lat: Angle,
        lon: Angle,
        carriers: &[Frequency],
        p: f64,
        diameter: Distance,
        polarisation_tilt: Angle,
    ) -> Result<SitePropagation, ItuProviderError> {
        let elevations = DEFAULT_PROFILE_ELEVATIONS_DEG.map(Angle::degrees);
        let profiles = carriers
            .iter()
            .map(|&carrier| {
                self.elevation_profile(
                    lat,
                    lon,
                    carrier,
                    p,
                    diameter,
                    polarisation_tilt,
                    &elevations,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(SitePropagation::new(profiles)?)
    }
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;
    use lox_comms::LossKind;

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
        let p_hpa = p835::standard_pressure(hs).to_hectopascals();
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
            .to_decibels();
        let a_cloud = p
            .cloud_attenuation(lat, lon, elevation, frequency, exceedance)
            .unwrap();
        let a_scint = p
            .scintillation_attenuation(
                frequency, elevation, exceedance, diameter, 0.5, None, lat, lon,
            )
            .unwrap()
            .to_decibels();
        let a_total = a_gas + ((a_rain + a_cloud).powi(2) + a_scint.powi(2)).sqrt();

        // Sanity: the fixture exercises every term.
        assert!(a_rain > 0.0 && a_gas > 0.0 && a_cloud > 0.0 && a_scint > 0.0);

        let lines = losses.lines();
        assert_eq!(lines.len(), 4);
        assert_eq!(*lines[0].kind(), LossKind::Rain);
        assert_approx_eq!(lines[0].value().to_decibels(), a_rain, rtol <= 1e-12);
        assert_eq!(*lines[1].kind(), LossKind::Gaseous);
        assert_approx_eq!(lines[1].value().to_decibels(), a_gas, rtol <= 1e-12);
        assert_eq!(*lines[2].kind(), LossKind::Cloud);
        assert_approx_eq!(lines[2].value().to_decibels(), a_cloud, rtol <= 1e-12);
        // The scintillation line carries the combination residual, which the
        // RSS combination keeps at or below the raw scintillation fade.
        assert_eq!(*lines[3].kind(), LossKind::Scintillation);
        assert!(lines[3].value() >= Decibel::ZERO);
        assert!(lines[3].value().to_decibels() <= a_scint);

        // total() is the ITU-exact combined attenuation — counted once.
        assert_approx_eq!(losses.total().to_decibels(), a_total, rtol <= 1e-12);
        // absorptive() excludes scintillation per P.618 §8.2.
        assert_approx_eq!(
            losses.absorptive().to_decibels(),
            a_rain + a_gas + a_cloud,
            rtol <= 1e-12
        );
    }

    /// (lat°, lon°, carrier GHz, exceedance %): Madrid, Svalbard, Nemea,
    /// Singapore and Tromsø, from heavy rain to the median.
    const PROFILE_FIXTURES: [(f64, f64, f64, f64); 5] = [
        (40.4, -3.7, 20.0, 1.0),
        (78.23, 15.39, 26.0, 5.0),
        (37.8, 22.66, 26.0, 0.1),
        (1.3, 103.8, 8.0, 0.01),
        (69.66, 18.94, 26.0, 50.0),
    ];

    #[test]
    fn test_elevation_profile_matches_direct_computation_at_its_samples() {
        let p = provider();
        let (lat, lon) = (Angle::degrees(78.23), Angle::degrees(15.39));
        let frequency = Frequency::gigahertz(26.0);
        let diameter = Distance::meters(3.0);
        let tilt = Angle::degrees(45.0);
        let elevations = [10.0, 30.0, 60.0].map(Angle::degrees);

        let profile = p
            .elevation_profile(lat, lon, frequency, 5.0, diameter, tilt, &elevations)
            .unwrap();

        assert_eq!(profile.carrier(), frequency);
        assert_eq!(profile.samples().len(), 3);
        for &elevation in &elevations {
            let direct = p
                .propagation_losses(lat, lon, frequency, elevation, 5.0, diameter, tilt)
                .unwrap();
            assert_eq!(profile.at(elevation), direct);
        }
        // The loss grows towards the horizon.
        let total = |deg: f64| profile.at(Angle::degrees(deg)).total().to_decibels();
        assert!(total(10.0) > 2.0 * total(30.0));
        assert!(total(30.0) > total(60.0));
    }

    #[test]
    fn test_elevation_profile_rejects_an_invalid_grid() {
        let p = provider();
        let result = p.elevation_profile(
            Angle::degrees(40.4),
            Angle::degrees(-3.7),
            Frequency::gigahertz(20.0),
            1.0,
            Distance::meters(1.2),
            Angle::degrees(45.0),
            &[Angle::degrees(30.0), Angle::degrees(10.0)],
        );
        assert!(matches!(result, Err(ItuProviderError::InvalidProfile(_))));
    }

    /// The default grid must interpolate every loss line close to the direct
    /// model output, or a profile-driven link budget drifts from the model it
    /// stands in for.
    #[test]
    fn test_site_propagation_default_grid_tracks_direct_computation() {
        let p = provider();
        let diameter = Distance::meters(3.0);
        let tilt = Angle::degrees(45.0);
        for (lat, lon, f_ghz, exceedance) in PROFILE_FIXTURES {
            let (lat, lon) = (Angle::degrees(lat), Angle::degrees(lon));
            let frequency = Frequency::gigahertz(f_ghz);
            let site = p
                .site_propagation(lat, lon, &[frequency], exceedance, diameter, tilt)
                .unwrap();
            let profile = site.for_carrier(frequency).unwrap();

            let mut el_deg = 5.0;
            while el_deg <= 90.0 {
                let elevation = Angle::degrees(el_deg);
                let direct = p
                    .propagation_losses(lat, lon, frequency, elevation, exceedance, diameter, tilt)
                    .unwrap()
                    .total()
                    .to_decibels();
                let interpolated = profile.at(elevation).total().to_decibels();
                // P.618 rain attenuation is not monotonic in the last few
                // degrees before zenith, which no smooth interpolant follows.
                let rtol = if el_deg <= 80.0 { 0.015 } else { 0.04 };
                assert_approx_eq!(interpolated, direct, rtol <= rtol);
                el_deg += 0.25;
            }
        }
    }

    #[test]
    fn test_site_propagation_holds_one_profile_per_carrier() {
        let p = provider();
        let (lat, lon) = (Angle::degrees(69.66), Angle::degrees(18.94));
        let (down, up) = (Frequency::gigahertz(26.0), Frequency::gigahertz(8.0));
        let site = p
            .site_propagation(
                lat,
                lon,
                &[down, up],
                5.0,
                Distance::meters(3.0),
                Angle::degrees(45.0),
            )
            .unwrap();
        assert_eq!(site.profiles().len(), 2);
        assert_eq!(site.for_carrier(down).unwrap().carrier(), down);
        assert_eq!(site.for_carrier(up).unwrap().carrier(), up);
        assert_eq!(
            site.for_carrier(down).unwrap().samples().len(),
            DEFAULT_PROFILE_ELEVATIONS_DEG.len()
        );

        // Two carriers too close to tell apart cannot share a site.
        let result = p.site_propagation(
            lat,
            lon,
            &[down, Frequency::gigahertz(26.1)],
            5.0,
            Distance::meters(3.0),
            Angle::degrees(45.0),
        );
        assert!(matches!(result, Err(ItuProviderError::InvalidSite(_))));
    }
}
