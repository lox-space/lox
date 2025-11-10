// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_core::units::Angle;
use lox_time::{Time, julian_dates::JulianDate, time_scales::Tt};

use crate::nutation::Nutation;

const PRECOR: Angle = Angle::arcseconds(-0.29965);
const OBLCOR: Angle = Angle::arcseconds(-0.02524);

pub type PrecessionCorrections = Nutation;

pub fn precession_rate_iau2000(time: Time<Tt>) -> PrecessionCorrections {
    let t = time.centuries_since_j2000();

    PrecessionCorrections {
        longitude: t * PRECOR,
        obliquity: t * OBLCOR,
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::AngleUnits;
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_precession_rate_iau2000() {
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 53736.0);
        let exp = PrecessionCorrections {
            longitude: -8.716_465_172_668_348e-8.rad(),
            obliquity: -7.342_018_386_722_813e-9.rad(),
        };
        let act = precession_rate_iau2000(time);
        assert_approx_eq!(act, exp, atol <= 1e-22);
    }
}
