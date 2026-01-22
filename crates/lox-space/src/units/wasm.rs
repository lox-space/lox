// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

use wasm_bindgen::prelude::*;

use lox_units::{Angle, Distance, Frequency, Velocity};

macro_rules! wasm_unit {
    ($(
        ($unit:ident, $rust_name:ident, $js_name:literal,
            [$(($from_fn:ident, $to_fn:ident)),* $(,)?]
        )
    ),* $(,)?) => {
        $(
            #[wasm_bindgen(js_name = $js_name)]
            pub struct $rust_name($unit);

            #[wasm_bindgen(js_class = $js_name)]
            impl $rust_name {
                #[wasm_bindgen(constructor)]
                pub fn new(value: f64) -> Self {
                    Self($unit::new(value))
                }

                pub fn mul(&self, scalar: f64) -> Self {
                    Self(scalar * self.0)
                }

                pub fn raw_value(&self) -> f64 {
                    f64::from(self.0)
                }

                pub fn as_int(&self) -> i64 {
                    let val: f64 = self.0.into();
                    val.round_ties_even() as i64
                }

                pub fn to_string_js(&self) -> String {
                    self.0.to_string()
                }

                pub fn repr(&self) -> String {
                    format!("{}({})", stringify!($unit), f64::from(self.0))
                }

                $(
                    pub fn $from_fn(value: f64) -> Self {
                        Self($unit::$from_fn(value))
                    }

                    pub fn $to_fn(&self) -> f64 {
                        self.0.$to_fn()
                    }
                )*
            }
        )*
    };
}

wasm_unit!(
    (
        Angle,
        JsAngle,
        "Angle",
        [
            (degrees, to_degrees),
            (radians, to_radians)
        ]
    ),
    (
        Distance,
        JsDistance,
        "Distance",
        [
            (kilometers, to_kilometers),
            (meters, to_meters),
            (astronomical_units, to_astronomical_units)
        ]
    ),
    (
        Frequency,
        JsFrequency,
        "Frequency",
        [
            (hertz, to_hertz),
            (kilohertz, to_kilohertz),
            (megahertz, to_megahertz),
            (gigahertz, to_gigahertz),
            (terahertz, to_terahertz)
        ]
    ),
    (
        Velocity,
        JsVelocity,
        "Velocity",
        [
            (meters_per_second, to_meters_per_second),
            (kilometers_per_second, to_kilometers_per_second),
            (astronomical_units_per_day, to_astronomical_units_per_day),
            (fraction_of_speed_of_light, to_fraction_of_speed_of_light)
        ]
    )
);
