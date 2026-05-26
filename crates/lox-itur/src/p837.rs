// SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MIT AND MPL-2.0

//! ITU-R P.837-7: Characteristics of precipitation for propagation modelling.
//!
//! Rainfall rate (mm/h) and rainfall probability are grid-based; they are served
//! by [`crate::ItuProvider::rainfall_rate_r001`],
//! [`crate::ItuProvider::rainfall_probability`], and
//! [`crate::ItuProvider::rainfall_rate`]. The grid-free cores below are shared
//! between the provider methods and the unit tests.

use std::f64::consts::FRAC_1_SQRT_2;

const DAYS_PER_MONTH: [f64; 12] = [
    31.0, 28.25, 31.0, 30.0, 31.0, 30.0, 31.0, 31.0, 30.0, 31.0, 30.0, 31.0,
];

/// Per-month rain rate `r_i` (mm/h) and probability-of-rain `p0_i` (%).
///
/// Pure — takes pre-fetched monthly MT [mm/month] and surface temperature [K].
pub(crate) fn monthly_rain_params_from(mt: &[f64; 12], t_k: &[f64; 12]) -> ([f64; 12], [f64; 12]) {
    let mut r = [0.0_f64; 12];
    let mut p0 = [0.0_f64; 12];

    for month in 0..12 {
        let t_c = t_k[month] - 273.15;

        let r_i = if t_c >= 0.0 {
            0.5874 * (0.0883 * t_c).exp()
        } else {
            0.5874
        };

        let n = DAYS_PER_MONTH[month];
        let mut p0_i = if r_i > 0.0 {
            100.0 * mt[month] / (24.0 * n * r_i)
        } else {
            0.0
        };

        let r_i = if p0_i > 70.0 {
            p0_i = 70.0;
            100.0 * mt[month] / (24.0 * n * 70.0)
        } else {
            r_i
        };

        r[month] = r_i;
        p0[month] = p0_i;
    }

    (r, p0)
}

/// Annual rainfall probability [%] from per-month p0 values. Pure.
pub(crate) fn rainfall_probability_from(p0: &[f64; 12]) -> f64 {
    let mut total = 0.0;
    for month in 0..12 {
        total += DAYS_PER_MONTH[month] * p0[month];
    }
    total / 365.25
}

fn q_function(x: f64) -> f64 {
    0.5 * erfc(x * FRAC_1_SQRT_2)
}

fn erfc(x: f64) -> f64 {
    libm::erfc(x)
}

pub(crate) fn bisect_rainfall_rate(r: &[f64; 12], p0: &[f64; 12], p_target: f64) -> f64 {
    let mut lo = 1e-10_f64;
    let mut hi = 1000.0_f64;

    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        let p_computed = annual_exceedance(r, p0, mid);
        let residual = p_computed / p_target - 1.0;

        if residual.abs() < 1e-5 {
            return mid;
        }

        if residual > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    (lo + hi) / 2.0
}

fn annual_exceedance(r: &[f64; 12], p0: &[f64; 12], r_ref: f64) -> f64 {
    let mut total = 0.0;
    for month in 0..12 {
        if p0[month] > 0.0 && r[month] > 0.0 {
            let arg = (r_ref.ln() + 0.7938 - r[month].ln()) / 1.26;
            let p_month = p0[month] * q_function(arg);
            total += DAYS_PER_MONTH[month] * p_month;
        }
    }
    total / 365.25
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::test_fixture::provider;
    use lox_core::units::Angle;

    #[test]
    fn test_rainfall_rate_r001_madrid() {
        let p = provider();
        let r = p
            .rainfall_rate_r001(Angle::degrees(40.4), Angle::degrees(-3.7))
            .unwrap();
        assert!(r > 5.0 && r < 80.0, "R001 Madrid = {r}");
    }

    #[test]
    fn test_rainfall_rate_at_001_equals_r001() {
        let p = provider();
        let lat = Angle::degrees(40.4);
        let lon = Angle::degrees(-3.7);
        let r001 = p.rainfall_rate_r001(lat, lon).unwrap();
        let r = p.rainfall_rate(lat, lon, 0.01).unwrap();
        assert!((r - r001).abs() < 1e-6, "r001={r001}, rate(0.01)={r}");
    }

    #[test]
    fn test_rainfall_rate_other_probability() {
        let p = provider();
        let r = p
            .rainfall_rate(Angle::degrees(40.4), Angle::degrees(-3.7), 0.1)
            .unwrap();
        assert!(r > 0.0, "rainfall rate at 0.1% should be > 0");
    }

    #[test]
    fn test_rainfall_probability() {
        let p = provider();
        let prob = p
            .rainfall_probability(Angle::degrees(40.4), Angle::degrees(-3.7))
            .unwrap();
        assert!(prob > 0.0 && prob < 100.0, "P0 = {prob}%");
    }

    #[test]
    fn test_q_function() {
        // Q(0) = 0.5
        assert!((q_function(0.0) - 0.5).abs() < 1e-10);
        // Q(large) ≈ 0
        assert!(q_function(10.0) < 1e-10);
    }
}
