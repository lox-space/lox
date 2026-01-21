// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0


use wasm_bindgen::prelude::*;

use lox_units::{Angle, Distance, Frequency, Velocity};

macro_rules! wasm_unit {
    ($(($unit:ident, $jsunit:ident)),*) => {
        $(
            #[wasm_bindgen]
            pub struct $jsunit($unit);

            #[wasm_bindgen]
            impl $jsunit {
                #[wasm_bindgen(constructor)]
                pub fn new(value: f64) -> Self {
                    Self($unit::new(value))
                }

                /// Scale this unit by a scalar (scalar * value).
                #[wasm_bindgen]
                pub fn mul(&self, scalar: f64) -> Self {
                    Self(scalar * self.0)
                }

                /// Get the numeric value as f64.
                #[wasm_bindgen]
                pub fn value(&self) -> f64 {
                    f64::from(self.0)
                }

                /// Get the value rounded to the nearest integer (ties to even).
                #[wasm_bindgen]
                pub fn as_int(&self) -> i64 {
                    let val: f64 = self.0.into();
                    val.round_ties_even() as i64
                }

                /// String representation (Display).
                #[wasm_bindgen]
                pub fn to_string_js(&self) -> String {
                    self.0.to_string()
                }

                /// Debug-style representation.
                #[wasm_bindgen]
                pub fn repr(&self) -> String {
                    format!("{}({})", stringify!($unit), f64::from(self.0))
                }
            }
        )*
    };
}

wasm_unit!(
    (Angle, JsAngle),
    (Distance, JsDistance),
    (Frequency, JsFrequency),
    (Velocity, JsVelocity)
);
