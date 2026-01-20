// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

use wasm_bindgen::prelude::*;

use lox_test_utils::{approx_eq, ApproxEq};

use crate::earth::wasm::ut1::{JsEopProvider, JsEopProviderError};
use crate::time::calendar_dates::CalendarDate;
use crate::time::time_of_day::CivilTime;
use crate::time::utc::{Utc, UtcError};
use crate::time::wasm::time::JsTime;
use crate::time::wasm::time_scales::JsTimeScale;
use crate::wasm::js_error_with_name;

pub struct JsUtcError(pub UtcError);

impl From<JsUtcError> for JsValue {
    fn from(value: JsUtcError) -> Self {
        js_error_with_name("ValueError", value.0.to_string())
    }
}

/// Represents a UTC (Coordinated Universal Time) timestamp.
///
/// UTC is the basis for civil time worldwide. Unlike `Time`, UTC handles
/// leap seconds and is discontinuous. Use `Time` for astronomical calculations
/// that require continuous time.
///
/// Args:
///     year: Calendar year.
///     month: Calendar month (1-12).
///     day: Day of month (1-31).
///     hour: Hour of day (0-23). Defaults to 0.
///     minute: Minute of hour (0-59). Defaults to 0.
///     seconds: Seconds (0.0-60.0, allows 60 for leap seconds). Defaults to 0.0.
///
/// Raises:
///     ValueError: If date or time components are out of valid range.
///
/// See Also:
///     Time: For continuous astronomical time scales.
#[wasm_bindgen(js_name = "UTC")]
#[derive(Clone, Debug, Eq, PartialEq, ApproxEq)]
pub struct JsUtc(Utc);

#[wasm_bindgen(js_class = "UTC")]
impl JsUtc {
    #[wasm_bindgen(constructor)]
    pub fn new(
        year: i64,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        seconds: f64,
    ) -> Result<JsUtc, JsValue> {
        let utc = Utc::builder()
            .with_ymd(year, month, day)
            .with_hms(hour, minute, seconds)
            .build()
            .map_err(JsUtcError)?;
        Ok(JsUtc(utc))
    }

    /// Create a UTC timestamp from an ISO 8601 formatted string.
    ///
    /// Args:
    ///     iso: ISO 8601 formatted datetime string (e.g., "2024-06-15T12:30:45Z").
    ///
    /// Returns:
    ///     A new UTC object.
    ///
    /// Raises:
    ///     ValueError: If the ISO string is invalid.
    #[wasm_bindgen]
    pub fn from_iso(iso: &str) -> Result<JsUtc, JsValue> {
        Ok(JsUtc(iso.parse().map_err(JsUtcError)?))
    }

    #[wasm_bindgen]
    pub fn to_string_js(&self) -> String {
        self.0.to_string()
    }

    #[wasm_bindgen]
    pub fn debug(&self) -> String {
        format!(
            "UTC({}, {}, {}, {}, {}, {})",
            self.0.year(),
            self.0.month(),
            self.0.day(),
            self.0.hour(),
            self.0.minute(),
            self.0.as_seconds_f64()
        )
    }

    #[wasm_bindgen]
    pub fn equals(&self, other: &JsUtc) -> bool {
        self.0 == other.0
    }

    /// Check if two UTC timestamps are approximately equal.
    ///
    /// Args:
    ///     other: The other UTC object to compare.
    ///     rel_tol: Relative tolerance. Defaults to 1e-8.
    ///     abs_tol: Absolute tolerance. Defaults to 1e-14.
    ///
    /// Returns:
    ///     True if the timestamps are approximately equal.
    #[wasm_bindgen]
    pub fn isclose(&self, other: &JsUtc, rel_tol: Option<f64>, abs_tol: Option<f64>) -> bool {
        let rel = rel_tol.unwrap_or(1e-8);
        let abs = abs_tol.unwrap_or(1e-14);
        approx_eq!(self, other, rtol <= rel, atol <= abs)
    }

    /// Return the year component.
    #[wasm_bindgen]
    pub fn year(&self) -> i64 {
        self.0.year()
    }

    /// Return the month component (1-12).
    #[wasm_bindgen]
    pub fn month(&self) -> u8 {
        self.0.month()
    }

    /// Return the day of month component (1-31).
    #[wasm_bindgen]
    pub fn day(&self) -> u8 {
        self.0.day()
    }

    /// Return the hour component (0-23).
    #[wasm_bindgen]
    pub fn hour(&self) -> u8 {
        self.0.hour()
    }

    /// Return the minute component (0-59).
    #[wasm_bindgen]
    pub fn minute(&self) -> u8 {
        self.0.minute()
    }

    /// Return the integer second component (0-60, 60 for leap second).
    #[wasm_bindgen]
    pub fn second(&self) -> u8 {
        self.0.second()
    }

    /// Return the millisecond component (0-999).
    #[wasm_bindgen]
    pub fn millisecond(&self) -> u32 {
        self.0.millisecond()
    }

    /// Return the microsecond component (0-999).
    #[wasm_bindgen]
    pub fn microsecond(&self) -> u32 {
        self.0.microsecond()
    }

    /// Return the nanosecond component (0-999).
    pub fn nanosecond(&self) -> u32 {
        self.0.nanosecond()
    }

    /// Return the picosecond component (0-999).
    #[wasm_bindgen]
    pub fn picosecond(&self) -> u32 {
        self.0.picosecond()
    }

    /// Return the decimal seconds (seconds + fractional part).
    #[wasm_bindgen]
    pub fn decimal_seconds(&self) -> f64 {
        self.0.as_seconds_f64()
    }

    /// Convert this UTC timestamp to a Time object in the specified scale.
    ///
    /// Args:
    ///     scale: Target time scale.
    ///     provider: EOP provider for UT1 conversions.
    ///
    /// Returns:
    ///     A Time object in the target scale.
    ///
    /// Examples:
    ///     >>> utc = lox.UTC(2024, 1, 1)
    ///     >>> t_tai = utc.to_scale("TAI")
    #[wasm_bindgen]
    pub fn to_scale(
        &self,
        scale: JsValue,
        provider: Option<JsEopProvider>,
    ) -> Result<JsTime, JsValue> {
        let scale: JsTimeScale = scale.try_into()?;
        let provider = provider.as_ref().map(|p| &p.0);
        let time = match provider {
            Some(provider) => self
                .0
                .to_dyn_time()
                .try_to_scale(scale.0, provider)
                .map_err(JsEopProviderError)?,
            None => self.0.to_dyn_time().to_scale(scale.0),
        };
        Ok(JsTime(time))
    }
}
