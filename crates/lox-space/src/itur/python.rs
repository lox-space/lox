// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Python bindings for ITU-R atmospheric propagation models.

use pyo3::prelude::*;

use crate::comms::python::{PyDecibel, PyEnvironmentalLosses};
use crate::units::python::{PyAngle, PyDistance, PyFrequency, PyPressure, PyTemperature};

use lox_core::units::Angle;

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
///     polarisation_tilt: Polarisation tilt angle (default 45° for circular).
///
/// Returns:
///     EnvironmentalLosses with rain, gaseous, cloud, scintillation, and
///     depolarization fields populated.
#[pyfunction]
#[pyo3(signature = (lat, lon, frequency, elevation, probability, diameter, polarisation_tilt=None))]
pub fn atmospheric_attenuation_slant_path(
    lat: PyAngle,
    lon: PyAngle,
    frequency: PyFrequency,
    elevation: PyAngle,
    probability: f64,
    diameter: PyDistance,
    polarisation_tilt: Option<PyAngle>,
) -> PyEnvironmentalLosses {
    let tau = polarisation_tilt
        .map(|a| a.0)
        .unwrap_or(Angle::degrees(45.0));
    PyEnvironmentalLosses(lox_itur::atmospheric_attenuation_slant_path(
        lat.0,
        lon.0,
        frequency.0,
        elevation.0,
        probability,
        diameter.0,
        tau,
    ))
}

/// Computes rain attenuation exceeded for a given probability (P.618-13).
///
/// Args:
///     lat: Latitude.
///     lon: Longitude.
///     frequency: Frequency.
///     elevation: Elevation angle.
///     probability: Exceedance probability (% of average year).
///     polarisation_tilt: Polarisation tilt angle (default 45° for circular).
///
/// Returns:
///     Rain attenuation.
#[pyfunction]
#[pyo3(signature = (lat, lon, frequency, elevation, probability, polarisation_tilt=None))]
pub fn rain_attenuation(
    lat: PyAngle,
    lon: PyAngle,
    frequency: PyFrequency,
    elevation: PyAngle,
    probability: f64,
    polarisation_tilt: Option<PyAngle>,
) -> PyDecibel {
    let tau = polarisation_tilt
        .map(|a| a.0)
        .unwrap_or(Angle::degrees(45.0));
    PyDecibel(lox_itur::p618::rain_attenuation(
        lat.0,
        lon.0,
        frequency.0,
        elevation.0,
        probability,
        tau,
        None,
    ))
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

/// Computes cloud attenuation on a slant path (P.840-8).
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
#[pyfunction]
pub fn cloud_attenuation(
    lat: PyAngle,
    lon: PyAngle,
    elevation: PyAngle,
    frequency: PyFrequency,
    probability: f64,
) -> PyDecibel {
    PyDecibel(lox_core::units::Decibel::new(
        lox_itur::p840::cloud_attenuation(lat.0, lon.0, elevation.0, frequency.0, probability),
    ))
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
#[pyfunction]
#[pyo3(signature = (frequency, elevation, probability, diameter, eta=0.5, lat=None, lon=None))]
pub fn scintillation_attenuation(
    frequency: PyFrequency,
    elevation: PyAngle,
    probability: f64,
    diameter: PyDistance,
    eta: f64,
    lat: Option<PyAngle>,
    lon: Option<PyAngle>,
) -> PyDecibel {
    let lat_angle = lat.map(|a| a.0).unwrap_or(Angle::degrees(0.0));
    let lon_angle = lon.map(|a| a.0).unwrap_or(Angle::degrees(0.0));
    PyDecibel(lox_itur::p618::scintillation_attenuation(
        frequency.0,
        elevation.0,
        probability,
        diameter.0,
        eta,
        None,
        lat_angle,
        lon_angle,
    ))
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

/// Returns the topographic altitude at the given location (P.1511-2).
///
/// Args:
///     lat: Latitude.
///     lon: Longitude.
///
/// Returns:
///     Altitude above mean sea level.
#[pyfunction]
pub fn topographic_altitude(lat: PyAngle, lon: PyAngle) -> PyDistance {
    PyDistance(lox_itur::p1511::topographic_altitude(lat.0, lon.0))
}

/// Returns the annual mean surface temperature at the given location (P.1510-1).
///
/// Args:
///     lat: Latitude.
///     lon: Longitude.
///
/// Returns:
///     Temperature.
#[pyfunction]
pub fn surface_mean_temperature(lat: PyAngle, lon: PyAngle) -> PyTemperature {
    PyTemperature(lox_itur::p1510::surface_mean_temperature(lat.0, lon.0))
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
#[pyfunction]
pub fn rainfall_rate(lat: PyAngle, lon: PyAngle, probability: f64) -> f64 {
    lox_itur::p837::rainfall_rate(lat.0, lon.0, probability)
}

/// Returns the mean annual rain height at the given location (P.839-4).
///
/// Args:
///     lat: Latitude.
///     lon: Longitude.
///
/// Returns:
///     Rain height.
#[pyfunction]
pub fn rain_height(lat: PyAngle, lon: PyAngle) -> PyDistance {
    PyDistance(lox_itur::p839::rain_height(lat.0, lon.0))
}

/// Registers all ITU-R propagation functions with the Python module.
pub fn register_itur_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(atmospheric_attenuation_slant_path, m)?)?;
    m.add_function(wrap_pyfunction!(rain_attenuation, m)?)?;
    m.add_function(wrap_pyfunction!(gaseous_attenuation_slant_path, m)?)?;
    m.add_function(wrap_pyfunction!(cloud_attenuation, m)?)?;
    m.add_function(wrap_pyfunction!(scintillation_attenuation, m)?)?;
    m.add_function(wrap_pyfunction!(rain_specific_attenuation, m)?)?;
    m.add_function(wrap_pyfunction!(topographic_altitude, m)?)?;
    m.add_function(wrap_pyfunction!(surface_mean_temperature, m)?)?;
    m.add_function(wrap_pyfunction!(rainfall_rate, m)?)?;
    m.add_function(wrap_pyfunction!(rain_height, m)?)?;
    Ok(())
}
