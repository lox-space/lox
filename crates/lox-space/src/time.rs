/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt::{Display, Formatter};

use pyo3::{pyclass, pymethods};

use lox_time::continuous::julian_dates::JulianDate;
use lox_time::continuous::{BaseTime, Time, TAI, TCB, TCG, TDB, TT, UT1};
use lox_time::dates::Date;
use lox_time::utc::UTC;
use lox_time::Subsecond;

use crate::LoxPyError;

#[pyclass(name = "TimeScale")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum PyTimeScale {
    TAI,
    TCB,
    TCG,
    TDB,
    TT,
    UT1,
}

#[pymethods]
impl PyTimeScale {
    #[new]
    fn new(name: &str) -> Result<Self, LoxPyError> {
        match name {
            "TAI" => Ok(PyTimeScale::TAI),
            "TCB" => Ok(PyTimeScale::TCB),
            "TCG" => Ok(PyTimeScale::TCG),
            "TDB" => Ok(PyTimeScale::TDB),
            "TT" => Ok(PyTimeScale::TT),
            "UT1" => Ok(PyTimeScale::UT1),
            _ => Err(LoxPyError::InvalidTimeScale(name.to_string())),
        }
    }

    fn __repr__(&self) -> String {
        format!("TimeScale(\"{}\")", self)
    }

    fn __str__(&self) -> String {
        format!("{}", self)
    }
}

impl Display for PyTimeScale {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PyTimeScale::TAI => "TAI",
            PyTimeScale::TCB => "TCB",
            PyTimeScale::TCG => "TCG",
            PyTimeScale::TDB => "TDB",
            PyTimeScale::TT => "TT",
            PyTimeScale::UT1 => "UT1",
        };
        write!(f, "{}", s)
    }
}

#[pyclass(name = "Time")]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PyTime {
    pub scale: PyTimeScale,
    pub timestamp: BaseTime,
}

#[pyclass(name = "Subsecond")]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct PySubsecond {
    subsecond: Subsecond,
}

#[pymethods]
impl PySubsecond {
    #[new]
    pub fn new(subsecond: f64) -> Result<Self, LoxPyError> {
        Ok(PySubsecond {
            subsecond: Subsecond::new(subsecond)?,
        })
    }

    fn __repr__(&self) -> String {
        format!("Subsecond({})", Into::<f64>::into(self.subsecond))
    }

    fn __str__(&self) -> String {
        self.subsecond.to_string()
    }
}

#[pymethods]
impl PyTime {
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (
    scale,
    year,
    month,
    day,
    hour = 0,
    minute = 0,
    second = 0,
    subsecond = PySubsecond::default()
    ))]
    #[new]
    pub fn new(
        scale: PyTimeScale,
        year: i64,
        month: i64,
        day: i64,
        hour: Option<u8>,
        minute: Option<u8>,
        second: Option<u8>,
        subsecond: Option<PySubsecond>,
    ) -> Result<Self, LoxPyError> {
        let date = Date::new(year, month, day)?;
        let hour = hour.unwrap_or_default();
        let minute = minute.unwrap_or_default();
        let second = second.unwrap_or_default();
        let subsecond = subsecond.unwrap_or_default();
        let utc = UTC::new(hour, minute, second, subsecond.subsecond)?;
        Ok(pytime_from_date_and_utc_timestamp(scale, date, utc))
    }

    pub fn days_since_j2000(&self) -> f64 {
        self.timestamp.days_since_j2000()
    }

    pub fn scale(&self) -> PyTimeScale {
        self.scale
    }
}

fn pytime_from_date_and_utc_timestamp(scale: PyTimeScale, date: Date, utc: UTC) -> PyTime {
    PyTime {
        timestamp: base_time_from_date_and_utc_timestamp(scale, date, utc),
        scale,
    }
}

/// Invokes the appropriate [Time::from_date_and_utc_timestamp] method based on the time scale, and returns the
/// result as a [BaseTime]. The Rust time library performs the appropriate transformation while keeping
/// generics out of the Python interface.
fn base_time_from_date_and_utc_timestamp(scale: PyTimeScale, date: Date, utc: UTC) -> BaseTime {
    match scale {
        PyTimeScale::TAI => Time::from_date_and_utc_timestamp(TAI, date, utc).base_time(),
        PyTimeScale::TCB => Time::from_date_and_utc_timestamp(TCB, date, utc).base_time(),
        PyTimeScale::TCG => Time::from_date_and_utc_timestamp(TCG, date, utc).base_time(),
        PyTimeScale::TDB => Time::from_date_and_utc_timestamp(TDB, date, utc).base_time(),
        PyTimeScale::TT => Time::from_date_and_utc_timestamp(TT, date, utc).base_time(),
        PyTimeScale::UT1 => Time::from_date_and_utc_timestamp(UT1, date, utc).base_time(),
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use lox_time::continuous::TimeScale;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("TAI", PyTimeScale::TAI)]
    #[case("TCB", PyTimeScale::TCB)]
    #[case("TCG", PyTimeScale::TCG)]
    #[case("TDB", PyTimeScale::TDB)]
    #[case("TT", PyTimeScale::TT)]
    #[case("UT1", PyTimeScale::UT1)]
    fn test_scale(#[case] name: &str, #[case] scale: PyTimeScale) {
        let py_scale = PyTimeScale::new(name).expect("time scale should be valid");
        assert_eq!(py_scale, scale);
        assert_eq!(py_scale.__str__(), name);
        assert_eq!(py_scale.__repr__(), format!("TimeScale(\"{}\")", name));
    }

    // Due to the conversion between enum PyTimeScale and generic Rust TimeScales, the following
    // time_new tests can't be parameterized with rstest.
    #[test]
    fn test_time_new_tai() {
        let actual = PyTime::new(PyTimeScale::TAI, 2024, 1, 1, None, None, None, None).unwrap();
        let expected = PyTime {
            scale: PyTimeScale::TAI,
            timestamp: Time::from_date_and_utc_timestamp(
                TAI,
                Date::new(2024, 1, 1).unwrap(),
                UTC::new(0, 0, 0, Subsecond::default()).unwrap(),
            )
            .base_time(),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_new_tcb() {
        let actual = PyTime::new(PyTimeScale::TCB, 2024, 1, 1, None, None, None, None).unwrap();
        let expected = PyTime {
            scale: PyTimeScale::TCB,
            timestamp: Time::from_date_and_utc_timestamp(
                TCB,
                Date::new(2024, 1, 1).unwrap(),
                UTC::new(0, 0, 0, Subsecond::default()).unwrap(),
            )
            .base_time(),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_new_tcg() {
        let actual = PyTime::new(PyTimeScale::TCG, 2024, 1, 1, None, None, None, None).unwrap();
        let expected = PyTime {
            scale: PyTimeScale::TCG,
            timestamp: Time::from_date_and_utc_timestamp(
                TCG,
                Date::new(2024, 1, 1).unwrap(),
                UTC::new(0, 0, 0, Subsecond::default()).unwrap(),
            )
            .base_time(),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_new_tdb() {
        let actual = PyTime::new(PyTimeScale::TDB, 2024, 1, 1, None, None, None, None).unwrap();
        let expected = PyTime {
            scale: PyTimeScale::TDB,
            timestamp: Time::from_date_and_utc_timestamp(
                TDB,
                Date::new(2024, 1, 1).unwrap(),
                UTC::new(0, 0, 0, Subsecond::default()).unwrap(),
            )
            .base_time(),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_new_tt() {
        let actual = PyTime::new(PyTimeScale::TT, 2024, 1, 1, None, None, None, None).unwrap();
        let expected = PyTime {
            scale: PyTimeScale::TT,
            timestamp: Time::from_date_and_utc_timestamp(
                TT,
                Date::new(2024, 1, 1).unwrap(),
                UTC::new(0, 0, 0, Subsecond::default()).unwrap(),
            )
            .base_time(),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_new_ut1() {
        let actual = PyTime::new(PyTimeScale::UT1, 2024, 1, 1, None, None, None, None).unwrap();
        let expected = PyTime {
            scale: PyTimeScale::UT1,
            timestamp: Time::from_date_and_utc_timestamp(
                UT1,
                Date::new(2024, 1, 1).unwrap(),
                UTC::new(0, 0, 0, Subsecond::default()).unwrap(),
            )
            .base_time(),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_invalid_scale() {
        let py_scale = PyTimeScale::new("disco time");
        assert!(py_scale.is_err())
    }

    #[test]
    fn test_time_days_since_j2000() {
        let time = PyTime::new(
            PyTimeScale::TDB,
            2024,
            1,
            1,
            Some(1),
            Some(1),
            Some(1),
            Some(PySubsecond::new(0.123456789123456).expect("PySubsecond should be valid")),
        )
        .expect("PyTime should be valid");
        assert_float_eq!(8765.542374114084, time.days_since_j2000(), rel <= 1e-8);
    }

    #[test]
    fn test_time_scale() {
        let time = PyTime::new(
            PyTimeScale::TDB,
            2024,
            1,
            1,
            Some(1),
            Some(1),
            Some(1),
            Some(PySubsecond::new(0.123456789123456).expect("PySubsecond should be valid")),
        )
        .expect("PyTime should be valid");
        assert_eq!(PyTimeScale::TDB, time.scale());
    }

    #[test]
    fn test_py_subsecond_new() {
        let actual = PySubsecond::new(0.123).expect("subsecond should be valid");
        let expected = PySubsecond {
            subsecond: Subsecond::new(0.123).expect("subsecond should be valid"),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_py_subsecond_repr() {
        let actual = PySubsecond::new(0.123456789123456)
            .expect("subsecond should be valid")
            .__repr__();
        let expected = "Subsecond(0.123456789123456)";
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_py_subsecond_str() {
        let actual = PySubsecond::new(0.123456789123456)
            .expect("subsecond should be valid")
            .__str__();
        let expected = "123.456.789.123.456";
        assert_eq!(expected, actual);
    }
}
