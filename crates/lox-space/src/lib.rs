/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_bodies::python::{PyBarycenter, PyMinorBody, PyPlanet, PySatellite, PySun};
use lox_ephem::python::PySpk;
use lox_orbits::python::{
    elevation, find_events, find_windows, visibility, PyEvent, PyFrame, PyGroundLocation,
    PyGroundPropagator, PyKeplerian, PyObservables, PySgp4, PyState, PyTopocentric, PyTrajectory,
    PyVallado, PyWindow,
};
use pyo3::prelude::*;

use lox_math::python::PySeries;
use lox_time::python::deltas::PyTimeDelta;
use lox_time::python::time::PyTime;
use lox_time::python::ut1::PyUt1Provider;
use lox_time::python::utc::PyUtc;

#[pymodule]
fn lox_space(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(find_events, m)?)?;
    m.add_function(wrap_pyfunction!(find_windows, m)?)?;
    m.add_function(wrap_pyfunction!(visibility, m)?)?;
    m.add_function(wrap_pyfunction!(elevation, m)?)?;
    m.add_class::<PySun>()?;
    m.add_class::<PyBarycenter>()?;
    m.add_class::<PyPlanet>()?;
    m.add_class::<PySatellite>()?;
    m.add_class::<PyMinorBody>()?;
    m.add_class::<PyTime>()?;
    m.add_class::<PyTimeDelta>()?;
    m.add_class::<PyUtc>()?;
    m.add_class::<PyUt1Provider>()?;
    m.add_class::<PyFrame>()?;
    m.add_class::<PyKeplerian>()?;
    m.add_class::<PyState>()?;
    m.add_class::<PyTrajectory>()?;
    m.add_class::<PyVallado>()?;
    m.add_class::<PySgp4>()?;
    m.add_class::<PyGroundLocation>()?;
    m.add_class::<PyGroundPropagator>()?;
    m.add_class::<PyEvent>()?;
    m.add_class::<PyWindow>()?;
    m.add_class::<PyTopocentric>()?;
    m.add_class::<PySeries>()?;
    m.add_class::<PyObservables>()?;
    m.add_class::<PySpk>()?;
    Ok(())
}
