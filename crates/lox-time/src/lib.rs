/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! lox-time provides structs and functions for working with instants in astronomical time scales.
//!
//! The main struct is [Time], which represents an instant in time generic over a [TimeScale]
//! without leap seconds.
//!
//! [Utc] and [Date] are used strictly as an I/O formats, avoiding much of the complexity inherent
//! in working with leap seconds.

use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};

use calendar_dates::DateError;

use lox_utils::constants::f64::time;
use num::ToPrimitive;
use time_of_day::{CivilTime, TimeOfDay, TimeOfDayError};

use thiserror::Error;

use crate::base_time::BaseTime;
use crate::calendar_dates::{CalendarDate, Date};
use crate::deltas::TimeDelta;
use crate::julian_dates::{Epoch, JulianDate, Unit};
use crate::subsecond::Subsecond;
use crate::time_scales::TimeScale;
use crate::transformations::TransformTimeScale;

pub mod base_time;
pub mod calendar_dates;
pub mod constants;
pub mod deltas;
pub mod julian_dates;
pub mod prelude;
#[cfg(feature = "python")]
pub mod python;
pub mod subsecond;
pub mod time_of_day;
pub mod time_scales;
pub mod transformations;
pub mod utc;

#[derive(Clone, Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimeError {
    #[error(transparent)]
    DateError(#[from] DateError),
    #[error(transparent)]
    TimeError(#[from] TimeOfDayError),
    #[error("leap seconds do not exist in continuous time scales. Use Utc instead.")]
    LeapSecondOutsideUtc,
}

/// An instant in time in a given [TimeScale].
///
/// `Time` supports femtosecond precision, but be aware that many algorithms operating on `Time`s
/// are not accurate to this level of precision.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Time<T: TimeScale + Copy> {
    scale: T,
    timestamp: BaseTime,
}

impl<T: TimeScale + Copy> Time<T> {
    pub fn new(scale: T, year: i64, month: u8, day: u8) -> Result<Self, TimeError> {
        let date = Date::new(year, month, day)?;
        Ok(Self::from_date(scale, date))
    }

    pub fn from_date(scale: T, date: Date) -> Self {
        let seconds = ((date.days_since_j2000() - 0.5) * time::SECONDS_PER_DAY)
            .to_i64()
            .unwrap_or_else(|| unreachable!("should be representable as i64"));
        let timestamp = BaseTime::new(seconds, Subsecond::default());
        Self { scale, timestamp }
    }

    /// Instantiates a [Time] in the given scale from seconds since J2000 subdived into integral
    /// seconds and [Subsecond].
    pub fn from_seconds(scale: T, seconds: i64, subsecond: Subsecond) -> Self {
        Self {
            scale,
            timestamp: BaseTime::new(seconds, subsecond),
        }
    }

    pub fn from_delta(scale: T, delta: TimeDelta) -> Self {
        let timestamp = BaseTime::new(delta.seconds, delta.subsecond);
        Self { scale, timestamp }
    }

    pub fn to_delta(self) -> TimeDelta {
        TimeDelta {
            seconds: self.timestamp.seconds(),
            subsecond: self.timestamp.subsecond,
        }
    }

    /// Instantiates a [Time] in the given scale from a [BaseTime].
    pub const fn from_base_time(scale: T, timestamp: BaseTime) -> Self {
        Self { scale, timestamp }
    }

    /// Returns the epoch for the given [Epoch] in the given timescale.
    pub fn from_epoch(scale: T, epoch: Epoch) -> Self {
        let timestamp = BaseTime::from_epoch(epoch);
        Self { scale, timestamp }
    }

    pub fn with_time_of_day(mut self, time: TimeOfDay) -> Result<Self, TimeError> {
        if time.second() == 60 {
            return Err(TimeError::LeapSecondOutsideUtc);
        }
        let seconds = self.base_time().seconds() + time.second_of_day();
        let base = BaseTime::new(seconds, time.subsecond());
        self.timestamp = base;
        Ok(self)
    }

    pub fn with_hms(self, hour: u8, minute: u8, seconds: f64) -> Result<Self, TimeError> {
        let time = TimeOfDay::from_hms(hour, minute, seconds)?;
        self.with_time_of_day(time)
    }

    /// Returns the timescale
    pub fn scale(&self) -> T {
        self.scale
    }

    /// Returns a new [Time] with [scale] without changing the underlying timestamp.
    pub fn override_scale<S: TimeScale + Copy>(&self, scale: S) -> Time<S> {
        Time::from_base_time(scale, self.timestamp)
    }

    /// Returns, as an epoch in the given timescale, midday on the first day of the proleptic Julian
    /// calendar.
    pub fn jd0(scale: T) -> Self {
        Self::from_epoch(scale, Epoch::JulianDate)
    }

    /// Returns the epoch of the Modified Julian Date in the given timescale.
    pub fn mjd0(scale: T) -> Self {
        Self::from_epoch(scale, Epoch::ModifiedJulianDate)
    }

    /// Returns the J1950 epoch in the given timescale.
    pub fn j1950(scale: T) -> Self {
        Self::from_epoch(scale, Epoch::J1950)
    }

    /// Returns the J2000 epoch in the given timescale.
    pub fn j2000(scale: T) -> Self {
        Self::from_epoch(scale, Epoch::J2000)
    }

    /// The underlying base timestamp.
    pub fn base_time(&self) -> BaseTime {
        self.timestamp
    }

    /// The number of whole seconds since J2000.
    pub fn seconds(&self) -> i64 {
        self.timestamp.seconds
    }

    /// The number of femtoseconds from the last whole second.
    pub fn subsecond(&self) -> f64 {
        self.timestamp.subsecond.into()
    }

    /// Given a `Time` in [TimeScale] `S`, and a transformer from `S` to `T`, returns a new Time in
    /// [TimeScale] `T`.
    pub fn from_scale<S: TimeScale + Copy>(
        time: Time<S>,
        transformer: impl TransformTimeScale<S, T>,
    ) -> Self {
        transformer.transform(time)
    }

    /// Given a transformer from `T` to `S`, returns a new `Time` in [TimeScale] `S`.
    pub fn into_scale<S: TimeScale + Copy>(
        self,
        transformer: impl TransformTimeScale<T, S>,
    ) -> Time<S> {
        Time::from_scale(self, transformer)
    }

    pub fn from_julian_day_number(scale: T, day_number: i32, epoch: Epoch) -> Self {
        let timestamp = BaseTime::from_julian_day_number(day_number, epoch);
        Self { scale, timestamp }
    }
}

impl<T: TimeScale + Copy> JulianDate for Time<T> {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        self.timestamp.julian_date(epoch, unit)
    }

    fn two_part_julian_date(&self) -> (f64, f64) {
        self.timestamp.two_part_julian_date()
    }
}

impl<T: TimeScale + Copy> Display for Time<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let precision = f.precision().unwrap_or(3);
        write!(
            f,
            "{}T{:.*} {}",
            self.date(),
            precision,
            self.time(),
            self.scale.abbreviation()
        )
    }
}

impl<T: TimeScale + Copy> Add<TimeDelta> for Time<T> {
    type Output = Self;

    fn add(self, rhs: TimeDelta) -> Self::Output {
        Self::from_base_time(self.scale, self.timestamp + rhs)
    }
}

impl<T: TimeScale + Copy> Sub<TimeDelta> for Time<T> {
    type Output = Self;

    fn sub(self, rhs: TimeDelta) -> Self::Output {
        Self::from_base_time(self.scale, self.timestamp - rhs)
    }
}

impl<T: TimeScale + Copy> CivilTime for Time<T> {
    fn time(&self) -> TimeOfDay {
        TimeOfDay::from_seconds_since_j2000(self.timestamp.seconds)
            .with_subsecond(self.timestamp.subsecond)
    }
}

impl<T: TimeScale + Copy> CalendarDate for Time<T> {
    fn date(&self) -> Date {
        Date::from_seconds_since_j2000(self.timestamp.seconds)
    }
}

#[macro_export]
macro_rules! time {
    ($scale:ident, $year:literal, $month:literal, $day:literal) => {
        Time::new($scale, $year, $month, $day)
    };
    ($scale:ident, $year:literal, $month:literal, $day:literal, $hour:literal) => {
        match Time::new($scale, $year, $month, $day) {
            Ok(time) => time.with_hms($hour, 0, 0.0),
            Err(e) => Err(e),
        }
    };
    ($scale:ident, $year:literal, $month:literal, $day:literal, $hour:literal, $minute:literal) => {
        match Time::new($scale, $year, $month, $day) {
            Ok(time) => time.with_hms($hour, $minute, 0.0),
            Err(e) => Err(e),
        }
    };
    ($scale:ident, $year:literal, $month:literal, $day:literal, $hour:literal, $minute:literal, $second:literal) => {
        match Time::new($scale, $year, $month, $day) {
            Ok(time) => time.with_hms($hour, $minute, $second),
            Err(e) => Err(e),
        }
    };
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use mockall::predicate;

    use crate::constants::i64::{SECONDS_PER_DAY, SECONDS_PER_HALF_DAY};
    use lox_utils::constants::f64::time::DAYS_PER_JULIAN_CENTURY;
    use lox_utils::constants::i32::time::JD_J2000;

    use crate::time_scales::{Tai, Tdb, Tt};
    use crate::transformations::MockTransformTimeScale;
    use crate::Time;

    use super::*;

    #[test]
    fn test_time_new() {
        let time = Time::new(Tai, 2000, 1, 2).unwrap();
        assert_eq!(time.seconds(), SECONDS_PER_HALF_DAY);
        let time = time.with_hms(12, 0, 0.0).unwrap();
        assert_eq!(time.seconds(), SECONDS_PER_DAY);
    }

    #[test]
    fn test_time_from_seconds() {
        let scale = Tai;
        let seconds = 1234567890;
        let subsecond = Subsecond(0.9876543210);
        let expected = Time {
            scale,
            timestamp: BaseTime { seconds, subsecond },
        };
        let actual = Time::from_seconds(scale, seconds, subsecond);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_hms() {
        let tai = Time::new(Tai, 2000, 1, 1)
            .unwrap()
            .with_hms(12, 0, 0.0)
            .unwrap();
        assert_eq!(tai.seconds(), 0);
    }

    #[test]
    fn test_time_from_julian_day_number() {
        let expected: Time<Tai> = Time::default();
        let actual = Time::from_julian_day_number(Tai, JD_J2000, Epoch::JulianDate);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_display() {
        let time = Time::j2000(Tai);
        let expected = "2000-01-01T12:00:00.000 TAI".to_string();
        let actual = time.to_string();
        assert_eq!(expected, actual);
        let expected = "2000-01-01T12:00:00.000000000000000 TAI".to_string();
        let actual = format!("{:.15}", time);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_j2000() {
        let actual = Time::j2000(Tai);
        let expected = Time {
            scale: Tai,
            timestamp: BaseTime::default(),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_jd0() {
        let actual = Time::jd0(Tai);
        let expected = Time::from_base_time(
            Tai,
            BaseTime {
                seconds: -211813488000,
                subsecond: Subsecond::default(),
            },
        );
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_seconds() {
        let time = Time::from_seconds(Tai, 1234567890, Subsecond(0.9876543210));
        let expected = 1234567890;
        let actual = time.seconds();
        assert_eq!(
            expected, actual,
            "expected Time to have {} seconds, but got {}",
            expected, actual
        );
    }

    #[test]
    fn test_time_subsecond() {
        let time = Time::from_seconds(Tai, 1234567890, Subsecond(0.9876543210));
        let expected = 0.9876543210;
        let actual = time.subsecond();
        assert_eq!(
            expected, actual,
            "expected Time to have {} subsecond, but got {}",
            expected, actual
        );
    }

    #[test]
    fn test_time_days_since_j2000() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.days_since_j2000();
        let actual = Time::from_base_time(Tai, base_time).days_since_j2000();
        assert_float_eq!(
            actual,
            expected,
            rel <= 1e-15,
            "expected {} days since J2000, but got {}",
            expected,
            actual
        );
    }

    #[test]
    fn test_time_centuries_since_j2000() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.centuries_since_j2000();
        let actual = Time::from_base_time(Tai, base_time).centuries_since_j2000();
        assert_float_eq!(
            actual,
            expected,
            rel <= 1e-15,
            "expected {} centuries since J2000, but got {}",
            expected,
            actual
        );
    }

    #[test]
    fn test_time_civil_time_hour() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.hour();
        let actual = Time::from_base_time(Tai, base_time).hour();
        assert_eq!(
            expected, actual,
            "expected Time to have hour {}, but got {}",
            expected, actual
        );
    }

    #[test]
    fn test_time_civil_time_minute() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.minute();
        let actual = Time::from_base_time(Tai, base_time).minute();
        assert_eq!(
            expected, actual,
            "expected Time to have minute {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_civil_time_second() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.second();
        let actual = Time::from_base_time(Tai, base_time).second();
        assert_eq!(
            expected, actual,
            "expected Time to have second {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_civil_time_millisecond() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.millisecond();
        let actual = Time::from_base_time(Tai, base_time).millisecond();
        assert_eq!(
            expected, actual,
            "expected Time to have millisecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_civil_time_microsecond() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.microsecond();
        let actual = Time::from_base_time(Tai, base_time).microsecond();
        assert_eq!(
            expected, actual,
            "expected Time to have microsecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_civil_time_nanosecond() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.nanosecond();
        let actual = Time::from_base_time(Tai, base_time).nanosecond();
        assert_eq!(
            expected, actual,
            "expected Time to have nanosecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_civil_time_picosecond() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.picosecond();
        let actual = Time::from_base_time(Tai, base_time).picosecond();
        assert_eq!(
            expected, actual,
            "expected Time to have picosecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_civil_time_femtosecond() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.femtosecond();
        let actual = Time::from_base_time(Tai, base_time).femtosecond();
        assert_eq!(
            expected, actual,
            "expected Time to have femtosecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_from_scale() {
        let time = Time::j2000(Tai);
        let mut transformer = MockTransformTimeScale::<Tai, Tt>::new();
        let expected = Time::j2000(Tt);

        transformer
            .expect_transform()
            .with(predicate::eq(time))
            .return_const(expected);

        let actual = Time::from_scale(time, transformer);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_into_scale() {
        let time = Time::j2000(Tai);
        let mut transformer = MockTransformTimeScale::<Tai, Tt>::new();
        let expected = Time::j2000(Tt);

        transformer
            .expect_transform()
            .with(predicate::eq(time))
            .return_const(expected);

        let actual = time.into_scale(transformer);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_julian_date() {
        let time = Time::jd0(Tdb);
        assert_eq!(time.julian_date(Epoch::JulianDate, Unit::Days), 0.0);
        assert_eq!(time.seconds_since_julian_epoch(), 0.0);
        assert_eq!(time.days_since_julian_epoch(), 0.0);
        assert_eq!(time.centuries_since_julian_epoch(), 0.0);
    }

    #[test]
    fn test_modified_julian_date() {
        let time = Time::mjd0(Tdb);
        assert_eq!(time.julian_date(Epoch::ModifiedJulianDate, Unit::Days), 0.0);
        assert_eq!(time.seconds_since_modified_julian_epoch(), 0.0);
        assert_eq!(time.days_since_modified_julian_epoch(), 0.0);
        assert_eq!(time.centuries_since_modified_julian_epoch(), 0.0);
    }

    #[test]
    fn test_j1950() {
        let time = Time::j1950(Tdb);
        assert_eq!(time.julian_date(Epoch::J1950, Unit::Days), 0.0);
        assert_eq!(time.seconds_since_j1950(), 0.0);
        assert_eq!(time.days_since_j1950(), 0.0);
        assert_eq!(time.centuries_since_j1950(), 0.0);
    }

    #[test]
    fn test_j2000() {
        let time = Time::j2000(Tdb);
        assert_eq!(time.julian_date(Epoch::J2000, Unit::Days), 0.0);
        assert_eq!(time.seconds_since_j2000(), 0.0);
        assert_eq!(time.days_since_j2000(), 0.0);
        assert_eq!(time.centuries_since_j2000(), 0.0);
    }

    #[test]
    fn test_j2100() {
        let time = Time::new(Tai, 2100, 1, 1)
            .unwrap()
            .with_hms(12, 0, 0.0)
            .unwrap();
        assert_eq!(
            time.julian_date(Epoch::J2000, Unit::Days),
            DAYS_PER_JULIAN_CENTURY
        );
        assert_eq!(time.seconds_since_j2000(), 3155760000.0);
        assert_eq!(time.days_since_j2000(), DAYS_PER_JULIAN_CENTURY);
        assert_eq!(time.centuries_since_j2000(), 1.0);
    }

    #[test]
    fn test_two_part_julian_date() {
        let time = Time::new(Tdb, 2100, 1, 2)
            .unwrap()
            .with_hms(0, 0, 0.0)
            .unwrap();
        let (jd1, jd2) = time.two_part_julian_date();
        assert_eq!(jd1, 2451545.0 + DAYS_PER_JULIAN_CENTURY);
        assert_eq!(jd2, 0.5);
    }

    #[test]
    fn test_time_add_time_delta() {
        let time = Time::j2000(Tai);
        let delta = TimeDelta::from_decimal_seconds(1.5).unwrap();
        let expected = Time {
            scale: Tai,
            timestamp: time.timestamp + delta,
        };
        let actual = Time::j2000(Tai) + delta;
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_sub_time_delta() {
        let time = Time::j2000(Tai);
        let delta = TimeDelta::from_decimal_seconds(1.5).unwrap();
        let expected = Time {
            scale: Tai,
            timestamp: time.timestamp - delta,
        };
        let actual = Time::j2000(Tai) - delta;
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_calendar_date() {
        let base_time = BaseTime::default();
        let expected = base_time.date();
        let tai = Time::from_base_time(Tai, base_time);
        let actual = tai.date();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_macro() {
        let time = time!(Tai, 2000, 1, 1).unwrap();
        assert_eq!(time.seconds(), -SECONDS_PER_HALF_DAY);
        let time = time!(Tai, 2000, 1, 1, 12).unwrap();
        assert_eq!(time.seconds(), 0);
        let time = time!(Tai, 2000, 1, 1, 12, 0).unwrap();
        assert_eq!(time.seconds(), 0);
        let time = time!(Tai, 2000, 1, 1, 12, 0, 0.0).unwrap();
        assert_eq!(time.seconds(), 0);
        let time = time!(Tai, 2000, 1, 1, 12, 0, 0.123).unwrap();
        assert_eq!(time.seconds(), 0);
        assert_eq!(time.subsecond(), 0.123);
    }
}
