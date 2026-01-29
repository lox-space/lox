// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0
// TODO: Not sure if this is the best way to handle multiple argument types in wasm
//       Not sure if this code will ever be called, who's handling files from javascript?
//       Are there backend node.js mission analysts?
//       Here for completeness.

use wasm_bindgen::prelude::*;
use crate::wasm::js_error_with_name;
use lox_earth::eop::{self, EopParser, EopProvider};

pub struct JsEopParserError(pub eop::EopParserError);

impl From<JsEopParserError> for JsValue {
    fn from(err: JsEopParserError) -> Self {
        js_error_with_name(err.0, "EopParserError")
    }
}

pub struct JsEopProviderError(pub lox_earth::eop::EopProviderError);

impl From<JsEopProviderError> for JsValue {
    fn from(err: JsEopProviderError) -> Self {
        js_error_with_name(err.0, "EopProviderError")
    }
}

/// Earth Orientation Parameters (EOP) data provider.
///
/// EOP data is required for accurate transformations involving UT1 and
/// polar motion corrections. The data can be loaded from IERS finals2000A
/// files (CSV format).
///
/// EOP data files can be obtained from:
///
/// - IERS: https://www.iers.org/IERS/EN/DataProducts/EarthOrientationData/eop.html
/// - Celestrak: https://celestrak.org/SpaceData/
///
///
/// Raises:
///     EopParserError: If the file contents cannot be parsed.
#[wasm_bindgen(js_name="EOPProvider")]
#[derive(Clone, Debug)]
pub struct JsEopProvider(EopProvider);

#[wasm_bindgen(js_class="EOPProvider")]
impl JsEopProvider {
    /// Read EOP data from bytes
    ///
    /// example usage:
    ///   const buf = await readFile(<filepath>));
    ///   return lox.EOPProvider.fromBytes(new Uint8Array(buf));
    #[wasm_bindgen(js_name = "fromBytes")]
    pub fn from_bytes(bytes: &[u8]) -> Result<JsEopProvider, JsValue> {
        let parser = EopParser::new();
        let provider = parser.from_bytes(bytes, None::<&[u8]>).map_err(JsEopParserError)?;
        Ok(JsEopProvider(provider))
    }
}

impl JsEopProvider {
    pub fn inner(&self) -> EopProvider {
        self.0.clone()
    }

    pub fn from_inner(provider: EopProvider) -> Self {
        Self(provider)
    }
}
