use std::f64::consts::TAU;

use lox_math::math::{arcsec_to_rad, normalize_two_pi};
use lox_math::types::units::{Arcseconds, JulianCenturies, Radians};

use crate::nutation::{Nutation, point1_milliarcsec_to_rad};

struct Coefficients {
    /// Coefficients of l, l', F, D and Ω.
    l: f64,
    lp: f64,
    f: f64,
    d: f64,
    om: f64,

    /// Coefficients of longitude, ψ.
    sin_psi: f64,
    sin_psi_t: f64,

    /// Coefficients of obliquity, ε.
    cos_eps: f64,
    cos_eps_t: f64,
}

pub(crate) fn nutation_iau1980(centuries_since_j2000_tdb: JulianCenturies) -> Nutation {
    let l = l(centuries_since_j2000_tdb);
    let lp = lp(centuries_since_j2000_tdb);
    let f = f(centuries_since_j2000_tdb);
    let d = d(centuries_since_j2000_tdb);
    let om = omega(centuries_since_j2000_tdb);

    let mut nutation = COEFFICIENTS
        .iter()
        // The coefficients are given by descending magnitude but folded by ascending
        // magnitude to minimise floating-point error.
        .rev()
        .fold(Nutation::default(), |mut nut, coeff| {
            // Form argument for current term.
            let arg = coeff.l * l + coeff.lp * lp + coeff.f * f + coeff.d * d + coeff.om * om;

            // Accumulate current term.
            let sin = coeff.sin_psi + coeff.sin_psi_t * centuries_since_j2000_tdb;
            let cos = coeff.cos_eps + coeff.cos_eps_t * centuries_since_j2000_tdb;
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
fn l(centuries_since_j2000_tdb: JulianCenturies) -> Radians {
    let l_poly: Arcseconds = fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[485866.733, 715922.633, 31.31, 0.064],
    );
    let l_poly: Radians = arcsec_to_rad(l_poly);
    let l_non_normal = l_poly + (1325.0 * centuries_since_j2000_tdb % 1.0) * TAU;
    normalize_two_pi(l_non_normal, 0.0)
}

/// `l'`, the mean longitude of the Sun measured from the mean position of the perigee,
/// normalized to the range [0, 2π).
fn lp(centuries_since_j2000_tdb: JulianCenturies) -> Radians {
    let lp_poly: Arcseconds = fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[1287099.804, 1292581.224, -0.577, -0.012],
    );
    let lp_poly: Radians = arcsec_to_rad(lp_poly);
    let lp_non_normal = lp_poly + (99.0 * centuries_since_j2000_tdb % 1.0) * TAU;
    normalize_two_pi(lp_non_normal, 0.0)
}

/// `F`, the mean longitude of the Moon minus the mean longitude of the Moon's ascending node,
/// normalized to the range [0, 2π).
fn f(centuries_since_j2000_tdb: JulianCenturies) -> Radians {
    let f_poly: Arcseconds = fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[335778.877, 295263.137, -13.257, 0.011],
    );
    let f_poly: Radians = arcsec_to_rad(f_poly);
    let f_non_normal = f_poly + (1342.0 * centuries_since_j2000_tdb % 1.0) * TAU;
    normalize_two_pi(f_non_normal, 0.0)
}

/// `D`, the mean elongation of the Moon from the Sun, normalized to the range [0, 2π).
fn d(centuries_since_j2000_tdb: JulianCenturies) -> Radians {
    let d_poly: Arcseconds = fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[1072261.307, 1105601.328, -6.891, 0.019],
    );
    let d: Radians = arcsec_to_rad(d_poly);
    let d_non_normal = d + (1236.0 * centuries_since_j2000_tdb % 1.0) * TAU;
    normalize_two_pi(d_non_normal, 0.0)
}

/// `Ω`, the longitude of the mean ascending node of the lunar orbit on the ecliptic, measured from
/// the mean equinox of date.
fn omega(centuries_since_j2000_tdb: JulianCenturies) -> Radians {
    let om_poly: Arcseconds = fast_polynomial::poly_array(
        centuries_since_j2000_tdb,
        &[450160.280, -482890.539, 7.455, 0.008],
    );
    let om_poly: Radians = arcsec_to_rad(om_poly);
    let om_non_normal = om_poly + (-5.0 * centuries_since_j2000_tdb % 1.0) * TAU;
    normalize_two_pi(om_non_normal, 0.0)
}

#[rustfmt::skip]
// @formatter:off (sometimes RustRover ignores rustfmt::skip)
const COEFFICIENTS: [Coefficients; 106] = [
    Coefficients{ l: 0.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 1.0,	sin_psi:-171996.0,	sin_psi_t:-174.2,	cos_eps: 92025.0,	cos_eps_t:  8.9 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 2.0,	sin_psi:   2062.0,	sin_psi_t:   0.2,	cos_eps:  -895.0,	cos_eps_t:  0.5 },
    Coefficients{ l: -2.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 1.0,	sin_psi:     46.0,	sin_psi_t:   0.0,	cos_eps:   -24.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f:-2.0,	d: 0.0,	om: 0.0,	sin_psi:     11.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -2.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 2.0,	sin_psi:     -3.0,	sin_psi_t:   0.0,	cos_eps:     1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp:-1.0,	f: 0.0,	d:-1.0,	om: 0.0,	sin_psi:     -3.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp:-2.0,	f: 2.0,	d:-2.0,	om: 1.0,	sin_psi:     -2.0,	sin_psi_t:   0.0,	cos_eps:     1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f:-2.0,	d: 0.0,	om: 1.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 2.0,	sin_psi: -13187.0,	sin_psi_t:  -1.6,	cos_eps:  5736.0,	cos_eps_t: -3.1 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 0.0,	d: 0.0,	om: 0.0,	sin_psi:   1426.0,	sin_psi_t:  -3.4,	cos_eps:    54.0,	cos_eps_t: -0.1 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 2.0,	d:-2.0,	om: 2.0,	sin_psi:   -517.0,	sin_psi_t:   1.2,	cos_eps:   224.0,	cos_eps_t: -0.6 },
    Coefficients{ l: 0.0,	lp:-1.0,	f: 2.0,	d:-2.0,	om: 2.0,	sin_psi:    217.0,	sin_psi_t:  -0.5,	cos_eps:   -95.0,	cos_eps_t:  0.3 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 1.0,	sin_psi:    129.0,	sin_psi_t:   0.1,	cos_eps:   -70.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 0.0,	d:-2.0,	om: 0.0,	sin_psi:     48.0,	sin_psi_t:   0.0,	cos_eps:     1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 0.0,	sin_psi:    -22.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 2.0,	f: 0.0,	d: 0.0,	om: 0.0,	sin_psi:     17.0,	sin_psi_t:  -0.1,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 0.0,	d: 0.0,	om: 1.0,	sin_psi:    -15.0,	sin_psi_t:   0.0,	cos_eps:     9.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 2.0,	f: 2.0,	d:-2.0,	om: 2.0,	sin_psi:    -16.0,	sin_psi_t:   0.1,	cos_eps:     7.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp:-1.0,	f: 0.0,	d: 0.0,	om: 1.0,	sin_psi:    -12.0,	sin_psi_t:   0.0,	cos_eps:     6.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -2.0,	lp: 0.0,	f: 0.0,	d: 2.0,	om: 1.0,	sin_psi:     -6.0,	sin_psi_t:   0.0,	cos_eps:     3.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp:-1.0,	f: 2.0,	d:-2.0,	om: 1.0,	sin_psi:     -5.0,	sin_psi_t:   0.0,	cos_eps:     3.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 0.0,	d:-2.0,	om: 1.0,	sin_psi:      4.0,	sin_psi_t:   0.0,	cos_eps:    -2.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 2.0,	d:-2.0,	om: 1.0,	sin_psi:      4.0,	sin_psi_t:   0.0,	cos_eps:    -2.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d:-1.0,	om: 0.0,	sin_psi:     -4.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 1.0,	f: 0.0,	d:-2.0,	om: 0.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f:-2.0,	d: 2.0,	om: 1.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f:-2.0,	d: 2.0,	om: 0.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 0.0,	d: 0.0,	om: 2.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 0.0,	d: 1.0,	om: 1.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 2.0,	d:-2.0,	om: 0.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 2.0,	sin_psi:  -2274.0,	sin_psi_t:  -0.2,	cos_eps:   977.0,	cos_eps_t: -0.5 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 0.0,	sin_psi:    712.0,	sin_psi_t:   0.1,	cos_eps:    -7.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 1.0,	sin_psi:   -386.0,	sin_psi_t:  -0.4,	cos_eps:   200.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 2.0,	sin_psi:   -301.0,	sin_psi_t:   0.0,	cos_eps:   129.0,	cos_eps_t: -0.1 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d:-2.0,	om: 0.0,	sin_psi:   -158.0,	sin_psi_t:   0.0,	cos_eps:    -1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 2.0,	sin_psi:    123.0,	sin_psi_t:   0.0,	cos_eps:   -53.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 0.0,	d: 2.0,	om: 0.0,	sin_psi:     63.0,	sin_psi_t:   0.0,	cos_eps:    -2.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 1.0,	sin_psi:     63.0,	sin_psi_t:   0.1,	cos_eps:   -33.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 1.0,	sin_psi:    -58.0,	sin_psi_t:  -0.1,	cos_eps:    32.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 2.0,	sin_psi:    -59.0,	sin_psi_t:   0.0,	cos_eps:    26.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 1.0,	sin_psi:    -51.0,	sin_psi_t:   0.0,	cos_eps:    27.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 2.0,	sin_psi:    -38.0,	sin_psi_t:   0.0,	cos_eps:    16.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 0.0,	sin_psi:     29.0,	sin_psi_t:   0.0,	cos_eps:    -1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 2.0,	sin_psi:     29.0,	sin_psi_t:   0.0,	cos_eps:   -12.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 2.0,	sin_psi:    -31.0,	sin_psi_t:   0.0,	cos_eps:    13.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 0.0,	sin_psi:     26.0,	sin_psi_t:   0.0,	cos_eps:    -1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 1.0,	sin_psi:     21.0,	sin_psi_t:   0.0,	cos_eps:   -10.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 0.0,	d: 2.0,	om: 1.0,	sin_psi:     16.0,	sin_psi_t:   0.0,	cos_eps:    -8.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d:-2.0,	om: 1.0,	sin_psi:    -13.0,	sin_psi_t:   0.0,	cos_eps:     7.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 1.0,	sin_psi:    -10.0,	sin_psi_t:   0.0,	cos_eps:     5.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 1.0,	f: 0.0,	d:-2.0,	om: 0.0,	sin_psi:     -7.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 2.0,	d: 0.0,	om: 2.0,	sin_psi:      7.0,	sin_psi_t:   0.0,	cos_eps:    -3.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp:-1.0,	f: 2.0,	d: 0.0,	om: 2.0,	sin_psi:     -7.0,	sin_psi_t:   0.0,	cos_eps:     3.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 2.0,	sin_psi:     -8.0,	sin_psi_t:   0.0,	cos_eps:     3.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d: 2.0,	om: 0.0,	sin_psi:      6.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 2.0,	sin_psi:      6.0,	sin_psi_t:   0.0,	cos_eps:    -3.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 0.0,	d: 2.0,	om: 1.0,	sin_psi:     -6.0,	sin_psi_t:   0.0,	cos_eps:     3.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 1.0,	sin_psi:     -7.0,	sin_psi_t:   0.0,	cos_eps:     3.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 1.0,	sin_psi:      6.0,	sin_psi_t:   0.0,	cos_eps:    -3.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 0.0,	d:-2.0,	om: 1.0,	sin_psi:     -5.0,	sin_psi_t:   0.0,	cos_eps:     3.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp:-1.0,	f: 0.0,	d: 0.0,	om: 0.0,	sin_psi:      5.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 1.0,	sin_psi:     -5.0,	sin_psi_t:   0.0,	cos_eps:     3.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 0.0,	d:-2.0,	om: 0.0,	sin_psi:     -4.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f:-2.0,	d: 0.0,	om: 0.0,	sin_psi:      4.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 0.0,	d: 1.0,	om: 0.0,	sin_psi:     -4.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 1.0,	f: 0.0,	d: 0.0,	om: 0.0,	sin_psi:     -3.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 0.0,	sin_psi:      3.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp:-1.0,	f: 2.0,	d: 0.0,	om: 2.0,	sin_psi:     -3.0,	sin_psi_t:   0.0,	cos_eps:     1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -1.0,	lp:-1.0,	f: 2.0,	d: 2.0,	om: 2.0,	sin_psi:     -3.0,	sin_psi_t:   0.0,	cos_eps:     1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -2.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 1.0,	sin_psi:     -2.0,	sin_psi_t:   0.0,	cos_eps:     1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 3.0,	lp: 0.0,	f: 2.0,	d: 0.0,	om: 2.0,	sin_psi:     -3.0,	sin_psi_t:   0.0,	cos_eps:     1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp:-1.0,	f: 2.0,	d: 2.0,	om: 2.0,	sin_psi:     -3.0,	sin_psi_t:   0.0,	cos_eps:     1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 1.0,	f: 2.0,	d: 0.0,	om: 2.0,	sin_psi:      2.0,	sin_psi_t:   0.0,	cos_eps:    -1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 1.0,	sin_psi:     -2.0,	sin_psi_t:   0.0,	cos_eps:     1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 1.0,	sin_psi:      2.0,	sin_psi_t:   0.0,	cos_eps:    -1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 2.0,	sin_psi:     -2.0,	sin_psi_t:   0.0,	cos_eps:     1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 3.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 0.0,	sin_psi:      2.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d: 1.0,	om: 2.0,	sin_psi:      2.0,	sin_psi_t:   0.0,	cos_eps:    -1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 0.0,	d: 0.0,	om: 2.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:    -1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d:-4.0,	om: 0.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -2.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 2.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:    -1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 2.0,	d: 4.0,	om: 2.0,	sin_psi:     -2.0,	sin_psi_t:   0.0,	cos_eps:     1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 0.0,	d:-4.0,	om: 0.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 1.0,	f: 2.0,	d:-2.0,	om: 2.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:    -1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 1.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -2.0,	lp: 0.0,	f: 2.0,	d: 4.0,	om: 2.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -1.0,	lp: 0.0,	f: 4.0,	d: 0.0,	om: 2.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp:-1.0,	f: 0.0,	d:-2.0,	om: 0.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 1.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:    -1.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 2.0,	d: 2.0,	om: 2.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 0.0,	d: 2.0,	om: 1.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 4.0,	d:-2.0,	om: 2.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 3.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 2.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f: 2.0,	d:-2.0,	om: 0.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 2.0,	d: 0.0,	om: 1.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: -1.0,	lp:-1.0,	f: 0.0,	d: 2.0,	om: 1.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f:-2.0,	d: 0.0,	om: 1.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d:-1.0,	om: 2.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 0.0,	d: 2.0,	om: 0.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f:-2.0,	d:-2.0,	om: 0.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp:-1.0,	f: 2.0,	d: 0.0,	om: 1.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 1.0,	f: 0.0,	d:-2.0,	om: 1.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 1.0,	lp: 0.0,	f:-2.0,	d: 2.0,	om: 0.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 2.0,	lp: 0.0,	f: 0.0,	d: 2.0,	om: 0.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 0.0,	f: 2.0,	d: 4.0,	om: 2.0,	sin_psi:     -1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
    Coefficients{ l: 0.0,	lp: 1.0,	f: 0.0,	d: 1.0,	om: 0.0,	sin_psi:      1.0,	sin_psi_t:   0.0,	cos_eps:     0.0,	cos_eps_t:  0.0 },
];
// @formatter:on

#[cfg(test)]
/// All fixtures and assertion values were generated using the ERFA C library unless otherwise
/// stated.
mod tests {
    use float_eq::assert_float_eq;

    use lox_math::types::units::JulianCenturies;

    use super::nutation_iau1980;

    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn test_nutation_iau1980_jd0() {
        let jd0: JulianCenturies = -67.11964407939767;
        let actual = nutation_iau1980(jd0);
        assert_float_eq!(0.00000693404778664026, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(0.00004131255061383108, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau1980_j2000() {
        let j2000: JulianCenturies = 0.0;
        let actual = nutation_iau1980(j2000);
        assert_float_eq!(-0.00006750247617532478, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(-0.00002799221238377013, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau1980_j2100() {
        let j2100: JulianCenturies = 1.0;
        let actual = nutation_iau1980(j2100);
        assert_float_eq!(0.00001584138015187132, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(0.00004158958379918889, actual.obliquity, rel <= TOLERANCE);
    }
}
