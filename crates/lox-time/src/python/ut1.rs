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
    pub fn new(path: &str) -> PyResult<PyUt1Provider> {
        let provider = DeltaUt1Tai::new(path, &BuiltinLeapSeconds)?;
        Ok(PyUt1Provider(provider))
    }
}

#[cfg(test)]
mod tests {
    use pyo3::{Bound, Python};

    use crate::{python::time::PyTime, test_helpers::data_dir};

    use super::*;

    #[test]
    #[should_panic(expected = "No such file")]
    fn test_ut1_provider_invalid_path() {
        let _provider = PyUt1Provider::new("invalid_path").unwrap();
    }

    #[test]
    #[should_panic(expected = "extrapolated")]
    fn test_ut1_provider_extrapolated() {
        Python::with_gil(|py| {
            let provider = Bound::new(
                py,
                PyUt1Provider::new(data_dir().join("finals2000A.all.csv").to_str().unwrap())
                    .unwrap(),
            )
            .unwrap();
            let tai = PyTime::new("TAI", 2100, 1, 1, 0, 0, 0.0).unwrap();
            let _ut1 = tai.to_ut1(Some(&provider)).unwrap();
        })
    }
}
