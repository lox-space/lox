// SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MIT AND MPL-2.0

//! ITU-R P.453-13: The radio refractive index.
//!
//! Provides formulas for computing water vapour pressure, saturation vapour pressure,
//! and radio refractivity (dry, wet, and total). The globally-gridded wet-term
//! refractivity map is served by [`crate::ItuProvider::map_wet_term_radio_refractivity`].

use lox_core::units::{Pressure, Temperature};

// ── Pure formulas ───────────────────────────────────────────────────────────

/// Computes the saturation water vapour pressure over liquid water.
///
/// # Arguments
///
/// * `temperature` — Temperature
/// * `pressure` — Total atmospheric pressure
pub fn saturation_vapour_pressure(temperature: Temperature, pressure: Pressure) -> Pressure {
    Pressure::hpa(saturation_vapour_pressure_raw(
        temperature.to_kelvin() - 273.15,
        pressure.to_hpa(),
    ))
}

pub(crate) fn saturation_vapour_pressure_raw(t_celsius: f64, p_hpa: f64) -> f64 {
    let t = t_celsius;
    let ef = 1.0 + 1e-4 * (7.2 + p_hpa * (0.0320 + 5.9e-6 * t * t));
    ef * 6.1121 * ((18.678 - t / 234.5) * t / (t + 257.14)).exp()
}

/// Computes the saturation water vapour pressure over ice.
///
/// # Arguments
///
/// * `temperature` — Temperature
/// * `pressure` — Total atmospheric pressure
pub fn saturation_vapour_pressure_ice(temperature: Temperature, pressure: Pressure) -> Pressure {
    Pressure::hpa(saturation_vapour_pressure_ice_raw(
        temperature.to_kelvin() - 273.15,
        pressure.to_hpa(),
    ))
}

pub(crate) fn saturation_vapour_pressure_ice_raw(t_celsius: f64, p_hpa: f64) -> f64 {
    let t = t_celsius;
    let ef = 1.0 + 1e-4 * (2.2 + p_hpa * (0.0383 + 6.4e-6 * t * t));
    ef * 6.1115 * ((23.036 - t / 333.7) * t / (t + 279.82)).exp()
}

/// Computes the water vapour pressure from temperature, pressure, and humidity.
///
/// # Arguments
///
/// * `temperature` — Temperature
/// * `pressure` — Total atmospheric pressure
/// * `humidity_pct` — Relative humidity in %
pub fn water_vapour_pressure(
    temperature: Temperature,
    pressure: Pressure,
    humidity_pct: f64,
) -> Pressure {
    Pressure::hpa(water_vapour_pressure_raw(
        temperature.to_kelvin() - 273.15,
        pressure.to_hpa(),
        humidity_pct,
    ))
}

pub(crate) fn water_vapour_pressure_raw(t_celsius: f64, p_hpa: f64, humidity_pct: f64) -> f64 {
    humidity_pct * saturation_vapour_pressure_raw(t_celsius, p_hpa) / 100.0
}

/// Computes the wet term of radio refractivity (N-units).
///
/// # Arguments
///
/// * `e` — Water vapour pressure
/// * `temperature` — Temperature
pub fn wet_term_radio_refractivity(e: Pressure, temperature: Temperature) -> f64 {
    wet_term_radio_refractivity_raw(e.to_hpa(), temperature.to_kelvin())
}

pub(crate) fn wet_term_radio_refractivity_raw(e_hpa: f64, t_k: f64) -> f64 {
    72.0 * e_hpa / t_k + 3.75e5 * e_hpa / (t_k * t_k)
}

/// Computes the dry term of radio refractivity (N-units).
///
/// # Arguments
///
/// * `pd` — Dry air pressure (total pressure minus water vapour pressure)
/// * `temperature` — Temperature
pub fn dry_term_radio_refractivity(pd: Pressure, temperature: Temperature) -> f64 {
    dry_term_radio_refractivity_raw(pd.to_hpa(), temperature.to_kelvin())
}

pub(crate) fn dry_term_radio_refractivity_raw(pd_hpa: f64, t_k: f64) -> f64 {
    77.6 * pd_hpa / t_k
}

/// Computes the radio refractive index n (dimensionless).
///
/// # Arguments
///
/// * `pressure` — Dry air pressure
/// * `e` — Water vapour pressure
/// * `temperature` — Temperature
pub fn radio_refractive_index(pressure: Pressure, e: Pressure, temperature: Temperature) -> f64 {
    radio_refractive_index_raw(pressure.to_hpa(), e.to_hpa(), temperature.to_kelvin())
}

pub(crate) fn radio_refractive_index_raw(p_hpa: f64, e_hpa: f64, t_k: f64) -> f64 {
    let n_units = 77.6 * p_hpa / t_k + 72.0 * e_hpa / t_k + 3.75e5 * e_hpa / (t_k * t_k);
    1.0 + n_units * 1e-6
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;

    use super::*;

    #[test]
    fn test_saturation_vapour_pressure_at_20c() {
        // At 20°C, ~1013 hPa, e_s ≈ 23.4 hPa (well-known value)
        let es = saturation_vapour_pressure_raw(20.0, 1013.25);
        assert_approx_eq!(es, 23.4, atol <= 0.5);
    }

    #[test]
    fn test_saturation_vapour_pressure_at_0c() {
        // At 0°C, e_s ≈ 6.1 hPa
        let es = saturation_vapour_pressure_raw(0.0, 1013.25);
        assert_approx_eq!(es, 6.1, atol <= 0.2);
    }

    #[test]
    fn test_water_vapour_pressure() {
        let e = water_vapour_pressure_raw(20.0, 1013.25, 50.0);
        let es = saturation_vapour_pressure_raw(20.0, 1013.25);
        assert_approx_eq!(e, es / 2.0, rtol <= 1e-10);
    }

    #[test]
    fn test_wet_term_refractivity() {
        // At e=10 hPa, T=288 K
        let nw = wet_term_radio_refractivity_raw(10.0, 288.0);
        assert!(nw > 40.0 && nw < 60.0, "N_wet = {nw}");
    }

    #[test]
    fn test_dry_term_refractivity() {
        // At P=1013 hPa, T=288 K, N_dry ≈ 272
        let nd = dry_term_radio_refractivity_raw(1013.25, 288.0);
        assert_approx_eq!(nd, 272.0, atol <= 2.0);
    }

    #[test]
    fn test_refractive_index_near_unity() {
        let n = radio_refractive_index_raw(1013.25, 10.0, 288.0);
        assert!(n > 1.0 && n < 1.001, "n = {n}");
    }

    #[test]
    fn test_saturation_vapour_pressure_ice() {
        let es = saturation_vapour_pressure_ice_raw(-10.0, 1013.25);
        assert!(es > 0.0 && es < 5.0, "e_s(ice, -10°C) = {es}");
    }

    #[test]
    fn test_map_wet_term_radio_refractivity() {
        use crate::provider::test_fixture::provider;
        use lox_core::units::Angle;
        let p = provider();
        let nw = p
            .map_wet_term_radio_refractivity(Angle::degrees(40.4), Angle::degrees(-3.7), 50.0)
            .unwrap();
        assert!(nw > 20.0 && nw < 200.0, "N_wet(Madrid, 50%) = {nw}");
    }

    #[test]
    fn test_map_wet_term_radio_refractivity_interpolated() {
        use crate::provider::test_fixture::provider;
        use lox_core::units::Angle;
        let p = provider();
        let nw = p
            .map_wet_term_radio_refractivity(
                Angle::degrees(40.4),
                Angle::degrees(-3.7),
                7.5, // between 5 and 10
            )
            .unwrap();
        let nw_5 = p
            .map_wet_term_radio_refractivity(Angle::degrees(40.4), Angle::degrees(-3.7), 5.0)
            .unwrap();
        let nw_10 = p
            .map_wet_term_radio_refractivity(Angle::degrees(40.4), Angle::degrees(-3.7), 10.0)
            .unwrap();
        assert!(nw >= nw_5.min(nw_10) && nw <= nw_5.max(nw_10));
    }

    #[test]
    fn test_unitful_api_consistency() {
        let e_raw = saturation_vapour_pressure_raw(20.0, 1013.25);
        let e_unit =
            saturation_vapour_pressure(Temperature::kelvin(293.15), Pressure::hpa(1013.25));
        assert_approx_eq!(e_unit.to_hpa(), e_raw, rtol <= 1e-10);
    }

    #[test]
    fn test_unitful_saturation_vapour_pressure_ice() {
        let es =
            saturation_vapour_pressure_ice(Temperature::kelvin(263.15), Pressure::hpa(1013.25));
        assert!(es.to_hpa() > 0.0);
    }

    #[test]
    fn test_unitful_water_vapour_pressure() {
        let e = water_vapour_pressure(Temperature::kelvin(293.15), Pressure::hpa(1013.25), 50.0);
        assert!(e.to_hpa() > 0.0);
    }

    #[test]
    fn test_unitful_dry_term() {
        let nd = dry_term_radio_refractivity(Pressure::hpa(1013.25), Temperature::kelvin(288.0));
        assert_approx_eq!(nd, 272.0, atol <= 2.0);
    }

    #[test]
    fn test_unitful_radio_refractive_index() {
        let n = radio_refractive_index(
            Pressure::hpa(1013.25),
            Pressure::hpa(10.0),
            Temperature::kelvin(288.0),
        );
        assert!(n > 1.0 && n < 1.001);
    }
}
