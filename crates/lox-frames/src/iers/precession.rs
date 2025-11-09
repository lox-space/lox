// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use fast_polynomial::poly_array;
use glam::DMat3;
use lox_test_utils::ApproxEq;
use lox_time::{Time, julian_dates::JulianDate, time_scales::Tt};
use lox_units::Angle;

use crate::iers::{ReferenceSystem, ecliptic::MeanObliquity};

impl ReferenceSystem {
    pub fn precession(&self, time: Time<Tt>) -> PrecessionAngles {
        match self {
            ReferenceSystem::Iers1996 => PrecessionAngles::iau1976(time),
            ReferenceSystem::Iers2003(_) => PrecessionAngles::iau2000(time),
            ReferenceSystem::Iers2010 => PrecessionAngles::iau2006(time),
        }
    }
    /// Returns the rotation from ICRF to the Mean-of-Date (MOD) frame for the given IERS convention.
    pub fn bias_precession_matrix(&self, time: Time<Tt>) -> DMat3 {
        self.precession(time).bias_precession_matrix()
    }
}

pub enum PrecessionAngles {
    /// IAU1976 precession angles.
    Iau1976 { zeta: Angle, z: Angle, theta: Angle },
    /// IAU2000 precession angles.
    Iau2000 {
        psia: Angle,
        oma: Angle,
        chia: Angle,
    },
    /// IAU2006 Fukushima-Williams precession angles.
    Iau2006 {
        gamb: Angle,
        phib: Angle,
        psib: Angle,
        epsa: Angle,
    },
}

impl PrecessionAngles {
    pub fn iau1976(time: Time<Tt>) -> Self {
        let t = time.centuries_since_j2000();

        let tas2r = Angle::arcseconds(t);
        let w = 2306.2181;
        let zeta = (w + ((0.30188) + 0.017998 * t) * t) * tas2r;
        let z = (w + ((1.09468) + 0.018203 * t) * t) * tas2r;
        let theta = (2004.3109 + ((-0.42665) - 0.041833 * t) * t) * tas2r;

        Self::Iau1976 { zeta, z, theta }
    }

    pub fn iau2000(time: Time<Tt>) -> Self {
        let t = time.centuries_since_j2000();

        // Precession angles (Lieske et al. 1977)
        let psia77 = Angle::arcseconds(poly_array(t, &[0.0, 5038.7784, -1.07259, -0.001147]));
        let oma77 = EPS0 + Angle::arcseconds(poly_array(t, &[0.0, 0.0, 0.05127, -0.007726]));
        let chia = Angle::arcseconds(poly_array(t, &[0.0, 10.5526, -2.38064, -0.001125]));

        // Apply IAU 2000 precession corrections
        let PrecessionCorrectionsIau2000 { dpsipr, depspr } =
            PrecessionCorrectionsIau2000::new(time);
        let psia = psia77 + dpsipr;
        let oma = oma77 + depspr;

        Self::Iau2000 { psia, oma, chia }
    }

    pub fn iau2006(time: Time<Tt>) -> Self {
        let t = time.centuries_since_j2000();

        let gamb = Angle::arcseconds(poly_array(
            t,
            &[
                -0.052928,
                10.556378,
                0.4932044,
                -0.00031238,
                -0.000002788,
                0.0000000260,
            ],
        ));
        let phib = Angle::arcseconds(poly_array(
            t,
            &[
                84381.412819,
                -46.811016,
                0.0511268,
                0.00053289,
                -0.000000440,
                -0.0000000176,
            ],
        ));
        let psib = Angle::arcseconds(poly_array(
            t,
            &[
                -0.041775,
                5038.481484,
                1.5584175,
                -0.00018522,
                -0.000026452,
                -0.0000000148,
            ],
        ));
        let epsa = MeanObliquity::iau2006(time.with_scale(Tt)).0;

        Self::Iau2006 {
            gamb,
            phib,
            psib,
            epsa,
        }
    }

    pub fn bias_precession_matrix(&self) -> DMat3 {
        match *self {
            PrecessionAngles::Iau1976 { zeta, z, theta } => {
                let rp = (-z).rotation_z() * theta.rotation_y() * (-zeta).rotation_z();
                rp * frame_bias()
            }
            PrecessionAngles::Iau2000 { psia, oma, chia } => {
                let rp = chia.rotation_z()
                    * (-oma).rotation_x()
                    * (-psia).rotation_z()
                    * EPS0.rotation_x();

                rp * frame_bias()
            }
            PrecessionAngles::Iau2006 {
                gamb,
                phib,
                psib,
                epsa,
            } => {
                (-epsa).rotation_x() * (-psib).rotation_z() * phib.rotation_x() * gamb.rotation_z()
            }
        }
    }
}

const D_PSI_BIAS: Angle = Angle::arcseconds(-0.041775);
const D_EPS_BIAS: Angle = Angle::arcseconds(-0.0068192);
const D_RA0: Angle = Angle::arcseconds(-0.0146);
const EPS0: Angle = Angle::arcseconds(84381.448);

pub fn frame_bias() -> DMat3 {
    (-D_EPS_BIAS).rotation_x() * (EPS0.sin() * D_PSI_BIAS).rotation_y() * D_RA0.rotation_z()
}

const PRECOR: Angle = Angle::arcseconds(-0.29965);
const OBLCOR: Angle = Angle::arcseconds(-0.02524);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, ApproxEq)]
pub struct PrecessionCorrectionsIau2000 {
    pub dpsipr: Angle,
    pub depspr: Angle,
}

impl PrecessionCorrectionsIau2000 {
    pub fn new(time: Time<Tt>) -> Self {
        let t = time.centuries_since_j2000();

        Self {
            dpsipr: t * PRECOR,
            depspr: t * OBLCOR,
        }
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;
    use lox_units::AngleUnits;

    use crate::iers::Iau2000Model;

    use super::*;

    #[test]
    fn test_precession_iers1996() {
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 50123.9999);
        let exp = DMat3::from_cols_array(&[
            0.999_999_550_432_835_1,
            8.696_632_209_480_961e-4,
            3.779_153_474_959_888_4e-4,
            -8.696_632_209_485_112e-4,
            0.999_999_621_842_856_1,
            -1.643_284_776_111_886_4e-7,
            -3.779_153_474_950_335e-4,
            -1.643_306_746_147_367e-7,
            0.999_999_928_589_979,
        ])
        .transpose()
            * frame_bias();
        let act = ReferenceSystem::Iers1996.bias_precession_matrix(time);
        assert_approx_eq!(act, exp, atol <= 1e-12);
    }

    #[test]
    fn test_bias_precession_iau2000() {
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 50123.9999);
        let exp = DMat3::from_cols_array(&[
            0.999_999_550_517_508_8,
            8.695_405_883_617_885e-4,
            3.779_734_722_239_007e-4,
            -8.695_405_990_410_864e-4,
            0.999_999_621_949_492_5,
            -1.360_775_820_404_982e-7,
            -3.779_734_476_558_185e-4,
            -1.925_857_585_832_024e-7,
            0.999_999_928_568_015_3,
        ])
        .transpose();
        let act = ReferenceSystem::Iers2003(Iau2000Model::A).bias_precession_matrix(time);
        assert_approx_eq!(act, exp, atol <= 1e-12);
    }

    #[test]
    fn test_precession_iau2006() {
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 50123.9999);
        let exp = DMat3::from_cols_array(&[
            0.999_999_550_517_600_7,
            8.695_404_617_348_209e-4,
            3.779_735_201_865_589e-4,
            -8.695_404_723_772_031e-4,
            0.999_999_621_949_602_7,
            -1.361_752_497_080_270_2e-7,
            -3.779_734_957_034_089_7e-4,
            -1.924_880_847_894_457e-7,
            0.999_999_928_567_997_2,
        ])
        .transpose();
        let act = ReferenceSystem::Iers2010.bias_precession_matrix(time);
        assert_approx_eq!(act, exp, atol <= 1e-12);
    }

    #[test]
    fn test_frame_bias_iau2000() {
        let exp = DMat3::from_cols_array(&[
            0.999_999_999_999_994_2,
            -7.078_279_744_199_197e-8,
            8.056_217_146_976_134e-8,
            7.078_279_477_857_338e-8,
            0.999_999_999_999_997,
            3.306_041_454_222_136_4e-8,
            -8.056_217_380_986_972e-8,
            -3.306_040_883_980_552_3e-8,
            0.999_999_999_999_996_2,
        ])
        .transpose();
        let act = frame_bias();
        assert_approx_eq!(act, exp, atol <= 1e-12);
    }

    #[test]
    fn test_precession_corrections_iau2000() {
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 53736.0);
        let exp = PrecessionCorrectionsIau2000 {
            dpsipr: -8.716_465_172_668_348e-8.rad(),
            depspr: -7.342_018_386_722_813e-9.rad(),
        };
        let act = PrecessionCorrectionsIau2000::new(time);
        assert_approx_eq!(act, exp, atol <= 1e-22);
    }
}
