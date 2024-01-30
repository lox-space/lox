/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::f64::consts::PI;

pub trait Antenna {
    fn gain(&self, frequency: f64) -> f64;
    fn beam_width(&self, frequency: f64) -> f64;
}

pub struct Parabolic {
    diameter: f64,
    efficiency: f64,
}

impl Antenna for Parabolic {
    fn gain(&self, frequency: f64) -> f64 {
        let a = area(self.diameter);
        let lambda = wavelength(frequency);
        let g = to_db(4.0 * PI * a / lambda.powi(2));
        g + to_db(self.efficiency)
    }

    fn beam_width(&self, frequency: f64) -> f64 {
        70.0 * wavelength(frequency) / self.diameter
    }
}

fn area(diameter: f64) -> f64 {
    PI * diameter.powi(2) / 4.0
}

const C0: f64 = 2.99792458e8;

fn wavelength(frequency: f64) -> f64 {
    C0 / frequency
}

fn to_db(val: f64) -> f64 {
    10.0 * val.log10()
}
