// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::math::series::{Series, SeriesError};
use lox_math::series::InterpolationType;
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
pub struct PySeries(pub Series);

#[pymethods]
impl PySeries {
    #[new]
    #[pyo3(signature = (x, y, interpolation="linear"))]
    fn new(x: Vec<f64>, y: Vec<f64>, interpolation: &str) -> PyResult<Self> {
        let interpolation = match interpolation {
            "linear" => InterpolationType::Linear,
            "cubic_spline" => InterpolationType::CubicSpline,
            _ => return Err(PyValueError::new_err("unknown interpolation type")),
        };
        let series = Series::try_new(x, y, interpolation).map_err(PySeriesError)?;
        Ok(PySeries(series))
    }

    fn interpolate(&self, xp: f64) -> f64 {
        self.0.interpolate(xp)
    }
}
