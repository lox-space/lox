// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::iter::zip;

use glam::{DMat3, DVec3};
use lox_core::{coords::Cartesian, f64::consts::SECONDS_PER_JULIAN_YEAR};
use lox_time::{Time, julian_dates::JulianDate, time_scales::Tdb};
use lox_units::{ASTRONOMICAL_UNIT, Distance, DistanceUnits, SPEED_OF_LIGHT};

mod coefficients;

const AM11: f64 = 1.0;
const AM12: f64 = 0.000000211284;
const AM13: f64 = -0.000000091603;
const AM21: f64 = -0.000000230286;
const AM22: f64 = 0.917482137087;
const AM23: f64 = -0.397776982902;
const AM31: f64 = 0.0;
const AM32: f64 = 0.397776982902;
const AM33: f64 = 0.917482137087;

const ECLIPTIC_TO_ICRF: DMat3 =
    DMat3::from_cols_array(&[AM11, AM21, AM31, AM12, AM22, AM32, AM13, AM23, AM33]);

pub struct EarthState {
    pub heliocentric: Cartesian,
    pub barycentric: Cartesian,
}

pub fn earth_state(time: Time<Tdb>) -> EarthState {
    let t = time.years_since_j2000();
    let t2 = t * t;

    let heliocentric = HELIOCENTRIC_COEFFS.evaluate(t, t2);
    let baricentric = heliocentric + BARICENTRIC_COEFFS.evaluate(t, t2);

    EarthState {
        heliocentric: Cartesian::from_vecs(
            ECLIPTIC_TO_ICRF * heliocentric.position(),
            ECLIPTIC_TO_ICRF * heliocentric.velocity(),
        ),
        barycentric: Cartesian::from_vecs(
            ECLIPTIC_TO_ICRF * baricentric.position(),
            ECLIPTIC_TO_ICRF * baricentric.velocity(),
        ),
    }
}

// TODO: Move to constants module
// Schwarzschild radius of the Sun (au)
// 2 * 1.32712440041e20 / (2.99792458e8)^2 / 1.49597870700e11
const SRS: Distance = Distance::astronomical_units(1.97412574336e-8);

fn aberration(pnat: DVec3, v: DVec3, s: Distance, bm1: f64) -> DVec3 {
    let mut p: [f64; 3] = [0.0; 3];
    let pdv = pnat.dot(v);
    let w1 = 1.0 + pdv / (1.0 + bm1);
    let w2 = SRS.as_f64() / s.as_f64();
    let r2 = zip(pnat.to_array(), v.to_array())
        .enumerate()
        .fold(0.0, |r2, (idx, (pnat, v))| {
            let w = pnat * bm1 + w1 * v + w2 * (v - pdv * pnat);
            p[idx] = w;
            r2 + w * w
        });
    let r = r2.sqrt();
    DVec3::from(p) / r
}

pub fn apparent_sun_position(time: Time<Tdb>) -> DVec3 {
    let s = earth_state(time);
    let pe = s.heliocentric.position();
    let vb = s.barycentric.velocity() / SPEED_OF_LIGHT;
    let dsun = pe.length().m();
    let lorentz_inv = (1.0 - vb.powf(2.0).element_sum()).powf(0.5);
    let proper = aberration(pe / dsun.as_f64(), -vb, dsun, lorentz_inv);
    -dsun.as_f64() * proper
}

struct Coeffs {
    c0: [&'static [f64]; 3],
    c1: [&'static [f64]; 3],
    c2: [&'static [f64]; 3],
}

impl Coeffs {
    pub fn evaluate(&self, t: f64, t2: f64) -> Cartesian {
        Self::eval(
            self.c0,
            |(mut xyz, mut xyzd): (f64, f64), [a, b, c]: &[f64; 3]| {
                let p = b + c * t;
                xyz += a * p.cos();
                xyzd -= a * c * p.sin();
                (xyz, xyzd)
            },
        ) + Self::eval(self.c1, |(mut xyz, mut xyzd), [a, b, c]| {
            let ct = c * t;
            let p = b + ct;
            let cp = p.cos();
            xyz += a * t * cp;
            xyzd += a * (cp - ct * p.sin());
            (xyz, xyzd)
        }) + Self::eval(self.c2, |(mut xyz, mut xyzd), [a, b, c]| {
            let ct = c * t;
            let p = b + ct;
            let cp = p.cos();
            xyz += a * t2 * cp;
            xyzd += a * t * (2.0 * cp - ct * p.sin());
            (xyz, xyzd)
        })
    }

    fn eval<F>(coeffs: [&[f64]; 3], f: F) -> Cartesian
    where
        F: Fn((f64, f64), &[f64; 3]) -> (f64, f64),
    {
        if let (x, []) = coeffs[0].as_chunks::<3>()
            && let (y, []) = coeffs[1].as_chunks::<3>()
            && let (z, []) = coeffs[2].as_chunks::<3>()
        {
            let (x, vx) = x.iter().fold((0.0, 0.0), &f);
            let (y, vy) = y.iter().fold((0.0, 0.0), &f);
            let (z, vz) = z.iter().fold((0.0, 0.0), &f);

            // Scale from AU to m
            let pos = DVec3::new(x, y, z) * ASTRONOMICAL_UNIT;
            // Scale from AU/year to m/s
            let vel = DVec3::new(vx, vy, vz) * ASTRONOMICAL_UNIT / SECONDS_PER_JULIAN_YEAR;
            Cartesian::from_vecs(pos, vel)
        } else {
            unreachable!()
        }
    }
}

const HELIOCENTRIC_COEFFS: Coeffs = Coeffs {
    c0: [&coefficients::E0X, &coefficients::E0Y, &coefficients::E0Z],
    c1: [&coefficients::E1X, &coefficients::E1Y, &coefficients::E1Z],
    c2: [&coefficients::E2X, &coefficients::E2Y, &coefficients::E2Z],
};

const BARICENTRIC_COEFFS: Coeffs = Coeffs {
    c0: [&coefficients::S0X, &coefficients::S0Y, &coefficients::S0Z],
    c1: [&coefficients::S1X, &coefficients::S1Y, &coefficients::S1Z],
    c2: [&coefficients::S2X, &coefficients::S2Y, &coefficients::S2Z],
};

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;
    use lox_units::{DistanceUnits, VelocityUnits};

    use super::*;

    #[test]
    fn test_earth_position() {
        let tdb = Time::from_two_part_julian_date(Tdb, 2400000.5, 53411.52501161);

        let helio_exp = Cartesian::new(
            -0.775_723_880_929_770_6.au(),
            0.559_805_224_136_334.au(),
            0.242_699_846_648_168_7.au(),
            -1.091_891_824_147_313_8e-2.aud(),
            -1.247_187_268_440_845e-2.aud(),
            -5.407_569_418_065_039e-3.aud(),
        );

        let bary_exp = Cartesian::new(
            -0.771_410_444_049_111_2.au(),
            0.559_841_206_182_417_2.au(),
            0.242_599_627_772_245_25.au(),
            -1.091_874_268_116_823_3e-2.aud(),
            -1.246_525_461_732_861_6e-2.aud(),
            -5.404_773_180_966_231_5e-3.aud(),
        );

        let s = earth_state(tdb);
        let helio_act = s.heliocentric;
        let bary_act = s.barycentric;
        assert_approx_eq!(helio_act, helio_exp, rtol <= 1e-12);
        assert_approx_eq!(bary_act, bary_exp, rtol <= 1e-12);

        // let geo_act = sun_geocentric(tdb);
        // assert_approx_eq!(geo_act, -helio_exp, rtol <= 1e-12);
    }

    #[test]
    fn test_aberration() {
        let pnat = DVec3::new(
            -0.763_219_685_467_379_5,
            -0.608_694_539_830_603_8,
            -0.21676408580639883,
        );
        let v = DVec3::new(
            2.1044018893653786e-5,
            -8.910_892_330_442_932e-5,
            -3.863_371_479_771_657e-5,
        );
        let s = 0.999_809_213_957_087_9.au();
        let bm1 = 0.999_999_995_062_092_6;

        let act = aberration(pnat, v, s, bm1);
        let exp = DVec3::new(
            -0.763_163_109_421_955_6,
            -0.608_755_308_250_559_1,
            -0.216_792_626_936_847_12,
        );
        assert_approx_eq!(act, exp, rtol <= 1e-12);
    }
}
