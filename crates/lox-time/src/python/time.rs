/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::calendar_dates::{CalendarDate, Date};
use crate::deltas::{TimeDelta, ToDelta};
use crate::julian_dates::{Epoch, JulianDate, Unit};
use crate::prelude::{CivilTime, Tai, Tcb, Tcg, Tdb, TimeOfDay, TimeScale, Tt, Ut1};
use crate::python::time_scales::PyTimeScale;
use crate::python::ut1::PyUt1Provider;
use crate::python::utc::PyUtc;
use crate::transformations::{NoOpOffsetProvider, ToTai, ToTcb, ToTcg, ToTdb, ToTt, TryToScale};
use crate::ut1::{DeltaUt1Tai, ExtrapolatedDeltaUt1Tai};
use crate::utc::transformations::ToUtc;
use crate::{Time, TimeError};
use pyo3::exceptions::PyValueError;
use pyo3::{pyclass, pymethods, Bound, PyErr, PyResult};
use std::str::FromStr;

impl From<TimeError> for PyErr {
    fn from(value: TimeError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

#[pyclass(name = "Time", module = "lox_space")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyTime(pub Time<PyTimeScale>);

#[pymethods]
impl PyTime {
    #[new]
    #[pyo3(signature=(scale, year, month, day, hour = 0, minute = 0, seconds = 0.0))]
    pub fn new(
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

    pub fn __str__(&self) -> String {
        self.0.to_string()
    }

    pub fn __repr__(&self) -> String {
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

    pub fn scale(&self) -> &'static str {
        self.0.scale().abbreviation()
    }

    pub fn to_tai<'py>(&self, provider: Option<&Bound<'py, PyUt1Provider>>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Tai, &provider.borrow().0)?,
            None => self.try_to_scale(Tai, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tai)))
    }

    pub fn to_tcb<'py>(&self, provider: Option<&Bound<'py, PyUt1Provider>>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Tcb, &provider.borrow().0)?,
            None => self.try_to_scale(Tcb, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tcb)))
    }

    pub fn to_tcg<'py>(&self, provider: Option<&Bound<'py, PyUt1Provider>>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Tcg, &provider.borrow().0)?,
            None => self.try_to_scale(Tcg, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tcg)))
    }

    pub fn to_tdb<'py>(&self, provider: Option<&Bound<'py, PyUt1Provider>>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Tdb, &provider.borrow().0)?,
            None => self.try_to_scale(Tdb, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tdb)))
    }

    pub fn to_tt<'py>(&self, provider: Option<&Bound<'py, PyUt1Provider>>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Tt, &provider.borrow().0)?,
            None => self.try_to_scale(Tt, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Tt)))
    }

    pub fn to_ut1<'py>(&self, provider: Option<&Bound<'py, PyUt1Provider>>) -> PyResult<PyTime> {
        let time = match provider {
            Some(provider) => self.try_to_scale(Ut1, &provider.borrow().0)?,
            None => self.try_to_scale(Ut1, &NoOpOffsetProvider)?,
        };
        Ok(PyTime(time.with_scale(PyTimeScale::Ut1)))
    }

    pub fn to_utc<'py>(&self, provider: Option<&Bound<'py, PyUt1Provider>>) -> PyResult<PyUtc> {
        let tai = match provider {
            Some(provider) => self.try_to_scale(Tai, &provider.borrow().0)?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time;

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
