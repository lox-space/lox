// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::error::Error;

use crate::math::series::{Series, SeriesError};
use lox_core::error::find_source;
use lox_core::math::series::{InterpolationType, UnknownInterpolationType};
use pyo3::exceptions::PyValueError;
use pyo3::{PyErr, PyResult, Python, pyclass, pymethods};

/// Recovers a Python exception embedded anywhere in `err`'s source chain.
///
/// Returns `None` when the chain contains no `PyErr`.
pub fn raised_exception(py: Python<'_>, err: &(dyn Error + 'static)) -> Option<PyErr> {
    find_source::<PyErr>(err).map(|e| e.clone_ref(py))
}

/// Converts an error from a fallible user callback into a `PyErr`.
///
/// A Python exception embedded in the source chain is re-raised as the
/// original exception object; any other error becomes a `ValueError` whose
/// message preserves the full error chain.
pub fn callback_pyerr(py: Python<'_>, err: &(dyn Error + 'static)) -> PyErr {
    if let Some(exc) = raised_exception(py, err) {
        return exc;
    }
    let mut msg = err.to_string();
    let mut source = err.source();
    while let Some(e) = source {
        let s = e.to_string();
        if !msg.contains(&s) {
            msg.push_str(": ");
            msg.push_str(&s);
        }
        source = e.source();
    }
    PyValueError::new_err(msg)
}

/// Python error wrapper for [`SeriesError`].
pub struct PySeriesError(pub SeriesError);

impl From<PySeriesError> for PyErr {
    fn from(err: PySeriesError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

/// Python error wrapper for [`UnknownInterpolationType`].
pub struct PyUnknownInterpolationType(pub UnknownInterpolationType);

impl From<PyUnknownInterpolationType> for PyErr {
    fn from(err: PyUnknownInterpolationType) -> Self {
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
///     method: Interpolation method ("linear" or "cubic").
///
/// Raises:
///     ValueError: If x and y have different lengths or x is not monotonic.
#[pyclass(name = "Series", module = "lox_space", frozen, from_py_object)]
#[derive(Clone, Debug)]
pub struct PySeries(pub Series);

#[pymethods]
impl PySeries {
    #[new]
    #[pyo3(signature = (x, y, interpolation="linear"))]
    fn new(x: Vec<f64>, y: Vec<f64>, interpolation: &str) -> PyResult<Self> {
        let interpolation: InterpolationType =
            interpolation.parse().map_err(PyUnknownInterpolationType)?;
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

    fn __repr__(&self) -> String {
        let x = self.0.x();
        let n = x.len();
        if n == 0 {
            return "Series([], [])".to_string();
        }
        format!("Series([{}, ..., {}], [...], {n} points)", x[0], x[n - 1],)
    }
}
