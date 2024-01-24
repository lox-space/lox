/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use pyo3::{pyclass, pymethods};

use lox_core::time::dates::{Date, Time};
use lox_core::time::epochs::{Epoch, TimeScale};

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

#[pyclass(name = "Epoch")]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PyEpoch(pub Epoch);

#[pymethods]
impl PyEpoch {
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
        hour: Option<i64>,
        minute: Option<i64>,
        second: Option<i64>,
        milli: Option<i64>,
        micro: Option<i64>,
        nano: Option<i64>,
        pico: Option<i64>,
        femto: Option<i64>,
        atto: Option<i64>,
    ) -> Result<Self, LoxPyError> {
        let time_scale = PyTimeScale::new(scale)?;
        let date = Date::new(year, month, day)?;

        let hour = hour.unwrap_or(0);
        let minute = minute.unwrap_or(0);
        let second = second.unwrap_or(0);
        let mut time = Time::new(hour, minute, second)?;
        if let Some(milli) = milli {
            time = time.milli(milli);
        }
        if let Some(micro) = micro {
            time = time.micro(micro);
        }
        if let Some(nano) = nano {
            time = time.nano(nano);
        }
        if let Some(pico) = pico {
            time = time.pico(pico);
        }
        if let Some(femto) = femto {
            time = time.femto(femto);
        }
        if let Some(atto) = atto {
            time = time.atto(atto);
        }
        Ok(PyEpoch(Epoch::from_date_and_time(time_scale.0, date, time)))
    }

    fn days_since_j2000(&self) -> f64 {
        self.0.days_since_j2000()
    }

    fn scale(&self) -> &str {
        self.0.scale()
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
        let time = PyEpoch::new(
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
        assert_eq!(time.0.attosecond(), 123456789123456789);
        assert_float_eq!(time.days_since_j2000(), 8765.542374114084, rel <= 1e-8);
        assert_eq!(time.scale(), "TDB");
    }
}
