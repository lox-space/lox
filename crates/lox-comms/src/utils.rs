// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Utility functions for RF calculations.

use std::f64::consts::PI;

use lox_core::units::{Angle, Decibel, Distance, Frequency};

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

/// Computes the slant range from a ground station to a satellite at a given elevation angle.
///
/// d = √(Re²·sin²(el) + 2·Re·h + h²) − Re·sin(el)
pub fn slant_range(elevation: Angle, earth_radius: Distance, altitude: Distance) -> Distance {
    let el = elevation.to_radians();
    let re = earth_radius.to_meters();
    let h = altitude.to_meters();
    let sin_el = el.sin();
    let d = (re * re * sin_el * sin_el + 2.0 * re * h + h * h).sqrt() - re * sin_el;
    Distance::meters(d)
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;
    use lox_core::units::{DistanceUnits, FrequencyUnits};

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

    #[test]
    fn test_slant_range_zenith() {
        // At 90° elevation, slant range = altitude
        let re = Distance::kilometers(6371.0);
        let h = Distance::kilometers(525.0);
        let range = slant_range(Angle::degrees(90.0), re, h);
        assert_approx_eq!(range.to_kilometers(), 525.0, atol <= 0.01);
    }

    #[test]
    fn test_slant_range_low_elevation() {
        let re = Distance::kilometers(6371.0);
        let h = Distance::kilometers(525.0);
        let range = slant_range(Angle::degrees(5.0), re, h);
        assert!(range.to_kilometers() > 1500.0);
        assert!(range.to_kilometers() < 3000.0);
    }

    #[test]
    fn test_slant_range_decreases_with_elevation() {
        let re = Distance::kilometers(6371.0);
        let h = Distance::kilometers(525.0);
        let range_low = slant_range(Angle::degrees(10.0), re, h);
        let range_high = slant_range(Angle::degrees(45.0), re, h);
        assert!(range_low.to_meters() > range_high.to_meters());
    }
}
