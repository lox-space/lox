use std::convert::Infallible;
use std::fmt::{Display, Formatter};

use pyo3::{exceptions::PyValueError, pyclass, pymethods, PyErr};

use crate::calendar_dates::{CalendarDate, Date};
use crate::deltas::{TimeDelta, ToDelta};
use crate::julian_dates::{Epoch, Unit};
use crate::time_of_day::{CivilTime, TimeOfDay};
use crate::time_scales::TimeScale;
use crate::transformations::{ToTai, ToTt, TryToScale};
use crate::ut1::{DeltaUt1Tai, ExtrapolatedDeltaUt1Tai};
use crate::utc::Utc;
use crate::{
    julian_dates::JulianDate,
    time_scales::{Tai, Tcb, Tcg, Tdb, Tt, Ut1},
    Time,
};

#[pyclass(name = "TimeScale", module = "lox_space")]
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

#[pyclass(name = "Time", module = "lox_space")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyTime(Time<PyTimeScale>);

#[pymethods]
impl PyTime {
    fn scale(&self) -> PyTimeScale {
        self.0.scale()
    }

    fn to_tai(&self, provider: Option<PyDeltaUt1Tai>) -> Result<PyTime, PyErr> {
        let time = match provider {
            Some(provider) => self
                .try_to_scale(Tai, &provider.0)
                .map_err(|_| PyValueError::new_err("unable to convert to TAI"))?,
            None => {
                if self.scale() == PyTimeScale::Ut1 {
                    return Err(PyValueError::new_err("missing UT1 provider"));
                }
                self.try_to_scale(Tai, &())
                    .map_err(|_| PyValueError::new_err("unable to convert to TAI"))?
            }
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tai)))
    }

    fn to_tt(&self, provider: Option<PyDeltaUt1Tai>) -> Result<PyTime, PyErr> {
        let time = match provider {
            Some(provider) => self
                .try_to_scale(Tt, &provider.0)
                .map_err(|_| PyValueError::new_err("unable to convert to Tt"))?,
            None => {
                if self.scale() == PyTimeScale::Ut1 {
                    return Err(PyValueError::new_err("missing UT1 provider"));
                }
                self.try_to_scale(Tt, &())
                    .map_err(|_| PyValueError::new_err("unable to convert to Tt"))?
            }
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tt)))
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
        match self.scale() {
            PyTimeScale::Tai => Ok(self.0.with_scale(Tai)),
            PyTimeScale::Tcb => Ok(self.0.with_scale(Tcb).to_tai()),
            PyTimeScale::Tcg => Ok(self.0.with_scale(Tcg).to_tai()),
            PyTimeScale::Tdb => Ok(self.0.with_scale(Tdb).to_tai()),
            PyTimeScale::Tt => Ok(self.0.with_scale(Tt).to_tai()),
            PyTimeScale::Ut1 => self.0.with_scale(Ut1).try_to_scale(Tai, provider),
        }
    }
}

impl TryToScale<Tai> for PyTime {
    fn try_to_scale(&self, _scale: Tai, _provider: &()) -> Result<Time<Tai>, Infallible> {
        match self.scale() {
            PyTimeScale::Tai => Ok(self.0.with_scale(Tai)),
            PyTimeScale::Tcb => Ok(self.0.with_scale(Tcb).to_tai()),
            PyTimeScale::Tcg => Ok(self.0.with_scale(Tcg).to_tai()),
            PyTimeScale::Tdb => Ok(self.0.with_scale(Tdb).to_tai()),
            PyTimeScale::Tt => Ok(self.0.with_scale(Tt).to_tai()),
            PyTimeScale::Ut1 => unreachable!("invalid for UT1"),
        }
    }
}

impl TryToScale<Tt, DeltaUt1Tai, ExtrapolatedDeltaUt1Tai> for PyTime {
    fn try_to_scale(
        &self,
        _scale: Tt,
        provider: &DeltaUt1Tai,
    ) -> Result<Time<Tt>, ExtrapolatedDeltaUt1Tai> {
        match self.scale() {
            PyTimeScale::Tai => Ok(self.0.with_scale(Tai).to_tt()),
            PyTimeScale::Tcb => Ok(self.0.with_scale(Tcb).to_tt()),
            PyTimeScale::Tcg => Ok(self.0.with_scale(Tcg).to_tt()),
            PyTimeScale::Tdb => Ok(self.0.with_scale(Tdb).to_tt()),
            PyTimeScale::Tt => Ok(self.0.with_scale(Tt)),
            PyTimeScale::Ut1 => self.0.with_scale(Ut1).try_to_scale(Tt, provider),
        }
    }
}

impl TryToScale<Tt> for PyTime {
    fn try_to_scale(&self, _scale: Tt, _provider: &()) -> Result<Time<Tt>, Infallible> {
        match self.scale() {
            PyTimeScale::Tai => Ok(self.0.with_scale(Tai).to_tt()),
            PyTimeScale::Tcb => Ok(self.0.with_scale(Tcb).to_tt()),
            PyTimeScale::Tcg => Ok(self.0.with_scale(Tcg).to_tt()),
            PyTimeScale::Tdb => Ok(self.0.with_scale(Tdb).to_tt()),
            PyTimeScale::Tt => Ok(self.0.with_scale(Tt)),
            PyTimeScale::Ut1 => unreachable!("invalid for UT1"),
        }
    }
}

impl JulianDate for PyTime {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        self.0.julian_date(epoch, unit)
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

#[pyclass(name = "UTC", module = "lox_space")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyUtc(Utc);

#[pyclass(name = "DeltaUT1TAI", module = "lox_space")]
#[derive(Clone, Debug, PartialEq)]
pub struct PyDeltaUt1Tai(DeltaUt1Tai);

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
