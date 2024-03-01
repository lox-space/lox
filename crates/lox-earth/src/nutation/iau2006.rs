/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_bodies::Earth;
use lox_time::intervals::TDBJulianCenturiesSinceJ2000;

use crate::nutation::iau2000::nutation_iau2000a;
use crate::nutation::Nutation;

/// The IAU 2000A nutation model adjusted to match the IAU 2006 precession model per
/// Wallace & Capitaine, 2006.
pub fn nutation_iau2006a(t: TDBJulianCenturiesSinceJ2000) -> Nutation {
    let mut nutation = nutation_iau2000a(t);
    let j2_correction = Earth::j2_correction_factor(t);

    nutation.longitude += nutation.longitude * (0.4697e-6 + j2_correction);
    nutation.obliquity += nutation.obliquity * j2_correction;

    nutation
}

pub trait J2Correction {
    fn j2_correction_factor(t: TDBJulianCenturiesSinceJ2000) -> f64;
}

impl J2Correction for Earth {
    /// Factor correcting for secular variation of J₂.
    #[inline]
    fn j2_correction_factor(t: TDBJulianCenturiesSinceJ2000) -> f64 {
        -2.7774e-6 * t
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use lox_time::intervals::TDBJulianCenturiesSinceJ2000;

    use super::nutation_iau2006a;

    const TOLERANCE: f64 = 1e-11;

    #[test]
    fn test_nutation_iau2006a_jd0() {
        let jd0: TDBJulianCenturiesSinceJ2000 = -67.11964407939767;
        let actual = nutation_iau2006a(jd0);
        assert_float_eq!(0.00000737285641780423, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(0.00004132905772755788, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2006a_j2000() {
        let j2000: TDBJulianCenturiesSinceJ2000 = 0.0;
        let actual = nutation_iau2006a(j2000);
        assert_float_eq!(-0.00006754425598969513, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(-0.00002797083119237414, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2006a_j2100() {
        let j2100: TDBJulianCenturiesSinceJ2000 = 1.0;
        let actual = nutation_iau2006a(j2100);
        assert_float_eq!(0.00001585983730501046, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(0.00004162315218980551, actual.obliquity, rel <= TOLERANCE);
    }
}