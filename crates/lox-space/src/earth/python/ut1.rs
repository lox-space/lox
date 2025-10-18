/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::path::PathBuf;

use lox_earth::eop::{EopParser, EopProvider};
use pyo3::exceptions::PyException;
use pyo3::types::{PyAnyMethods, PyTuple};
use pyo3::{Bound, PyErr, PyResult, create_exception, pyclass, pymethods};

create_exception!(lox_space, EopParserError, PyException);

pub struct PyEopParserError(pub lox_earth::eop::EopParserError);

impl From<PyEopParserError> for PyErr {
    fn from(err: PyEopParserError) -> Self {
        EopParserError::new_err(err.0.to_string())
    }
}

create_exception!(lox_space, EopProviderError, PyException);

pub struct PyEopProviderError(pub lox_earth::eop::EopProviderError);

impl From<PyEopProviderError> for PyErr {
    fn from(err: PyEopProviderError) -> Self {
        EopProviderError::new_err(err.0.to_string())
    }
}

#[pyclass(name = "EOPProvider", module = "lox_space", frozen)]
#[derive(Debug)]
pub struct PyEopProvider(pub EopProvider);

#[pymethods]
impl PyEopProvider {
    #[pyo3(signature = (*args))]
    #[new]
    pub fn new(args: &Bound<'_, PyTuple>) -> PyResult<PyEopProvider> {
        let mut parser = EopParser::new();
        if let Ok((path1, path2)) = args.extract::<(PathBuf, PathBuf)>() {
            parser.from_paths(path1, path2);
        } else if let Ok((path,)) = args.extract::<(PathBuf,)>() {
            parser.from_path(path);
        }
        Ok(PyEopProvider(parser.parse().map_err(PyEopParserError)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::time::python::time::PyTime;
    use lox_test_utils::data_dir;
    use pyo3::{Bound, IntoPyObject, IntoPyObjectExt, Python};

    #[test]
    #[should_panic(expected = "No such file")]
    fn test_ut1_provider_invalid_path() {
        Python::attach(|py| {
            let path = (data_dir().join("invalid"),).into_pyobject(py).unwrap();
            let _provider = PyEopProvider::new(&path).unwrap();
        })
    }

    #[test]
    #[should_panic(expected = "extrapolated")]
    fn test_ut1_provider_extrapolated() {
        Python::attach(|py| {
            let path = (data_dir().join("finals2000A.all.csv"),)
                .into_pyobject(py)
                .unwrap();
            let provider = Bound::new(py, PyEopProvider::new(&path).unwrap()).unwrap();
            let tai =
                PyTime::new(&"TAI".into_bound_py_any(py).unwrap(), 2100, 1, 1, 0, 0, 0.0).unwrap();
            let _ut1 = tai
                .to_scale(&"UT1".into_bound_py_any(py).unwrap(), Some(&provider))
                .unwrap();
        })
    }
}
