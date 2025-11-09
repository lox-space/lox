// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
//
// SPDX-License-Identifier: MPL-2.0

//! Module cip exposes functions for calculating the position of the
//! Celestial Intermediate Pole (CIP).

use std::ops::{Add, AddAssign};

use glam::DMat3;
use lox_core::units::{Angle, AngleUnits};
use lox_time::{Time, julian_dates::JulianDate, time_scales::Tdb};

use crate::iers::cio::CioLocator;

mod iau2006;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct CipCoords {
    pub x: Angle,
    pub y: Angle,
}

impl CipCoords {
    /// Calculates the (X, Y) coordinates of the Celestial Intermediate Pole (CIP) using the the IAU
    /// 2006 precession and IAU 2000A nutation models.
    pub fn iau2006(time: Time<Tdb>) -> Self {
        let t = time.centuries_since_j2000();
        let (x, y) = iau2006::cip_coords(t);
        Self { x, y }
    }

    /// Extract the CIP coordinates from a bias-precession-nutation matrix.
    pub fn from_matrix(bpn: DMat3) -> Self {
        let x = bpn.x_axis.z.rad();
        let y = bpn.y_axis.z.rad();
        Self { x, y }
    }

    pub fn celestial_to_intermediate_matrix(&self, s: CioLocator) -> DMat3 {
        let x = self.x.to_radians();
        let y = self.y.to_radians();
        let r2 = x.powi(2) + y.powi(2);

        let e = if r2 > 0.0 {
            Angle::from_atan2(y, x)
        } else {
            Angle::default()
        };
        let d = Angle::from_atan((r2 / (1.0 - r2)).sqrt());

        (-(e + s.0)).rotation_z() * d.rotation_y() * e.rotation_z()
    }
}

impl Add for CipCoords {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        CipCoords {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for CipCoords {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;

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

    #[test]
    fn test_cip_celestial_to_intermediate_matrix() {
        let xy = CipCoords {
            x: 5.791_308_486_706_011e-4.rad(),
            y: 4.020_579_816_732_961e-5.rad(),
        };
        let s = CioLocator(-1.220_040_848_472_272e-8.rad());
        let exp = DMat3::from_cols_array(&[
            0.999_999_832_303_715_7,
            5.581_984_869_168_499e-10,
            -5.791_308_491_611_282e-4,
            -2.384_261_642_670_440_2e-8,
            0.999_999_999_191_746_9,
            -4.020_579_110_169_669e-5,
            5.791_308_486_706_011e-4,
            4.020_579_816_732_961e-5,
            0.999_999_831_495_462_8,
        ])
        .transpose();
        let act = xy.celestial_to_intermediate_matrix(s);
        assert_approx_eq!(act, exp, atol <= 1e-12);
    }
}
