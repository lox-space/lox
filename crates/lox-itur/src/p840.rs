// SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MIT AND MPL-2.0

//! ITU-R P.840-9: Attenuation due to clouds and fog.
//!
//! The dielectric model below is pure formula; the gridded reduced cloud liquid
//! water content, cloud attenuation, and log-normal coefficients are served by
//! [`crate::ItuProvider::columnar_content_reduced_liquid`],
//! [`crate::ItuProvider::cloud_attenuation`], and
//! [`crate::ItuProvider::lognormal_approximation_coefficient`].

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
        use crate::provider::test_fixture::provider;
        use lox_core::units::Angle;
        let p = provider();
        let lred = p
            .columnar_content_reduced_liquid(Angle::degrees(40.4), Angle::degrees(-3.7), 1.0)
            .unwrap();
        assert!(lred >= 0.0, "Lred = {lred}");
    }

    #[test]
    fn test_cloud_attenuation_positive() {
        use crate::provider::test_fixture::provider;
        use lox_core::units::{Angle, Frequency};
        let p = provider();
        let a = p
            .cloud_attenuation(
                Angle::degrees(40.4),
                Angle::degrees(-3.7),
                Angle::degrees(30.0),
                Frequency::gigahertz(14.25),
                1.0,
            )
            .unwrap();
        assert!(a >= 0.0, "cloud attenuation = {a}");
    }

    #[test]
    fn test_lognormal_coefficients() {
        use crate::provider::test_fixture::provider;
        use lox_core::units::Angle;
        let p = provider();
        let c = p
            .lognormal_approximation_coefficient(Angle::degrees(40.4), Angle::degrees(-3.7))
            .unwrap();
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
