// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

use std::ops::{Add, Sub};
use std::str::FromStr;

use js_sys::Array;
use wasm_bindgen::prelude::*;

use lox_test_utils::{approx_eq, ApproxEq};
use lox_time::subsecond::Subsecond;

use crate::earth::wasm::ut1::{JsEopProvider, JsEopProviderError};
use crate::time::calendar_dates::{CalendarDate, Date};
use crate::time::deltas::{TimeDelta, ToDelta};
use crate::time::julian_dates::{Epoch, JulianDate, Unit};
use crate::time::time::{DynTime, Time, TimeError};
use crate::time::time_of_day::{CivilTime, TimeOfDay};
use crate::time::time_scales::Tai;
use crate::time::utc::transformations::TryToUtc;
use crate::wasm::{js_error_with_name, js_error_with_name_from_string};

use super::time_scales::JsTimeScale;
use crate::time::wasm::deltas::JsTimeDelta;
use crate::time::wasm::utc::{JsUtc, JsUtcError};

pub struct JsTimeError(pub TimeError);

impl From<JsTimeError> for JsValue {
    fn from(err: JsTimeError) -> Self {
        js_error_with_name("TimeError", &err.0.to_string())
    }
}

pub struct JsEpoch(Epoch);

impl FromStr for JsEpoch {
    type Err = JsValue;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "jd" | "JD" => Ok(Epoch::JulianDate),
            "mjd" | "MJD" => Ok(Epoch::ModifiedJulianDate),
            "j1950" | "J1950" => Ok(Epoch::J1950),
            "j2000" | "J2000" => Ok(Epoch::J2000),
            _ => Err(js_error_with_name_from_string("ValueError", format!("unknown epoch: {s}"))),
        }
        .map(JsEpoch)
    }
}

pub struct JsUnit(Unit);

impl FromStr for JsUnit {
    type Err = JsValue;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "seconds" => Ok(Unit::Seconds),
            "days" => Ok(Unit::Days),
            "centuries" => Ok(Unit::Centuries),
            _ => Err(js_error_with_name_from_string("ValueError", format!("unknown unit: {s}"))),
        }
        .map(JsUnit)
    }
}

#[wasm_bindgen(js_name = "Times")]
#[derive(Clone)]
pub struct JsTimes(Vec<JsTime>);

#[wasm_bindgen(js_class = "Times")]
impl JsTimes {
    #[wasm_bindgen(js_name = "generateTimes")]
    pub fn generate_times(
        start: &JsTime,
        end: &JsTime,
        step: &JsTimeDelta
    ) -> Self {
        let mut times = Vec::new();
        let mut current = start.clone();
        let step = step.inner();
        while current.inner() <= end.inner() {
            times.push(current.clone());
            current = current.add(step);
        }
        JsTimes(times)
    }
}

impl JsTimes {
    pub fn inner(&self) -> Vec<JsTime> {
        self.0.clone()
    }
    pub fn from_inner(times: Vec<JsTime>) -> Self {
        Self(times)
    }

    pub fn vec_inner(&self) -> Vec<DynTime> {
        self.0.iter().map(|t| t.inner()).collect()
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
#[wasm_bindgen(js_name = "Time")]
#[derive(Clone)]
pub struct JsTime(DynTime);

#[wasm_bindgen(js_class = "Time")]
impl JsTime {
    #[wasm_bindgen(constructor)]
    pub fn new(
        scale: JsValue,
        year: i32,
        month: u8,
        day: u8,
        hour: Option<u8>,
        minute: Option<u8>,
        seconds: Option<f64>,
    ) -> Result<JsTime, JsValue> {
        let hour = hour.unwrap_or(0);
        let minute = minute.unwrap_or(0);
        let seconds = seconds.unwrap_or(0.0);
        let scale: JsTimeScale = scale.try_into()?;
        let time = Time::builder_with_scale(scale.inner())
            .with_ymd(year as i64, month, day)
            .with_hms(hour, minute, seconds)
            .build()
            .map_err(JsTimeError)?;
        Ok(JsTime(time))
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
    #[wasm_bindgen(js_name="fromJulianDate")]
    pub fn from_julian_date(
        scale: JsValue,
        jd: f64,
        epoch: Option<String>, // defaults to "jd" if empty
    ) -> Result<JsTime, JsValue> {
        let scale: JsTimeScale = scale.try_into()?;
        let epoch: JsEpoch = epoch.unwrap_or_else(|| "jd".to_string()).parse()?;
        Ok(Self(Time::from_julian_date(scale.inner(), jd, epoch.0)))
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
    #[wasm_bindgen(js_name="fromTwoPartJulianDate")]
    pub fn from_two_part_julian_date(
        scale: JsValue,
        jd1: f64,
        jd2: f64,
    ) -> Result<JsTime, JsValue> {
        let scale: JsTimeScale = scale.try_into()?;
        Ok(Self(Time::from_two_part_julian_date(scale.inner(), jd1, jd2)))
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
    #[wasm_bindgen(js_name="fromDayOfYear")]
    pub fn from_day_of_year(
        scale: JsValue,
        year: i32,
        day: u16,
        hour: Option<u8>,
        minute: Option<u8>,
        seconds: Option<f64>,
    ) -> Result<JsTime, JsValue> {
        let scale: JsTimeScale = scale.try_into()?;
        let time = Time::builder_with_scale(scale.inner())
            .with_doy(year as i64, day)
            .with_hms(hour.unwrap_or(0), minute.unwrap_or(0), seconds.unwrap_or(0.0))
            .build()
            .map_err(JsTimeError)?;
        Ok(JsTime(time))
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
    #[wasm_bindgen(js_name="fromISO")]
    pub fn from_iso(iso: &str, scale: Option<JsValue>) -> Result<JsTime, JsValue> {
        let scale: JsTimeScale =
            scale.map_or(Ok(JsTimeScale::default()), |scale| scale.try_into())?;
        let time = Time::from_iso(scale.inner(), iso).map_err(JsTimeError)?;
        Ok(JsTime(time))
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
    #[wasm_bindgen(js_name="fromSeconds")]
    pub fn from_seconds(
        scale: JsValue,
        seconds: i64,
        subsecond: f64,
    ) -> Result<JsTime, JsValue> {
        let scale: JsTimeScale = scale.try_into()?;
        let subsecond =
            Subsecond::from_f64(subsecond).ok_or_else(|| js_error_with_name("ValueError", "invalid subsecond"))?;
        let time = Time::new(scale.inner(), seconds, subsecond);
        Ok(JsTime(time))
    }

    /// Return the integer seconds component of the internal representation.
    ///
    /// Returns:
    ///     Integer seconds since the internal epoch.
    ///
    /// Raises:
    ///     NonFiniteTimeError: If the time is non-finite.
    pub fn seconds(&self) -> Result<i64, JsValue> {
        self.0.seconds().ok_or_else(|| {
            js_error_with_name("NonFiniteTimeError", "cannot access seconds for non-finite time")
        })
    }

    /// Return the subsecond (fractional second) component.
    ///
    /// Returns:
    ///     Fractional seconds (0.0 to 1.0).
    ///
    /// Raises:
    ///     NonFiniteTimeError: If the time is non-finite.
    pub fn subsecond(&self) -> Result<f64, JsValue> {
        self.0.subsecond().ok_or_else(|| {
            js_error_with_name(
                "NonFiniteTimeError",
                "cannot access subsecond for non-finite time",
            )
        })
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    pub fn repr(&self) -> String {
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

    pub fn add(&self, delta: &JsTimeDelta) -> JsTime {
        JsTime(self.0 + delta.inner())
    }

    /// Subtract a TimeDelta from this Time.
    #[wasm_bindgen(js_name = "subtractDelta")]
    pub fn subtract_delta(&self, delta: &JsTimeDelta) -> JsTime {
        JsTime(self.0 - delta.inner())
    }

    /// Compute the TimeDelta between this Time and another Time.
    #[wasm_bindgen(js_name = "subtractTime")]
    pub fn subtract_time(&self, rhs: &JsTime) -> Result<JsTimeDelta, JsValue> {
        if self.0.scale() != rhs.0.scale() {
            return Err(js_error_with_name(
                "ValueError",
                "cannot subtract `Time` objects with different time scales",
            ));
        }
        Ok(JsTimeDelta::from_inner(self.0 - rhs.0))
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
    #[wasm_bindgen(js_name = "isClose")]
    pub fn is_close(
        &self,
        rhs: &JsTime,
        rel_tol: Option<f64>,
        abs_tol: Option<f64>,
    ) -> Result<bool, JsValue> {
        if self.0.scale() != rhs.0.scale() {
            return Err(js_error_with_name(
                "ValueError",
                "cannot compare `Time` objects with different time scales",
            ));
        }
        let rel = rel_tol.unwrap_or(1e-8);
        let abs = abs_tol.unwrap_or(1e-14);
        Ok(approx_eq!(self.0, rhs.0, rtol <= rel, atol <= abs))
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
    #[wasm_bindgen(js_name = "julianDate")]
    pub fn julian_date(&self, epoch: Option<String>, unit: Option<String>) -> Result<f64, JsValue> {
        let epoch: JsEpoch = epoch.unwrap_or_else(|| "jd".to_string()).parse()?;
        let unit: JsUnit = unit.unwrap_or_else(|| "days".to_string()).parse()?;
        Ok(self.0.julian_date(epoch.0, unit.0))
    }

    /// Return the two-part Julian date for maximum precision.
    ///
    /// Returns:
    ///     A tuple of (jd1, jd2) where the Julian date is jd1 + jd2.
    #[wasm_bindgen(js_name = "twoPartJulianDate")]
    pub fn two_part_julian_date(&self) -> Array {
        let (jd1, jd2) = self.0.two_part_julian_date();
        Array::of2(&JsValue::from_f64(jd1), &JsValue::from_f64(jd2))
    }


    /// Return the time scale of this Time object.
    ///
    /// Returns:
    ///     The TimeScale of this Time.
    pub fn scale(&self) -> JsTimeScale {
        JsTimeScale::from_inner(self.0.scale())
    }

    /// Return the year component.
    pub fn year(&self) -> Result<i32, JsValue> {
        let year: i64 = self.0.year();

        if year > i32::MAX as i64 || year < i32::MIN as i64 {
                Err(js_error_with_name(
                    "OverflowError",
                    "seconds component out of range for i32",
                ))
        } else {
            Ok(year as i32)
        }
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
    #[wasm_bindgen(js_name = "dayOfYear")]
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
    #[wasm_bindgen(js_name = "decimalSeconds")]
    pub fn decimal_seconds(&self) -> f64 {
        self.0.as_seconds_f64()
    }

    /// Convert this Time to another time scale.
    ///
    /// Args:
    ///     scale: Target time scale.
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
    /// TODO: how does this fail when there's no provider? Document and test
    #[wasm_bindgen(js_name = "toScale")]
    pub fn to_scale(
        &self,
        scale: JsValue
    ) -> Result<JsTime, JsValue> {
        let scale: JsTimeScale = scale.try_into()?;
        let time = self.0.to_scale(scale.inner());
        Ok(JsTime(time))
    }

    // same as above, mandatory provider.
    // Can convert anything
    #[wasm_bindgen(js_name = "toScaleWithProvider")]
    pub fn to_scale_with_provider(
        &self,
        scale: JsValue,
        provider: &JsEopProvider,
    ) -> Result<JsTime, JsValue> {
        let scale: JsTimeScale = scale.try_into()?;
        let time = self
            .0
            .try_to_scale(scale.inner(), &provider.inner())
            .map_err(JsEopProviderError)?;
        Ok(JsTime(time))
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
    #[wasm_bindgen(js_name = "toUtc")]
    pub fn to_utc(&self) -> Result<JsUtc, JsValue> {
        let utc = self.0.to_scale(Tai)
            .try_to_utc()
            .map_err(JsUtcError)?;
        Ok(JsUtc::from_inner(utc))
    }

    // same as above, mandatory provider.
    #[wasm_bindgen(js_name = "toUtcWithProvider")]
    pub fn to_utc_with_provider(&self, provider: &JsEopProvider) -> Result<JsUtc, JsValue> {
        let tai = self
            .0
            .try_to_scale(Tai, &provider.inner())
            .map_err(JsEopProviderError)?;
        let utc = tai.try_to_utc().map_err(JsUtcError)?;
        Ok(JsUtc::from_inner(utc))
    }

}

impl JsTime {
    pub fn inner(&self) -> DynTime {
        self.0.clone()
    }

    pub fn from_inner(time: DynTime) -> Self {
        Self(time)
    }
}

impl ToDelta for JsTime {
    fn to_delta(&self) -> TimeDelta {
        self.0.to_delta()
    }
}

impl JulianDate for JsTime {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        self.0.julian_date(epoch, unit)
    }
}

impl Add<TimeDelta> for JsTime {
    type Output = JsTime;

    fn add(self, rhs: TimeDelta) -> Self::Output {
        JsTime(self.0 + rhs)
    }
}

impl Sub<TimeDelta> for JsTime {
    type Output = JsTime;

    fn sub(self, rhs: TimeDelta) -> Self::Output {
        JsTime(self.0 - rhs)
    }
}

impl Sub<JsTime> for JsTime {
    type Output = TimeDelta;

    fn sub(self, rhs: JsTime) -> TimeDelta {
        self.0 - rhs.0
    }
}

impl CalendarDate for JsTime {
    fn date(&self) -> Date {
        self.0.date()
    }
}

impl CivilTime for JsTime {
    fn time(&self) -> TimeOfDay {
        self.0.time()
    }
}
