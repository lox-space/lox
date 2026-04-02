// SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MIT AND MPL-2.0

//! ITU-R P.836-6: Water vapour: surface density and total columnar content.
//!
//! Provides surface water vapour density (g/m³) and total columnar water vapour
//! content (kg/m²) exceeded for a given percentage of the average year.

use lox_core::units::Angle;

use crate::data::LazyGrid;

/// Available probability levels (% of average year) for P.836-6 data.
const PROB_LEVELS: [f64; 18] = [
    0.1, 0.2, 0.3, 0.5, 1.0, 2.0, 3.0, 5.0, 10.0, 20.0, 30.0, 50.0, 60.0, 70.0, 80.0, 90.0, 95.0,
    99.0,
];

static RHO_GRIDS: [LazyGrid; 18] = [
    LazyGrid::new("836/v6_rho_01.bin.zst"),
    LazyGrid::new("836/v6_rho_02.bin.zst"),
    LazyGrid::new("836/v6_rho_03.bin.zst"),
    LazyGrid::new("836/v6_rho_05.bin.zst"),
    LazyGrid::new("836/v6_rho_1.bin.zst"),
    LazyGrid::new("836/v6_rho_2.bin.zst"),
    LazyGrid::new("836/v6_rho_3.bin.zst"),
    LazyGrid::new("836/v6_rho_5.bin.zst"),
    LazyGrid::new("836/v6_rho_10.bin.zst"),
    LazyGrid::new("836/v6_rho_20.bin.zst"),
    LazyGrid::new("836/v6_rho_30.bin.zst"),
    LazyGrid::new("836/v6_rho_50.bin.zst"),
    LazyGrid::new("836/v6_rho_60.bin.zst"),
    LazyGrid::new("836/v6_rho_70.bin.zst"),
    LazyGrid::new("836/v6_rho_80.bin.zst"),
    LazyGrid::new("836/v6_rho_90.bin.zst"),
    LazyGrid::new("836/v6_rho_95.bin.zst"),
    LazyGrid::new("836/v6_rho_99.bin.zst"),
];

static V_GRIDS: [LazyGrid; 18] = [
    LazyGrid::new("836/v6_v_01.bin.zst"),
    LazyGrid::new("836/v6_v_02.bin.zst"),
    LazyGrid::new("836/v6_v_03.bin.zst"),
    LazyGrid::new("836/v6_v_05.bin.zst"),
    LazyGrid::new("836/v6_v_1.bin.zst"),
    LazyGrid::new("836/v6_v_2.bin.zst"),
    LazyGrid::new("836/v6_v_3.bin.zst"),
    LazyGrid::new("836/v6_v_5.bin.zst"),
    LazyGrid::new("836/v6_v_10.bin.zst"),
    LazyGrid::new("836/v6_v_20.bin.zst"),
    LazyGrid::new("836/v6_v_30.bin.zst"),
    LazyGrid::new("836/v6_v_50.bin.zst"),
    LazyGrid::new("836/v6_v_60.bin.zst"),
    LazyGrid::new("836/v6_v_70.bin.zst"),
    LazyGrid::new("836/v6_v_80.bin.zst"),
    LazyGrid::new("836/v6_v_90.bin.zst"),
    LazyGrid::new("836/v6_v_95.bin.zst"),
    LazyGrid::new("836/v6_v_99.bin.zst"),
];

fn interpolate_probability(grids: &[LazyGrid; 18], lat: Angle, lon: Angle, p: f64) -> f64 {
    let lat_deg = lat.to_degrees();
    let lon_deg = lon.to_degrees();
    let idx = PROB_LEVELS
        .iter()
        .position(|&pl| pl >= p)
        .unwrap_or(PROB_LEVELS.len() - 1);

    if (PROB_LEVELS[idx] - p).abs() < 1e-10 {
        return grids[idx].get().bilinear(lat_deg, lon_deg);
    }

    if idx == 0 {
        return grids[0].get().bilinear(lat_deg, lon_deg);
    }

    let p_below = PROB_LEVELS[idx - 1];
    let p_above = PROB_LEVELS[idx];
    let val_below = grids[idx - 1].get().bilinear(lat_deg, lon_deg);
    let val_above = grids[idx].get().bilinear(lat_deg, lon_deg);

    let t = (p.ln() - p_below.ln()) / (p_above.ln() - p_below.ln());
    val_below + (val_above - val_below) * t
}

/// Returns the surface water vapour density (g/m³) exceeded for `p` % of the average year.
pub fn surface_water_vapour_density(lat: Angle, lon: Angle, p: f64) -> f64 {
    interpolate_probability(&RHO_GRIDS, lat, lon, p)
}

/// Returns the total columnar water vapour content (kg/m²) exceeded for `p` %
/// of the average year.
pub fn total_water_vapour_content(lat: Angle, lon: Angle, p: f64) -> f64 {
    interpolate_probability(&V_GRIDS, lat, lon, p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn madrid() -> (Angle, Angle) {
        (Angle::degrees(40.4), Angle::degrees(-3.7))
    }

    #[test]
    fn test_surface_water_vapour_density_exact_prob() {
        let (lat, lon) = madrid();
        let rho = surface_water_vapour_density(lat, lon, 1.0);
        assert!(rho > 0.0, "rho at 1% = {rho}");
    }

    #[test]
    fn test_surface_water_vapour_density_interpolated_prob() {
        let (lat, lon) = madrid();
        let rho = surface_water_vapour_density(lat, lon, 1.5);
        let rho_lo = surface_water_vapour_density(lat, lon, 1.0);
        let rho_hi = surface_water_vapour_density(lat, lon, 2.0);
        assert!(rho >= rho_lo.min(rho_hi) && rho <= rho_lo.max(rho_hi));
    }

    #[test]
    fn test_surface_water_vapour_density_below_range() {
        let (lat, lon) = madrid();
        let rho = surface_water_vapour_density(lat, lon, 0.05);
        assert!(rho > 0.0);
    }

    #[test]
    fn test_total_water_vapour_content() {
        let (lat, lon) = madrid();
        let v = total_water_vapour_content(lat, lon, 50.0);
        assert!(v > 0.0, "V at 50% = {v}");
    }
}
