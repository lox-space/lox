// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Useful constants for the `i64` type.

/*
 * Time conversion constants
 */

/// Number of seconds in a minute.
pub const SECONDS_PER_MINUTE: i64 = 60;

/// Number of seconds in an hour.
pub const SECONDS_PER_HOUR: i64 = 60 * SECONDS_PER_MINUTE;

/// Number of seconds in a day.
pub const SECONDS_PER_DAY: i64 = 24 * SECONDS_PER_HOUR;

/// Number of seconds in half a day.
pub const SECONDS_PER_HALF_DAY: i64 = SECONDS_PER_DAY / 2;

/// Number of seconds in a Julian year.
pub const SECONDS_PER_JULIAN_YEAR: i64 = 31_557_600;

/// Number of seconds in a Julian century.
pub const SECONDS_PER_JULIAN_CENTURY: i64 = SECONDS_PER_JULIAN_YEAR * 100;

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

/*
 * Julian date constants
 */

pub const SECONDS_BETWEEN_JD_AND_J2000: i64 = 211813488000;

pub const SECONDS_BETWEEN_MJD_AND_J2000: i64 = 4453444800;

pub const SECONDS_BETWEEN_J1950_AND_J2000: i64 = 1577880000;
