/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::series::{Series, SeriesError};
use pyo3::exceptions::PyValueError;
use pyo3::{pyclass, pymethods, PyErr, PyResult};

impl From<SeriesError> for PyErr {
    fn from(err: SeriesError) -> Self {
        PyValueError::new_err(err.to_string())
    }
}

#[pyclass(name = "Series", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PySeries(pub Series<Vec<f64>, Vec<f64>>);

#[pymethods]
impl PySeries {
    #[new]
    #[pyo3(signature = (x, y, method="linear"))]
    fn new(x: Vec<f64>, y: Vec<f64>, method: &str) -> PyResult<Self> {
        let series = match method {
            "linear" => Series::new(x, y)?,
            "cubic_spline" => Series::with_cubic_spline(x, y)?,
            _ => return Err(PyValueError::new_err("unknown method")),
        };
        Ok(PySeries(series))
    }

    fn interpolate(&self, xp: f64) -> f64 {
        self.0.interpolate(xp)
    }
}
