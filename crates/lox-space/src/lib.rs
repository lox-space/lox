/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::f64::consts::PI;

use lox_bodies::python::PyOrigin;
use lox_ephem::python::PySpk;
use lox_frames::python::PyFrame;
use lox_orbits::python::{
    PyElevationMask, PyEnsemble, PyEvent, PyGroundLocation, PyGroundPropagator, PyKeplerian,
    PyObservables, PyPass, PySgp4, PyState, PyTrajectory, PyVallado, PyWindow, find_events,
    find_windows, visibility, visibility_all,
};
use lox_units::ASTRONOMICAL_UNIT;
use lox_units::python::{PyAngle, PyDistance, PyFrequency, PyVelocity};
use pyo3::prelude::*;

use lox_math::python::PySeries;
use lox_time::python::deltas::PyTimeDelta;
use lox_time::python::time::PyTime;
use lox_time::python::time_scales::PyTimeScale;
use lox_time::python::ut1::PyUt1Provider;
use lox_time::python::utc::PyUtc;

#[pymodule]
fn lox_space(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // lox-units
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

    m.add_function(wrap_pyfunction!(find_events, m)?)?;
    m.add_function(wrap_pyfunction!(find_windows, m)?)?;
    m.add_function(wrap_pyfunction!(visibility, m)?)?;
    m.add_function(wrap_pyfunction!(visibility_all, m)?)?;
    m.add_class::<PyElevationMask>()?;
    m.add_class::<PyEnsemble>()?;
    m.add_class::<PyEvent>()?;
    m.add_class::<PyFrame>()?;
    m.add_class::<PyGroundLocation>()?;
    m.add_class::<PyGroundPropagator>()?;
    m.add_class::<PyKeplerian>()?;
    m.add_class::<PyObservables>()?;
    m.add_class::<PyOrigin>()?;
    m.add_class::<PyPass>()?;
    m.add_class::<PySeries>()?;
    m.add_class::<PySgp4>()?;
    m.add_class::<PySpk>()?;
    m.add_class::<PyState>()?;
    m.add_class::<PyTime>()?;
    m.add_class::<PyTimeDelta>()?;
    m.add_class::<PyTimeScale>()?;
    m.add_class::<PyTrajectory>()?;
    m.add_class::<PyUt1Provider>()?;
    m.add_class::<PyUtc>()?;
    m.add_class::<PyVallado>()?;
    m.add_class::<PyWindow>()?;
    Ok(())
}
