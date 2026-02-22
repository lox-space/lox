// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Utility functions for RF calculations.

use std::f64::consts::PI;

use lox_core::units::{Decibel, Distance, Frequency};

/// Computes the free-space path loss (FSPL) in decibels.
///
/// FSPL = 10·log₁₀((4π·d / λ)²)
///
/// where `d` is the distance in meters and `λ` is the wavelength in meters.
pub fn free_space_path_loss(distance: Distance, frequency: Frequency) -> Decibel {
    let wavelength_m = frequency.wavelength().to_meters();
    let d_m = distance.to_meters();
    let ratio = 4.0 * PI * d_m / wavelength_m;
    Decibel::from_linear(ratio * ratio)
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DistanceUnits, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_fspl_known_value() {
        // 1000 km distance, 29 GHz frequency
        // Expected: 10*log10((4*pi*1e6 / (299792458/29e9))^2)
        // wavelength = 0.010337670965517241 m
        // ratio = 4*pi*1e6 / 0.010337... = 1.2167e9
        // FSPL = 10*log10(1.2167e9^2) = 10*log10(1.4804e18) ≈ 181.7 dB
        let distance = Distance::kilometers(1000.0);
        let frequency = 29.0.ghz();
        let fspl = free_space_path_loss(distance, frequency);
        // Cross-check: FSPL = 20*log10(d_m) + 20*log10(f_hz) + 20*log10(4*pi/c)
        //            = 20*log10(1e6) + 20*log10(29e9) + 20*log10(4*pi/299792458)
        //            = 120 + 209.248 + (-147.552) = 181.696 dB
        assert_approx_eq!(fspl.as_f64(), 181.696, atol <= 0.01);
    }

    #[test]
    fn test_fspl_one_wavelength() {
        // At distance = 1 wavelength, FSPL = 10*log10((4*pi)^2) ≈ 21.98 dB
        let frequency = 1.0.ghz();
        let wavelength = frequency.wavelength();
        let fspl = free_space_path_loss(wavelength, frequency);
        let expected = Decibel::from_linear((4.0 * PI) * (4.0 * PI));
        assert_approx_eq!(fspl.as_f64(), expected.as_f64(), rtol <= 1e-10);
    }

    #[test]
    fn test_fspl_doubles_with_double_distance() {
        // Doubling distance adds 6.02 dB (factor of 4 in power)
        let frequency = 2.0.ghz();
        let fspl_1 = free_space_path_loss(100.0.km(), frequency);
        let fspl_2 = free_space_path_loss(200.0.km(), frequency);
        let diff = fspl_2 - fspl_1;
        assert_approx_eq!(diff.as_f64(), 6.0206, atol <= 0.001);
    }
}
