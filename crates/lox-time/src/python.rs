use std::fmt::{Display, Formatter};
use std::str::FromStr;

use pyo3::PyResult;
use pyo3::{exceptions::PyValueError, pyclass, pymethods, PyErr};

use crate::calendar_dates::{CalendarDate, Date};
use crate::deltas::{TimeDelta, ToDelta};
use crate::julian_dates::{Epoch, Unit};
use crate::time_of_day::{CivilTime, TimeOfDay};
use crate::time_scales::TimeScale;
use crate::transformations::{
    NoOpOffsetProvider, ToTai, ToTcb, ToTcg, ToTdb, ToTt, ToUt1, TryToScale,
};
use crate::ut1::{DeltaUt1Tai, DeltaUt1TaiError, ExtrapolatedDeltaUt1Tai};
use crate::utc::leap_seconds::BuiltinLeapSeconds;
use crate::utc::transformations::ToUtc;
use crate::utc::{Utc, UtcError};
use crate::TimeError;
use crate::{
    julian_dates::JulianDate,
    time_scales::{Tai, Tcb, Tcg, Tdb, Tt, Ut1},
    Time,
};

pub mod deltas;

impl From<TimeError> for PyErr {
    fn from(value: TimeError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

impl From<UtcError> for PyErr {
    fn from(value: UtcError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

impl From<ExtrapolatedDeltaUt1Tai> for PyErr {
    fn from(value: ExtrapolatedDeltaUt1Tai) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

impl From<DeltaUt1TaiError> for PyErr {
    fn from(value: DeltaUt1TaiError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PyTimeScale {
    Tai,
    Tcb,
    Tcg,
    Tdb,
    Tt,
    Ut1,
}

impl FromStr for PyTimeScale {
    type Err = PyErr;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
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
    #[new]
    #[pyo3(signature=(scale, year, month, day, hour = 0, minute = 0, seconds = 0.0))]
    fn new(
        scale: &str,
        year: i64,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        seconds: f64,
    ) -> PyResult<PyTime> {
        let scale = PyTimeScale::from_str(scale)?;
        let time = Time::builder_with_scale(scale)
            .with_ymd(year, month, day)
            .with_hms(hour, minute, seconds)
            .build()?;
        Ok(PyTime(time))
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "Time({}, {}, {}, {}, {}, {}, {})",
            self.scale(),
            self.0.year(),
            self.0.month(),
            self.0.day(),
            self.0.hour(),
            self.0.minute(),
            self.0.decimal_seconds(),
        )
    }

    fn scale(&self) -> &'static str {
        self.0.scale().abbreviation()
    }

    fn to_tai(&self, provider: Option<PyUt1Provider>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Tai, &provider.0)?,
            None => self.try_to_scale(Tai, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tai)))
    }

    fn to_tcb(&self, provider: Option<PyUt1Provider>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Tcb, &provider.0)?,
            None => self.try_to_scale(Tcb, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tcb)))
    }

    fn to_tcg(&self, provider: Option<PyUt1Provider>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Tcg, &provider.0)?,
            None => self.try_to_scale(Tcg, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tcg)))
    }

    fn to_tdb(&self, provider: Option<PyUt1Provider>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Tdb, &provider.0)?,
            None => self.try_to_scale(Tdb, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tdb)))
    }

    fn to_tt(&self, provider: Option<PyUt1Provider>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Tt, &provider.0)?,
            None => self.try_to_scale(Tt, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tt)))
    }

    fn to_ut1(&self, provider: Option<PyUt1Provider>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Ut1, &provider.0)?,
            None => self.try_to_scale(Ut1, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Ut1)))
    }

    fn to_utc(&self, provider: Option<PyUt1Provider>) -> PyResult<PyUtc> {
        let tai = match provider {
            Some(provider) => self.try_to_scale(Tai, &provider.0)?,
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

#[pymethods]
impl PyUtc {
    #[new]
    #[pyo3(signature = (year, month, day, hour = 0, minute = 0, seconds = 0.0))]
    fn new(year: i64, month: u8, day: u8, hour: u8, minute: u8, seconds: f64) -> PyResult<PyUtc> {
        let utc = Utc::builder()
            .with_ymd(year, month, day)
            .with_hms(hour, minute, seconds)
            .build()?;
        Ok(PyUtc(utc))
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "UTC({}, {}, {}, {}, {}, {})",
            self.0.year(),
            self.0.month(),
            self.0.day(),
            self.0.hour(),
            self.0.minute(),
            self.0.decimal_seconds()
        )
    }

    fn __eq__(&self, other: PyUtc) -> bool {
        self.0 == other.0
    }

    fn year(&self) -> i64 {
        self.0.year()
    }

    fn month(&self) -> u8 {
        self.0.month()
    }

    fn day(&self) -> u8 {
        self.0.day()
    }

    fn hour(&self) -> u8 {
        self.0.hour()
    }

    fn minute(&self) -> u8 {
        self.0.minute()
    }

    fn second(&self) -> u8 {
        self.0.second()
    }

    fn millisecond(&self) -> i64 {
        self.0.millisecond()
    }

    fn microsecond(&self) -> i64 {
        self.0.microsecond()
    }

    fn nanosecond(&self) -> i64 {
        self.0.nanosecond()
    }

    fn picosecond(&self) -> i64 {
        self.0.picosecond()
    }

    fn decimal_seconds(&self) -> f64 {
        self.0.decimal_seconds()
    }

    fn to_tai(&self) -> PyTime {
        PyTime(self.0.to_tai().with_scale(PyTimeScale::Tai))
    }

    fn to_tcb(&self) -> PyTime {
        PyTime(self.0.to_tcb().with_scale(PyTimeScale::Tcb))
    }

    fn to_tcg(&self) -> PyTime {
        PyTime(self.0.to_tcg().with_scale(PyTimeScale::Tcg))
    }

    fn to_tdb(&self) -> PyTime {
        PyTime(self.0.to_tdb().with_scale(PyTimeScale::Tdb))
    }

    fn to_tt(&self) -> PyTime {
        PyTime(self.0.to_tt().with_scale(PyTimeScale::Tt))
    }

    fn to_ut1(&self, provider: PyUt1Provider) -> PyResult<PyTime> {
        Ok(PyTime(
            self.0.try_to_ut1(&provider.0)?.with_scale(PyTimeScale::Ut1),
        ))
    }
}

#[pyclass(name = "UT1Provider", module = "lox_space")]
#[derive(Clone, Debug, PartialEq)]
pub struct PyUt1Provider(DeltaUt1Tai);

#[pymethods]
impl PyUt1Provider {
    #[new]
    fn new(path: &str) -> PyResult<PyUt1Provider> {
        let provider = DeltaUt1Tai::new(path, &BuiltinLeapSeconds)?;
        Ok(PyUt1Provider(provider))
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
        let scale = PyTimeScale::from_str(abbreviation).unwrap();
        assert_eq!(scale.abbreviation(), abbreviation);
        assert_eq!(scale.name(), name);
    }

    #[test]
    fn test_pytime_scale() {
        let time = PyTime(time!(PyTimeScale::Tai, 2000, 1, 1, 12).unwrap());
        assert_eq!(time.scale(), "TAI".to_string());
    }

    #[test]
    fn test_pytime_julian_date() {
        let time = PyTime(time!(PyTimeScale::Tai, 2000, 1, 1, 12).unwrap());
        assert_eq!(time.seconds_since_j2000(), 0.0);
    }
}
