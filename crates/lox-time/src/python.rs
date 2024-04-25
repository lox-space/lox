use std::fmt::{Display, Formatter};

use pyo3::{exceptions::PyValueError, pyclass, pymethods, PyErr};

use crate::julian_dates::{Epoch, Unit};
use crate::time_scales::TimeScale;
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

#[pyclass(name = "Time")]
#[derive(Clone, Debug, Eq, PartialEq)]
struct PyTime(Time<PyTimeScale>);

#[pymethods]
impl PyTime {
    fn scale(&self) -> PyTimeScale {
        self.0.scale()
    }
}

impl JulianDate for PyTime {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        self.0.julian_date(epoch, unit)
    }
}

#[cfg(test)]
mod tests {
    use crate::time;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("TAI", "International Atomic Time")]
    #[case("TT", "Terrestrial Time")]
    #[case("TCG", "Geocentric Coordinate Time")]
    #[case("TCB", "Barycentric Coordinate Time")]
    #[case("TDB", "Barycentric Dynamical Time")]
    #[case("UT1", "Universal Time")]
    #[should_panic(expected = "invalid timescale: NotATimeScale")]
    #[case("NotATimeScale", "not a timescale")]
    fn test_pytimescale(#[case] abbreviation: &'static str, #[case] name: &'static str) {
        let scale = PyTimeScale::new(abbreviation).unwrap();
        assert_eq!(scale.abbreviation(), abbreviation);
        assert_eq!(scale.name(), name);
        assert_eq!(scale.__repr__(), format!("TimeScale(\"{}\")", abbreviation));
        assert_eq!(scale.__str__(), abbreviation);
    }

    #[test]
    fn test_pytime_scale() {
        let time = PyTime(time!(PyTimeScale::Tai, 2000, 1, 1, 12).unwrap());
        assert_eq!(time.scale(), PyTimeScale::Tai);
    }

    #[test]
    fn test_pytime_julian_date() {
        let time = PyTime(time!(PyTimeScale::Tai, 2000, 1, 1, 12).unwrap());
        assert_eq!(time.seconds_since_j2000(), 0.0);
    }
}
