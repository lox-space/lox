// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_time::Time;
use lox_time::julian_dates::JulianDate;
use lox_time::time_scales::Tdb;

use crate::nutation::Nutation;

impl Nutation {
    /// The IAU 2000A nutation model adjusted to match the IAU 2006 precession model per
    /// Wallace & Capitaine, 2006.
    pub fn iau2006a(time: Time<Tdb>) -> Nutation {
        let mut nutation = Self::iau2000a(time);

        let j2_correction = -2.7774e-6 * time.centuries_since_j2000();
        nutation.longitude += (0.4697e-6 + j2_correction) * nutation.longitude;
        nutation.obliquity += j2_correction * nutation.obliquity;

        nutation
    }
}

#[cfg(test)]
mod tests {
    use crate::nutation::Nutation;

    use lox_test_utils::assert_approx_eq;
    use lox_time::{Time, time_scales::Tdb};
    use lox_units::AngleUnits;

    #[test]
    fn test_nutation_iau2006a() {
        let time = Time::from_two_part_julian_date(Tdb, 2400000.5, 53736.0);
        let expected = Nutation {
            longitude: -9.630_912_025_820_31e-6.rad(),
            obliquity: 4.063_238_496_887_25e-5.rad(),
        };
        let actual = Nutation::iau2006a(time);
        assert_approx_eq!(expected, actual, rtol <= 1e-13);
    }
}
