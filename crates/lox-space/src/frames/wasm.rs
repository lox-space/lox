// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

use crate::frames::{
    dynamic::{DynFrame, UnknownFrameError},
    traits::ReferenceFrame,
};
use lox_frames::rotations::DynRotationError;

use crate::wasm::js_error_with_name;
use wasm_bindgen::prelude::*;

pub struct JsUnknownFrameError(pub UnknownFrameError);

impl From<JsUnknownFrameError> for JsValue {
    fn from(err: JsUnknownFrameError) -> Self {
        js_error_with_name(err.0, "UnknownFrameError")
    }
}

pub struct JsDynRotationError(pub DynRotationError);

impl From<JsDynRotationError> for JsValue {
    fn from(err: JsDynRotationError) -> Self {
        js_error_with_name(err.0, "DynRotationError")
    }
}

/// Represents a reference frame for positioning and transformations.
///
/// Reference frames define coordinate systems for expressing positions and
/// velocities. Lox supports both inertial (non-rotating) and rotating frames.
///
/// Supported frames:
///
/// - **ICRF**: International Celestial Reference Frame (inertial)
/// - **GCRF**: Geocentric Celestial Reference Frame (inertial, Earth-centered)
/// - **CIRF**: Celestial Intermediate Reference Frame
/// - **TIRF**: Terrestrial Intermediate Reference Frame
/// - **ITRF**: International Terrestrial Reference Frame (Earth-fixed)
/// - **Body-fixed frames**: IAU_EARTH, IAU_MOON, IAU_MARS, etc.
///
/// Args:
///     abbreviation: Frame abbreviation (e.g., "ICRF", "ITRF", "IAU_MOON").
///
/// Raises:
///     ValueError: If the frame abbreviation is not recognized.
#[wasm_bindgen(js_name = "Frame")]
#[derive(Clone, Debug)]
pub struct JsFrame(DynFrame);

#[wasm_bindgen(js_class = "Frame")]
impl JsFrame {
    #[wasm_bindgen(constructor)]
    pub fn new(abbreviation: &str) -> Result<Self, JsValue> {
        Ok(Self(abbreviation.parse().map_err(JsUnknownFrameError)?))
    }

    /// Return the full name of this reference frame.
    ///
    /// Returns:
    ///     The descriptive name (e.g., "International Celestial Reference Frame").
    pub fn name(&self) -> String {
        self.0.name()
    }

    /// Return the abbreviation of this reference frame.
    ///
    /// Returns:
    ///     The short abbreviation (e.g., "ICRF", "ITRF").
    pub fn abbreviation(&self) -> String {
        self.0.abbreviation()
    }
}

impl JsFrame {
    pub fn inner(&self) -> DynFrame {
        self.0.clone()
    }

    pub fn from_inner(provider: DynFrame) -> Self {
        Self(provider)
    }
}
