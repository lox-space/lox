use std::f64::consts::PI;

use lox_units::{Angle, Decibel, DecibelUnits, Distance, Frequency, coords::AzEl};

pub trait AntennaPattern {
    fn gain(&self, f: Frequency, azel: AzEl) -> f64;

    fn beamwidth(&self, f: Frequency) -> Angle;
}

pub struct ParabolicPattern {
    diameter: Distance,
    efficiency: f64,
}

impl ParabolicPattern {
    pub fn area(&self) -> f64 {
        PI * self.diameter.0.powi(2) / 4.0
    }

    pub fn peak_gain(&self, f: Frequency) -> Decibel {
        let lambda = f.wavelength();
        let g = (4.0 * PI * self.area() / lambda.0.powi(2)).db();
        g + self.efficiency.db()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
