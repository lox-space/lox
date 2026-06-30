// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2013-2021 NumFOCUS Foundation
//
// SPDX-License-Identifier: MPL-2.0 AND LicenseRef-ERFA

/*!
    Module `time_of_day` exposes the concrete representation of a time of day with leap second
    support, [TimeOfDay].

    The [CivilTime] trait supports arbitrary time representations to express themselves as a
    human-readable time of day.
*/

use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;

use core::cmp::Ordering;
use core::fmt::Display;
use core::str::FromStr;

use crate::units::Angle;
use crate::units::Sign;
use nom::{Parser, combinator::all_consuming};
use thiserror::Error;

use super::iso;
use super::subsecond::Subsecond;
use crate::i64::consts::{
    SECONDS_PER_DAY, SECONDS_PER_HALF_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE,
};

/// Error type returned when attempting to construct a [TimeOfDay] with a greater number of
/// floating-point seconds than are in a day.
#[derive(Debug, Copy, Clone, Error)]
#[error("seconds must be in the range [0.0..86401.0) but was {0}")]
pub struct InvalidSeconds(f64);

impl PartialEq for InvalidSeconds {
    fn eq(&self, other: &Self) -> bool {
        self.0.total_cmp(&other.0) == Ordering::Equal
    }
}

impl Eq for InvalidSeconds {}

/// Error type returned when attempting to construct a [TimeOfDay] from invalid components.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum TimeOfDayError {
    /// Hour is outside the valid range `[0, 24)`.
    #[error("hour must be in the range [0..24) but was {0}")]
    InvalidHour(u8),
    /// Minute is outside the valid range `[0, 60)`.
    #[error("minute must be in the range [0..60) but was {0}")]
    InvalidMinute(u8),
    /// Second is outside the valid range `[0, 61)`.
    #[error("second must be in the range [0..61) but was {0}")]
    InvalidSecond(u8),
    /// Second of day is outside the valid range `[0, 86401)`.
    #[error("second must be in the range [0..86401) but was {0}")]
    InvalidSecondOfDay(u64),
    /// Floating-point seconds value is out of range.
    #[error(transparent)]
    InvalidSeconds(#[from] InvalidSeconds),
    /// A leap second was specified at a time other than the end of the day.
    #[error("leap seconds are only valid at the end of the day")]
    InvalidLeapSecond,
    /// The input string is not a valid ISO 8601 time.
    #[error("invalid ISO string `{0}`")]
    InvalidIsoString(String),
}

/// `CivilTime` is the trait by which high-precision time representations expose human-readable time
/// components.
pub trait CivilTime {
    /// Returns the time-of-day component.
    fn time(&self) -> TimeOfDay;

    /// Returns the hour (0–23).
    fn hour(&self) -> u8 {
        self.time().hour()
    }

    /// Returns the minute (0–59).
    fn minute(&self) -> u8 {
        self.time().minute()
    }

    /// Returns the second (0–60, where 60 represents a leap second).
    fn second(&self) -> u8 {
        self.time().second()
    }

    /// Returns the second including the subsecond fraction as an `f64`.
    fn as_seconds_f64(&self) -> f64 {
        self.time().subsecond().as_seconds_f64() + self.time().second() as f64
    }

    /// Returns the millisecond component (0–999).
    fn millisecond(&self) -> u32 {
        self.time().subsecond().milliseconds()
    }

    /// Returns the microsecond component (0–999).
    fn microsecond(&self) -> u32 {
        self.time().subsecond().microseconds()
    }

    /// Returns the nanosecond component (0–999).
    fn nanosecond(&self) -> u32 {
        self.time().subsecond().nanoseconds()
    }

    /// Returns the picosecond component (0–999).
    fn picosecond(&self) -> u32 {
        self.time().subsecond().picoseconds()
    }

    /// Returns the femtosecond component (0–999).
    fn femtosecond(&self) -> u32 {
        self.time().subsecond().femtoseconds()
    }
}

/// A human-readable time representation with support for representing leap seconds.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeOfDay {
    hour: u8,
    minute: u8,
    second: u8,
    subsecond: Subsecond,
}

impl TimeOfDay {
    /// Midnight (00:00:00.000).
    pub const MIDNIGHT: Self = TimeOfDay {
        hour: 0,
        minute: 0,
        second: 0,
        subsecond: Subsecond::ZERO,
    };

    /// Noon (12:00:00.000).
    pub const NOON: Self = TimeOfDay {
        hour: 12,
        minute: 0,
        second: 0,
        subsecond: Subsecond::ZERO,
    };
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

    /// Creates a [TimeOfDay] without validation. Use when the caller has
    /// already verified that the components are in range (and, for second=60,
    /// that this is a leap-second instant).
    pub const fn new_unchecked(hour: u8, minute: u8, second: u8) -> Self {
        Self {
            hour,
            minute,
            second,
            subsecond: Subsecond::ZERO,
        }
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
    pub fn from_iso(input: &str) -> Result<Self, TimeOfDayError> {
        let (_, (hour, minute, second, fraction)) = all_consuming(iso::time)
            .parse(input)
            .map_err(|_| TimeOfDayError::InvalidIsoString(input.to_owned()))?;
        let mut time = TimeOfDay::new(hour, minute, second)?;
        if let Some(subsecond_str) = fraction {
            let subsecond: Subsecond = subsecond_str
                .parse()
                .map_err(|_| TimeOfDayError::InvalidIsoString(input.to_owned()))?;
            time.with_subsecond(subsecond);
        }
        Ok(time)
    }

    /// Constructs a `TimeOfDay` from an hour only (minute and second default to zero).
    pub fn from_hour(hour: u8) -> Result<Self, TimeOfDayError> {
        Self::new(hour, 0, 0)
    }

    /// Constructs a `TimeOfDay` from hour and minute (second defaults to zero).
    pub fn from_hour_and_minute(hour: u8, minute: u8) -> Result<Self, TimeOfDayError> {
        Self::new(hour, minute, 0)
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
        let second = crate::math::float::trunc(seconds) as u8;
        let fraction = crate::math::float::fract(seconds);
        let subsecond = Subsecond::from_f64(fraction)
            .ok_or(TimeOfDayError::InvalidSeconds(InvalidSeconds(seconds)))?;
        Ok(Self::new(hour, minute, second)?.with_subsecond(subsecond))
    }

    /// Constructs a `TimeOfDay` from a fractional day in `[0.0, 1.0)`.
    ///
    /// Unlike ERFA `d2tf`, this function does not accept negative inputs
    /// or fractions ≥ 1.0 — they don't represent a valid `TimeOfDay`.
    ///
    /// # Errors
    ///
    /// Returns [`TimeOfDayError::InvalidSeconds`] for non-finite inputs and
    /// inputs outside `[0.0, 1.0)`.
    ///
    /// # References
    ///
    /// - ERFA [`d2tf`](https://github.com/liberfa/erfa/blob/master/src/d2tf.c)
    pub fn from_day_fraction(days: f64) -> Result<Self, TimeOfDayError> {
        if !days.is_finite() || !(0.0..1.0).contains(&days) {
            return Err(TimeOfDayError::InvalidSeconds(InvalidSeconds(
                days * SECONDS_PER_DAY as f64,
            )));
        }
        let total_seconds = days * SECONDS_PER_DAY as f64;
        let hour = (total_seconds / SECONDS_PER_HOUR as f64) as u8;
        let rem = total_seconds - (hour as f64) * SECONDS_PER_HOUR as f64;
        let minute = (rem / SECONDS_PER_MINUTE as f64) as u8;
        let seconds = rem - (minute as f64) * SECONDS_PER_MINUTE as f64;
        Self::from_hms(hour, minute, seconds)
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
        Self::from_second_of_day(second_of_day as u64)
            .unwrap_or_else(|_| unreachable!("second of day should be in range"))
    }

    /// Sets the [TimeOfDay]'s subsecond component.
    pub fn with_subsecond(&mut self, subsecond: Subsecond) -> Self {
        self.subsecond = subsecond;
        *self
    }

    /// Returns the hour (0–23).
    pub fn hour(&self) -> u8 {
        self.hour
    }

    /// Returns the minute (0–59).
    pub fn minute(&self) -> u8 {
        self.minute
    }

    /// Returns the second (0–60, where 60 represents a leap second).
    pub fn second(&self) -> u8 {
        self.second
    }

    /// Returns the subsecond component.
    pub fn subsecond(&self) -> Subsecond {
        self.subsecond
    }

    /// Returns the second including the subsecond fraction as an `f64`.
    pub fn seconds_f64(&self) -> f64 {
        self.subsecond.as_seconds_f64() + self.second as f64
    }

    /// Returns the time of day as a fraction of a day in `[0.0, 1.0)`.
    ///
    /// Inverse of [`TimeOfDay::from_day_fraction`].
    ///
    /// # References
    ///
    /// - ERFA [`tf2d`](https://github.com/liberfa/erfa/blob/master/src/tf2d.c)
    pub fn to_day_fraction(&self) -> f64 {
        let total_seconds = self.hour as f64 * SECONDS_PER_HOUR as f64
            + self.minute as f64 * SECONDS_PER_MINUTE as f64
            + self.seconds_f64();
        total_seconds / SECONDS_PER_DAY as f64
    }

    /// Returns the number of integral seconds since the start of the day.
    pub fn second_of_day(&self) -> i64 {
        self.hour as i64 * SECONDS_PER_HOUR
            + self.minute as i64 * SECONDS_PER_MINUTE
            + self.second as i64
    }

    /// Converts the time of day to an [`Angle`] (hour angle representation).
    pub fn to_angle(&self) -> Angle {
        // `TimeOfDay` hours are always non-negative.
        Angle::from_hms(
            Sign::Positive,
            self.hour as u32,
            self.minute,
            self.seconds_f64(),
        )
    }
}

impl Display for TimeOfDay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
    use alloc::string::ToString;
    use lox_approx::assert_approx_eq;
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
        let subsecond: Subsecond = "123456789123456".parse().unwrap();
        let time = TimeOfDay::new(12, 0, 0).unwrap().with_subsecond(subsecond);
        assert_eq!(format!("{time}"), "12:00:00.123");
        assert_eq!(format!("{time:.15}"), "12:00:00.123456789123456");
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
    #[case("12:13:14.123", Ok(TimeOfDay::new(12, 13, 14).unwrap().with_subsecond("123".parse().unwrap())))]
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
    fn test_invalid_seconds_eq() {
        let a = InvalidSeconds(-f64::NAN);
        let b = InvalidSeconds(f64::NAN);
        // NaN values with different signs should not be equal
        assert_ne!(a, b);
        // Same NaN values should be equal
        let c = InvalidSeconds(f64::NAN);
        let d = InvalidSeconds(f64::NAN);
        assert_eq!(c, d);
    }

    #[test]
    fn test_time_of_day_from_day_fraction_erfa_d2tf() {
        // ERFA t_erfa_c.c::t_d2tf: |d2tf(4, -0.987654321)| = 23h 42m 13.3333s
        let tod = TimeOfDay::from_day_fraction(0.987654321).unwrap();
        assert_eq!(tod.hour(), 23);
        assert_eq!(tod.minute(), 42);
        assert_approx_eq!(tod.seconds_f64(), 13.3333, atol <= 1e-4);
    }

    #[test]
    fn test_time_of_day_from_day_fraction_zero() {
        let tod = TimeOfDay::from_day_fraction(0.0).unwrap();
        assert_eq!(tod.hour(), 0);
        assert_eq!(tod.minute(), 0);
        assert_eq!(tod.second(), 0);
    }

    #[test]
    fn test_time_of_day_from_day_fraction_negative_errors() {
        // Negative day fractions don't represent a valid TimeOfDay.
        let result = TimeOfDay::from_day_fraction(-0.5);
        assert!(matches!(result, Err(TimeOfDayError::InvalidSeconds(_))));
    }

    #[test]
    fn test_time_of_day_from_day_fraction_one_or_above_errors() {
        let result = TimeOfDay::from_day_fraction(1.5);
        assert!(matches!(result, Err(TimeOfDayError::InvalidSeconds(_))));
    }

    #[test]
    fn test_time_of_day_to_day_fraction_erfa_tf2d() {
        // ERFA t_erfa_c.c::t_tf2d: tf2d(' ', 23, 55, 10.9) = 0.9966539351851851852
        let d = TimeOfDay::from_hms(23, 55, 10.9).unwrap().to_day_fraction();
        assert_approx_eq!(d, 0.996_653_935_185_185_2, atol <= 1e-12);
    }

    #[test]
    fn test_time_of_day_to_day_fraction_roundtrip() {
        let original = TimeOfDay::from_hms(12, 34, 56.789).unwrap();
        let d = original.to_day_fraction();
        let recovered = TimeOfDay::from_day_fraction(d).unwrap();
        assert_eq!(recovered.hour(), original.hour());
        assert_eq!(recovered.minute(), original.minute());
        assert_approx_eq!(
            recovered.seconds_f64(),
            original.seconds_f64(),
            atol <= 1e-10
        );
    }
}
