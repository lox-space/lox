// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Power Flux Density (PFD) calculations and ITU regulatory masks.

use std::f64::consts::PI;

use lox_core::units::{Angle, Decibel, Distance, Frequency};

/// Computes the power flux density in dBW/m²/ref_bw.
///
/// PFD = 10·log₁₀(EIRP_linear / (4π·d²) · (ref_bw / occupied_bw))
pub fn power_flux_density(
    eirp: Decibel,
    distance: Distance,
    occupied_bw: Frequency,
    reference_bw: Frequency,
) -> Decibel {
    let eirp_w = eirp.to_linear();
    let d_m = distance.to_meters();
    let pfd_linear =
        eirp_w / (4.0 * PI * d_m * d_m) * (reference_bw.to_hertz() / occupied_bw.to_hertz());
    Decibel::from_linear(pfd_linear)
}

/// Computes the ITU RR Article 21.16 PFD mask value for a given elevation angle.
///
/// - θ < 5°: `start_val`
/// - 5° ≤ θ < 25°: `start_val + 0.5 · (θ − 5)`
/// - θ ≥ 25°: `end_val`
///
/// `start_val` and `end_val` are in dBW/m²/ref_bw.
pub fn pfd_mask(elevation: Angle, start_val: Decibel, end_val: Decibel) -> Decibel {
    let el_deg = elevation.to_degrees();
    debug_assert!(
        (end_val.as_f64() - start_val.as_f64() - 10.0).abs() < 1e-6,
        "ITU Art. 21.16 mask requires end_val = start_val + 10 dB, got start_val={}, end_val={}",
        start_val.as_f64(),
        end_val.as_f64(),
    );
    if el_deg < 5.0 {
        start_val
    } else if el_deg < 25.0 {
        Decibel::new(start_val.as_f64() + 0.5 * (el_deg - 5.0))
    } else {
        end_val
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_pfd_isotropic() {
        // 0 dBW EIRP, 1000 km, equal bandwidths
        // PFD = 1 / (4*pi*(1e6)^2) = 7.958e-14 → -131.0 dBW/m²
        let pfd = power_flux_density(0.0.db(), Distance::kilometers(1000.0), 1.0.mhz(), 1.0.mhz());
        let expected = 10.0 * (1.0 / (4.0 * std::f64::consts::PI * 1e12)).log10();
        assert_approx_eq!(pfd.as_f64(), expected, atol <= 0.01);
    }

    #[test]
    fn test_pfd_bandwidth_scaling() {
        // ref_bw < occupied_bw should reduce PFD
        let pfd_wide = power_flux_density(
            10.0.db(),
            Distance::kilometers(500.0),
            10.0.mhz(),
            1.0.mhz(),
        );
        let pfd_equal =
            power_flux_density(10.0.db(), Distance::kilometers(500.0), 1.0.mhz(), 1.0.mhz());
        // 10 dB difference (10 MHz vs 1 MHz)
        assert_approx_eq!((pfd_equal - pfd_wide).as_f64(), 10.0, atol <= 0.01);
    }

    #[test]
    fn test_pfd_mask_low_elevation() {
        let mask = pfd_mask(Angle::degrees(0.0), (-154.0).db(), (-144.0).db());
        assert_approx_eq!(mask.as_f64(), -154.0, atol <= 1e-10);
        let mask = pfd_mask(Angle::degrees(4.9), (-154.0).db(), (-144.0).db());
        assert_approx_eq!(mask.as_f64(), -154.0, atol <= 1e-10);
    }

    #[test]
    fn test_pfd_mask_transition() {
        // At 15°: -154 + 0.5*(15-5) = -154 + 5 = -149
        let mask = pfd_mask(Angle::degrees(15.0), (-154.0).db(), (-144.0).db());
        assert_approx_eq!(mask.as_f64(), -149.0, atol <= 1e-10);
    }

    #[test]
    fn test_pfd_mask_high_elevation() {
        let mask = pfd_mask(Angle::degrees(25.0), (-154.0).db(), (-144.0).db());
        assert_approx_eq!(mask.as_f64(), -144.0, atol <= 1e-10);
        let mask = pfd_mask(Angle::degrees(90.0), (-154.0).db(), (-144.0).db());
        assert_approx_eq!(mask.as_f64(), -144.0, atol <= 1e-10);
    }

    #[test]
    fn test_pfd_mask_boundary_at_5_degrees() {
        // Exactly at 5°: should enter transition zone
        let mask = pfd_mask(Angle::degrees(5.0), (-154.0).db(), (-144.0).db());
        assert_approx_eq!(mask.as_f64(), -154.0, atol <= 1e-10);
    }

    #[test]
    fn test_pfd_with_high_eirp() {
        // 30 dBW EIRP, 500 km, 10 MHz occupied, 4 kHz ref
        let pfd_val = power_flux_density(
            30.0.db(),
            Distance::kilometers(500.0),
            10.0.mhz(),
            Frequency::hertz(4e3),
        );
        let eirp_w = 10.0_f64.powf(3.0); // 30 dBW = 1000 W
        let d_m = 500e3;
        let expected =
            10.0 * (eirp_w / (4.0 * std::f64::consts::PI * d_m * d_m) * (4e3 / 10e6)).log10();
        assert_approx_eq!(pfd_val.as_f64(), expected, atol <= 0.01);
    }
}
