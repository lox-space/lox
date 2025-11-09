// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_core::types::units::JulianCenturies;
use lox_core::units::AngleUnits;
use lox_time::Time;
use lox_time::julian_dates::JulianDate;
use lox_time::time_scales::Tdb;

use crate::iers::fundamental::iers03::{
    earth_l_iers03, f_iers03, jupiter_l_iers03, l_iers03, mars_l_iers03, mercury_l_iers03,
    omega_iers03, pa_iers03, saturn_l_iers03, uranus_l_iers03, venus_l_iers03,
};
use crate::iers::fundamental::mhb2000::{
    d_mhb2000_luni_solar, d_mhb2000_planetary, f_mhb2000, l_mhb2000, lp_mhb2000, neptune_l_mhb2000,
    omega_mhb2000,
};
use crate::iers::nutation::Nutation;
use crate::iers::nutation::iau2000::{DelaunayArguments, luni_solar_nutation};

mod luni_solar;
mod planetary;

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

impl Nutation {
    pub fn iau2000a(time: Time<Tdb>) -> Nutation {
        let t = time.centuries_since_j2000();
        let luni_solar_args = DelaunayArguments {
            l: l_iers03(t),
            lp: lp_mhb2000(t),
            f: f_iers03(t),
            d: d_mhb2000_luni_solar(t),
            om: omega_iers03(t),
        };

        let planetary_args = DelaunayArguments {
            l: l_mhb2000(t),
            lp: 0.0.rad(), // unused
            f: f_mhb2000(t),
            d: d_mhb2000_planetary(t),
            om: omega_mhb2000(t),
        };

        luni_solar_nutation(t, &luni_solar_args, &luni_solar::COEFFICIENTS)
            + planetary_nutation(t, planetary_args)
    }
}

fn planetary_nutation(
    centuries_since_j2000_tdb: JulianCenturies,
    args: DelaunayArguments,
) -> Nutation {
    let (dpsi, deps) = planetary::COEFFICIENTS
        .iter()
        // The coefficients are given by descending magnitude but folded by ascending
        // magnitude to minimise floating-point error.
        .rev()
        .fold((0.0, 0.0), |(mut dpsi, mut deps), coeff| {
            // Form argument for current term.
            let arg = (coeff.l * args.l
                + coeff.f * args.f
                + coeff.d * args.d
                + coeff.om * args.om
                + coeff.mercury * mercury_l_iers03(centuries_since_j2000_tdb)
                + coeff.venus * venus_l_iers03(centuries_since_j2000_tdb)
                + coeff.earth * earth_l_iers03(centuries_since_j2000_tdb)
                + coeff.mars * mars_l_iers03(centuries_since_j2000_tdb)
                + coeff.jupiter * jupiter_l_iers03(centuries_since_j2000_tdb)
                + coeff.saturn * saturn_l_iers03(centuries_since_j2000_tdb)
                + coeff.uranus * uranus_l_iers03(centuries_since_j2000_tdb)
                + coeff.neptune * neptune_l_mhb2000(centuries_since_j2000_tdb)
                + coeff.pa * pa_iers03(centuries_since_j2000_tdb))
            .mod_two_pi_signed();

            // Accumulate current term.
            let sin_arg = arg.sin();
            let cos_arg = arg.cos();
            dpsi += coeff.sin_psi * sin_arg + coeff.cos_psi * cos_arg;
            deps += coeff.sin_eps * sin_arg + coeff.cos_eps * cos_arg;

            (dpsi, deps)
        });

    Nutation {
        dpsi: (dpsi * 1e-1).uas(),
        deps: (deps * 1e-1).uas(),
    }
}

#[cfg(test)]
/// All fixtures and assertion values were generated using the ERFA C library unless otherwise
/// stated.
mod tests {
    use lox_core::units::AngleUnits;
    use lox_test_utils::assert_approx_eq;
    use lox_time::{Time, time_scales::Tdb};

    use crate::iers::nutation::Nutation;

    #[test]
    fn test_nutation_iau2000a() {
        let time = Time::from_two_part_julian_date(Tdb, 2400000.5, 53736.0);
        let expected = Nutation {
            dpsi: -9.630_909_107_115_518e-6.rad(),
            deps: 4.063_239_174_001_679e-5.rad(),
        };
        let actual = Nutation::iau2000a(time);
        assert_approx_eq!(expected, actual, rtol <= 1e-13);
    }
}
