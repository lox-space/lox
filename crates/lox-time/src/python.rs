use std::fmt::{Display, Formatter};

use pyo3::{exceptions::PyValueError, pyclass, pymethods, PyErr};

use crate::time_scales::TimeScale;
use crate::transformations::TimeScaleTransformer;
use crate::{
    julian_dates::JulianDate,
    time_scales::{Tai, Tcb, Tcg, Tdb, Tt, Ut1},
    Time,
};

#[pyclass(name = "TimeScale")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PyTimeScale {
    Tai,
    Tcb,
    Tcg,
    Tdb,
    Tt,
    Ut1,
}

#[pymethods]
impl PyTimeScale {
    #[new]
    fn new(name: &str) -> Result<Self, PyErr> {
        match name {
            "TAI" => Ok(PyTimeScale::Tai),
            "TCB" => Ok(PyTimeScale::Tcb),
            "TCG" => Ok(PyTimeScale::Tcg),
            "TDB" => Ok(PyTimeScale::Tdb),
            "TT" => Ok(PyTimeScale::Tt),
            "UT1" => Ok(PyTimeScale::Ut1),
            _ => Err(PyValueError::new_err(format!(
                "invalid timescale: {}",
                name
            ))),
        }
    }

    fn __repr__(&self) -> String {
        format!("TimeScale(\"{}\")", self)
    }

    fn __str__(&self) -> String {
        format!("{}", self)
    }
}

impl TimeScale for PyTimeScale {
    fn abbreviation(&self) -> &'static str {
        match self {
            PyTimeScale::Tai => Tai.abbreviation(),
            PyTimeScale::Tcb => Tcb.abbreviation(),
            PyTimeScale::Tcg => Tcg.abbreviation(),
            PyTimeScale::Tdb => Tdb.abbreviation(),
            PyTimeScale::Tt => Tt.abbreviation(),
            PyTimeScale::Ut1 => Ut1.abbreviation(),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            PyTimeScale::Tai => Tai.name(),
            PyTimeScale::Tcb => Tcb.name(),
            PyTimeScale::Tcg => Tcg.name(),
            PyTimeScale::Tdb => Tdb.name(),
            PyTimeScale::Tt => Tt.name(),
            PyTimeScale::Ut1 => Ut1.name(),
        }
    }
}

impl Display for PyTimeScale {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

#[pyclass(name = "TimeScaleTransformer")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PyTimeScaleTransformer(TimeScaleTransformer);

#[pyclass(name = "Time")]
#[derive(Clone, Debug, Eq, PartialEq)]
struct PyTime(Time<PyTimeScale>);

impl JulianDate for PyTime {
    fn julian_date(
        &self,
        epoch: crate::julian_dates::Epoch,
        unit: crate::julian_dates::Unit,
    ) -> f64 {
        self.0.julian_date(epoch, unit)
    }

    fn two_part_julian_date(&self) -> (f64, f64) {
        self.0.two_part_julian_date()
    }
}
