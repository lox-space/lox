// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use pyo3::exceptions::PyException;
use pyo3::types::PyType;
use pyo3::{Bound, PyResult, create_exception, pyclass, pymethods};

use crate::time::deltas::TimeDelta;

create_exception!(lox_space, NonFiniteTimeDeltaError, PyException);

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
#[pyclass(name = "TimeDelta", module = "lox_space", frozen)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyTimeDelta(pub TimeDelta);

#[pymethods]
impl PyTimeDelta {
    #[new]
    pub fn new(seconds: f64) -> Self {
        Self(TimeDelta::from_seconds_f64(seconds))
    }

    pub fn __repr__(&self) -> String {
        format!("TimeDelta({})", self.to_decimal_seconds())
    }

    pub fn __str__(&self) -> String {
        format!("{} seconds", self.to_decimal_seconds())
    }

    pub fn __float__(&self) -> f64 {
        self.to_decimal_seconds()
    }

    pub fn __neg__(&self) -> Self {
        Self(-self.0)
    }

    pub fn __add__(&self, other: PyTimeDelta) -> Self {
        Self(self.0 + other.0)
    }

    pub fn __sub__(&self, other: PyTimeDelta) -> Self {
        Self(self.0 - other.0)
    }

    pub fn __eq__(&self, other: PyTimeDelta) -> bool {
        self.0 == other.0
    }

    /// Return the integer seconds component.
    ///
    /// Returns:
    ///     Integer seconds (sign matches the delta).
    ///
    /// Raises:
    ///     NonFiniteTimeDeltaError: If the delta is non-finite.
    pub fn seconds(&self) -> PyResult<i64> {
        self.0.seconds().ok_or(NonFiniteTimeDeltaError::new_err(
            "cannot access seconds for non-finite time delta",
        ))
    }

    /// Return the subsecond (fractional second) component.
    ///
    /// Returns:
    ///     Fractional seconds (0.0 to 1.0).
    ///
    /// Raises:
    ///     NonFiniteTimeDeltaError: If the delta is non-finite.
    pub fn subsecond(&self) -> PyResult<f64> {
        self.0.subsecond().ok_or(NonFiniteTimeDeltaError::new_err(
            "cannot access subsecond for non-finite time delta",
        ))
    }

    /// Create a TimeDelta from integer seconds.
    #[classmethod]
    pub fn from_seconds(_cls: &Bound<'_, PyType>, seconds: i64) -> Self {
        Self(TimeDelta::from_seconds(seconds))
    }

    /// Create a TimeDelta from minutes.
    #[classmethod]
    pub fn from_minutes(_cls: &Bound<'_, PyType>, minutes: f64) -> PyResult<Self> {
        Ok(Self(TimeDelta::from_minutes(minutes)))
    }

    /// Create a TimeDelta from hours.
    #[classmethod]
    pub fn from_hours(_cls: &Bound<'_, PyType>, hours: f64) -> PyResult<Self> {
        Ok(Self(TimeDelta::from_hours(hours)))
    }

    /// Create a TimeDelta from days (86400 seconds per day).
    #[classmethod]
    pub fn from_days(_cls: &Bound<'_, PyType>, days: f64) -> PyResult<Self> {
        Ok(Self(TimeDelta::from_days(days)))
    }

    /// Create a TimeDelta from Julian years (365.25 days per year).
    #[classmethod]
    pub fn from_julian_years(_cls: &Bound<'_, PyType>, years: f64) -> PyResult<Self> {
        Ok(Self(TimeDelta::from_julian_years(years)))
    }

    /// Create a TimeDelta from Julian centuries (36525 days per century).
    #[classmethod]
    pub fn from_julian_centuries(_cls: &Bound<'_, PyType>, centuries: f64) -> PyResult<Self> {
        Ok(Self(TimeDelta::from_julian_centuries(centuries)))
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
        let range = TimeDelta::range(start..=end).with_step(step);
        Ok(range.into_iter().map(Self).collect())
    }

    /// Convert to decimal seconds.
    ///
    /// Returns:
    ///     The duration as a float in seconds.
    pub fn to_decimal_seconds(&self) -> f64 {
        self.0.to_seconds().to_f64()
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;
    use pyo3::Python;

    use super::*;

    #[test]
    fn test_pytimedelta_repr() {
        let td = PyTimeDelta::new(123.456);
        assert_eq!(td.__repr__(), "TimeDelta(123.456)");
        assert_eq!(td.__str__(), "123.456 seconds");
    }

    #[test]
    fn test_pytimedelta_ops() {
        let td1 = PyTimeDelta::new(123.456);
        let td2 = PyTimeDelta::new(654.321);
        assert_eq!(td1.__add__(td2.clone()).to_decimal_seconds(), 777.777);
        assert_eq!(td1.__sub__(td2.clone()).to_decimal_seconds(), -530.865);
        assert_eq!(td1.__neg__().to_decimal_seconds(), -123.456);
        assert_eq!(td1.__float__(), 123.456);
    }

    #[test]
    fn test_pytimedelta_seconds() {
        let td = PyTimeDelta::new(123.456);
        assert_eq!(td.seconds().unwrap(), 123);
        assert_approx_eq!(td.subsecond().unwrap(), 0.456, atol <= 1e-14);
    }

    #[test]
    fn test_pytimedelta_constructors() {
        Python::attach(|py| {
            let cls = PyType::new::<PyTimeDelta>(py);
            let td = PyTimeDelta::from_seconds(&cls, 123);
            assert_eq!(td.to_decimal_seconds(), 123.0);
            let td = PyTimeDelta::from_minutes(&cls, 2.0).unwrap();
            assert_eq!(td.to_decimal_seconds(), 120.0);
            let td = PyTimeDelta::from_hours(&cls, 2.0).unwrap();
            assert_eq!(td.to_decimal_seconds(), 7200.0);
            let td = PyTimeDelta::from_days(&cls, 2.0).unwrap();
            assert_eq!(td.to_decimal_seconds(), 172800.0);
            let td = PyTimeDelta::from_julian_years(&cls, 2.0).unwrap();
            assert_eq!(td.to_decimal_seconds(), 63115200.0);
            let td = PyTimeDelta::from_julian_centuries(&cls, 2.0).unwrap();
            assert_eq!(td.to_decimal_seconds(), 6311520000.0);
        })
    }

    #[test]
    #[should_panic]
    fn test_pytimedelta_error() {
        PyTimeDelta::new(f64::NAN).0.seconds().unwrap();
    }
}
