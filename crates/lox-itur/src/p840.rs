// SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MIT AND MPL-2.0

//! ITU-R P.840-9: Attenuation due to clouds and fog.
//!
//! Computes cloud attenuation on Earth-space paths using a double-Debye model
//! for the dielectric permittivity of water and globally-gridded reduced cloud
//! liquid water content data.

use lox_core::units::{Angle, Frequency};

use crate::data::LazyGrid;

/// Available probability levels for P.840-7/8 Lred data.
const PROB_LEVELS: [f64; 18] = [
    0.1, 0.2, 0.3, 0.5, 1.0, 2.0, 3.0, 5.0, 10.0, 20.0, 30.0, 50.0, 60.0, 70.0, 80.0, 90.0, 95.0,
    99.0,
];

static LRED_GRIDS: [LazyGrid; 18] = [
    LazyGrid::new("840/v7_lred_01.bin.zst"),
    LazyGrid::new("840/v7_lred_02.bin.zst"),
    LazyGrid::new("840/v7_lred_03.bin.zst"),
    LazyGrid::new("840/v7_lred_05.bin.zst"),
    LazyGrid::new("840/v7_lred_1.bin.zst"),
    LazyGrid::new("840/v7_lred_2.bin.zst"),
    LazyGrid::new("840/v7_lred_3.bin.zst"),
    LazyGrid::new("840/v7_lred_5.bin.zst"),
    LazyGrid::new("840/v7_lred_10.bin.zst"),
    LazyGrid::new("840/v7_lred_20.bin.zst"),
    LazyGrid::new("840/v7_lred_30.bin.zst"),
    LazyGrid::new("840/v7_lred_50.bin.zst"),
    LazyGrid::new("840/v7_lred_60.bin.zst"),
    LazyGrid::new("840/v7_lred_70.bin.zst"),
    LazyGrid::new("840/v7_lred_80.bin.zst"),
    LazyGrid::new("840/v7_lred_90.bin.zst"),
    LazyGrid::new("840/v7_lred_95.bin.zst"),
    LazyGrid::new("840/v7_lred_99.bin.zst"),
];

static M_GRID: LazyGrid = LazyGrid::new("840/v7_m.bin.zst");
static SIGMA_GRID: LazyGrid = LazyGrid::new("840/v7_sigma.bin.zst");
static PCLW_GRID: LazyGrid = LazyGrid::new("840/v7_pclw.bin.zst");

/// Computes the specific cloud attenuation coefficient K_l ((dB/km)/(g/m³))
/// using the double-Debye dielectric model (P.840-8 Eq. 2–11).
///
/// # Arguments
///
/// * `f_ghz` — Frequency in GHz
/// * `t_celsius` — Temperature in °C (typically 0°C for clouds)
pub fn specific_attenuation_coefficient(f_ghz: f64, t_celsius: f64) -> f64 {
    let theta = 300.0 / (t_celsius + 273.15);

    // Debye permittivity model parameters (P.840-8 Eqs. 6–11)
    let epsilon0 = 77.66 + 103.3 * (theta - 1.0);
    let epsilon1 = 0.0671 * epsilon0;
    let epsilon2 = 3.52;

    let f_p = 20.20 - 146.0 * (theta - 1.0) + 316.0 * (theta - 1.0).powi(2);
    let f_s = 39.8 * f_p;

    let f = f_ghz;

    // Real and imaginary parts of permittivity (Eqs. 4–5)
    let _eps_real = (epsilon0 - epsilon1) / (1.0 + (f / f_p).powi(2))
        + (epsilon1 - epsilon2) / (1.0 + (f / f_s).powi(2))
        + epsilon2;

    let eps_imag = f * (epsilon0 - epsilon1) / (f_p * (1.0 + (f / f_p).powi(2)))
        + f * (epsilon1 - epsilon2) / (f_s * (1.0 + (f / f_s).powi(2)));

    // Eta (Eq. 3)
    let eta = (2.0 + _eps_real) / eps_imag;

    // Specific attenuation coefficient (Eq. 2)
    0.819 * f / (eps_imag * (1.0 + eta.powi(2)))
}

/// Computes the cloud liquid mass absorption coefficient K_L (dB/(kg/m²))
/// per P.840-9 Eq. 12.
///
/// This applies a Gaussian correction to the double-Debye Kl evaluated at T=273.75 K.
pub fn cloud_liquid_mass_absorption_coefficient(f_ghz: f64) -> f64 {
    let t_ref = 273.75 - 273.15; // 0.60 °C
    let kl = specific_attenuation_coefficient(f_ghz, t_ref);

    const A1: f64 = 0.1522;
    const A2: f64 = 11.51;
    const A3: f64 = -10.4912;
    const F1: f64 = -23.9589;
    const F2: f64 = 219.2096;
    const SIGMA1: f64 = 3.2991e3;
    const SIGMA2: f64 = 2.7595e6;

    let correction = A1 * (-(f_ghz - F1).powi(2) / SIGMA1).exp()
        + A2 * (-(f_ghz - F2).powi(2) / SIGMA2).exp()
        + A3;

    kl * correction
}

/// Returns the total columnar content of reduced cloud liquid water (kg/m²)
/// exceeded for `p` % of the average year.
///
/// Uses logarithmic interpolation between available probability levels.
pub fn columnar_content_reduced_liquid(lat: Angle, lon: Angle, p: f64) -> f64 {
    let lat_deg = lat.to_degrees();
    let lon_deg = lon.to_degrees();
    let idx = PROB_LEVELS
        .iter()
        .position(|&pl| pl >= p)
        .unwrap_or(PROB_LEVELS.len() - 1);

    if (PROB_LEVELS[idx] - p).abs() < 1e-10 {
        return LRED_GRIDS[idx].get().bilinear(lat_deg, lon_deg);
    }

    if idx == 0 {
        return LRED_GRIDS[0].get().bilinear(lat_deg, lon_deg);
    }

    let p_below = PROB_LEVELS[idx - 1];
    let p_above = PROB_LEVELS[idx];
    let val_below = LRED_GRIDS[idx - 1].get().bilinear(lat_deg, lon_deg);
    let val_above = LRED_GRIDS[idx].get().bilinear(lat_deg, lon_deg);

    let t = (p.ln() - p_below.ln()) / (p_above.ln() - p_below.ln());
    val_below + (val_above - val_below) * t
}

/// Computes the cloud attenuation (dB) on a slant path (P.840-9).
pub fn cloud_attenuation(
    lat: Angle,
    lon: Angle,
    elevation: Angle,
    frequency: Frequency,
    p: f64,
) -> f64 {
    let lred = columnar_content_reduced_liquid(lat, lon, p);
    let kl = cloud_liquid_mass_absorption_coefficient(frequency.to_gigahertz());
    lred * kl / elevation.to_radians().sin()
}

/// Coefficients for the log-normal approximation of cloud liquid water content.
#[derive(Debug, Clone, Copy)]
pub struct LognormalCoefficients {
    /// Mean of the log-normal distribution.
    pub m: f64,
    /// Standard deviation of the log-normal distribution.
    pub sigma: f64,
    /// Probability of non-zero cloud liquid water content.
    pub pclw: f64,
}

/// Returns the log-normal approximation coefficients for the total columnar
/// content of reduced cloud liquid water (P.840-8).
///
/// # Arguments
///
/// * `lat_deg` — Latitude in degrees
/// * `lon_deg` — Longitude in degrees
pub fn lognormal_approximation_coefficient(lat: Angle, lon: Angle) -> LognormalCoefficients {
    let lat_deg = lat.to_degrees();
    let lon_deg = lon.to_degrees();
    LognormalCoefficients {
        m: M_GRID.get().bilinear(lat_deg, lon_deg),
        sigma: SIGMA_GRID.get().bilinear(lat_deg, lon_deg),
        pclw: PCLW_GRID.get().bilinear(lat_deg, lon_deg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_attenuation_at_10ghz() {
        // K_l at 10 GHz, 0°C should be about 0.05 (dB/km)/(g/m³)
        let kl = specific_attenuation_coefficient(10.0, 0.0);
        assert!(kl > 0.01 && kl < 0.2, "K_l at 10 GHz = {}", kl);
    }

    #[test]
    fn test_cloud_liquid_mass_absorption_coefficient() {
        let kl = cloud_liquid_mass_absorption_coefficient(10.0);
        assert!(kl > 0.0, "KL at 10 GHz = {kl}");
        let kl_30 = cloud_liquid_mass_absorption_coefficient(30.0);
        assert!(kl_30 > kl, "KL should increase with frequency");
    }

    #[test]
    fn test_columnar_content_reduced_liquid() {
        let lred = columnar_content_reduced_liquid(Angle::degrees(40.4), Angle::degrees(-3.7), 1.0);
        assert!(lred >= 0.0, "Lred = {lred}");
    }

    #[test]
    fn test_cloud_attenuation_positive() {
        let a = cloud_attenuation(
            Angle::degrees(40.4),
            Angle::degrees(-3.7),
            Angle::degrees(30.0),
            Frequency::gigahertz(14.25),
            1.0,
        );
        assert!(a >= 0.0, "cloud attenuation = {a}");
    }

    #[test]
    fn test_lognormal_coefficients() {
        let c = lognormal_approximation_coefficient(Angle::degrees(40.4), Angle::degrees(-3.7));
        assert!(c.pclw >= 0.0 && c.pclw <= 100.0, "pclw = {}", c.pclw);
    }

    #[test]
    fn test_specific_attenuation_increases_with_frequency() {
        let kl_10 = specific_attenuation_coefficient(10.0, 0.0);
        let kl_30 = specific_attenuation_coefficient(30.0, 0.0);
        assert!(
            kl_30 > kl_10,
            "K_l should increase with frequency: {} vs {}",
            kl_10,
            kl_30
        );
    }
}
