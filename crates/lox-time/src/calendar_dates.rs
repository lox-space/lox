/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::errors::LoxTimeError;

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
    month: i64,
    day: i64,
}

const LAST_PROLEPTIC_JULIAN_DAY_J2K: i64 = -730122;
const LAST_JULIAN_DAY_J2K: i64 = -152384;

impl Date {
    /// Create a Date from raw parts. This is particularly useful for generating test dates that
    /// are known to be correct, without exposing the internals of the Date struct.
    #[cfg(test)]
    pub(crate) fn new_unchecked(calendar: Calendar, year: i64, month: i64, day: i64) -> Self {
        Self {
            calendar,
            year,
            month,
            day,
        }
    }

    pub fn calendar(&self) -> Calendar {
        self.calendar
    }

    pub fn year(&self) -> i64 {
        self.year
    }

    pub fn month(&self) -> i64 {
        self.month
    }

    pub fn day(&self) -> i64 {
        self.day
    }

    pub fn new(year: i64, month: i64, day: i64) -> Result<Self, LoxTimeError> {
        if !(1..=12).contains(&month) {
            Err(LoxTimeError::InvalidDate(year, month, day))
        } else {
            let calendar = get_calendar(year, month, day);
            let check = Date::from_days(j2000(calendar, year, month, day))?;

            if check.year() != year || check.month() != month || check.day() != day {
                Err(LoxTimeError::InvalidDate(year, month, day))
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

    pub fn from_days(offset: i64) -> Result<Self, LoxTimeError> {
        let calendar = if offset < LAST_JULIAN_DAY_J2K {
            if offset > LAST_PROLEPTIC_JULIAN_DAY_J2K {
                Calendar::Julian
            } else {
                Calendar::ProlepticJulian
            }
        } else {
            Calendar::Gregorian
        };

        let year = find_year(calendar, offset);
        let leap = is_leap(calendar, year);
        let day_in_year = offset - last_day_of_year_j2k(calendar, year - 1);
        let month = find_month(day_in_year, leap);
        let day = find_day(day_in_year, month, leap)?;

        Ok(Date {
            calendar,
            year,
            month,
            day,
        })
    }

    pub fn j2000(&self) -> i64 {
        j2000(self.calendar, self.year, self.month, self.day)
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

fn is_leap(calendar: Calendar, year: i64) -> bool {
    match calendar {
        Calendar::ProlepticJulian | Calendar::Julian => year % 4 == 0,
        Calendar::Gregorian => year % 4 == 0 && (year % 400 == 0 || year % 100 != 0),
    }
}

const PREVIOUS_MONTH_END_DAY_LEAP: [i64; 12] =
    [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335];

const PREVIOUS_MONTH_END_DAY: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

fn find_month(day_in_year: i64, is_leap: bool) -> i64 {
    let offset = if is_leap { 313 } else { 323 };
    if day_in_year < 32 {
        1
    } else {
        (10 * day_in_year + offset) / 306
    }
}

fn find_day(day_in_year: i64, month: i64, is_leap: bool) -> Result<i64, LoxTimeError> {
    if !is_leap && day_in_year > 365 {
        Err(LoxTimeError::NonLeapYear)
    } else {
        let previous_days = if is_leap {
            PREVIOUS_MONTH_END_DAY_LEAP
        } else {
            PREVIOUS_MONTH_END_DAY
        };
        Ok(day_in_year - previous_days[(month - 1) as usize])
    }
}

fn find_day_in_year(month: i64, day: i64, is_leap: bool) -> i64 {
    let previous_days = if is_leap {
        PREVIOUS_MONTH_END_DAY_LEAP
    } else {
        PREVIOUS_MONTH_END_DAY
    };
    day + previous_days[(month - 1) as usize]
}

fn get_calendar(year: i64, month: i64, day: i64) -> Calendar {
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

fn j2000(calendar: Calendar, year: i64, month: i64, day: i64) -> i64 {
    let d1 = last_day_of_year_j2k(calendar, year - 1);
    let d2 = find_day_in_year(month, day, is_leap(calendar, year));
    d1 + d2
}

/// CalendarDate allows continuous time formats to report their date in their respective calendar.
pub trait CalendarDate {
    fn calendar_date(&self) -> Date;
}

#[cfg(test)]
mod tests {
    use crate::calendar_dates::{Calendar, Date};

    #[test]
    fn test_date_new_unchecked() {
        let date = Date::new_unchecked(Calendar::Gregorian, 2021, 1, 1);
        assert_eq!(Calendar::Gregorian, date.calendar);
        assert_eq!(2021, date.year);
        assert_eq!(1, date.month);
        assert_eq!(1, date.day);
    }
}
