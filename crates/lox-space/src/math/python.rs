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

/// Interpolation series for 1D data.
///
/// Series provides interpolation between data points using either linear
/// or cubic spline methods.
///
/// Args:
///     x: Array of x values (must be monotonically increasing).
///     y: Array of y values (same length as x).
///     method: Interpolation method ("linear" or "cubic_spline").
///
/// Raises:
///     ValueError: If x and y have different lengths or x is not monotonic.
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

    /// Interpolate a y value at the given x coordinate.
    ///
    /// Args:
    ///     xp: The x value to interpolate at.
    ///
    /// Returns:
    ///     The interpolated y value.
    fn interpolate(&self, xp: f64) -> f64 {
        self.0.interpolate(xp)
    }
}
