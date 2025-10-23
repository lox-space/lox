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
