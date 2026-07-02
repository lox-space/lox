// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_core::math::series::InterpolationType;
use pyo3::exceptions::PyValueError;
use pyo3::types::PyType;
use pyo3::{Bound, PyResult, pyclass, pymethods};

use crate::math::python::{PySeriesError, PyUnknownInterpolationType};
use crate::time::python::time::PyTime;
use crate::time::series::TimeSeries;
use crate::time::time_scales::DynTimeScale;

/// Time-indexed interpolation series.
///
/// Wraps a `Series` with a start epoch, allowing interpolation by `Time` values
/// rather than raw float offsets.
///
/// Args:
///     times: List of Time objects (must be in chronological order).
///     values: List of y values (same length as times).
///     interpolation: Interpolation method ("linear" or "cubic"). Defaults to "linear".
///
/// Raises:
///     ValueError: If times is empty, lengths mismatch, times are not monotonic,
///         or interpolation is unknown.
///
/// Examples:
///     >>> epoch = lox.Time("TAI", 2024, 1, 1)
///     >>> times = [epoch, epoch + 60 * lox.seconds, epoch + 120 * lox.seconds]
///     >>> ts = lox.TimeSeries(times, [1.0, 2.0, 3.0])
///     >>> ts.interpolate(epoch + 30 * lox.seconds)
///     1.5
#[pyclass(name = "TimeSeries", module = "lox_space", frozen, from_py_object)]
#[derive(Clone, Debug)]
pub struct PyTimeSeries(pub TimeSeries<DynTimeScale>);

#[pymethods]
impl PyTimeSeries {
    #[new]
    #[pyo3(signature = (times, values, interpolation="linear"))]
    fn new(times: Vec<PyTime>, values: Vec<f64>, interpolation: &str) -> PyResult<Self> {
        if times.is_empty() {
            return Err(PyValueError::new_err("times must not be empty"));
        }
        let interpolation: InterpolationType =
            interpolation.parse().map_err(PyUnknownInterpolationType)?;
        let epoch = times[0].0;
        let x: Vec<f64> = times
            .iter()
            .map(|t| (t.0 - epoch).to_seconds().to_f64())
            .collect();
        let ts = TimeSeries::try_new(epoch, x, values, interpolation).map_err(PySeriesError)?;
        Ok(PyTimeSeries(ts))
    }

    /// Create a TimeSeries from an epoch and relative offsets in seconds.
    ///
    /// Args:
    ///     epoch: The reference epoch for the time axis.
    ///     x: Array of time offsets in seconds from the epoch (must be monotonically increasing).
    ///     y: Array of y values (same length as x).
    ///     interpolation: Interpolation method ("linear" or "cubic"). Defaults to "linear".
    ///
    /// Returns:
    ///     A new TimeSeries.
    ///
    /// Raises:
    ///     ValueError: If x and y have different lengths, x is not monotonic, or interpolation is unknown.
    #[classmethod]
    #[pyo3(signature = (epoch, x, y, interpolation="linear"))]
    fn from_offsets(
        _cls: &Bound<'_, PyType>,
        epoch: PyTime,
        x: Vec<f64>,
        y: Vec<f64>,
        interpolation: &str,
    ) -> PyResult<Self> {
        let interpolation: InterpolationType =
            interpolation.parse().map_err(PyUnknownInterpolationType)?;
        let ts = TimeSeries::try_new(epoch.0, x, y, interpolation).map_err(PySeriesError)?;
        Ok(PyTimeSeries(ts))
    }

    /// Interpolate a y value at the given time.
    ///
    /// Args:
    ///     time: The time at which to interpolate.
    ///
    /// Returns:
    ///     The interpolated y value.
    fn interpolate(&self, time: PyTime) -> f64 {
        self.0.interpolate(time.0)
    }

    /// Return the start epoch.
    ///
    /// Returns:
    ///     The reference epoch of this time series.
    #[getter]
    fn epoch(&self) -> PyTime {
        PyTime(self.0.epoch())
    }

    /// Return absolute timestamps for each data point.
    ///
    /// Returns:
    ///     List of Time objects corresponding to each x value.
    fn times(&self) -> Vec<PyTime> {
        self.0.times().into_iter().map(PyTime).collect()
    }

    /// Return the y values.
    ///
    /// Returns:
    ///     List of y values.
    fn values(&self) -> Vec<f64> {
        self.0.values().to_vec()
    }

    /// Return the first data point as (time, value).
    ///
    /// Returns:
    ///     A tuple of (Time, float) for the first data point.
    fn first(&self) -> (PyTime, f64) {
        let (t, v) = self.0.first();
        (PyTime(t), v)
    }

    /// Return the last data point as (time, value).
    ///
    /// Returns:
    ///     A tuple of (Time, float) for the last data point.
    fn last(&self) -> (PyTime, f64) {
        let (t, v) = self.0.last();
        (PyTime(t), v)
    }

    fn __repr__(&self) -> String {
        let x = self.0.series().x();
        let n = x.len();
        format!(
            "TimeSeries(epoch={}, [{}, ..., {}], {n} points)",
            self.0.epoch(),
            x[0],
            x[n - 1],
        )
    }
}
