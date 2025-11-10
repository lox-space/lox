// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use fast_polynomial::poly_array;
use lox_core::units::Angle;
use lox_test_utils::ApproxEq;
use lox_time::{Time, julian_dates::JulianDate, time_scales::Tt};

/// Mean obliquity of the ecliptic.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, ApproxEq)]
pub struct MeanObliquity(pub Angle);

impl MeanObliquity {
    /// Returns the mean obliquity of the ecliptic based on the IAU1980 precession model.
    pub fn iau1980(time: Time<Tt>) -> Self {
        let t = time.centuries_since_j2000();

        Self(Angle::arcseconds(poly_array(
            t,
            &[84381.448, -46.8150, -0.00059, 0.001813],
        )))
    }

    /// Returns the mean obliquity of the ecliptic based on the IAU2006 precession model.
    pub fn iau2006(time: Time<Tt>) -> Self {
        let t = time.centuries_since_j2000();

        Self(Angle::arcseconds(poly_array(
            t,
            &[
                84381.406,
                -46.836769,
                -0.0001831,
                0.00200340,
                -0.000000576,
                -0.0000000434,
            ],
        )))
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_mean_obliquity_iau1980() {
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 54388.0);
        let exp = MeanObliquity(Angle::new(0.409_075_134_764_381_6));
        let act = MeanObliquity::iau1980(time);
        assert_approx_eq!(act, exp, rtol <= 1e-14);
    }

    #[test]
    fn test_mean_obliquity_iau2006() {
        let time = Time::from_two_part_julian_date(Tt, 2400000.5, 54388.0);
        let exp = MeanObliquity(Angle::new(0.409_074_922_938_725_8));
        let act = MeanObliquity::iau2006(time);
        assert_approx_eq!(act, exp, rtol <= 1e-14);
    }
}
