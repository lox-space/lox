use pyo3::exceptions::PyValueError;
use pyo3::types::PyType;
use pyo3::{pyclass, pymethods, Bound, PyErr, PyResult};

use crate::deltas::{TimeDelta, TimeDeltaError};

impl From<TimeDeltaError> for PyErr {
    fn from(value: TimeDeltaError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

#[pyclass(name = "TimeDelta", module = "lox_space")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PyTimeDelta(pub TimeDelta);

#[pymethods]
impl PyTimeDelta {
    #[new]
    fn new(seconds: f64) -> PyResult<Self> {
        Ok(Self(TimeDelta::from_decimal_seconds(seconds)?))
    }

    fn __repr__(&self) -> String {
        format!("TimeDelta({})", self.to_decimal_seconds())
    }

    fn __str__(&self) -> String {
        format!("{} seconds", self.to_decimal_seconds())
    }

    fn __float__(&self) -> f64 {
        self.to_decimal_seconds()
    }

    fn __neg__(&self) -> Self {
        Self(-self.0)
    }

    fn __add__(&self, other: PyTimeDelta) -> Self {
        Self(self.0 + other.0)
    }

    fn __sub__(&self, other: PyTimeDelta) -> Self {
        Self(self.0 - other.0)
    }

    fn seconds(&self) -> i64 {
        self.0.seconds
    }

    fn subsecond(&self) -> f64 {
        self.0.subsecond.0
    }

    #[classmethod]
    fn from_seconds(_cls: &Bound<'_, PyType>, seconds: i64) -> Self {
        Self(TimeDelta::from_seconds(seconds))
    }

    #[classmethod]
    fn from_minutes(_cls: &Bound<'_, PyType>, minutes: f64) -> PyResult<Self> {
        Ok(Self(TimeDelta::from_minutes(minutes)?))
    }

    #[classmethod]
    fn from_hours(_cls: &Bound<'_, PyType>, hours: f64) -> PyResult<Self> {
        Ok(Self(TimeDelta::from_hours(hours)?))
    }

    #[classmethod]
    fn from_days(_cls: &Bound<'_, PyType>, days: f64) -> PyResult<Self> {
        Ok(Self(TimeDelta::from_days(days)?))
    }

    #[classmethod]
    fn from_julian_years(_cls: &Bound<'_, PyType>, years: f64) -> PyResult<Self> {
        Ok(Self(TimeDelta::from_julian_years(years)?))
    }

    #[classmethod]
    fn from_julian_centuries(_cls: &Bound<'_, PyType>, centuries: f64) -> PyResult<Self> {
        Ok(Self(TimeDelta::from_julian_centuries(centuries)?))
    }

    fn to_decimal_seconds(&self) -> f64 {
        self.0.to_decimal_seconds()
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use pyo3::Python;

    use super::*;

    #[test]
    fn test_pytimedelta_repr() {
        let td = PyTimeDelta::new(123.456).unwrap();
        assert_eq!(td.__repr__(), "TimeDelta(123.456)");
        assert_eq!(td.__str__(), "123.456 seconds");
    }

    #[test]
    fn test_pytimedelta_ops() {
        let td1 = PyTimeDelta::new(123.456).unwrap();
        let td2 = PyTimeDelta::new(654.321).unwrap();
        assert_eq!(td1.__add__(td2).to_decimal_seconds(), 777.777);
        assert_eq!(td1.__sub__(td2).to_decimal_seconds(), -530.865);
        assert_eq!(td1.__neg__().to_decimal_seconds(), -123.456);
        assert_eq!(td1.__float__(), 123.456);
    }

    #[test]
    fn test_pytimedelta_seconds() {
        let td = PyTimeDelta::new(123.456).unwrap();
        assert_eq!(td.seconds(), 123);
        assert_float_eq!(td.subsecond(), 0.456, abs <= 1e-14);
    }

    #[test]
    fn test_pytimedelta_constructors() {
        Python::with_gil(|py| {
            let cls = PyType::new_bound::<PyTimeDelta>(py);
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
}
