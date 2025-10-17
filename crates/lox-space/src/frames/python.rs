/*
 * Copyright (c) 2025. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::frames::{
    dynamic::{DynFrame, UnknownFrameError},
    traits::ReferenceFrame,
};
use lox_earth::itrf::DynTransformEopError;
use lox_frames::dynamic::DynTransformError;
use pyo3::{
    PyErr, PyResult, create_exception,
    exceptions::{PyException, PyValueError},
    pyclass, pymethods,
};

pub struct PyUnknownFrameError(pub UnknownFrameError);

impl From<PyUnknownFrameError> for PyErr {
    fn from(err: PyUnknownFrameError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

create_exception!(lox_space, FrameTransformationError, PyException);

pub struct PyDynTransformError(pub DynTransformError);

impl From<PyDynTransformError> for PyErr {
    fn from(err: PyDynTransformError) -> Self {
        FrameTransformationError::new_err(err.0.to_string())
    }
}

pub struct PyDynTransformEopError(pub DynTransformEopError);

impl From<PyDynTransformEopError> for PyErr {
    fn from(err: PyDynTransformEopError) -> Self {
        FrameTransformationError::new_err(err.0.to_string())
    }
}

#[pyclass(name = "Frame", module = "lox_space", frozen)]
#[pyo3(eq)]
#[derive(Debug, Clone, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct PyFrame(pub DynFrame);

#[pymethods]
impl PyFrame {
    #[new]
    fn new(abbreviation: &str) -> PyResult<Self> {
        Ok(Self(abbreviation.parse().map_err(PyUnknownFrameError)?))
    }

    fn __getnewargs__(&self) -> (String,) {
        (self.abbreviation(),)
    }

    fn name(&self) -> String {
        self.0.name()
    }

    fn abbreviation(&self) -> String {
        self.0.abbreviation()
    }
}
