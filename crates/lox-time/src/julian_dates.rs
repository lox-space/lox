// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

/*!
    Module `julian_dates` exposes the [JulianDate] trait for expressing arbitrary time
    representations as Julian dates relative to standard [Epoch]s and in a variety of [Unit]s.
*/

use crate::{deltas::TimeDelta, subsecond::Subsecond};

pub const SECONDS_BETWEEN_JD_AND_J2000: i64 = 211813488000;

pub const SECONDS_BETWEEN_MJD_AND_J2000: i64 = 4453444800;

pub const SECONDS_BETWEEN_J1950_AND_J2000: i64 = 1577880000;

pub const SECONDS_BETWEEN_J1977_AND_J2000: i64 = 725803200;

/// 4713 BC January 1 12:00
pub const J0: TimeDelta = TimeDelta {
    seconds: -SECONDS_BETWEEN_JD_AND_J2000,
    subsecond: Subsecond(0.0),
};

/// 1977 January 1 00:00, at which the following are equal:
/// * 1977-01-01T00:00:00.000 TAI
/// * 1977-01-01T00:00:32.184 TT
/// * 1977-01-01T00:00:32.184 TCG
/// * 1977-01-01T00:00:32.184 TCB
pub const J77: TimeDelta = TimeDelta {
    seconds: -SECONDS_BETWEEN_J1977_AND_J2000,
    subsecond: Subsecond(0.0),
};

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
