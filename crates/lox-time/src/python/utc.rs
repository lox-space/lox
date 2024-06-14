/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::calendar_dates::CalendarDate;
use crate::prelude::CivilTime;
use crate::python::time::PyTime;
use crate::python::time_scales::PyTimeScale;
use crate::python::ut1::PyUt1Provider;
use crate::transformations::{ToTai, ToTcb, ToTcg, ToTdb, ToTt, ToUt1};
use crate::utc::{Utc, UtcError};
use pyo3::exceptions::PyValueError;
use pyo3::types::PyType;
use pyo3::{pyclass, pymethods, Bound, PyErr, PyResult};

impl From<UtcError> for PyErr {
    fn from(value: UtcError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

#[pyclass(name = "UTC", module = "lox_space", frozen)]
#[derive(Clone, Debug, Eq, PartialEq)]
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
            .build()?;
        Ok(PyUtc(utc))
    }

    #[classmethod]
    pub fn from_iso(_cls: &Bound<'_, PyType>, iso: &str) -> PyResult<PyUtc> {
        Ok(PyUtc(iso.parse()?))
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
            self.0.decimal_seconds()
        )
    }

    pub fn __eq__(&self, other: PyUtc) -> bool {
        self.0 == other.0
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

    pub fn millisecond(&self) -> i64 {
        self.0.millisecond()
    }

    pub fn microsecond(&self) -> i64 {
        self.0.microsecond()
    }

    pub fn nanosecond(&self) -> i64 {
        self.0.nanosecond()
    }

    pub fn picosecond(&self) -> i64 {
        self.0.picosecond()
    }

    pub fn decimal_seconds(&self) -> f64 {
        self.0.decimal_seconds()
    }

    pub fn to_tai(&self) -> PyTime {
        PyTime(self.0.to_tai().with_scale(PyTimeScale::Tai))
    }

    pub fn to_tcb(&self) -> PyTime {
        PyTime(self.0.to_tcb().with_scale(PyTimeScale::Tcb))
    }

    pub fn to_tcg(&self) -> PyTime {
        PyTime(self.0.to_tcg().with_scale(PyTimeScale::Tcg))
    }

    pub fn to_tdb(&self) -> PyTime {
        PyTime(self.0.to_tdb().with_scale(PyTimeScale::Tdb))
    }

    pub fn to_tt(&self) -> PyTime {
        PyTime(self.0.to_tt().with_scale(PyTimeScale::Tt))
    }

    pub fn to_ut1(&self, provider: &Bound<'_, PyUt1Provider>) -> PyResult<PyTime> {
        Ok(PyTime(
            self.0
                .try_to_ut1(&provider.borrow().0)?
                .with_scale(PyTimeScale::Ut1),
        ))
    }
}

#[cfg(test)]
mod tests {
    use pyo3::{Bound, Python};

    use crate::test_helpers::data_dir;

    use super::*;

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
        Python::with_gil(|py| {
            let cls = PyType::new_bound::<PyUtc>(py);
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
        Python::with_gil(|py| {
            let cls = PyType::new_bound::<PyUtc>(py);
            let _ = PyUtc::from_iso(&cls, "2000-01-01X00:00:00 UTC").unwrap();
        })
    }

    #[test]
    fn test_pyutc_transformations() {
        Python::with_gil(|py| {
            let provider = Bound::new(
                py,
                PyUt1Provider::new(data_dir().join("finals2000A.all.csv").to_str().unwrap())
                    .unwrap(),
            )
            .unwrap();
            let utc_exp = PyUtc::new(2000, 1, 1, 0, 0, 0.0).unwrap();
            let utc_act = utc_exp.to_tai().to_utc(Some(&provider)).unwrap();
            assert_eq!(utc_act, utc_exp);
            let utc_act = utc_exp.to_tcb().to_utc(Some(&provider)).unwrap();
            assert_eq!(utc_act, utc_exp);
            let utc_act = utc_exp.to_tcg().to_utc(Some(&provider)).unwrap();
            assert_eq!(utc_act, utc_exp);
            let utc_act = utc_exp.to_tdb().to_utc(Some(&provider)).unwrap();
            assert_eq!(utc_act, utc_exp);
            let utc_act = utc_exp.to_tt().to_utc(Some(&provider)).unwrap();
            assert_eq!(utc_act, utc_exp);
            let utc_act = utc_exp
                .to_ut1(&provider)
                .unwrap()
                .to_utc(Some(&provider))
                .unwrap();
            assert_eq!(utc_act, utc_exp);
        });
    }
}
