// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use glam::DMat3;
use lox_core::units::Angle;
use lox_time::{Time, time_scales::Tt};

use crate::iers::{ReferenceSystem, tio::TioLocator};

impl ReferenceSystem {
    pub fn polar_motion_matrix(&self, time: Time<Tt>, pole_coords: PoleCoords) -> DMat3 {
        if pole_coords.is_zero() {
            return DMat3::IDENTITY;
        };
        match self {
            ReferenceSystem::Iers1996 => pole_coords.polar_motion_matrix(),
            ReferenceSystem::Iers2003(_) | ReferenceSystem::Iers2010 => {
                let sp = TioLocator::iau2000(time);
                pole_coords.polar_motion_matrix_iau2000(sp)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PoleCoords {
    pub xp: Angle,
    pub yp: Angle,
}

impl PoleCoords {
    pub fn polar_motion_matrix(&self) -> DMat3 {
        (-self.xp).rotation_y() * (-self.yp).rotation_x()
    }

    pub fn polar_motion_matrix_iau2000(&self, sp: TioLocator) -> DMat3 {
        self.polar_motion_matrix() * sp.0.rotation_z()
    }

    pub fn is_zero(&self) -> bool {
        self.xp.is_zero() && self.yp.is_zero()
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::AngleUnits;
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_polar_motion_matrix_iau2000() {
        let pole = PoleCoords {
            xp: 2.55060238e-7.rad(),
            yp: 1.860359247e-6.rad(),
        };
        let sp = TioLocator(-1.367_174_580_728_891_5e-11.rad());
        let exp = DMat3::from_cols_array(&[
            0.999_999_999_999_967_5,
            -1.367_174_580_728_847e-11,
            2.550_602_379_999_972e-7,
            1.414_624_947_957_029_7e-11,
            0.999_999_999_998_269_5,
            -1.860_359_246_998_866_3e-6,
            -2.550_602_379_741_215_3e-7,
            1.860_359_247_002_414e-6,
            0.999_999_999_998_237,
        ])
        .transpose();
        let act = pole.polar_motion_matrix_iau2000(sp);
        assert_approx_eq!(act, exp, atol <= 1e-12);
    }
}
