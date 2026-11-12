// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Python bindings for ITU-R atmospheric propagation models.

use std::path::PathBuf;
use std::sync::Arc;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

use crate::units::python::{
    PyAngle, PyDecibel, PyDistance, PyFrequency, PyPressure, PyTemperature,
};

use lox_core::units::{Angle, Decibel};
use lox_itur::ItuProvider;

use crate::comms::python::{PyElevationProfile, PyPropagationLosses, PySitePropagation};

/// An open ITU-R data bundle (`lox-itur-data.npz`).
///
/// Grid-based recommendations (rain, cloud, scintillation, topography, …) are
/// served as methods on this object. Build a bundle once with
/// `cargo run -p lox-itur --bin pack -- <itur-wheel.whl> lox-itur-data.npz`.
#[pyclass(name = "ItuProvider", module = "lox_space", frozen)]
pub struct PyItuProvider(pub Arc<ItuProvider>);

#[pymethods]
impl PyItuProvider {
    /// Opens an ITU-R data bundle.
    ///
    /// Args:
    ///     path: filesystem path to lox-itur-data.npz.
    ///
    /// Raises:
    ///     RuntimeError: if the file is missing, malformed, or the manifest
    ///         version is unsupported.
    #[new]
    fn new(path: PathBuf) -> PyResult<Self> {
        let p = ItuProvider::open(&path).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self(Arc::new(p)))
    }

    /// The upstream `itur` package version this bundle was built from.
    fn upstream_version(&self) -> &str {
        self.0.upstream_version()
    }

    /// Returns the topographic altitude above mean sea level (P.1511-2).
    fn topographic_altitude(&self, lat: PyAngle, lon: PyAngle) -> PyResult<PyDistance> {
        let d = self
            .0
            .topographic_altitude(lat.0, lon.0)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyDistance(d))
    }

    /// Returns the annual mean surface temperature (P.1510-1).
    fn surface_mean_temperature(&self, lat: PyAngle, lon: PyAngle) -> PyResult<PyTemperature> {
        let t = self
            .0
            .surface_mean_temperature(lat.0, lon.0)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyTemperature(t))
    }

    /// Returns the mean annual rain height (P.839-4).
    fn rain_height(&self, lat: PyAngle, lon: PyAngle) -> PyResult<PyDistance> {
        let d = self
            .0
            .rain_height(lat.0, lon.0)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyDistance(d))
    }

    /// Returns the rainfall rate exceeded for a given probability (P.837-7).
    ///
    /// Args:
    ///     lat: Latitude.
    ///     lon: Longitude.
    ///     probability: Exceedance probability (% of average year).
    ///
    /// Returns:
    ///     Rainfall rate in mm/h.
    fn rainfall_rate_mm_per_hour(
        &self,
        lat: PyAngle,
        lon: PyAngle,
        probability: f64,
    ) -> PyResult<f64> {
        self.0
            .rainfall_rate(lat.0, lon.0, probability)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Computes rain attenuation exceeded for a given probability (P.618-13).
    ///
    /// Args:
    ///     lat: Latitude.
    ///     lon: Longitude.
    ///     frequency: Frequency.
    ///     elevation: Elevation angle.
    ///     probability: Exceedance probability (% of average year).
    ///     polarisation_tilt: Polarisation tilt angle (default 45 deg for circular).
    ///     station_altitude: Station altitude (default: looked up from P.1511).
    ///
    /// Returns:
    ///     Rain attenuation.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (lat, lon, frequency, elevation, probability, polarisation_tilt=None, station_altitude=None))]
    fn rain_attenuation(
        &self,
        lat: PyAngle,
        lon: PyAngle,
        frequency: PyFrequency,
        elevation: PyAngle,
        probability: f64,
        polarisation_tilt: Option<PyAngle>,
        station_altitude: Option<PyDistance>,
    ) -> PyResult<PyDecibel> {
        let tau = polarisation_tilt
            .map(|a| a.0)
            .unwrap_or(Angle::degrees(45.0));
        let d = self
            .0
            .rain_attenuation(
                lat.0,
                lon.0,
                frequency.0,
                elevation.0,
                probability,
                tau,
                station_altitude.map(|s| s.0),
            )
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyDecibel(d))
    }

    /// Computes cloud attenuation on a slant path (P.840-9).
    ///
    /// Args:
    ///     lat: Latitude.
    ///     lon: Longitude.
    ///     elevation: Elevation angle.
    ///     frequency: Frequency.
    ///     probability: Exceedance probability (% of average year).
    ///
    /// Returns:
    ///     Cloud attenuation.
    fn cloud_attenuation(
        &self,
        lat: PyAngle,
        lon: PyAngle,
        elevation: PyAngle,
        frequency: PyFrequency,
        probability: f64,
    ) -> PyResult<PyDecibel> {
        let v = self
            .0
            .cloud_attenuation(lat.0, lon.0, elevation.0, frequency.0, probability)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyDecibel(Decibel::decibels(v)))
    }

    /// Computes tropospheric scintillation fade depth exceeded for a given
    /// probability (P.618-13).
    ///
    /// Args:
    ///     frequency: Frequency.
    ///     elevation: Elevation angle.
    ///     probability: Exceedance probability (% of average year).
    ///     diameter: Physical antenna diameter.
    ///     eta: Antenna efficiency (default 0.5).
    ///     lat: Latitude (for N_wet lookup).
    ///     lon: Longitude (for N_wet lookup).
    ///
    /// Returns:
    ///     Scintillation attenuation.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (frequency, elevation, probability, diameter, eta=0.5, lat=None, lon=None))]
    fn scintillation_attenuation(
        &self,
        frequency: PyFrequency,
        elevation: PyAngle,
        probability: f64,
        diameter: PyDistance,
        eta: f64,
        lat: Option<PyAngle>,
        lon: Option<PyAngle>,
    ) -> PyResult<PyDecibel> {
        let lat_angle = lat.map(|a| a.0).unwrap_or(Angle::degrees(0.0));
        let lon_angle = lon.map(|a| a.0).unwrap_or(Angle::degrees(0.0));
        let d = self
            .0
            .scintillation_attenuation(
                frequency.0,
                elevation.0,
                probability,
                diameter.0,
                eta,
                None,
                lat_angle,
                lon_angle,
            )
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyDecibel(d))
    }

    /// Computes excess propagation losses on a slant path, returning
    /// itemized lines as a `PropagationLosses` object.
    ///
    /// The total follows the ITU-R P.618 combination, with the combination
    /// residual booked on the scintillation line; the absorptive part
    /// excludes scintillation.
    ///
    /// Args:
    ///     lat: Latitude.
    ///     lon: Longitude.
    ///     frequency: Frequency.
    ///     elevation: Elevation angle.
    ///     probability: Exceedance probability (% of average year).
    ///     diameter: Physical antenna diameter.
    ///     polarisation_tilt: Polarisation tilt angle (default 45 deg for circular).
    ///
    /// Returns:
    ///     PropagationLosses with rain, gaseous, cloud, and scintillation lines.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (lat, lon, frequency, elevation, probability, diameter, polarisation_tilt=None))]
    fn propagation_losses(
        &self,
        lat: PyAngle,
        lon: PyAngle,
        frequency: PyFrequency,
        elevation: PyAngle,
        probability: f64,
        diameter: PyDistance,
        polarisation_tilt: Option<PyAngle>,
    ) -> PyResult<PyPropagationLosses> {
        let tau = polarisation_tilt
            .map(|a| a.0)
            .unwrap_or(Angle::degrees(45.0));
        let losses = self
            .0
            .propagation_losses(
                lat.0,
                lon.0,
                frequency.0,
                elevation.0,
                probability,
                diameter.0,
                tau,
            )
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyPropagationLosses(losses))
    }

    /// Tabulates `propagation_losses` over a grid of elevations.
    ///
    /// The models are only valid from 5° upwards and hold their 5° value
    /// below it, and so does the profile.
    ///
    /// Args:
    ///     lat: Latitude.
    ///     lon: Longitude.
    ///     frequency: Carrier frequency.
    ///     probability: Exceedance probability (% of average year).
    ///     diameter: Physical antenna diameter.
    ///     polarisation_tilt: Polarisation tilt angle (default 45 deg for circular).
    ///     elevations: Strictly increasing elevation grid in (0°, 90°].
    ///         Defaults to 14 elevations from 5° to 90°, denser towards the
    ///         horizon.
    ///
    /// Returns:
    ///     ElevationProfile for the given carrier.
    ///
    /// Raises:
    ///     ValueError: If the elevations do not form a valid grid.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (lat, lon, frequency, probability, diameter, polarisation_tilt=None, elevations=None))]
    fn elevation_profile(
        &self,
        lat: PyAngle,
        lon: PyAngle,
        frequency: PyFrequency,
        probability: f64,
        diameter: PyDistance,
        polarisation_tilt: Option<PyAngle>,
        elevations: Option<Vec<PyAngle>>,
    ) -> PyResult<PyElevationProfile> {
        let tau = polarisation_tilt
            .map(|a| a.0)
            .unwrap_or(Angle::degrees(45.0));
        let elevations: Vec<Angle> = match elevations {
            Some(elevations) => elevations.into_iter().map(|a| a.0).collect(),
            None => lox_itur::DEFAULT_PROFILE_ELEVATIONS_DEG
                .map(Angle::degrees)
                .to_vec(),
        };
        self.0
            .elevation_profile(
                lat.0,
                lon.0,
                frequency.0,
                probability,
                diameter.0,
                tau,
                &elevations,
            )
            .map(PyElevationProfile)
            .map_err(profile_error)
    }

    /// Computes the elevation-dependent propagation losses of a site: one
    /// `ElevationProfile` per carrier, on the default elevation grid.
    ///
    /// Pass the downlink and the uplink carrier to cover both directions of
    /// a ground station in one call.
    ///
    /// Args:
    ///     lat: Latitude.
    ///     lon: Longitude.
    ///     carriers: Carrier frequencies, more than 1 % apart.
    ///     probability: Exceedance probability (% of average year).
    ///     diameter: Physical antenna diameter.
    ///     polarisation_tilt: Polarisation tilt angle (default 45 deg for circular).
    ///
    /// Returns:
    ///     SitePropagation, to be passed to `LinkBudgetAnalysis`.
    ///
    /// Raises:
    ///     ValueError: If two carriers lie within 1 % of each other.
    #[pyo3(signature = (lat, lon, carriers, probability, diameter, polarisation_tilt=None))]
    fn site_propagation(
        &self,
        lat: PyAngle,
        lon: PyAngle,
        carriers: Vec<PyFrequency>,
        probability: f64,
        diameter: PyDistance,
        polarisation_tilt: Option<PyAngle>,
    ) -> PyResult<PySitePropagation> {
        let tau = polarisation_tilt
            .map(|a| a.0)
            .unwrap_or(Angle::degrees(45.0));
        let carriers: Vec<_> = carriers.into_iter().map(|f| f.0).collect();
        self.0
            .site_propagation(lat.0, lon.0, &carriers, probability, diameter.0, tau)
            .map(PySitePropagation)
            .map_err(profile_error)
    }

    fn __repr__(&self) -> String {
        format!("ItuProvider(upstream={:?})", self.0.upstream_version())
    }
}

/// Computes gaseous attenuation on a slant path (P.676-12 approximate method).
///
/// Args:
///     frequency: Frequency.
///     elevation: Elevation angle.
///     pressure: Surface pressure.
///     rho_g_per_m3: Surface water vapour density in g/m³.
///     temperature: Surface temperature.
///
/// Returns:
///     Tuple of (oxygen attenuation, water vapour attenuation).
#[pyfunction]
pub fn gaseous_attenuation_slant_path(
    frequency: PyFrequency,
    elevation: PyAngle,
    pressure: PyPressure,
    rho_g_per_m3: f64,
    temperature: PyTemperature,
) -> (PyDecibel, PyDecibel) {
    let (a_o, a_w) = lox_itur::p676::gaseous_attenuation_slant_path(
        frequency.0,
        elevation.0,
        pressure.0,
        rho_g_per_m3,
        temperature.0,
    );
    (PyDecibel(a_o), PyDecibel(a_w))
}

/// Computes the specific attenuation from rain (P.838-3).
///
/// Args:
///     rain_rate_mm_per_hour: Rainfall rate in mm/h.
///     frequency: Frequency.
///     elevation: Elevation angle.
///     polarisation_tilt: Polarisation tilt angle (default 45° for circular).
///
/// Returns:
///     Specific rain attenuation in dB/km.
#[pyfunction]
#[pyo3(signature = (rain_rate_mm_per_hour, frequency, elevation, polarisation_tilt=None))]
pub fn rain_specific_attenuation_db_per_km(
    rain_rate_mm_per_hour: f64,
    frequency: PyFrequency,
    elevation: PyAngle,
    polarisation_tilt: Option<PyAngle>,
) -> f64 {
    let tau = polarisation_tilt
        .map(|a| a.0)
        .unwrap_or(Angle::degrees(45.0));
    lox_itur::p838::rain_specific_attenuation(rain_rate_mm_per_hour, frequency.0, elevation.0, tau)
}

/// Registers the pure-formula ITU-R functions with the Python module.
///
/// Grid-based recommendations are exposed as methods on `ItuProvider`.
pub fn register_itur_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(gaseous_attenuation_slant_path, m)?)?;
    m.add_function(wrap_pyfunction!(rain_specific_attenuation_db_per_km, m)?)?;
    Ok(())
}

/// Maps a profile construction error: an invalid grid or carrier set is the
/// caller's `ValueError`, anything else a `RuntimeError` like the other
/// provider methods.
fn profile_error(err: lox_itur::ItuProviderError) -> PyErr {
    match err {
        lox_itur::ItuProviderError::InvalidProfile(_)
        | lox_itur::ItuProviderError::InvalidSite(_) => PyValueError::new_err(err.to_string()),
        _ => PyRuntimeError::new_err(err.to_string()),
    }
}
