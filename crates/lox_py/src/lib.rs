/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use thiserror::Error;

use crate::bodies::{PyBarycenter, PyMinorBody, PyPlanet, PySatellite, PySun};
use crate::time::{PyEpoch, PyTimeScale};
use crate::twobody::PyCartesian;
use lox_core::errors::LoxError;

mod bodies;
mod time;
mod twobody;

#[derive(Error, Debug)]
pub enum LoxPyError {
    #[error("invalid time scale `{0}`")]
    InvalidTimeScale(String),
    #[error("unknown body `{0}`")]
    InvalidBody(String),
    #[error(transparent)]
    LoxError(#[from] LoxError),
    #[error(transparent)]
    PyError(#[from] PyErr),
}

impl From<LoxPyError> for PyErr {
    fn from(value: LoxPyError) -> Self {
        match value {
            LoxPyError::InvalidTimeScale(_) => PyValueError::new_err(value.to_string()),
            LoxPyError::InvalidBody(_) => PyValueError::new_err(value.to_string()),
            LoxPyError::LoxError(value) => PyValueError::new_err(value.to_string()),
            LoxPyError::PyError(value) => value,
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn lox_space(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTimeScale>()?;
    m.add_class::<PyEpoch>()?;
    m.add_class::<PySun>()?;
    m.add_class::<PyBarycenter>()?;
    m.add_class::<PyPlanet>()?;
    m.add_class::<PySatellite>()?;
    m.add_class::<PyMinorBody>()?;
    m.add_class::<PyCartesian>()?;
    Ok(())
}
