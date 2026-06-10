// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::f64::consts::PI;

use crate::analysis::python::{
    PyAccessResults, PyAccessWindow, PyAoi, PyElevationMask, PyEnsemble, PyGroundStation,
    PyLookSide, PyObservables, PyOpticalAccessAnalysis, PyOpticalPayload, PyPass, PyPassDirection,
    PyPowerBudgetAnalysis, PyPowerBudgetResults, PySarAccessAnalysis, PySarPayload, PyScenario,
    PySpacecraft, PyVisibilityAnalysis, PyVisibilityResults,
};
use crate::bodies::python::PyOrigin;
use crate::comms::python::{
    PyAmplifierTransmitter, PyAntennaFrame, PyCascadeReceiver, PyChannel, PyCommunicationSystem,
    PyConstantAntenna, PyDecibel, PyDipolePattern, PyEirpTransmitter, PyGaussianPattern,
    PyGtReceiver, PyInterferenceStats, PyLinkStats, PyModulatedLinkStats, PyModulation,
    PyNoiseStage, PyNoiseTempReceiver, PyParabolicPattern, PyPatternedAntenna, freq_overlap, fspl,
    pfd_mask, power_flux_density, slant_range,
};
use crate::constellations::python::{PyConstellation, PyConstellationSatellite};
use crate::earth::python::ut1::{EopParserError, EopProviderError, PyEopProvider};
use crate::ephem::python::PySpk;
use crate::frames::python::PyFrame;
use crate::itur::python::PyEnvironmentalLosses;
use crate::itur::python::PyItuProvider;
use crate::itur::python::register_itur_functions;
use crate::math::python::PySeries;
use crate::orbits::python::{
    PyCartesian, PyEvent, PyGroundLocation, PyGroundPropagator, PyInterval, PyJ2Propagator,
    PyJ4Propagator, PyKeplerian, PyModifiedEquinoctial, PyNumericalPropagator, PySgp4, PyTle,
    PyTrajectory, PyVallado, find_events, find_windows, py_complement_intervals,
    py_intersect_intervals, py_union_intervals,
};
use crate::time::python::{
    deltas::{NonFiniteTimeDeltaError, PyTimeDelta},
    time::PyTime,
    time_scales::PyTimeScale,
    time_series::PyTimeSeries,
    utc::PyUtc,
};
use crate::units::{
    ASTRONOMICAL_UNIT,
    python::{
        PyAngle, PyAngularRate, PyDistance, PyFrequency, PyGravitationalParameter, PyPower,
        PyPressure, PyTemperature, PyVelocity,
    },
};

use pyo3::prelude::*;

/// Register all lox-space Python types, functions, and constants into the given module.
///
/// This can be used to embed all Lox types into a downstream Python extension module.
pub fn register_types(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // bodies
    m.add_class::<PyOrigin>()?;

    // comms
    m.add_class::<PyDecibel>()?;
    m.add("dB", PyDecibel::new(1.0))?;
    m.add_class::<PyModulation>()?;
    m.add_class::<PyParabolicPattern>()?;
    m.add_class::<PyGaussianPattern>()?;
    m.add_class::<PyDipolePattern>()?;
    m.add_class::<PyAntennaFrame>()?;
    m.add_class::<PyConstantAntenna>()?;
    m.add_class::<PyPatternedAntenna>()?;
    m.add_class::<PyAmplifierTransmitter>()?;
    m.add_class::<PyEirpTransmitter>()?;
    m.add_class::<PyNoiseTempReceiver>()?;
    m.add_class::<PyCascadeReceiver>()?;
    m.add_class::<PyGtReceiver>()?;
    m.add_class::<PyNoiseStage>()?;
    m.add_class::<PyChannel>()?;
    m.add_class::<PyItuProvider>()?;
    m.add_class::<PyEnvironmentalLosses>()?;
    m.add_class::<PyCommunicationSystem>()?;
    m.add_class::<PyLinkStats>()?;
    m.add_class::<PyInterferenceStats>()?;
    m.add_class::<PyModulatedLinkStats>()?;
    m.add_function(wrap_pyfunction!(fspl, m)?)?;
    m.add_function(wrap_pyfunction!(freq_overlap, m)?)?;
    m.add_function(wrap_pyfunction!(power_flux_density, m)?)?;
    m.add_function(wrap_pyfunction!(pfd_mask, m)?)?;
    m.add_function(wrap_pyfunction!(slant_range, m)?)?;

    // itur
    register_itur_functions(m)?;

    // earth
    m.add_class::<PyEopProvider>()?;
    m.add("EopParserError", m.py().get_type::<EopParserError>())?;
    m.add("EopProviderError", m.py().get_type::<EopProviderError>())?;

    // ephem
    m.add_class::<PySpk>()?;

    // frames
    m.add_class::<PyFrame>()?;

    // math
    m.add_class::<PySeries>()?;

    // analysis
    m.add_class::<PyElevationMask>()?;
    m.add_class::<PyGroundStation>()?;
    m.add_class::<PyObservables>()?;
    m.add_class::<PyPass>()?;
    m.add_class::<PyScenario>()?;
    m.add_class::<PySpacecraft>()?;
    m.add_class::<PyEnsemble>()?;
    m.add_class::<PyVisibilityAnalysis>()?;
    m.add_class::<PyVisibilityResults>()?;
    m.add_class::<PyPowerBudgetAnalysis>()?;
    m.add_class::<PyPowerBudgetResults>()?;
    m.add_class::<PyAoi>()?;
    m.add_class::<PyPassDirection>()?;
    m.add_class::<PyAccessWindow>()?;
    m.add_class::<PyAccessResults>()?;
    m.add_class::<PyOpticalPayload>()?;
    m.add_class::<PyOpticalAccessAnalysis>()?;
    m.add_class::<PyLookSide>()?;
    m.add_class::<PySarPayload>()?;
    m.add_class::<PySarAccessAnalysis>()?;

    // constellations
    m.add_class::<PyConstellation>()?;
    m.add_class::<PyConstellationSatellite>()?;

    // orbits
    m.add_class::<PyCartesian>()?;
    m.add_class::<PyEvent>()?;
    m.add_class::<PyGroundLocation>()?;
    m.add_class::<PyGroundPropagator>()?;
    m.add_class::<PyInterval>()?;
    m.add_class::<PyJ2Propagator>()?;
    m.add_class::<PyJ4Propagator>()?;
    m.add_class::<PyNumericalPropagator>()?;
    m.add_class::<PyKeplerian>()?;
    m.add_class::<PyModifiedEquinoctial>()?;
    m.add_class::<PySgp4>()?;
    m.add_class::<PyTle>()?;
    m.add_class::<PyTrajectory>()?;
    m.add_class::<PyVallado>()?;
    m.add_function(wrap_pyfunction!(find_events, m)?)?;
    m.add_function(wrap_pyfunction!(find_windows, m)?)?;
    m.add_function(wrap_pyfunction!(py_intersect_intervals, m)?)?;
    m.add_function(wrap_pyfunction!(py_union_intervals, m)?)?;
    m.add_function(wrap_pyfunction!(py_complement_intervals, m)?)?;

    // time
    m.add_class::<PyTime>()?;
    m.add_class::<PyTimeDelta>()?;
    m.add("seconds", PyTimeDelta::new(1.0))?;
    m.add("minutes", PyTimeDelta::new(60.0))?;
    m.add("hours", PyTimeDelta::new(3600.0))?;
    m.add("days", PyTimeDelta::new(86400.0))?;
    m.add_class::<PyTimeScale>()?;
    m.add_class::<PyTimeSeries>()?;
    m.add_class::<PyUtc>()?;
    m.add(
        "NonFiniteTimeDeltaError",
        m.py().get_type::<NonFiniteTimeDeltaError>(),
    )?;

    // units
    m.add_class::<PyAngle>()?;
    m.add("rad", PyAngle::new(1.0))?;
    m.add("deg", PyAngle::new(PI / 180.0))?;
    m.add_class::<PyAngularRate>()?;
    m.add("rad_per_s", PyAngularRate::new(1.0))?;
    m.add("deg_per_s", PyAngularRate::new(PI / 180.0))?;

    m.add_class::<PyDistance>()?;
    m.add("m", PyDistance::new(1.0))?;
    m.add("km", PyDistance::new(1e3))?;
    m.add("au", PyDistance::new(ASTRONOMICAL_UNIT))?;
    m.add_class::<PyFrequency>()?;
    m.add("Hz", PyFrequency::new(1.0))?;
    m.add("kHz", PyFrequency::new(1e3))?;
    m.add("MHz", PyFrequency::new(1e6))?;
    m.add("GHz", PyFrequency::new(1e9))?;
    m.add("THz", PyFrequency::new(1e12))?;
    m.add_class::<PyGravitationalParameter>()?;
    m.add_class::<PyPower>()?;
    m.add("W", PyPower::new(1.0))?;
    m.add("kW", PyPower::new(1e3))?;
    m.add_class::<PyPressure>()?;
    m.add("Pa", PyPressure::new(1.0))?;
    m.add("hPa", PyPressure::new(100.0))?;
    m.add_class::<PyTemperature>()?;
    m.add("K", PyTemperature::new(1.0))?;
    m.add_class::<PyVelocity>()?;
    m.add("m_per_s", PyVelocity::new(1.0))?;
    m.add("km_per_s", PyVelocity::new(1e3))?;

    Ok(())
}

#[pymodule]
fn lox_space(m: &Bound<'_, PyModule>) -> PyResult<()> {
    register_types(m)
}
