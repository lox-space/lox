// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use glam::DVec3;
use lox_core::f64::consts::ROTATION_RATE_EARTH;
use lox_frames::transformations::Rotation;
use lox_time::{
    Time,
    time_scales::{Tdb, Tt, Ut1},
};

use crate::{
    cio::CioLocator, cip::CipCoords, polar_motion::PoleCoords, rotation::EarthRotationAngle,
    tio::TioLocator,
};

pub fn icrf_to_cirf(time: Time<Tdb>, corrections: Option<CipCoords>) -> Rotation {
    let mut xy = CipCoords::iau2006(time);
    let s = CioLocator::iau2006(time, xy);

    xy += corrections.unwrap_or_default();

    Rotation::new(xy.celestial_to_intermediate_matrix(s))
}

pub fn cirf_to_itrf(time: Time<Tdb>, corrections: Option<CipCoords>) -> Rotation {
    icrf_to_cirf(time, corrections).transpose()
}

pub fn cirf_to_tirf(time: Time<Ut1>) -> Rotation {
    let era = EarthRotationAngle::iau2000(time);
    Rotation::new(era.0.rotation_z()).with_angular_velocity(DVec3::new(
        0.0,
        0.0,
        ROTATION_RATE_EARTH,
    ))
}

pub fn tirf_to_cirf(time: Time<Ut1>) -> Rotation {
    cirf_to_tirf(time).transpose()
}

pub fn tirf_to_itrf(time: Time<Tt>, pole: Option<PoleCoords>) -> Rotation {
    match pole {
        Some(pole) => {
            let sp = TioLocator::iau2000(time);
            Rotation::new(pole.polar_motion_matrix_iau2000(sp))
        }
        None => Rotation::default(),
    }
}

pub fn itrf_to_tirf(time: Time<Tt>, pole: Option<PoleCoords>) -> Rotation {
    tirf_to_itrf(time, pole)
}

#[cfg(test)]
mod tests {
    use glam::DMat3;
    use lox_core::units::AngleUnits;
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_celestial_to_terrestrial_iau2006() {
        let tt = Time::from_two_part_julian_date(Tt, 2454195.5, 0.500754444444444);
        let ut1 = Time::from_two_part_julian_date(Ut1, 2454195.5, 0.499999165813831);
        let pole = PoleCoords {
            xp: 0.0349282.arcsec(),
            yp: 0.4833163.arcsec(),
        };
        let corrections = CipCoords {
            x: 0.1750e-3.arcsec(),
            y: -0.2259e-3.arcsec(),
        };
        let npb_exp = DMat3::from_cols_array(&[
            0.999999746339445,
            -0.000000005138822,
            -0.000712264730072,
            -0.000000026475227,
            0.999999999014975,
            -0.000044385242827,
            0.000712264729599,
            0.000044385250426,
            0.999999745354420,
        ])
        .transpose();
        let c2t_exp = DMat3::from_cols_array(&[
            0.973104317573127,
            0.230363826247709,
            -0.000703332818845,
            -0.230363798804182,
            0.973104570735574,
            0.000120888549586,
            0.000712264729599,
            0.000044385250426,
            0.999999745354420,
        ])
        .transpose();
        let c2t_pm_exp = DMat3::from_cols_array(&[
            0.973104317697535,
            0.230363826239128,
            -0.000703163482198,
            -0.230363800456037,
            0.973104570632801,
            0.000118545366625,
            0.000711560162668,
            0.000046626403995,
            0.999999745754024,
        ])
        .transpose();

        let npb_act = icrf_to_cirf(tt.with_scale(Tdb), Some(corrections));
        assert_approx_eq!(npb_act.m, npb_exp, atol <= 1e-11);

        let c2t_act = npb_act.compose(cirf_to_tirf(ut1));
        assert_approx_eq!(c2t_act.m, c2t_exp, atol <= 1e-11);

        let c2t_pm_act = c2t_act.compose(tirf_to_itrf(tt, Some(pole)));
        assert_approx_eq!(c2t_pm_act.m, c2t_pm_exp, atol <= 1e-11);
    }
}
