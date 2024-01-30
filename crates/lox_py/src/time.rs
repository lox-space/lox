/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use pyo3::{pyclass, pymethods};

use lox_core::time::continuous::{Time, TimeScale};
use lox_core::time::dates::Date;
use lox_core::time::utc::UTC;
use lox_core::time::PerMille;

use crate::LoxPyError;

#[pyclass(name = "TimeScale")]
pub struct PyTimeScale(pub TimeScale);

#[pymethods]
impl PyTimeScale {
    #[new]
    fn new(name: &str) -> Result<Self, LoxPyError> {
        match name {
            "TAI" => Ok(PyTimeScale(TimeScale::TAI)),
            "TCB" => Ok(PyTimeScale(TimeScale::TCB)),
            "TCG" => Ok(PyTimeScale(TimeScale::TCG)),
            "TDB" => Ok(PyTimeScale(TimeScale::TDB)),
            "TT" => Ok(PyTimeScale(TimeScale::TT)),
            "UT1" => Ok(PyTimeScale(TimeScale::UT1)),
            _ => Err(LoxPyError::InvalidTimeScale(name.to_string())),
        }
    }

    fn __repr__(&self) -> String {
        format!("TimeScale(\"{}\")", self.0)
    }

    fn __str__(&self) -> String {
        format!("{}", self.0)
    }
}

#[pyclass(name = "Time")]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PyTime(pub Time);

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
        scale: &str,
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
        let time_scale = PyTimeScale::new(scale)?;
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
        Ok(PyTime(Time::from_date_and_utc_timestamp(
            time_scale.0,
            date,
            utc,
        )))
    }

    fn days_since_j2000(&self) -> f64 {
        self.0.days_since_j2000()
    }

    fn scale(&self) -> &str {
        self.0.scale().into()
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("TAI", TimeScale::TAI)]
    #[case("TCB", TimeScale::TCB)]
    #[case("TCG", TimeScale::TCG)]
    #[case("TDB", TimeScale::TDB)]
    #[case("TT", TimeScale::TT)]
    #[case("UT1", TimeScale::UT1)]
    fn test_scale(#[case] name: &str, #[case] scale: TimeScale) {
        let py_scale = PyTimeScale::new(name).expect("time scale should be valid");
        assert_eq!(py_scale.0, scale);
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
            "TDB",
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
        assert_eq!(time.0.attoseconds(), 123456789123456789);
        assert_float_eq!(time.days_since_j2000(), 8765.542374114084, rel <= 1e-8);
        assert_eq!(time.scale(), "TDB");
    }
}
