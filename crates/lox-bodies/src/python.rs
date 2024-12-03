/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use std::str::FromStr;

use crate::dynamic::{DynOrigin, UnknownOriginId, UnknownOriginName};
use crate::Origin;

impl From<UnknownOriginId> for PyErr {
    fn from(err: UnknownOriginId) -> Self {
        PyValueError::new_err(err.to_string())
    }
}

impl From<UnknownOriginName> for PyErr {
    fn from(err: UnknownOriginName) -> Self {
        PyValueError::new_err(err.to_string())
    }
}

#[pyclass(name = "Origin", module = "lox_space", frozen, eq)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PyOrigin(pub DynOrigin);

#[pymethods]
impl PyOrigin {
    #[new]
    fn new(origin: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(origin) = origin.extract::<i32>() {
            return Ok(Self(origin.try_into()?));
        }
        if let Ok(origin) = origin.extract::<&str>() {
            return Ok(Self(DynOrigin::from_str(origin)?));
        }
        Err(PyTypeError::new_err(
            "`origin` must be either a string or an integer",
        ))
    }

    fn __repr__(&self) -> String {
        format!("Origin(\"{}\")", self.name())
    }

    fn __str__(&self) -> &str {
        self.name()
    }

    fn __getnewargs__(&self) -> (&str,) {
        (self.name(),)
    }

    pub fn id(&self) -> i32 {
        self.0.id().0
    }

    pub fn name(&self) -> &'static str {
        self.0.name()
    }
}
