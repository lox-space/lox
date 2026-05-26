// SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MIT AND MPL-2.0

//! ITU-R P.836-6: Water vapour: surface density and total columnar content.
//!
//! Both quantities are grid-based; they are served by
//! [`crate::ItuProvider::surface_water_vapour_density`] and
//! [`crate::ItuProvider::total_water_vapour_content`].

#[cfg(test)]
mod tests {
    use crate::provider::test_fixture::provider;
    use lox_core::units::Angle;

    fn madrid() -> (Angle, Angle) {
        (Angle::degrees(40.4), Angle::degrees(-3.7))
    }

    #[test]
    fn test_surface_water_vapour_density_exact_prob() {
        let p = provider();
        let (lat, lon) = madrid();
        let rho = p.surface_water_vapour_density(lat, lon, 1.0).unwrap();
        assert!(rho > 0.0, "rho at 1% = {rho}");
    }

    #[test]
    fn test_surface_water_vapour_density_interpolated_prob() {
        let p = provider();
        let (lat, lon) = madrid();
        let rho = p.surface_water_vapour_density(lat, lon, 1.5).unwrap();
        let rho_lo = p.surface_water_vapour_density(lat, lon, 1.0).unwrap();
        let rho_hi = p.surface_water_vapour_density(lat, lon, 2.0).unwrap();
        assert!(rho >= rho_lo.min(rho_hi) && rho <= rho_lo.max(rho_hi));
    }

    #[test]
    fn test_surface_water_vapour_density_below_range() {
        let p = provider();
        let (lat, lon) = madrid();
        let rho = p.surface_water_vapour_density(lat, lon, 0.05).unwrap();
        assert!(rho > 0.0);
    }

    #[test]
    fn test_total_water_vapour_content() {
        let p = provider();
        let (lat, lon) = madrid();
        let v = p.total_water_vapour_content(lat, lon, 50.0).unwrap();
        assert!(v > 0.0, "V at 50% = {v}");
    }
}
