// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::fundamental::iers03::general_accum_precession_in_longitude_iers03;
use lox_bodies::fundamental::mhb2000::{
    mean_moon_sun_elongation_mhb2000_luni_solar, mean_moon_sun_elongation_mhb2000_planetary,
};
use lox_bodies::*;
use lox_units::AngleUnits;
use lox_units::types::units::JulianCenturies;

use crate::nutation::Nutation;
use crate::nutation::iau2000::{DelaunayArguments, luni_solar_nutation};

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

pub(crate) fn nutation_iau2000a(centuries_since_j2000_tdb: JulianCenturies) -> Nutation {
    let luni_solar_args = DelaunayArguments {
        l: Moon.mean_anomaly_iers03(centuries_since_j2000_tdb),
        lp: Sun.mean_anomaly_mhb2000(centuries_since_j2000_tdb),
        f: Moon
            .mean_longitude_minus_ascending_node_mean_longitude_iers03(centuries_since_j2000_tdb),
        d: mean_moon_sun_elongation_mhb2000_luni_solar(centuries_since_j2000_tdb),
        om: Moon.ascending_node_mean_longitude_iers03(centuries_since_j2000_tdb),
    };

    let planetary_args = DelaunayArguments {
        l: Moon.mean_anomaly_mhb2000(centuries_since_j2000_tdb),
        lp: 0.0.rad(), // unused
        f: Moon
            .mean_longitude_minus_ascending_node_mean_longitude_mhb2000(centuries_since_j2000_tdb),
        d: mean_moon_sun_elongation_mhb2000_planetary(centuries_since_j2000_tdb),
        om: Moon.ascending_node_mean_longitude_mhb2000(centuries_since_j2000_tdb),
    };

    luni_solar_nutation(
        centuries_since_j2000_tdb,
        &luni_solar_args,
        &luni_solar::COEFFICIENTS,
    ) + planetary_nutation(centuries_since_j2000_tdb, planetary_args)
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
                + coeff.mercury * Mercury.mean_longitude_iers03(centuries_since_j2000_tdb)
                + coeff.venus * Venus.mean_longitude_iers03(centuries_since_j2000_tdb)
                + coeff.earth * Earth.mean_longitude_iers03(centuries_since_j2000_tdb)
                + coeff.mars * Mars.mean_longitude_iers03(centuries_since_j2000_tdb)
                + coeff.jupiter * Jupiter.mean_longitude_iers03(centuries_since_j2000_tdb)
                + coeff.saturn * Saturn.mean_longitude_iers03(centuries_since_j2000_tdb)
                + coeff.uranus * Uranus.mean_longitude_iers03(centuries_since_j2000_tdb)
                + coeff.neptune * Neptune.mean_longitude_mhb2000(centuries_since_j2000_tdb)
                + coeff.pa
                    * general_accum_precession_in_longitude_iers03(centuries_since_j2000_tdb))
            .mod_two_pi_signed();

            // Accumulate current term.
            let sin_arg = arg.sin();
            let cos_arg = arg.cos();
            dpsi += coeff.sin_psi * sin_arg + coeff.cos_psi * cos_arg;
            deps += coeff.sin_eps * sin_arg + coeff.cos_eps * cos_arg;

            (dpsi, deps)
        });

    Nutation {
        longitude: (dpsi * 1e-1).uas(),
        obliquity: (deps * 1e-1).uas(),
    }
}

#[cfg(test)]
/// All fixtures and assertion values were generated using the ERFA C library unless otherwise
/// stated.
mod tests {
    use lox_test_utils::assert_approx_eq;
    use lox_units::{AngleUnits, types::units::JulianCenturies};

    use crate::nutation::Nutation;

    use super::nutation_iau2000a;

    const TOLERANCE: f64 = 1e-11;

    #[test]
    fn test_nutation_iau2000a_jd0() {
        let jd0: JulianCenturies = -67.11964407939767;
        let expected = Nutation {
            longitude: 0.00000737147877835653.rad(),
            obliquity: 0.00004132135467915123.rad(),
        };
        let actual = nutation_iau2000a(jd0);
        assert_approx_eq!(expected, actual, rtol <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2000a_j2000() {
        let j2000: JulianCenturies = 0.0;
        let expected = Nutation {
            longitude: -0.00006754422426417299.rad(),
            obliquity: -0.00002797083119237414.rad(),
        };
        let actual = nutation_iau2000a(j2000);
        assert_approx_eq!(expected, actual, rtol <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2000a_j2100() {
        let j2100: JulianCenturies = 1.0;
        let expected = Nutation {
            longitude: 0.00001585987390484147.rad(),
            obliquity: 0.00004162326779426948.rad(),
        };
        let actual = nutation_iau2000a(j2100);
        assert_approx_eq!(expected, actual, rtol <= TOLERANCE);
    }
}
