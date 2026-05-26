// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Python bindings for ITU-R atmospheric propagation models.

use std::path::PathBuf;
use std::sync::Arc;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use crate::comms::python::PyDecibel;
use crate::units::python::{PyAngle, PyDistance, PyFrequency, PyPressure, PyTemperature};

use lox_core::units::{Angle, Decibel};
use lox_itur::{EnvironmentalLosses, ItuProvider};

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
    fn rainfall_rate(&self, lat: PyAngle, lon: PyAngle, probability: f64) -> PyResult<f64> {
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
        Ok(PyDecibel(Decibel::new(v)))
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

    /// Computes atmospheric attenuation on a slant path, returning individual
    /// contributions as an `EnvironmentalLosses` object.
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
    ///     EnvironmentalLosses with rain, gaseous, cloud, scintillation, and
    ///     depolarization fields populated.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (lat, lon, frequency, elevation, probability, diameter, polarisation_tilt=None))]
    fn atmospheric_attenuation_slant_path(
        &self,
        lat: PyAngle,
        lon: PyAngle,
        frequency: PyFrequency,
        elevation: PyAngle,
        probability: f64,
        diameter: PyDistance,
        polarisation_tilt: Option<PyAngle>,
    ) -> PyResult<PyEnvironmentalLosses> {
        let tau = polarisation_tilt
            .map(|a| a.0)
            .unwrap_or(Angle::degrees(45.0));
        let losses = EnvironmentalLosses::new(
            &self.0,
            lat.0,
            lon.0,
            frequency.0,
            elevation.0,
            probability,
            diameter.0,
            tau,
        )
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyEnvironmentalLosses(losses))
    }

    fn __repr__(&self) -> String {
        format!("ItuProvider(upstream={:?})", self.0.upstream_version())
    }
}

/// Atmospheric environmental losses computed from ITU-R models.
#[pyclass(
    name = "EnvironmentalLosses",
    module = "lox_space",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PyEnvironmentalLosses(pub EnvironmentalLosses);

#[pymethods]
impl PyEnvironmentalLosses {
    /// Computes atmospheric attenuation on a slant path from ITU-R models.
    ///
    /// Args:
    ///     provider: Open ItuProvider supplying the gridded reference data.
    ///     lat: Latitude.
    ///     lon: Longitude.
    ///     frequency: Frequency.
    ///     elevation: Elevation angle (clamped to >= 5 deg).
    ///     probability: Exceedance probability (% of average year).
    ///     diameter: Physical antenna diameter.
    ///     polarisation_tilt: Polarisation tilt angle (default 45 deg for circular).
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (provider, lat, lon, frequency, elevation, probability, diameter, polarisation_tilt=None))]
    fn new(
        provider: &PyItuProvider,
        lat: PyAngle,
        lon: PyAngle,
        frequency: PyFrequency,
        elevation: PyAngle,
        probability: f64,
        diameter: PyDistance,
        polarisation_tilt: Option<PyAngle>,
    ) -> PyResult<Self> {
        let tau = polarisation_tilt
            .map(|a| a.0)
            .unwrap_or(Angle::degrees(45.0));
        let losses = EnvironmentalLosses::new(
            &provider.0,
            lat.0,
            lon.0,
            frequency.0,
            elevation.0,
            probability,
            diameter.0,
            tau,
        )
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self(losses))
    }

    /// Returns zero environmental losses.
    #[staticmethod]
    fn none() -> Self {
        Self(EnvironmentalLosses::none())
    }

    /// Creates environmental losses from individual values.
    #[staticmethod]
    #[pyo3(signature = (rain=None, gaseous=None, scintillation=None, atmospheric=None, cloud=None, depolarization=None))]
    fn from_values(
        rain: Option<PyDecibel>,
        gaseous: Option<PyDecibel>,
        scintillation: Option<PyDecibel>,
        atmospheric: Option<PyDecibel>,
        cloud: Option<PyDecibel>,
        depolarization: Option<PyDecibel>,
    ) -> Self {
        Self(EnvironmentalLosses {
            rain: rain.map_or(Decibel::new(0.0), |d| d.0),
            gaseous: gaseous.map_or(Decibel::new(0.0), |d| d.0),
            scintillation: scintillation.map_or(Decibel::new(0.0), |d| d.0),
            atmospheric: atmospheric.map_or(Decibel::new(0.0), |d| d.0),
            cloud: cloud.map_or(Decibel::new(0.0), |d| d.0),
            depolarization: depolarization.map_or(Decibel::new(0.0), |d| d.0),
        })
    }

    /// Returns the total environmental loss.
    fn total(&self) -> PyDecibel {
        PyDecibel(self.0.total())
    }

    /// Rain attenuation.
    #[getter]
    fn rain(&self) -> PyDecibel {
        PyDecibel(self.0.rain)
    }
    /// Gaseous absorption.
    #[getter]
    fn gaseous(&self) -> PyDecibel {
        PyDecibel(self.0.gaseous)
    }
    /// Scintillation loss.
    #[getter]
    fn scintillation(&self) -> PyDecibel {
        PyDecibel(self.0.scintillation)
    }
    /// General atmospheric loss (combined total).
    #[getter]
    fn atmospheric(&self) -> PyDecibel {
        PyDecibel(self.0.atmospheric)
    }
    /// Cloud attenuation.
    #[getter]
    fn cloud(&self) -> PyDecibel {
        PyDecibel(self.0.cloud)
    }
    /// Depolarization loss.
    #[getter]
    fn depolarization(&self) -> PyDecibel {
        PyDecibel(self.0.depolarization)
    }

    fn __eq__(&self, other: &PyEnvironmentalLosses) -> bool {
        self.0.rain.as_f64() == other.0.rain.as_f64()
            && self.0.gaseous.as_f64() == other.0.gaseous.as_f64()
            && self.0.scintillation.as_f64() == other.0.scintillation.as_f64()
            && self.0.atmospheric.as_f64() == other.0.atmospheric.as_f64()
            && self.0.cloud.as_f64() == other.0.cloud.as_f64()
            && self.0.depolarization.as_f64() == other.0.depolarization.as_f64()
    }

    fn __repr__(&self) -> String {
        format!(
            "EnvironmentalLosses(rain={}, gaseous={}, scintillation={}, atmospheric={}, cloud={}, depolarization={})",
            PyDecibel(self.0.rain).__repr__(),
            PyDecibel(self.0.gaseous).__repr__(),
            PyDecibel(self.0.scintillation).__repr__(),
            PyDecibel(self.0.atmospheric).__repr__(),
            PyDecibel(self.0.cloud).__repr__(),
            PyDecibel(self.0.depolarization).__repr__(),
        )
    }
}

/// Computes gaseous attenuation on a slant path (P.676-12 approximate method).
///
/// Args:
///     frequency: Frequency.
///     elevation: Elevation angle.
///     pressure: Surface pressure.
///     rho: Surface water vapour density in g/m³.
///     temperature: Surface temperature.
///
/// Returns:
///     Tuple of (oxygen attenuation, water vapour attenuation).
#[pyfunction]
pub fn gaseous_attenuation_slant_path(
    frequency: PyFrequency,
    elevation: PyAngle,
    pressure: PyPressure,
    rho: f64,
    temperature: PyTemperature,
) -> (PyDecibel, PyDecibel) {
    let (a_o, a_w) = lox_itur::p676::gaseous_attenuation_slant_path(
        frequency.0,
        elevation.0,
        pressure.0,
        rho,
        temperature.0,
    );
    (PyDecibel(a_o), PyDecibel(a_w))
}

/// Computes the specific attenuation from rain (P.838-3).
///
/// Args:
///     rain_rate: Rainfall rate in mm/h.
///     frequency: Frequency.
///     elevation: Elevation angle.
///     polarisation_tilt: Polarisation tilt angle (default 45° for circular).
///
/// Returns:
///     Specific rain attenuation in dB/km.
#[pyfunction]
#[pyo3(signature = (rain_rate, frequency, elevation, polarisation_tilt=None))]
pub fn rain_specific_attenuation(
    rain_rate: f64,
    frequency: PyFrequency,
    elevation: PyAngle,
    polarisation_tilt: Option<PyAngle>,
) -> f64 {
    let tau = polarisation_tilt
        .map(|a| a.0)
        .unwrap_or(Angle::degrees(45.0));
    lox_itur::p838::rain_specific_attenuation(rain_rate, frequency.0, elevation.0, tau)
}

/// Registers the pure-formula ITU-R functions with the Python module.
///
/// Grid-based recommendations are exposed as methods on `ItuProvider`.
pub fn register_itur_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(gaseous_attenuation_slant_path, m)?)?;
    m.add_function(wrap_pyfunction!(rain_specific_attenuation, m)?)?;
    Ok(())
}
