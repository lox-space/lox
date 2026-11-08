// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_core::time::constants::ATTOSECONDS_IN_SECOND;
use lox_time::{InvalidFloatSeconds, TimeDeltaUnit};
use numpy::PyUntypedArray;
use pyo3::IntoPyObjectExt;
use pyo3::exceptions::{PyOverflowError, PyTypeError, PyValueError, PyZeroDivisionError};
use pyo3::prelude::*;
use pyo3::types::{PyFloat, PyInt, PyType};

use crate::time::TimeDelta;
use crate::time::intervals::TimeDeltaInterval;
use crate::units::python::{format_with_unit, repr_f64};

/// The largest number of values `TimeDelta.range` returns.
const MAX_RANGE_LEN: usize = 10_000_000;

/// A unit of `TimeDelta`.
///
/// Multiply a number by a unit to create a duration, e.g. `90 * lox.minutes`.
///
/// Args:
///     symbol: The unit symbol, e.g. "min".
#[pyclass(
    name = "TimeDeltaUnit",
    module = "lox_space",
    frozen,
    eq,
    hash,
    from_py_object
)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PyTimeDeltaUnit(pub TimeDeltaUnit);

#[pymethods]
impl PyTimeDeltaUnit {
    #[new]
    fn new(symbol: &str) -> PyResult<Self> {
        symbol
            .parse()
            .map(Self)
            .map_err(|_| PyValueError::new_err(format!("unknown unit `{symbol}`")))
    }

    /// The unit symbol.
    #[getter]
    fn symbol(&self) -> &'static str {
        self.0.symbol()
    }

    fn __mul__(&self, py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        match new_in(value, self.0)? {
            Some(delta) => Ok(PyTimeDelta(delta).into_pyobject(py)?.into_any().unbind()),
            None => Ok(py.NotImplemented()),
        }
    }

    fn __rmul__(&self, py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.__mul__(py, value)
    }

    fn __getnewargs__(&self) -> (&'static str,) {
        (self.0.symbol(),)
    }

    fn __repr__(&self) -> String {
        format!("TimeDeltaUnit(\"{}\")", self.0.symbol())
    }

    fn __str__(&self) -> &'static str {
        self.0.symbol()
    }
}

/// Represents a duration or time difference.
///
/// `TimeDelta` represents a time interval with femtosecond precision.
/// It can be added to or subtracted from `Time` objects, and arithmetic
/// operations between `TimeDelta` objects are supported.
///
/// Create one by multiplying a number by a unit, e.g. `90 * lox.minutes`, or with a
/// named constructor such as `TimeDelta.from_seconds(5400)`.
///
/// See Also:
///     Time: For representing instants in time.
#[pyclass(
    name = "TimeDelta",
    module = "lox_space",
    frozen,
    eq,
    ord,
    from_py_object
)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PyTimeDelta(pub TimeDelta);

#[pymethods]
impl PyTimeDelta {
    /// Returns a representation that evaluates to an equal value.
    pub fn __repr__(&self) -> String {
        let seconds = self.to_seconds();
        if TimeDelta::try_new_in(seconds, TimeDeltaUnit::Seconds).is_ok_and(|delta| delta == self.0)
        {
            return format!("TimeDelta.from_seconds({})", repr_f64(seconds));
        }
        format!(
            "TimeDelta.from_seconds({}) + TimeDelta.from_attoseconds({})",
            self.0.seconds(),
            self.0.attoseconds()
        )
    }

    /// Returns the human-readable string representation of the `TimeDelta`.
    pub fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __format__(&self, py: Python<'_>, format_spec: &str) -> PyResult<String> {
        if format_spec.is_empty() {
            return Ok(self.0.to_string());
        }
        self.display_in(py, PyTimeDeltaUnit(TimeDeltaUnit::Seconds), format_spec)
    }

    fn __reduce__<'py>(slf: &Bound<'py, Self>) -> PyResult<(Bound<'py, PyAny>, (i64, i64))> {
        let ctor = slf.get_type().getattr("_from_parts")?;
        let delta = slf.get().0;
        Ok((ctor, (delta.seconds(), delta.attoseconds())))
    }

    #[staticmethod]
    fn _from_parts(seconds: i64, attoseconds: i64) -> PyResult<Self> {
        if !(0..ATTOSECONDS_IN_SECOND).contains(&attoseconds) {
            return Err(PyValueError::new_err(format!(
                "attoseconds must be in [0, {ATTOSECONDS_IN_SECOND}), got {attoseconds}"
            )));
        }
        Ok(Self(TimeDelta::new(seconds, attoseconds)))
    }

    /// Returns the duration in the given unit.
    pub fn to(&self, unit: PyTimeDeltaUnit) -> f64 {
        self.0.to(unit.0)
    }

    /// Returns the duration formatted in the given unit.
    ///
    /// Args:
    ///     unit: The unit to display the duration in.
    ///     format_spec: A format spec for the number, e.g. ".3f".
    #[pyo3(signature = (unit, format_spec = ""))]
    pub fn display_in(
        &self,
        py: Python<'_>,
        unit: PyTimeDeltaUnit,
        format_spec: &str,
    ) -> PyResult<String> {
        format_with_unit(py, self.0.to(unit.0), format_spec, unit.0.symbol())
    }

    /// Returns the negation of the `TimeDelta`.
    pub fn __neg__(&self) -> Self {
        Self(-self.0)
    }

    fn __pos__(&self) -> Self {
        self.clone()
    }

    fn __abs__(&self) -> Self {
        if self.0.is_negative() {
            Self(-self.0)
        } else {
            self.clone()
        }
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

    /// Returns the ratio of two `TimeDelta` values as a float, or the `TimeDelta` divided by a
    /// number.
    fn __truediv__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        if let Ok(rhs) = other.cast::<Self>() {
            return (self.0.as_seconds_f64() / rhs.get().0.as_seconds_f64()).into_py_any(py);
        }
        let Ok(divisor) = other.extract::<f64>() else {
            return Ok(py.NotImplemented());
        };
        if divisor == 0.0 {
            return Err(PyZeroDivisionError::new_err("TimeDelta division by zero"));
        }
        Self(
            self.0
                .try_mul(1.0 / divisor)
                .map_err(PyInvalidFloatSeconds)?,
        )
        .into_py_any(py)
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

    /// Create a TimeDelta from seconds.
    ///
    /// Integers are converted exactly.
    #[classmethod]
    pub fn from_seconds(_cls: &Bound<'_, PyType>, seconds: &Bound<'_, PyAny>) -> PyResult<Self> {
        from_number(seconds, TimeDeltaUnit::Seconds).map(Self)
    }

    /// Create a TimeDelta from minutes.
    #[classmethod]
    pub fn from_minutes(_cls: &Bound<'_, PyType>, minutes: &Bound<'_, PyAny>) -> PyResult<Self> {
        from_number(minutes, TimeDeltaUnit::Minutes).map(Self)
    }

    /// Create a TimeDelta from hours.
    #[classmethod]
    pub fn from_hours(_cls: &Bound<'_, PyType>, hours: &Bound<'_, PyAny>) -> PyResult<Self> {
        from_number(hours, TimeDeltaUnit::Hours).map(Self)
    }

    /// Create a TimeDelta from days (86400 seconds per day).
    #[classmethod]
    pub fn from_days(_cls: &Bound<'_, PyType>, days: &Bound<'_, PyAny>) -> PyResult<Self> {
        from_number(days, TimeDeltaUnit::Days).map(Self)
    }

    /// Create a TimeDelta from Julian years (365.25 days per year).
    #[classmethod]
    pub fn from_julian_years(_cls: &Bound<'_, PyType>, years: &Bound<'_, PyAny>) -> PyResult<Self> {
        from_number(years, TimeDeltaUnit::JulianYears).map(Self)
    }

    /// Create a TimeDelta from Julian centuries (36525 days per century).
    #[classmethod]
    pub fn from_julian_centuries(
        _cls: &Bound<'_, PyType>,
        centuries: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        from_number(centuries, TimeDeltaUnit::JulianCenturies).map(Self)
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
    ///     start: Start of the range (inclusive).
    ///     end: End of the range (inclusive if it falls on a step).
    ///     step: Non-zero step size. Defaults to one second. Its sign is adjusted to the
    ///         direction from `start` to `end`.
    ///
    /// Returns:
    ///     A list of TimeDelta objects.
    ///
    /// Raises:
    ///     ValueError: If `step` is zero, or if the range has more than 10,000,000 values.
    ///
    /// Examples:
    ///     >>> deltas = lox.TimeDelta.range(0 * lox.seconds, 10 * lox.seconds, 2 * lox.seconds)
    #[classmethod]
    #[pyo3(signature = (start, end, step=None))]
    pub fn range(
        _cls: &Bound<'_, PyType>,
        start: PyTimeDelta,
        end: PyTimeDelta,
        step: Option<PyTimeDelta>,
    ) -> PyResult<Vec<Self>> {
        let step = step.map_or(TimeDelta::from_seconds(1), |step| step.0);
        if step.is_zero() {
            return Err(PyValueError::new_err("step must be non-zero"));
        }
        let too_long = end.0.checked_sub(start.0).is_none_or(|span| {
            (span.as_seconds_f64() / step.as_seconds_f64()).abs() >= MAX_RANGE_LEN as f64
        });
        if too_long {
            return Err(PyValueError::new_err(format!(
                "range would have more than {MAX_RANGE_LEN} values"
            )));
        }
        Ok(TimeDeltaInterval::new(start.0, end.0)
            .step_by(step)
            .map(Self)
            .collect())
    }

    /// Convert to seconds.
    ///
    /// Returns:
    ///     The duration as a float in seconds.
    pub fn to_seconds(&self) -> f64 {
        self.0.as_seconds_f64()
    }
}

/// Creates a `TimeDelta` from a number in the given unit, exactly if the number is an integer.
///
/// Returns `None` if `value` is not a number, including NumPy arrays, so that NumPy can apply
/// the operation elementwise.
fn new_in(value: &Bound<'_, PyAny>, unit: TimeDeltaUnit) -> PyResult<Option<TimeDelta>> {
    if value.is_instance_of::<PyFloat>() {
        return float_in(value, unit).map(Some);
    }
    if value.is_instance_of::<PyUntypedArray>() {
        return Ok(None);
    }
    // Integer types such as `int` and the NumPy integer scalars implement `__index__`.
    if value.is_instance_of::<PyInt>() || value.get_type().hasattr("__index__")? {
        let count: i128 = value.extract().map_err(|err: PyErr| {
            if err.is_instance_of::<PyOverflowError>(value.py()) {
                out_of_range(value, unit)
            } else {
                err
            }
        })?;
        return TimeDelta::checked_new_in(count, unit)
            .map(Some)
            .ok_or_else(|| out_of_range(value, unit));
    }
    match float_in(value, unit) {
        Ok(delta) => Ok(Some(delta)),
        Err(err) if err.is_instance_of::<PyTypeError>(value.py()) => Ok(None),
        Err(err) => Err(err),
    }
}

/// Creates a `TimeDelta` from a value that converts to a float.
fn float_in(value: &Bound<'_, PyAny>, unit: TimeDeltaUnit) -> PyResult<TimeDelta> {
    TimeDelta::try_new_in(value.extract()?, unit).map_err(|_| out_of_range(value, unit))
}

fn out_of_range(value: &Bound<'_, PyAny>, unit: TimeDeltaUnit) -> PyErr {
    PyValueError::new_err(format!(
        "value {value} {unit} is non-finite or out-of-range"
    ))
}

/// Like [`new_in`], but raises a `TypeError` if `value` is not a number.
fn from_number(value: &Bound<'_, PyAny>, unit: TimeDeltaUnit) -> PyResult<TimeDelta> {
    new_in(value, unit)?.ok_or_else(|| {
        PyTypeError::new_err(format!(
            "expected a number, got '{}'",
            value
                .get_type()
                .name()
                .map_or_else(|_| "?".into(), |name| name.to_string())
        ))
    })
}

pub(crate) struct PyInvalidFloatSeconds(pub InvalidFloatSeconds);

impl From<PyInvalidFloatSeconds> for PyErr {
    fn from(err: PyInvalidFloatSeconds) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}
