// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_time::Time;
use lox_time::julian_dates::JulianDate;
use lox_time::time_scales::Tdb;

use crate::iers::fundamental::simon1994::{
    d_simon1994, f_simon1994, l_simon1994, lp_simon1994, omega_simon1994,
};
use crate::iers::nutation::Nutation;
use crate::iers::nutation::iau2000::{DelaunayArguments, luni_solar_nutation};

mod luni_solar;
mod planetary;

impl Nutation {
    pub fn iau2000b(time: Time<Tdb>) -> Nutation {
        let t = time.centuries_since_j2000();
        let luni_solar_args = DelaunayArguments {
            l: l_simon1994(t),
            lp: lp_simon1994(t),
            f: f_simon1994(t),
            d: d_simon1994(t),
            om: omega_simon1994(t),
        };

        luni_solar_nutation(t, &luni_solar_args, &luni_solar::COEFFICIENTS) + planetary::OFFSETS
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::AngleUnits;
    use lox_test_utils::assert_approx_eq;
    use lox_time::{Time, time_scales::Tdb};

    use crate::iers::nutation::Nutation;

    #[test]
    fn test_nutation_iau2000b() {
        let time = Time::from_two_part_julian_date(Tdb, 2400000.5, 53736.0);
        let expected = Nutation {
            dpsi: -9.632_552_291_148_363e-6.rad(),
            deps: 4.063_197_106_621_159e-5.rad(),
        };
        let actual = Nutation::iau2000b(time);
        assert_approx_eq!(expected, actual, rtol <= 1e-13);
    }
}
