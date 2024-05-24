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
use pyo3::{pyclass, pymethods, PyErr, PyResult};

impl From<UtcError> for PyErr {
    fn from(value: UtcError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

#[pyclass(name = "UTC", module = "lox_space")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyUtc(pub Utc);

#[pymethods]
impl PyUtc {
    #[new]
    #[pyo3(signature = (year, month, day, hour = 0, minute = 0, seconds = 0.0))]
    fn new(year: i64, month: u8, day: u8, hour: u8, minute: u8, seconds: f64) -> PyResult<PyUtc> {
        let utc = Utc::builder()
            .with_ymd(year, month, day)
            .with_hms(hour, minute, seconds)
            .build()?;
        Ok(PyUtc(utc))
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
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

    fn __eq__(&self, other: PyUtc) -> bool {
        self.0 == other.0
    }

    fn year(&self) -> i64 {
        self.0.year()
    }

    fn month(&self) -> u8 {
        self.0.month()
    }

    fn day(&self) -> u8 {
        self.0.day()
    }

    fn hour(&self) -> u8 {
        self.0.hour()
    }

    fn minute(&self) -> u8 {
        self.0.minute()
    }

    fn second(&self) -> u8 {
        self.0.second()
    }

    fn millisecond(&self) -> i64 {
        self.0.millisecond()
    }

    fn microsecond(&self) -> i64 {
        self.0.microsecond()
    }

    fn nanosecond(&self) -> i64 {
        self.0.nanosecond()
    }

    fn picosecond(&self) -> i64 {
        self.0.picosecond()
    }

    fn decimal_seconds(&self) -> f64 {
        self.0.decimal_seconds()
    }

    fn to_tai(&self) -> PyTime {
        PyTime(self.0.to_tai().with_scale(PyTimeScale::Tai))
    }

    fn to_tcb(&self) -> PyTime {
        PyTime(self.0.to_tcb().with_scale(PyTimeScale::Tcb))
    }

    fn to_tcg(&self) -> PyTime {
        PyTime(self.0.to_tcg().with_scale(PyTimeScale::Tcg))
    }

    fn to_tdb(&self) -> PyTime {
        PyTime(self.0.to_tdb().with_scale(PyTimeScale::Tdb))
    }

    fn to_tt(&self) -> PyTime {
        PyTime(self.0.to_tt().with_scale(PyTimeScale::Tt))
    }

    fn to_ut1(&self, provider: PyUt1Provider) -> PyResult<PyTime> {
        Ok(PyTime(
            self.0.try_to_ut1(&provider.0)?.with_scale(PyTimeScale::Ut1),
        ))
    }
}
