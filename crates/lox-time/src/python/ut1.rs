/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::ut1::{DeltaUt1Tai, DeltaUt1TaiError, ExtrapolatedDeltaUt1Tai};
use crate::utc::leap_seconds::BuiltinLeapSeconds;
use pyo3::exceptions::PyValueError;
use pyo3::{pyclass, pymethods, PyErr, PyResult};

impl From<ExtrapolatedDeltaUt1Tai> for PyErr {
    fn from(value: ExtrapolatedDeltaUt1Tai) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

impl From<DeltaUt1TaiError> for PyErr {
    fn from(value: DeltaUt1TaiError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

#[pyclass(name = "UT1Provider", module = "lox_space")]
#[derive(Clone, Debug, PartialEq)]
pub struct PyUt1Provider(pub DeltaUt1Tai);

#[pymethods]
impl PyUt1Provider {
    #[new]
    fn new(path: &str) -> PyResult<PyUt1Provider> {
        let provider = DeltaUt1Tai::new(path, &BuiltinLeapSeconds)?;
        Ok(PyUt1Provider(provider))
    }
}
