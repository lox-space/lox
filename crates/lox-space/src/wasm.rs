use js_sys::Error;
use wasm_bindgen::prelude::*;

pub fn js_error_with_name<E: std::fmt::Display>(e: E, name: &str) -> JsValue {
    let err = Error::new(&e.to_string());
    err.set_name(name);
    err.into()
}
