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
///
/// Examples:
///     >>> import lox_space as lox
///     >>> x = [0.0, 1.0, 2.0, 3.0]
///     >>> y = [0.0, 1.0, 4.0, 9.0]
///
///     >>> # Linear interpolation
///     >>> series = lox.Series(x, y)
///     >>> series.interpolate(1.5)
///     2.5
///
///     >>> # Cubic spline interpolation
///     >>> series = lox.Series(x, y, method="cubic_spline")
///     >>> series.interpolate(1.5)
///     2.25
#[pyclass(name = "Series", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PySeries(pub Series<Vec<f64>, Vec<f64>>);

#[pymethods]
impl PySeries {
    #[new]
    #[pyo3(signature = (x, y, method="linear"))]
    fn new(x: Vec<f64>, y: Vec<f64>, method: &str) -> PyResult<Self> {
        let series = match method {
            "linear" => Series::try_linear(x, y).map_err(PySeriesError)?,
            "cubic_spline" => Series::try_cubic_spline(x, y).map_err(PySeriesError)?,
            _ => return Err(PyValueError::new_err("unknown method")),
        };
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
