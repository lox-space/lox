// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Useful constants for the `f64` type.

/*
 * Time conversion constants
 */

/// Number of days in a Julian year.
pub const DAYS_PER_JULIAN_YEAR: f64 = 365.25;

/// Number of days in a Julian century.
pub const DAYS_PER_JULIAN_CENTURY: f64 = DAYS_PER_JULIAN_YEAR * 100.0;

/// Number of seconds in a minute.
pub const SECONDS_PER_MINUTE: f64 = 60.0;

/// Number of seconds in an hour.
pub const SECONDS_PER_HOUR: f64 = SECONDS_PER_MINUTE * 60.0;

/// Number of seconds in a day.
pub const SECONDS_PER_HALF_DAY: f64 = SECONDS_PER_HOUR * 12.0;

/// Number of seconds in a day.
pub const SECONDS_PER_DAY: f64 = SECONDS_PER_HOUR * 24.0;

/// Number of seconds in a Julian year (365.25 days).
pub const SECONDS_PER_JULIAN_YEAR: f64 = SECONDS_PER_DAY * DAYS_PER_JULIAN_YEAR;

/// Number of seconds in a Julian century.
pub const SECONDS_PER_JULIAN_CENTURY: f64 = SECONDS_PER_JULIAN_YEAR * 100.0;

/// Number of seconds in a femtosecond.
pub const SECONDS_PER_FEMTOSECOND: f64 = 1e-15;

/// Number of seconds in an attosecond.
pub const SECONDS_PER_ATTOSECOND: f64 = 1e-18;

/*
 * Julian date constants
 */

/// Modified Julian Date at J2000 epoch.
pub const MJD_J2000: f64 = 51544.5;

/// The number of seconds between the Julian Epoch and the J2000 Epoch.
pub const SECONDS_BETWEEN_JD_AND_J2000: f64 = 211813488000.0;

/// The number of seconds between the Modified Julian Epoch and the J2000 Epoch.
pub const SECONDS_BETWEEN_MJD_AND_J2000: f64 = 4453444800.0;

/// The number of seconds between the J1950 Epoch and the J2000 Epoch.
pub const SECONDS_BETWEEN_J1950_AND_J2000: f64 = 1577880000.0;

/*
 * Physical constants
 */

pub const ROTATION_RATE_EARTH: f64 = 7.2921150e-5;
