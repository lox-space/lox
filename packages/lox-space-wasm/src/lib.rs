// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2026 Lox Space Contributors
//
// SPDX-License-Identifier: MPL-2.0

use std::str::FromStr;

use lox_space::bodies::DynOrigin;
use lox_space::bodies::Origin as OriginTrait;
use lox_space::bodies::TryMeanRadius;
use lox_space::bodies::TryPointMass;
use lox_space::frames::dynamic::DynFrame;
use lox_space::frames::traits::ReferenceFrame;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

/// Initialize the WASM module with panic hook for better error messages.
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Represents a celestial body (planet, moon, barycenter, etc.).
#[wasm_bindgen]
pub struct Origin(DynOrigin);

#[wasm_bindgen]
impl Origin {
    /// Construct an Origin from a body name (string) or NAIF ID (number).
    #[wasm_bindgen(constructor)]
    pub fn new(value: JsValue) -> Result<Origin, JsValue> {
        if let Some(name) = value.as_string() {
            let origin =
                DynOrigin::from_str(&name).map_err(|e| JsValue::from_str(&e.to_string()))?;
            return Ok(Origin(origin));
        }
        if let Some(id) = value.as_f64() {
            let origin =
                DynOrigin::try_from(id as i32).map_err(|e| JsValue::from_str(&e.to_string()))?;
            return Ok(Origin(origin));
        }
        Err(JsValue::from_str(
            "`origin` must be either a string (name) or a number (NAIF ID)",
        ))
    }

    /// Return the name of this body.
    pub fn name(&self) -> String {
        OriginTrait::name(&self.0).to_string()
    }

    /// Return the NAIF ID of this body.
    pub fn id(&self) -> i32 {
        OriginTrait::id(&self.0).0
    }

    /// Return the gravitational parameter (GM) in m³/s².
    pub fn gravitational_parameter(&self) -> Result<f64, JsValue> {
        TryPointMass::try_gravitational_parameter(&self.0)
            .map(|gp| gp.as_f64())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Return the mean radius in meters.
    pub fn mean_radius(&self) -> Result<f64, JsValue> {
        TryMeanRadius::try_mean_radius(&self.0)
            .map(|r| r.to_meters())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Represents a reference frame for positioning and transformations.
#[wasm_bindgen]
pub struct Frame(DynFrame);

#[wasm_bindgen]
impl Frame {
    /// Construct a Frame from its abbreviation (e.g., "ICRF", "ITRF").
    #[wasm_bindgen(constructor)]
    pub fn new(abbreviation: &str) -> Result<Frame, JsValue> {
        DynFrame::from_str(abbreviation)
            .map(Frame)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Return the full name of this reference frame.
    pub fn name(&self) -> String {
        ReferenceFrame::name(&self.0)
    }

    /// Return the abbreviation of this reference frame.
    pub fn abbreviation(&self) -> String {
        self.0.abbreviation()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Origin tests

    #[test]
    fn test_origin_from_name() {
        let origin = DynOrigin::from_str("Earth").unwrap();
        assert_eq!(OriginTrait::name(&origin), "Earth");
    }

    #[test]
    fn test_origin_from_naif_id() {
        let origin = DynOrigin::try_from(399i32).unwrap();
        assert_eq!(OriginTrait::name(&origin), "Earth");
    }

    #[test]
    fn test_origin_gravitational_parameter() {
        let origin = DynOrigin::from_str("Earth").unwrap();
        let gp = TryPointMass::try_gravitational_parameter(&origin)
            .unwrap()
            .as_f64();
        assert!((gp - 3.986e14).abs() < 1e11, "gp = {gp}");
    }

    #[test]
    fn test_origin_mean_radius() {
        let origin = DynOrigin::from_str("Earth").unwrap();
        let r = TryMeanRadius::try_mean_radius(&origin).unwrap().to_meters();
        assert!((r - 6.371e6).abs() < 1e4, "r = {r}");
    }

    #[test]
    fn test_origin_unknown_name() {
        assert!(DynOrigin::from_str("Tatooine").is_err());
    }

    #[test]
    fn test_origin_unknown_id() {
        assert!(DynOrigin::try_from(999999i32).is_err());
    }

    // Frame tests

    #[test]
    fn test_frame_icrf() {
        let frame = DynFrame::from_str("ICRF").unwrap();
        assert_eq!(frame.abbreviation(), "ICRF");
    }

    #[test]
    fn test_frame_itrf() {
        let frame = DynFrame::from_str("ITRF").unwrap();
        assert_eq!(frame.abbreviation(), "ITRF");
    }

    #[test]
    fn test_frame_unknown() {
        assert!(DynFrame::from_str("UNKNOWN_FRAME_XYZ").is_err());
    }
}
