/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::calendar_dates::CalendarDate;
use crate::deltas::{TimeDelta, ToDelta};
use crate::julian_dates::{Epoch, JulianDate, Unit};
use crate::prelude::{CivilTime, Tai, Tcb, Tcg, Tdb, TimeScale, Tt, Ut1};
use crate::python::deltas::PyTimeDelta;
use crate::python::time_scales::PyTimeScale;
use crate::python::ut1::PyUt1Provider;
use crate::python::utc::PyUtc;
use crate::transformations::{NoOpOffsetProvider, ToTai, ToTcb, ToTcg, ToTdb, ToTt, TryToScale};
use crate::ut1::{DeltaUt1Tai, ExtrapolatedDeltaUt1Tai};
use crate::utc::transformations::ToUtc;
use crate::{Time, TimeError};
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::types::{PyAnyMethods, PyType};
use pyo3::{pyclass, pymethods, Bound, PyAny, PyErr, PyResult, Python};
use std::str::FromStr;

impl From<TimeError> for PyErr {
    fn from(value: TimeError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

impl FromStr for Epoch {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "jd" | "JD" => Ok(Epoch::JulianDate),
            "mjd" | "MJD" => Ok(Epoch::ModifiedJulianDate),
            "j1950" | "J1950" => Ok(Epoch::J1950),
            "j2000" | "J2000" => Ok(Epoch::J2000),
            _ => Err(PyValueError::new_err(format!("unknown epoch: {}", s))),
        }
    }
}

impl FromStr for Unit {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "seconds" => Ok(Unit::Seconds),
            "days" => Ok(Unit::Days),
            "centuries" => Ok(Unit::Centuries),
            _ => Err(PyValueError::new_err(format!("unknown unit: {}", s))),
        }
    }
}

#[pyclass(name = "Time", module = "lox_space")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyTime(pub Time<PyTimeScale>);

#[pymethods]
impl PyTime {
    #[new]
    #[pyo3(signature=(scale, year, month, day, hour = 0, minute = 0, seconds = 0.0))]
    pub fn new(
        scale: &str,
        year: i64,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        seconds: f64,
    ) -> PyResult<PyTime> {
        let scale: PyTimeScale = scale.parse()?;
        let time = Time::builder_with_scale(scale)
            .with_ymd(year, month, day)
            .with_hms(hour, minute, seconds)
            .build()?;
        Ok(PyTime(time))
    }

    #[classmethod]
    #[pyo3(signature = (scale, jd, epoch = "jd"))]
    pub fn from_julian_date(
        _cls: &Bound<'_, PyType>,
        scale: &str,
        jd: f64,
        epoch: &str,
    ) -> PyResult<Self> {
        let scale: PyTimeScale = scale.parse()?;
        let epoch: Epoch = epoch.parse()?;
        Ok(Self(Time::from_julian_date(scale, jd, epoch)?))
    }

    pub fn __str__(&self) -> String {
        self.0.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Time(\"{}\", {}, {}, {}, {}, {}, {})",
            self.scale(),
            self.0.year(),
            self.0.month(),
            self.0.day(),
            self.0.hour(),
            self.0.minute(),
            self.0.decimal_seconds(),
        )
    }

    pub fn __add__(&self, delta: PyTimeDelta) -> Self {
        PyTime(self.0 + delta.0)
    }

    pub fn __sub__<'py>(
        &self,
        py: Python<'py>,
        rhs: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Ok(delta) = rhs.extract::<PyTimeDelta>() {
            Ok(Bound::new(py, PyTime(self.0 - delta.0))?.into_any())
        } else if let Ok(rhs) = rhs.extract::<PyTime>() {
            if self.scale() != rhs.scale() {
                return Err(PyValueError::new_err(
                    "cannot subtract `Time` objects with different time scales",
                ));
            }
            Ok(Bound::new(py, PyTimeDelta(self.0 - rhs.0))?.into_any())
        } else {
            Err(PyTypeError::new_err(
                "`rhs` must be either a `Time` or a `TimeDelta` object",
            ))
        }
    }

    pub fn __eq__(&self, rhs: PyTime) -> bool {
        self.0 == rhs.0
    }

    #[pyo3(signature = (epoch = "jd", unit = "days"))]
    pub fn julian_date(&self, epoch: &str, unit: &str) -> PyResult<f64> {
        let epoch: Epoch = epoch.parse()?;
        let unit: Unit = unit.parse()?;
        Ok(self.0.julian_date(epoch, unit))
    }

    pub fn scale(&self) -> &'static str {
        self.0.scale().abbreviation()
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

    pub fn femtosecond(&self) -> i64 {
        self.0.femtosecond()
    }

    pub fn decimal_seconds(&self) -> f64 {
        self.0.decimal_seconds()
    }

    pub fn to_tai(&self, provider: Option<&Bound<'_, PyUt1Provider>>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Tai, &provider.borrow().0)?,
            None => self.try_to_scale(Tai, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tai)))
    }

    pub fn to_tcb(&self, provider: Option<&Bound<'_, PyUt1Provider>>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Tcb, &provider.borrow().0)?,
            None => self.try_to_scale(Tcb, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tcb)))
    }

    pub fn to_tcg(&self, provider: Option<&Bound<'_, PyUt1Provider>>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Tcg, &provider.borrow().0)?,
            None => self.try_to_scale(Tcg, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tcg)))
    }

    pub fn to_tdb(&self, provider: Option<&Bound<'_, PyUt1Provider>>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Tdb, &provider.borrow().0)?,
            None => self.try_to_scale(Tdb, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tdb)))
    }

    pub fn to_tt(&self, provider: Option<&Bound<'_, PyUt1Provider>>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Tt, &provider.borrow().0)?,
            None => self.try_to_scale(Tt, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tt)))
    }

    pub fn to_ut1(&self, provider: Option<&Bound<'_, PyUt1Provider>>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Ut1, &provider.borrow().0)?,
            None => self.try_to_scale(Ut1, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Ut1)))
    }

    pub fn to_utc(&self, provider: Option<&Bound<'_, PyUt1Provider>>) -> PyResult<PyUtc> {
        let tai = match provider {
            Some(provider) => self.try_to_scale(Tai, &provider.borrow().0)?,
            None => self.try_to_scale(Tai, &NoOpOffsetProvider)?,
        };
        Ok(PyUtc(tai.to_utc()?))
    }
}

impl ToDelta for PyTime {
    fn to_delta(&self) -> TimeDelta {
        self.0.to_delta()
    }
}

impl TryToScale<Tai, DeltaUt1Tai, ExtrapolatedDeltaUt1Tai> for PyTime {
    fn try_to_scale(
        &self,
        _scale: Tai,
        provider: &DeltaUt1Tai,
    ) -> Result<Time<Tai>, ExtrapolatedDeltaUt1Tai> {
        match self.0.scale() {
            PyTimeScale::Tai => Ok(self.0.with_scale(Tai)),
            PyTimeScale::Tcb => Ok(self.0.with_scale(Tcb).to_tai()),
            PyTimeScale::Tcg => Ok(self.0.with_scale(Tcg).to_tai()),
            PyTimeScale::Tdb => Ok(self.0.with_scale(Tdb).to_tai()),
            PyTimeScale::Tt => Ok(self.0.with_scale(Tt).to_tai()),
            PyTimeScale::Ut1 => self.0.with_scale(Ut1).try_to_scale(Tai, provider),
        }
    }
}

impl TryToScale<Tai, NoOpOffsetProvider, PyErr> for PyTime {
    fn try_to_scale(&self, _scale: Tai, _provider: &NoOpOffsetProvider) -> PyResult<Time<Tai>> {
        match self.0.scale() {
            PyTimeScale::Tai => Ok(self.0.with_scale(Tai)),
            PyTimeScale::Tcb => Ok(self.0.with_scale(Tcb).to_tai()),
            PyTimeScale::Tcg => Ok(self.0.with_scale(Tcg).to_tai()),
            PyTimeScale::Tdb => Ok(self.0.with_scale(Tdb).to_tai()),
            PyTimeScale::Tt => Ok(self.0.with_scale(Tt).to_tai()),
            PyTimeScale::Ut1 => Err(PyValueError::new_err(
                "`provider` argument needs to be present for UT1 transformations",
            )),
        }
    }
}

impl TryToScale<Tcg, DeltaUt1Tai, ExtrapolatedDeltaUt1Tai> for PyTime {
    fn try_to_scale(
        &self,
        _scale: Tcg,
        provider: &DeltaUt1Tai,
    ) -> Result<Time<Tcg>, ExtrapolatedDeltaUt1Tai> {
        match self.0.scale() {
            PyTimeScale::Tai => Ok(self.0.with_scale(Tai).to_tcg()),
            PyTimeScale::Tcb => Ok(self.0.with_scale(Tcb).to_tcg()),
            PyTimeScale::Tcg => Ok(self.0.with_scale(Tcg)),
            PyTimeScale::Tdb => Ok(self.0.with_scale(Tdb).to_tcg()),
            PyTimeScale::Tt => Ok(self.0.with_scale(Tt).to_tcg()),
            PyTimeScale::Ut1 => self.0.with_scale(Ut1).try_to_scale(Tcg, provider),
        }
    }
}

impl TryToScale<Tcg, NoOpOffsetProvider, PyErr> for PyTime {
    fn try_to_scale(&self, _scale: Tcg, _provider: &NoOpOffsetProvider) -> PyResult<Time<Tcg>> {
        match self.0.scale() {
            PyTimeScale::Tai => Ok(self.0.with_scale(Tai).to_tcg()),
            PyTimeScale::Tcb => Ok(self.0.with_scale(Tcb).to_tcg()),
            PyTimeScale::Tcg => Ok(self.0.with_scale(Tcg)),
            PyTimeScale::Tdb => Ok(self.0.with_scale(Tdb).to_tcg()),
            PyTimeScale::Tt => Ok(self.0.with_scale(Tt).to_tcg()),
            PyTimeScale::Ut1 => Err(PyValueError::new_err(
                "`provider` argument needs to be present for UT1 transformations",
            )),
        }
    }
}

impl TryToScale<Tcb, DeltaUt1Tai, ExtrapolatedDeltaUt1Tai> for PyTime {
    fn try_to_scale(
        &self,
        _scale: Tcb,
        provider: &DeltaUt1Tai,
    ) -> Result<Time<Tcb>, ExtrapolatedDeltaUt1Tai> {
        match self.0.scale() {
            PyTimeScale::Tai => Ok(self.0.with_scale(Tai).to_tcb()),
            PyTimeScale::Tcb => Ok(self.0.with_scale(Tcb)),
            PyTimeScale::Tcg => Ok(self.0.with_scale(Tcg).to_tcb()),
            PyTimeScale::Tdb => Ok(self.0.with_scale(Tdb).to_tcb()),
            PyTimeScale::Tt => Ok(self.0.with_scale(Tt).to_tcb()),
            PyTimeScale::Ut1 => self.0.with_scale(Ut1).try_to_scale(Tcb, provider),
        }
    }
}

impl TryToScale<Tcb, NoOpOffsetProvider, PyErr> for PyTime {
    fn try_to_scale(&self, _scale: Tcb, _provider: &NoOpOffsetProvider) -> PyResult<Time<Tcb>> {
        match self.0.scale() {
            PyTimeScale::Tai => Ok(self.0.with_scale(Tai).to_tcb()),
            PyTimeScale::Tcb => Ok(self.0.with_scale(Tcb)),
            PyTimeScale::Tcg => Ok(self.0.with_scale(Tcg).to_tcb()),
            PyTimeScale::Tdb => Ok(self.0.with_scale(Tdb).to_tcb()),
            PyTimeScale::Tt => Ok(self.0.with_scale(Tt).to_tcb()),
            PyTimeScale::Ut1 => Err(PyValueError::new_err(
                "`provider` argument needs to be present for UT1 transformations",
            )),
        }
    }
}

impl TryToScale<Tdb, DeltaUt1Tai, ExtrapolatedDeltaUt1Tai> for PyTime {
    fn try_to_scale(
        &self,
        _scale: Tdb,
        provider: &DeltaUt1Tai,
    ) -> Result<Time<Tdb>, ExtrapolatedDeltaUt1Tai> {
        match self.0.scale() {
            PyTimeScale::Tai => Ok(self.0.with_scale(Tai).to_tdb()),
            PyTimeScale::Tcb => Ok(self.0.with_scale(Tcb).to_tdb()),
            PyTimeScale::Tcg => Ok(self.0.with_scale(Tcg).to_tdb()),
            PyTimeScale::Tdb => Ok(self.0.with_scale(Tdb)),
            PyTimeScale::Tt => Ok(self.0.with_scale(Tt).to_tdb()),
            PyTimeScale::Ut1 => self.0.with_scale(Ut1).try_to_scale(Tdb, provider),
        }
    }
}

impl TryToScale<Tdb, NoOpOffsetProvider, PyErr> for PyTime {
    fn try_to_scale(&self, _scale: Tdb, _provider: &NoOpOffsetProvider) -> PyResult<Time<Tdb>> {
        match self.0.scale() {
            PyTimeScale::Tai => Ok(self.0.with_scale(Tai).to_tdb()),
            PyTimeScale::Tcb => Ok(self.0.with_scale(Tcb).to_tdb()),
            PyTimeScale::Tcg => Ok(self.0.with_scale(Tcg).to_tdb()),
            PyTimeScale::Tdb => Ok(self.0.with_scale(Tdb)),
            PyTimeScale::Tt => Ok(self.0.with_scale(Tt).to_tdb()),
            PyTimeScale::Ut1 => Err(PyValueError::new_err(
                "`provider` argument needs to be present for UT1 transformations",
            )),
        }
    }
}

impl TryToScale<Tt, DeltaUt1Tai, ExtrapolatedDeltaUt1Tai> for PyTime {
    fn try_to_scale(
        &self,
        _scale: Tt,
        provider: &DeltaUt1Tai,
    ) -> Result<Time<Tt>, ExtrapolatedDeltaUt1Tai> {
        match self.0.scale() {
            PyTimeScale::Tai => Ok(self.0.with_scale(Tai).to_tt()),
            PyTimeScale::Tcb => Ok(self.0.with_scale(Tcb).to_tt()),
            PyTimeScale::Tcg => Ok(self.0.with_scale(Tcg).to_tt()),
            PyTimeScale::Tdb => Ok(self.0.with_scale(Tdb).to_tt()),
            PyTimeScale::Tt => Ok(self.0.with_scale(Tt)),
            PyTimeScale::Ut1 => self.0.with_scale(Ut1).try_to_scale(Tt, provider),
        }
    }
}

impl TryToScale<Tt, NoOpOffsetProvider, PyErr> for PyTime {
    fn try_to_scale(&self, _scale: Tt, _provider: &NoOpOffsetProvider) -> PyResult<Time<Tt>> {
        match self.0.scale() {
            PyTimeScale::Tai => Ok(self.0.with_scale(Tai).to_tt()),
            PyTimeScale::Tcb => Ok(self.0.with_scale(Tcb).to_tt()),
            PyTimeScale::Tcg => Ok(self.0.with_scale(Tcg).to_tt()),
            PyTimeScale::Tdb => Ok(self.0.with_scale(Tdb).to_tt()),
            PyTimeScale::Tt => Ok(self.0.with_scale(Tt)),
            PyTimeScale::Ut1 => Err(PyValueError::new_err(
                "`provider` argument needs to be present for UT1 transformations",
            )),
        }
    }
}

impl TryToScale<Ut1, DeltaUt1Tai, ExtrapolatedDeltaUt1Tai> for PyTime {
    fn try_to_scale(
        &self,
        _scale: Ut1,
        provider: &DeltaUt1Tai,
    ) -> Result<Time<Ut1>, ExtrapolatedDeltaUt1Tai> {
        match self.0.scale() {
            PyTimeScale::Tai => self.0.with_scale(Tai).try_to_scale(Ut1, provider),
            PyTimeScale::Tcb => self.0.with_scale(Tcb).try_to_scale(Ut1, provider),
            PyTimeScale::Tcg => self.0.with_scale(Tcg).try_to_scale(Ut1, provider),
            PyTimeScale::Tdb => self.0.with_scale(Tdb).try_to_scale(Ut1, provider),
            PyTimeScale::Tt => self.0.with_scale(Tt).try_to_scale(Ut1, provider),
            PyTimeScale::Ut1 => Ok(self.0.with_scale(Ut1)),
        }
    }
}

impl TryToScale<Ut1, NoOpOffsetProvider, PyErr> for PyTime {
    fn try_to_scale(&self, _scale: Ut1, _provider: &NoOpOffsetProvider) -> PyResult<Time<Ut1>> {
        match self.0.scale() {
            PyTimeScale::Ut1 => Ok(self.0.with_scale(Ut1)),
            _ => Err(PyValueError::new_err(
                "`provider` argument needs to be present for UT1 transformations",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use pyo3::{types::PyDict, Python};

    use super::*;

    #[test]
    fn test_pytime() {
        let time = PyTime::new("TAI", 2000, 1, 1, 0, 0, 12.123456789123).unwrap();
        dbg!(&time);
        assert_eq!(
            time.__repr__(),
            "Time(\"TAI\", 2000, 1, 1, 0, 0, 12.123456789123)"
        );
        assert_eq!(time.__str__(), "2000-01-01T00:00:12.123 TAI");
        assert_eq!(time.scale(), "TAI".to_string());
        assert_eq!(time.year(), 2000);
        assert_eq!(time.month(), 1);
        assert_eq!(time.day(), 1);
        assert_eq!(time.hour(), 0);
        assert_eq!(time.minute(), 0);
        assert_eq!(time.second(), 12);
        assert_eq!(time.millisecond(), 123);
        assert_eq!(time.microsecond(), 456);
        assert_eq!(time.nanosecond(), 789);
        assert_eq!(time.picosecond(), 123);
        assert_eq!(time.femtosecond(), 0);
        assert_float_eq!(time.decimal_seconds(), 12.123456789123, rel <= 1e-15);
    }

    #[test]
    #[should_panic(expected = "invalid date")]
    fn test_pytime_invalid_date() {
        PyTime::new("TAI", 2000, 13, 1, 0, 0, 0.0).unwrap();
    }

    #[test]
    #[should_panic(expected = "hour must be in the range")]
    fn test_pytime_invalid_time() {
        PyTime::new("TAI", 2000, 12, 1, 24, 0, 0.0).unwrap();
    }

    #[test]
    fn test_pytime_ops() {
        Python::with_gil(|py| {
            let t0 = PyTime::new("TAI", 2000, 1, 1, 0, 0, 0.0).unwrap();
            let dt = PyTimeDelta::new(1.0).unwrap();
            let t1 = PyTime::new("TAI", 2000, 1, 1, 0, 0, 1.0).unwrap();
            assert!(t0.__add__(dt.clone()).__eq__(t1.clone()));
            let dtb = Bound::new(py, PyTimeDelta::new(1.0).unwrap()).unwrap();
            assert!(t1
                .__sub__(py, &dtb)
                .unwrap()
                .extract::<PyTime>()
                .unwrap()
                .__eq__(t0));
            let t0b = Bound::new(py, PyTime::new("TAI", 2000, 1, 1, 0, 0, 0.0).unwrap()).unwrap();
            assert!(t1
                .__sub__(py, &t0b)
                .unwrap()
                .extract::<PyTimeDelta>()
                .unwrap()
                .__eq__(dt.clone()));
        });
    }

    #[test]
    #[should_panic(expected = "cannot subtract `Time` objects with different time scales")]
    fn test_pytime_ops_different_scales() {
        Python::with_gil(|py| {
            let t1 = PyTime::new("TAI", 2000, 1, 1, 0, 0, 1.0).unwrap();
            let t0 = Bound::new(py, PyTime::new("TT", 2000, 1, 1, 0, 0, 1.0).unwrap()).unwrap();
            t1.__sub__(py, &t0).unwrap();
        });
    }

    #[test]
    #[should_panic(expected = "`rhs` must be either a `Time` or a `TimeDelta` object")]
    fn test_pytime_ops_invalid_rhs() {
        Python::with_gil(|py| {
            let t1 = PyTime::new("TAI", 2000, 1, 1, 0, 0, 1.0).unwrap();
            let invalid = PyDict::new_bound(py);
            t1.__sub__(py, &invalid).unwrap();
        });
    }

    #[test]
    fn test_pytime_julian_date() {
        Python::with_gil(|py| {
            let cls = PyType::new_bound::<PyTime>(py);
            let time = PyTime::from_julian_date(&cls, "TAI", 0.0, "j2000").unwrap();
            assert_eq!(time.julian_date("j2000", "seconds").unwrap(), 0.0);
            assert_eq!(time.julian_date("j2000", "days").unwrap(), 0.0);
            assert_eq!(time.julian_date("j2000", "centuries").unwrap(), 0.0);
            assert_eq!(time.julian_date("jd", "days").unwrap(), 2451545.0);
            assert_eq!(time.julian_date("mjd", "days").unwrap(), 51544.5);
            assert_eq!(time.julian_date("j1950", "days").unwrap(), 18262.5);
            let time = PyTime::from_julian_date(&cls, "TAI", 0.0, "j1950").unwrap();
            assert_eq!(time.julian_date("j1950", "days").unwrap(), 0.0);
            let time = PyTime::from_julian_date(&cls, "TAI", 0.0, "mjd").unwrap();
            assert_eq!(time.julian_date("mjd", "days").unwrap(), 0.0);
            let time = PyTime::from_julian_date(&cls, "TAI", 0.0, "jd").unwrap();
            assert_eq!(time.julian_date("jd", "days").unwrap(), 0.0);
        })
    }

    #[test]
    #[should_panic(expected = "unknown epoch: unknown")]
    fn test_pytime_invalid_epoch() {
        let time = PyTime::new("TAI", 2000, 1, 1, 0, 0, 0.0).unwrap();
        time.julian_date("unknown", "days").unwrap();
    }

    #[test]
    #[should_panic(expected = "unknown unit: unknown")]
    fn test_pytime_invalid_unit() {
        let time = PyTime::new("TAI", 2000, 1, 1, 0, 0, 0.0).unwrap();
        time.julian_date("jd", "unknown").unwrap();
    }
}
