// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0
// TODO: Not sure if this is the best way to handle multiple argument types in wasm
//       Not sure if this code will ever be called, who's handling files from javascript?
//       Are there backend node.js mission analysts?
//       Here for completeness.

use serde::Deserialize;
use std::path::PathBuf;
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

#[derive(Deserialize)]
#[serde(untagged)]
enum EopArgs {
    Object { path: PathBuf, #[serde(default = "PathBuf::new")] path2: PathBuf },
    SingleObject { path: PathBuf },
    Tuple((PathBuf, PathBuf)),
    SingleTuple((PathBuf,)),
}

fn parse_paths(args: JsValue) -> Result<(PathBuf, PathBuf), JsValue> {
    let parsed: EopArgs = serde_wasm_bindgen::from_value(args)
        .map_err(|_| JsEopParserError(eop::EopParserError::NoFiles))?;
    let (path1, path2) = match parsed {
        EopArgs::Object { path, path2 } if !path2.as_os_str().is_empty() => (path, path2),
        EopArgs::Object { path, .. } => (path.clone(), path),
        EopArgs::SingleObject { path } => (path.clone(), path),
        EopArgs::Tuple((p1, p2)) => (p1, p2),
        EopArgs::SingleTuple((p,)) => (p.clone(), p),
    };
    Ok((path1, path2))
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
/// Args:
///     path: Path to the EOP data file (CSV format).
///     path2: Optional second path for separate polar motion and UT1 files.
///
/// Raises:
///     EopParserError: If the file cannot be parsed.
///     OSError: If the file cannot be read.
#[wasm_bindgen(js_name="EOPProvider")]
#[derive(Clone, Debug)]
pub struct JsEopProvider(EopProvider);

#[wasm_bindgen(js_class="EOPProvider")]
impl JsEopProvider {
    #[wasm_bindgen(constructor)]
    pub fn new(args: JsValue) -> Result<JsEopProvider, JsValue> {
        let (path1, path2) = parse_paths(args)?;
        Ok(JsEopProvider(
            EopParser::new()
                .from_paths(path1, path2)
                .parse()
                .map_err(JsEopParserError)?,
        ))
    }

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
