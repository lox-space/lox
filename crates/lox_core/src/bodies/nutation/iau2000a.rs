/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::f64::consts::TAU;

use crate::bodies::nutation::{point1_milliarcsec_to_rad, Nutation};
use crate::math::{arcsec_to_rad, normalize_two_pi};
use crate::time::intervals::TDBJulianCenturiesSinceJ2000;
use crate::types::{Arcsec, Radians};

mod luni_solar;
mod planetary;

struct LuniSolarCoefficients {
    /// Coefficients of l, l', F, D and Ω.
    l: f64,
    lp: f64,
    f: f64,
    d: f64,
    om: f64,

    /// Longitude coefficients.
    sin_psi: f64,
    sin_psi_t: f64,
    cos_psi: f64,

    /// Obliquity coefficients.
    cos_eps: f64,
    cos_eps_t: f64,
    sin_eps: f64,
}

struct PlanetaryCoefficients {
    /// Coefficients of l, F, D and Ω.
    l: f64,
    f: f64,
    d: f64,
    om: f64,

    /// Planetary longitude coefficients.
    mercury: f64,
    venus: f64,
    earth: f64,
    mars: f64,
    jupiter: f64,
    saturn: f64,
    uranus: f64,
    neptune: f64,

    /// Coefficient of general precession.
    pa: f64,

    /// Longitude coefficients.
    sin_psi: f64,
    cos_psi: f64,

    /// Obliquity coefficients.
    sin_eps: f64,
    cos_eps: f64,
}

pub(crate) fn nutation_iau2000a(t: TDBJulianCenturiesSinceJ2000) -> Nutation {
    let l = l(t);
    let lp = l_prime(t);
    let f = f(t);
    let d = d(t);
    let om = omega(t);

    let mut nutation = COEFFICIENTS
        .iter()
        // The coefficients are given by descending magnitude but folded by ascending
        // magnitude to minimise floating-point errors.
        .rev()
        .fold(Nutation::default(), |mut nut, coeff| {
            // Form argument for current term.
            let arg = coeff.l * l + coeff.lp * lp + coeff.f * f + coeff.d * d + coeff.om * om;

            // Accumulate current term.
            let sin = coeff.sin_psi + coeff.sin_psi_t * t;
            let cos = coeff.cos_eps + coeff.cos_eps_t * t;
            nut.longitude += sin * arg.sin();
            nut.obliquity += cos * arg.cos();

            nut
        });

    nutation.longitude = point1_milliarcsec_to_rad(nutation.longitude);
    nutation.obliquity = point1_milliarcsec_to_rad(nutation.obliquity);

    nutation
}

/// `l`, the mean longitude of the Moon measured from the mean position of the perigee,
/// normalized to the range [0, 2π).
fn l(t: TDBJulianCenturiesSinceJ2000) -> Radians {
    let l_poly: Arcsec = fast_polynomial::poly_array(t, &[485866.733, 715922.633, 31.31, 0.064]);
    let l_poly: Radians = arcsec_to_rad(l_poly);
    let l_non_normal = l_poly + (1325.0 * t % 1.0) * TAU;
    normalize_two_pi(l_non_normal, 0.0)
}

/// `l'`, the mean longitude of the Sun measured from the mean position of the perigee,
/// normalized to the range [0, 2π).
fn l_prime(t: TDBJulianCenturiesSinceJ2000) -> Radians {
    let lp_poly: Arcsec =
        fast_polynomial::poly_array(t, &[1287099.804, 1292581.224, -0.577, -0.012]);
    let lp_poly: Radians = arcsec_to_rad(lp_poly);
    let lp_non_normal = lp_poly + (99.0 * t % 1.0) * TAU;
    normalize_two_pi(lp_non_normal, 0.0)
}

/// `F`, the mean longitude of the Moon minus the mean longitude of the Moon's ascending node,
/// normalized to the range [0, 2π).
fn f(t: TDBJulianCenturiesSinceJ2000) -> Radians {
    let f_poly: Arcsec = fast_polynomial::poly_array(t, &[335778.877, 295263.137, -13.257, 0.011]);
    let f_poly: Radians = arcsec_to_rad(f_poly);
    let f_non_normal = f_poly + (1342.0 * t % 1.0) * TAU;
    normalize_two_pi(f_non_normal, 0.0)
}

/// `D`, the mean elongation of the Moon from the Sun, normalized to the range [0, 2π).
fn d(t: TDBJulianCenturiesSinceJ2000) -> Radians {
    let d_poly: Arcsec = fast_polynomial::poly_array(t, &[1072261.307, 1105601.328, -6.891, 0.019]);
    let d: Radians = arcsec_to_rad(d_poly);
    let d_non_normal = d + (1236.0 * t % 1.0) * TAU;
    normalize_two_pi(d_non_normal, 0.0)
}

/// `Ω`, the longitude of the mean ascending node of the lunar orbit on the ecliptic, measured from
/// the mean equinox of date.
fn omega(t: TDBJulianCenturiesSinceJ2000) -> Radians {
    let om_poly: Arcsec = fast_polynomial::poly_array(t, &[450160.280, -482890.539, 7.455, 0.008]);
    let om_poly: Radians = arcsec_to_rad(om_poly);
    let om_non_normal = om_poly + (-5.0 * t % 1.0) * TAU;
    normalize_two_pi(om_non_normal, 0.0)
}

#[cfg(test)]
/// All fixtures and assertion values were generated using the ERFA C library unless otherwise
/// stated.
mod tests {
    use float_eq::assert_float_eq;

    use crate::time::intervals::TDBJulianCenturiesSinceJ2000;

    use super::nutation_iau2000a;

    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn test_nutation_iau1980_jd0() {
        let jd0: TDBJulianCenturiesSinceJ2000 = -67.11964407939767;
        let actual = nutation_iau2000a(jd0);
        assert_float_eq!(0.00000693404778664026, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(0.00004131255061383108, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau1980_j2000() {
        let j2000: TDBJulianCenturiesSinceJ2000 = 0.0;
        let actual = nutation_iau2000a(j2000);
        assert_float_eq!(-0.00006750247617532478, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(-0.00002799221238377013, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau1980_j2100() {
        let j2100: TDBJulianCenturiesSinceJ2000 = 1.0;
        let actual = nutation_iau2000a(j2100);
        assert_float_eq!(0.00001584138015187132, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(0.00004158958379918889, actual.obliquity, rel <= TOLERANCE);
    }
}
