// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

use crate::wasm::js_error_with_name;
use wasm_bindgen::prelude::*;
use js_sys::Array;

use crate::time::deltas::TimeDelta;

/// Represents a duration or time difference.
///
/// `TimeDelta` represents a time interval with femtosecond precision.
/// It can be added to or subtracted from `Time` objects, and arithmetic
/// operations between `TimeDelta` objects are supported.
///
/// Args:
///     seconds: Duration in seconds (can be negative).
///
/// See Also:
///     Time: For representing instants in time.
#[wasm_bindgen(js_name = "TimeDelta")]
pub struct JsTimeDelta(TimeDelta);

#[wasm_bindgen(js_class = "TimeDelta")]
impl JsTimeDelta {
    #[wasm_bindgen(constructor)]
    pub fn new(seconds: f64) -> Self {
        Self(TimeDelta::from_seconds_f64(seconds))
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string(&self) -> String {
        format!("{} seconds", self.to_decimal_seconds())
    }

    pub fn debug(&self) -> String {
        format!("TimeDelta({})", self.to_decimal_seconds())
    }

    pub fn neg(&self) -> Self {
        Self(-self.0)
    }

    pub fn add(&self, other: &JsTimeDelta) -> Self {
        Self(self.0 + other.0)
    }

    pub fn subtract(&self, other: &JsTimeDelta) -> Self {
        Self(self.0 - other.0)
    }

    pub fn equals(&self, other: &JsTimeDelta) -> bool {
        self.0 == other.0
    }

    /// Return the integer seconds component.
    ///
    /// Returns:
    ///     Integer seconds (sign matches the delta).
    ///
    /// Raises:
    ///     NonFiniteTimeDeltaError: If the delta is non-finite.
    pub fn seconds(&self) -> Result<i32, JsValue> {
        self
        .0
        .seconds()
        .ok_or_else(|| {
            js_error_with_name(
                "NonFiniteTimeDeltaError",
                "cannot access seconds for non-finite time delta",
            )
        })
        .and_then(|seconds| {
            if seconds > i32::MAX as i64 || seconds < i32::MIN as i64 {
                Err(js_error_with_name(
                    "OverflowError",
                    "seconds component out of range for i32",
                ))
            } else {
                Ok(seconds as i32)
            }
        })
    }

    /// Return the subsecond (fractional second) component.
    ///
    /// Returns:
    ///     Fractional seconds (0.0 to 1.0).
    ///
    /// Raises:
    ///     NonFiniteTimeDeltaError: If the delta is non-finite.
    pub fn subsecond(&self) -> Result<f64, JsValue> {
        self.0.subsecond().ok_or_else(|| {
            js_error_with_name(
                "NonFiniteTimeDeltaError",
                "cannot access subsecond for non-finite time delta",
            )
        })
    }

    /// Create a TimeDelta from integer seconds.
    #[wasm_bindgen(js_name = "fromSeconds")]
    pub fn from_seconds(seconds: i32) -> Self {
        Self(TimeDelta::from_seconds(seconds as i64))
    }

    /// Create a TimeDelta from minutes.
    #[wasm_bindgen(js_name = "fromMinutes")]
    pub fn from_minutes(minutes: f64) -> Self {
        Self(TimeDelta::from_minutes(minutes))
    }

    /// Create a TimeDelta from hours.
    #[wasm_bindgen(js_name = "fromHours")]
    pub fn from_hours(hours: f64) -> Self {
        Self(TimeDelta::from_hours(hours))
    }

    /// Create a TimeDelta from days (86400 seconds per day).
    #[wasm_bindgen(js_name = "fromDays")]
    pub fn from_days(days: f64) -> Self {
        Self(TimeDelta::from_days(days))
    }

    /// Create a TimeDelta from Julian years (365.25 days per year).
    #[wasm_bindgen(js_name = "fromJulianYears")]
    pub fn from_julian_years(years: f64) -> Self {
        Self(TimeDelta::from_julian_years(years))
    }

    /// Create a TimeDelta from Julian centuries (36525 days per century).
    #[wasm_bindgen(js_name = "fromJulianCenturies")]
    pub fn from_julian_centuries(centuries: f64) -> Self {
        Self(TimeDelta::from_julian_centuries(centuries))
    }

    /// Create a range of TimeDelta values.
    ///
    /// Args:
    ///     start: Start value in seconds (inclusive).
    ///     end: End value in seconds (inclusive).
    ///     step: Step size in seconds. Defaults to 1.
    ///
    /// Returns:
    ///     A list of TimeDelta objects.
    ///
    /// Examples:
    ///     >>> deltas = lox.TimeDelta.range(0, 10, 2)  # [0, 2, 4, 6, 8, 10]
    pub fn range(start: i32, end: i32, step: Option<i32>) -> Array {
        let step = TimeDelta::from_seconds(step.unwrap_or(1) as i64);
        let range = TimeDelta::range(start as i64..=end as i64).with_step(step);
        let arr = Array::new();
        for delta in range {
            arr.push(&JsValue::from(JsTimeDelta(delta)));
        }
        arr
    }

    /// Convert to decimal seconds.
    ///
    /// Returns:
    ///     The duration as a float in seconds.
    #[wasm_bindgen(js_name = "toDecimalSeconds")]
    pub fn to_decimal_seconds(&self) -> f64 {
        self.0.to_seconds().to_f64()
    }
}

impl JsTimeDelta {
    pub fn inner(&self) -> TimeDelta {
        self.0.clone()
    }

    pub fn from_inner(delta: TimeDelta) -> Self {
        Self(delta)
    }
}
