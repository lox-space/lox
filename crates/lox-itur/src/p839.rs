// SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MIT AND MPL-2.0

//! ITU-R P.839-4: Rain height model for prediction methods.
//!
//! The mean annual rain height is computed from the mean annual 0°C isotherm height.

use lox_core::units::{Angle, Distance};

use crate::data::LazyGrid;

static ISOTHERM_HEIGHT: LazyGrid = LazyGrid::new("839/v4_esa0height.bin.zst");

/// Returns the mean annual 0°C isotherm height at the given location.
pub fn isotherm_0c_height(lat: Angle, lon: Angle) -> Distance {
    Distance::kilometers(
        ISOTHERM_HEIGHT
            .get()
            .bilinear(lat.to_degrees(), lon.to_degrees()),
    )
}

/// Returns the mean annual rain height at the given location (P.839-4 Eq. 1).
///
/// h_R = h_0 + 0.36 km
pub fn rain_height(lat: Angle, lon: Angle) -> Distance {
    let h0_km = ISOTHERM_HEIGHT
        .get()
        .bilinear(lat.to_degrees(), lon.to_degrees());
    Distance::kilometers(h0_km + 0.36)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isotherm_0c_height() {
        let h = isotherm_0c_height(Angle::degrees(40.4), Angle::degrees(-3.7));
        assert!(h.to_kilometers() > 1.0 && h.to_kilometers() < 6.0);
    }

    #[test]
    fn test_rain_height() {
        let h = rain_height(Angle::degrees(40.4), Angle::degrees(-3.7));
        let h0 = isotherm_0c_height(Angle::degrees(40.4), Angle::degrees(-3.7));
        assert!((h.to_kilometers() - h0.to_kilometers() - 0.36).abs() < 1e-10);
    }
}
