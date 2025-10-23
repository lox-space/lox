// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::path::PathBuf;

use pyo3::{
    Bound, PyAny, PyErr, PyResult, exceptions::PyValueError, pyclass, pymethods,
    types::PyAnyMethods,
};

use crate::ephem::spk::parser::{DafSpkError, Spk, parse_daf_spk};

pub struct PyDafSpkError(pub DafSpkError);

impl From<PyDafSpkError> for PyErr {
    fn from(err: PyDafSpkError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

#[pyclass(name = "SPK", module = "lox_space", frozen)]
pub struct PySpk(pub Spk);

#[pymethods]
impl PySpk {
    #[new]
    fn new(path: &Bound<'_, PyAny>) -> PyResult<Self> {
        let path = path.extract::<PathBuf>()?;
        let data = std::fs::read(path)?;
        let spk = parse_daf_spk(&data).map_err(PyDafSpkError)?;
        Ok(PySpk(spk))
    }
}
