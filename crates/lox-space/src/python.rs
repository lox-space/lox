// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::f64::consts::PI;

use crate::analysis::python::{
    PyElevationMask, PyGroundStation, PyObservables, PyPass, PySpacecraft, PyVisibilityAnalysis,
    PyVisibilityResults,
};
use crate::bodies::python::PyOrigin;
use crate::comms::python::{
    PyChannel, PyCommunicationSystem, PyComplexAntenna, PyComplexReceiver, PyDecibel,
    PyDipolePattern, PyEnvironmentalLosses, PyGaussianPattern, PyLinkStats, PyModulation,
    PyParabolicPattern, PySimpleAntenna, PySimpleReceiver, PyTransmitter, freq_overlap, fspl,
};
use crate::earth::python::ut1::{EopParserError, EopProviderError, PyEopProvider};
use crate::ephem::python::PySpk;
use crate::frames::python::PyFrame;
use crate::math::python::PySeries;
use crate::orbits::python::{
    PyCartesian, PyEvent, PyGroundLocation, PyGroundPropagator, PyInterval, PyJ2Propagator,
    PyKeplerian, PySgp4, PyTrajectory, PyVallado, find_events, find_windows,
    py_complement_intervals, py_intersect_intervals, py_union_intervals,
};
use crate::time::python::{
    deltas::{NonFiniteTimeDeltaError, PyTimeDelta},
    time::PyTime,
    time_scales::PyTimeScale,
    utc::PyUtc,
};
use crate::units::{
    ASTRONOMICAL_UNIT,
    python::{
        PyAngle, PyAngularRate, PyDataRate, PyDistance, PyFrequency, PyGravitationalParameter,
        PyPower, PyTemperature, PyVelocity,
    },
};

use pyo3::prelude::*;

// LCOV_EXCL_START - PyO3 module initialization cannot be directly tested.
// See: https://github.com/rust-lang/rust/issues/84605
#[pymodule]
fn lox_space(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // bodies
    m.add_class::<PyOrigin>()?;

    // comms
    m.add_class::<PyDecibel>()?;
    m.add("dB", PyDecibel::new(1.0))?;
    m.add_class::<PyModulation>()?;
    m.add_class::<PyParabolicPattern>()?;
    m.add_class::<PyGaussianPattern>()?;
    m.add_class::<PyDipolePattern>()?;
    m.add_class::<PySimpleAntenna>()?;
    m.add_class::<PyComplexAntenna>()?;
    m.add_class::<PyTransmitter>()?;
    m.add_class::<PySimpleReceiver>()?;
    m.add_class::<PyComplexReceiver>()?;
    m.add_class::<PyChannel>()?;
    m.add_class::<PyEnvironmentalLosses>()?;
    m.add_class::<PyCommunicationSystem>()?;
    m.add_class::<PyLinkStats>()?;
    m.add_function(wrap_pyfunction!(fspl, m)?)?;
    m.add_function(wrap_pyfunction!(freq_overlap, m)?)?;

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
    m.add_class::<PySpacecraft>()?;
    m.add_class::<PyVisibilityAnalysis>()?;
    m.add_class::<PyVisibilityResults>()?;

    // orbits
    m.add_class::<PyCartesian>()?;
    m.add_class::<PyEvent>()?;
    m.add_class::<PyGroundLocation>()?;
    m.add_class::<PyGroundPropagator>()?;
    m.add_class::<PyInterval>()?;
    m.add_class::<PyJ2Propagator>()?;
    m.add_class::<PyKeplerian>()?;
    m.add_class::<PySgp4>()?;
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
    m.add_class::<PyDataRate>()?;
    m.add("bps", PyDataRate::new(1.0))?;
    m.add("kbps", PyDataRate::new(1e3))?;
    m.add("Mbps", PyDataRate::new(1e6))?;
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
    m.add_class::<PyTemperature>()?;
    m.add("K", PyTemperature::new(1.0))?;
    m.add_class::<PyVelocity>()?;
    m.add("m_per_s", PyVelocity::new(1.0))?;
    m.add("km_per_s", PyVelocity::new(1e3))?;

    Ok(())
}
// LCOV_EXCL_STOP
