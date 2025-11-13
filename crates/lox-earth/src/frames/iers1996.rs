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
    ecliptic::MeanObliquity,
    nutation::Nutation,
    polar_motion::PoleCoords,
    precession::PrecessionIau1976,
    rotation::{EquationOfTheEquinoxes, GreenwichApparentSiderealTime, GreenwichMeanSiderealTime},
};

pub fn j2000_to_mod(time: Time<Tdb>) -> Rotation {
    Rotation::new(PrecessionIau1976::matrix(time))
}

pub fn mod_to_j2000(time: Time<Tdb>) -> Rotation {
    j2000_to_mod(time).transpose()
}

pub fn mod_to_tod(time: Time<Tdb>, corrections: Option<Nutation>) -> Rotation {
    let nut = Nutation::iau1980(time) + corrections.unwrap_or_default();
    let epsa = MeanObliquity::iau1980(time.with_scale(Tt));
    Rotation::new(nut.nutation_matrix(epsa))
}

pub fn tod_to_mod(time: Time<Tdb>, corrections: Option<Nutation>) -> Rotation {
    mod_to_tod(time, corrections).transpose()
}

pub fn tod_to_pef(tt: Time<Tt>, ut1: Time<Ut1>, corrections: Option<Nutation>) -> Rotation {
    let gst = match corrections {
        Some(Nutation {
            longitude: dpsi, ..
        }) => {
            let gmst = GreenwichMeanSiderealTime::iau1982(ut1);
            let epsa = MeanObliquity::iau1980(tt);
            let ee = EquationOfTheEquinoxes::iau1994(tt.with_scale(Tdb));
            (gmst.0 + ee.0 + epsa.0.cos() * dpsi).mod_two_pi()
        }
        None => GreenwichApparentSiderealTime::iau1994(ut1).0,
    };
    Rotation::new(gst.rotation_z()).with_angular_velocity(DVec3::new(0.0, 0.0, ROTATION_RATE_EARTH))
}

pub fn pef_to_tod(tt: Time<Tt>, ut1: Time<Ut1>, corrections: Option<Nutation>) -> Rotation {
    tod_to_pef(tt, ut1, corrections).transpose()
}

pub fn pef_to_itrf(pole: PoleCoords) -> Rotation {
    Rotation::new(pole.polar_motion_matrix())
}

pub fn itrf_to_pef(pole: PoleCoords) -> Rotation {
    pef_to_itrf(pole).transpose()
}

#[cfg(test)]
mod tests {
    use glam::DMat3;
    use lox_core::units::AngleUnits;
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_celestial_to_terrestrial_iers1996() {
        let tt = Time::from_two_part_julian_date(Tt, 2454195.5, 0.500754444444444);
        let ut1 = Time::from_two_part_julian_date(Ut1, 2454195.5, 0.499999165813831);
        let pole = PoleCoords {
            xp: 0.0349282.arcsec(),
            yp: 0.4833163.arcsec(),
        };
        let corrections = Nutation {
            longitude: -55.0655e-3.arcsec(),
            obliquity: -6.3580e-3.arcsec(),
        };
        let npb_exp = DMat3::from_cols_array(&[
            0.999998403176203,
            -0.001639032970562,
            -0.000712190961847,
            0.001639000942243,
            0.999998655799521,
            -0.000045552846624,
            0.000712264667137,
            0.000044385492226,
            0.999999745354454,
        ])
        .transpose();
        let c2t_exp = DMat3::from_cols_array(&[
            0.973104317592265,
            0.230363826166883,
            -0.000703332813776,
            -0.230363798723533,
            0.973104570754697,
            0.000120888299841,
            0.000712264667137,
            0.000044385492226,
            0.999999745354454,
        ])
        .transpose();
        let c2t_pm_exp = DMat3::from_cols_array(&[
            0.973104317712772,
            0.230363826174782,
            -0.000703163477127,
            -0.230363800391868,
            0.973104570648022,
            0.000118545116892,
            0.000711560100206,
            0.000046626645796,
            0.999999745754058,
        ])
        .transpose();

        let npb_act = j2000_to_mod(tt.with_scale(Tdb))
            .compose(mod_to_tod(tt.with_scale(Tdb), Some(corrections.clone())));
        assert_approx_eq!(npb_act.m, npb_exp, atol <= 1e-14);

        let c2t_act = npb_act.compose(tod_to_pef(tt, ut1, Some(corrections)));
        assert_approx_eq!(c2t_act.m, c2t_exp, atol <= 1e-12);

        let c2t_pm_act = c2t_act.compose(pef_to_itrf(pole));
        assert_approx_eq!(c2t_pm_act.m, c2t_pm_exp, atol <= 1e-12);
    }
}
