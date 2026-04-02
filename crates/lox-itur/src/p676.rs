// SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MIT AND MPL-2.0

//! ITU-R P.676-13: Attenuation by atmospheric gases.
//!
//! Computes specific attenuation from oxygen and water vapour absorption using
//! a line-by-line spectroscopic model, plus equivalent-height approximation
//! for slant-path attenuation.

use lox_core::units::{Angle, Decibel, Distance, Frequency, Pressure, Temperature};

// ── Oxygen spectral line data (P.676-12 Table 1) ───────────────────────────
// Columns: f0 (GHz), a1, a2, a3, a4, a5, a6
#[rustfmt::skip]
const OXYGEN_LINES: &[(f64, f64, f64, f64, f64, f64, f64)] = &[
    (50.474214, 0.975, 9.651, 6.690, 0.0, 2.566, 6.850),
    (50.987745, 2.529, 8.653, 7.170, 0.0, 2.246, 6.800),
    (51.503360, 6.193, 7.709, 7.640, 0.0, 1.947, 6.729),
    (52.021429, 14.320, 6.819, 8.110, 0.0, 1.667, 6.640),
    (52.542418, 31.240, 5.983, 8.580, 0.0, 1.388, 6.526),
    (53.066934, 64.290, 5.201, 9.060, 0.0, 1.349, 6.206),
    (53.595775, 124.600, 4.474, 9.550, 0.0, 2.227, 5.085),
    (54.130025, 227.300, 3.800, 9.960, 0.0, 3.170, 3.750),
    (54.671180, 389.700, 3.182, 10.370, 0.0, 3.558, 2.654),
    (55.221384, 627.100, 2.618, 10.890, 0.0, 2.560, 2.952),
    (55.783815, 945.300, 2.109, 11.340, 0.0, -1.172, 6.135),
    (56.264774, 543.400, 0.014, 17.030, 0.0, 3.525, -0.978),
    (56.363399, 1331.800, 1.654, 11.890, 0.0, -3.595, 6.547),
    (56.968211, 1746.600, 1.255, 12.230, 0.0, -5.416, 6.451),
    (57.612486, 2120.100, 0.910, 12.620, 0.0, -1.932, 2.654),
    (58.323877, 2363.700, 0.621, 12.950, 0.0, 1.730, -0.151),
    (58.446588, 1442.100, 0.083, 14.910, 0.0, -0.240, 3.434),
    (59.164204, 2379.900, 0.387, 13.530, 0.0, 0.158, -2.273),
    (59.590983, 2090.700, 0.207, 14.080, 0.0, 2.691, -4.529),
    (60.306056, 2103.400, 0.207, 14.150, 0.0, -4.562, 5.050),
    (60.434778, 2438.000, 0.386, 13.390, 0.0, 0.028, -2.406),
    (61.150562, 2479.500, 0.621, 12.920, 0.0, 1.202, 0.776),
    (61.800158, 2275.900, 0.910, 12.630, 0.0, 0.756, 3.385),
    (62.411220, 1915.400, 1.255, 12.170, 0.0, 5.798, -3.693),
    (62.486253, 1503.000, 0.083, 15.130, 0.0, -0.079, 0.714),
    (62.997984, 1490.200, 1.654, 11.740, 0.0, 3.693, -2.825),
    (63.568526, 1078.000, 2.108, 11.340, 0.0, 1.030, -3.039),
    (64.127775, 728.700, 2.617, 10.880, 0.0, -4.271, 3.784),
    (64.678910, 461.300, 3.181, 10.380, 0.0, -3.362, 1.462),
    (65.224078, 274.000, 3.800, 9.960, 0.0, -1.982, -1.150),
    (65.764779, 153.000, 4.473, 9.550, 0.0, 0.781, -2.310),
    (66.302096, 80.400, 5.200, 9.060, 0.0, -1.391, 0.264),
    (66.836834, 39.800, 5.982, 8.580, 0.0, -3.536, 0.894),
    (67.369601, 18.560, 6.818, 8.110, 0.0, -3.230, -0.042),
    (67.900868, 8.172, 7.708, 7.640, 0.0, -1.396, 1.174),
    (68.431006, 3.397, 8.652, 7.170, 0.0, -1.142, 1.382),
    (68.960312, 1.334, 9.650, 6.690, 0.0, 0.457, -0.697),
    (118.750334, 940.300, 0.010, 16.640, 0.0, -0.184, 6.150),
    (368.498246, 67.400, 0.048, 16.400, 0.0, 0.0, 0.0),
    (424.763020, 637.700, 0.044, 16.400, 0.0, 0.0, 0.0),
    (487.249273, 237.400, 0.049, 16.000, 0.0, 0.0, 0.0),
    (715.392902, 98.100, 0.145, 16.000, 0.0, 0.0, 0.0),
    (773.839490, 572.300, 0.141, 16.000, 0.0, 0.0, 0.0),
    (834.145546, 183.100, 0.145, 16.000, 0.0, 0.0, 0.0),
];

// ── Water vapour spectral line data (P.676-12 Table 2) ─────────────────────
#[rustfmt::skip]
const WATER_VAPOUR_LINES: &[(f64, f64, f64, f64, f64, f64, f64)] = &[
    (22.235080, 0.1079, 2.144, 26.38, 0.76, 5.087, 1.0),
    (67.803960, 0.0011, 8.732, 28.58, 0.69, 4.930, 0.82),
    (119.995940, 0.0007, 8.353, 29.48, 0.70, 4.780, 0.79),
    (183.310087, 2.273, 0.668, 29.06, 0.77, 5.022, 0.85),
    (321.225630, 0.0470, 6.179, 24.04, 0.67, 4.398, 0.54),
    (325.152888, 1.514, 1.541, 28.23, 0.64, 4.893, 0.78),
    (336.227764, 0.0010, 9.825, 26.93, 0.69, 4.740, 0.78),
    (380.197353, 11.67, 1.048, 28.11, 0.54, 5.063, 0.89),
    (390.134508, 0.0045, 7.347, 21.52, 0.63, 4.810, 0.53),
    (437.346667, 0.0632, 5.048, 18.45, 0.60, 4.230, 0.36),
    (439.150807, 0.9098, 3.595, 20.07, 0.63, 4.483, 0.46),
    (443.018343, 0.1920, 5.048, 15.55, 0.60, 5.083, 0.17),
    (448.001085, 10.41, 1.405, 25.64, 0.66, 5.028, 0.83),
    (470.888999, 0.3254, 3.597, 21.34, 0.66, 4.506, 0.45),
    (474.689092, 1.260, 2.379, 23.20, 0.65, 4.804, 0.57),
    (488.490108, 0.2529, 2.852, 25.86, 0.69, 5.201, 0.67),
    (503.568532, 0.0372, 6.731, 16.12, 0.61, 3.980, 0.26),
    (504.482692, 0.0124, 6.731, 16.12, 0.61, 4.010, 0.26),
    (547.676440, 0.9785, 0.158, 26.00, 0.70, 4.500, 1.0),
    (552.020960, 0.1840, 0.158, 26.00, 0.70, 4.500, 1.0),
    (556.935985, 497.0, 0.159, 30.86, 0.69, 4.552, 1.0),
    (620.700807, 5.015, 2.391, 24.38, 0.71, 4.856, 0.68),
    (645.766085, 0.0067, 8.633, 18.00, 0.60, 4.000, 0.50),
    (658.005280, 0.2732, 7.816, 32.10, 0.69, 4.140, 1.0),
    (752.033113, 243.4, 0.396, 30.86, 0.68, 4.352, 0.84),
    (841.051732, 0.0134, 8.177, 15.90, 0.33, 5.760, 0.45),
    (859.965698, 0.1325, 8.055, 30.60, 0.68, 4.090, 0.84),
    (899.303175, 0.0547, 7.914, 29.85, 0.68, 4.530, 0.90),
    (902.611085, 0.0386, 8.429, 28.65, 0.70, 5.100, 0.95),
    (906.205957, 0.1836, 5.110, 24.08, 0.70, 4.700, 0.53),
    (916.171582, 8.400, 1.441, 26.73, 0.70, 5.150, 0.78),
    (923.112692, 0.0079, 10.293, 29.00, 0.70, 5.000, 0.80),
    (970.315022, 9.009, 1.919, 25.50, 0.64, 4.940, 0.67),
    (987.926764, 134.6, 0.257, 29.85, 0.68, 4.550, 0.90),
    (1780.000000, 17506.0, 0.952, 196.30, 2.00, 24.150, 5.0),
];

/// Computes the specific attenuation from dry air (oxygen) γ₀ (dB/km).
///
/// # Arguments
///
/// * `frequency` — Frequency
/// * `pressure` — Dry air pressure
/// * `rho` — Water vapour density in g/m³
/// * `temperature` — Temperature
pub fn gamma0(frequency: Frequency, pressure: Pressure, rho: f64, temperature: Temperature) -> f64 {
    gamma0_raw(
        frequency.to_gigahertz(),
        pressure.to_hpa(),
        rho,
        temperature.to_kelvin(),
    )
}

pub(crate) fn gamma0_raw(f_ghz: f64, p_hpa: f64, rho: f64, t_k: f64) -> f64 {
    let theta = 300.0 / t_k;
    let e = rho * t_k / 216.7; // water vapour partial pressure (hPa)
    let f = f_ghz;

    let mut n_pp_sum = 0.0;

    for &(f0, a1, a2, a3, a4, a5, a6) in OXYGEN_LINES {
        // Line intensity
        let s_i = a1 * 1e-7 * p_hpa * theta.powi(3) * (a2 * (1.0 - theta)).exp();

        // Line broadening
        let df = a3 * 1e-4 * (p_hpa * theta.powf(0.8 - a4) + 1.1 * e * theta);
        let df = (df * df + 2.25e-6).sqrt();

        // Line shift
        let delta = (a5 + a6 * theta) * 1e-4 * (p_hpa + e) * theta.powf(0.8);

        // Line shape
        let f_shape = (f / f0)
            * ((df - delta * (f0 - f)) / ((f0 - f).powi(2) + df * df)
                + (df - delta * (f0 + f)) / ((f0 + f).powi(2) + df * df));

        n_pp_sum += s_i * f_shape;
    }

    // Dry air continuum
    let d = 5.6e-4 * (p_hpa + e) * theta.powf(0.8);
    let n_d_pp = f
        * p_hpa
        * theta
        * theta
        * (6.14e-5 / (d * (1.0 + (f / d).powi(2)))
            + 1.4e-12 * p_hpa * theta.powf(1.5) / (1.0 + 1.9e-5 * f.powf(1.5)));

    0.1820 * f * (n_pp_sum + n_d_pp)
}

/// Computes the specific attenuation from water vapour γ_w (dB/km).
///
/// # Arguments
///
/// * `frequency` — Frequency
/// * `pressure` — Dry air pressure
/// * `rho` — Water vapour density in g/m³
/// * `temperature` — Temperature
pub fn gammaw(frequency: Frequency, pressure: Pressure, rho: f64, temperature: Temperature) -> f64 {
    gammaw_raw(
        frequency.to_gigahertz(),
        pressure.to_hpa(),
        rho,
        temperature.to_kelvin(),
    )
}

pub(crate) fn gammaw_raw(f_ghz: f64, p_hpa: f64, rho: f64, t_k: f64) -> f64 {
    let theta = 300.0 / t_k;
    let e = rho * t_k / 216.7;
    let f = f_ghz;

    let mut n_pp_sum = 0.0;

    for &(f0, b1, b2, b3, b4, b5, b6) in WATER_VAPOUR_LINES {
        // Line intensity
        let s_i = b1 * 1e-1 * e * theta.powf(3.5) * (b2 * (1.0 - theta)).exp();

        // Line broadening
        let df_raw = b3 * 1e-4 * (p_hpa * theta.powf(b4) + b5 * e * theta.powf(b6));
        let df = 0.535 * df_raw + (0.217 * df_raw * df_raw + 2.1316e-12 * f0 * f0 / theta).sqrt();

        // Line shape (no shift for water vapour lines)
        let f_shape =
            (f / f0) * (df / ((f0 - f).powi(2) + df * df) + df / ((f0 + f).powi(2) + df * df));

        n_pp_sum += s_i * f_shape;
    }

    0.1820 * f * n_pp_sum
}

/// Computes the equivalent heights for dry air (h₀) and water vapour (h_w).
///
/// These are used in the approximate slant-path method.
///
/// # Arguments
///
/// * `frequency` — Frequency
/// * `pressure` — Pressure
/// * `rho` — Water vapour density in g/m³
/// * `temperature` — Temperature
pub fn equivalent_heights(
    frequency: Frequency,
    pressure: Pressure,
    rho: f64,
    temperature: Temperature,
) -> (Distance, Distance) {
    let (h0, hw) = equivalent_heights_raw(
        frequency.to_gigahertz(),
        pressure.to_hpa(),
        rho,
        temperature.to_kelvin(),
    );
    (Distance::kilometers(h0), Distance::kilometers(hw))
}

// P.676-13 Annex 2: oxygen equivalent-height coefficients (700 frequencies, 1–350 GHz).
// Columns: (f_ghz, a0, b0, c0, d0) for Eq. 31: h0 = a0 + b0*T + c0*Ps + d0*rho
include!("p676_h0_coefficients.rs");

// P.676-13 Table 4: water-vapour equivalent-height coefficients (Eq. 37)
const HW_A: f64 = 5.6585e-5; // km/GHz
const HW_B: f64 = 1.8348; // km
const HW_LINE_COEFFS: [(f64, f64, f64); 3] = [
    (22.235080, 2.6846, 2.7649),
    (183.310087, 5.8905, 4.9219),
    (325.152888, 2.9810, 3.0748),
];

/// Computes the equivalent heights for dry air (h₀) and water vapour (h_w)
/// per P.676-13 Annex 2.
///
/// h₀ is computed from a coefficient table (Eq. 31) via linear interpolation.
/// h_w uses the analytical formula (Eq. 37).
pub(crate) fn equivalent_heights_raw(f_ghz: f64, p_hpa: f64, rho: f64, t_k: f64) -> (f64, f64) {
    // Oxygen equivalent height h0 (Eq. 31)
    // Ps = total surface pressure = P(dry) + e(water vapour)
    let e = rho * t_k / 216.7;
    let ps = p_hpa + e;

    // Linear interpolation in the coefficient table
    let a0 = interp_h0_coeff(f_ghz, |c| c.1);
    let b0 = interp_h0_coeff(f_ghz, |c| c.2);
    let c0 = interp_h0_coeff(f_ghz, |c| c.3);
    let d0 = interp_h0_coeff(f_ghz, |c| c.4);
    let h0 = a0 + b0 * t_k + c0 * ps + d0 * rho;

    // Water vapour equivalent height hw (Eq. 37)
    let hw = HW_A * f_ghz
        + HW_B
        + HW_LINE_COEFFS
            .iter()
            .map(|&(fi, ai, bi)| ai / ((f_ghz - fi).powi(2) + bi))
            .sum::<f64>();

    (h0, hw)
}

/// Linear interpolation in the h0 coefficient table.
fn interp_h0_coeff(f_ghz: f64, select: impl Fn(&(f64, f64, f64, f64, f64)) -> f64) -> f64 {
    let n = H0_COEFFICIENTS.len();
    if n == 0 {
        return 0.0;
    }
    let f0 = H0_COEFFICIENTS[0].0;
    let f_last = H0_COEFFICIENTS[n - 1].0;
    if f_ghz <= f0 {
        return select(&H0_COEFFICIENTS[0]);
    }
    if f_ghz >= f_last {
        return select(&H0_COEFFICIENTS[n - 1]);
    }
    // Uniform spacing: 0.5 GHz
    let step = 0.5;
    let idx = ((f_ghz - f0) / step) as usize;
    let idx = idx.min(n - 2);
    let f_lo = H0_COEFFICIENTS[idx].0;
    let t = (f_ghz - f_lo) / step;
    let v_lo = select(&H0_COEFFICIENTS[idx]);
    let v_hi = select(&H0_COEFFICIENTS[idx + 1]);
    v_lo + t * (v_hi - v_lo)
}

/// Computes the gaseous attenuation on a slant path using the approximate method.
///
/// Returns `(A_oxygen, A_water_vapour)`.
///
/// # Arguments
///
/// * `frequency` — Frequency
/// * `elevation` — Elevation angle
/// * `pressure` — Surface pressure
/// * `rho` — Surface water vapour density in g/m³
/// * `temperature` — Surface temperature
pub fn gaseous_attenuation_slant_path(
    frequency: Frequency,
    elevation: Angle,
    pressure: Pressure,
    rho: f64,
    temperature: Temperature,
) -> (Decibel, Decibel) {
    let (a_o, a_w) = gaseous_attenuation_slant_path_raw(
        frequency.to_gigahertz(),
        elevation.to_degrees().max(5.0),
        pressure.to_hpa(),
        rho,
        temperature.to_kelvin(),
    );
    (Decibel::new(a_o), Decibel::new(a_w))
}

pub(crate) fn gaseous_attenuation_slant_path_raw(
    f_ghz: f64,
    el_deg: f64,
    p_hpa: f64,
    rho: f64,
    t_k: f64,
) -> (f64, f64) {
    let g0 = gamma0_raw(f_ghz, p_hpa, rho, t_k);
    let gw = gammaw_raw(f_ghz, p_hpa, rho, t_k);
    let (h0, hw) = equivalent_heights_raw(f_ghz, p_hpa, rho, t_k);

    let sin_el = el_deg.to_radians().sin();

    let a_o = g0 * h0 / sin_el;
    let a_w = gw * hw / sin_el;

    (a_o, a_w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamma0_sea_level() {
        // At 60 GHz (O2 absorption peak), attenuation should be very high
        let g = gamma0_raw(60.0, 1013.25, 7.5, 288.15);
        assert!(g > 10.0, "gamma0 at 60 GHz should be >10 dB/km, got {}", g);
    }

    #[test]
    fn test_gammaw_22ghz() {
        // 22.235 GHz is the main water vapour line
        let gw = gammaw_raw(22.235, 1013.25, 7.5, 288.15);
        assert!(
            gw > 0.05,
            "gammaw at 22.235 GHz should be significant, got {}",
            gw
        );
    }

    #[test]
    fn test_gamma0_low_at_window_freq() {
        // At 10 GHz (atmospheric window), oxygen attenuation should be low
        let g = gamma0_raw(10.0, 1013.25, 7.5, 288.15);
        assert!(g < 0.1, "gamma0 at 10 GHz should be small, got {}", g);
    }

    #[test]
    fn test_equivalent_heights_reasonable() {
        let (h0, hw) = equivalent_heights_raw(14.25, 1013.25, 7.5, 288.15);
        // h0 should be around 5-6 km
        assert!(h0 > 3.0 && h0 < 10.0, "h0 = {}", h0);
        // hw should be around 1-3 km
        assert!(hw > 0.5 && hw < 5.0, "hw = {}", hw);
    }

    #[test]
    fn test_slant_path_attenuation() {
        let (a_o, a_w) = gaseous_attenuation_slant_path_raw(14.25, 30.0, 1013.25, 7.5, 288.15);
        let total = a_o + a_w;
        assert!(total > 0.01 && total < 2.0, "total gaseous = {} dB", total);
    }

    #[test]
    fn test_equivalent_heights_edge_frequencies() {
        // Below table range
        let (h0, hw) = equivalent_heights_raw(0.5, 1013.25, 7.5, 288.15);
        assert!(h0 != 0.0, "h0 at 0.5 GHz = {h0}"); // just not NaN
        assert!(hw > 0.0, "hw at 0.5 GHz = {hw}");

        // At 60 GHz (O2 absorption peak)
        let (h0_60, _) = equivalent_heights_raw(60.0, 1013.25, 7.5, 288.15);
        assert!(h0_60.is_finite(), "h0 at 60 GHz should be finite");

        // Above table range
        let (h0_high, hw_high) = equivalent_heights_raw(400.0, 1013.25, 7.5, 288.15);
        assert!(h0_high.is_finite());
        assert!(hw_high.is_finite());
    }

    #[test]
    fn test_gamma0_at_various_pressures() {
        let g_low = gamma0_raw(14.25, 500.0, 3.0, 260.0);
        let g_high = gamma0_raw(14.25, 1013.25, 7.5, 288.15);
        assert!(g_low.is_finite());
        assert!(g_high.is_finite());
        // Higher pressure → higher attenuation (at same frequency)
        assert!(g_high.abs() > g_low.abs());
    }

    #[test]
    fn test_gammaw_at_various_humidity() {
        let gw_dry = gammaw_raw(22.235, 1013.25, 1.0, 288.15);
        let gw_wet = gammaw_raw(22.235, 1013.25, 20.0, 288.15);
        assert!(gw_wet > gw_dry, "more humidity → more attenuation");
    }

    #[test]
    fn test_unitful_gamma0() {
        let g = gamma0(
            Frequency::gigahertz(60.0),
            Pressure::hpa(1013.25),
            7.5,
            Temperature::kelvin(288.15),
        );
        assert!(g > 10.0);
    }

    #[test]
    fn test_unitful_gammaw() {
        let g = gammaw(
            Frequency::gigahertz(22.235),
            Pressure::hpa(1013.25),
            7.5,
            Temperature::kelvin(288.15),
        );
        assert!(g > 0.05);
    }

    #[test]
    fn test_unitful_equivalent_heights() {
        let (h0, hw) = equivalent_heights(
            Frequency::gigahertz(14.25),
            Pressure::hpa(1013.25),
            7.5,
            Temperature::kelvin(288.15),
        );
        assert!(h0.to_kilometers() > 1.0);
        assert!(hw.to_kilometers() > 0.5);
    }

    #[test]
    fn test_unitful_gaseous_attenuation_slant_path() {
        let (a_o, a_w) = gaseous_attenuation_slant_path(
            Frequency::gigahertz(14.25),
            Angle::degrees(30.0),
            Pressure::hpa(1013.25),
            7.5,
            Temperature::kelvin(288.15),
        );
        assert!(a_o.as_f64() + a_w.as_f64() > 0.0);
    }

    #[test]
    fn test_interp_h0_coeff_clamping() {
        // Test below and above table range
        let v_below = interp_h0_coeff(0.1, |c| c.1);
        let v_first = interp_h0_coeff(1.0, |c| c.1);
        assert_eq!(v_below, v_first);

        let v_above = interp_h0_coeff(500.0, |c| c.1);
        let v_last = interp_h0_coeff(350.0, |c| c.1);
        assert_eq!(v_above, v_last);
    }
}
