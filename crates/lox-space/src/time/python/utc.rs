// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::earth::python::ut1::{PyEopProvider, PyEopProviderError};
use crate::time::calendar_dates::CalendarDate;
use crate::time::python::time::PyTime;
use crate::time::python::time_scales::PyTimeScale;
use crate::time::time_of_day::CivilTime;
use crate::time::utc::{Utc, UtcError};
use lox_test_utils::{ApproxEq, approx_eq};
use pyo3::exceptions::PyValueError;
use pyo3::types::PyType;
use pyo3::{Bound, PyAny, PyErr, PyResult, pyclass, pymethods};

pub struct PyUtcError(pub UtcError);

impl From<PyUtcError> for PyErr {
    fn from(value: PyUtcError) -> Self {
        PyValueError::new_err(value.0.to_string())
    }
}

#[pyclass(name = "UTC", module = "lox_space", frozen)]
#[derive(Clone, Debug, Eq, PartialEq, ApproxEq)]
pub struct PyUtc(pub Utc);

#[pymethods]
impl PyUtc {
    #[new]
    #[pyo3(signature = (year, month, day, hour = 0, minute = 0, seconds = 0.0))]
    pub fn new(
        year: i64,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        seconds: f64,
    ) -> PyResult<PyUtc> {
        let utc = Utc::builder()
            .with_ymd(year, month, day)
            .with_hms(hour, minute, seconds)
            .build()
            .map_err(PyUtcError)?;
        Ok(PyUtc(utc))
    }

    #[classmethod]
    pub fn from_iso(_cls: &Bound<'_, PyType>, iso: &str) -> PyResult<PyUtc> {
        Ok(PyUtc(iso.parse().map_err(PyUtcError)?))
    }

    pub fn __str__(&self) -> String {
        self.0.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!(
            "UTC({}, {}, {}, {}, {}, {})",
            self.0.year(),
            self.0.month(),
            self.0.day(),
            self.0.hour(),
            self.0.minute(),
            self.0.as_seconds_f64()
        )
    }

    pub fn __eq__(&self, other: PyUtc) -> bool {
        self.0 == other.0
    }

    #[pyo3(signature = (other, rel_tol = 1e-8, abs_tol = 1e-14))]
    pub fn isclose(&self, other: PyUtc, rel_tol: f64, abs_tol: f64) -> bool {
        approx_eq!(self, &other, rtol <= rel_tol, atol <= abs_tol)
    }

    pub fn year(&self) -> i64 {
        self.0.year()
    }

    pub fn month(&self) -> u8 {
        self.0.month()
    }

    pub fn day(&self) -> u8 {
        self.0.day()
    }

    pub fn hour(&self) -> u8 {
        self.0.hour()
    }

    pub fn minute(&self) -> u8 {
        self.0.minute()
    }

    pub fn second(&self) -> u8 {
        self.0.second()
    }

    pub fn millisecond(&self) -> u32 {
        self.0.millisecond()
    }

    pub fn microsecond(&self) -> u32 {
        self.0.microsecond()
    }

    pub fn nanosecond(&self) -> u32 {
        self.0.nanosecond()
    }

    pub fn picosecond(&self) -> u32 {
        self.0.picosecond()
    }

    pub fn decimal_seconds(&self) -> f64 {
        self.0.as_seconds_f64()
    }

    #[pyo3(signature = (scale, provider=None))]
    pub fn to_scale(
        &self,
        scale: &Bound<'_, PyAny>,
        provider: Option<&Bound<'_, PyEopProvider>>,
    ) -> PyResult<PyTime> {
        let scale: PyTimeScale = scale.try_into()?;
        let provider = provider.map(|p| &p.get().0);
        let time = match provider {
            Some(provider) => self
                .0
                .to_dyn_time()
                .try_to_scale(scale.0, provider)
                .map_err(PyEopProviderError)?,
            None => self.0.to_dyn_time().to_scale(scale.0),
        };
        Ok(PyTime(time))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use lox_test_utils::{assert_approx_eq, data_file};

    use pyo3::{Bound, IntoPyObject, IntoPyObjectExt, Python};
    use rstest::rstest;

    #[test]
    fn test_pyutc() {
        let utc = PyUtc::new(2000, 1, 1, 12, 13, 14.123456789123).unwrap();
        assert_eq!(utc.year(), 2000);
        assert_eq!(utc.month(), 1);
        assert_eq!(utc.day(), 1);
        assert_eq!(utc.hour(), 12);
        assert_eq!(utc.minute(), 13);
        assert_eq!(utc.second(), 14);
        assert_eq!(utc.millisecond(), 123);
        assert_eq!(utc.microsecond(), 456);
        assert_eq!(utc.nanosecond(), 789);
        assert_eq!(utc.picosecond(), 123);
        assert_eq!(utc.decimal_seconds(), 14.123456789123);
        assert_eq!(utc.__str__(), "2000-01-01T12:13:14.123 UTC");
        assert_eq!(utc.__repr__(), "UTC(2000, 1, 1, 12, 13, 14.123456789123)");
        assert!(utc.__eq__(utc.clone()));
    }

    #[test]
    #[should_panic(expected = "invalid date")]
    fn test_pyutc_error() {
        PyUtc::new(2000, 0, 1, 0, 0, 0.0).unwrap();
    }

    #[test]
    fn test_pytime_from_iso() {
        Python::attach(|py| {
            let cls = PyType::new::<PyUtc>(py);
            let expected = PyUtc::new(2000, 1, 1, 0, 0, 0.0).unwrap();
            let actual = PyUtc::from_iso(&cls, "2000-01-01T00:00:00 UTC").unwrap();
            assert_eq!(actual, expected);
            let actual = PyUtc::from_iso(&cls, "2000-01-01T00:00:00Z").unwrap();
            assert_eq!(actual, expected);
            let actual = PyUtc::from_iso(&cls, "2000-01-01T00:00:00").unwrap();
            assert_eq!(actual, expected);
        })
    }

    #[test]
    #[should_panic(expected = "invalid ISO")]
    fn test_pytime_from_iso_invalid() {
        Python::attach(|py| {
            let cls = PyType::new::<PyUtc>(py);
            let _ = PyUtc::from_iso(&cls, "2000-01-01X00:00:00 UTC").unwrap();
        })
    }

    #[rstest]
    #[case::tai("TAI")]
    #[case::tcb("TCB")]
    #[case::tcg("TCG")]
    #[case::tdb("TDB")]
    #[case::tt("TT")]
    #[case::ut1("UT1")]
    fn test_pyutc_transformations(#[case] scale: &str) {
        Python::attach(|py| {
            let path = (data_file("iers/finals2000A.all.csv"),)
                .into_pyobject(py)
                .unwrap();
            let provider = Bound::new(py, PyEopProvider::new(&path).unwrap()).unwrap();
            let scale = scale.into_bound_py_any(py).unwrap();
            let exp = PyUtc::new(2000, 1, 1, 0, 0, 0.0).unwrap();
            let act = exp
                .to_scale(&scale, Some(&provider))
                .unwrap()
                .to_utc(Some(&provider))
                .unwrap();
            assert_approx_eq!(act.0, exp.0);
        });
    }
}
