use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use thiserror::Error;

use lox_core::errors::LoxError;
use lox_core::time::dates::{Date, Time};
use lox_core::time::epochs::Epoch as _Epoch;
use lox_core::time::epochs::TimeScale as _TimeScale;

#[derive(Error, Debug)]
pub enum LoxPyError {
    #[error("invalid time scale `{0}`")]
    InvalidTimeScale(String),
    #[error(transparent)]
    LoxError(#[from] LoxError),
    #[error(transparent)]
    PyError(#[from] PyErr),
}

impl From<LoxPyError> for PyErr {
    fn from(value: LoxPyError) -> Self {
        match value {
            LoxPyError::InvalidTimeScale(_) => PyValueError::new_err(value.to_string()),
            LoxPyError::LoxError(value) => PyValueError::new_err(value.to_string()),
            LoxPyError::PyError(value) => value,
        }
    }
}

#[pyclass]
struct TimeScale(_TimeScale);

#[pymethods]
impl TimeScale {
    #[new]
    fn new(name: &str) -> Result<Self, LoxPyError> {
        match name {
            "TAI" => Ok(TimeScale(_TimeScale::TAI)),
            "TCB" => Ok(TimeScale(_TimeScale::TCB)),
            "TCG" => Ok(TimeScale(_TimeScale::TCG)),
            "TDB" => Ok(TimeScale(_TimeScale::TDB)),
            "TT" => Ok(TimeScale(_TimeScale::TT)),
            "UT1" => Ok(TimeScale(_TimeScale::UT1)),
            _ => Err(LoxPyError::InvalidTimeScale(name.to_string())),
        }
    }

    fn __repr__(&self) -> String {
        format!("TimeScale(\"{}\")", self.0)
    }
}

#[pyclass]
struct Epoch(_Epoch);

#[pymethods]
impl Epoch {
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
    fn new(
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
        let time_scale = TimeScale::new(scale)?;
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
        Ok(Epoch(_Epoch::from_date_and_time(time_scale.0, date, time)))
    }

    fn attosecond(&self) -> i64 {
        self.0.attosecond()
    }

    fn __str__(&self) -> String {
        "foo".to_string()
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn lox_space(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<TimeScale>()?;
    m.add_class::<Epoch>()?;
    Ok(())
}
