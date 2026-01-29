// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

use crate::wasm::js_error_with_name;
use js_sys::Reflect;
use wasm_bindgen::prelude::*;

use crate::time::time_scales::{DynTimeScale, TimeScale, UnknownTimeScaleError};

pub struct JsUnknownTimeScaleError(pub UnknownTimeScaleError);

impl From<JsUnknownTimeScaleError> for JsValue {
    fn from(err: JsUnknownTimeScaleError) -> Self {
        js_error_with_name(err.0.to_string(), "ValueError")
    }
}

/// Represents an astronomical time scale.
///
/// Supported time scales:
///
/// - **TAI**: International Atomic Time - the basis for civil time
/// - **TT**: Terrestrial Time - used for geocentric ephemerides
/// - **TDB**: Barycentric Dynamical Time - used for solar system ephemerides
/// - **TCB**: Barycentric Coordinate Time - relativistic coordinate time
/// - **TCG**: Geocentric Coordinate Time - relativistic coordinate time
/// - **UT1**: Universal Time - tied to Earth's rotation
///
/// Args:
///     abbreviation: Time scale abbreviation ("TAI", "TT", "TDB", "TCB", "TCG", "UT1").
///
/// Raises:
///     ValueError: If the abbreviation is not recognized.
#[wasm_bindgen(js_name = "TimeScale")]
pub struct JsTimeScale(DynTimeScale);

#[wasm_bindgen(js_class = "TimeScale")]
impl JsTimeScale {
    #[wasm_bindgen(constructor)]
    pub fn new(abbreviation: &str) -> Result<Self, JsValue> {
        Ok(JsTimeScale(
            abbreviation.parse().map_err(JsUnknownTimeScaleError)?,
        ))
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string(&self) -> String {
        format!("{}", self.0)
    }

    pub fn debug(&self) -> String {
        format!("TimeScale(\"{}\")", self.0)
    }

    pub fn default() -> Self {
        Self(DynTimeScale::default())
    }

    /// Return the time scale abbreviation (e.g., "TAI").
    #[wasm_bindgen(getter)]
    pub fn abbreviation(&self) -> String {
        self.0.abbreviation().to_owned()
    }

    /// Return the full name of the time scale (e.g., "International Atomic Time").
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.0.name().to_owned()
    }
}

impl JsTimeScale {
    pub fn inner(&self) -> DynTimeScale {
        self.0.clone()
    }

    pub fn from_inner(scale: DynTimeScale) -> Self {
        Self(scale)
    }
}

impl TryFrom<JsValue> for JsTimeScale {
    type Error = JsValue;

    fn try_from(value: JsValue) -> Result<Self, JsValue> {
        if let Some(name) = value.as_string() {
            return Ok(JsTimeScale(name.parse().map_err(JsUnknownTimeScaleError)?));
        }
        if value.is_object() {
            // XXX: ugly, maybe make a serde thing instead?
            // Checks if the object has an "abbreviation" property
            if let Ok(prop) = Reflect::get(&value, &JsValue::from_str("abbreviation")) {
                if let Some(name) = prop.as_string() {
                    return Ok(JsTimeScale(name.parse().map_err(JsUnknownTimeScaleError)?));
                }
            }
        }
        Err(js_error_with_name(
            "'scale' argument must be a string or a TimeScale instance",
            "TypeError",
        ))
    }
}
