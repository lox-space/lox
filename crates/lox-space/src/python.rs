// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::f64::consts::PI;

use crate::bodies::python::PyOrigin;
use crate::earth::python::ut1::{EopParserError, EopProviderError, PyEopProvider};
use crate::ephem::python::PySpk;
use crate::frames::python::PyFrame;
use crate::math::python::PySeries;
use crate::orbits::python::{
    PyElevationMask, PyEnsemble, PyEvent, PyGroundLocation, PyGroundPropagator, PyKeplerian,
    PyObservables, PyPass, PySgp4, PyState, PyTrajectory, PyVallado, PyWindow, find_events,
    find_windows, visibility, visibility_all,
};
use crate::time::python::{
    deltas::{NonFiniteTimeDeltaError, PyTimeDelta},
    time::PyTime,
    time_scales::PyTimeScale,
    utc::PyUtc,
};
use crate::units::{
    ASTRONOMICAL_UNIT,
    python::{PyAngle, PyDistance, PyFrequency, PyVelocity},
};

use pyo3::prelude::*;

// LCOV_EXCL_START - PyO3 module initialization cannot be directly tested.
// See: https://github.com/rust-lang/rust/issues/84605
#[pymodule]
fn lox_space(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // bodies
    m.add_class::<PyOrigin>()?;

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

    // orbits
    m.add_function(wrap_pyfunction!(find_events, m)?)?;
    m.add_function(wrap_pyfunction!(find_windows, m)?)?;
    m.add_function(wrap_pyfunction!(visibility, m)?)?;
    m.add_function(wrap_pyfunction!(visibility_all, m)?)?;
    m.add_class::<PyElevationMask>()?;
    m.add_class::<PyEnsemble>()?;
    m.add_class::<PyEvent>()?;
    m.add_class::<PyGroundLocation>()?;
    m.add_class::<PyGroundPropagator>()?;
    m.add_class::<PyKeplerian>()?;
    m.add_class::<PyObservables>()?;
    m.add_class::<PyPass>()?;
    m.add_class::<PySgp4>()?;
    m.add_class::<PyState>()?;
    m.add_class::<PyTrajectory>()?;
    m.add_class::<PyVallado>()?;
    m.add_class::<PyWindow>()?;

    // time
    m.add_class::<PyTime>()?;
    m.add_class::<PyTimeDelta>()?;
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
    m.add_class::<PyDistance>()?;
    m.add("m", PyDistance::new(1.0))?;
    m.add("km", PyDistance::new(1e3))?;
    m.add("au", PyDistance::new(ASTRONOMICAL_UNIT))?;
    m.add_class::<PyFrequency>()?;
    m.add("hz", PyFrequency::new(1.0))?;
    m.add("khz", PyFrequency::new(1e3))?;
    m.add("mhz", PyFrequency::new(1e6))?;
    m.add("ghz", PyFrequency::new(1e9))?;
    m.add("thz", PyFrequency::new(1e12))?;
    m.add_class::<PyVelocity>()?;
    m.add("ms", PyVelocity::new(1.0))?;
    m.add("kms", PyVelocity::new(1e3))?;

    Ok(())
}
// LCOV_EXCL_STOP
