// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

/*!
    Module `julian_dates` exposes the [JulianDate] trait for expressing arbitrary time
    representations as Julian dates relative to standard [Epoch]s and in a variety of [Unit]s.
*/

use crate::i64::consts::SECONDS_BETWEEN_JD_AND_J2000;

use super::deltas::TimeDelta;

/// 4713 BC January 1 12:00
pub const J0: TimeDelta = TimeDelta::from_seconds(-SECONDS_BETWEEN_JD_AND_J2000);

/// The Julian epochs supported by Lox.
pub enum Epoch {
    JulianDate,
    ModifiedJulianDate,
    J1950,
    J2000,
}

/// The units of time in which a Julian date may be expressed.
pub enum Unit {
    Seconds,
    Days,
    Years,
    Centuries,
}

/// Enables a time or date type to be expressed as a Julian date.
pub trait JulianDate {
    /// Expresses `self` as a Julian date in the specified [Unit], relative to the given [Epoch].
    ///
    /// This is the only required method for implementing the [JulianDate] trait.
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64;

    /// Expresses `self` as a two-part Julian date in the specified [Unit], relative to the given
    /// [Epoch].
    ///
    /// The default implementation calls [JulianDate::julian_date] and returns the integer and
    /// fractional parts of the single `f64` result. Applications that cannot afford the associated
    /// loss of precision should provide their own implementations.
    fn two_part_julian_date(&self) -> (f64, f64) {
        let jd = self.julian_date(Epoch::JulianDate, Unit::Days);
        (jd.trunc(), jd.fract())
    }

    /// Returns the number of seconds since the Julian epoch as an `f64`.
    fn seconds_since_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::JulianDate, Unit::Seconds)
    }

    /// Returns the number of seconds since the Modified Julian epoch as an `f64`.
    fn seconds_since_modified_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::ModifiedJulianDate, Unit::Seconds)
    }

    /// Returns the number of seconds since J1950 as an `f64`.
    fn seconds_since_j1950(&self) -> f64 {
        self.julian_date(Epoch::J1950, Unit::Seconds)
    }

    /// Returns the number of seconds since J2000 as an `f64`.
    fn seconds_since_j2000(&self) -> f64 {
        self.julian_date(Epoch::J2000, Unit::Seconds)
    }

    /// Returns the number of days since the Julian epoch as an `f64`.
    fn days_since_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::JulianDate, Unit::Days)
    }

    /// Returns the number of days since the Modified Julian epoch as an `f64`.
    fn days_since_modified_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::ModifiedJulianDate, Unit::Days)
    }

    /// Returns the number of days since J1950 as an `f64`.
    fn days_since_j1950(&self) -> f64 {
        self.julian_date(Epoch::J1950, Unit::Days)
    }

    /// Returns the number of days since J2000 as an `f64`.
    fn days_since_j2000(&self) -> f64 {
        self.julian_date(Epoch::J2000, Unit::Days)
    }

    /// Returns the number of years since the Julian epoch as an `f64`.
    fn years_since_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::JulianDate, Unit::Years)
    }

    /// Returns the number of years since the Modified Julian epoch as an `f64`.
    fn years_since_modified_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::ModifiedJulianDate, Unit::Years)
    }

    /// Returns the number of years since J1950 as an `f64`.
    fn years_since_j1950(&self) -> f64 {
        self.julian_date(Epoch::J1950, Unit::Years)
    }

    /// Returns the number of years since J2000 as an `f64`.
    fn years_since_j2000(&self) -> f64 {
        self.julian_date(Epoch::J2000, Unit::Years)
    }

    /// Returns the number of centuries since the Julian epoch as an `f64`.
    fn centuries_since_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::JulianDate, Unit::Centuries)
    }

    /// Returns the number of centuries since the Modified Julian epoch as an `f64`.
    fn centuries_since_modified_julian_epoch(&self) -> f64 {
        self.julian_date(Epoch::ModifiedJulianDate, Unit::Centuries)
    }

    /// Returns the number of centuries since J1950 as an `f64`.
    fn centuries_since_j1950(&self) -> f64 {
        self.julian_date(Epoch::J1950, Unit::Centuries)
    }

    /// Returns the number of centuries since J2000 as an `f64`.
    fn centuries_since_j2000(&self) -> f64 {
        self.julian_date(Epoch::J2000, Unit::Centuries)
    }
}
