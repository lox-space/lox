// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_time::deltas::InvalidFloatSeconds;
use pyo3::exceptions::PyValueError;
use pyo3::types::PyType;
use pyo3::{Bound, PyErr, PyResult, pyclass, pymethods};

use crate::time::deltas::TimeDelta;
use crate::time::intervals::TimeDeltaInterval;

/// Represents a duration or time difference.
///
/// `TimeDelta` represents a time interval with femtosecond precision.
/// It can be added to or subtracted from `Time` objects, and arithmetic
/// operations between `TimeDelta` objects are supported.
///
/// Args:
///     seconds: Duration in seconds (can be negative).
///
/// See Also:
///     Time: For representing instants in time.
#[pyclass(name = "TimeDelta", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyTimeDelta(pub TimeDelta);

#[pymethods]
impl PyTimeDelta {
    #[new]
    /// Constructs a `TimeDelta` from a duration in seconds.
    pub fn new(seconds: f64) -> PyResult<Self> {
        Ok(Self(
            TimeDelta::try_from_seconds_f64(seconds).map_err(PyInvalidFloatSeconds)?,
        ))
    }

    /// Returns the developer representation of the `TimeDelta`.
    pub fn __repr__(&self) -> String {
        format!("TimeDelta({})", self.to_decimal_seconds())
    }

    /// Returns the human-readable string representation of the `TimeDelta`.
    pub fn __str__(&self) -> String {
        format!("{} seconds", self.to_decimal_seconds())
    }

    /// Returns the duration as a `float` in decimal seconds.
    pub fn __float__(&self) -> f64 {
        self.to_decimal_seconds()
    }

    /// Returns the negation of the `TimeDelta`.
    pub fn __neg__(&self) -> Self {
        Self(-self.0)
    }

    /// Returns the sum of two `TimeDelta` values.
    pub fn __add__(&self, other: PyTimeDelta) -> Self {
        Self(self.0 + other.0)
    }

    /// Returns the difference of two `TimeDelta` values.
    pub fn __sub__(&self, other: PyTimeDelta) -> Self {
        Self(self.0 - other.0)
    }

    /// Returns the `TimeDelta` scaled by a floating-point factor.
    pub fn __mul__(&self, other: f64) -> PyResult<Self> {
        Ok(Self(self.0.try_mul(other).map_err(PyInvalidFloatSeconds)?))
    }

    /// Returns the `TimeDelta` scaled by a floating-point factor (right-hand side).
    pub fn __rmul__(&self, other: f64) -> PyResult<Self> {
        Ok(Self(self.0.try_mul(other).map_err(PyInvalidFloatSeconds)?))
    }

    /// Returns `true` if two `TimeDelta` values are equal.
    pub fn __eq__(&self, other: PyTimeDelta) -> bool {
        self.0 == other.0
    }

    /// Return the integer seconds component.
    ///
    /// Returns:
    ///     Integer seconds (sign matches the delta).
    pub fn seconds(&self) -> i64 {
        self.0.seconds()
    }

    /// Return the subsecond (fractional second) component.
    ///
    /// Returns:
    ///     Fractional seconds (0.0 to 1.0).
    pub fn subsecond(&self) -> f64 {
        self.0.subsecond()
    }

    /// Create a TimeDelta from integer seconds.
    #[classmethod]
    pub fn from_seconds(_cls: &Bound<'_, PyType>, seconds: i64) -> Self {
        Self(TimeDelta::from_seconds(seconds))
    }

    /// Create a TimeDelta from minutes.
    #[classmethod]
    pub fn from_minutes(_cls: &Bound<'_, PyType>, minutes: f64) -> PyResult<Self> {
        Ok(Self(
            TimeDelta::try_from_minutes_f64(minutes).map_err(PyInvalidFloatSeconds)?,
        ))
    }

    /// Create a TimeDelta from hours.
    #[classmethod]
    pub fn from_hours(_cls: &Bound<'_, PyType>, hours: f64) -> PyResult<Self> {
        Ok(Self(
            TimeDelta::try_from_hours_f64(hours).map_err(PyInvalidFloatSeconds)?,
        ))
    }

    /// Create a TimeDelta from days (86400 seconds per day).
    #[classmethod]
    pub fn from_days(_cls: &Bound<'_, PyType>, days: f64) -> PyResult<Self> {
        Ok(Self(
            TimeDelta::try_from_days_f64(days).map_err(PyInvalidFloatSeconds)?,
        ))
    }

    /// Create a TimeDelta from Julian years (365.25 days per year).
    #[classmethod]
    pub fn from_julian_years(_cls: &Bound<'_, PyType>, years: f64) -> PyResult<Self> {
        Ok(Self(
            TimeDelta::try_from_julian_years(years).map_err(PyInvalidFloatSeconds)?,
        ))
    }

    /// Create a TimeDelta from Julian centuries (36525 days per century).
    #[classmethod]
    pub fn from_julian_centuries(_cls: &Bound<'_, PyType>, centuries: f64) -> PyResult<Self> {
        Ok(Self(
            TimeDelta::try_from_julian_centuries(centuries).map_err(PyInvalidFloatSeconds)?,
        ))
    }

    /// Create a TimeDelta from integer milliseconds.
    #[classmethod]
    pub fn from_milliseconds(_cls: &Bound<'_, PyType>, ms: i64) -> Self {
        Self(TimeDelta::from_milliseconds(ms))
    }

    /// Create a TimeDelta from integer microseconds.
    #[classmethod]
    pub fn from_microseconds(_cls: &Bound<'_, PyType>, us: i64) -> Self {
        Self(TimeDelta::from_microseconds(us))
    }

    /// Create a TimeDelta from integer nanoseconds.
    #[classmethod]
    pub fn from_nanoseconds(_cls: &Bound<'_, PyType>, ns: i64) -> Self {
        Self(TimeDelta::from_nanoseconds(ns))
    }

    /// Create a TimeDelta from integer picoseconds.
    #[classmethod]
    pub fn from_picoseconds(_cls: &Bound<'_, PyType>, ps: i64) -> Self {
        Self(TimeDelta::from_picoseconds(ps))
    }

    /// Create a TimeDelta from integer femtoseconds.
    #[classmethod]
    pub fn from_femtoseconds(_cls: &Bound<'_, PyType>, fs: i64) -> Self {
        Self(TimeDelta::from_femtoseconds(fs))
    }

    /// Create a TimeDelta from integer attoseconds.
    #[classmethod]
    pub fn from_attoseconds(_cls: &Bound<'_, PyType>, atto: i64) -> Self {
        Self(TimeDelta::from_attoseconds(atto))
    }

    /// Create a range of TimeDelta values.
    ///
    /// Args:
    ///     start: Start value in seconds (inclusive).
    ///     end: End value in seconds (inclusive).
    ///     step: Step size in seconds. Defaults to 1.
    ///
    /// Returns:
    ///     A list of TimeDelta objects.
    ///
    /// Examples:
    ///     >>> deltas = lox.TimeDelta.range(0, 10, 2)  # [0, 2, 4, 6, 8, 10]
    #[classmethod]
    #[pyo3(signature = (start, end, step=None))]
    pub fn range(
        _cls: &Bound<'_, PyType>,
        start: i64,
        end: i64,
        step: Option<i64>,
    ) -> PyResult<Vec<Self>> {
        let step = TimeDelta::from_seconds(step.unwrap_or(1));
        let interval =
            TimeDeltaInterval::new(TimeDelta::from_seconds(start), TimeDelta::from_seconds(end));
        Ok(interval.step_by(step).map(Self).collect())
    }

    /// Convert to decimal seconds.
    ///
    /// Returns:
    ///     The duration as a float in seconds.
    pub fn to_decimal_seconds(&self) -> f64 {
        self.0.to_seconds().to_f64()
    }
}

pub(crate) struct PyInvalidFloatSeconds(pub InvalidFloatSeconds);

impl From<PyInvalidFloatSeconds> for PyErr {
    fn from(err: PyInvalidFloatSeconds) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}
