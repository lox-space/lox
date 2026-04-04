// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_space::core::{
    elements::{GravitationalParameter, Keplerian},
    units::{AngleUnits, DistanceUnits},
};
use lox_space::orbits::DynKeplerianOrbit;
use wasm_bindgen::prelude::*;

/// Initialize the WASM module with panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// A simple greeting function demonstrating string handling
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Lox Space WASM.", name)
}

/// Example math function: calculate orbital velocity
/// v = sqrt(μ / r)
#[wasm_bindgen]
pub fn orbital_velocity(gravitational_parameter: f64, radius: f64) -> f64 {
    (gravitational_parameter / radius).sqrt()
}

/// Example function: convert degrees to radians
#[wasm_bindgen]
pub fn deg_to_rad(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

/// Example function: convert radians to degrees
#[wasm_bindgen]
pub fn rad_to_deg(radians: f64) -> f64 {
    radians * 180.0 / std::f64::consts::PI
}

#[wasm_bindgen]
pub struct KeplerianOrbit(DynKeplerianOrbit);

#[wasm_bindgen]
impl KeplerianOrbit {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        todo!()
    }
}

#[wasm_bindgen]
pub struct KeplerianElements {
    semi_major_axis: f64,
    eccentricity: f64,
    inclination: f64,
    longitude_of_ascending_node: f64,
    argument_of_periapsis: f64,
    true_anomaly: f64,
}

#[wasm_bindgen]
impl KeplerianElements {
    #[wasm_bindgen(constructor)]
    pub fn new(
        semi_major_axis: f64,
        eccentricity: f64,
        inclination: f64,
        longitude_of_ascending_node: f64,
        argument_of_periapsis: f64,
        true_anomaly: f64,
    ) -> Self {
        Self {
            semi_major_axis,
            eccentricity,
            inclination,
            longitude_of_ascending_node,
            argument_of_periapsis,
            true_anomaly,
        }
    }

    #[wasm_bindgen]
    pub fn position(&self, grav_param: f64) -> Vec<f64> {
        let grav_param = GravitationalParameter::km3_per_s2(grav_param);
        self.to_lox()
            .to_cartesian(grav_param)
            .position()
            .to_array()
            .map(|v| v * 1e-3)
            .to_vec()
    }

    #[wasm_bindgen]
    pub fn trace(&self, grav_param: f64, n: usize) -> Positions {
        let grav_param = GravitationalParameter::km3_per_s2(grav_param);
        let orbit = self.to_lox().trace(grav_param, n).unwrap();

        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);
        let mut z = Vec::with_capacity(n);
        for c in orbit {
            x.push(c.x().to_kilometers());
            y.push(c.y().to_kilometers());
            z.push(c.z().to_kilometers());
        }

        Positions { x, y, z }
    }

    fn to_lox(&self) -> Keplerian {
        Keplerian::builder()
            .with_semi_major_axis(self.semi_major_axis.km(), self.eccentricity)
            .with_inclination(self.inclination.deg())
            .with_longitude_of_ascending_node(self.longitude_of_ascending_node.deg())
            .with_argument_of_periapsis(self.argument_of_periapsis.deg())
            .with_true_anomaly(self.true_anomaly.deg())
            .build()
            .unwrap()
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct Positions {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let result = greet("World");
        assert!(result.contains("Hello, World!"));
    }

    #[test]
    fn test_orbital_velocity() {
        // Earth's standard gravitational parameter: 398600.4418 km³/s²
        // Low Earth orbit radius: ~6700 km
        let velocity = orbital_velocity(398600.4418, 6700.0);
        assert!((velocity - 7.714).abs() < 0.01);
    }

    #[test]
    fn test_deg_to_rad() {
        let radians = deg_to_rad(180.0);
        assert!((radians - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_rad_to_deg() {
        let degrees = rad_to_deg(std::f64::consts::PI);
        assert!((degrees - 180.0).abs() < 1e-10);
    }
}
