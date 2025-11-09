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

use crate::iers::{Corrections, Iau2000Model, ReferenceSystem, ecliptic::MeanObliquity};

mod iau1980;
mod iau2000;
mod iau2006;

impl ReferenceSystem {
    pub fn nutation(&self, time: Time<Tdb>) -> Nutation {
        match self {
            ReferenceSystem::Iers1996 => Nutation::iau1980(time),
            ReferenceSystem::Iers2003(model) => match model {
                Iau2000Model::A => Nutation::iau2000a(time),
                Iau2000Model::B => Nutation::iau2000b(time),
            },
            ReferenceSystem::Iers2010 => Nutation::iau2006a(time),
        }
    }

    pub fn nutation_matrix(&self, time: Time<Tdb>, corr: Corrections) -> DMat3 {
        match self {
            ReferenceSystem::Iers1996 => self
                .nutation(time)
                .nutation_matrix(self.mean_obliquity(time.with_scale(Tt))),
            ReferenceSystem::Iers2003(_) | ReferenceSystem::Iers2010 => {
                let tt = time.with_scale(Tt);
                let epsa = self.mean_obliquity(tt);
                let mut nut = self.nutation(time);
                let rpb = self.bias_precession_matrix(tt);

                nut += self.ecliptic_corrections(corr, nut, epsa, rpb);

                nut.nutation_matrix(epsa)
            }
        }
    }
}

/// Nutation components with respect to some ecliptic of date.
#[derive(Debug, Default, Clone, Copy, PartialEq, ApproxEq)]
pub struct Nutation {
    /// δψ
    pub dpsi: Angle,
    /// δε
    pub deps: Angle,
}

impl Nutation {
    pub fn nutation_matrix(&self, epsa: MeanObliquity) -> DMat3 {
        let epsa = epsa.0;
        let rot1 = epsa.rotation_x();
        let rot2 = (-self.dpsi).rotation_z();
        let rot3 = (-(self.deps + epsa)).rotation_x();
        rot3 * rot2 * rot1
    }
}

impl Add for Nutation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            dpsi: self.dpsi + rhs.dpsi,
            deps: self.deps + rhs.deps,
        }
    }
}

impl Add<&Self> for Nutation {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        Nutation {
            dpsi: self.dpsi + rhs.dpsi,
            deps: self.deps + rhs.deps,
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
            dpsi: -9.630_909_107_115_582e-6.rad(),
            deps: 4.063_239_174_001_679e-5.rad(),
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
}
