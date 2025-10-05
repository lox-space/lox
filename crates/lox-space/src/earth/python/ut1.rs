/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::path::PathBuf;

use lox_earth::iers::{EopParser, EopProvider};
use pyo3::exceptions::PyException;
use pyo3::{PyErr, PyResult, create_exception, pyclass, pymethods};

create_exception!(lox_space, EopParserError, PyException);

pub struct PyEopParserError(pub lox_earth::iers::EopParserError);

impl From<PyEopParserError> for PyErr {
    fn from(err: PyEopParserError) -> Self {
        EopParserError::new_err(err.0.to_string())
    }
}

create_exception!(lox_space, EopProviderError, PyException);

pub struct PyEopProviderError(pub lox_earth::iers::EopProviderError);

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
    #[pyo3(signature = (iau1980=None, iau2000=None))]
    #[new]
    pub fn new(iau1980: Option<PathBuf>, iau2000: Option<PathBuf>) -> PyResult<PyEopProvider> {
        let mut parser = EopParser::new();
        if let Some(iau1980) = iau1980 {
            parser.with_iau1980(iau1980);
        }
        if let Some(iau2000) = iau2000 {
            parser.with_iau2000(iau2000);
        }
        Ok(PyEopProvider(parser.parse().map_err(PyEopParserError)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_helpers::data_dir;
    use crate::time::python::time::PyTime;

    use pyo3::{Bound, IntoPyObjectExt, Python};

    #[test]
    #[should_panic(expected = "No such file")]
    fn test_ut1_provider_invalid_path() {
        let _provider = PyEopProvider::new(Some("invalid_path".into()), None).unwrap();
    }

    #[test]
    #[should_panic(expected = "extrapolated")]
    fn test_ut1_provider_extrapolated() {
        Python::attach(|py| {
            let provider = Bound::new(
                py,
                PyEopProvider::new(None, Some(data_dir().join("finals2000A.all.csv"))).unwrap(),
            )
            .unwrap();
            let tai =
                PyTime::new(&"TAI".into_bound_py_any(py).unwrap(), 2100, 1, 1, 0, 0, 0.0).unwrap();
            let _ut1 = tai
                .to_scale(&"UT1".into_bound_py_any(py).unwrap(), Some(&provider))
                .unwrap();
        })
    }
}
