// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::ops::{Add, Sub};
use std::str::FromStr;

use lox_test_utils::{ApproxEq, approx_eq};
use lox_time::subsecond::Subsecond;
use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyException, PyTypeError, PyValueError};
use pyo3::types::{PyAnyMethods, PyType};
use pyo3::{
    Bound, IntoPyObjectExt, Py, PyAny, PyErr, PyResult, Python, create_exception, pyclass,
    pymethods,
};

use crate::earth::python::ut1::{PyEopProvider, PyEopProviderError};
use crate::time::calendar_dates::{CalendarDate, Date};
use crate::time::deltas::{TimeDelta, ToDelta};
use crate::time::julian_dates::{Epoch, JulianDate, Unit};
use crate::time::python::deltas::PyTimeDelta;
use crate::time::python::utc::PyUtcError;
use crate::time::time::{DynTime, Time, TimeError};
use crate::time::time_of_day::{CivilTime, TimeOfDay};
use crate::time::time_scales::Tai;
use crate::time::utc::transformations::TryToUtc;

use super::time_scales::PyTimeScale;
use super::utc::PyUtc;

create_exception!(lox_space, NonFiniteTimeError, PyException);

pub struct PyTimeError(pub TimeError);

impl From<PyTimeError> for PyErr {
    fn from(err: PyTimeError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

pub struct PyEpoch(pub Epoch);

impl FromStr for PyEpoch {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "jd" | "JD" => Ok(Epoch::JulianDate),
            "mjd" | "MJD" => Ok(Epoch::ModifiedJulianDate),
            "j1950" | "J1950" => Ok(Epoch::J1950),
            "j2000" | "J2000" => Ok(Epoch::J2000),
            _ => Err(PyValueError::new_err(format!("unknown epoch: {s}"))),
        }
        .map(PyEpoch)
    }
}

pub struct PyUnit(pub Unit);

impl FromStr for PyUnit {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "seconds" => Ok(Unit::Seconds),
            "days" => Ok(Unit::Days),
            "centuries" => Ok(Unit::Centuries),
            _ => Err(PyValueError::new_err(format!("unknown unit: {s}"))),
        }
        .map(PyUnit)
    }
}

/// Represents an instant in time on a specific astronomical time scale.
///
/// `Time` is the fundamental time representation in lox, providing
/// femtosecond precision and support for multiple astronomical time scales
/// (TAI, TT, TDB, TCB, TCG, UT1).
///
/// Args:
///     scale: Time scale ("TAI", "TT", "TDB", "TCB", "TCG", "UT1") or TimeScale object.
///     year: Calendar year.
///     month: Calendar month (1-12).
///     day: Day of month (1-31).
///     hour: Hour of day (0-23). Defaults to 0.
///     minute: Minute of hour (0-59). Defaults to 0.
///     seconds: Seconds with fractional part (0.0-60.0). Defaults to 0.0.
///
/// Raises:
///     ValueError: If date or time components are out of valid range.
///
/// See Also:
///     TimeDelta: For representing time differences.
///     UTC: For UTC time with leap second handling.
#[pyclass(name = "Time", module = "lox_space", frozen)]
#[derive(Clone, Debug, Eq, PartialEq, ApproxEq)]
pub struct PyTime(pub DynTime);

#[pymethods]
impl PyTime {
    #[new]
    #[pyo3(signature=(scale, year, month, day, hour = 0, minute = 0, seconds = 0.0))]
    pub fn new(
        scale: &Bound<'_, PyAny>,
        year: i64,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        seconds: f64,
    ) -> PyResult<PyTime> {
        let scale: PyTimeScale = scale.try_into()?;
        let time = Time::builder_with_scale(scale.0)
            .with_ymd(year, month, day)
            .with_hms(hour, minute, seconds)
            .build()
            .map_err(PyTimeError)?;
        Ok(PyTime(time))
    }

    fn __getnewargs__<'py>(
        &self,
        py: Python<'py>,
    ) -> (Bound<'py, PyAny>, i64, u8, u8, u8, u8, f64) {
        (
            self.scale().into_bound_py_any(py).unwrap(),
            self.0.year(),
            self.0.month(),
            self.0.day(),
            self.0.hour(),
            self.0.minute(),
            self.0.as_seconds_f64(),
        )
    }

    /// Create a Time from a Julian date.
    ///
    /// Args:
    ///     scale: Time scale for the resulting Time object.
    ///     jd: Julian date value.
    ///     epoch: Reference epoch ("jd", "mjd", "j1950", "j2000"). Defaults to "jd".
    ///
    /// Returns:
    ///     A new Time object.
    ///
    /// Examples:
    ///     >>> t = Time.from_julian_date("TAI", 2451545.0, "jd")  # J2000.0
    #[classmethod]
    #[pyo3(signature = (scale, jd, epoch = "jd"))]
    pub fn from_julian_date(
        _cls: &Bound<'_, PyType>,
        scale: &Bound<'_, PyAny>,
        jd: f64,
        epoch: &str,
    ) -> PyResult<Self> {
        let scale: PyTimeScale = scale.try_into()?;
        let epoch: PyEpoch = epoch.parse()?;
        Ok(Self(Time::from_julian_date(scale.0, jd, epoch.0)))
    }

    /// Create a Time from a two-part Julian date for maximum precision.
    ///
    /// This method preserves full precision by accepting the Julian date
    /// as two separate float components that are added together.
    ///
    /// Args:
    ///     scale: Time scale for the resulting Time object.
    ///     jd1: First part of the Julian date (typically the integer part).
    ///     jd2: Second part of the Julian date (typically the fractional part).
    ///
    /// Returns:
    ///     A new Time object.
    #[classmethod]
    pub fn from_two_part_julian_date(
        _cls: &Bound<'_, PyType>,
        scale: &Bound<'_, PyAny>,
        jd1: f64,
        jd2: f64,
    ) -> PyResult<Self> {
        let scale: PyTimeScale = scale.try_into()?;
        Ok(Self(Time::from_two_part_julian_date(scale.0, jd1, jd2)))
    }

    /// Create a Time from year and day of year.
    ///
    /// Args:
    ///     scale: Time scale for the resulting Time object.
    ///     year: Calendar year.
    ///     day: Day of year (1-366).
    ///     hour: Hour of day (0-23). Defaults to 0.
    ///     minute: Minute of hour (0-59). Defaults to 0.
    ///     seconds: Seconds with fractional part. Defaults to 0.0.
    ///
    /// Returns:
    ///     A new Time object.
    ///
    /// Raises:
    ///     ValueError: If day of year is out of range for the given year.
    ///
    /// Examples:
    ///     >>> t = Time.from_day_of_year("TAI", 2024, 1)  # Jan 1, 2024
    ///     >>> t = Time.from_day_of_year("TAI", 2024, 366)  # Dec 31, 2024 (leap year)
    #[classmethod]
    #[pyo3(signature=(scale, year, day, hour=0, minute=0, seconds=0.0))]
    pub fn from_day_of_year(
        _cls: &Bound<'_, PyType>,
        scale: &Bound<'_, PyAny>,
        year: i64,
        day: u16,
        hour: u8,
        minute: u8,
        seconds: f64,
    ) -> PyResult<PyTime> {
        let scale: PyTimeScale = scale.try_into()?;
        let time = Time::builder_with_scale(scale.0)
            .with_doy(year, day)
            .with_hms(hour, minute, seconds)
            .build()
            .map_err(PyTimeError)?;
        Ok(PyTime(time))
    }

    /// Create a Time from an ISO 8601 formatted string.
    ///
    /// Args:
    ///     iso: ISO 8601 formatted datetime string (e.g., "2024-06-15T12:30:45.5 TAI").
    ///     scale: Time scale. If not provided, the scale must be in the ISO string.
    ///
    /// Returns:
    ///     A new Time object.
    ///
    /// Raises:
    ///     ValueError: If the ISO string is invalid or the scale cannot be determined.
    ///
    /// Examples:
    ///     >>> t = Time.from_iso("2024-06-15T12:30:45.5 TAI")
    ///     >>> t = Time.from_iso("2024-06-15T12:30:45.5", "TAI")
    #[classmethod]
    #[pyo3(signature = (iso, scale=None))]
    pub fn from_iso(
        _cls: &Bound<'_, PyType>,
        iso: &str,
        scale: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<PyTime> {
        let scale: PyTimeScale =
            scale.map_or(Ok(PyTimeScale::default()), |scale| scale.try_into())?;
        let time = Time::from_iso(scale.0, iso).map_err(PyTimeError)?;
        Ok(PyTime(time))
    }

    /// Create a Time from raw seconds and subsecond components.
    ///
    /// This is a low-level constructor for maximum precision.
    ///
    /// Args:
    ///     scale: Time scale for the resulting Time object.
    ///     seconds: Integer seconds since the internal epoch.
    ///     subsecond: Fractional second component (0.0 to 1.0).
    ///
    /// Returns:
    ///     A new Time object.
    ///
    /// Raises:
    ///     ValueError: If subsecond is not in the valid range.
    #[classmethod]
    pub fn from_seconds(
        _cls: &Bound<'_, PyType>,
        scale: &Bound<'_, PyAny>,
        seconds: i64,
        subsecond: f64,
    ) -> PyResult<PyTime> {
        let scale: PyTimeScale = scale.try_into()?;
        let subsecond =
            Subsecond::from_f64(subsecond).ok_or(PyValueError::new_err("invalid subsecond"))?;
        let time = Time::new(scale.0, seconds, subsecond);
        Ok(PyTime(time))
    }

    /// Return the integer seconds component of the internal representation.
    ///
    /// Returns:
    ///     Integer seconds since the internal epoch.
    ///
    /// Raises:
    ///     NonFiniteTimeError: If the time is non-finite.
    pub fn seconds(&self) -> PyResult<i64> {
        self.0.seconds().ok_or(NonFiniteTimeError::new_err(
            "cannot access seconds for non-finite time",
        ))
    }

    /// Return the subsecond (fractional second) component.
    ///
    /// Returns:
    ///     Fractional seconds (0.0 to 1.0).
    ///
    /// Raises:
    ///     NonFiniteTimeError: If the time is non-finite.
    pub fn subsecond(&self) -> PyResult<f64> {
        self.0.subsecond().ok_or(NonFiniteTimeError::new_err(
            "cannot access subsecond for non-finite time",
        ))
    }

    #[classattr]
    const __hash__: Option<Py<PyAny>> = None;

    pub fn __str__(&self) -> String {
        self.0.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Time(\"{}\", {}, {}, {}, {}, {}, {})",
            self.scale().abbreviation(),
            self.0.year(),
            self.0.month(),
            self.0.day(),
            self.0.hour(),
            self.0.minute(),
            self.0.as_seconds_f64(),
        )
    }

    pub fn __add__(&self, delta: PyTimeDelta) -> Self {
        PyTime(self.0 + delta.0)
    }

    pub fn __sub__<'py>(
        &self,
        py: Python<'py>,
        rhs: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Ok(delta) = rhs.extract::<PyTimeDelta>() {
            Ok(Bound::new(py, PyTime(self.0 - delta.0))?.into_any())
        } else if let Ok(rhs) = rhs.extract::<PyTime>() {
            if self.scale() != rhs.scale() {
                return Err(PyValueError::new_err(
                    "cannot subtract `Time` objects with different time scales",
                ));
            }
            Ok(Bound::new(py, PyTimeDelta(self.0 - rhs.0))?.into_any())
        } else {
            Err(PyTypeError::new_err(
                "`rhs` must be either a `Time` or a `TimeDelta` object",
            ))
        }
    }

    fn __richcmp__(&self, other: PyTime, op: CompareOp) -> bool {
        op.matches(self.0.cmp(&other.0))
    }

    /// Check if two Time objects are approximately equal.
    ///
    /// Args:
    ///     rhs: The other Time object to compare.
    ///     rel_tol: Relative tolerance. Defaults to 1e-8.
    ///     abs_tol: Absolute tolerance. Defaults to 1e-14.
    ///
    /// Returns:
    ///     True if the times are approximately equal within the tolerances.
    ///
    /// Raises:
    ///     ValueError: If the Time objects have different time scales.
    #[pyo3(signature = (rhs, rel_tol = 1e-8, abs_tol = 1e-14))]
    pub fn isclose(&self, rhs: PyTime, rel_tol: f64, abs_tol: f64) -> PyResult<bool> {
        if self.scale() != rhs.scale() {
            return Err(PyValueError::new_err(
                "cannot compare `Time` objects with different time scales",
            ));
        }
        Ok(approx_eq!(self, &rhs, rtol <= rel_tol, atol <= abs_tol))
    }

    /// Return the Julian date relative to the specified epoch.
    ///
    /// Args:
    ///     epoch: Reference epoch ("jd", "mjd", "j1950", "j2000"). Defaults to "jd".
    ///     unit: Output unit ("seconds", "days", "centuries"). Defaults to "days".
    ///
    /// Returns:
    ///     The Julian date in the specified units relative to the epoch.
    ///
    /// Raises:
    ///     ValueError: If epoch or unit is invalid.
    #[pyo3(signature = (epoch = "jd", unit = "days"))]
    pub fn julian_date(&self, epoch: &str, unit: &str) -> PyResult<f64> {
        let epoch: PyEpoch = epoch.parse()?;
        let unit: PyUnit = unit.parse()?;
        Ok(self.0.julian_date(epoch.0, unit.0))
    }

    /// Return the two-part Julian date for maximum precision.
    ///
    /// Returns:
    ///     A tuple of (jd1, jd2) where the Julian date is jd1 + jd2.
    pub fn two_part_julian_date(&self) -> (f64, f64) {
        self.0.two_part_julian_date()
    }

    /// Return the time scale of this Time object.
    ///
    /// Returns:
    ///     The TimeScale of this Time.
    pub fn scale(&self) -> PyTimeScale {
        PyTimeScale(self.0.scale())
    }

    /// Return the year component.
    pub fn year(&self) -> i64 {
        self.0.year()
    }

    /// Return the month component (1-12).
    pub fn month(&self) -> u8 {
        self.0.month()
    }

    /// Return the day of month component (1-31).
    pub fn day(&self) -> u8 {
        self.0.day()
    }

    /// Return the day of year (1-366).
    pub fn day_of_year(&self) -> u16 {
        self.0.day_of_year()
    }

    /// Return the hour component (0-23).
    pub fn hour(&self) -> u8 {
        self.0.hour()
    }

    /// Return the minute component (0-59).
    pub fn minute(&self) -> u8 {
        self.0.minute()
    }

    /// Return the integer second component (0-59, or 60 for leap second).
    pub fn second(&self) -> u8 {
        self.0.second()
    }

    /// Return the millisecond component (0-999).
    pub fn millisecond(&self) -> u32 {
        self.0.millisecond()
    }

    /// Return the microsecond component (0-999).
    pub fn microsecond(&self) -> u32 {
        self.0.microsecond()
    }

    /// Return the nanosecond component (0-999).
    pub fn nanosecond(&self) -> u32 {
        self.0.nanosecond()
    }

    /// Return the picosecond component (0-999).
    pub fn picosecond(&self) -> u32 {
        self.0.picosecond()
    }

    /// Return the femtosecond component (0-999).
    pub fn femtosecond(&self) -> u32 {
        self.0.femtosecond()
    }

    /// Return the decimal seconds (seconds + fractional part).
    pub fn decimal_seconds(&self) -> f64 {
        self.0.as_seconds_f64()
    }

    /// Convert this Time to another time scale.
    ///
    /// Args:
    ///     scale: Target time scale.
    ///     provider: EOP provider for UT1 conversions. Required when converting
    ///         to/from UT1.
    ///
    /// Returns:
    ///     A new Time object in the target scale.
    ///
    /// Raises:
    ///     ValueError: If conversion requires EOP data but no provider is given.
    ///
    /// Examples:
    ///     >>> t_tai = Time("TAI", 2024, 1, 1)
    ///     >>> t_tt = t_tai.to_scale("TT")
    #[pyo3(signature = (scale, provider=None))]
    pub fn to_scale(
        &self,
        scale: &Bound<'_, PyAny>,
        provider: Option<&Bound<'_, PyEopProvider>>,
    ) -> PyResult<PyTime> {
        let scale: PyTimeScale = scale.try_into()?;
        let provider = provider.map(|p| &p.get().0);
        let time = match provider {
            Some(provider) => self
                .0
                .try_to_scale(scale.0, provider)
                .map_err(PyEopProviderError)?,
            None => self.0.to_scale(scale.0),
        };
        Ok(PyTime(time))
    }

    /// Convert this Time to UTC.
    ///
    /// Args:
    ///     provider: EOP provider for UT1 conversions.
    ///
    /// Returns:
    ///     A UTC object representing this instant in UTC.
    ///
    /// Raises:
    ///     ValueError: If the time is outside the valid UTC range.
    #[pyo3(signature = (provider=None))]
    pub fn to_utc(&self, provider: Option<&Bound<'_, PyEopProvider>>) -> PyResult<PyUtc> {
        let provider = provider.map(|p| &p.get().0);
        let utc = match provider {
            Some(provider) => self
                .0
                .try_to_scale(Tai, provider)
                .map_err(PyEopProviderError)?,
            None => self.0.to_scale(Tai),
        }
        .try_to_utc()
        .map_err(PyUtcError)?;
        Ok(PyUtc(utc))
    }
}

impl ToDelta for PyTime {
    fn to_delta(&self) -> TimeDelta {
        self.0.to_delta()
    }
}

impl JulianDate for PyTime {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        self.0.julian_date(epoch, unit)
    }
}

impl Add<TimeDelta> for PyTime {
    type Output = PyTime;

    fn add(self, rhs: TimeDelta) -> Self::Output {
        PyTime(self.0 + rhs)
    }
}

impl Sub<TimeDelta> for PyTime {
    type Output = PyTime;

    fn sub(self, rhs: TimeDelta) -> Self::Output {
        PyTime(self.0 - rhs)
    }
}

impl Sub<PyTime> for PyTime {
    type Output = TimeDelta;

    fn sub(self, rhs: PyTime) -> TimeDelta {
        self.0 - rhs.0
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
