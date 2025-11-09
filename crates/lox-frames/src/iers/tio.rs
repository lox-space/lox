// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Module tio exposes functions for calculating the Terrestrial Intermediate Origin (TIO) locator,
//! s', which places the TIO on the equator of the Celestial Intermediate Pole (CIP).

use lox_core::types::units::JulianCenturies;
use lox_core::units::{Angle, AngleUnits};
use lox_test_utils::ApproxEq;
use lox_time::Time;
use lox_time::julian_dates::JulianDate;
use lox_time::time_scales::Tt;

type ArcsecondsPerCentury = f64;

/// The TIO locator is unpredictable, being dependent on the integration of observations of polar
/// motion, but is dominated by secular drift, providing a close approximation.
const SECULAR_DRIFT: ArcsecondsPerCentury = -47e-6;

/// Approximate the TIO locator, s', in radians using the IAU 2000 model.
#[inline]
pub fn tio_locator(centuries_since_j2000_tt: JulianCenturies) -> Angle {
    (SECULAR_DRIFT * centuries_since_j2000_tt).arcsec()
}

/// The TIO locator s'
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, ApproxEq)]
pub struct TioLocator(pub Angle);

impl TioLocator {
    pub fn iau2000(time: Time<Tt>) -> Self {
        let t = time.centuries_since_j2000();
        TioLocator(Angle::arcseconds(SECULAR_DRIFT * t))
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_sp_00() {
        let t = 123.45;
        let expected = -2.812961699849694e-8.rad();
        assert_approx_eq!(expected, tio_locator(t), rtol <= 1e-12);
    }

    #[test]
    fn test_tio_locator_iau2000() {
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 52541.0);
        let exp = TioLocator(Angle::new(-6.216_698_469_981_019e-12));
        let act = TioLocator::iau2000(time);
        assert_approx_eq!(act, exp, atol <= 1e-12);
    }
}
