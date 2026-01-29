// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

use wasm_bindgen::prelude::*;

use lox_units::{Angle, Distance, Frequency, Velocity};

macro_rules! wasm_unit {
    ($(
        ($unit:ident, $rust_name:ident, $js_name:literal,
            [$(($from_fn:ident, $from_fn_camel:literal, $to_fn:ident, $to_fn_camel:literal)),* $(,)?]
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

                #[wasm_bindgen(js_name = "rawValue")]
                pub fn raw_value(&self) -> f64 {
                    f64::from(self.0)
                }

                #[wasm_bindgen(js_name = "asInt")]
                pub fn as_int(&self) -> i64 {
                    let val: f64 = self.0.into();
                    val.round_ties_even() as i64
                }

                #[wasm_bindgen(js_name = "toString")]
                pub fn to_string(&self) -> String {
                    self.0.to_string()
                }

                pub fn debug(&self) ->  String {
                    format!("{}({})", stringify!($unit), f64::from(self.0))
                }

                $(
                    #[wasm_bindgen(js_name = $from_fn_camel)]
                    pub fn $from_fn(value: f64) -> Self {
                        Self($unit::$from_fn(value))
                    }

                    #[wasm_bindgen(js_name = $to_fn_camel)]
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
            (degrees, "degrees", to_degrees, "toDegrees"),
            (radians, "radians", to_radians, "toRadians")
        ]
    ),
    (
        Distance,
        JsDistance,
        "Distance",
        [
            (kilometers, "kilometers", to_kilometers, "toKilometers"),
            (meters, "meters", to_meters, "toMeters"),
            (astronomical_units, "astronomical_units", to_astronomical_units, "toAstronomicalUnits")
        ]
    ),
    (
        Frequency,
        JsFrequency,
        "Frequency",
        [
            (hertz, "hertz", to_hertz, "toHertz"),
            (kilohertz, "kilohertz", to_kilohertz, "toKilohertz"),
            (megahertz, "megahertz", to_megahertz, "toMegahertz"),
            (gigahertz, "gigahertz", to_gigahertz, "toGigahertz"),
            (terahertz, "terahertz", to_terahertz, "toTerahertz")
        ]
    ),
    (
        Velocity,
        JsVelocity,
        "Velocity",
        [
            (meters_per_second, "metersPerSecond", to_meters_per_second, "toMetersPerSecond"),
            (kilometers_per_second, "kilometersPerSecond", to_kilometers_per_second, "toKilometersPerSecond"),
            (astronomical_units_per_day, "astronomicalUnitsPerDay", to_astronomical_units_per_day, "toAstronomicalUnitsPerDay"),
            (fraction_of_speed_of_light, "fractionOfSpeedOfLight", to_fraction_of_speed_of_light, "toFractionOfSpeedOfLight")
        ]
    )
);
