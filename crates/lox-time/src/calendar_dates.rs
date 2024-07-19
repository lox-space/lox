/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

/*!
    `calendar_dates` exposes a concrete [Date] struct and the [CalendarDate] trait for working with
    human-readable dates.
*/

use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    str::FromStr,
    sync::OnceLock,
};

use lox_math::constants::f64::time::{self, SECONDS_PER_JULIAN_CENTURY};
use num::ToPrimitive;
use thiserror::Error;

use regex::Regex;

use crate::constants::i64::{SECONDS_PER_DAY, SECONDS_PER_HALF_DAY};
use crate::constants::julian_dates::{
    SECONDS_BETWEEN_J1950_AND_J2000, SECONDS_BETWEEN_JD_AND_J2000, SECONDS_BETWEEN_MJD_AND_J2000,
};
use crate::julian_dates::{Epoch, JulianDate, Unit};

fn iso_regex() -> &'static Regex {
    static ISO: OnceLock<Regex> = OnceLock::new();
    ISO.get_or_init(|| Regex::new(r"(?<year>-?\d{4,})-(?<month>\d{2})-(?<day>\d{2})").unwrap())
}

/// Error type returned when attempting to construct a [Date] from invalid inputs.
#[derive(Debug, Clone, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateError {
    #[error("invalid date `{0}-{1}-{2}`")]
    InvalidDate(i64, u8, u8),
    #[error("invalid ISO string `{0}`")]
    InvalidIsoString(String),
    #[error("day of year cannot be 366 for a non-leap year")]
    NonLeapYear,
}

/// The calendars supported by Lox.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Calendar {
    ProlepticJulian,
    Julian,
    Gregorian,
}

/// A calendar date.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date {
    calendar: Calendar,
    year: i64,
    month: u8,
    day: u8,
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl FromStr for Date {
    type Err = DateError;

    fn from_str(iso: &str) -> Result<Self, Self::Err> {
        Self::from_iso(iso)
    }
}

impl Default for Date {
    /// [Date] defaults to 2000-01-01 of the Gregorian calendar.
    fn default() -> Self {
        Self {
            calendar: Calendar::Gregorian,
            year: 2000,
            month: 1,
            day: 1,
        }
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Date {
    // The implementation of `Ord` for `Date` assumes that the `Calendar`s of the inputs date are
    // the same. This assumption is true at 2024-03-30, since the `Date` constructor doesn't allow
    // the creation of overlapping dates in different calendars, and `Date`s are immutable.
    //
    // If this changes, the implementation of `Ord` for `Date` must be updated too.
    // See https://github.com/lox-space/lox/issues/87.
    fn cmp(&self, other: &Self) -> Ordering {
        match self.year.cmp(&other.year) {
            Ordering::Equal => match self.month.cmp(&other.month) {
                Ordering::Equal => self.day.cmp(&other.day),
                other => other,
            },
            other => other,
        }
    }
}

const LAST_PROLEPTIC_JULIAN_DAY_J2K: i64 = -730122;
const LAST_JULIAN_DAY_J2K: i64 = -152384;

impl Date {
    pub fn calendar(&self) -> Calendar {
        self.calendar
    }

    pub fn year(&self) -> i64 {
        self.year
    }

    pub fn month(&self) -> u8 {
        self.month
    }

    pub fn day(&self) -> u8 {
        self.day
    }

    /// Construct a new [Date] from a year, month and day. The [Calendar] is inferred from the input
    /// fields.
    ///
    /// # Errors
    ///
    /// - [DateError::InvalidDate] if the input fields do not represent a valid date.
    pub fn new(year: i64, month: u8, day: u8) -> Result<Self, DateError> {
        if !(1..=12).contains(&month) {
            Err(DateError::InvalidDate(year, month, day))
        } else {
            let calendar = calendar(year, month, day);
            let check = Date::from_days_since_j2000(j2000_day_number(calendar, year, month, day));

            if check.year() != year || check.month() != month || check.day() != day {
                Err(DateError::InvalidDate(year, month, day))
            } else {
                Ok(Date {
                    calendar,
                    year,
                    month,
                    day,
                })
            }
        }
    }

    /// Constructs a new [Date] from an ISO 8601 string.
    ///
    /// # Errors
    ///
    /// - [DateError::InvalidIsoString] if the input string does not contain a valid ISO 8601 date.
    /// - [DateError::InvalidDate] if the date parsed from the ISO 8601 string is invalid.
    pub fn from_iso(iso: &str) -> Result<Self, DateError> {
        let caps = iso_regex()
            .captures(iso)
            .ok_or(DateError::InvalidIsoString(iso.to_owned()))?;
        let year: i64 = caps["year"]
            .parse()
            .map_err(|_| DateError::InvalidIsoString(iso.to_owned()))?;
        let month = caps["month"]
            .parse()
            .map_err(|_| DateError::InvalidIsoString(iso.to_owned()))?;
        let day = caps["day"]
            .parse()
            .map_err(|_| DateError::InvalidIsoString(iso.to_owned()))?;
        Date::new(year, month, day)
    }

    /// Constructs a new [Date] from a signed number of days since J2000. The [Calendar] is
    /// inferred.
    pub fn from_days_since_j2000(days: i64) -> Self {
        let calendar = if days < LAST_JULIAN_DAY_J2K {
            if days > LAST_PROLEPTIC_JULIAN_DAY_J2K {
                Calendar::Julian
            } else {
                Calendar::ProlepticJulian
            }
        } else {
            Calendar::Gregorian
        };

        let year = find_year(calendar, days);
        let leap = is_leap_year(calendar, year);
        let day_of_year = (days - last_day_of_year_j2k(calendar, year - 1)) as u16;
        let month = find_month(day_of_year, leap);
        let day = find_day(day_of_year, month, leap).unwrap_or_else(|err| {
            unreachable!("{} is not a valid day of the year: {}", day_of_year, err)
        });

        Date {
            calendar,
            year,
            month,
            day,
        }
    }

    /// Constructs a new [Date] from a signed number of seconds since J2000. The [Calendar] is
    /// inferred.
    pub fn from_seconds_since_j2000(seconds: i64) -> Self {
        let seconds = seconds + SECONDS_PER_HALF_DAY;
        let mut time = seconds % SECONDS_PER_DAY;
        if time < 0 {
            time += SECONDS_PER_DAY;
        }
        let days = (seconds - time) / SECONDS_PER_DAY;
        Self::from_days_since_j2000(days)
    }

    /// Constructs a new [Date] from a year and a day number within that year. The [Calendar] is
    /// inferred.
    ///
    /// # Errors
    ///
    /// - [DateError::NonLeapYear] if the input day number is 366 and the year is not a leap year.
    pub fn from_day_of_year(year: i64, day_of_year: u16) -> Result<Self, DateError> {
        let calendar = calendar(year, 1, 1);
        let leap = is_leap_year(calendar, year);
        let month = find_month(day_of_year, leap);
        let day = find_day(day_of_year, month, leap)?;

        Ok(Date {
            calendar,
            year,
            month,
            day,
        })
    }

    /// Returns the day number of `self` relative to J2000.
    pub fn j2000_day_number(&self) -> i64 {
        j2000_day_number(self.calendar, self.year, self.month, self.day)
    }
}

impl JulianDate for Date {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        let mut seconds = j2000_day_number(self.calendar, self.year, self.month, self.day)
            * SECONDS_PER_DAY
            - SECONDS_PER_HALF_DAY;
        seconds = match epoch {
            Epoch::JulianDate => seconds + SECONDS_BETWEEN_JD_AND_J2000,
            Epoch::ModifiedJulianDate => seconds + SECONDS_BETWEEN_MJD_AND_J2000,
            Epoch::J1950 => seconds + SECONDS_BETWEEN_J1950_AND_J2000,
            Epoch::J2000 => seconds,
        };
        let seconds = seconds as f64;

        match unit {
            Unit::Seconds => seconds,
            Unit::Days => seconds / time::SECONDS_PER_DAY,
            Unit::Centuries => seconds / SECONDS_PER_JULIAN_CENTURY,
        }
    }
}

fn find_year(calendar: Calendar, j2000day: i64) -> i64 {
    match calendar {
        Calendar::ProlepticJulian => -((-4 * j2000day - 2920488) / 1461),
        Calendar::Julian => -((-4 * j2000day - 2921948) / 1461),
        Calendar::Gregorian => {
            let year = (400 * j2000day + 292194288) / 146097;
            if j2000day <= last_day_of_year_j2k(Calendar::Gregorian, year - 1) {
                year - 1
            } else {
                year
            }
        }
    }
}

fn last_day_of_year_j2k(calendar: Calendar, year: i64) -> i64 {
    match calendar {
        Calendar::ProlepticJulian => 365 * year + (year + 1) / 4 - 730123,
        Calendar::Julian => 365 * year + year / 4 - 730122,
        Calendar::Gregorian => 365 * year + year / 4 - year / 100 + year / 400 - 730120,
    }
}

fn is_leap_year(calendar: Calendar, year: i64) -> bool {
    match calendar {
        Calendar::ProlepticJulian | Calendar::Julian => year % 4 == 0,
        Calendar::Gregorian => year % 4 == 0 && (year % 400 == 0 || year % 100 != 0),
    }
}

const PREVIOUS_MONTH_END_DAY_LEAP: [u16; 12] =
    [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335];

const PREVIOUS_MONTH_END_DAY: [u16; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

fn find_month(day_in_year: u16, is_leap: bool) -> u8 {
    let offset = if is_leap { 313 } else { 323 };
    let month = if day_in_year < 32 {
        1
    } else {
        (10 * day_in_year + offset) / 306
    };
    month
        .to_u8()
        .unwrap_or_else(|| unreachable!("month could not be represented as u8: {}", month))
}

fn find_day(day_in_year: u16, month: u8, is_leap: bool) -> Result<u8, DateError> {
    if !is_leap && day_in_year > 365 {
        Err(DateError::NonLeapYear)
    } else {
        let previous_days = if is_leap {
            PREVIOUS_MONTH_END_DAY_LEAP
        } else {
            PREVIOUS_MONTH_END_DAY
        };
        let day = day_in_year - previous_days[(month - 1) as usize];
        Ok(day
            .to_u8()
            .unwrap_or_else(|| unreachable!("day could not be represented as u8: {}", day)))
    }
}

fn find_day_in_year(month: u8, day: u8, is_leap: bool) -> u16 {
    let previous_days = if is_leap {
        PREVIOUS_MONTH_END_DAY_LEAP
    } else {
        PREVIOUS_MONTH_END_DAY
    };
    day as u16 + previous_days[(month - 1) as usize]
}

fn calendar(year: i64, month: u8, day: u8) -> Calendar {
    if year < 1583 {
        if year < 1 {
            Calendar::ProlepticJulian
        } else if year < 1582 || month < 10 || (month < 11 && day < 5) {
            Calendar::Julian
        } else {
            Calendar::Gregorian
        }
    } else {
        Calendar::Gregorian
    }
}

fn j2000_day_number(calendar: Calendar, year: i64, month: u8, day: u8) -> i64 {
    let d1 = last_day_of_year_j2k(calendar, year - 1);
    let d2 = find_day_in_year(month, day, is_leap_year(calendar, year));
    d1 + d2 as i64
}

/// `CalendarDate` allows any date-time format to report its date in a human-readable way.
pub trait CalendarDate {
    fn date(&self) -> Date;

    fn year(&self) -> i64 {
        self.date().year()
    }

    fn month(&self) -> u8 {
        self.date().month()
    }

    fn day(&self) -> u8 {
        self.date().day()
    }

    fn day_of_year(&self) -> u16 {
        let date = self.date();
        let leap = is_leap_year(date.calendar(), date.year());
        find_day_in_year(date.month(), date.day(), leap)
    }
}

#[cfg(test)]
mod tests {
    use lox_math::constants::f64::time::DAYS_PER_JULIAN_CENTURY;
    use rstest::rstest;

    use crate::calendar_dates::{Calendar, Date};

    use super::*;

    #[rstest]
    #[case::equal_same_calendar(Date { calendar: Calendar::Gregorian, year: 2000, month: 1, day: 1}, Date { calendar: Calendar::Gregorian, year: 2000, month: 1, day: 1}, Ordering::Equal)]
    #[case::equal_different_calendar(Date { calendar: Calendar::Gregorian, year: 2000, month: 1, day: 1}, Date { calendar: Calendar::Julian, year: 2000, month: 1, day: 1}, Ordering::Equal)]
    #[case::less_than_year(Date { calendar: Calendar::Gregorian, year: 1999, month: 1, day: 1}, Date { calendar: Calendar::Gregorian, year: 2000, month: 1, day: 1}, Ordering::Less)]
    #[case::less_than_month(Date { calendar: Calendar::Gregorian, year: 2000, month: 1, day: 1}, Date { calendar: Calendar::Gregorian, year: 2000, month: 2, day: 1}, Ordering::Less)]
    #[case::less_than_day(Date { calendar: Calendar::Gregorian, year: 2000, month: 1, day: 1}, Date { calendar: Calendar::Gregorian, year: 2000, month: 1, day: 2}, Ordering::Less)]
    #[case::greater_than_year(Date { calendar: Calendar::Gregorian, year: 2001, month: 1, day: 1}, Date { calendar: Calendar::Gregorian, year: 2000, month: 1, day: 1}, Ordering::Greater)]
    #[case::greater_than_month(Date { calendar: Calendar::Gregorian, year: 2000, month: 2, day: 1}, Date { calendar: Calendar::Gregorian, year: 2000, month: 1, day: 1}, Ordering::Greater)]
    #[case::greater_than_day(Date { calendar: Calendar::Gregorian, year: 2000, month: 1, day: 2}, Date { calendar: Calendar::Gregorian, year: 2000, month: 1, day: 1}, Ordering::Greater)]
    fn test_date_ord(#[case] lhs: Date, #[case] rhs: Date, #[case] expected: Ordering) {
        assert_eq!(expected, lhs.cmp(&rhs));
    }

    #[rstest]
    #[case::j2000("2000-01-01", Date { calendar: Calendar::Gregorian, year: 2000, month: 1, day: 1})]
    #[case::j2000("0000-01-01", Date { calendar: Calendar::ProlepticJulian, year: 0, month: 1, day: 1})]
    fn test_date_iso(#[case] str: &str, #[case] expected: Date) {
        let actual = Date::from_iso(str).expect("date should parse");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_date_from_day_of_year() {
        let date = Date::from_day_of_year(2000, 366).unwrap();
        assert_eq!(date.year(), 2000);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 31);
    }

    #[test]
    fn test_date_from_invalid_day_of_year() {
        let actual = Date::from_day_of_year(2001, 366);
        let expected = Err(DateError::NonLeapYear);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_date_jd_epoch() {
        let date = Date::default();
        assert_eq!(date.days_since_julian_epoch(), 2451544.5);
    }

    #[test]
    fn test_date_julian_date() {
        let date = Date::default();
        assert_eq!(date.days_since_julian_epoch(), 2451544.5);

        let date = Date::new(2100, 1, 1).unwrap();
        assert_eq!(
            date.seconds_since_j2000(),
            SECONDS_PER_JULIAN_CENTURY - SECONDS_PER_HALF_DAY as f64
        );
        assert_eq!(date.days_since_j2000(), DAYS_PER_JULIAN_CENTURY - 0.5);
        assert_eq!(
            date.centuries_since_j2000(),
            1.0 - 0.5 / DAYS_PER_JULIAN_CENTURY
        );
        assert_eq!(
            date.centuries_since_j1950(),
            1.5 - 0.5 / DAYS_PER_JULIAN_CENTURY
        );
        assert_eq!(
            date.centuries_since_modified_julian_epoch(),
            2.411211498973306 - 0.5 / DAYS_PER_JULIAN_CENTURY
        );
        assert_eq!(
            date.centuries_since_julian_epoch(),
            68.11964407939767 - 0.5 / DAYS_PER_JULIAN_CENTURY
        );
    }
}
