/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

/*!
    Module `time_of_day` exposes the concrete representation of a time of day with leap second
    support, [TimeOfDay].

    The [CivilTime] trait supports arbitrary time representations to express themselves as a
    human-readable time of day.
*/

use std::fmt::Display;
use std::str::FromStr;
use std::{cmp::Ordering, sync::OnceLock};

use num::ToPrimitive;
use regex::Regex;
use thiserror::Error;

use crate::{
    constants::i64::{SECONDS_PER_DAY, SECONDS_PER_HALF_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE},
    subsecond::{InvalidSubsecond, Subsecond},
};

fn iso_regex() -> &'static Regex {
    static ISO: OnceLock<Regex> = OnceLock::new();
    ISO.get_or_init(|| {
        Regex::new(r"(?<hour>\d{2}):(?<minute>\d{2}):(?<second>\d{2})(?<subsecond>\.\d+)?").unwrap()
    })
}

/// Error type returned when attempting to construct a [TimeOfDay] with a greater number of
/// floating-point seconds than are in a day.
#[derive(Debug, Copy, Clone, Error)]
#[error("seconds must be in the range [0.0..86401.0) but was {0}")]
pub struct InvalidSeconds(f64);

impl PartialOrd for InvalidSeconds {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InvalidSeconds {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialEq for InvalidSeconds {
    fn eq(&self, other: &Self) -> bool {
        self.0.total_cmp(&other.0) == Ordering::Equal
    }
}

impl Eq for InvalidSeconds {}

/// Error type returned when attempting to construct a [TimeOfDay] from invalid components.
#[derive(Debug, Clone, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimeOfDayError {
    #[error("hour must be in the range [0..24) but was {0}")]
    InvalidHour(u8),
    #[error("minute must be in the range [0..60) but was {0}")]
    InvalidMinute(u8),
    #[error("second must be in the range [0..61) but was {0}")]
    InvalidSecond(u8),
    #[error("second must be in the range [0..86401) but was {0}")]
    InvalidSecondOfDay(u64),
    #[error(transparent)]
    InvalidSeconds(#[from] InvalidSeconds),
    #[error("leap seconds are only valid at the end of the day")]
    InvalidLeapSecond,
    #[error(transparent)]
    InvalidSubsecond(#[from] InvalidSubsecond),
    #[error("invalid ISO string `{0}`")]
    InvalidIsoString(String),
}

/// `CivilTime` is the trait by which high-precision time representations expose human-readable time
/// components.
pub trait CivilTime {
    fn time(&self) -> TimeOfDay;

    fn hour(&self) -> u8 {
        self.time().hour()
    }

    fn minute(&self) -> u8 {
        self.time().minute()
    }

    fn second(&self) -> u8 {
        self.time().second()
    }

    fn decimal_seconds(&self) -> f64 {
        self.time().subsecond().0 + self.time().second() as f64
    }

    fn millisecond(&self) -> i64 {
        self.time().subsecond().millisecond()
    }

    fn microsecond(&self) -> i64 {
        self.time().subsecond().microsecond()
    }

    fn nanosecond(&self) -> i64 {
        self.time().subsecond().nanosecond()
    }

    fn picosecond(&self) -> i64 {
        self.time().subsecond().picosecond()
    }

    fn femtosecond(&self) -> i64 {
        self.time().subsecond().femtosecond()
    }
}

/// A human-readable time representation with support for representing leap seconds.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeOfDay {
    hour: u8,
    minute: u8,
    second: u8,
    subsecond: Subsecond,
}

impl TimeOfDay {
    /// Constructs a new `TimeOfDay` instance from the given hour, minute, and second components.
    ///
    /// # Errors
    ///
    /// - [TimeOfDayError::InvalidHour] if `hour` is not in the range `0..24`.
    /// - [TimeOfDayError::InvalidMinute] if `minute` is not in the range `0..60`.
    /// - [TimeOfDayError::InvalidSecond] if `second` is not in the range `0..61`.
    pub fn new(hour: u8, minute: u8, second: u8) -> Result<Self, TimeOfDayError> {
        if !(0..24).contains(&hour) {
            return Err(TimeOfDayError::InvalidHour(hour));
        }
        if !(0..60).contains(&minute) {
            return Err(TimeOfDayError::InvalidMinute(minute));
        }
        if !(0..61).contains(&second) {
            return Err(TimeOfDayError::InvalidSecond(second));
        }
        Ok(Self {
            hour,
            minute,
            second,
            subsecond: Subsecond::default(),
        })
    }

    /// Constructs a new `TimeOfDay` instance from an ISO 8601 time string.
    ///
    /// # Errors
    ///
    /// - [TimeOfDayError::InvalidIsoString] if the input string is not a valid ISO 8601 time
    ///   string.
    /// - [TimeOfDayError::InvalidHour] if the hour component is not in the range `0..24`.
    /// - [TimeOfDayError::InvalidMinute] if the minute component is not in the range `0..60`.
    /// - [TimeOfDayError::InvalidSecond] if the second component is not in the range `0..61`.
    pub fn from_iso(iso: &str) -> Result<Self, TimeOfDayError> {
        let caps = iso_regex()
            .captures(iso)
            .ok_or(TimeOfDayError::InvalidIsoString(iso.to_owned()))?;
        let hour: u8 = caps["hour"]
            .parse()
            .map_err(|_| TimeOfDayError::InvalidIsoString(iso.to_owned()))?;
        let minute: u8 = caps["minute"]
            .parse()
            .map_err(|_| TimeOfDayError::InvalidIsoString(iso.to_owned()))?;
        let second: u8 = caps["second"]
            .parse()
            .map_err(|_| TimeOfDayError::InvalidIsoString(iso.to_owned()))?;
        let mut time = TimeOfDay::new(hour, minute, second)?;
        if let Some(subsecond) = caps.name("subsecond") {
            let subsecond: f64 = subsecond
                .as_str()
                .parse()
                .map_err(|_| TimeOfDayError::InvalidIsoString(iso.to_owned()))?;
            time.with_subsecond(Subsecond(subsecond));
        }
        Ok(time)
    }

    /// Constructs a new `TimeOfDay` instance from the given hour, minute, and floating-point second
    /// components.
    ///
    /// # Errors
    ///
    /// - [TimeOfDayError::InvalidHour] if `hour` is not in the range `0..24`.
    /// - [TimeOfDayError::InvalidMinute] if `minute` is not in the range `0..60`.
    /// - [TimeOfDayError::InvalidSeconds] if `seconds` is not in the range `0.0..86401.0`.
    pub fn from_hms(hour: u8, minute: u8, seconds: f64) -> Result<Self, TimeOfDayError> {
        if !(0.0..86401.0).contains(&seconds) {
            return Err(TimeOfDayError::InvalidSeconds(InvalidSeconds(seconds)));
        }
        let second = seconds.trunc() as u8;
        let fraction = seconds.fract();
        let subsecond = Subsecond::new(fraction).unwrap();
        Ok(Self::new(hour, minute, second)?.with_subsecond(subsecond))
    }

    /// Constructs a new `TimeOfDay` instance from the given second of a day.
    ///
    /// # Errors
    ///
    /// - [TimeOfDayError::InvalidSecondOfDay] if `second_of_day` is not in the range `0..86401`.
    pub fn from_second_of_day(second_of_day: u64) -> Result<Self, TimeOfDayError> {
        if !(0..86401).contains(&second_of_day) {
            return Err(TimeOfDayError::InvalidSecondOfDay(second_of_day));
        }
        if second_of_day == SECONDS_PER_DAY as u64 {
            return Self::new(23, 59, 60);
        }
        let hour = (second_of_day / 3600) as u8;
        let minute = ((second_of_day % 3600) / 60) as u8;
        let second = (second_of_day % 60) as u8;
        Self::new(hour, minute, second)
    }

    /// Constructs a new `TimeOfDay` instance from an integral number of seconds since J2000.
    ///
    /// Note that this constructor is not leap-second aware.
    pub fn from_seconds_since_j2000(seconds: i64) -> Self {
        let mut second_of_day = (seconds + SECONDS_PER_HALF_DAY) % SECONDS_PER_DAY;
        if second_of_day.is_negative() {
            second_of_day += SECONDS_PER_DAY;
        }
        Self::from_second_of_day(
            second_of_day
                .to_u64()
                .unwrap_or_else(|| unreachable!("second of day should be positive")),
        )
        .unwrap_or_else(|_| unreachable!("second of day should be in range"))
    }

    /// Sets the [TimeOfDay]'s subsecond component.
    pub fn with_subsecond(&mut self, subsecond: Subsecond) -> Self {
        self.subsecond = subsecond;
        *self
    }

    pub fn hour(&self) -> u8 {
        self.hour
    }

    pub fn minute(&self) -> u8 {
        self.minute
    }

    pub fn second(&self) -> u8 {
        self.second
    }

    pub fn subsecond(&self) -> Subsecond {
        self.subsecond
    }

    /// Returns the number of integral seconds since the start of the day.
    pub fn second_of_day(&self) -> i64 {
        self.hour as i64 * SECONDS_PER_HOUR
            + self.minute as i64 * SECONDS_PER_MINUTE
            + self.second as i64
    }
}

impl Display for TimeOfDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let precision = f.precision().unwrap_or(3);
        write!(
            f,
            "{:02}:{:02}:{:02}{}",
            self.hour,
            self.minute,
            self.second,
            format!("{:.*}", precision, self.subsecond).trim_start_matches('0')
        )
    }
}

impl FromStr for TimeOfDay {
    type Err = TimeOfDayError;

    fn from_str(iso: &str) -> Result<Self, Self::Err> {
        Self::from_iso(iso)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(43201, TimeOfDay::new(12, 0, 1))]
    #[case(86399, TimeOfDay::new(23, 59, 59))]
    #[case(86400, TimeOfDay::new(23, 59, 60))]
    fn test_time_of_day_from_second_of_day(
        #[case] second_of_day: u64,
        #[case] expected: Result<TimeOfDay, TimeOfDayError>,
    ) {
        let actual = TimeOfDay::from_second_of_day(second_of_day);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_time_of_day_display() {
        let subsecond = Subsecond::new(0.123456789123456).unwrap();
        let time = TimeOfDay::new(12, 0, 0).unwrap().with_subsecond(subsecond);
        assert_eq!(format!("{}", time), "12:00:00.123");
        assert_eq!(format!("{:.15}", time), "12:00:00.123456789123456");
    }

    #[rstest]
    #[case(TimeOfDay::new(24, 0, 0), Err(TimeOfDayError::InvalidHour(24)))]
    #[case(TimeOfDay::new(0, 60, 0), Err(TimeOfDayError::InvalidMinute(60)))]
    #[case(TimeOfDay::new(0, 0, 61), Err(TimeOfDayError::InvalidSecond(61)))]
    #[case(
        TimeOfDay::from_second_of_day(86401),
        Err(TimeOfDayError::InvalidSecondOfDay(86401))
    )]
    #[case(TimeOfDay::from_hms(12, 0, -0.123), Err(TimeOfDayError::InvalidSeconds(InvalidSeconds(-0.123))))]
    fn test_time_of_day_error(
        #[case] actual: Result<TimeOfDay, TimeOfDayError>,
        #[case] expected: Result<TimeOfDay, TimeOfDayError>,
    ) {
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case("12:13:14", Ok(TimeOfDay::new(12, 13, 14).unwrap()))]
    #[case("12:13:14.123", Ok(TimeOfDay::new(12, 13, 14).unwrap().with_subsecond(Subsecond(0.123))))]
    #[case("2:13:14.123", Err(TimeOfDayError::InvalidIsoString("2:13:14.123".to_string())))]
    #[case("12:3:14.123", Err(TimeOfDayError::InvalidIsoString("12:3:14.123".to_string())))]
    #[case("12:13:4.123", Err(TimeOfDayError::InvalidIsoString("12:13:4.123".to_string())))]
    fn test_time_of_day_from_string(
        #[case] iso: &str,
        #[case] expected: Result<TimeOfDay, TimeOfDayError>,
    ) {
        let actual: Result<TimeOfDay, TimeOfDayError> = iso.parse();
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_invalid_seconds_ord() {
        let actual = InvalidSeconds(-f64::NAN).partial_cmp(&InvalidSeconds(f64::NAN));
        let expected = Some(Ordering::Less);
        assert_eq!(actual, expected);
        let actual = InvalidSeconds(-f64::NAN).cmp(&InvalidSeconds(f64::NAN));
        let expected = Ordering::Less;
        assert_eq!(actual, expected);
    }
}
