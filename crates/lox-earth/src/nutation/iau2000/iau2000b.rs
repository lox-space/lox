/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_bodies::fundamental::simon1994::mean_moon_sun_elongation_simon1994;
use lox_bodies::*;
use lox_math::types::units::JulianCenturies;

use crate::nutation::Nutation;
use crate::nutation::iau2000::{DelaunayArguments, luni_solar_nutation};

mod luni_solar;
mod planetary;

pub(crate) fn nutation_iau2000b(centuries_since_j2000_tdb: JulianCenturies) -> Nutation {
    let luni_solar_args = DelaunayArguments {
        l: Moon.mean_anomaly_simon1994(centuries_since_j2000_tdb),
        lp: Sun.mean_anomaly_simon1994(centuries_since_j2000_tdb),
        f: Moon.mean_argument_of_latitude_simon1994(centuries_since_j2000_tdb),
        d: mean_moon_sun_elongation_simon1994(centuries_since_j2000_tdb),
        om: Moon.ascending_node_mean_longitude_simon1994(centuries_since_j2000_tdb),
    };

    luni_solar_nutation(
        centuries_since_j2000_tdb,
        &luni_solar_args,
        &luni_solar::COEFFICIENTS,
    ) + planetary::OFFSETS
}

#[cfg(test)]
/// All fixtures and assertion values were generated using the ERFA C library unless otherwise
/// stated.
mod tests {
    use float_eq::assert_float_eq;

    use lox_math::types::units::JulianCenturies;

    use super::nutation_iau2000b;

    const TOLERANCE: f64 = 1e-11;

    #[test]
    fn test_nutation_iau2000b_jd0() {
        let jd0: JulianCenturies = -67.11964407939767;
        let actual = nutation_iau2000b(jd0);
        assert_float_eq!(0.00001795252319583832, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(0.00004024546928325646, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2000b_j2000() {
        let j2000: JulianCenturies = 0.0;
        let actual = nutation_iau2000b(j2000);
        assert_float_eq!(-0.00006754261253992235, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(-0.00002797092331098565, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2000b_j2100() {
        let j2100: JulianCenturies = 1.0;
        let actual = nutation_iau2000b(j2100);
        assert_float_eq!(0.00001586677813945249, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(0.00004162057618703116, actual.obliquity, rel <= TOLERANCE);
    }
}
