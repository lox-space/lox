// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

use crate::wasm::js_error_with_name;
use wasm_bindgen::prelude::*;

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
#[wasm_bindgen(js_name = "TimeDelta")]
pub struct JsTimeDelta(pub TimeDelta);

#[wasm_bindgen(js_class = "TimeDelta")]
impl JsTimeDelta {
    #[wasm_bindgen(constructor)]
    pub fn new(seconds: f64) -> Self {
        Self(TimeDelta::from_seconds_f64(seconds))
    }

    pub fn __repr__(&self) -> String {
        format!("TimeDelta({})", self.to_decimal_seconds())
    }

    pub fn __str__(&self) -> String {
        format!("{} seconds", self.to_decimal_seconds())
    }

    pub fn __float__(&self) -> f64 {
        self.to_decimal_seconds()
    }

    pub fn __neg__(&self) -> Self {
        Self(-self.0)
    }

    pub fn __add__(&self, other: JsTimeDelta) -> Self {
        Self(self.0 + other.0)
    }

    pub fn __sub__(&self, other: JsTimeDelta) -> Self {
        Self(self.0 - other.0)
    }

    pub fn __eq__(&self, other: JsTimeDelta) -> bool {
        self.0 == other.0
    }

    /// Return the integer seconds component.
    ///
    /// Returns:
    ///     Integer seconds (sign matches the delta).
    ///
    /// Raises:
    ///     NonFiniteTimeDeltaError: If the delta is non-finite.
    pub fn seconds(&self) -> Result<i64, JsValue> {
        self.0.seconds().ok_or(NonFiniteTimeDeltaError::new_err(
            "cannot access seconds for non-finite time delta",
        ))
    }

    /// Return the subsecond (fractional second) component.
    ///
    /// Returns:
    ///     Fractional seconds (0.0 to 1.0).
    ///
    /// Raises:
    ///     NonFiniteTimeDeltaError: If the delta is non-finite.
    pub fn subsecond(&self) -> Result<f64, JsValue> {
        self.0.subsecond().ok_or(NonFiniteTimeDeltaError::new_err(
            "cannot access subsecond for non-finite time delta",
        ))
    }

    /// Create a TimeDelta from integer seconds.
    #[classmethod]
    pub fn from_seconds(_cls: &Bound<'_, PyType>, seconds: i64) -> Self {
        Self(TimeDelta::from_seconds(seconds))
    }

    /// Create a TimeDelta from minutes.
    #[classmethod]
    pub fn from_minutes(_cls: &Bound<'_, PyType>, minutes: f64) -> Result<Self, JsValue> {
        Ok(Self(TimeDelta::from_minutes(minutes)))
    }

    /// Create a TimeDelta from hours.
    #[classmethod]
    pub fn from_hours(_cls: &Bound<'_, PyType>, hours: f64) -> Result<Self, JsValue> {
        Ok(Self(TimeDelta::from_hours(hours)))
    }

    /// Create a TimeDelta from days (86400 seconds per day).
    #[classmethod]
    pub fn from_days(_cls: &Bound<'_, PyType>, days: f64) -> Result<Self, JsValue> {
        Ok(Self(TimeDelta::from_days(days)))
    }

    /// Create a TimeDelta from Julian years (365.25 days per year).
    #[classmethod]
    pub fn from_julian_years(_cls: &Bound<'_, PyType>, years: f64) -> Result<Self, JsValue> {
        Ok(Self(TimeDelta::from_julian_years(years)))
    }

    /// Create a TimeDelta from Julian centuries (36525 days per century).
    #[classmethod]
    pub fn from_julian_centuries(_cls: &Bound<'_, PyType>, centuries: f64) -> Result<Self, JsValue> {
        Ok(Self(TimeDelta::from_julian_centuries(centuries)))
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
    #[classmethod]
    #[pyo3(signature = (start, end, step=None))]
    pub fn range(
        _cls: &Bound<'_, PyType>,
        start: i64,
        end: i64,
        step: Option<i64>,
    ) -> Result<Vec, JsValue<Self>> {
        let step = TimeDelta::from_seconds(step.unwrap_or(1));
        let range = TimeDelta::range(start..=end).with_step(step);
        Ok(range.into_iter().map(Self).collect())
    }

    /// Convert to decimal seconds.
    ///
    /// Returns:
    ///     The duration as a float in seconds.
    pub fn to_decimal_seconds(&self) -> f64 {
        self.0.to_seconds().to_f64()
    }
}
