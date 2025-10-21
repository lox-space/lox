// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-License-Identifier: MPL-2.0

pub(crate) use iau2000a::nutation_iau2000a;
pub(crate) use iau2000b::nutation_iau2000b;
use lox_units::{Angle, AngleUnits, types::units::JulianCenturies};

use crate::nutation::Nutation;

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

#[derive(Debug)]
struct DelaunayArguments {
    l: Angle,
    lp: Angle,
    f: Angle,
    d: Angle,
    om: Angle,
}

/// Calculate the luni-solar nutation for `t` given `args` and coefficients for either models A or
/// B.
fn luni_solar_nutation(
    centuries_since_j2000_tdb: JulianCenturies,
    args: &DelaunayArguments,
    coeffs: &[LuniSolarCoefficients],
) -> Nutation {
    let (dpsi, deps) = coeffs
        .iter()
        // The coefficients are given by descending magnitude but folded by ascending
        // magnitude to minimise floating-point error.
        .rev()
        .fold((0.0, 0.0), |(mut dpsi, mut deps), coeff| {
            // Form argument for current term.
            let arg = (coeff.l * args.l
                + coeff.lp * args.lp
                + coeff.f * args.f
                + coeff.d * args.d
                + coeff.om * args.om)
                .mod_two_pi_signed();

            // Accumulate current term.
            let sin_arg = arg.sin();
            let cos_arg = arg.cos();
            dpsi += (coeff.sin_psi + coeff.sin_psi_t * centuries_since_j2000_tdb) * sin_arg
                + coeff.cos_psi * cos_arg;
            deps += (coeff.cos_eps + coeff.cos_eps_t * centuries_since_j2000_tdb) * cos_arg
                + coeff.sin_eps * sin_arg;

            (dpsi, deps)
        });

    Nutation {
        longitude: (dpsi * 1e-1).uas(),
        obliquity: (deps * 1e-1).uas(),
    }
}
