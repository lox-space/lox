// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
//
// SPDX-License-Identifier: MPL-2.0

//! Module cip exposes functions for calculating the position of the
//! Celestial Intermediate Pole (CIP).

use glam::DMat3;
use lox_core::{
    types::units::JulianCenturies,
    units::{Angle, AngleUnits},
};

mod xy06;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct CipCoords {
    pub x: Angle,
    pub y: Angle,
}

impl CipCoords {
    /// Calculates the (X, Y) coordinates of the Celestial Intermediate Pole (CIP) using the the IAU
    /// 2006 precession and IAU 2000A nutation models.
    pub fn new(centuries_since_j2000_tdb: JulianCenturies) -> Self {
        let (x, y) = xy06::cip_coords(centuries_since_j2000_tdb);
        Self { x, y }
    }

    /// Extract the CIP coordinates from a bias-precession-nutation matrix.
    pub fn from_matrix(bpn: DMat3) -> Self {
        let x = bpn.x_axis.z.rad();
        let y = bpn.y_axis.z.rad();
        Self { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cip_from_matrix() {
        let bpn = DMat3::from_cols_array(&[
            9.999962358680738e-1,
            -2.516417057665452e-3,
            -1.093_569_785_342_37e-3,
            2.516462370370876e-3,
            9.999968329010883e-1,
            4.006_159_587_358_31e-5,
            1.093465510215479e-3,
            -4.281337229063151e-5,
            9.999994012499173e-1,
        ])
        .transpose();
        let cip = CipCoords::from_matrix(bpn);
        assert_eq!(cip.x, 1.093465510215479e-3.rad());
        assert_eq!(cip.y, -4.281337229063151e-5.rad());
    }
}
