use js_sys::Error;
use wasm_bindgen::prelude::*;

pub fn js_error_with_name<E: std::fmt::Display>(e: E, name: &str) -> JsValue {
    let err = Error::new(&e.to_string());
    err.set_name(name);
    err.into()
}


/// XXX: Temporary function to allow passing owned String as name
/// Not sure if js_error_with_name should even exist, maybe use proper error structs everywhere.
pub fn js_error_with_name_from_string<E: std::fmt::Display>(e: E, name: String) -> JsValue {
    let err = Error::new(&e.to_string());
    err.set_name(&name);
    err.into()
}
