use std::f64::consts::TAU;

pub(crate) use iau2000a::nutation_iau2000a;
pub(crate) use iau2000b::nutation_iau2000b;
use lox_math::types::units::JulianCenturies;

use crate::nutation::{point1_microarcsec_to_rad, Nutation};

mod iau2000a;
mod iau2000b;

/// IAU 2000A and 2000B use the same structure for luni-solar coefficients.
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

struct DelaunayArguments {
    l: f64,
    lp: f64,
    f: f64,
    d: f64,
    om: f64,
}

/// Calculate the luni-solar nutation for `t` given `args` and coefficients for either models A or
/// B.
fn luni_solar_nutation(
    centuries_since_j2000_tdb: JulianCenturies,
    args: &DelaunayArguments,
    coeffs: &[LuniSolarCoefficients],
) -> Nutation {
    let mut nutation = coeffs
        .iter()
        // The coefficients are given by descending magnitude but folded by ascending
        // magnitude to minimise floating-point error.
        .rev()
        .fold(Nutation::default(), |mut nut, coeff| {
            // Form argument for current term.
            let arg = (coeff.l * args.l
                + coeff.lp * args.lp
                + coeff.f * args.f
                + coeff.d * args.d
                + coeff.om * args.om)
                % TAU;

            // Accumulate current term.
            let sin_arg = arg.sin();
            let cos_arg = arg.cos();
            nut.longitude += (coeff.sin_psi + coeff.sin_psi_t * centuries_since_j2000_tdb)
                * sin_arg
                + coeff.cos_psi * cos_arg;
            nut.obliquity += (coeff.cos_eps + coeff.cos_eps_t * centuries_since_j2000_tdb)
                * cos_arg
                + coeff.sin_eps * sin_arg;

            nut
        });

    nutation.longitude = point1_microarcsec_to_rad(nutation.longitude);
    nutation.obliquity = point1_microarcsec_to_rad(nutation.obliquity);

    nutation
}
