// SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MIT AND MPL-2.0

//! ITU-R P.839-4: Rain height model for prediction methods.
//!
//! The mean annual 0°C isotherm height and rain height (h_R = h_0 + 0.36 km)
//! are grid-based; they are served by [`crate::ItuProvider::isotherm_0c_height`]
//! and [`crate::ItuProvider::rain_height`].

#[cfg(test)]
mod tests {
    use crate::provider::test_fixture::provider;
    use lox_core::units::Angle;

    #[test]
    fn test_isotherm_0c_height() {
        let p = provider();
        let h = p
            .isotherm_0c_height(Angle::degrees(40.4), Angle::degrees(-3.7))
            .unwrap();
        assert!(h.to_kilometers() > 1.0 && h.to_kilometers() < 6.0);
    }

    #[test]
    fn test_rain_height() {
        let p = provider();
        let h = p
            .rain_height(Angle::degrees(40.4), Angle::degrees(-3.7))
            .unwrap();
        let h0 = p
            .isotherm_0c_height(Angle::degrees(40.4), Angle::degrees(-3.7))
            .unwrap();
        assert!((h.to_kilometers() - h0.to_kilometers() - 0.36).abs() < 1e-10);
    }
}
