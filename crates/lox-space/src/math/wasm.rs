// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

use crate::math::series::{Series, SeriesError};
use lox_math::series::InterpolationType;
use crate::wasm::js_error_with_name;
use wasm_bindgen::prelude::*;

pub struct JsSeriesError(pub SeriesError);

impl From<JsSeriesError> for JsValue {
    fn from(err: JsSeriesError) -> Self {
        js_error_with_name(err.0, "SeriesError")
    }
}

/// Interpolation series for 1D data.
///
/// Series provides interpolation between data points using either linear
/// or cubic spline methods.
///
/// Args:
///     x: Array of x values (must be monotonically increasing).
///     y: Array of y values (same length as x).
///     method: Interpolation method ("linear" or "cubic_spline").
///
/// Raises:
///     ValueError: If x and y have different lengths or x is not monotonic.
#[wasm_bindgen(js_name = "Series")]
#[derive(Clone, Debug)]
pub struct JsSeries(Series);

#[wasm_bindgen(js_class = "Series")]
impl JsSeries {
    #[wasm_bindgen(constructor)]
    pub fn new(x: Vec<f64>, y: Vec<f64>, interpolation: &str) -> Result<Self, JsValue> {
        let interpolation = match interpolation {
            "linear" => InterpolationType::Linear,
            "cubic_spline" => InterpolationType::CubicSpline,
            _ => return Err(js_error_with_name("unknown interpolation type", "ValueError")),
        };
        let series = Series::try_new(x, y, interpolation).map_err(JsSeriesError)?;
        Ok(JsSeries(series))
    }

    /// Interpolate a y value at the given x coordinate.
    ///
    /// Args:
    ///     xp: The x value to interpolate at.
    ///
    /// Returns:
    ///     The interpolated y value.
    fn interpolate(&self, xp: f64) -> f64 {
        self.0.interpolate(xp)
    }
}
