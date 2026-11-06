// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Well-known time constants.

use crate::time::{calendar_dates::Date, deltas::TimeDelta};

/// Number of seconds in a minute.
pub const SECONDS_PER_MINUTE: i64 = 60;

/// Number of seconds in an hour.
pub const SECONDS_PER_HOUR: i64 = 60 * SECONDS_PER_MINUTE;

/// Number of seconds in a day.
pub const SECONDS_PER_DAY: i64 = 24 * SECONDS_PER_HOUR;

/// Number of seconds in half a day.
pub const SECONDS_PER_HALF_DAY: i64 = SECONDS_PER_DAY / 2;

/// Number of seconds in a Julian year (365.25 days).
pub const SECONDS_PER_JULIAN_YEAR: i64 = 31_557_600;

/// Number of seconds in a Julian century.
pub const SECONDS_PER_JULIAN_CENTURY: i64 = SECONDS_PER_JULIAN_YEAR * 100;

/// Number of days in a Julian year.
pub const DAYS_PER_JULIAN_YEAR: f64 = 365.25;

/// Number of days in a Julian century.
pub const DAYS_PER_JULIAN_CENTURY: f64 = DAYS_PER_JULIAN_YEAR * 100.0;

/// Number of attoseconds in a second (1e18).
pub const ATTOSECONDS_IN_SECOND: i64 = 1_000_000_000_000_000_000;

/// Number of attoseconds in a millisecond (1e15).
pub const ATTOSECONDS_IN_MILLISECOND: i64 = 1_000_000_000_000_000;

/// Number of attoseconds in a microsecond (1e12).
pub const ATTOSECONDS_IN_MICROSECOND: i64 = 1_000_000_000_000;

/// Number of attoseconds in a nanosecond (1e9).
pub const ATTOSECONDS_IN_NANOSECOND: i64 = 1_000_000_000;

/// Number of attoseconds in a picosecond (1e6).
pub const ATTOSECONDS_IN_PICOSECOND: i64 = 1_000_000;

/// Number of attoseconds in a femtosecond (1e3).
pub const ATTOSECONDS_IN_FEMTOSECOND: i64 = 1_000;

/// The Modified Julian Date of J2000.
pub const MJD_J2000: f64 = 51544.5;

/// Seconds between the Julian Date epoch (JD 0) and J2000.
pub const SECONDS_BETWEEN_JD_AND_J2000: i64 = 211813488000;

/// Seconds between the Modified Julian Date epoch and J2000.
pub const SECONDS_BETWEEN_MJD_AND_J2000: i64 = 4453444800;

/// Seconds between J1950 and J2000.
pub const SECONDS_BETWEEN_J1950_AND_J2000: i64 = 1577880000;

/// The duration from J2000 (2000-01-01T12:00:00) to the Unix epoch (1970-01-01T00:00:00).
pub const UNIX_EPOCH: TimeDelta = Date::new_unchecked(1970, 1, 1).to_delta();
