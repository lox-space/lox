// SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MIT AND MPL-2.0

//! ITU-R P.838-3: Specific attenuation model for rain.
//!
//! Computes the specific attenuation γ_R (dB/km) from rainfall rate using a power-law
//! relationship: γ_R = k · R^α, where k and α depend on frequency, elevation, and
//! polarisation tilt angle.

use lox_core::units::{Angle, Frequency};

/// Coefficients k and α for rain specific attenuation.
#[derive(Debug, Clone, Copy)]
pub struct RainCoefficients {
    /// Coefficient k (dimensionless).
    pub k: f64,
    /// Coefficient α (dimensionless).
    pub alpha: f64,
}

/// Computes the rain specific attenuation coefficients k and α (P.838-3).
///
/// # Arguments
///
/// * `frequency` — Frequency
/// * `elevation` — Elevation angle
/// * `polarisation_tilt` — Polarisation tilt angle (45° for circular polarisation)
pub fn rain_specific_attenuation_coefficients(
    frequency: Frequency,
    elevation: Angle,
    polarisation_tilt: Angle,
) -> RainCoefficients {
    rain_specific_attenuation_coefficients_raw(
        frequency.to_gigahertz(),
        elevation.to_degrees(),
        polarisation_tilt.to_degrees(),
    )
}

pub(crate) fn rain_specific_attenuation_coefficients_raw(
    f_ghz: f64,
    el_deg: f64,
    tau_deg: f64,
) -> RainCoefficients {
    let log_f = f_ghz.log10();

    // Horizontal polarisation coefficients for k
    const KH_AJ: [f64; 4] = [-5.33980, -0.35351, -0.23789, -0.94158];
    const KH_BJ: [f64; 4] = [-0.10008, 1.2697, 0.86036, 0.64552];
    const KH_CJ: [f64; 4] = [1.13098, 0.454, 0.15354, 0.16817];
    const KH_MK: f64 = -0.18961;
    const KH_CK: f64 = 0.71147;

    // Vertical polarisation coefficients for k
    const KV_AJ: [f64; 4] = [-3.80595, -3.44965, -0.39902, 0.50167];
    const KV_BJ: [f64; 4] = [0.56934, -0.22911, 0.73042, 1.07319];
    const KV_CJ: [f64; 4] = [0.81061, 0.51059, 0.11899, 0.27195];
    const KV_MK: f64 = -0.16398;
    const KV_CK: f64 = 0.63297;

    // Horizontal polarisation coefficients for α
    const AH_AJ: [f64; 5] = [-0.14318, 0.29591, 0.32177, -5.37610, 16.1721];
    const AH_BJ: [f64; 5] = [1.82442, 0.77564, 0.63773, -0.96230, -3.29980];
    const AH_CJ: [f64; 5] = [-0.55187, 0.19822, 0.13164, 1.47828, 3.4399];
    const AH_MA: f64 = 0.67849;
    const AH_CA: f64 = -1.95537;

    // Vertical polarisation coefficients for α
    const AV_AJ: [f64; 5] = [-0.07771, 0.56727, -0.20238, -48.2991, 48.5833];
    const AV_BJ: [f64; 5] = [2.3384, 0.95545, 1.1452, 0.791669, 0.791459];
    const AV_CJ: [f64; 5] = [-0.76284, 0.54039, 0.26809, 0.116226, 0.116479];
    const AV_MA: f64 = -0.053739;
    const AV_CA: f64 = 0.83433;

    fn curve(log_f: f64, a: f64, b: f64, c: f64) -> f64 {
        a * (-((log_f - b) / c).powi(2)).exp()
    }

    // Compute kH, kV
    let sum_kh: f64 = (0..4)
        .map(|j| curve(log_f, KH_AJ[j], KH_BJ[j], KH_CJ[j]))
        .sum();
    let kh = 10.0_f64.powf(sum_kh + KH_MK * log_f + KH_CK);

    let sum_kv: f64 = (0..4)
        .map(|j| curve(log_f, KV_AJ[j], KV_BJ[j], KV_CJ[j]))
        .sum();
    let kv = 10.0_f64.powf(sum_kv + KV_MK * log_f + KV_CK);

    // Compute αH, αV
    let alpha_h: f64 = (0..5)
        .map(|j| curve(log_f, AH_AJ[j], AH_BJ[j], AH_CJ[j]))
        .sum::<f64>()
        + AH_MA * log_f
        + AH_CA;

    let alpha_v: f64 = (0..5)
        .map(|j| curve(log_f, AV_AJ[j], AV_BJ[j], AV_CJ[j]))
        .sum::<f64>()
        + AV_MA * log_f
        + AV_CA;

    // Combine for given elevation and polarisation tilt
    let el_rad = el_deg.to_radians();
    let tau_rad = tau_deg.to_radians();
    let cos2_el = el_rad.cos().powi(2);
    let cos_2tau = (2.0 * tau_rad).cos();

    let k = (kh + kv + (kh - kv) * cos2_el * cos_2tau) / 2.0;
    let alpha = (kh * alpha_h + kv * alpha_v + (kh * alpha_h - kv * alpha_v) * cos2_el * cos_2tau)
        / (2.0 * k);

    RainCoefficients { k, alpha }
}

/// Computes the specific attenuation γ_R (dB/km) from rainfall rate (P.838-3).
///
/// # Arguments
///
/// * `rain_rate` — Rainfall rate R in mm/h
/// * `frequency` — Frequency
/// * `elevation` — Elevation angle
/// * `polarisation_tilt` — Polarisation tilt angle (45° for circular polarisation)
pub fn rain_specific_attenuation(
    rain_rate: f64,
    frequency: Frequency,
    elevation: Angle,
    polarisation_tilt: Angle,
) -> f64 {
    rain_specific_attenuation_raw(
        rain_rate,
        frequency.to_gigahertz(),
        elevation.to_degrees(),
        polarisation_tilt.to_degrees(),
    )
}

pub(crate) fn rain_specific_attenuation_raw(
    rain_rate: f64,
    f_ghz: f64,
    el_deg: f64,
    tau_deg: f64,
) -> f64 {
    let c = rain_specific_attenuation_coefficients_raw(f_ghz, el_deg, tau_deg);
    c.k * rain_rate.powf(c.alpha)
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;

    use super::*;

    #[test]
    fn test_rain_coefficients_14ghz() {
        // Reference: ITU-R P.838-3 at 14.25 GHz, circular polarisation (tau=45°)
        let c = rain_specific_attenuation_coefficients_raw(14.25, 30.0, 45.0);
        // k and alpha should be reasonable for Ku-band
        assert!(c.k > 0.0 && c.k < 1.0, "k={}", c.k);
        assert!(c.alpha > 0.5 && c.alpha < 2.0, "alpha={}", c.alpha);
    }

    #[test]
    fn test_rain_attenuation_increases_with_rate() {
        let a1 = rain_specific_attenuation_raw(5.0, 14.25, 30.0, 45.0);
        let a2 = rain_specific_attenuation_raw(25.0, 14.25, 30.0, 45.0);
        assert!(a2 > a1, "Higher rain rate should give higher attenuation");
    }

    #[test]
    fn test_circular_polarisation_averages_h_v() {
        let c = rain_specific_attenuation_coefficients_raw(10.0, 0.0, 45.0);
        let c_h = rain_specific_attenuation_coefficients_raw(10.0, 0.0, 0.0);
        let c_v = rain_specific_attenuation_coefficients_raw(10.0, 0.0, 90.0);
        let k_avg = (c_h.k + c_v.k) / 2.0;
        assert_approx_eq!(c.k, k_avg, rtol <= 1e-10);
    }

    #[test]
    fn test_unitful_api() {
        let a = rain_specific_attenuation(
            25.0,
            Frequency::gigahertz(14.25),
            Angle::degrees(30.0),
            Angle::degrees(45.0),
        );
        let a_raw = rain_specific_attenuation_raw(25.0, 14.25, 30.0, 45.0);
        assert_approx_eq!(a, a_raw, rtol <= 1e-10);
    }

    #[test]
    fn test_unitful_coefficients() {
        let c = rain_specific_attenuation_coefficients(
            Frequency::gigahertz(14.25),
            Angle::degrees(30.0),
            Angle::degrees(45.0),
        );
        assert!(c.k > 0.0 && c.alpha > 0.0);
    }

    #[test]
    fn test_coefficients_at_various_frequencies() {
        for f in [1.0, 5.0, 10.0, 20.0, 30.0, 50.0] {
            let c = rain_specific_attenuation_coefficients_raw(f, 30.0, 45.0);
            assert!(c.k > 0.0, "k({f} GHz) = {}", c.k);
            assert!(c.alpha > 0.0, "alpha({f} GHz) = {}", c.alpha);
        }
    }
}
