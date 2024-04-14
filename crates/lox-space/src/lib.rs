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

use lox_bodies::errors::LoxBodiesError;
use lox_time::errors::LoxTimeError;

use crate::bodies::{PyBarycenter, PyMinorBody, PyPlanet, PySatellite, PySun};
use crate::coords::{PyCartesian, PyKeplerian};
use crate::time::{PyTime, PyTimeScale};

mod bodies;
mod coords;
mod frames;
mod time;

#[derive(Error, Debug)]
pub enum LoxPyError {
    #[error("unknown time scale `{0}`")]
    InvalidTimeScale(String),
    #[error("unknown body `{0}`")]
    InvalidBody(String),
    #[error("unknown frame `{0}`")]
    InvalidFrame(String),
    #[error(transparent)]
    LoxBodiesError(#[from] LoxBodiesError),
    #[error(transparent)]
    LoxTimeError(#[from] LoxTimeError),
    #[error(transparent)]
    PyError(#[from] PyErr),
}

impl From<LoxPyError> for PyErr {
    fn from(value: LoxPyError) -> Self {
        match value {
            LoxPyError::InvalidTimeScale(_)
            | LoxPyError::InvalidFrame(_)
            | LoxPyError::InvalidBody(_) => PyValueError::new_err(value.to_string()),
            LoxPyError::LoxBodiesError(value) => PyValueError::new_err(value.to_string()),
            LoxPyError::LoxTimeError(value) => PyValueError::new_err(value.to_string()),
            LoxPyError::PyError(value) => value,
        }
    }
}

#[pymodule]
fn lox_space(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTimeScale>()?;
    m.add_class::<PyTime>()?;
    m.add_class::<PySun>()?;
    m.add_class::<PyBarycenter>()?;
    m.add_class::<PyPlanet>()?;
    m.add_class::<PySatellite>()?;
    m.add_class::<PyMinorBody>()?;
    m.add_class::<PyCartesian>()?;
    m.add_class::<PyKeplerian>()?;
    Ok(())
}
