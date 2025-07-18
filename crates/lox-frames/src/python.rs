/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use pyo3::{PyErr, PyResult, exceptions::PyValueError, pyclass, pymethods};
use crate::{UnknownFrameError, iau::IauFrameTransformationError, DynFrame, ReferenceFrame};

impl From<UnknownFrameError> for PyErr {
    fn from(err: UnknownFrameError) -> Self {
        PyValueError::new_err(err.to_string())
    }
}

impl From<IauFrameTransformationError> for PyErr {
    fn from(err: IauFrameTransformationError) -> Self {
        // FIXME: wrong error type
        PyValueError::new_err(err.to_string())
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
        Ok(Self(abbreviation.parse()?))
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