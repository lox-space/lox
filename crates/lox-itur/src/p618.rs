// SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MIT AND MPL-2.0

//! ITU-R P.618-14: Propagation data for Earth-space telecommunication systems.
//!
//! Provides rain attenuation, tropospheric scintillation, cross-polarization
//! discrimination, and combined atmospheric attenuation on slant paths.

use lox_core::units::{Angle, Decibel, Distance, Frequency};

use crate::{p453, p837, p838, p839, p1511};

/// Computes rain attenuation exceeded for `p` % of the average year (P.618-13).
///
/// # Arguments
///
/// * `lat` — Latitude
/// * `lon` — Longitude
/// * `frequency` — Frequency
/// * `elevation` — Elevation angle (must be > 0)
/// * `p` — Exceedance probability (% of average year), range [0.001, 5]
/// * `polarisation_tilt` — Polarisation tilt angle (45° for circular)
/// * `station_altitude` — Station altitude (None to look up from P.1511)
pub fn rain_attenuation(
    lat: Angle,
    lon: Angle,
    frequency: Frequency,
    elevation: Angle,
    p: f64,
    polarisation_tilt: Angle,
    station_altitude: Option<Distance>,
) -> Decibel {
    Decibel::new(rain_attenuation_raw(
        lat.to_degrees(),
        lon.to_degrees(),
        frequency.to_gigahertz(),
        elevation.to_degrees().max(5.0),
        p,
        polarisation_tilt.to_degrees(),
        station_altitude.map(|d| d.to_kilometers()),
    ))
}

/// Grid-free core of P.618 rain attenuation. Pure math.
///
/// Takes pre-fetched grid values (`hs_km`, `hr_km`, `r001`) so the same code
/// is shared between the legacy free-fn entry point and the ItuProvider method.
#[allow(clippy::too_many_arguments)]
pub(crate) fn rain_attenuation_core(
    lat_deg: f64,
    f_ghz: f64,
    el_deg: f64,
    p: f64,
    tau_deg: f64,
    hs_km: f64,
    hr_km: f64,
    r001: f64,
) -> f64 {
    let re = 8500.0; // Effective Earth radius (km)
    let hs = hs_km;
    let hr = hr_km;

    // If station is above rain height, no rain attenuation
    if hr <= hs {
        return 0.0;
    }

    // Step 2: Slant-path length (km)
    let el_rad = el_deg.to_radians();
    let sin_el = el_rad.sin();
    let ls = if el_deg >= 5.0 {
        (hr - hs) / sin_el
    } else {
        2.0 * (hr - hs) / ((sin_el.powi(2) + 2.0 * (hr - hs) / re).sqrt() + sin_el)
    };

    // Step 3: Horizontal projection
    let lg = ls * el_rad.cos();

    // Step 4–5: Rainfall rate and specific attenuation
    let r001 = r001.max(1e-10);
    let gamma_r = p838::rain_specific_attenuation_raw(r001, f_ghz, el_deg, tau_deg);

    // Step 6: Horizontal reduction factor r_0.01
    let r_001 =
        1.0 / (1.0 + 0.78 * (lg * gamma_r / f_ghz).sqrt() - 0.38 * (1.0 - (-2.0 * lg).exp()));

    // Step 7: Vertical adjustment factor v_0.01
    let eta = ((hr - hs) / (lg * r_001)).atan().to_degrees();
    let lr = if eta > el_deg {
        lg * r_001 / el_rad.cos()
    } else {
        (hr - hs) / sin_el
    };

    let xi = if lat_deg.abs() < 36.0 {
        36.0 - lat_deg.abs()
    } else {
        0.0
    };

    let v001 = 1.0
        / (1.0
            + sin_el.sqrt()
                * (31.0 * (1.0 - (-(el_deg / (1.0 + xi))).exp()) * (lr * gamma_r).sqrt()
                    / f_ghz.powi(2)
                    - 0.45));

    // Step 8: Effective path length
    let le = lr * v001;

    // Step 9: Attenuation exceeded for 0.01%
    let a001 = gamma_r * le;

    // Step 10: Scale to other percentages
    if (p - 0.01).abs() < 1e-10 {
        return a001;
    }

    let beta = if p >= 1.0 || lat_deg.abs() >= 36.0 {
        0.0
    } else if el_deg > 25.0 {
        -0.005 * (lat_deg.abs() - 36.0)
    } else {
        -0.005 * (lat_deg.abs() - 36.0) + 1.8 - 4.25 * sin_el
    };

    let exponent =
        -(0.655 + 0.033 * p.ln() - 0.045 * a001.max(1e-10).ln() - beta * (1.0 - p) * sin_el);
    a001 * (p / 0.01_f64).powf(exponent)
}

pub(crate) fn rain_attenuation_raw(
    lat_deg: f64,
    lon_deg: f64,
    f_ghz: f64,
    el_deg: f64,
    p: f64,
    tau_deg: f64,
    hs_km: Option<f64>,
) -> f64 {
    let lat = Angle::degrees(lat_deg);
    let lon = Angle::degrees(lon_deg);
    let hs = hs_km.unwrap_or_else(|| p1511::topographic_altitude(lat, lon).to_kilometers());
    let hr = p839::rain_height(lat, lon).to_kilometers();
    let r001 = p837::rainfall_rate_r001(lat, lon);
    rain_attenuation_core(lat_deg, f_ghz, el_deg, p, tau_deg, hs, hr, r001)
}

/// Computes the standard deviation of tropospheric scintillation (dB).
///
/// This is steps 1–7 of the scintillation method (P.618-13 §2.4.1).
///
/// # Arguments
///
/// * `frequency` — Frequency
/// * `elevation` — Elevation angle
/// * `diameter` — Physical antenna diameter
/// * `eta` — Antenna efficiency (typically 0.5)
/// * `n_wet` — Wet term of radio refractivity (N-units)
pub fn scintillation_attenuation_sigma(
    frequency: Frequency,
    elevation: Angle,
    diameter: Distance,
    eta: f64,
    n_wet: f64,
) -> Decibel {
    Decibel::new(scintillation_attenuation_sigma_raw(
        frequency.to_gigahertz(),
        elevation.to_degrees().max(5.0),
        diameter.to_meters(),
        eta,
        n_wet,
    ))
}

pub(crate) fn scintillation_attenuation_sigma_raw(
    f_ghz: f64,
    el_deg: f64,
    d_m: f64,
    eta: f64,
    n_wet: f64,
) -> f64 {
    let h_l = 1000.0; // Turbulent layer height (m)

    // Step 3: Reference signal amplitude standard deviation
    let sigma_ref = 3.6e-3 + 1e-4 * n_wet;

    // Step 4: Effective path length
    let sin_el = el_deg.to_radians().sin();
    let l = 2.0 * h_l / ((sin_el.powi(2) + 2.35e-4).sqrt() + sin_el);

    // Step 5: Effective antenna diameter
    let d_eff = eta.sqrt() * d_m;

    // Step 6: Antenna averaging factor
    let x = 1.22 * d_eff.powi(2) * f_ghz / l;
    let g = if x >= 7.0 {
        0.0
    } else {
        let term1 = 3.86 * (x * x + 1.0).powf(11.0 / 12.0) * (11.0 / 6.0 * x.recip().atan()).sin();
        let term2 = 7.08 * x.powf(5.0 / 6.0);
        (term1 - term2).max(0.0).sqrt()
    };

    // Step 7: Signal standard deviation
    sigma_ref * f_ghz.powf(7.0 / 12.0) * g / sin_el.powf(1.2)
}

/// Computes the tropospheric scintillation fade depth exceeded for `p` %
/// of the average year (P.618-13).
///
/// # Arguments
///
/// * `frequency` — Frequency
/// * `elevation` — Elevation angle
/// * `p` — Exceedance probability (% of average year)
/// * `diameter` — Physical antenna diameter
/// * `eta` — Antenna efficiency (typically 0.5)
/// * `n_wet` — Wet term of radio refractivity (if None, uses P.453 map at 50th percentile)
/// * `lat` — Latitude (used when `n_wet` is None)
/// * `lon` — Longitude (used when `n_wet` is None)
#[allow(clippy::too_many_arguments)]
pub fn scintillation_attenuation(
    frequency: Frequency,
    elevation: Angle,
    p: f64,
    diameter: Distance,
    eta: f64,
    n_wet: Option<f64>,
    lat: Angle,
    lon: Angle,
) -> Decibel {
    Decibel::new(scintillation_attenuation_raw(
        frequency.to_gigahertz(),
        elevation.to_degrees().max(5.0),
        p,
        diameter.to_meters(),
        eta,
        n_wet,
        lat.to_degrees(),
        lon.to_degrees(),
    ))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn scintillation_attenuation_raw(
    f_ghz: f64,
    el_deg: f64,
    p: f64,
    d_m: f64,
    eta: f64,
    n_wet: Option<f64>,
    lat_deg: f64,
    lon_deg: f64,
) -> f64 {
    let n_wet = n_wet.unwrap_or_else(|| {
        p453::map_wet_term_radio_refractivity(
            Angle::degrees(lat_deg),
            Angle::degrees(lon_deg),
            50.0,
        )
    });

    let sigma = scintillation_attenuation_sigma_raw(f_ghz, el_deg, d_m, eta, n_wet);

    // Step 8: Time percentage factor
    let log_p = p.log10();
    let a_p = -0.061 * log_p.powi(3) + 0.072 * log_p.powi(2) - 1.71 * log_p + 3.0;

    // Step 9: Fade depth
    a_p * sigma
}

/// Computes the rain cross-polarization discrimination XPD (dB) not exceeded
/// for `p` % of the time (P.618-13 §4.1).
///
/// Valid for 4 < f < 55 GHz and el < 60°.
///
/// # Arguments
///
/// * `a_rain` — Co-polar rain attenuation exceeded for `p` % of time
/// * `frequency` — Frequency
/// * `elevation` — Elevation angle
/// * `p` — Exceedance probability (% of average year)
/// * `polarisation_tilt` — Polarisation tilt angle (45° for circular)
pub fn rain_cross_polarization_discrimination(
    a_rain: Decibel,
    frequency: Frequency,
    elevation: Angle,
    p: f64,
    polarisation_tilt: Angle,
) -> Decibel {
    Decibel::new(rain_cross_polarization_discrimination_raw(
        a_rain.as_f64(),
        frequency.to_gigahertz(),
        elevation.to_degrees(),
        p,
        polarisation_tilt.to_degrees(),
    ))
}

pub(crate) fn rain_cross_polarization_discrimination_raw(
    a_rain: f64,
    f_ghz: f64,
    el_deg: f64,
    p: f64,
    tau_deg: f64,
) -> f64 {
    // Handle 4–6 GHz scaling
    let (f, scale_from) = if (4.0..6.0).contains(&f_ghz) {
        (6.0, Some(f_ghz))
    } else {
        (f_ghz, None)
    };

    // Step 1: Frequency-dependent term
    let c_f = if f < 9.0 {
        60.0 * f.log10() - 28.3
    } else if f < 36.0 {
        26.0 * f.log10() + 4.1
    } else {
        35.9 * f.log10() - 11.3
    };

    // Step 2: Rain attenuation dependent term
    let v = if f < 9.0 {
        30.8 * f.powf(-0.21)
    } else if f < 20.0 {
        12.8 * f.powf(0.19)
    } else if f < 40.0 {
        22.6
    } else {
        13.0 * f.powf(0.15)
    };
    let c_a = v * a_rain.max(1e-10).log10();

    // Step 3: Polarization improvement factor
    let c_tau = -10.0 * (1.0 - 0.484 * (1.0 + (4.0 * tau_deg).to_radians().cos())).log10();

    // Step 4: Elevation angle-dependent term
    let c_theta = -40.0 * el_deg.to_radians().cos().log10();

    // Step 5: Canting angle dependent term
    let c_sigma = if p <= 0.001 {
        0.0053 * 225.0 // 15²
    } else if p <= 0.01 {
        0.0053 * 100.0 // 10²
    } else if p <= 0.1 {
        0.0053 * 25.0 // 5²
    } else {
        0.0
    };

    // Step 6: Rain XPD
    let xpd_rain = c_f - c_a + c_tau + c_theta + c_sigma;

    // Step 7: Ice crystal dependent term
    let c_ice = xpd_rain * (0.3 + 0.1 * p.log10()) / 2.0;

    // Step 8: Total XPD including ice
    let mut xpd_p = xpd_rain - c_ice;

    // Scale back if frequency was in 4–6 GHz range
    if let Some(f_orig) = scale_from {
        let tau_factor = (1.0 - 0.484 * (1.0 + (4.0 * tau_deg).to_radians().cos())).sqrt();
        xpd_p -= 20.0 * (f_orig * tau_factor / (f * tau_factor)).log10();
    }

    xpd_p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scintillation_increases_with_frequency() {
        let s1 = scintillation_attenuation_raw(10.0, 30.0, 0.01, 1.2, 0.5, Some(50.0), 0.0, 0.0);
        let s2 = scintillation_attenuation_raw(30.0, 30.0, 0.01, 1.2, 0.5, Some(50.0), 0.0, 0.0);
        assert!(
            s2 > s1,
            "Scintillation should increase with frequency: {} vs {}",
            s1,
            s2
        );
    }

    #[test]
    fn test_scintillation_decreases_with_elevation() {
        let s1 = scintillation_attenuation_raw(14.25, 10.0, 0.01, 1.2, 0.5, Some(50.0), 0.0, 0.0);
        let s2 = scintillation_attenuation_raw(14.25, 45.0, 0.01, 1.2, 0.5, Some(50.0), 0.0, 0.0);
        assert!(
            s2 < s1,
            "Scintillation should decrease with elevation: {} vs {}",
            s1,
            s2
        );
    }

    #[test]
    fn test_scintillation_reasonable_values() {
        let s = scintillation_attenuation_raw(14.25, 30.0, 0.01, 1.2, 0.5, Some(50.0), 0.0, 0.0);
        assert!(s > 0.0 && s < 2.0, "scintillation = {} dB", s);
    }

    #[test]
    fn test_xpd_reasonable_values() {
        // At 14 GHz, 2 dB rain attenuation, XPD should be >20 dB
        let xpd = rain_cross_polarization_discrimination_raw(2.0, 14.0, 30.0, 0.01, 45.0);
        assert!(xpd > 15.0 && xpd < 50.0, "XPD = {} dB", xpd);
    }

    #[test]
    fn test_xpd_increases_with_frequency() {
        let xpd1 = rain_cross_polarization_discrimination_raw(2.0, 10.0, 30.0, 0.01, 45.0);
        let xpd2 = rain_cross_polarization_discrimination_raw(2.0, 30.0, 30.0, 0.01, 45.0);
        assert!(
            xpd2 > xpd1,
            "XPD should increase with freq: {} vs {}",
            xpd1,
            xpd2
        );
    }

    // ── Rain attenuation branch coverage ────────────────────────────

    #[test]
    fn test_rain_attenuation_madrid() {
        let a = rain_attenuation_raw(40.4, -3.7, 14.25, 30.0, 0.01, 45.0, None);
        assert!(a > 0.0 && a < 20.0, "rain attenuation = {a}");
    }

    #[test]
    fn test_rain_attenuation_low_elevation() {
        // el < 5° uses the extended formula
        let a = rain_attenuation_raw(40.4, -3.7, 14.25, 3.0, 0.01, 45.0, None);
        assert!(a > 0.0, "low elevation rain atten = {a}");
    }

    #[test]
    fn test_rain_attenuation_different_probabilities() {
        let a001 = rain_attenuation_raw(40.4, -3.7, 14.25, 30.0, 0.01, 45.0, None);
        let a1 = rain_attenuation_raw(40.4, -3.7, 14.25, 30.0, 1.0, 45.0, None);
        let a5 = rain_attenuation_raw(40.4, -3.7, 14.25, 30.0, 5.0, 45.0, None);
        // Higher probability → lower attenuation
        assert!(a1 < a001, "a1={a1} should be < a001={a001}");
        assert!(a5 < a1, "a5={a5} should be < a1={a1}");
    }

    #[test]
    fn test_rain_attenuation_high_latitude() {
        // |lat| > 36°, beta = 0
        let a = rain_attenuation_raw(60.0, 10.0, 14.25, 30.0, 0.1, 45.0, None);
        assert!(a >= 0.0);
    }

    #[test]
    fn test_rain_attenuation_station_above_rain_height() {
        // Very high station, should be 0
        let a = rain_attenuation_raw(40.4, -3.7, 14.25, 30.0, 0.01, 45.0, Some(10.0));
        assert_eq!(a, 0.0);
    }

    // ── XPD branch coverage ─────────────────────────────────────────

    #[test]
    fn test_xpd_4_to_6_ghz_scaling() {
        // Tests the 4–6 GHz scaling branch
        let xpd = rain_cross_polarization_discrimination_raw(2.0, 5.0, 30.0, 0.01, 45.0);
        assert!(xpd > 0.0, "XPD at 5 GHz = {xpd}");
    }

    #[test]
    fn test_xpd_high_frequency() {
        // 36–55 GHz branch
        let xpd = rain_cross_polarization_discrimination_raw(2.0, 40.0, 30.0, 0.01, 45.0);
        assert!(xpd > 0.0, "XPD at 40 GHz = {xpd}");
    }

    #[test]
    fn test_xpd_canting_angle_branches() {
        let xpd_001 = rain_cross_polarization_discrimination_raw(2.0, 14.0, 30.0, 0.001, 45.0);
        let xpd_01 = rain_cross_polarization_discrimination_raw(2.0, 14.0, 30.0, 0.01, 45.0);
        let xpd_1 = rain_cross_polarization_discrimination_raw(2.0, 14.0, 30.0, 0.1, 45.0);
        let xpd_5 = rain_cross_polarization_discrimination_raw(2.0, 14.0, 30.0, 5.0, 45.0);
        // All should be positive
        assert!(xpd_001 > 0.0);
        assert!(xpd_01 > 0.0);
        assert!(xpd_1 > 0.0);
        assert!(xpd_5 > 0.0);
    }

    // ── Scintillation branch coverage ───────────────────────────────

    #[test]
    fn test_scintillation_large_antenna() {
        // x >= 7.0 branch (g = 0) — need very large antenna
        let s = scintillation_attenuation_raw(14.25, 30.0, 0.01, 100.0, 0.5, Some(50.0), 0.0, 0.0);
        assert_eq!(s, 0.0, "large antenna should have zero scintillation");
    }

    #[test]
    fn test_scintillation_with_n_wet_lookup() {
        let s = scintillation_attenuation_raw(14.25, 30.0, 0.01, 1.2, 0.5, None, 40.4, -3.7);
        assert!(s > 0.0, "scintillation with N_wet lookup = {s}");
    }

    // ── Unitful API tests ───────────────────────────────────────────

    #[test]
    fn test_unitful_rain_attenuation() {
        use lox_core::units::Decibel;
        let a: Decibel = rain_attenuation(
            Angle::degrees(40.4),
            Angle::degrees(-3.7),
            Frequency::gigahertz(14.25),
            Angle::degrees(30.0),
            0.01,
            Angle::degrees(45.0),
            None,
        );
        assert!(a.as_f64() > 0.0);
    }

    #[test]
    fn test_unitful_scintillation_sigma() {
        let sigma = scintillation_attenuation_sigma(
            Frequency::gigahertz(14.25),
            Angle::degrees(30.0),
            Distance::meters(1.2),
            0.5,
            50.0,
        );
        assert!(sigma.as_f64() > 0.0);
    }

    #[test]
    fn test_unitful_scintillation() {
        use lox_core::units::Decibel;
        let s: Decibel = scintillation_attenuation(
            Frequency::gigahertz(14.25),
            Angle::degrees(30.0),
            0.01,
            Distance::meters(1.2),
            0.5,
            None,
            Angle::degrees(40.4),
            Angle::degrees(-3.7),
        );
        assert!(s.as_f64() > 0.0);
    }

    #[test]
    fn test_unitful_xpd() {
        use lox_core::units::Decibel;
        let xpd: Decibel = rain_cross_polarization_discrimination(
            Decibel::new(2.0),
            Frequency::gigahertz(14.0),
            Angle::degrees(30.0),
            0.01,
            Angle::degrees(45.0),
        );
        assert!(xpd.as_f64() > 15.0);
    }
}
