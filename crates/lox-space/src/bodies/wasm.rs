// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

use crate::bodies::dynamic::{DynOrigin, UnknownOriginId, UnknownOriginName};
use crate::bodies::{
    Origin as RustOrigin, TryMeanRadius, TryPointMass, TryRotationalElements, TrySpheroid,
    TryTriaxialEllipsoid,
};
use lox_core::types::units::Seconds;
use wasm_bindgen::prelude::*;
use crate::wasm::js_error_with_name;
use std::str::FromStr;


pub struct JsUndefinedOriginPropertyError(crate::bodies::UndefinedOriginPropertyError);

impl From<JsUndefinedOriginPropertyError> for JsValue {
    fn from(err: JsUndefinedOriginPropertyError) -> Self {
        js_error_with_name(err.0, "UndefinedOriginPropertyError")
    }
}

pub struct JsUnknownOriginId(UnknownOriginId);

impl From<JsUnknownOriginId> for JsValue {
    fn from(err: JsUnknownOriginId) -> Self {
        js_error_with_name(err.0, "UnknownOriginId")
    }
}

pub struct JsUnknownOriginName(UnknownOriginName);


impl From<JsUnknownOriginName> for JsValue {
    fn from(err: JsUnknownOriginName) -> Self {
        js_error_with_name(err.0, "UnknownOriginName")
    }
}

// not too sure about the naming of these structs, they're from type aliases
#[wasm_bindgen(js_name = "Radii")]
pub struct JsRadii {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[wasm_bindgen(js_name = "Elements")]
pub struct JsElements {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Represents a celestial body (planet, moon, barycenter, etc.).
///
/// Origin objects represent celestial bodies using NAIF/SPICE identifiers.
/// They provide access to physical properties such as gravitational parameters,
/// radii, and rotational elements.
///
/// Args:
///     origin: Body name (e.g., "Earth", "Moon") or NAIF ID (e.g., 399 for Earth).
///
/// Raises:
///     ValueError: If the origin name or ID is not recognized.
///     TypeError: If the argument is neither a string nor an integer.
#[wasm_bindgen(js_name = "Origin")]
pub struct JsOrigin(DynOrigin);

#[wasm_bindgen(js_class = "Origin")]
impl JsOrigin {
   #[wasm_bindgen(constructor)]
    pub fn new(origin: JsValue) -> Result<JsOrigin, JsValue> {
        if let Some(id_f64) = origin.as_f64() {
            let id = id_f64 as i32;
            return Ok(JsOrigin(
                id.try_into()
                    .map_err(JsUnknownOriginId)?,
            ));
        }

        if let Some(name) = origin.as_string() {
            return Ok(
                JsOrigin(DynOrigin::from_str(&name)
                .map_err(JsUnknownOriginName)?),
            );
        }

        Err(js_error_with_name(
            "`origin` must be either a string or an integer",
            "TypeError",
        ))
    }

    fn __repr__(&self) -> String {
        format!("Origin(\"{}\")", self.0.name())
    }

    fn __str__(&self) -> String {
        self.0.name().to_string()
    }

    fn __getnewargs__(&self) -> (String,) {
        (self.0.name().to_string(),)
    }

    /// Return the NAIF ID of this body.
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> i32 {
        self.0.id().0
    }

    /// Return the name of this body.
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.0.name().to_string()
    }

    /// Return the gravitational parameter (GM) in km³/s².
    ///
    /// Raises:
    ///     UndefinedOriginPropertyError: If not defined for this body.
    pub fn gravitational_parameter(&self) -> Result<f64, JsValue> {
        Ok(self
            .0
            .try_gravitational_parameter()
            .map_err(JsUndefinedOriginPropertyError)?)
    }

    /// Return the mean radius in km.
    ///
    /// Raises:
    ///     UndefinedOriginPropertyError: If not defined for this body.
    pub fn mean_radius(&self) -> Result<f64, JsValue> {
        Ok(self
            .0
            .try_mean_radius()
            .map_err(JsUndefinedOriginPropertyError)?)
    }

    /// Return the triaxial radii (x, y, z) in km.
    ///
    /// Raises:
    ///     UndefinedOriginPropertyError: If not defined for this body.
    pub fn radii(&self) -> Result<JsRadii, JsValue> {
        Ok(
            self
            .0
            .try_radii()
            .map(|r| JsRadii { x: r.0, y: r.1, z: r.2 })
            .map_err(JsUndefinedOriginPropertyError)?)
    }

    /// Return the equatorial radius in km.
    ///
    /// Raises:
    ///     UndefinedOriginPropertyError: If not defined for this body.
    pub fn equatorial_radius(&self) -> Result<f64, JsValue> {
        Ok(self
            .0
            .try_equatorial_radius()
            .map_err(JsUndefinedOriginPropertyError)?)
    }

    /// Return the polar radius in km.
    ///
    /// Raises:
    ///     UndefinedOriginPropertyError: If not defined for this body.
    pub fn polar_radius(&self) -> Result<f64, JsValue> {
        Ok(self
            .0
            .try_polar_radius()
            .map_err(JsUndefinedOriginPropertyError)?)
    }

    /// Return rotational elements (right ascension, declination, rotation angle) in radians.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    ///
    /// Returns:
    ///     Tuple of (right_ascension, declination, rotation_angle) in radians.
    ///
    /// Raises:
    ///     UndefinedOriginPropertyError: If not defined for this body.
    pub fn rotational_elements(&self, et: Seconds) -> Result<JsElements, JsValue> {
        Ok(self
            .0
            .try_rotational_elements(et)
            .map(|e| JsElements { x: e.0, y: e.1, z: e.2 })
            .map_err(JsUndefinedOriginPropertyError)?)
    }

    /// Return rotational element rates in radians/second.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    ///
    /// Returns:
    ///     Tuple of (ra_rate, dec_rate, rotation_rate) in radians/second.
    ///
    /// Raises:
    ///     UndefinedOriginPropertyError: If not defined for this body.
    pub fn rotational_element_rates(&self, et: Seconds) -> Result<JsElements, JsValue> {
        Ok(self
            .0
            .try_rotational_element_rates(et)
            .map(|e| JsElements { x: e.0, y: e.1, z: e.2 })
            .map_err(JsUndefinedOriginPropertyError)?)
    }

    /// Return the right ascension of the pole in radians.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    pub fn right_ascension(&self, et: Seconds) -> Result<f64, JsValue> {
        Ok(self
            .0
            .try_right_ascension(et)
            .map_err(JsUndefinedOriginPropertyError)?)
    }

    /// Return the rate of change of right ascension in radians/second.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    pub fn right_ascension_rate(&self, et: Seconds) -> Result<f64, JsValue> {
        Ok(self
            .0
            .try_right_ascension_rate(et)
            .map_err(JsUndefinedOriginPropertyError)?)
    }

    /// Return the declination of the pole in radians.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    pub fn declination(&self, et: Seconds) -> Result<f64, JsValue> {
        Ok(self
            .0
            .try_declination(et)
            .map_err(JsUndefinedOriginPropertyError)?)
    }

    /// Return the rate of change of declination in radians/second.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    pub fn declination_rate(&self, et: Seconds) -> Result<f64, JsValue> {
        Ok(self
            .0
            .try_declination_rate(et)
            .map_err(JsUndefinedOriginPropertyError)?)
    }

    /// Return the rotation angle (prime meridian) in radians.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    pub fn rotation_angle(&self, et: Seconds) -> Result<f64, JsValue> {
        Ok(self
            .0
            .try_rotation_angle(et)
            .map_err(JsUndefinedOriginPropertyError)?)
    }

    /// Return the rotation rate in radians/second.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    pub fn rotation_rate(&self, et: Seconds) -> Result<f64, JsValue> {
        Ok(self
            .0
            .try_rotation_rate(et)
            .map_err(JsUndefinedOriginPropertyError)?)
    }
}
