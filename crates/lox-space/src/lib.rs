/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_bodies::python::{PyBarycenter, PyMinorBody, PyPlanet, PySatellite, PySun};
use pyo3::prelude::*;

use lox_time::python::deltas::PyTimeDelta;
use lox_time::python::time::PyTime;
use lox_time::python::ut1::PyUt1Provider;
use lox_time::python::utc::PyUtc;

#[pymodule]
fn lox_space(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySun>()?;
    m.add_class::<PyBarycenter>()?;
    m.add_class::<PyPlanet>()?;
    m.add_class::<PySatellite>()?;
    m.add_class::<PyMinorBody>()?;
    m.add_class::<PyTime>()?;
    m.add_class::<PyTimeDelta>()?;
    m.add_class::<PyUtc>()?;
    m.add_class::<PyUt1Provider>()?;
    Ok(())
}
