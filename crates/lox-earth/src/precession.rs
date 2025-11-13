// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use fast_polynomial::poly_array;
use glam::DMat3;
use lox_core::units::Angle;
use lox_test_utils::ApproxEq;
use lox_time::{
    Time,
    julian_dates::JulianDate,
    time_scales::{Tdb, Tt},
};

use crate::ecliptic::MeanObliquity;

/// Frame bias longitude correction.
pub const D_PSI_BIAS: Angle = Angle::arcseconds(-0.041775);
/// Frame bias obliquity correction.
pub const D_EPS_BIAS: Angle = Angle::arcseconds(-0.0068192);
/// The ICRS RA of the J2000.0 equinox (Chapront et al., 2002)
pub const D_RA0: Angle = Angle::arcseconds(-0.0146);

/// J2000.0 obliquity (Lieske et al. 1977)
const EPS0: Angle = Angle::arcseconds(84381.448);

pub struct PrecessionIau1976 {
    pub zeta: Angle,
    pub z: Angle,
    pub theta: Angle,
}

impl PrecessionIau1976 {
    pub fn new(time0: Time<Tdb>, time1: Time<Tdb>) -> Self {
        let t0 = time0.centuries_since_j2000();
        let t = (time1 - time0).centuries_since_j2000();
        let tas2r = Angle::arcseconds(t);
        let w = poly_array(t0, &[2306.2181, 1.39656, -0.000139]);
        let zeta = (w + ((0.30188 - 0.000344 * t0) + 0.017998 * t) * t) * tas2r;

        let z = (w + ((1.09468 + 0.000066 * t0) + 0.018203 * t) * t) * tas2r;

        let theta = ((2004.3109 + (-0.85330 - 0.000217 * t0) * t0)
            + ((-0.42665 - 0.000217 * t0) - 0.041833 * t) * t)
            * tas2r;
        Self { zeta, z, theta }
    }
}

pub struct BiasPrecessionIau2000 {
    /// Frame bias matrix.
    pub rb: DMat3,
    /// Precession matrix.
    pub rp: DMat3,
    /// Bias-precession matrix.
    pub rbp: DMat3,
}

impl BiasPrecessionIau2000 {
    pub fn new(time: Time<Tt>) -> Self {
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

        // Frame bias matrix: GCRS to J2000.0
        let rb = (-D_EPS_BIAS).rotation_x()
            * (EPS0.sin() * D_PSI_BIAS).rotation_y()
            * D_RA0.rotation_z();

        // Precession matrix: J2000.0 to mean of date
        let rp = chia.rotation_z() * (-oma).rotation_x() * (-psia).rotation_z() * EPS0.rotation_x();

        // Bias-precession matrix: GCRS to mean of date
        let rbp = rp * rb;
        Self { rb, rp, rbp }
    }
}

pub struct PrecessionIau2006 {
    pub gamb: Angle,
    pub phib: Angle,
    pub psib: Angle,
    pub epsa: Angle,
}

impl PrecessionIau2006 {
    pub fn new(time: Time<Tt>) -> Self {
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
        let epsa = MeanObliquity::iau2006(time).0;

        Self {
            gamb,
            phib,
            psib,
            epsa,
        }
    }

    pub fn bias_precession_matrix(&self) -> DMat3 {
        (-self.epsa).rotation_x()
            * (-self.psib).rotation_z()
            * self.phib.rotation_x()
            * self.gamb.rotation_z()
    }
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
    use lox_core::units::AngleUnits;
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_precession_rate_iau2000() {
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 53736.0);
        let exp = PrecessionCorrectionsIau2000 {
            dpsipr: -8.716_465_172_668_348e-8.rad(),
            depspr: -7.342_018_386_722_813e-9.rad(),
        };
        let act = PrecessionCorrectionsIau2000::new(time);
        assert_approx_eq!(act, exp, atol <= 1e-22);
    }

    #[test]
    fn test_precession_iau1976() {
        let time0 = Time::from_two_part_julian_date(Tdb, 2400000.5, 33282.0);
        let time1 = Time::from_two_part_julian_date(Tdb, 2400000.5, 51544.0);
        let prec = PrecessionIau1976::new(time0, time1);
        let zeta_exp = Angle::new(5.588_961_642_000_161e-3);
        let z_exp = Angle::new(5.589_922_365_870_681e-3);
        let theta_exp = Angle::new(4.858_945_471_687_297e-3);
        assert_approx_eq!(prec.zeta, zeta_exp, atol <= 1e-12);
        assert_approx_eq!(prec.z, z_exp, atol <= 1e-12);
        assert_approx_eq!(prec.theta, theta_exp, atol <= 1e-12);
    }

    #[test]
    fn test_bias_precession_iau2000() {
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 50123.9999);
        let bp = BiasPrecessionIau2000::new(time);
        let rb_exp = DMat3::from_cols_array(&[
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
        let rp_exp = DMat3::from_cols_array(&[
            0.999_999_550_486_404_9,
            8.696_113_836_207_084e-4,
            3.778_928_813_389_333e-4,
            -8.696_113_818_227_266e-4,
            0.999_999_621_887_936_6,
            -1.690_679_263_009_242_2e-7,
            -3.778_928_854_764_695e-4,
            -1.595_521_004_195_286_6e-7,
            0.999_999_928_598_468_3,
        ])
        .transpose();
        let rbp_exp = DMat3::from_cols_array(&[
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
        assert_approx_eq!(bp.rb, rb_exp, atol <= 1e-12);
        assert_approx_eq!(bp.rp, rp_exp, atol <= 1e-12);
        assert_approx_eq!(bp.rbp, rbp_exp, atol <= 1e-12);
    }

    #[test]
    fn test_precession_iau2006() {
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 50123.9999);
        let prec = PrecessionIau2006::new(time);
        let gamb_exp = Angle::new(-2.243_387_670_997_995_8e-6);
        let phib_exp = Angle::new(0.409_101_460_239_131_3);
        let psib_exp = Angle::new(-9.501_954_178_013_031e-4);
        let epsa_exp = Angle::new(0.409_101_431_658_736_75);
        assert_approx_eq!(prec.gamb, gamb_exp, atol <= 1e-16);
        assert_approx_eq!(prec.phib, phib_exp, atol <= 1e-12);
        assert_approx_eq!(prec.psib, psib_exp, atol <= 1e-14);
        assert_approx_eq!(prec.epsa, epsa_exp, atol <= 1e-12);

        let bp_exp = DMat3::from_cols_array(&[
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
        assert_approx_eq!(prec.bias_precession_matrix(), bp_exp, atol <= 1e-12);
    }
}
