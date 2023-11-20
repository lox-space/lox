/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::f64::consts::TAU;

use crate::bodies::fundamental::MeanAnomaly;
use crate::bodies::nutation::{point1_milliarcsec_to_rad, Nutation};
use crate::bodies::Moon;
use crate::math::{arcsec_to_rad, arcsec_to_rad_two_pi, normalize_two_pi};
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
    luni_solar_nutation(t) + planetary_nutation(t)
}

fn luni_solar_nutation(t: TDBJulianCenturiesSinceJ2000) -> Nutation {
    // Delaunay arguments.
    let l = Moon.mean_anomaly(t);
    let lp = l_prime(t);
    let f = Moon.mean_longitude_minus_ascending_node_mean_longitude(t);
    let d = d(t);
    let om = Moon.ascending_node_mean_longitude(t);

    let mut nutation = luni_solar::COEFFICIENTS
        .iter()
        // The coefficients are given by descending magnitude but folded by ascending
        // magnitude to minimise floating-point error.
        .rev()
        .fold(Nutation::default(), |mut nut, coeff| {
            // Form argument for current term.
            let arg =
                (coeff.l * l + coeff.lp * lp + coeff.f * f + coeff.d * d + coeff.om * om) % TAU;

            // Accumulate current term.
            let sin_arg = arg.sin();
            let cos_arg = arg.cos();
            nut.longitude +=
                (coeff.sin_psi + coeff.sin_psi_t * t) * sin_arg + coeff.cos_psi * cos_arg;
            nut.obliquity +=
                (coeff.cos_eps + coeff.cos_eps_t * t) * cos_arg + coeff.sin_eps * sin_arg;

            nut
        });

    nutation.longitude = point1_milliarcsec_to_rad(nutation.longitude);
    nutation.obliquity = point1_milliarcsec_to_rad(nutation.obliquity);

    nutation
}

fn planetary_nutation(t: TDBJulianCenturiesSinceJ2000) -> Nutation {
    Nutation {
        longitude: 0.0,
        obliquity: 0.0,
    }
}

/// `l'`, the mean anomaly of the Sun (MHB2000).
fn l_prime(t: TDBJulianCenturiesSinceJ2000) -> Radians {
    let lp: Arcsec = fast_polynomial::poly_array(
        t,
        &[
            1287104.79305,
            129596581.0481,
            -0.5532,
            0.000136,
            -0.00001149,
        ],
    );
    arcsec_to_rad_two_pi(lp)
}

/// `D`, the mean elongation of the Moon from the Sun (MHB2000).
fn d(t: TDBJulianCenturiesSinceJ2000) -> Radians {
    let d: Arcsec = fast_polynomial::poly_array(
        t,
        &[
            1072260.70369,
            1602961601.2090,
            -6.3706,
            0.006593,
            -0.00003169,
        ],
    );
    arcsec_to_rad_two_pi(d)
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
