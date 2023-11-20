/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::errors::LoxError;
use num::ToPrimitive;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Calendar {
    ProlepticJulian,
    Julian,
    Gregorian,
}

#[derive(Debug, Copy, Clone)]
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

    pub fn new(year: i64, month: i64, day: i64) -> Result<Self, LoxError> {
        if !(1..=12).contains(&month) {
            Err(LoxError::InvalidDate(year, month, day))
        } else {
            let calendar = get_calendar(year, month, day);
            let check = Date::from_days(j2000(calendar, year, month, day))?;

            if check.year() != year || check.month() != month || check.day() != day {
                Err(LoxError::InvalidDate(year, month, day))
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

    pub fn from_days(offset: i64) -> Result<Self, LoxError> {
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

#[derive(Debug, Copy, Clone, Default)]
pub struct Time {
    hour: i64,
    minute: i64,
    second: i64,
    milli: i64,
    micro: i64,
    nano: i64,
    pico: i64,
    femto: i64,
    atto: i64,
}

impl Time {
    pub fn new(hour: i64, minute: i64, second: i64) -> Result<Self, LoxError> {
        if !(0..24).contains(&hour) || !(0..60).contains(&minute) || !(0..61).contains(&second) {
            Err(LoxError::InvalidTime(hour, minute, second))
        } else {
            Ok(Self {
                hour,
                minute,
                second,
                ..Default::default()
            })
        }
    }

    pub fn milli(mut self, milli: i64) -> Self {
        self.milli = milli;
        self
    }

    pub fn micro(mut self, micro: i64) -> Self {
        self.micro = micro;
        self
    }

    pub fn nano(mut self, nano: i64) -> Self {
        self.nano = nano;
        self
    }

    pub fn pico(mut self, pico: i64) -> Self {
        self.pico = pico;
        self
    }

    pub fn femto(mut self, femto: i64) -> Self {
        self.femto = femto;
        self
    }

    pub fn atto(mut self, atto: i64) -> Self {
        self.atto = atto;
        self
    }

    pub fn from_seconds(hour: i64, minute: i64, seconds: f64) -> Result<Self, LoxError> {
        if !(0.0..61.0).contains(&seconds) {
            return Err(LoxError::InvalidSeconds(hour, minute, seconds));
        }
        let sub = split_seconds(seconds.fract()).unwrap();
        let second = seconds.round().to_i64().unwrap();
        Self::new(hour, minute, second)?;
        Ok(Self {
            hour,
            minute,
            second,
            milli: sub[0],
            micro: sub[1],
            nano: sub[2],
            pico: sub[3],
            femto: sub[4],
            atto: sub[5],
        })
    }

    pub fn hour(&self) -> i64 {
        self.hour
    }

    pub fn minute(&self) -> i64 {
        self.minute
    }

    pub fn second(&self) -> i64 {
        self.second
    }

    pub fn attosecond(&self) -> i64 {
        self.milli * i64::pow(10, 15)
            + self.micro * i64::pow(10, 12)
            + self.nano * i64::pow(10, 9)
            + self.pico * i64::pow(10, 6)
            + self.femto * i64::pow(10, 3)
            + self.atto
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DateTime {
    date: Date,
    time: Time,
}

impl DateTime {
    pub fn new(date: Date, time: Time) -> Self {
        Self { date, time }
    }

    pub fn date(&self) -> Date {
        self.date
    }

    pub fn time(&self) -> Time {
        self.time
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

fn find_day(day_in_year: i64, month: i64, is_leap: bool) -> Result<i64, LoxError> {
    if !is_leap && day_in_year > 365 {
        Err(LoxError::NonLeapYear)
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

fn split_seconds(seconds: f64) -> Option<[i64; 6]> {
    if !(0.0..1.0).contains(&seconds) {
        return None;
    }
    let mut atto = (seconds * 1e18).to_i64()?;
    let mut parts: [i64; 6] = [0; 6];
    for (i, exponent) in (3..18).step_by(3).rev().enumerate() {
        let factor = i64::pow(10, exponent);
        parts[i] = atto / factor;
        atto -= parts[i] * factor;
    }
    parts[5] = atto / 10 * 10;
    Some(parts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_test_split_seconds(s in 0.0..1.0) {
            prop_assert!(split_seconds(s).is_some())
        }
    }

    #[test]
    fn test_sub_second() {
        let s1 = split_seconds(0.123).expect("seconds should be valid");
        assert_eq!(123, s1[0]);
        assert_eq!(0, s1[1]);
        assert_eq!(0, s1[2]);
        assert_eq!(0, s1[3]);
        assert_eq!(0, s1[4]);
        assert_eq!(0, s1[5]);
        let s2 = split_seconds(0.123_456).expect("seconds should be valid");
        assert_eq!(123, s2[0]);
        assert_eq!(456, s2[1]);
        assert_eq!(0, s2[2]);
        assert_eq!(0, s2[3]);
        assert_eq!(0, s2[4]);
        assert_eq!(0, s2[5]);
        let s3 = split_seconds(0.123_456_789).expect("seconds should be valid");
        assert_eq!(123, s3[0]);
        assert_eq!(456, s3[1]);
        assert_eq!(789, s3[2]);
        assert_eq!(0, s3[3]);
        assert_eq!(0, s3[4]);
        assert_eq!(0, s3[5]);
        let s4 = split_seconds(0.123_456_789_123).expect("seconds should be valid");
        assert_eq!(123, s4[0]);
        assert_eq!(456, s4[1]);
        assert_eq!(789, s4[2]);
        assert_eq!(123, s4[3]);
        assert_eq!(0, s4[4]);
        assert_eq!(0, s4[5]);
        let s5 = split_seconds(0.123_456_789_123_456).expect("seconds should be valid");
        assert_eq!(123, s5[0]);
        assert_eq!(456, s5[1]);
        assert_eq!(789, s5[2]);
        assert_eq!(123, s5[3]);
        assert_eq!(456, s5[4]);
        assert_eq!(0, s5[5]);
        let s6 = split_seconds(0.123_456_789_123_456_78).expect("seconds should be valid");
        assert_eq!(123, s6[0]);
        assert_eq!(456, s6[1]);
        assert_eq!(789, s6[2]);
        assert_eq!(123, s6[3]);
        assert_eq!(456, s6[4]);
        assert_eq!(780, s6[5]);
        let s7 = split_seconds(0.000_000_000_000_000_01).expect("seconds should be valid");
        assert_eq!(0, s7[0]);
        assert_eq!(0, s7[1]);
        assert_eq!(0, s7[2]);
        assert_eq!(0, s7[3]);
        assert_eq!(0, s7[4]);
        assert_eq!(10, s7[5]);
    }

    #[test]
    fn test_illegal_split_second() {
        assert!(split_seconds(2.0).is_none());
        assert!(split_seconds(-0.2).is_none());
    }

    #[test]
    fn test_date_new_unchecked() {
        let date = Date::new_unchecked(Calendar::Gregorian, 2021, 1, 1);
        assert_eq!(Calendar::Gregorian, date.calendar);
        assert_eq!(2021, date.year);
        assert_eq!(1, date.month);
        assert_eq!(1, date.day);
    }
}
