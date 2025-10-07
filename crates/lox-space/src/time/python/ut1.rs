/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::time::ut1::{DeltaUt1Tai, DeltaUt1TaiError, ExtrapolatedDeltaUt1Tai};
use crate::time::utc::leap_seconds::BuiltinLeapSeconds;
use pyo3::exceptions::PyValueError;
use pyo3::{PyErr, PyResult, pyclass, pymethods};

pub struct PyExtrapolatedDeltaUt1Tai(pub ExtrapolatedDeltaUt1Tai);

impl From<PyExtrapolatedDeltaUt1Tai> for PyErr {
    fn from(err: PyExtrapolatedDeltaUt1Tai) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

pub struct PyDeltaUt1TaiError(pub DeltaUt1TaiError);

impl From<PyDeltaUt1TaiError> for PyErr {
    fn from(err: PyDeltaUt1TaiError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

#[pyclass(name = "UT1Provider", module = "lox_space", frozen)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyUt1Provider(pub DeltaUt1Tai);

#[pymethods]
impl PyUt1Provider {
    #[new]
    pub fn new(path: &str) -> PyResult<PyUt1Provider> {
        let provider = DeltaUt1Tai::new(path, &BuiltinLeapSeconds).map_err(PyDeltaUt1TaiError)?;
        Ok(PyUt1Provider(provider))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::time::python::time::PyTime;
    use crate::time::test_helpers::data_dir;

    use pyo3::{Bound, IntoPyObjectExt, Python};

    #[test]
    #[should_panic(expected = "No such file")]
    fn test_ut1_provider_invalid_path() {
        let _provider = PyUt1Provider::new("invalid_path").unwrap();
    }

    #[test]
    #[should_panic(expected = "extrapolated")]
    fn test_ut1_provider_extrapolated() {
        Python::attach(|py| {
            let provider = Bound::new(
                py,
                PyUt1Provider::new(data_dir().join("finals2000A.all.csv").to_str().unwrap())
                    .unwrap(),
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
