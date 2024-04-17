/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    sync::OnceLock,
};

use lox_utils::constants::f64::time::{self, SECONDS_PER_JULIAN_CENTURY};
use num::ToPrimitive;
use thiserror::Error;

use regex::Regex;

use crate::constants::i64::SECONDS_PER_DAY;
use crate::constants::julian_dates::{
    SECONDS_BETWEEN_J1950_AND_J2000, SECONDS_BETWEEN_JD_AND_J2000, SECONDS_BETWEEN_MJD_AND_J2000,
};
use crate::julian_dates::{Epoch, JulianDate, Unit};

fn iso_regex() -> &'static Regex {
    static ISO: OnceLock<Regex> = OnceLock::new();
    ISO.get_or_init(|| Regex::new(r"(?<year>-?\d{4,})-(?<month>\d{2})-(?<day>\d{2})").unwrap())
}

#[derive(Debug, Clone, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateError {
    #[error("invalid date `{0}-{1}-{2}`")]
    InvalidDate(i64, u8, u16),
    #[error("invalid ISO string `{0}`")]
    InvalidIsoString(String),
    #[error("day of year cannot be 366 for a non-leap year")]
    NonLeapYear,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Calendar {
    ProlepticJulian,
    Julian,
    Gregorian,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date {
    calendar: Calendar,
    year: i64,
    month: u8,
    day: u16,
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl Default for Date {
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

    pub fn day(&self) -> u16 {
        self.day
    }

    pub fn new(year: i64, month: u8, day: u16) -> Result<Self, DateError> {
        if !(1..=12).contains(&month) {
            Err(DateError::InvalidDate(year, month, day))
        } else {
            let calendar = calendar(year, month, day);
            let check = Date::from_days_since_j2000(days_since_j2000(calendar, year, month, day));

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
        let day = find_day(day_of_year, month, leap)
            .unwrap_or_else(|_| unreachable!("day of year should be valid"));

        Date {
            calendar,
            year,
            month,
            day,
        }
    }

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
}

impl JulianDate for Date {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        let mut seconds =
            days_since_j2000(self.calendar, self.year, self.month, self.day) * SECONDS_PER_DAY;
        seconds = match epoch {
            Epoch::JulianDate => seconds + SECONDS_BETWEEN_JD_AND_J2000,
            Epoch::ModifiedJulianDate => seconds + SECONDS_BETWEEN_MJD_AND_J2000,
            Epoch::J1950 => seconds + SECONDS_BETWEEN_J1950_AND_J2000,
            Epoch::J2000 => seconds,
        };
        let seconds = seconds
            .to_f64()
            .unwrap_or_else(|| unreachable!("should be representable as f64"));

        match unit {
            Unit::Seconds => seconds,
            Unit::Days => seconds / time::SECONDS_PER_DAY,
            Unit::Centuries => seconds / SECONDS_PER_JULIAN_CENTURY,
        }
    }

    fn two_part_julian_date(&self) -> (f64, f64) {
        (self.julian_date(Epoch::JulianDate, Unit::Days), 0.0)
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
    if day_in_year < 32 {
        1
    } else {
        ((10 * day_in_year + offset) / 306)
            .to_u8()
            .unwrap_or_else(|| unreachable!("should be representable as u8"))
    }
}

fn find_day(day_in_year: u16, month: u8, is_leap: bool) -> Result<u16, DateError> {
    if !is_leap && day_in_year > 365 {
        Err(DateError::NonLeapYear)
    } else {
        let previous_days = if is_leap {
            PREVIOUS_MONTH_END_DAY_LEAP
        } else {
            PREVIOUS_MONTH_END_DAY
        };
        Ok(day_in_year - previous_days[(month - 1) as usize])
    }
}

fn find_day_in_year(month: u8, day: u16, is_leap: bool) -> u16 {
    let previous_days = if is_leap {
        PREVIOUS_MONTH_END_DAY_LEAP
    } else {
        PREVIOUS_MONTH_END_DAY
    };
    day + previous_days[(month - 1) as usize]
}

fn calendar(year: i64, month: u8, day: u16) -> Calendar {
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

fn days_since_j2000(calendar: Calendar, year: i64, month: u8, day: u16) -> i64 {
    let d1 = last_day_of_year_j2k(calendar, year - 1);
    let d2 = find_day_in_year(month, day, is_leap_year(calendar, year));
    d1 + d2 as i64
}

/// CalendarDate allows continuous time formats to report their date in their respective calendar.
pub trait CalendarDate {
    fn calendar_date(&self) -> Date;
}

#[cfg(test)]
mod tests {
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
}
