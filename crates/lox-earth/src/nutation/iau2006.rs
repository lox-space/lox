// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::Earth;

use crate::nutation::Nutation;
use crate::nutation::iau2000::nutation_iau2000a;

use lox_core::types::units::JulianCenturies;

/// The IAU 2000A nutation model adjusted to match the IAU 2006 precession model per
/// Wallace & Capitaine, 2006.
pub fn nutation_iau2006a(centuries_since_j2000_tdb: JulianCenturies) -> Nutation {
    let mut nutation = nutation_iau2000a(centuries_since_j2000_tdb);
    let j2_correction = Earth::j2_correction_factor(centuries_since_j2000_tdb);

    nutation.longitude += (0.4697e-6 + j2_correction) * nutation.longitude;
    nutation.obliquity += j2_correction * nutation.obliquity;

    nutation
}

pub trait J2Correction {
    fn j2_correction_factor(centuries_since_j2000_tdb: JulianCenturies) -> f64;
}

impl J2Correction for Earth {
    /// Factor correcting for secular variation of J₂.
    #[inline]
    fn j2_correction_factor(centuries_since_j2000_tdb: JulianCenturies) -> f64 {
        -2.7774e-6 * centuries_since_j2000_tdb
    }
}

#[cfg(test)]
mod tests {
    use crate::nutation::Nutation;

    use super::nutation_iau2006a;

    use lox_core::types::units::JulianCenturies;
    use lox_test_utils::assert_approx_eq;
    use lox_units::AngleUnits;

    const TOLERANCE: f64 = 1e-11;

    #[test]
    fn test_nutation_iau2006a_jd0() {
        let jd0: JulianCenturies = -67.11964407939767;
        let expected = Nutation {
            longitude: 0.00000737285641780423.rad(),
            obliquity: 0.00004132905772755788.rad(),
        };
        let actual = nutation_iau2006a(jd0);
        assert_approx_eq!(expected, actual, rtol <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2006a_j2000() {
        let j2000: JulianCenturies = 0.0;
        let expected = Nutation {
            longitude: -0.00006754425598969513.rad(),
            obliquity: -0.00002797083119237414.rad(),
        };
        let actual = nutation_iau2006a(j2000);
        assert_approx_eq!(expected, actual, rtol <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2006a_j2100() {
        let j2100: JulianCenturies = 1.0;
        let expected = Nutation {
            longitude: 0.00001585983730501046.rad(),
            obliquity: 0.00004162315218980551.rad(),
        };
        let actual = nutation_iau2006a(j2100);
        assert_approx_eq!(expected, actual, rtol <= TOLERANCE);
    }
}
