// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Module rotation_angle exposes functions for calculating the Earth Rotation Angle (ERA).

use std::f64::consts::TAU;
use std::iter::zip;

use fast_polynomial::poly_array;
use glam::DMat3;
use lox_core::f64::consts::{SECONDS_PER_DAY, SECONDS_PER_HALF_DAY};
use lox_core::units::{Angle, AngleUnits};
use lox_test_utils::ApproxEq;
use lox_time::time_scales::{Tdb, Tt, Ut1};
use lox_time::{Time, julian_dates::JulianDate};

use crate::iers::ecliptic::MeanObliquity;
use crate::iers::fundamental::iers03::{
    d_iers03, earth_l_iers03, f_iers03, l_iers03, lp_iers03, omega_iers03, pa_iers03,
    venus_l_iers03,
};
use crate::iers::nutation::Nutation;
use crate::iers::precession::PrecessionCorrectionsIau2000;
use crate::iers::{Corrections, Iau2000Model, ReferenceSystem};

mod complementary_terms;

impl ReferenceSystem {
    pub fn earth_rotation(&self, tt: Time<Tt>, ut1: Time<Ut1>, corr: Corrections) -> DMat3 {
        self.greenwich_apparent_sidereal_time(tt, ut1, corr)
            .0
            .rotation_z()
    }

    pub fn greenwich_mean_sidereal_time(
        &self,
        tt: Time<Tt>,
        ut1: Time<Ut1>,
    ) -> GreenwichMeanSiderealTime {
        match self {
            ReferenceSystem::Iers1996 => GreenwichMeanSiderealTime::iau1982(ut1),
            ReferenceSystem::Iers2003(_) => GreenwichMeanSiderealTime::iau2000(tt, ut1),
            ReferenceSystem::Iers2010 => GreenwichMeanSiderealTime::iau2006(tt, ut1),
        }
    }

    pub fn greenwich_apparent_sidereal_time(
        &self,
        tt: Time<Tt>,
        ut1: Time<Ut1>,
        corr: Corrections,
    ) -> GreenwichApparentSiderealTime {
        if corr.is_zero() {
            return match self {
                ReferenceSystem::Iers1996 => Gast::iau1994(ut1),
                ReferenceSystem::Iers2003(model) => match model {
                    Iau2000Model::A => Gast::iau2000a(tt, ut1),
                    Iau2000Model::B => Gast::iau2000b(tt, ut1),
                },
                ReferenceSystem::Iers2010 => Gast::iau2006a(tt, ut1),
            };
        };
        let gmst = self.greenwich_mean_sidereal_time(tt, ut1);
        let tdb = tt.with_scale(Tdb);
        let rpb = self.bias_precession_matrix(tt);
        let epsa = self.mean_obliquity(tt);
        let mut nut = self.nutation(tdb);
        let ecl_corr = self.ecliptic_corrections(corr, nut, epsa, rpb);
        nut += ecl_corr;
        match self {
            ReferenceSystem::Iers1996 => {
                let ee = EquationOfTheEquinoxes::iau1994(tdb);
                GreenwichApparentSiderealTime(
                    (gmst.0 + ee.0 + epsa.0.cos() * ecl_corr.0).mod_two_pi(),
                )
            }
            ReferenceSystem::Iers2003(_) => {
                let ee = EquationOfTheEquinoxes::iau2000(tt, epsa.0, nut.dpsi);
                GreenwichApparentSiderealTime((gmst.0 + ee.0).mod_two_pi())
            }
            ReferenceSystem::Iers2010 => {
                let ee = EquationOfTheEquinoxes::iau2000(tt, epsa.0, nut.dpsi);
                GreenwichApparentSiderealTime((gmst.0 + ee.0).mod_two_pi())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, ApproxEq)]
pub struct EarthRotationAngle(pub Angle);

impl EarthRotationAngle {
    pub fn iau2000(time: Time<Ut1>) -> Self {
        let d = time.days_since_j2000();
        let f = d.rem_euclid(1.0); // fractional part of t
        Self(
            (TAU * (f + 0.7790572732640 + 0.00273781191135448 * d))
                .rad()
                .mod_two_pi(),
        )
    }
}

pub type Era = EarthRotationAngle;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, ApproxEq)]
pub struct GreenwichMeanSiderealTime(pub Angle);

pub type Gmst = GreenwichMeanSiderealTime;

// Coefficients of IAU 1982 GMST-UT1 model
const A: f64 = 24110.54841 - SECONDS_PER_HALF_DAY;
const B: f64 = 8640184.812866;
const C: f64 = 0.093104;
const D: f64 = -6.2e-6;

impl GreenwichMeanSiderealTime {
    pub fn iau1982(time: Time<Ut1>) -> Self {
        let t = time.centuries_since_j2000();
        let f = time.days_since_j2000().rem_euclid(1.0) * SECONDS_PER_DAY;
        Self(Angle::from_hms(0, 0, poly_array(t, &[A, B, C, D]) + f).mod_two_pi())
    }

    pub fn iau2000(tt: Time<Tt>, ut1: Time<Ut1>) -> Self {
        let t = tt.centuries_since_j2000();
        Self(
            EarthRotationAngle::iau2000(ut1).0
                + Angle::arcseconds(poly_array(
                    t,
                    &[0.014506, 4612.15739966, 1.39667721, -0.00009344, 0.00001882],
                ))
                .mod_two_pi(),
        )
    }

    pub fn iau2006(tt: Time<Tt>, ut1: Time<Ut1>) -> Self {
        let t = tt.centuries_since_j2000();
        Self(
            EarthRotationAngle::iau2000(ut1).0
                + Angle::arcseconds(poly_array(
                    t,
                    &[
                        0.014506,
                        4612.156534,
                        1.3915817,
                        -0.00000044,
                        -0.000029956,
                        -0.0000000368,
                    ],
                ))
                .mod_two_pi(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, ApproxEq)]
pub struct GreenwichApparentSiderealTime(pub Angle);

pub type Gast = GreenwichApparentSiderealTime;

impl GreenwichApparentSiderealTime {
    pub fn iau1994(time: Time<Ut1>) -> Self {
        Self(
            (GreenwichMeanSiderealTime::iau1982(time).0
                + EquationOfTheEquinoxes::iau1994(time.with_scale(Tdb)).0)
                .mod_two_pi(),
        )
    }

    pub fn iau2000a(tt: Time<Tt>, ut1: Time<Ut1>) -> Self {
        Self(
            (GreenwichMeanSiderealTime::iau2000(tt, ut1).0
                + EquationOfTheEquinoxes::iau2000a(tt).0)
                .mod_two_pi(),
        )
    }

    pub fn iau2000b(tt: Time<Tt>, ut1: Time<Ut1>) -> Self {
        Self(
            (GreenwichMeanSiderealTime::iau2000(tt, ut1).0
                + EquationOfTheEquinoxes::iau2000b(tt).0)
                .mod_two_pi(),
        )
    }

    pub fn iau2006a(tt: Time<Tt>, ut1: Time<Ut1>) -> Self {
        Self(
            (GreenwichMeanSiderealTime::iau2006(tt, ut1).0
                + EquationOfTheEquinoxes::iau2006a(tt).0)
                .mod_two_pi(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, ApproxEq)]
pub struct EquationOfTheEquinoxes(pub Angle);

impl EquationOfTheEquinoxes {
    pub fn iau1994(time: Time<Tdb>) -> Self {
        let t = time.centuries_since_j2000();
        let om = (Angle::arcseconds(poly_array(t, &[450160.280, -482890.539, 7.455, 0.008]))
            + Angle::new((-5.0 * t).rem_euclid(1.0) * TAU))
        .mod_two_pi();
        let Nutation { dpsi, .. } = Nutation::iau1980(time);
        let eps0 = MeanObliquity::iau1980(time.with_scale(Tt));
        Self(
            eps0.0.cos() * dpsi
                + Angle::arcseconds(0.00264 * om.sin() + 0.000063 * (om + om).sin()),
        )
    }

    pub fn iau2000a(time: Time<Tt>) -> Self {
        let PrecessionCorrectionsIau2000 { depspr, .. } = PrecessionCorrectionsIau2000::new(time);
        let epsa = MeanObliquity::iau1980(time).0 + depspr;
        let Nutation { dpsi, .. } = Nutation::iau2000a(time.with_scale(Tdb));
        Self::iau2000(time, epsa, dpsi)
    }

    pub fn iau2000b(time: Time<Tt>) -> Self {
        let PrecessionCorrectionsIau2000 { depspr, .. } = PrecessionCorrectionsIau2000::new(time);
        let epsa = MeanObliquity::iau1980(time).0 + depspr;
        let Nutation { dpsi, .. } = Nutation::iau2000b(time.with_scale(Tdb));
        Self::iau2000(time, epsa, dpsi)
    }

    pub fn iau2006a(time: Time<Tt>) -> Self {
        let epsa = MeanObliquity::iau2006(time);
        let Nutation { dpsi, .. } = Nutation::iau2006a(time.with_scale(Tdb));
        Self::iau2000(time, epsa.0, dpsi)
    }

    pub fn iau2000(time: Time<Tt>, epsa: Angle, dpsi: Angle) -> Self {
        Self(epsa.cos() * dpsi + Self::complimentary_terms_iau2000(time))
    }

    fn complimentary_terms_iau2000(time: Time<Tt>) -> Angle {
        let t = time.centuries_since_j2000();

        let fa = [
            l_iers03(t).as_f64(),
            lp_iers03(t).as_f64(),
            f_iers03(t).as_f64(),
            d_iers03(t).as_f64(),
            omega_iers03(t).as_f64(),
            venus_l_iers03(t).as_f64(),
            earth_l_iers03(t).as_f64(),
            pa_iers03(t).as_f64(),
        ];

        let s0 = complementary_terms::E0.iter().rev().fold(0.0, |s0, term| {
            let a = zip(&term.nfa, &fa).fold(0.0, |a, (&nfa, &fa)| a + nfa as f64 * fa);
            let (sa, ca) = a.sin_cos();
            s0 + term.s * sa + term.c * ca
        });

        let s1 = complementary_terms::E1.iter().rev().fold(0.0, |s1, term| {
            let a = zip(&term.nfa, &fa).fold(0.0, |a, (&nfa, &fa)| a + nfa as f64 * fa);
            let (sa, ca) = a.sin_cos();
            s1 + term.s * sa + term.c * ca
        });

        Angle::arcseconds(s0 + s1 * t)
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_earth_rotation_angle_iau2000() {
        let time = Time::from_two_part_julian_date(Ut1, 2400000.5, 54388.0);
        let exp = EarthRotationAngle(Angle::new(0.402_283_724_002_815_8));
        let act = EarthRotationAngle::iau2000(time);
        assert_approx_eq!(act, exp, rtol <= 1e-12);
    }

    #[test]
    fn test_greenwich_apparent_sidereal_time_iau1994() {
        let ut1 = Time::from_two_part_julian_date(Ut1, 2400000.5, 53736.0);
        let exp = GreenwichApparentSiderealTime(Angle::new(1.754_166_136_020_645_3));
        let act = Gast::iau1994(ut1);
        assert_approx_eq!(act, exp, rtol <= 1e-12);
    }

    #[test]
    fn test_greenwich_apparent_sidereal_time_iau2000a() {
        let tt = Time::from_two_part_julian_date(Tt, 2400000.5, 53736.0);
        let ut1 = Time::from_two_part_julian_date(Ut1, 2400000.5, 53736.0);
        let exp = GreenwichApparentSiderealTime(Angle::new(1.754_166_138_018_281_4));
        let act = Gast::iau2000a(tt, ut1);
        assert_approx_eq!(act, exp, rtol <= 1e-12);
    }

    #[test]
    fn test_greenwich_apparent_sidereal_time_iau2000b() {
        let tt = Time::from_two_part_julian_date(Tt, 2400000.5, 53736.0);
        let ut1 = Time::from_two_part_julian_date(Ut1, 2400000.5, 53736.0);
        let exp = GreenwichApparentSiderealTime(Angle::new(1.754_166_136_510_680_7));
        let act = Gast::iau2000b(tt, ut1);
        assert_approx_eq!(act, exp, rtol <= 1e-12);
    }

    #[test]
    fn test_greenwich_mean_sidereal_time_iau1982() {
        let ut1 = Time::from_two_part_julian_date(Ut1, 2400000.5, 53736.0);
        let exp = GreenwichMeanSiderealTime(Angle::new(1.754_174_981_860_675));
        let act = Gmst::iau1982(ut1);
        assert_approx_eq!(act, exp, rtol <= 1e-12);
    }

    #[test]
    fn test_greenwich_mean_sidereal_time_iau2000() {
        let tt = Time::from_two_part_julian_date(Tt, 2400000.5, 53736.0);
        let ut1 = Time::from_two_part_julian_date(Ut1, 2400000.5, 53736.0);
        let exp = GreenwichMeanSiderealTime(Angle::new(1.754_174_972_210_740_7));
        let act = Gmst::iau2000(tt, ut1);
        assert_approx_eq!(act, exp, rtol <= 1e-12);
    }

    #[test]
    fn test_greenwich_mean_sidereal_time_iau2006() {
        let tt = Time::from_two_part_julian_date(Tt, 2400000.5, 53736.0);
        let ut1 = Time::from_two_part_julian_date(Ut1, 2400000.5, 53736.0);
        let exp = GreenwichMeanSiderealTime(Angle::new(1.754_174_971_870_091_2));
        let act = Gmst::iau2006(tt, ut1);
        assert_approx_eq!(act, exp, rtol <= 1e-12);
    }

    #[test]
    fn test_equation_of_the_equinoxes_iau1994() {
        let time = Time::from_two_part_julian_date(Tdb, 2400000.5, 41234.0);
        let exp = EquationOfTheEquinoxes(Angle::new(5.357_758_254_609_257e-5));
        let act = EquationOfTheEquinoxes::iau1994(time);
        assert_approx_eq!(act, exp, atol <= 1e-17);
    }

    #[test]
    fn test_equation_of_the_equinoxes_iau2000a() {
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 53736.0);
        let exp = EquationOfTheEquinoxes(Angle::new(-8.834_192_459_222_587e-6));
        let act = EquationOfTheEquinoxes::iau2000a(time);
        assert_approx_eq!(act, exp, atol <= 1e-18);
    }

    #[test]
    fn test_equation_of_the_equinoxes_iau2000b() {
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 53736.0);
        let exp = EquationOfTheEquinoxes(Angle::new(-8.835_700_060_003_032e-6));
        let act = EquationOfTheEquinoxes::iau2000b(time);
        assert_approx_eq!(act, exp, atol <= 1e-18);
    }

    #[test]
    fn test_equation_of_the_equinoxes_iau2000() {
        let epsa = Angle::new(0.409_078_976_335_651);
        let dpsi = Angle::new(-9.630_909_107_115_582e-6);
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 53736.0);
        let exp = EquationOfTheEquinoxes(Angle::new(-8.834_193_235_367_966e-6));
        let act = EquationOfTheEquinoxes::iau2000(time, epsa, dpsi);
        assert_approx_eq!(act, exp, atol <= 1e-20);
    }

    #[test]
    fn test_equation_of_the_equinoxes_complimentary_terms() {
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 53736.0);
        let exp = Angle::new(2.046_085_004_885_125e-9);
        let act = EquationOfTheEquinoxes::complimentary_terms_iau2000(time);
        assert_approx_eq!(act, exp, atol <= 1e-20);
    }

    #[test]
    fn test_greenwich_apparent_sidereal_time_iau2006a() {
        let tt = Time::from_two_part_julian_date(Tt, 2400000.5, 53736.0);
        let ut1 = Time::from_two_part_julian_date(Ut1, 2400000.5, 53736.0);
        let exp = GreenwichApparentSiderealTime(Angle::new(1.754_166_137_675_019_2));
        let act = Gast::iau2006a(tt, ut1);
        assert_approx_eq!(act, exp, rtol <= 1e-12);
    }

    #[test]
    fn test_equation_of_the_equinoxes_iau2006a() {
        // ERFA ee06a reference value. ERFA computes this via gst06a - gmst06 which
        // takes a different computational path (through the full NPB matrix) than our
        // direct epsa.cos() * dpsi + complementary_terms. The ~8e-13 rad difference
        // (~0.2 μas) is expected.
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 53736.0);
        let exp = EquationOfTheEquinoxes(Angle::new(-8.834_195_072_043_79e-6));
        let act = EquationOfTheEquinoxes::iau2006a(time);
        assert_approx_eq!(act, exp, atol <= 1e-12);
    }
}
