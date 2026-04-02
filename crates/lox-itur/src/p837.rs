// SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MIT AND MPL-2.0

//! ITU-R P.837-7: Characteristics of precipitation for propagation modelling.
//!
//! Provides the rainfall rate (mm/h) and rainfall probability exceeded for a
//! given percentage of the average year at any location on Earth.

use std::f64::consts::FRAC_1_SQRT_2;

use lox_core::units::Angle;

use crate::data::LazyGrid;
use crate::p1510;

static R001: LazyGrid = LazyGrid::new("837/v7_r001.bin.zst");

static MT_MONTHS: [LazyGrid; 12] = [
    LazyGrid::new("837/v7_mt_month01.bin.zst"),
    LazyGrid::new("837/v7_mt_month02.bin.zst"),
    LazyGrid::new("837/v7_mt_month03.bin.zst"),
    LazyGrid::new("837/v7_mt_month04.bin.zst"),
    LazyGrid::new("837/v7_mt_month05.bin.zst"),
    LazyGrid::new("837/v7_mt_month06.bin.zst"),
    LazyGrid::new("837/v7_mt_month07.bin.zst"),
    LazyGrid::new("837/v7_mt_month08.bin.zst"),
    LazyGrid::new("837/v7_mt_month09.bin.zst"),
    LazyGrid::new("837/v7_mt_month10.bin.zst"),
    LazyGrid::new("837/v7_mt_month11.bin.zst"),
    LazyGrid::new("837/v7_mt_month12.bin.zst"),
];

const DAYS_PER_MONTH: [f64; 12] = [
    31.0, 28.25, 31.0, 30.0, 31.0, 30.0, 31.0, 31.0, 30.0, 31.0, 30.0, 31.0,
];

/// Returns the rainfall rate (mm/h) exceeded for 0.01% of the average year.
pub fn rainfall_rate_r001(lat: Angle, lon: Angle) -> f64 {
    R001.get().bilinear(lat.to_degrees(), lon.to_degrees())
}

fn monthly_rain_params(lat: Angle, lon: Angle) -> ([f64; 12], [f64; 12]) {
    let lat_deg = lat.to_degrees();
    let lon_deg = lon.to_degrees();
    let mut r = [0.0_f64; 12];
    let mut p0 = [0.0_f64; 12];

    for month in 0..12 {
        let mt = MT_MONTHS[month].get().bilinear(lat_deg, lon_deg);
        let t_k = p1510::surface_month_mean_temperature(lat, lon, (month + 1) as u8).to_kelvin();
        let t_c = t_k - 273.15;

        let r_i = if t_c >= 0.0 {
            0.5874 * (0.0883 * t_c).exp()
        } else {
            0.5874
        };

        let n = DAYS_PER_MONTH[month];
        let mut p0_i = if r_i > 0.0 {
            100.0 * mt / (24.0 * n * r_i)
        } else {
            0.0
        };

        let r_i = if p0_i > 70.0 {
            p0_i = 70.0;
            100.0 * mt / (24.0 * n * 70.0)
        } else {
            r_i
        };

        r[month] = r_i;
        p0[month] = p0_i;
    }

    (r, p0)
}

/// Returns the annual probability of rain (%) at the given location.
pub fn rainfall_probability(lat: Angle, lon: Angle) -> f64 {
    let (_r, p0) = monthly_rain_params(lat, lon);
    let mut total = 0.0;
    for month in 0..12 {
        total += DAYS_PER_MONTH[month] * p0[month];
    }
    total / 365.25
}

/// Returns the rainfall rate (mm/h) exceeded for `p` % of the average year.
pub fn rainfall_rate(lat: Angle, lon: Angle, p: f64) -> f64 {
    if (p - 0.01).abs() < 1e-10 {
        return rainfall_rate_r001(lat, lon);
    }

    let (r, p0) = monthly_rain_params(lat, lon);
    bisect_rainfall_rate(&r, &p0, p)
}

fn q_function(x: f64) -> f64 {
    0.5 * erfc(x * FRAC_1_SQRT_2)
}

fn erfc(x: f64) -> f64 {
    libm::erfc(x)
}

fn bisect_rainfall_rate(r: &[f64; 12], p0: &[f64; 12], p_target: f64) -> f64 {
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

    #[test]
    fn test_rainfall_rate_r001_madrid() {
        let r = rainfall_rate_r001(Angle::degrees(40.4), Angle::degrees(-3.7));
        assert!(r > 5.0 && r < 80.0, "R001 Madrid = {r}");
    }

    #[test]
    fn test_rainfall_rate_at_001_equals_r001() {
        let lat = Angle::degrees(40.4);
        let lon = Angle::degrees(-3.7);
        let r001 = rainfall_rate_r001(lat, lon);
        let r = rainfall_rate(lat, lon, 0.01);
        assert!((r - r001).abs() < 1e-6, "r001={r001}, rate(0.01)={r}");
    }

    #[test]
    fn test_rainfall_rate_other_probability() {
        let r = rainfall_rate(Angle::degrees(40.4), Angle::degrees(-3.7), 0.1);
        assert!(r > 0.0, "rainfall rate at 0.1% should be > 0");
    }

    #[test]
    fn test_rainfall_probability() {
        let p = rainfall_probability(Angle::degrees(40.4), Angle::degrees(-3.7));
        assert!(p > 0.0 && p < 100.0, "P0 = {p}%");
    }

    #[test]
    fn test_q_function() {
        // Q(0) = 0.5
        assert!((q_function(0.0) - 0.5).abs() < 1e-10);
        // Q(large) ≈ 0
        assert!(q_function(10.0) < 1e-10);
    }
}
