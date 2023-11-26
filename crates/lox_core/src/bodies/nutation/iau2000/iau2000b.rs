/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::bodies::fundamental::simon1994::mean_moon_sun_elongation_simon1994;
use crate::bodies::nutation::iau2000::{luni_solar_nutation, DelaunayArguments};
use crate::bodies::nutation::Nutation;
use crate::bodies::*;
use crate::math::RADIANS_IN_ARCSECOND;
use crate::time::intervals::TDBJulianCenturiesSinceJ2000;

mod luni_solar;
mod planetary;

pub(crate) fn nutation_iau2000b(t: TDBJulianCenturiesSinceJ2000) -> Nutation {
    let luni_solar_args = DelaunayArguments {
        l: Moon.mean_anomaly_simon1994(t),
        lp: Sun.mean_anomaly_simon1994(t),
        f: Moon.mean_argument_of_latitude_simon1994(t),
        d: mean_moon_sun_elongation_simon1994(t),
        om: Moon.ascending_node_mean_longitude_simon1994(t),
    };

    luni_solar_nutation(t, luni_solar_args, &luni_solar::COEFFICIENTS) + planetary::OFFSETS
}

#[cfg(test)]
/// All fixtures and assertion values were generated using the ERFA C library unless otherwise
/// stated.
mod tests {
    use float_eq::assert_float_eq;

    use crate::time::intervals::TDBJulianCenturiesSinceJ2000;

    use super::nutation_iau2000b;

    const TOLERANCE: f64 = 1e-11;

    #[test]
    fn test_nutation_iau2000a_jd0() {
        let jd0: TDBJulianCenturiesSinceJ2000 = -67.11964407939767;
        let actual = nutation_iau2000b(jd0);
        assert_float_eq!(0.00000737147877835653, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(0.00004132135467915123, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2000a_j2000() {
        let j2000: TDBJulianCenturiesSinceJ2000 = 0.0;
        let actual = nutation_iau2000b(j2000);
        assert_float_eq!(-0.00006754422426417299, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(-0.00002797083119237414, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2000a_j2100() {
        let j2100: TDBJulianCenturiesSinceJ2000 = 1.0;
        let actual = nutation_iau2000b(j2100);
        assert_float_eq!(0.00001585987390484147, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(0.00004162326779426948, actual.obliquity, rel <= TOLERANCE);
    }
}
