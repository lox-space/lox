// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Module nutation exposes a function for calculating Earth nutation using a number of IAU nutation
//! models.

use std::ops::Add;

use glam::DMat3;
use lox_core::units::Angle;
use lox_test_utils::ApproxEq;
use lox_time::{
    Time,
    time_scales::{Tdb, Tt},
};

use crate::ecliptic::MeanObliquity;

mod iau1980;
mod iau2000;
mod iau2006;

/// Nutation components with respect to some ecliptic of date.
#[derive(Debug, Default, Clone, PartialEq, ApproxEq)]
pub struct Nutation {
    /// δψ
    pub longitude: Angle,
    /// δε
    pub obliquity: Angle,
}

impl Nutation {
    pub fn nutation_matrix(&self, mean_obliquity: MeanObliquity) -> DMat3 {
        let rot1 = mean_obliquity.0.rotation_x();
        let rot2 = (-self.longitude).rotation_z();
        let rot3 = (-(self.obliquity + mean_obliquity.0)).rotation_x();
        rot3 * rot2 * rot1
    }

    pub fn nutation_matrix_iau1980(tdb: Time<Tdb>) -> DMat3 {
        let tt = tdb.with_scale(Tt);
        let nut = Self::iau1980(tdb);
        let obl = MeanObliquity::iau1980(tt);
        nut.nutation_matrix(obl)
    }
}

impl Add for Nutation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            longitude: self.longitude + rhs.longitude,
            obliquity: self.obliquity + rhs.obliquity,
        }
    }
}

impl Add<&Self> for Nutation {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        Nutation {
            longitude: self.longitude + rhs.longitude,
            obliquity: self.obliquity + rhs.obliquity,
        }
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::AngleUnits;
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_nutation_matrix() {
        let nut = Nutation {
            longitude: -9.630_909_107_115_582e-6.rad(),
            obliquity: 4.063_239_174_001_679e-5.rad(),
        };
        let obl = MeanObliquity(Angle::new(0.409_078_976_335_651));
        let exp = DMat3::from_cols_array(&[
            0.999_999_999_953_622_8,
            8.836_239_320_236_25e-6,
            3.830_833_447_458_252e-6,
            -8.836_083_657_016_69e-6,
            0.999_999_999_135_465_4,
            -4.063_240_865_361_857_4e-5,
            -3.831_192_481_833_385_5e-6,
            4.063_237_480_216_934e-5,
            0.999_999_999_167_166_1,
        ])
        .transpose();
        let act = nut.nutation_matrix(obl);
        assert_approx_eq!(act, exp, rtol <= 1e-12);
    }

    #[test]
    fn test_nutation_matrix_iau1980() {
        let time = Time::from_two_part_julian_date(Tdb, 2400000.5, 53736.0);
        let exp = DMat3::from_cols_array(&[
            0.999_999_999_953_5,
            8.847_935_789_636_432e-6,
            3.835_906_502_164_019_5e-6,
            -8.847_780_042_583_437e-6,
            0.999_999_999_136_657,
            -4.060_052_702_727_131e-5,
            -3.836_265_729_708_479e-6,
            4.060_049_308_612_638_4e-5,
            0.999_999_999_168_441_5,
        ])
        .transpose();
        let act = Nutation::nutation_matrix_iau1980(time);
        assert_approx_eq!(act, exp, rtol <= 1e-12);
    }
}
