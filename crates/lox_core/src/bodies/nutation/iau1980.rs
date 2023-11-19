use std::f64::consts::TAU;

use crate::bodies::nutation::{point1_milliarcsec_to_rad, Coefficients, Nutation};
use crate::math::{arcsec_to_rad, normalize_two_pi};
use crate::time::intervals::TDBJulianCenturiesSinceJ2000;
use crate::types::{Arcsec, Radians};

pub(crate) fn nutation_iau1980(t: TDBJulianCenturiesSinceJ2000) -> Nutation {
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
            let sin = coeff.long_sin_1 + coeff.long_sin_t * t;
            let cos = coeff.obl_cos_1 + coeff.obl_cos_t * t;
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

#[rustfmt::skip]
const COEFFICIENTS: [Coefficients; 106] = [
    Coefficients{ l: 0.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 1.0,	long_sin_1:-171996.0,	long_sin_t:-174.2,	obl_cos_1: 92025.0,	obl_cos_t:  8.9 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 2.0,	long_sin_1:   2062.0,	long_sin_t:   0.2,	obl_cos_1:  -895.0,	obl_cos_t:  0.5 },
    Coefficients{ l: -2.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 1.0,	long_sin_1:     46.0,	long_sin_t:   0.0,	obl_cos_1:   -24.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f:-2.0,	d: 0.0,	om: 0.0,	long_sin_1:     11.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -2.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 2.0,	long_sin_1:     -3.0,	long_sin_t:   0.0,	obl_cos_1:     1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp:-1.0,	f: 0.0,	d:-1.0,	om: 0.0,	long_sin_1:     -3.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp:-2.0,	f: 2.0,	d:-2.0,	om: 1.0,	long_sin_1:     -2.0,	long_sin_t:   0.0,	obl_cos_1:     1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f:-2.0,	d: 0.0,	om: 1.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 2.0,	long_sin_1: -13187.0,	long_sin_t:  -1.6,	obl_cos_1:  5736.0,	obl_cos_t: -3.1 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 0.0,	d: 0.0,	om: 0.0,	long_sin_1:   1426.0,	long_sin_t:  -3.4,	obl_cos_1:    54.0,	obl_cos_t: -0.1 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 2.0,	d:-2.0,	om: 2.0,	long_sin_1:   -517.0,	long_sin_t:   1.2,	obl_cos_1:   224.0,	obl_cos_t: -0.6 },
    Coefficients{ l: 0.0,	lp:-1.0,	f: 2.0,	d:-2.0,	om: 2.0,	long_sin_1:    217.0,	long_sin_t:  -0.5,	obl_cos_1:   -95.0,	obl_cos_t:  0.3 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 1.0,	long_sin_1:    129.0,	long_sin_t:   0.1,	obl_cos_1:   -70.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 0.0,	d:-2.0,	om: 0.0,	long_sin_1:     48.0,	long_sin_t:   0.0,	obl_cos_1:     1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 0.0,	long_sin_1:    -22.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 2.0,	f: 0.0,	d: 0.0,	om: 0.0,	long_sin_1:     17.0,	long_sin_t:  -0.1,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 0.0,	d: 0.0,	om: 1.0,	long_sin_1:    -15.0,	long_sin_t:   0.0,	obl_cos_1:     9.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 2.0,	f: 2.0,	d:-2.0,	om: 2.0,	long_sin_1:    -16.0,	long_sin_t:   0.1,	obl_cos_1:     7.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp:-1.0,	f: 0.0,	d: 0.0,	om: 1.0,	long_sin_1:    -12.0,	long_sin_t:   0.0,	obl_cos_1:     6.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -2.0,	lp: 0.0,	f: 0.0,	d: 2.0,	om: 1.0,	long_sin_1:     -6.0,	long_sin_t:   0.0,	obl_cos_1:     3.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp:-1.0,	f: 2.0,	d:-2.0,	om: 1.0,	long_sin_1:     -5.0,	long_sin_t:   0.0,	obl_cos_1:     3.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 0.0,	d:-2.0,	om: 1.0,	long_sin_1:      4.0,	long_sin_t:   0.0,	obl_cos_1:    -2.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 2.0,	d:-2.0,	om: 1.0,	long_sin_1:      4.0,	long_sin_t:   0.0,	obl_cos_1:    -2.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d:-1.0,	om: 0.0,	long_sin_1:     -4.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 1.0,	f: 0.0,	d:-2.0,	om: 0.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f:-2.0,	d: 2.0,	om: 1.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f:-2.0,	d: 2.0,	om: 0.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 0.0,	d: 0.0,	om: 2.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 0.0,	d: 1.0,	om: 1.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 2.0,	d:-2.0,	om: 0.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 2.0,	long_sin_1:  -2274.0,	long_sin_t:  -0.2,	obl_cos_1:   977.0,	obl_cos_t: -0.5 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 0.0,	long_sin_1:    712.0,	long_sin_t:   0.1,	obl_cos_1:    -7.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 1.0,	long_sin_1:   -386.0,	long_sin_t:  -0.4,	obl_cos_1:   200.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 2.0,	long_sin_1:   -301.0,	long_sin_t:   0.0,	obl_cos_1:   129.0,	obl_cos_t: -0.1 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d:-2.0,	om: 0.0,	long_sin_1:   -158.0,	long_sin_t:   0.0,	obl_cos_1:    -1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 2.0,	long_sin_1:    123.0,	long_sin_t:   0.0,	obl_cos_1:   -53.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 0.0,	d: 2.0,	om: 0.0,	long_sin_1:     63.0,	long_sin_t:   0.0,	obl_cos_1:    -2.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 1.0,	long_sin_1:     63.0,	long_sin_t:   0.1,	obl_cos_1:   -33.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 1.0,	long_sin_1:    -58.0,	long_sin_t:  -0.1,	obl_cos_1:    32.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 2.0,	long_sin_1:    -59.0,	long_sin_t:   0.0,	obl_cos_1:    26.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 1.0,	long_sin_1:    -51.0,	long_sin_t:   0.0,	obl_cos_1:    27.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 2.0,	long_sin_1:    -38.0,	long_sin_t:   0.0,	obl_cos_1:    16.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 0.0,	long_sin_1:     29.0,	long_sin_t:   0.0,	obl_cos_1:    -1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 2.0,	long_sin_1:     29.0,	long_sin_t:   0.0,	obl_cos_1:   -12.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 2.0,	long_sin_1:    -31.0,	long_sin_t:   0.0,	obl_cos_1:    13.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 0.0,	long_sin_1:     26.0,	long_sin_t:   0.0,	obl_cos_1:    -1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 1.0,	long_sin_1:     21.0,	long_sin_t:   0.0,	obl_cos_1:   -10.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 0.0,	d: 2.0,	om: 1.0,	long_sin_1:     16.0,	long_sin_t:   0.0,	obl_cos_1:    -8.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d:-2.0,	om: 1.0,	long_sin_1:    -13.0,	long_sin_t:   0.0,	obl_cos_1:     7.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 1.0,	long_sin_1:    -10.0,	long_sin_t:   0.0,	obl_cos_1:     5.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 1.0,	f: 0.0,	d:-2.0,	om: 0.0,	long_sin_1:     -7.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 2.0,	d: 0.0,	om: 2.0,	long_sin_1:      7.0,	long_sin_t:   0.0,	obl_cos_1:    -3.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp:-1.0,	f: 2.0,	d: 0.0,	om: 2.0,	long_sin_1:     -7.0,	long_sin_t:   0.0,	obl_cos_1:     3.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 2.0,	long_sin_1:     -8.0,	long_sin_t:   0.0,	obl_cos_1:     3.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d: 2.0,	om: 0.0,	long_sin_1:      6.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 2.0,	long_sin_1:      6.0,	long_sin_t:   0.0,	obl_cos_1:    -3.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 0.0,	d: 2.0,	om: 1.0,	long_sin_1:     -6.0,	long_sin_t:   0.0,	obl_cos_1:     3.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 1.0,	long_sin_1:     -7.0,	long_sin_t:   0.0,	obl_cos_1:     3.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 1.0,	long_sin_1:      6.0,	long_sin_t:   0.0,	obl_cos_1:    -3.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 0.0,	d:-2.0,	om: 1.0,	long_sin_1:     -5.0,	long_sin_t:   0.0,	obl_cos_1:     3.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp:-1.0,	f: 0.0,	d: 0.0,	om: 0.0,	long_sin_1:      5.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 1.0,	long_sin_1:     -5.0,	long_sin_t:   0.0,	obl_cos_1:     3.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 0.0,	d:-2.0,	om: 0.0,	long_sin_1:     -4.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f:-2.0,	d: 0.0,	om: 0.0,	long_sin_1:      4.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 0.0,	d: 1.0,	om: 0.0,	long_sin_1:     -4.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 1.0,	f: 0.0,	d: 0.0,	om: 0.0,	long_sin_1:     -3.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 0.0,	long_sin_1:      3.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp:-1.0,	f: 2.0,	d: 0.0,	om: 2.0,	long_sin_1:     -3.0,	long_sin_t:   0.0,	obl_cos_1:     1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -1.0,	lp:-1.0,	f: 2.0,	d: 2.0,	om: 2.0,	long_sin_1:     -3.0,	long_sin_t:   0.0,	obl_cos_1:     1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -2.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 1.0,	long_sin_1:     -2.0,	long_sin_t:   0.0,	obl_cos_1:     1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 3.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 2.0,	long_sin_1:     -3.0,	long_sin_t:   0.0,	obl_cos_1:     1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp:-1.0,	f: 2.0,	d: 2.0,	om: 2.0,	long_sin_1:     -3.0,	long_sin_t:   0.0,	obl_cos_1:     1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 1.0,	f: 2.0,	d: 0.0,	om: 2.0,	long_sin_1:      2.0,	long_sin_t:   0.0,	obl_cos_1:    -1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 1.0,	long_sin_1:     -2.0,	long_sin_t:   0.0,	obl_cos_1:     1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 1.0,	long_sin_1:      2.0,	long_sin_t:   0.0,	obl_cos_1:    -1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 2.0,	long_sin_1:     -2.0,	long_sin_t:   0.0,	obl_cos_1:     1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 3.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 0.0,	long_sin_1:      2.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d: 1.0,	om: 2.0,	long_sin_1:      2.0,	long_sin_t:   0.0,	obl_cos_1:    -1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 2.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:    -1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d:-4.0,	om: 0.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -2.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 2.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:    -1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 2.0,	d: 4.0,	om: 2.0,	long_sin_1:     -2.0,	long_sin_t:   0.0,	obl_cos_1:     1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 0.0,	d:-4.0,	om: 0.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 1.0,	f: 2.0,	d:-2.0,	om: 2.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:    -1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 1.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -2.0,	lp: 0.0,	f: 2.0,	d: 4.0,	om: 2.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 4.0,	d: 0.0,	om: 2.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp:-1.0,	f: 0.0,	d:-2.0,	om: 0.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 1.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:    -1.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 2.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d: 2.0,	om: 1.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 4.0,	d:-2.0,	om: 2.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 3.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 2.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 0.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 2.0,	d: 0.0,	om: 1.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: -1.0,	lp:-1.0,	f: 0.0,	d: 2.0,	om: 1.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f:-2.0,	d: 0.0,	om: 1.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d:-1.0,	om: 2.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 0.0,	d: 2.0,	om: 0.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f:-2.0,	d:-2.0,	om: 0.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp:-1.0,	f: 2.0,	d: 0.0,	om: 1.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 1.0,	f: 0.0,	d:-2.0,	om: 1.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f:-2.0,	d: 2.0,	om: 0.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 0.0,	d: 2.0,	om: 0.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d: 4.0,	om: 2.0,	long_sin_1:     -1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 0.0,	d: 1.0,	om: 0.0,	long_sin_1:      1.0,	long_sin_t:   0.0,	obl_cos_1:     0.0,	obl_cos_t:  0.0 },
];

#[cfg(test)]
/// All fixtures and assertion values were generated using the ERFA C library unless otherwise
/// stated.
mod tests {
    use float_eq::assert_float_eq;

    use crate::time::intervals::TDBJulianCenturiesSinceJ2000;

    use super::nutation_iau1980;

    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn test_nutation_iau1980_jd0() {
        let jd0: TDBJulianCenturiesSinceJ2000 = -67.11964407939767;
        let actual = nutation_iau1980(jd0);
        assert_float_eq!(0.00000693404778664026, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(0.00004131255061383108, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau1980_j2000() {
        let j2000: TDBJulianCenturiesSinceJ2000 = 0.0;
        let actual = nutation_iau1980(j2000);
        assert_float_eq!(-0.00006750247617532478, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(-0.00002799221238377013, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau1980_j2100() {
        let j2100: TDBJulianCenturiesSinceJ2000 = 1.0;
        let actual = nutation_iau1980(j2100);
        assert_float_eq!(0.00001584138015187132, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(0.00004158958379918889, actual.obliquity, rel <= TOLERANCE);
    }
}
