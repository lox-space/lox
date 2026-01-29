// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

use serde_wasm_bindgen;
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
///     DafSpkError: If the file cannot be parsed or is invalid.
#[wasm_bindgen(js_name = "SPK")]
pub struct JsSpk(Spk);

#[wasm_bindgen(js_class = "SPK")]
impl JsSpk {
    /// Read SPK data from bytes
    ///
    /// example usage:
    ///   const buf = await readFile(<filepath>));
    ///   return lox.SPK.fromBytes(new Uint8Array(buf));
    #[wasm_bindgen(js_name = "fromBytes")]
    pub fn from_bytes(bytes: &[u8]) -> Result<JsSpk, JsValue> {
        let spk = parse_daf_spk(bytes).map_err(JsDafSpkError)?;
        Ok(JsSpk(spk))
    }
}

impl JsSpk {
    pub fn inner(&self) -> &Spk {
        &self.0
    }

    pub fn from_inner(provider: Spk) -> Self {
        Self(provider)
    }
}
