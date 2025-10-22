// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::math::series::{Series, SeriesError};
use pyo3::exceptions::PyValueError;
use pyo3::{PyErr, PyResult, pyclass, pymethods};

pub struct PySeriesError(pub SeriesError);

impl From<PySeriesError> for PyErr {
    fn from(err: PySeriesError) -> Self {
        PyValueError::new_err(err.0.to_string())
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
            "linear" => Series::new(x, y).map_err(PySeriesError)?,
            "cubic_spline" => Series::with_cubic_spline(x, y).map_err(PySeriesError)?,
            _ => return Err(PyValueError::new_err("unknown method")),
        };
        Ok(PySeries(series))
    }

    fn interpolate(&self, xp: f64) -> f64 {
        self.0.interpolate(xp)
    }
}
