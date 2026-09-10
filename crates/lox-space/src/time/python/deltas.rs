// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_time::deltas::InvalidFloatSeconds;
use numpy::PyArray;
use numpy::ndarray::arr0;
use pyo3::exceptions::PyValueError;
use pyo3::types::{PyAnyMethods, PyFloat, PyModule, PyModuleMethods, PyType};
use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, intern, pyclass, pymethods, types::PyTuple};

use crate::time::deltas::TimeDelta;
use crate::time::intervals::TimeDeltaInterval;
use crate::units::python::{Scalar, format_in_unit, split_unit_token};

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

impl PyTimeDelta {
    /// Constructs a `TimeDelta` from a duration in seconds.
    ///
    /// The Rust-side counterpart of the Python constructor, which takes a
    /// [`Scalar`] so that a quantity cannot be coerced into it.
    pub fn new(seconds: f64) -> PyResult<Self> {
        Ok(Self(
            TimeDelta::try_from_seconds_f64(seconds).map_err(PyInvalidFloatSeconds)?,
        ))
    }
}

#[pymethods]
impl PyTimeDelta {
    #[new]
    /// Constructs a `TimeDelta` from a duration in seconds.
    fn py_new(seconds: Scalar) -> PyResult<Self> {
        Self::new(seconds.0)
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

    /// Returns the `TimeDelta` scaled by a scalar.
    pub fn __mul__(&self, other: Scalar) -> PyResult<Self> {
        Ok(Self(
            self.0.try_mul(other.0).map_err(PyInvalidFloatSeconds)?,
        ))
    }

    /// Returns the `TimeDelta` scaled by a scalar (right-hand side).
    pub fn __rmul__(&self, other: Scalar) -> PyResult<Self> {
        Ok(Self(
            self.0.try_mul(other.0).map_err(PyInvalidFloatSeconds)?,
        ))
    }

    /// Returns the magnitude of the `TimeDelta`.
    pub fn __abs__(&self) -> Self {
        if self.0 < TimeDelta::default() {
            Self(-self.0)
        } else {
            Self(self.0)
        }
    }

    /// Divides by a scalar, or by another `TimeDelta` or a time unit for a
    /// plain number.
    pub fn __truediv__<'py>(
        &self,
        py: Python<'py>,
        other: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        use pyo3::IntoPyObject;
        if let Ok(delta) = other.extract::<PyTimeDelta>() {
            let ratio = self.to_decimal_seconds() / delta.to_decimal_seconds();
            return Ok(ratio.into_pyobject(py)?.into_any());
        }
        if let Ok(unit) = other.extract::<PyTimeDeltaUnit>() {
            let converted = self.to_decimal_seconds() / unit.scale;
            return Ok(converted.into_pyobject(py)?.into_any());
        }
        match other.extract::<Scalar>() {
            Ok(scalar) => {
                let scaled = Self(
                    self.0
                        .try_mul(1.0 / scalar.0)
                        .map_err(PyInvalidFloatSeconds)?,
                );
                Ok(scaled.into_pyobject(py)?.into_any())
            }
            Err(_) => Ok(py.NotImplemented().into_bound(py)),
        }
    }

    /// Rounds the duration to the given number of decimal seconds.
    #[pyo3(signature = (ndigits=None))]
    pub fn __round__(&self, py: Python<'_>, ndigits: Option<i32>) -> PyResult<Self> {
        let rounded: f64 = PyFloat::new(py, self.to_decimal_seconds())
            .call_method1(intern!(py, "__round__"), (ndigits,))?
            .extract()?;
        Self::new(rounded)
    }

    /// Returns `true` if this `TimeDelta` is shorter than `other`.
    pub fn __lt__(&self, other: PyTimeDelta) -> bool {
        self.0 < other.0
    }

    /// Returns `true` if this `TimeDelta` is shorter than or equal to `other`.
    pub fn __le__(&self, other: PyTimeDelta) -> bool {
        self.0 <= other.0
    }

    /// Returns `true` if this `TimeDelta` is longer than `other`.
    pub fn __gt__(&self, other: PyTimeDelta) -> bool {
        self.0 > other.0
    }

    /// Returns `true` if this `TimeDelta` is longer than or equal to `other`.
    pub fn __ge__(&self, other: PyTimeDelta) -> bool {
        self.0 >= other.0
    }

    /// Hashes the exact duration, consistently with `__eq__`.
    ///
    /// Hashing the two integer components rather than the decimal seconds
    /// keeps attosecond-distinct deltas distinct.
    pub fn __hash__(&self, py: Python<'_>) -> PyResult<isize> {
        PyTuple::new(py, [self.0.seconds(), self.0.attoseconds()])?.hash()
    }

    /// Renders the duration using a Python format spec.
    ///
    /// The default unit is seconds; a trailing unit name selects another, as
    /// in `f"{duration:.1f minutes}"`.
    pub fn __format__(&self, py: Python<'_>, spec: &str) -> PyResult<String> {
        let (spec, unit) = split_unit_token(spec);
        match unit {
            None => format_in_unit(py, self.to_decimal_seconds(), "seconds", spec),
            Some(symbol) => {
                let unit = PyTimeDeltaUnit::lookup(symbol)
                    .ok_or_else(|| PyTimeDeltaUnit::unknown(symbol))?;
                format_in_unit(
                    py,
                    self.to_decimal_seconds() / unit.scale,
                    unit.symbol,
                    spec,
                )
            }
        }
    }

    /// Returns the duration in seconds as a 0-d float64 NumPy array.
    #[pyo3(signature = (dtype=None, copy=None))]
    pub fn __array__<'py>(
        &self,
        py: Python<'py>,
        dtype: Option<&Bound<'py, PyAny>>,
        copy: Option<bool>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = copy;
        let array = PyArray::from_owned_array(py, arr0(self.to_decimal_seconds())).into_any();
        match dtype {
            None => Ok(array),
            Some(dtype) => array.call_method1(intern!(py, "astype"), (dtype,)),
        }
    }

    /// Opts out of NumPy's ufunc machinery so scalar arithmetic keeps the type.
    #[allow(non_upper_case_globals)]
    #[classattr]
    const __array_ufunc__: Option<Py<PyAny>> = None;

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
    pub fn from_minutes(_cls: &Bound<'_, PyType>, minutes: Scalar) -> PyResult<Self> {
        Ok(Self(
            TimeDelta::try_from_minutes_f64(minutes.0).map_err(PyInvalidFloatSeconds)?,
        ))
    }

    /// Create a TimeDelta from hours.
    #[classmethod]
    pub fn from_hours(_cls: &Bound<'_, PyType>, hours: Scalar) -> PyResult<Self> {
        Ok(Self(
            TimeDelta::try_from_hours_f64(hours.0).map_err(PyInvalidFloatSeconds)?,
        ))
    }

    /// Create a TimeDelta from days (86400 seconds per day).
    #[classmethod]
    pub fn from_days(_cls: &Bound<'_, PyType>, days: Scalar) -> PyResult<Self> {
        Ok(Self(
            TimeDelta::try_from_days_f64(days.0).map_err(PyInvalidFloatSeconds)?,
        ))
    }

    /// Create a TimeDelta from Julian years (365.25 days per year).
    #[classmethod]
    pub fn from_julian_years(_cls: &Bound<'_, PyType>, years: Scalar) -> PyResult<Self> {
        Ok(Self(
            TimeDelta::try_from_julian_years(years.0).map_err(PyInvalidFloatSeconds)?,
        ))
    }

    /// Create a TimeDelta from Julian centuries (36525 days per century).
    #[classmethod]
    pub fn from_julian_centuries(_cls: &Bound<'_, PyType>, centuries: Scalar) -> PyResult<Self> {
        Ok(Self(
            TimeDelta::try_from_julian_centuries(centuries.0).map_err(PyInvalidFloatSeconds)?,
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

// --- TimeDeltaUnit ---

/// A unit a `TimeDelta` can be expressed in.
///
/// Multiplying by a number produces a `TimeDelta`, and dividing a `TimeDelta`
/// by one converts it: `5 * lox.minutes` is a `TimeDelta`, and
/// `duration / lox.minutes` is that duration in minutes.
#[pyclass(name = "TimeDeltaUnit", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone, Copy)]
pub struct PyTimeDeltaUnit {
    symbol: &'static str,
    scale: f64,
}

impl PyTimeDeltaUnit {
    /// The units a duration can be expressed in, as `(name, seconds per unit)`.
    const UNITS: &'static [(&'static str, f64)] = &[
        ("seconds", 1.0),
        ("minutes", 60.0),
        ("hours", 3600.0),
        ("days", 86400.0),
    ];

    pub(crate) fn lookup(symbol: &str) -> Option<Self> {
        Self::UNITS
            .iter()
            .find(|(name, _)| *name == symbol)
            .map(|&(symbol, scale)| Self { symbol, scale })
    }

    pub(crate) fn unknown(symbol: &str) -> PyErr {
        let known: Vec<&str> = Self::UNITS.iter().map(|(name, _)| *name).collect();
        PyValueError::new_err(format!(
            "'{}' is not a TimeDeltaUnit; expected one of {}",
            symbol,
            known.join(", ")
        ))
    }

    /// Registers the class and its unit constants on the module.
    pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_class::<PyTimeDeltaUnit>()?;
        for &(symbol, scale) in Self::UNITS {
            m.add(symbol, Self { symbol, scale })?;
        }
        Ok(())
    }
}

#[pymethods]
impl PyTimeDeltaUnit {
    #[new]
    /// Looks up a `TimeDeltaUnit` by name, e.g. `"minutes"`.
    fn new(symbol: &str) -> PyResult<Self> {
        Self::lookup(symbol).ok_or_else(|| Self::unknown(symbol))
    }

    /// The name this unit is exported under, e.g. `"minutes"`.
    #[getter]
    fn symbol(&self) -> &'static str {
        self.symbol
    }

    /// The number of seconds in one of this unit.
    #[getter]
    fn scale(&self) -> f64 {
        self.scale
    }

    /// Scales this unit into a `TimeDelta`.
    fn __mul__(&self, other: Scalar) -> PyResult<PyTimeDelta> {
        PyTimeDelta::new(other.0 * self.scale)
    }

    /// Scales this unit into a `TimeDelta` (right-hand side).
    fn __rmul__(&self, other: Scalar) -> PyResult<PyTimeDelta> {
        PyTimeDelta::new(other.0 * self.scale)
    }

    fn __eq__(&self, other: &PyTimeDeltaUnit) -> bool {
        self.symbol == other.symbol
    }

    fn __hash__(&self, py: Python<'_>) -> PyResult<isize> {
        pyo3::types::PyString::new(py, self.symbol).hash()
    }

    fn __getnewargs__(&self) -> (&'static str,) {
        (self.symbol,)
    }

    fn __repr__(&self) -> String {
        format!("TimeDeltaUnit(\"{}\")", self.symbol)
    }

    fn __str__(&self) -> &'static str {
        self.symbol
    }
}
