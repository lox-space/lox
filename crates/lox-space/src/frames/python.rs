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
    transformations::iau::IauFrameTransformationError,
};
use pyo3::{PyErr, PyResult, exceptions::PyValueError, pyclass, pymethods};

pub struct PyUnknownFrameError(pub UnknownFrameError);

impl From<PyUnknownFrameError> for PyErr {
    fn from(err: PyUnknownFrameError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

pub struct PyIauFrameTransformationError(pub IauFrameTransformationError);

impl From<PyIauFrameTransformationError> for PyErr {
    fn from(err: PyIauFrameTransformationError) -> Self {
        // FIXME: wrong error type
        PyValueError::new_err(err.0.to_string())
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
