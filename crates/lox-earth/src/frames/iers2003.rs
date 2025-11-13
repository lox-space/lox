// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use glam::{DMat3, DVec3};
use lox_core::{f64::consts::ROTATION_RATE_EARTH, units::Angle};
use lox_frames::transformations::Rotation;
use lox_time::{
    Time,
    time_scales::{Tdb, Tt, Ut1},
};

use crate::{
    cip::CipCoords,
    ecliptic::MeanObliquity,
    nutation::Nutation,
    polar_motion::PoleCoords,
    precession::BiasPrecessionIau2000,
    rotation::{EquationOfTheEquinoxes, GreenwichApparentSiderealTime, GreenwichMeanSiderealTime},
    tio::TioLocator,
};

pub fn icrf_to_mod(time: Time<Tt>) -> Rotation {
    let bp = BiasPrecessionIau2000::new(time);
    Rotation::new(bp.rpb)
}

pub fn mod_to_icrf(time: Time<Tt>) -> Rotation {
    icrf_to_mod(time).transpose()
}

pub fn mod_to_tod_iau2000a(time: Time<Tt>, corrections: Option<CipCoords>) -> Rotation {
    let nut = Nutation::iau2000a(time.with_scale(Tdb));
    let epsa = MeanObliquity::iau1980(time);

    let corrections = match corrections {
        Some(corrections) => {
            let numat = nut.nutation_matrix(epsa);
            let BiasPrecessionIau2000 { rpb, .. } = BiasPrecessionIau2000::new(time);
            let rbpn = numat * rpb;
            cip_to_nut_corrections(corrections, rbpn, epsa)
        }
        None => Nutation::default(),
    };

    Rotation::new((nut + corrections).nutation_matrix(epsa))
}

pub fn tod_to_mod_iau2000a(time: Time<Tt>, corrections: Option<CipCoords>) -> Rotation {
    mod_to_tod_iau2000a(time, corrections).transpose()
}

pub fn mod_to_tod_iau2000b(time: Time<Tt>, corrections: Option<CipCoords>) -> Rotation {
    let nut = Nutation::iau2000b(time.with_scale(Tdb));
    let epsa = MeanObliquity::iau1980(time);

    let corrections = match corrections {
        Some(corrections) => {
            let numat = nut.nutation_matrix(epsa);
            let BiasPrecessionIau2000 { rpb, .. } = BiasPrecessionIau2000::new(time);
            let rbpn = numat * rpb;
            cip_to_nut_corrections(corrections, rbpn, epsa)
        }
        None => Nutation::default(),
    };

    Rotation::new((nut + corrections).nutation_matrix(epsa))
}

pub fn tod_to_mod_iau2000b(time: Time<Tt>, corrections: Option<CipCoords>) -> Rotation {
    mod_to_tod_iau2000b(time, corrections).transpose()
}

pub fn tod_to_pef_iau2000a(
    tt: Time<Tt>,
    ut1: Time<Ut1>,
    corrections: Option<CipCoords>,
) -> Rotation {
    let gst = match corrections {
        Some(corrections) => {
            let epsa = MeanObliquity::iau1980(tt);
            let nut = Nutation::iau2000a(tt.with_scale(Tdb));
            let numat = nut.nutation_matrix(epsa);
            let BiasPrecessionIau2000 { rpb, .. } = BiasPrecessionIau2000::new(tt);
            let rbpn = numat * rpb;
            let Nutation {
                longitude: dpsi, ..
            } = nut + cip_to_nut_corrections(corrections, rbpn, epsa);
            let gmst = GreenwichMeanSiderealTime::iau2000(tt, ut1);
            let ee = EquationOfTheEquinoxes::iau2000(tt, epsa.0, dpsi);
            (gmst.0 + ee.0).mod_two_pi()
        }
        None => GreenwichApparentSiderealTime::iau2000b(tt, ut1).0,
    };
    Rotation::new(gst.rotation_z()).with_angular_velocity(DVec3::new(0.0, 0.0, ROTATION_RATE_EARTH))
}

pub fn pef_to_tod_iau2000a(
    tt: Time<Tt>,
    ut1: Time<Ut1>,
    corrections: Option<CipCoords>,
) -> Rotation {
    tod_to_pef_iau2000a(tt, ut1, corrections).transpose()
}

pub fn tod_to_pef_iau2000b(
    tt: Time<Tt>,
    ut1: Time<Ut1>,
    corrections: Option<CipCoords>,
) -> Rotation {
    let gst = match corrections {
        Some(corrections) => {
            let epsa = MeanObliquity::iau1980(tt);
            let nut = Nutation::iau2000a(tt.with_scale(Tdb));
            let numat = nut.nutation_matrix(epsa);
            let BiasPrecessionIau2000 { rpb, .. } = BiasPrecessionIau2000::new(tt);
            let rbpn = numat * rpb;
            let Nutation {
                longitude: dpsi, ..
            } = cip_to_nut_corrections(corrections, rbpn, epsa);
            let gmst = GreenwichMeanSiderealTime::iau2000(tt, ut1);
            let ee = EquationOfTheEquinoxes::iau2000(tt, epsa.0, dpsi);
            (gmst.0 + ee.0).mod_two_pi()
        }
        None => GreenwichApparentSiderealTime::iau2000b(tt, ut1).0,
    };
    Rotation::new(gst.rotation_z()).with_angular_velocity(DVec3::new(0.0, 0.0, ROTATION_RATE_EARTH))
}

pub fn pef_to_tod_iau2000b(
    tt: Time<Tt>,
    ut1: Time<Ut1>,
    corrections: Option<CipCoords>,
) -> Rotation {
    tod_to_pef_iau2000b(tt, ut1, corrections).transpose()
}

pub fn pef_to_itrf(tt: Time<Tt>, pole: Option<PoleCoords>) -> Rotation {
    let pole = pole.unwrap_or_default();
    let sp = TioLocator::iau2000(tt);
    Rotation::new(pole.polar_motion_matrix_iau2000(sp))
}

pub fn itrf_to_pef(tt: Time<Tt>, pole: Option<PoleCoords>) -> Rotation {
    pef_to_itrf(tt, pole).transpose()
}

fn cip_to_nut_corrections(
    CipCoords { x: dx, y: dy }: CipCoords,
    rbpn: DMat3,
    epsa: MeanObliquity,
) -> Nutation {
    let v1 = DVec3::new(dx.as_f64(), dy.as_f64(), 0.0);
    let v2 = rbpn * v1;
    Nutation {
        longitude: Angle::new(v2.x / epsa.0.sin()),
        obliquity: Angle::new(v2.y),
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::AngleUnits;
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_celestial_to_terrestrial_iers2003() {
        let tt = Time::from_two_part_julian_date(Tt, 2454195.5, 0.500754444444444);
        let ut1 = Time::from_two_part_julian_date(Ut1, 2454195.5, 0.499999165813831);
        let pole = PoleCoords {
            xp: 0.0349282.arcsec(),
            yp: 0.4833163.arcsec(),
        };
        let corrections = CipCoords {
            x: 0.1725e-3.arcsec(),
            y: -0.2650e-3.arcsec(),
        };
        let npb_exp = DMat3::from_cols_array(&[
            0.999998402755640,
            -0.001639289519579,
            -0.000712191013215,
            0.001639257491365,
            0.999998655379006,
            -0.000045552787478,
            0.000712264729795,
            0.000044385250265,
            0.999999745354420,
        ])
        .transpose();
        let c2t_exp = DMat3::from_cols_array(&[
            0.973104317573209,
            0.230363826247361,
            -0.000703332818999,
            -0.230363798803834,
            0.973104570735656,
            0.000120888549787,
            0.000712264729795,
            0.000044385250265,
            0.999999745354420,
        ])
        .transpose();
        let c2t_pm_exp = DMat3::from_cols_array(&[
            0.973104317697618,
            0.230363826238780,
            -0.000703163482352,
            -0.230363800455689,
            0.973104570632883,
            0.000118545366826,
            0.000711560162864,
            0.000046626403835,
            0.999999745754024,
        ])
        .transpose();

        let npb_act = icrf_to_mod(tt).compose(mod_to_tod_iau2000a(tt, Some(corrections)));
        assert_approx_eq!(npb_act.m, npb_exp, atol <= 1e-12);

        let c2t_act = npb_act.compose(tod_to_pef_iau2000a(tt, ut1, Some(corrections)));
        assert_approx_eq!(c2t_act.m, c2t_exp, atol <= 1e-12);

        let c2t_pm_act = c2t_act.compose(pef_to_itrf(tt, Some(pole)));
        assert_approx_eq!(c2t_pm_act.m, c2t_pm_exp, atol <= 1e-12);
    }
}
