// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Julian dates.
//!
//! The [JulianDate] trait expresses time representations as Julian dates relative to standard
//! [Epoch]s and in a variety of [TimeDeltaUnit]s.

use lox_core::time::constants::{
    ATTOSECONDS_IN_SECOND, SECONDS_BETWEEN_J1950_AND_J2000, SECONDS_BETWEEN_JD_AND_J2000,
    SECONDS_BETWEEN_MJD_AND_J2000, SECONDS_PER_DAY,
};

use crate::deltas::{InvalidFloatSeconds, TimeDelta, TimeDeltaUnit};

/// A Julian date or a duration in days, as an `f64`.
pub type Days = f64;

/// A duration in Julian centuries (36525 days), as an `f64`.
pub type JulianCenturies = f64;

/// 4713 BC January 1 12:00
pub const J0: TimeDelta = TimeDelta::from_seconds(-SECONDS_BETWEEN_JD_AND_J2000);

/// The Julian epochs supported by Lox.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Epoch {
    /// Julian Date epoch (JD 0 = 4713 BC January 1 12:00).
    JulianDate,
    /// Modified Julian Date epoch (MJD 0 = 1858 November 17 00:00).
    ModifiedJulianDate,
    /// J1950 epoch (1950 January 1 12:00).
    J1950,
    /// J2000 epoch (2000 January 1 12:00).
    J2000,
}

impl Epoch {
    /// Returns the epoch as a time since J2000, which is negative for every epoch but J2000.
    pub const fn to_delta(&self) -> TimeDelta {
        TimeDelta::from_seconds(-self.seconds_before_j2000())
    }

    const fn seconds_before_j2000(&self) -> i64 {
        match self {
            Epoch::JulianDate => SECONDS_BETWEEN_JD_AND_J2000,
            Epoch::ModifiedJulianDate => SECONDS_BETWEEN_MJD_AND_J2000,
            Epoch::J1950 => SECONDS_BETWEEN_J1950_AND_J2000,
            Epoch::J2000 => 0,
        }
    }
}

/// Converts a Julian date relative to `epoch` into the time since J2000.
///
/// # Panics
///
/// Panics if the Julian date is not finite or the result is out of range.
#[track_caller]
pub const fn julian_date_to_delta(julian_date: Days, epoch: Epoch) -> TimeDelta {
    epoch
        .to_delta()
        .add_const(TimeDelta::new_in(julian_date, TimeDeltaUnit::Days))
}

/// Converts a Julian date relative to `epoch` into the time since J2000.
///
/// # Errors
///
/// Returns an error if the Julian date is not finite or the result is out of range.
pub fn try_julian_date_to_delta(
    julian_date: Days,
    epoch: Epoch,
) -> Result<TimeDelta, InvalidFloatSeconds> {
    let error = || InvalidFloatSeconds(julian_date * SECONDS_PER_DAY as f64);
    TimeDelta::try_new_in(julian_date, TimeDeltaUnit::Days)?
        .checked_add(epoch.to_delta())
        .ok_or_else(error)
}

/// Converts a two-part Julian date `jd1 + jd2` in days into the time since J2000.
///
/// # Panics
///
/// Panics if either part is not finite or the result is out of range.
#[track_caller]
pub const fn two_part_julian_date_to_delta(jd1: Days, jd2: Days) -> TimeDelta {
    J0.add_const(TimeDelta::new_in(jd1, TimeDeltaUnit::Days))
        .add_const(TimeDelta::new_in(jd2, TimeDeltaUnit::Days))
}

/// Converts a two-part Julian date `jd1 + jd2` in days into the time since J2000.
///
/// # Errors
///
/// Returns an error if either part is not finite or the result is out of range.
pub fn try_two_part_julian_date_to_delta(
    jd1: Days,
    jd2: Days,
) -> Result<TimeDelta, InvalidFloatSeconds> {
    let error = || InvalidFloatSeconds((jd1 + jd2) * SECONDS_PER_DAY as f64);
    let dt1 = TimeDelta::try_new_in(jd1, TimeDeltaUnit::Days)?;
    let dt2 = TimeDelta::try_new_in(jd2, TimeDeltaUnit::Days)?;
    J0.checked_add(dt1)
        .and_then(|dt| dt.checked_add(dt2))
        .ok_or_else(error)
}

/// Enables a time or date type to be expressed as a Julian date.
///
/// Implementors only provide [`to_delta`](Self::to_delta), the time since J2000.
pub trait JulianDate {
    /// Returns the time since J2000.
    fn to_delta(&self) -> TimeDelta;

    /// Expresses `self` as a Julian date in the specified unit, relative to the given [Epoch].
    fn julian_date(&self, epoch: Epoch, unit: TimeDeltaUnit) -> f64 {
        let delta = self.to_delta();
        // Whole seconds and the integral epoch offsets add exactly in f64, so the
        // subsecond part is only rounded once.
        let seconds = (delta.seconds() as f64 + epoch.seconds_before_j2000() as f64)
            + delta.attoseconds() as f64 / ATTOSECONDS_IN_SECOND as f64;
        unit.convert_seconds(seconds)
    }

    /// Expresses `self` as a two-part Julian date in the specified unit, relative to the given
    /// [Epoch].
    ///
    /// The default implementation calls [JulianDate::julian_date] and returns the integer and
    /// fractional parts of the single `f64` result. Applications that cannot afford the associated
    /// loss of precision should provide their own implementations.
    fn two_part_julian_date(&self) -> (f64, f64) {
        let jd = self.julian_date(Epoch::JulianDate, TimeDeltaUnit::Days);
        (
            lox_core::math::float::trunc(jd),
            lox_core::math::float::fract(jd),
        )
    }

    /// Returns the number of seconds since the Julian epoch as an `f64`.
    fn seconds_since_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::JulianDate, TimeDeltaUnit::Seconds)
    }

    /// Returns the number of seconds since the Modified Julian epoch as an `f64`.
    fn seconds_since_modified_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::ModifiedJulianDate, TimeDeltaUnit::Seconds)
    }

    /// Returns the number of seconds since J1950 as an `f64`.
    fn seconds_since_j1950(&self) -> f64 {
        self.julian_date(Epoch::J1950, TimeDeltaUnit::Seconds)
    }

    /// Returns the number of seconds since J2000 as an `f64`.
    fn seconds_since_j2000(&self) -> f64 {
        self.julian_date(Epoch::J2000, TimeDeltaUnit::Seconds)
    }

    /// Returns the number of days since the Julian epoch as an `f64`.
    fn days_since_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::JulianDate, TimeDeltaUnit::Days)
    }

    /// Returns the number of days since the Modified Julian epoch as an `f64`.
    fn days_since_modified_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::ModifiedJulianDate, TimeDeltaUnit::Days)
    }

    /// Returns the number of days since J1950 as an `f64`.
    fn days_since_j1950(&self) -> f64 {
        self.julian_date(Epoch::J1950, TimeDeltaUnit::Days)
    }

    /// Returns the number of days since J2000 as an `f64`.
    fn days_since_j2000(&self) -> f64 {
        self.julian_date(Epoch::J2000, TimeDeltaUnit::Days)
    }

    /// Returns the number of years since the Julian epoch as an `f64`.
    fn years_since_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::JulianDate, TimeDeltaUnit::JulianYears)
    }

    /// Returns the number of years since the Modified Julian epoch as an `f64`.
    fn years_since_modified_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::ModifiedJulianDate, TimeDeltaUnit::JulianYears)
    }

    /// Returns the number of years since J1950 as an `f64`.
    fn years_since_j1950(&self) -> f64 {
        self.julian_date(Epoch::J1950, TimeDeltaUnit::JulianYears)
    }

    /// Returns the number of years since J2000 as an `f64`.
    fn years_since_j2000(&self) -> f64 {
        self.julian_date(Epoch::J2000, TimeDeltaUnit::JulianYears)
    }

    /// Returns the number of centuries since the Julian epoch as an `f64`.
    fn centuries_since_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::JulianDate, TimeDeltaUnit::JulianCenturies)
    }

    /// Returns the number of centuries since the Modified Julian epoch as an `f64`.
    fn centuries_since_modified_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::ModifiedJulianDate, TimeDeltaUnit::JulianCenturies)
    }

    /// Returns the number of centuries since J1950 as an `f64`.
    fn centuries_since_j1950(&self) -> f64 {
        self.julian_date(Epoch::J1950, TimeDeltaUnit::JulianCenturies)
    }

    /// Returns the number of centuries since J2000 as an `f64`.
    fn centuries_since_j2000(&self) -> f64 {
        self.julian_date(Epoch::J2000, TimeDeltaUnit::JulianCenturies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_to_delta() {
        assert_eq!(Epoch::J2000.to_delta(), TimeDelta::ZERO);
        assert_eq!(Epoch::JulianDate.to_delta(), J0);
        assert_eq!(
            Epoch::ModifiedJulianDate.to_delta(),
            TimeDelta::from_seconds(-SECONDS_BETWEEN_MJD_AND_J2000)
        );
    }

    #[test]
    fn test_julian_date_to_delta() {
        assert_eq!(
            julian_date_to_delta(2451545.0, Epoch::JulianDate),
            TimeDelta::ZERO
        );
        assert_eq!(
            julian_date_to_delta(1.5, Epoch::J2000),
            TimeDelta::from_hours(36)
        );
        assert_eq!(
            try_julian_date_to_delta(51544.5, Epoch::ModifiedJulianDate),
            Ok(TimeDelta::ZERO)
        );
        assert!(try_julian_date_to_delta(f64::NAN, Epoch::J2000).is_err());
    }

    #[test]
    fn test_two_part_julian_date_to_delta() {
        assert_eq!(
            two_part_julian_date_to_delta(2400000.5, 51544.5),
            TimeDelta::ZERO
        );
        assert_eq!(
            try_two_part_julian_date_to_delta(2451545.0, 0.0),
            Ok(TimeDelta::ZERO)
        );
        assert!(try_two_part_julian_date_to_delta(2451545.0, f64::NAN).is_err());
        assert!(try_two_part_julian_date_to_delta(f64::INFINITY, 0.0).is_err());
    }
}
