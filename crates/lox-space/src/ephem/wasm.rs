// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

use serde_wasm_bindgen;
use std::path::PathBuf;
use wasm_bindgen::prelude::*;
use crate::wasm::js_error_with_name;
use crate::ephem::spk::parser::{DafSpkError, Spk, parse_daf_spk};

pub struct JsDafSpkError(pub DafSpkError);

impl From<JsDafSpkError> for JsValue {
    fn from(err: JsDafSpkError) -> Self {
        js_error_with_name(err.0, "DafSpkError")
    }
}

/// SPICE SPK (Spacecraft and Planet Kernel) ephemeris data.
///
/// SPK files contain position and velocity data for celestial bodies and
/// spacecraft. They are used to compute accurate positions for orbit
/// propagation, frame transformations, and visibility analysis.
///
/// SPK files can be obtained from:
///
/// - NASA NAIF: https://naif.jpl.nasa.gov/naif/data.html
/// - ESA SPICE Service: https://spice.esac.esa.int/
///
/// Args:
///     path: Path to the SPK file (.bsp).
///
/// Raises:
///     ValueError: If the file cannot be parsed or is invalid.
///     OSError: If the file cannot be read.
#[wasm_bindgen(js_name = "SPK")]
pub struct JsSpk(Spk);

#[wasm_bindgen(js_class = "SPK")]
impl JsSpk {
    #[wasm_bindgen(constructor)]
    pub fn new(path: JsValue) -> Result<Self, JsValue> {
        let path: PathBuf = serde_wasm_bindgen::from_value(path)
            .map_err(|_| JsValue::from_str("Invalid path"))?;
        let data = std::fs::read(&path).map_err(|e| js_error_with_name(e, "OSError"))?;
        let spk = parse_daf_spk(&data).map_err(JsDafSpkError)?;
        Ok(JsSpk(spk))
    }
}
