/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use pyo3::{pyclass, pymethods};
use std::fmt::{Display, Formatter};

use lox_core::time::continuous::{Time, TimeScale, UnscaledTime, TAI, TCB, TCG, TDB, TT, UT1};
use lox_core::time::dates::Date;
use lox_core::time::utc::UTC;
use lox_core::time::PerMille;

use crate::LoxPyError;

#[pyclass(name = "TimeScale")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    pub timestamp: UnscaledTime,
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
    milli = 0,
    micro = 0,
    nano = 0,
    pico = 0,
    femto = 0,
    atto = 0
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
        milli: Option<u16>,
        micro: Option<u16>,
        nano: Option<u16>,
        pico: Option<u16>,
        femto: Option<u16>,
        atto: Option<u16>,
    ) -> Result<Self, LoxPyError> {
        let date = Date::new(year, month, day)?;

        let hour = hour.unwrap_or(0);
        let minute = minute.unwrap_or(0);
        let second = second.unwrap_or(0);
        let mut utc = UTC::new(hour, minute, second)?;
        if let Some(milli) = milli {
            utc.milli = PerMille::new(milli)?;
        }
        if let Some(micro) = micro {
            utc.micro = PerMille::new(micro)?;
        }
        if let Some(nano) = nano {
            utc.nano = PerMille::new(nano)?;
        }
        if let Some(pico) = pico {
            utc.pico = PerMille::new(pico)?;
        }
        if let Some(femto) = femto {
            utc.femto = PerMille::new(femto)?;
        }
        if let Some(atto) = atto {
            utc.atto = PerMille::new(atto)?;
        }

        Ok(Self::from_date_and_utc_timestamp(scale, date, utc))
    }

    pub fn from_date_and_utc_timestamp(scale: PyTimeScale, date: Date, utc: UTC) -> Self {
        let timestamp = match scale {
            PyTimeScale::TAI => Time::<TAI>::from_date_and_utc_timestamp(date, utc),
            PyTimeScale::TCB => Time::<TCB>::from_date_and_utc_timestamp(date, utc),
            PyTimeScale::TCG => Time::<TCG>::from_date_and_utc_timestamp(date, utc),
            PyTimeScale::TDB => Time::<TDB>::from_date_and_utc_timestamp(date, utc),
            PyTimeScale::TT => Time::<TT>::from_date_and_utc_timestamp(date, utc),
            PyTimeScale::UT1 => Time::<UT1>::from_date_and_utc_timestamp(date, utc),
        }
        .unscaled();

        PyTime { timestamp, scale }
    }

    pub fn days_since_j2000(&self) -> f64 {
        self.timestamp.days_since_j2000()
    }

    pub fn scale(&self) -> PyTimeScale {
        self.scale
    }
}

impl From<Time<TDB>> for PyTime {
    fn from(time: Time<TDB>) -> Self {
        PyTime {
            scale: PyTimeScale::TDB,
            timestamp: time.unscaled(),
        }
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
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

    #[test]
    fn test_invalid_scale() {
        let py_scale = PyTimeScale::new("disco time");
        assert!(py_scale.is_err())
    }

    #[test]
    fn test_time() {
        let time = PyTime::new(
            PyTimeScale::TDB,
            2024,
            1,
            1,
            Some(1),
            Some(1),
            Some(1),
            Some(123),
            Some(456),
            Some(789),
            Some(123),
            Some(456),
            Some(789),
        )
        .expect("time should be valid");
        assert_eq!(time.timestamp.attoseconds(), 123456789123456789);
        assert_float_eq!(time.days_since_j2000(), 8765.542374114084, rel <= 1e-8);
        assert_eq!(time.scale(), "TDB");
    }
}
