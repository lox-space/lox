/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_bodies::python::PyOrigin;
use lox_ephem::python::PySpk;
use lox_orbits::python::{
    PyElevationMask, PyEnsemble, PyEvent, PyFrame, PyGroundLocation, PyGroundPropagator,
    PyKeplerian, PyObservables, PyPass, PySgp4, PyState, PyTrajectory, PyVallado, PyWindow,
    find_events, find_windows, visibility, visibility_all,
};
use pyo3::prelude::*;

use lox_math::python::PySeries;
use lox_time::python::deltas::PyTimeDelta;
use lox_time::python::time::PyTime;
use lox_time::python::time_scales::PyTimeScale;
use lox_time::python::ut1::PyUt1Provider;
use lox_time::python::utc::PyUtc;

#[pymodule]
fn lox_space(m: &Bound<'_, PyModule>) -> PyResult<()> {
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
