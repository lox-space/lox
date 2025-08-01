/*
 * Copyright (c) 2025. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::ops::{Add, Sub};
use std::str::FromStr;

use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::types::{PyAnyMethods, PyType};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyErr, PyObject, PyResult, Python, pyclass, pymethods};

use lox_math::is_close::IsClose;

use crate::calendar_dates::{CalendarDate, Date};
use crate::deltas::{TimeDelta, ToDelta};
use crate::julian_dates::{Epoch, JulianDate, Unit};
use crate::python::deltas::PyTimeDelta;
use crate::python::ut1::PyUt1Provider;
use crate::subsecond::{InvalidSubsecond, Subsecond};
use crate::time::{DynTime, Time, TimeError};
use crate::time_of_day::{CivilTime, TimeOfDay};
use crate::time_scales::{DynTimeScale, Tai};
use crate::utc::transformations::ToUtc;

use super::time_scales::PyTimeScale;
use super::utc::PyUtc;

impl From<InvalidSubsecond> for PyErr {
    fn from(value: InvalidSubsecond) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

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
            _ => Err(PyValueError::new_err(format!("unknown epoch: {s}"))),
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
            _ => Err(PyValueError::new_err(format!("unknown unit: {s}"))),
        }
    }
}

#[pyclass(name = "Time", module = "lox_space", frozen)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyTime(pub DynTime);

#[pymethods]
impl PyTime {
    #[new]
    #[pyo3(signature=(scale, year, month, day, hour = 0, minute = 0, seconds = 0.0))]
    pub fn new(
        scale: &Bound<'_, PyAny>,
        year: i64,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        seconds: f64,
    ) -> PyResult<PyTime> {
        let scale: DynTimeScale = scale.try_into()?;
        let time = Time::builder_with_scale(scale)
            .with_ymd(year, month, day)
            .with_hms(hour, minute, seconds)
            .build()?;
        Ok(PyTime(time))
    }

    fn __getnewargs__<'py>(
        &self,
        py: Python<'py>,
    ) -> (Bound<'py, PyAny>, i64, u8, u8, u8, u8, f64) {
        (
            self.scale().into_bound_py_any(py).unwrap(),
            self.0.year(),
            self.0.month(),
            self.0.day(),
            self.0.hour(),
            self.0.minute(),
            self.0.decimal_seconds(),
        )
    }

    #[classmethod]
    #[pyo3(signature = (scale, jd, epoch = "jd"))]
    pub fn from_julian_date(
        _cls: &Bound<'_, PyType>,
        scale: &Bound<'_, PyAny>,
        jd: f64,
        epoch: &str,
    ) -> PyResult<Self> {
        let scale: DynTimeScale = scale.try_into()?;
        let epoch: Epoch = epoch.parse()?;
        Ok(Self(Time::from_julian_date(scale, jd, epoch)?))
    }

    #[classmethod]
    pub fn from_two_part_julian_date(
        _cls: &Bound<'_, PyType>,
        scale: &Bound<'_, PyAny>,
        jd1: f64,
        jd2: f64,
    ) -> PyResult<Self> {
        let scale: DynTimeScale = scale.try_into()?;
        Ok(Self(Time::from_two_part_julian_date(scale, jd1, jd2)?))
    }

    #[classmethod]
    #[pyo3(signature=(scale, year, day, hour=0, minute=0, seconds=0.0))]
    pub fn from_day_of_year(
        _cls: &Bound<'_, PyType>,
        scale: &Bound<'_, PyAny>,
        year: i64,
        day: u16,
        hour: u8,
        minute: u8,
        seconds: f64,
    ) -> PyResult<PyTime> {
        let scale: DynTimeScale = scale.try_into()?;
        let time = Time::builder_with_scale(scale)
            .with_doy(year, day)
            .with_hms(hour, minute, seconds)
            .build()?;
        Ok(PyTime(time))
    }

    #[classmethod]
    #[pyo3(signature = (iso, scale=None))]
    pub fn from_iso(
        _cls: &Bound<'_, PyType>,
        iso: &str,
        scale: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<PyTime> {
        let scale: DynTimeScale =
            scale.map_or(Ok(DynTimeScale::default()), |scale| scale.try_into())?;
        let time = Time::from_iso(scale, iso)?;
        Ok(PyTime(time))
    }

    #[classmethod]
    pub fn from_seconds(
        _cls: &Bound<'_, PyType>,
        scale: &Bound<'_, PyAny>,
        seconds: i64,
        subsecond: f64,
    ) -> PyResult<PyTime> {
        let scale: DynTimeScale = scale.try_into()?;
        let subsecond = Subsecond::new(subsecond)?;
        let time = Time::new(scale, seconds, subsecond);
        Ok(PyTime(time))
    }

    pub fn seconds(&self) -> i64 {
        self.0.seconds()
    }

    pub fn subsecond(&self) -> f64 {
        self.0.subsecond()
    }

    #[classattr]
    const __hash__: Option<PyObject> = None;

    pub fn __str__(&self) -> String {
        self.0.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Time(\"{}\", {}, {}, {}, {}, {}, {})",
            self.scale().abbreviation(),
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

    fn __richcmp__(&self, other: PyTime, op: CompareOp) -> bool {
        op.matches(self.0.cmp(&other.0))
    }

    #[pyo3(signature = (rhs, rel_tol = 1e-8, abs_tol = 1e-14))]
    pub fn isclose(&self, rhs: PyTime, rel_tol: f64, abs_tol: f64) -> PyResult<bool> {
        if self.scale() != rhs.scale() {
            return Err(PyValueError::new_err(
                "cannot compare `Time` objects with different time scales",
            ));
        }
        Ok(self.0.is_close_with_tolerances(&rhs.0, rel_tol, abs_tol))
    }

    #[pyo3(signature = (epoch = "jd", unit = "days"))]
    pub fn julian_date(&self, epoch: &str, unit: &str) -> PyResult<f64> {
        let epoch: Epoch = epoch.parse()?;
        let unit: Unit = unit.parse()?;
        Ok(self.0.julian_date(epoch, unit))
    }

    pub fn two_part_julian_date(&self) -> (f64, f64) {
        self.0.two_part_julian_date()
    }

    pub fn scale(&self) -> PyTimeScale {
        PyTimeScale(self.0.scale())
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

    pub fn day_of_year(&self) -> u16 {
        self.0.day_of_year()
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

    #[pyo3(signature = (scale, provider=None))]
    pub fn to_scale(
        &self,
        scale: &Bound<'_, PyAny>,
        provider: Option<&Bound<'_, PyUt1Provider>>,
    ) -> PyResult<PyTime> {
        let scale: DynTimeScale = scale.try_into()?;
        let provider = provider.map(|p| &p.get().0);
        Ok(PyTime(
            self.0
                .try_to_scale(scale, provider)
                // FIXME: Better error type
                .map_err(|err| PyValueError::new_err(err.to_string()))?,
        ))
    }

    #[pyo3(signature = (provider=None))]
    pub fn to_utc(&self, provider: Option<&Bound<'_, PyUt1Provider>>) -> PyResult<PyUtc> {
        let provider = provider.map(|p| &p.get().0);
        Ok(PyUtc(
            self.0
                .try_to_scale(Tai, provider)
                // FIXME: Better error type
                .map_err(|err| PyValueError::new_err(err.to_string()))?
                .to_utc()?,
        ))
    }
}

impl ToDelta for PyTime {
    fn to_delta(&self) -> TimeDelta {
        self.0.to_delta()
    }
}

impl JulianDate for PyTime {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        self.0.julian_date(epoch, unit)
    }
}

impl Add<TimeDelta> for PyTime {
    type Output = PyTime;

    fn add(self, rhs: TimeDelta) -> Self::Output {
        PyTime(self.0 + rhs)
    }
}

impl Sub<TimeDelta> for PyTime {
    type Output = PyTime;

    fn sub(self, rhs: TimeDelta) -> Self::Output {
        PyTime(self.0 - rhs)
    }
}

impl Sub<PyTime> for PyTime {
    type Output = TimeDelta;

    fn sub(self, rhs: PyTime) -> TimeDelta {
        self.0 - rhs.0
    }
}

impl CalendarDate for PyTime {
    fn date(&self) -> Date {
        self.0.date()
    }
}

impl CivilTime for PyTime {
    fn time(&self) -> TimeOfDay {
        self.0.time()
    }
}

impl IsClose for PyTime {
    const DEFAULT_RELATIVE: f64 = 1e-10;

    const DEFAULT_ABSOLUTE: f64 = 1e-14;

    fn is_close_with_tolerances(&self, rhs: &Self, rel_tol: f64, abs_tol: f64) -> bool {
        self.0.is_close_with_tolerances(&rhs.0, rel_tol, abs_tol)
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use pyo3::{IntoPyObjectExt, Python, types::PyDict};

    use lox_math::assert_close;
    use rstest::rstest;

    use crate::test_helpers::data_dir;

    use super::*;

    #[test]
    fn test_pytimfe() {
        let time = Python::with_gil(|py| {
            PyTime::new(&scale_to_any(py, "TAI"), 2000, 1, 1, 0, 0, 12.123456789123).unwrap()
        });
        assert_eq!(
            time.__repr__(),
            "Time(\"TAI\", 2000, 1, 1, 0, 0, 12.123456789123)"
        );
        assert_eq!(time.__str__(), "2000-01-01T00:00:12.123 TAI");
        assert_eq!(time.scale().abbreviation(), "TAI".to_string());
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
        Python::with_gil(|py| {
            PyTime::new(&scale_to_any(py, "TAI"), 2000, 13, 1, 0, 0, 0.0).unwrap()
        });
    }

    #[test]
    #[should_panic(expected = "hour must be in the range")]
    fn test_pytime_invalid_time() {
        Python::with_gil(|py| {
            PyTime::new(&scale_to_any(py, "TAI"), 2000, 12, 1, 24, 0, 0.0).unwrap()
        });
    }

    #[test]
    fn test_pytime_ops() {
        Python::with_gil(|py| {
            let t0 = PyTime::new(&scale_to_any(py, "TAI"), 2000, 1, 1, 0, 0, 0.0).unwrap();
            let dt = PyTimeDelta::new(1.0).unwrap();
            let t1 = PyTime::new(&scale_to_any(py, "TAI"), 2000, 1, 1, 0, 0, 1.0).unwrap();
            assert_eq!(t0.__add__(dt.clone()), t1.clone());
            let dtb = Bound::new(py, PyTimeDelta::new(1.0).unwrap()).unwrap();
            assert_eq!(
                t1.__sub__(py, &dtb).unwrap().extract::<PyTime>().unwrap(),
                t0
            );
            let t0b = Bound::new(
                py,
                PyTime::new(&scale_to_any(py, "TAI"), 2000, 1, 1, 0, 0, 0.0).unwrap(),
            )
            .unwrap();
            assert_eq!(
                t1.__sub__(py, &t0b)
                    .unwrap()
                    .extract::<PyTimeDelta>()
                    .unwrap(),
                dt.clone()
            );
        });
    }

    #[test]
    #[should_panic(expected = "cannot subtract `Time` objects with different time scales")]
    fn test_pytime_ops_different_scales() {
        Python::with_gil(|py| {
            let t1 = PyTime::new(&scale_to_any(py, "TAI"), 2000, 1, 1, 0, 0, 1.0).unwrap();
            let t0 = Bound::new(
                py,
                PyTime::new(&scale_to_any(py, "TT"), 2000, 1, 1, 0, 0, 1.0).unwrap(),
            )
            .unwrap();
            t1.__sub__(py, &t0).unwrap();
        });
    }

    #[test]
    #[should_panic(expected = "`rhs` must be either a `Time` or a `TimeDelta` object")]
    fn test_pytime_ops_invalid_rhs() {
        Python::with_gil(|py| {
            let t1 = PyTime::new(&scale_to_any(py, "TAI"), 2000, 1, 1, 0, 0, 1.0).unwrap();
            let invalid = PyDict::new(py);
            t1.__sub__(py, &invalid).unwrap();
        });
    }

    #[test]
    fn test_pytime_richcmp() {
        Python::with_gil(|py| {
            let t0 = PyTime::new(&scale_to_any(py, "TAI"), 2000, 1, 1, 0, 0, 0.0).unwrap();
            let t1 = PyTime::new(&scale_to_any(py, "TAI"), 2000, 1, 1, 0, 0, 1.0).unwrap();
            assert!(t0.__richcmp__(t1.clone(), CompareOp::Lt));
            assert!(t0.__richcmp__(t1.clone(), CompareOp::Le));
            assert!(t0.__richcmp__(t1.clone(), CompareOp::Ne));
            assert!(t1.__richcmp__(t0.clone(), CompareOp::Gt));
            assert!(t1.__richcmp__(t0.clone(), CompareOp::Ge));
        })
    }

    #[test]
    fn test_pytime_is_close() {
        Python::with_gil(|py| {
            let t0 = PyTime::new(
                &scale_to_any(py, "TAI"),
                1999,
                12,
                31,
                23,
                59,
                59.999999999999,
            )
            .unwrap();
            let t1 =
                PyTime::new(&scale_to_any(py, "TAI"), 2000, 1, 1, 0, 0, 0.0000000000001).unwrap();
            assert!(t0.isclose(t1, 1e-8, 1e-13).unwrap());
        })
    }

    #[test]
    #[should_panic(expected = "cannot compare `Time` objects with different time scales")]
    fn test_pytime_is_close_different_scales() {
        Python::with_gil(|py| {
            let t0 = PyTime::new(&scale_to_any(py, "TAI"), 2000, 1, 1, 0, 0, 0.0).unwrap();
            let t1 = PyTime::new(&scale_to_any(py, "TT"), 2000, 1, 1, 0, 0, 0.0).unwrap();
            t0.isclose(t1, 1e-8, 1e-13).unwrap();
        })
    }

    #[test]
    fn test_pytime_to_delta() {
        Python::with_gil(|py| {
            let t = PyTime::new(&scale_to_any(py, "TAI"), 2000, 1, 1, 12, 0, 0.3).unwrap();
            assert_eq!(t.to_delta(), t.0.to_delta())
        })
    }

    #[test]
    fn test_pytime_from_iso() {
        Python::with_gil(|py| {
            let cls = PyType::new::<PyTime>(py);
            let expected = PyTime::new(&scale_to_any(py, "TAI"), 2000, 1, 1, 0, 0, 0.0).unwrap();
            let actual = PyTime::from_iso(&cls, "2000-01-01T00:00:00 TAI", None).unwrap();
            assert_eq!(actual, expected);
            let actual = PyTime::from_iso(&cls, "2000-01-01T00:00:00", None).unwrap();
            assert_eq!(actual, expected);
            let actual =
                PyTime::from_iso(&cls, "2000-01-01T00:00:00", Some(&scale_to_any(py, "TAI")))
                    .unwrap();
            assert_eq!(actual, expected);
        })
    }

    #[test]
    #[should_panic(expected = "invalid ISO")]
    fn test_pytime_from_iso_invalid() {
        Python::with_gil(|py| {
            let cls = PyType::new::<PyTime>(py);
            let _ = PyTime::from_iso(&cls, "2000-01-01X00:00:00 TAI", None).unwrap();
        })
    }

    #[test]
    #[should_panic(expected = "invalid ISO")]
    fn test_pytime_from_iso_invalid_scale() {
        Python::with_gil(|py| {
            let cls = PyType::new::<PyTime>(py);
            let _ = PyTime::from_iso(&cls, "2000-01-01T00:00:00 UTC", None).unwrap();
        })
    }

    #[test]
    #[should_panic(expected = "unknown time scale: UTC")]
    fn test_pytime_from_iso_invalid_scale_arg() {
        Python::with_gil(|py| {
            let cls = PyType::new::<PyTime>(py);
            let _ = PyTime::from_iso(
                &cls,
                "2000-01-01T00:00:00 TAI",
                Some(&scale_to_any(py, "UTC")),
            )
            .unwrap();
        })
    }

    #[test]
    fn test_pytime_julian_date() {
        Python::with_gil(|py| {
            let cls = PyType::new::<PyTime>(py);
            let time =
                PyTime::from_julian_date(&cls, &scale_to_any(py, "TAI"), 0.0, "j2000").unwrap();
            assert_eq!(time.julian_date("j2000", "seconds").unwrap(), 0.0);
            assert_eq!(time.julian_date("j2000", "days").unwrap(), 0.0);
            assert_eq!(time.julian_date("j2000", "centuries").unwrap(), 0.0);
            assert_eq!(time.julian_date("jd", "days").unwrap(), 2451545.0);
            assert_eq!(time.julian_date("mjd", "days").unwrap(), 51544.5);
            assert_eq!(time.julian_date("j1950", "days").unwrap(), 18262.5);
            let time =
                PyTime::from_julian_date(&cls, &scale_to_any(py, "TAI"), 0.0, "j1950").unwrap();
            assert_eq!(time.julian_date("j1950", "days").unwrap(), 0.0);
            let time =
                PyTime::from_julian_date(&cls, &scale_to_any(py, "TAI"), 0.0, "mjd").unwrap();
            assert_eq!(time.julian_date("mjd", "days").unwrap(), 0.0);
            let time = PyTime::from_julian_date(&cls, &scale_to_any(py, "TAI"), 0.0, "jd").unwrap();
            assert_eq!(time.julian_date("jd", "days").unwrap(), 0.0);
        })
    }

    #[test]
    #[should_panic(expected = "unknown epoch: unknown")]
    fn test_pytime_invalid_epoch() {
        Python::with_gil(|py| {
            let time = PyTime::new(&scale_to_any(py, "TAI"), 2000, 1, 1, 0, 0, 0.0).unwrap();
            time.julian_date("unknown", "days").unwrap();
        })
    }

    #[test]
    #[should_panic(expected = "unknown unit: unknown")]
    fn test_pytime_invalid_unit() {
        Python::with_gil(|py| {
            let time = PyTime::new(&scale_to_any(py, "TAI"), 2000, 1, 1, 0, 0, 0.0).unwrap();
            time.julian_date("jd", "unknown").unwrap();
        })
    }

    #[test]
    fn test_pytime_from_two_part_julian_date() {
        Python::with_gil(|py| {
            let cls = PyType::new::<PyTime>(py);
            let expected = PyTime::new(&scale_to_any(py, "TAI"), 2024, 7, 11, 8, 2, 14.0).unwrap();
            let (jd1, jd2) = expected.two_part_julian_date();
            let actual =
                PyTime::from_two_part_julian_date(&cls, &scale_to_any(py, "TAI"), jd1, jd2)
                    .unwrap();
            assert_close!(actual.0, expected.0);
        })
    }

    #[test]
    fn test_pytime_from_day_of_year() {
        Python::with_gil(|py| {
            let cls = PyType::new::<PyTime>(py);
            let expected = PyTime::new(&scale_to_any(py, "TAI"), 2024, 12, 31, 0, 0, 0.0).unwrap();
            let actual =
                PyTime::from_day_of_year(&cls, &scale_to_any(py, "TAI"), 2024, 366, 0, 0, 0.0)
                    .unwrap();
            assert_eq!(actual, expected);
        })
    }

    #[rstest]
    #[case("TAI", "TAI")]
    #[case("TAI", "TCB")]
    #[case("TAI", "TCG")]
    #[case("TAI", "TDB")]
    #[case("TAI", "TT")]
    #[case("TAI", "UT1")]
    #[case("TCB", "TAI")]
    #[case("TCB", "TCB")]
    #[case("TCB", "TCG")]
    #[case("TCB", "TDB")]
    #[case("TCB", "TT")]
    #[case("TCB", "UT1")]
    #[case("TCG", "TAI")]
    #[case("TCG", "TCB")]
    #[case("TCG", "TCG")]
    #[case("TCG", "TDB")]
    #[case("TCG", "TT")]
    #[case("TCG", "UT1")]
    #[case("TDB", "TAI")]
    #[case("TDB", "TCB")]
    #[case("TDB", "TCG")]
    #[case("TDB", "TDB")]
    #[case("TDB", "TT")]
    #[case("TDB", "UT1")]
    #[case("TT", "TAI")]
    #[case("TT", "TCB")]
    #[case("TT", "TCG")]
    #[case("TT", "TDB")]
    #[case("TT", "TT")]
    #[case("TT", "UT1")]
    #[case("UT1", "TAI")]
    #[case("UT1", "TCB")]
    #[case("UT1", "TCG")]
    #[case("UT1", "TDB")]
    #[case("UT1", "TT")]
    #[case("UT1", "UT1")]
    fn test_pytime_to_scale(#[case] scale1: &str, #[case] scale2: &str) {
        Python::with_gil(|py| {
            let provider = Bound::new(
                py,
                PyUt1Provider::new(data_dir().join("finals2000A.all.csv").to_str().unwrap())
                    .unwrap(),
            )
            .unwrap();
            let scale1 = scale_to_any(py, scale1);
            let scale2 = scale_to_any(py, scale2);
            let exp = PyTime::new(&scale1, 2000, 1, 1, 0, 0, 0.0).unwrap();
            let act = exp
                .to_scale(&scale2, Some(&provider))
                .unwrap()
                .to_scale(&scale1, Some(&provider))
                .unwrap();
            assert_close!(act, exp)
        })
    }

    #[test]
    #[should_panic(expected = "a UT1-TAI provider is required")]
    fn test_pytime_ut1_tai_no_provider() {
        Python::with_gil(|py| {
            let time = PyTime::new(&scale_to_any(py, "UT1"), 2000, 1, 1, 0, 0, 0.0).unwrap();
            time.to_scale(&scale_to_any(py, "TAI"), None).unwrap();
        })
    }

    fn scale_to_any<'py>(py: Python<'py>, scale: &str) -> Bound<'py, PyAny> {
        scale.into_bound_py_any(py).unwrap()
    }
}
