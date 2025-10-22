// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::time::offsets::MissingEopProviderError;
use crate::time::time_scales::{DynTimeScale, TimeScale, UnknownTimeScaleError};
use pyo3::{
    Bound, PyAny, PyErr, PyResult, exceptions::PyValueError, pyclass, pymethods,
    types::PyAnyMethods,
};

pub struct PyUnknownTimeScaleError(pub UnknownTimeScaleError);

impl From<PyUnknownTimeScaleError> for PyErr {
    fn from(err: PyUnknownTimeScaleError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

pub struct PyMissingEopProviderError(pub MissingEopProviderError);

impl From<PyMissingEopProviderError> for PyErr {
    fn from(err: PyMissingEopProviderError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
#[pyclass(name = "TimeScale", module = "lox_space", frozen, eq)]
pub struct PyTimeScale(pub DynTimeScale);

#[pymethods]
impl PyTimeScale {
    #[new]
    pub fn new(abbreviation: &str) -> PyResult<Self> {
        Ok(PyTimeScale(
            abbreviation.parse().map_err(PyUnknownTimeScaleError)?,
        ))
    }
    fn __getnewargs__(&self) -> (String,) {
        (self.abbreviation(),)
    }

    pub fn __repr__(&self) -> String {
        format!("TimeScale(\"{}\")", self.0)
    }

    pub fn __str__(&self) -> String {
        format!("{}", self.0)
    }

    pub fn abbreviation(&self) -> String {
        self.0.abbreviation().to_owned()
    }

    pub fn name(&self) -> String {
        self.0.name().to_owned()
    }
}

impl TryFrom<&Bound<'_, PyAny>> for PyTimeScale {
    type Error = PyErr;

    fn try_from(value: &Bound<'_, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(name) = value.extract::<&str>() {
            return Ok(PyTimeScale(name.parse().map_err(PyUnknownTimeScaleError)?));
        } else if let Ok(scale) = value.extract::<PyTimeScale>() {
            return Ok(scale);
        }
        Err(PyValueError::new_err(
            "'scale' argument must either a string or a 'TimeScale' instance",
        ))
    }
}
