// SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MIT AND MPL-2.0

//! ITU-R P.835-6: Reference standard atmospheres.
//!
//! Provides temperature, pressure, and water vapour density as a function of height
//! and latitude for standard, low-latitude, mid-latitude, and high-latitude atmospheres.

use lox_core::units::{Angle, Distance, Pressure, Temperature};

/// Season selector for latitude-dependent atmosphere profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Season {
    Summer,
    Winter,
}

// ── Standard atmosphere (Section 1) ──────────────────────────────────────────

/// Standard atmosphere temperature at geopotential height h.
pub fn standard_temperature(h: Distance) -> Temperature {
    Temperature::kelvin(standard_temperature_raw(h.to_kilometers()))
}

pub(crate) fn standard_temperature_raw(h_km: f64) -> f64 {
    let h_p = 6356.766 * h_km / (6356.766 + h_km);

    if h_p <= 11.0 {
        288.15 - 6.5 * h_p
    } else if h_p <= 20.0 {
        216.65
    } else if h_p <= 32.0 {
        216.65 + (h_p - 20.0)
    } else if h_p <= 47.0 {
        228.65 + 2.8 * (h_p - 32.0)
    } else if h_p <= 51.0 {
        270.65
    } else if h_p <= 71.0 {
        270.65 - 2.8 * (h_p - 51.0)
    } else if h_p <= 84.852 {
        214.65 - 2.0 * (h_p - 71.0)
    } else if (86.0..=91.0).contains(&h_km) {
        186.8673
    } else if h_km > 91.0 && h_km <= 100.0 {
        263.1905 - 76.3232 * (1.0 - ((h_km - 91.0) / 19.9429).powi(2)).sqrt()
    } else {
        195.08134
    }
}

/// Standard atmosphere pressure at geopotential height h.
pub fn standard_pressure(h: Distance) -> Pressure {
    Pressure::hpa(standard_pressure_raw(h.to_kilometers()))
}

pub(crate) fn standard_pressure_raw(h_km: f64) -> f64 {
    let h_p = 6356.766 * h_km / (6356.766 + h_km);

    if h_p <= 11.0 {
        1013.25 * (288.15 / (288.15 - 6.5 * h_p)).powf(-34.1632 / 6.5)
    } else if h_p <= 20.0 {
        226.3226 * (-34.1632 * (h_p - 11.0) / 216.65).exp()
    } else if h_p <= 32.0 {
        54.74980 * (216.65 / (216.65 + (h_p - 20.0))).powf(34.1632)
    } else if h_p <= 47.0 {
        8.680422 * (228.65 / (228.65 + 2.8 * (h_p - 32.0))).powf(34.1632 / 2.8)
    } else if h_p <= 51.0 {
        1.109106 * (-34.1632 * (h_p - 47.0) / 270.65).exp()
    } else if h_p <= 71.0 {
        0.6694167 * (270.65 / (270.65 - 2.8 * (h_p - 51.0))).powf(-34.1632 / 2.8)
    } else if h_p <= 84.852 {
        0.03956649 * (214.65 / (214.65 - 2.0 * (h_p - 71.0))).powf(-34.1632 / 2.0)
    } else if (86.0..=100.0).contains(&h_km) {
        (95.571899 - 4.011801 * h_km + 6.424731e-2 * h_km.powi(2) - 4.789660e-4 * h_km.powi(3)
            + 1.340543e-6 * h_km.powi(4))
        .exp()
    } else {
        1e-62
    }
}

/// Standard atmosphere water vapour density (g/m³) at height h.
pub fn standard_water_vapour_density(h: Distance, h_0: f64, rho_0: f64) -> f64 {
    standard_water_vapour_density_raw(h.to_kilometers(), h_0, rho_0)
}

fn standard_water_vapour_density_raw(h_km: f64, h_0: f64, rho_0: f64) -> f64 {
    rho_0 * (-h_km / h_0).exp()
}

/// Standard atmosphere water vapour pressure (hPa) at height h.
pub fn standard_water_vapour_pressure(h: Distance, h_0: f64, rho_0: f64) -> Pressure {
    let h_km = h.to_kilometers();
    let rho_h = standard_water_vapour_density_raw(h_km, h_0, rho_0);
    let t_h = standard_temperature_raw(h_km);
    Pressure::hpa(rho_h * t_h / 216.7)
}

// ── Low-latitude atmosphere (Section 2, |lat| < 22°) ────────────────────────

fn low_latitude_temperature(h: f64) -> f64 {
    if h < 17.0 {
        300.4222 - 6.3533 * h + 0.005886 * h.powi(2)
    } else if h < 47.0 {
        194.0 + (h - 17.0) * 2.533
    } else if h < 52.0 {
        270.0
    } else if h < 80.0 {
        270.0 - (h - 52.0) * 3.0714
    } else {
        184.0
    }
}

fn low_latitude_pressure(h: f64) -> f64 {
    const P10: f64 = 284.8526;
    const P72: f64 = 0.0313660;
    if h <= 10.0 {
        1012.0306 - 109.0338 * h + 3.6316 * h.powi(2)
    } else if h <= 72.0 {
        P10 * (-0.147 * (h - 10.0)).exp()
    } else {
        P72 * (-0.165 * (h - 72.0)).exp()
    }
}

fn low_latitude_water_vapour(h: f64) -> f64 {
    if h <= 15.0 {
        19.6542
            * (-0.2313 * h - 0.1122 * h.powi(2) + 0.01351 * h.powi(3) - 0.0005923 * h.powi(4)).exp()
    } else {
        0.0
    }
}

// ── Mid-latitude summer (Section 3.1, 22° ≤ |lat| < 45°) ───────────────────

fn mid_latitude_temperature_summer(h: f64) -> f64 {
    if h < 13.0 {
        294.9838 - 5.2159 * h - 0.07109 * h.powi(2)
    } else if h < 17.0 {
        215.15
    } else if h < 47.0 {
        215.15 * ((h - 17.0) * 0.008128).exp()
    } else if h < 53.0 {
        275.0
    } else if h < 80.0 {
        275.0 + 20.0 * (1.0 - ((h - 53.0) * 0.06).exp())
    } else {
        175.0
    }
}

fn mid_latitude_pressure_summer(h: f64) -> f64 {
    const P10: f64 = 283.7096;
    const P72: f64 = 0.03124022;
    if h <= 10.0 {
        1012.8186 - 111.5569 * h + 3.8646 * h.powi(2)
    } else if h <= 72.0 {
        P10 * (-0.147 * (h - 10.0)).exp()
    } else {
        P72 * (-0.165 * (h - 72.0)).exp()
    }
}

fn mid_latitude_water_vapour_summer(h: f64) -> f64 {
    if h <= 15.0 {
        14.3542 * (-0.4174 * h - 0.02290 * h.powi(2) + 0.001007 * h.powi(3)).exp()
    } else {
        0.0
    }
}

// ── Mid-latitude winter (Section 3.2) ───────────────────────────────────────

fn mid_latitude_temperature_winter(h: f64) -> f64 {
    if h < 10.0 {
        272.7241 - 3.6217 * h - 0.1759 * h.powi(2)
    } else if h < 33.0 {
        218.0
    } else if h < 47.0 {
        218.0 + (h - 33.0) * 3.3571
    } else if h < 53.0 {
        265.0
    } else if h < 80.0 {
        265.0 - (h - 53.0) * 2.0370
    } else {
        210.0
    }
}

fn mid_latitude_pressure_winter(h: f64) -> f64 {
    const P10: f64 = 258.9787;
    const P72: f64 = 0.02851702;
    if h <= 10.0 {
        1018.8627 - 124.2954 * h + 4.8307 * h.powi(2)
    } else if h <= 72.0 {
        P10 * (-0.147 * (h - 10.0)).exp()
    } else {
        P72 * (-0.155 * (h - 72.0)).exp()
    }
}

fn mid_latitude_water_vapour_winter(h: f64) -> f64 {
    if h <= 10.0 {
        3.4742 * (-0.2697 * h - 0.03604 * h.powi(2) + 0.0004489 * h.powi(3)).exp()
    } else {
        0.0
    }
}

// ── High-latitude summer (Section 4.1, |lat| ≥ 45°) ────────────────────────

fn high_latitude_temperature_summer(h: f64) -> f64 {
    if h < 10.0 {
        286.8374 - 4.7805 * h - 0.1402 * h.powi(2)
    } else if h < 23.0 {
        225.0
    } else if h < 48.0 {
        225.0 * ((h - 23.0) * 0.008317).exp()
    } else if h < 53.0 {
        277.0
    } else if h < 79.0 {
        277.0 - (h - 53.0) * 4.0769
    } else {
        171.0
    }
}

fn high_latitude_pressure_summer(h: f64) -> f64 {
    const P10: f64 = 269.6138;
    const P72: f64 = 0.04582115;
    if h <= 10.0 {
        1008.0278 - 113.2494 * h + 3.9408 * h.powi(2)
    } else if h <= 72.0 {
        P10 * (-0.140 * (h - 10.0)).exp()
    } else {
        P72 * (-0.165 * (h - 72.0)).exp()
    }
}

fn high_latitude_water_vapour_summer(h: f64) -> f64 {
    if h <= 15.0 {
        8.988 * (-0.3614 * h - 0.005402 * h.powi(2) - 0.001955 * h.powi(3)).exp()
    } else {
        0.0
    }
}

// ── High-latitude winter (Section 4.2) ──────────────────────────────────────

fn high_latitude_temperature_winter(h: f64) -> f64 {
    if h < 8.5 {
        257.4345 + 2.3474 * h - 1.5479 * h.powi(2) + 0.08473 * h.powi(3)
    } else if h < 30.0 {
        217.5
    } else if h < 50.0 {
        217.5 + (h - 30.0) * 2.125
    } else if h < 54.0 {
        260.0
    } else {
        260.0 - (h - 54.0) * 1.667
    }
}

fn high_latitude_pressure_winter(h: f64) -> f64 {
    const P10: f64 = 243.8718;
    const P72: f64 = 0.02685355;
    if h <= 10.0 {
        1010.8828 - 122.2411 * h + 4.554 * h.powi(2)
    } else if h <= 72.0 {
        P10 * (-0.147 * (h - 10.0)).exp()
    } else {
        P72 * (-0.150 * (h - 72.0)).exp()
    }
}

fn high_latitude_water_vapour_winter(h: f64) -> f64 {
    if h <= 10.0 {
        1.2319 * (0.07481 * h - 0.0981 * h.powi(2) + 0.00281 * h.powi(3)).exp()
    } else {
        0.0
    }
}

// ── Latitude-dependent dispatchers ──────────────────────────────────────────

/// Temperature at a given height, latitude, and season.
///
/// Dispatches to low/mid/high latitude profiles per P.835-6 Sections 2–4.
pub fn temperature(lat: Angle, h: Distance, season: Season) -> Temperature {
    Temperature::kelvin(temperature_raw(lat.to_degrees(), h.to_kilometers(), season))
}

fn temperature_raw(lat_deg: f64, h_km: f64, season: Season) -> f64 {
    let abs_lat = lat_deg.abs();
    if abs_lat < 22.0 {
        low_latitude_temperature(h_km)
    } else if abs_lat < 45.0 {
        match season {
            Season::Summer => mid_latitude_temperature_summer(h_km),
            Season::Winter => mid_latitude_temperature_winter(h_km),
        }
    } else {
        match season {
            Season::Summer => high_latitude_temperature_summer(h_km),
            Season::Winter => high_latitude_temperature_winter(h_km),
        }
    }
}

/// Pressure at a given height, latitude, and season.
pub fn pressure(lat: Angle, h: Distance, season: Season) -> Pressure {
    Pressure::hpa(pressure_raw(lat.to_degrees(), h.to_kilometers(), season))
}

fn pressure_raw(lat_deg: f64, h_km: f64, season: Season) -> f64 {
    let abs_lat = lat_deg.abs();
    if abs_lat < 22.0 {
        low_latitude_pressure(h_km)
    } else if abs_lat < 45.0 {
        match season {
            Season::Summer => mid_latitude_pressure_summer(h_km),
            Season::Winter => mid_latitude_pressure_winter(h_km),
        }
    } else {
        match season {
            Season::Summer => high_latitude_pressure_summer(h_km),
            Season::Winter => high_latitude_pressure_winter(h_km),
        }
    }
}

/// Water vapour density (g/m³) at a given height, latitude, and season.
pub fn water_vapour_density(lat: Angle, h: Distance, season: Season) -> f64 {
    water_vapour_density_raw(lat.to_degrees(), h.to_kilometers(), season)
}

fn water_vapour_density_raw(lat_deg: f64, h_km: f64, season: Season) -> f64 {
    let abs_lat = lat_deg.abs();
    if abs_lat < 22.0 {
        low_latitude_water_vapour(h_km)
    } else if abs_lat < 45.0 {
        match season {
            Season::Summer => mid_latitude_water_vapour_summer(h_km),
            Season::Winter => mid_latitude_water_vapour_winter(h_km),
        }
    } else {
        match season {
            Season::Summer => high_latitude_water_vapour_summer(h_km),
            Season::Winter => high_latitude_water_vapour_winter(h_km),
        }
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_standard_temperature_sea_level() {
        let t = standard_temperature(Distance::kilometers(0.0));
        assert_approx_eq!(t.to_kelvin(), 288.15, atol <= 1e-6);
    }

    #[test]
    fn test_standard_temperature_tropopause() {
        let t = standard_temperature(Distance::kilometers(11.0));
        assert_approx_eq!(t.to_kelvin(), 216.77, atol <= 0.1);
    }

    #[test]
    fn test_standard_pressure_sea_level() {
        let p = standard_pressure(Distance::kilometers(0.0));
        assert_approx_eq!(p.to_hpa(), 1013.25, atol <= 1e-6);
    }

    #[test]
    fn test_standard_pressure_decreases_with_height() {
        let p0 = standard_pressure(Distance::kilometers(0.0));
        let p5 = standard_pressure(Distance::kilometers(5.0));
        let p10 = standard_pressure(Distance::kilometers(10.0));
        assert!(p5 < p0);
        assert!(p10 < p5);
    }

    #[test]
    fn test_water_vapour_decreases_with_height() {
        let rho0 = standard_water_vapour_density(Distance::kilometers(0.0), 2.0, 7.5);
        let rho5 = standard_water_vapour_density(Distance::kilometers(5.0), 2.0, 7.5);
        assert_approx_eq!(rho0, 7.5, atol <= 1e-10);
        assert!(rho5 < rho0);
    }

    // ── Low-latitude profiles ────────────────────────────────────────

    #[test]
    fn test_low_latitude_temperature_all_layers() {
        assert_approx_eq!(low_latitude_temperature(0.0), 300.4222, atol <= 1e-3);
        assert!(low_latitude_temperature(20.0) > 194.0); // 17–47 km
        assert_approx_eq!(low_latitude_temperature(50.0), 270.0, atol <= 1e-3); // 47–52 km
        assert!(low_latitude_temperature(60.0) < 270.0); // 52–80 km
        assert_approx_eq!(low_latitude_temperature(85.0), 184.0, atol <= 1e-3); // >80 km
    }

    #[test]
    fn test_low_latitude_pressure_all_layers() {
        assert!(low_latitude_pressure(5.0) > 0.0); // 0–10 km
        assert!(low_latitude_pressure(20.0) > 0.0); // 10–72 km
        assert!(low_latitude_pressure(80.0) > 0.0); // >72 km
        assert!(low_latitude_pressure(5.0) > low_latitude_pressure(20.0));
    }

    #[test]
    fn test_low_latitude_water_vapour() {
        assert!(low_latitude_water_vapour(0.0) > 10.0);
        assert!(low_latitude_water_vapour(10.0) > 0.0);
        assert_approx_eq!(low_latitude_water_vapour(20.0), 0.0, atol <= 1e-10);
    }

    // ── Mid-latitude profiles ───────────────────────────────────────

    #[test]
    fn test_mid_latitude_summer_all_layers() {
        assert!(mid_latitude_temperature_summer(0.0) > 290.0);
        assert_approx_eq!(mid_latitude_temperature_summer(15.0), 215.15, atol <= 1e-3);
        assert!(mid_latitude_temperature_summer(30.0) > 215.0);
        assert_approx_eq!(mid_latitude_temperature_summer(50.0), 275.0, atol <= 1e-3);
        assert!(mid_latitude_temperature_summer(60.0) < 275.0);
        assert_approx_eq!(mid_latitude_temperature_summer(85.0), 175.0, atol <= 1e-3);
    }

    #[test]
    fn test_mid_latitude_summer_pressure() {
        assert!(mid_latitude_pressure_summer(5.0) > 0.0);
        assert!(mid_latitude_pressure_summer(20.0) > 0.0);
        assert!(mid_latitude_pressure_summer(80.0) > 0.0);
    }

    #[test]
    fn test_mid_latitude_summer_water_vapour() {
        assert!(mid_latitude_water_vapour_summer(0.0) > 10.0);
        assert_approx_eq!(mid_latitude_water_vapour_summer(20.0), 0.0, atol <= 1e-10);
    }

    #[test]
    fn test_mid_latitude_winter_all_layers() {
        assert!(mid_latitude_temperature_winter(0.0) > 260.0);
        assert_approx_eq!(mid_latitude_temperature_winter(15.0), 218.0, atol <= 1e-3);
        assert!(mid_latitude_temperature_winter(40.0) > 218.0);
        assert_approx_eq!(mid_latitude_temperature_winter(50.0), 265.0, atol <= 1e-3);
        assert!(mid_latitude_temperature_winter(60.0) < 265.0);
        assert_approx_eq!(mid_latitude_temperature_winter(85.0), 210.0, atol <= 1e-3);
    }

    #[test]
    fn test_mid_latitude_winter_pressure() {
        assert!(mid_latitude_pressure_winter(5.0) > 0.0);
        assert!(mid_latitude_pressure_winter(20.0) > 0.0);
        assert!(mid_latitude_pressure_winter(80.0) > 0.0);
    }

    #[test]
    fn test_mid_latitude_winter_water_vapour() {
        assert!(mid_latitude_water_vapour_winter(0.0) > 1.0);
        assert_approx_eq!(mid_latitude_water_vapour_winter(15.0), 0.0, atol <= 1e-10);
    }

    // ── High-latitude profiles ──────────────────────────────────────

    #[test]
    fn test_high_latitude_summer_all_layers() {
        assert!(high_latitude_temperature_summer(0.0) > 280.0);
        assert_approx_eq!(high_latitude_temperature_summer(15.0), 225.0, atol <= 1e-3);
        assert!(high_latitude_temperature_summer(30.0) > 225.0);
        assert_approx_eq!(high_latitude_temperature_summer(50.0), 277.0, atol <= 1e-3);
        assert!(high_latitude_temperature_summer(60.0) < 277.0);
        assert_approx_eq!(high_latitude_temperature_summer(85.0), 171.0, atol <= 1e-3);
    }

    #[test]
    fn test_high_latitude_summer_pressure() {
        assert!(high_latitude_pressure_summer(5.0) > 0.0);
        assert!(high_latitude_pressure_summer(20.0) > 0.0);
        assert!(high_latitude_pressure_summer(80.0) > 0.0);
    }

    #[test]
    fn test_high_latitude_summer_water_vapour() {
        assert!(high_latitude_water_vapour_summer(0.0) > 5.0);
        assert_approx_eq!(high_latitude_water_vapour_summer(20.0), 0.0, atol <= 1e-10);
    }

    #[test]
    fn test_high_latitude_winter_all_layers() {
        assert!(high_latitude_temperature_winter(0.0) > 250.0);
        assert_approx_eq!(high_latitude_temperature_winter(15.0), 217.5, atol <= 1e-3);
        assert!(high_latitude_temperature_winter(40.0) > 217.5);
        assert_approx_eq!(high_latitude_temperature_winter(52.0), 260.0, atol <= 1e-3);
        assert!(high_latitude_temperature_winter(60.0) < 260.0);
    }

    #[test]
    fn test_high_latitude_winter_pressure() {
        assert!(high_latitude_pressure_winter(5.0) > 0.0);
        assert!(high_latitude_pressure_winter(20.0) > 0.0);
        assert!(high_latitude_pressure_winter(80.0) > 0.0);
    }

    #[test]
    fn test_high_latitude_winter_water_vapour() {
        assert!(high_latitude_water_vapour_winter(0.0) > 1.0);
        assert_approx_eq!(high_latitude_water_vapour_winter(15.0), 0.0, atol <= 1e-10);
    }

    // ── Standard atmosphere edge cases ──────────────────────────────

    #[test]
    fn test_standard_temperature_high_altitude() {
        // 86–91 km
        assert_approx_eq!(standard_temperature_raw(88.0), 186.8673, atol <= 1e-3);
        // 91–100 km
        assert!(standard_temperature_raw(95.0) > 180.0);
        // > 100 km
        assert_approx_eq!(standard_temperature_raw(105.0), 195.08134, atol <= 1e-3);
    }

    #[test]
    fn test_standard_pressure_high_altitude() {
        // 86–100 km
        assert!(standard_pressure_raw(90.0) > 0.0);
        // > 100 km
        assert!(standard_pressure_raw(105.0) > 0.0);
    }

    #[test]
    fn test_standard_pressure_all_layers() {
        // Each layer boundary
        for h in [5.0, 15.0, 25.0, 40.0, 49.0, 60.0, 75.0] {
            assert!(standard_pressure_raw(h) > 0.0, "P({h}) should be > 0");
        }
    }

    #[test]
    fn test_standard_water_vapour_pressure() {
        let e = standard_water_vapour_pressure(Distance::kilometers(0.0), 2.0, 7.5);
        assert!(e.to_hpa() > 0.0);
    }

    // ── Latitude dispatchers ────────────────────────────────────────

    #[test]
    fn test_latitude_dispatcher_low() {
        let t = temperature(
            Angle::degrees(10.0),
            Distance::kilometers(0.0),
            Season::Summer,
        );
        assert_approx_eq!(t.to_kelvin(), low_latitude_temperature(0.0), atol <= 1e-10);
    }

    #[test]
    fn test_latitude_dispatcher_mid_summer() {
        let t = temperature(
            Angle::degrees(35.0),
            Distance::kilometers(0.0),
            Season::Summer,
        );
        assert_approx_eq!(
            t.to_kelvin(),
            mid_latitude_temperature_summer(0.0),
            atol <= 1e-10
        );
    }

    #[test]
    fn test_latitude_dispatcher_mid_winter() {
        let t = temperature(
            Angle::degrees(35.0),
            Distance::kilometers(0.0),
            Season::Winter,
        );
        assert_approx_eq!(
            t.to_kelvin(),
            mid_latitude_temperature_winter(0.0),
            atol <= 1e-10
        );
    }

    #[test]
    fn test_latitude_dispatcher_high_summer() {
        let t = temperature(
            Angle::degrees(60.0),
            Distance::kilometers(0.0),
            Season::Summer,
        );
        assert_approx_eq!(
            t.to_kelvin(),
            high_latitude_temperature_summer(0.0),
            atol <= 1e-10
        );
    }

    #[test]
    fn test_latitude_dispatcher_high_winter() {
        let t = temperature(
            Angle::degrees(60.0),
            Distance::kilometers(5.0),
            Season::Winter,
        );
        assert_approx_eq!(
            t.to_kelvin(),
            high_latitude_temperature_winter(5.0),
            atol <= 1e-10
        );
    }

    #[test]
    fn test_pressure_dispatchers() {
        let p_low = pressure(
            Angle::degrees(10.0),
            Distance::kilometers(0.0),
            Season::Summer,
        );
        let p_mid_s = pressure(
            Angle::degrees(35.0),
            Distance::kilometers(0.0),
            Season::Summer,
        );
        let p_mid_w = pressure(
            Angle::degrees(35.0),
            Distance::kilometers(0.0),
            Season::Winter,
        );
        let p_high_s = pressure(
            Angle::degrees(60.0),
            Distance::kilometers(0.0),
            Season::Summer,
        );
        let p_high_w = pressure(
            Angle::degrees(60.0),
            Distance::kilometers(0.0),
            Season::Winter,
        );
        assert!(p_low.to_hpa() > 900.0);
        assert!(p_mid_s.to_hpa() > 900.0);
        assert!(p_mid_w.to_hpa() > 900.0);
        assert!(p_high_s.to_hpa() > 900.0);
        assert!(p_high_w.to_hpa() > 900.0);
    }

    #[test]
    fn test_water_vapour_dispatchers() {
        let w_low = water_vapour_density(
            Angle::degrees(10.0),
            Distance::kilometers(0.0),
            Season::Summer,
        );
        let w_mid_s = water_vapour_density(
            Angle::degrees(35.0),
            Distance::kilometers(0.0),
            Season::Summer,
        );
        let w_mid_w = water_vapour_density(
            Angle::degrees(35.0),
            Distance::kilometers(0.0),
            Season::Winter,
        );
        let w_high_s = water_vapour_density(
            Angle::degrees(60.0),
            Distance::kilometers(0.0),
            Season::Summer,
        );
        let w_high_w = water_vapour_density(
            Angle::degrees(60.0),
            Distance::kilometers(0.0),
            Season::Winter,
        );
        assert!(w_low > 0.0);
        assert!(w_mid_s > 0.0);
        assert!(w_mid_w > 0.0);
        assert!(w_high_s > 0.0);
        assert!(w_high_w > 0.0);
    }
}
