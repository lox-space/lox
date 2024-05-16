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

use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};

use num::ToPrimitive;
use thiserror::Error;

use calendar_dates::DateError;
use constants::julian_dates::{
    SECONDS_BETWEEN_J1950_AND_J2000, SECONDS_BETWEEN_JD_AND_J2000, SECONDS_BETWEEN_MJD_AND_J2000,
};
use lox_utils::constants::f64::time;
use lox_utils::types::units::Days;
use time_of_day::{CivilTime, TimeOfDay, TimeOfDayError};

use crate::calendar_dates::{CalendarDate, Date};
use crate::deltas::{TimeDelta, ToDelta};
use crate::julian_dates::{Epoch, JulianDate, Unit};
use crate::subsecond::Subsecond;
use crate::time_scales::TimeScale;
use crate::transformations::TransformTimeScale;

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
pub mod ut1;
pub mod utc;

#[derive(Clone, Debug, Error)]
#[error(
    "Julian date must be between {} and {} seconds since J2000 but was {0}",
    i64::MIN,
    i64::MAX
)]
pub struct JulianDateOutOfRange(f64);

impl PartialOrd for JulianDateOutOfRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JulianDateOutOfRange {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialEq for JulianDateOutOfRange {
    fn eq(&self, other: &Self) -> bool {
        self.0.total_cmp(&other.0) == Ordering::Equal
    }
}

impl Eq for JulianDateOutOfRange {}

#[derive(Clone, Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimeError {
    #[error(transparent)]
    DateError(#[from] DateError),
    #[error(transparent)]
    TimeError(#[from] TimeOfDayError),
    #[error("leap seconds do not exist in continuous time scales; use `Utc` instead")]
    LeapSecondOutsideUtc,
    #[error(transparent)]
    JulianDateOutOfRange(#[from] JulianDateOutOfRange),
}

/// An instant in time in a given [TimeScale].
///
/// `Time` supports femtosecond precision, but be aware that many algorithms operating on `Time`s
/// are not accurate to this level of precision.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Time<T: TimeScale + Copy> {
    scale: T,
    seconds: i64,
    subsecond: Subsecond,
}

impl<T: TimeScale + Copy> Time<T> {
    /// Instantiates a [Time] in the given scale from seconds since J2000 subdivided into integral
    /// seconds and [Subsecond].
    pub fn new(scale: T, seconds: i64, subsecond: Subsecond) -> Self {
        Self {
            scale,
            seconds,
            subsecond,
        }
    }

    /// Instantiates a [Time] in the given `scale` from a [Date] and a [TimeOfDay].
    pub fn from_date_and_time(scale: T, date: Date, time: TimeOfDay) -> Result<Self, TimeError> {
        let mut seconds = (date.days_since_j2000() * time::SECONDS_PER_DAY)
            .to_i64()
            .unwrap_or_else(|| {
                unreachable!(
                    "seconds since J2000 for date {} are not representable as i64: {}",
                    date,
                    date.days_since_j2000()
                )
            });
        if time.second() == 60 {
            return Err(TimeError::LeapSecondOutsideUtc);
        }
        seconds += time.second_of_day();
        Ok(Self::new(scale, seconds, time.subsecond()))
    }

    /// Instantiates a [Time] in the given `scale` from an offset from the J2000 epoch given as a [TimeDelta].
    pub fn from_delta(scale: T, delta: TimeDelta) -> Self {
        Self {
            scale,
            seconds: delta.seconds,
            subsecond: delta.subsecond,
        }
    }

    /// Returns the offset from the J2000 epoch as a [TimeDelta].
    pub fn to_delta(self) -> TimeDelta {
        TimeDelta {
            seconds: self.seconds,
            subsecond: self.subsecond,
        }
    }

    /// Returns the epoch for the given [Epoch] in the given timescale.
    pub fn from_epoch(scale: T, epoch: Epoch) -> Self {
        match epoch {
            Epoch::JulianDate => Self {
                scale,
                seconds: -SECONDS_BETWEEN_JD_AND_J2000,
                subsecond: Subsecond::default(),
            },
            Epoch::ModifiedJulianDate => Self {
                scale,
                seconds: -SECONDS_BETWEEN_MJD_AND_J2000,
                subsecond: Subsecond::default(),
            },
            Epoch::J1950 => Self {
                scale,
                seconds: -SECONDS_BETWEEN_J1950_AND_J2000,
                subsecond: Subsecond::default(),
            },
            Epoch::J2000 => Self {
                scale,
                seconds: 0,
                subsecond: Subsecond::default(),
            },
        }
    }

    /// Instantiates a [Time] in the given scale from a `julian_date` with the given `epoch`.
    pub fn from_julian_date(scale: T, julian_date: Days, epoch: Epoch) -> Result<Self, TimeError> {
        let seconds = julian_date * time::SECONDS_PER_DAY;
        if !(i64::MIN as f64..=i64::MAX as f64).contains(&seconds) {
            return Err(TimeError::JulianDateOutOfRange(JulianDateOutOfRange(
                seconds,
            )));
        }
        let subsecond = Subsecond::new(seconds.fract()).unwrap();
        let seconds = seconds.to_i64().unwrap_or_else(|| {
            unreachable!(
                "seconds since J2000 for Julian date {} are not representable as i64: {}",
                julian_date, seconds
            )
        });
        let seconds = match epoch {
            Epoch::JulianDate => seconds - SECONDS_BETWEEN_JD_AND_J2000,
            Epoch::ModifiedJulianDate => seconds - SECONDS_BETWEEN_MJD_AND_J2000,
            Epoch::J1950 => seconds - SECONDS_BETWEEN_J1950_AND_J2000,
            Epoch::J2000 => seconds,
        };
        Ok(Self::new(scale, seconds, subsecond))
    }

    /// Returns a [TimeBuilder] for constructing a new [Time] in the given `scale`.
    pub fn builder_with_scale(scale: T) -> TimeBuilder<T> {
        TimeBuilder::new(scale)
    }

    /// Returns the timescale
    pub fn scale(&self) -> T {
        self.scale
    }

    /// Returns a new [Time] with `scale` without changing the underlying timestamp.
    pub fn with_scale<S: TimeScale + Copy>(&self, scale: S) -> Time<S> {
        Time::new(scale, self.seconds, self.subsecond)
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

    /// The number of whole seconds since J2000.
    pub fn seconds(&self) -> i64 {
        self.seconds
    }

    /// The number of femtoseconds from the last whole second.
    pub fn subsecond(&self) -> f64 {
        self.subsecond.into()
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
}

impl<T: TimeScale + Copy> ToDelta for Time<T> {
    fn to_delta(&self) -> TimeDelta {
        TimeDelta {
            seconds: self.seconds,
            subsecond: self.subsecond,
        }
    }
}

impl<T: TimeScale + Copy> JulianDate for Time<T> {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        let mut decimal_seconds = (match epoch {
            Epoch::JulianDate => self.seconds + SECONDS_BETWEEN_JD_AND_J2000,
            Epoch::ModifiedJulianDate => self.seconds + SECONDS_BETWEEN_MJD_AND_J2000,
            Epoch::J1950 => self.seconds + SECONDS_BETWEEN_J1950_AND_J2000,
            Epoch::J2000 => self.seconds,
        })
        .to_f64()
        .unwrap();
        decimal_seconds += self.subsecond.0;
        match unit {
            Unit::Seconds => decimal_seconds,
            Unit::Days => decimal_seconds / time::SECONDS_PER_DAY,
            Unit::Centuries => decimal_seconds / time::SECONDS_PER_JULIAN_CENTURY,
        }
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

    /// The implementation of [Add] for [Time] follows the default Rust rules for integer overflow, which
    /// should be sufficient for all practical purposes.
    fn add(self, rhs: TimeDelta) -> Self::Output {
        if rhs.is_negative() {
            return self - (-rhs);
        }

        let subsec_and_carry = self.subsecond.0 + rhs.subsecond.0;
        let seconds = subsec_and_carry.trunc().to_i64().unwrap() + self.seconds + rhs.seconds;
        Self::new(self.scale, seconds, Subsecond(subsec_and_carry.fract()))
    }
}

impl<T: TimeScale + Copy> Sub<TimeDelta> for Time<T> {
    type Output = Self;

    /// The implementation of [Sub] for [Time] follows the default Rust rules for integer overflow, which
    /// should be sufficient for all practical purposes.
    fn sub(self, rhs: TimeDelta) -> Self::Output {
        if rhs.is_negative() {
            return self + (-rhs);
        }

        let mut subsec = self.subsecond.0 - rhs.subsecond.0;
        let mut seconds = self.seconds - rhs.seconds;
        if subsec.is_sign_negative() {
            seconds -= 1;
            subsec += 1.0;
        }
        Self::new(self.scale, seconds, Subsecond(subsec))
    }
}

impl<T: TimeScale + Copy> Sub<Time<T>> for Time<T> {
    type Output = TimeDelta;

    fn sub(self, rhs: Time<T>) -> Self::Output {
        let mut subsec = self.subsecond.0 - rhs.subsecond.0;
        let mut seconds = self.seconds - rhs.seconds;
        if subsec.is_sign_negative() {
            seconds -= 1;
            subsec += 1.0;
        }
        TimeDelta {
            seconds,
            subsecond: Subsecond(subsec),
        }
    }
}

impl<T: TimeScale + Copy> CivilTime for Time<T> {
    fn time(&self) -> TimeOfDay {
        TimeOfDay::from_seconds_since_j2000(self.seconds).with_subsecond(self.subsecond)
    }
}

impl<T: TimeScale + Copy> CalendarDate for Time<T> {
    fn date(&self) -> Date {
        Date::from_seconds_since_j2000(self.seconds)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeBuilder<T: TimeScale> {
    scale: T,
    date: Result<Date, DateError>,
    time: Result<TimeOfDay, TimeOfDayError>,
}

impl<T: TimeScale + Copy> TimeBuilder<T> {
    pub fn new(scale: T) -> Self {
        Self {
            scale,
            date: Ok(Date::default()),
            time: Ok(TimeOfDay::default()),
        }
    }

    /// Sets the `year`, `month`, and `day` of the [Time] under construction.
    pub fn with_ymd(self, year: i64, month: u8, day: u8) -> Self {
        Self {
            date: Date::new(year, month, day),
            ..self
        }
    }

    /// Sets the `hour`, `minute`, and decimal `seconds` of the [Time] under construction.
    pub fn with_hms(self, hour: u8, minute: u8, seconds: f64) -> Self {
        Self {
            time: TimeOfDay::from_hms(hour, minute, seconds),
            ..self
        }
    }

    /// Builds the [Time] instance.
    pub fn build(self) -> Result<Time<T>, TimeError> {
        let date = self.date?;
        let time = self.time?;
        Time::from_date_and_time(self.scale, date, time)
    }
}

#[macro_export]
macro_rules! time {
    ($scale:expr, $year:literal, $month:literal, $day:literal) => {
        Time::builder_with_scale($scale)
            .with_ymd($year, $month, $day)
            .build()
    };
    ($scale:expr, $year:literal, $month:literal, $day:literal, $hour:literal) => {
        Time::builder_with_scale($scale)
            .with_ymd($year, $month, $day)
            .with_hms($hour, 0, 0.0)
            .build()
    };
    ($scale:expr, $year:literal, $month:literal, $day:literal, $hour:literal, $minute:literal) => {
        Time::builder_with_scale($scale)
            .with_ymd($year, $month, $day)
            .with_hms($hour, $minute, 0.0)
            .build()
    };
    ($scale:expr, $year:literal, $month:literal, $day:literal, $hour:literal, $minute:literal, $second:literal) => {
        Time::builder_with_scale($scale)
            .with_ymd($year, $month, $day)
            .with_hms($hour, $minute, $second)
            .build()
    };
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use mockall::predicate;
    use rstest::rstest;

    use lox_utils::constants::f64::time::DAYS_PER_JULIAN_CENTURY;

    use crate::constants::i64::{SECONDS_PER_DAY, SECONDS_PER_HALF_DAY};
    use crate::time_scales::{Tai, Tdb, Tt};
    use crate::transformations::MockTransformTimeScale;
    use crate::Time;

    use super::*;

    use self::constants::i64::{SECONDS_PER_HOUR, SECONDS_PER_JULIAN_CENTURY, SECONDS_PER_MINUTE};

    #[test]
    fn test_time_builder() {
        let time = Time::builder_with_scale(Tai)
            .with_ymd(2000, 1, 1)
            .build()
            .unwrap();
        assert_eq!(time.seconds(), -SECONDS_PER_HALF_DAY);
        let time = Time::builder_with_scale(Tai)
            .with_ymd(2000, 1, 1)
            .with_hms(12, 0, 0.0)
            .build()
            .unwrap();
        assert_eq!(time.seconds(), 0);
    }

    #[test]
    fn test_time_from_seconds() {
        let scale = Tai;
        let seconds = 1234567890;
        let subsecond = Subsecond(0.9876543210);
        let expected = Time {
            scale,
            seconds,
            subsecond,
        };
        let actual = Time::new(scale, seconds, subsecond);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(Epoch::JulianDate, -SECONDS_BETWEEN_JD_AND_J2000)]
    #[case(Epoch::ModifiedJulianDate, -SECONDS_BETWEEN_MJD_AND_J2000)]
    #[case(Epoch::J1950, -SECONDS_BETWEEN_J1950_AND_J2000)]
    #[case(Epoch::J2000, 0)]
    fn test_time_from_julian_date(#[case] epoch: Epoch, #[case] seconds: i64) {
        let time = Time::from_julian_date(Tai, 0.0, epoch).unwrap();
        assert_eq!(time.seconds(), seconds);
    }

    #[test]
    fn test_time_from_julian_date_subsecond() {
        let time = Time::from_julian_date(Tai, 0.3 / time::SECONDS_PER_DAY, Epoch::J2000).unwrap();
        assert_float_eq!(time.subsecond(), 0.3, abs <= 1e-15);
    }

    #[rstest]
    #[case(f64::INFINITY)]
    #[case(-f64::INFINITY)]
    #[case(f64::NAN)]
    #[case(-f64::NAN)]
    #[case(i64::MAX as f64 / time::SECONDS_PER_DAY + 1.0)]
    #[case(i64::MIN as f64 / time::SECONDS_PER_DAY - 1.0)]
    fn test_time_from_julian_date_invalid(#[case] julian_date: f64) {
        let expected = Err(TimeError::JulianDateOutOfRange(JulianDateOutOfRange(
            julian_date * time::SECONDS_PER_DAY,
        )));
        let actual = Time::from_julian_date(Tai, julian_date, Epoch::J2000);
        assert_eq!(actual, expected);
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
            ..Default::default()
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_jd0() {
        let actual = Time::jd0(Tai);
        let expected = Time {
            scale: Tai,
            seconds: -211813488000,
            subsecond: Subsecond::default(),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_seconds() {
        let time = Time::new(Tai, 1234567890, Subsecond(0.9876543210));
        let expected = 1234567890;
        let actual = time.seconds();
        assert_eq!(
            expected, actual,
            "expected Time to have {} seconds, but got {}",
            expected, actual
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
        let time = time!(Tdb, 2100, 1, 1, 12).unwrap();
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
        let time = time!(Tdb, 2100, 1, 2).unwrap();
        let (jd1, jd2) = time.two_part_julian_date();
        assert_eq!(jd1, 2451545.0 + DAYS_PER_JULIAN_CENTURY);
        assert_eq!(jd2, 0.5);
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

    #[test]
    fn test_time_subsecond() {
        let time = Time {
            scale: Tai,
            seconds: 0,
            subsecond: Subsecond(0.123),
        };
        assert_eq!(time.subsecond(), 0.123);
    }

    #[rstest]
    #[case::zero_delta(Time::default(), Time::default(), TimeDelta::default())]
    #[case::positive_delta(Time::default(), Time {scale: Tai, seconds: 1, subsecond: Subsecond::default() }, TimeDelta { seconds: -1, subsecond: Subsecond::default() })]
    #[case::negative_delta(Time::default(), Time {scale: Tai, seconds: -1, subsecond: Subsecond::default() }, TimeDelta { seconds: 1, subsecond: Subsecond::default() })]
    fn test_time_delta(
        #[case] lhs: Time<Tai>,
        #[case] rhs: Time<Tai>,
        #[case] expected: TimeDelta,
    ) {
        assert_eq!(expected, lhs - rhs);
    }

    const MAX_FEMTOSECONDS: Subsecond = Subsecond(0.999_999_999_999_999);

    #[rstest]
    #[case::zero_value(Time {scale: Tai, seconds: 0, subsecond: Subsecond::default() }, 12)]
    #[case::one_femtosecond_less_than_an_hour(Time {scale: Tai, seconds: SECONDS_PER_HOUR - 1, subsecond: MAX_FEMTOSECONDS, }, 12)]
    #[case::exactly_one_hour(Time {scale: Tai, seconds: SECONDS_PER_HOUR, subsecond: Subsecond::default() }, 13)]
    #[case::half_day(Time {scale: Tai, seconds: SECONDS_PER_DAY / 2, subsecond: Subsecond::default() }, 0)]
    #[case::negative_half_day(Time {scale: Tai, seconds: -SECONDS_PER_DAY / 2, subsecond: Subsecond::default() }, 0)]
    #[case::one_day_and_one_hour(Time {scale: Tai, seconds: SECONDS_PER_HOUR * 25, subsecond: Subsecond::default() }, 13)]
    #[case::one_femtosecond_less_than_the_epoch(Time {scale: Tai, seconds: - 1, subsecond: MAX_FEMTOSECONDS, }, 11)]
    #[case::one_hour_less_than_the_epoch(Time {scale: Tai, seconds: - SECONDS_PER_HOUR, subsecond: Subsecond::default() }, 11)]
    #[case::one_hour_and_one_femtosecond_less_than_the_epoch(Time {scale: Tai, seconds: - SECONDS_PER_HOUR - 1, subsecond: MAX_FEMTOSECONDS, }, 10)]
    #[case::one_day_less_than_the_epoch(Time {scale: Tai, seconds: - SECONDS_PER_DAY, subsecond: Subsecond::default() }, 12)]
    #[case::one_day_and_one_hour_less_than_the_epoch(Time {scale: Tai, seconds: - SECONDS_PER_DAY - SECONDS_PER_HOUR, subsecond: Subsecond::default() }, 11)]
    #[case::two_days_less_than_the_epoch(Time {scale: Tai, seconds: - SECONDS_PER_DAY * 2, subsecond: Subsecond::default() }, 12)]
    fn test_time_civil_time_hour(#[case] time: Time<Tai>, #[case] expected: u8) {
        let actual = time.hour();
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_value(Time {scale: Tai, seconds: 0, subsecond: Subsecond::default() }, 0)]
    #[case::one_femtosecond_less_than_one_minute(Time {scale: Tai, seconds: SECONDS_PER_MINUTE - 1, subsecond: MAX_FEMTOSECONDS, }, 0)]
    #[case::one_minute(Time {scale: Tai, seconds: SECONDS_PER_MINUTE, subsecond: Subsecond::default() }, 1)]
    #[case::one_femtosecond_less_than_an_hour(Time {scale: Tai, seconds: SECONDS_PER_HOUR - 1, subsecond: MAX_FEMTOSECONDS, }, 59)]
    #[case::exactly_one_hour(Time {scale: Tai, seconds: SECONDS_PER_HOUR, subsecond: Subsecond::default() }, 0)]
    #[case::one_hour_and_one_minute(Time {scale: Tai, seconds: SECONDS_PER_HOUR + SECONDS_PER_MINUTE, subsecond: Subsecond::default() }, 1)]
    #[case::one_hour_less_than_the_epoch(Time {scale: Tai, seconds: - SECONDS_PER_HOUR, subsecond: Subsecond::default() }, 0)]
    #[case::one_femtosecond_less_than_the_epoch(Time {scale: Tai, seconds: - 1, subsecond: MAX_FEMTOSECONDS, }, 59)]
    #[case::one_minute_less_than_the_epoch(Time {scale: Tai, seconds: - SECONDS_PER_MINUTE, subsecond: Subsecond::default() }, 59)]
    #[case::one_minute_and_one_femtosecond_less_than_the_epoch(Time {scale: Tai, seconds: - SECONDS_PER_MINUTE - 1, subsecond: MAX_FEMTOSECONDS, }, 58)]
    fn test_time_civil_time_minute(#[case] time: Time<Tai>, #[case] expected: u8) {
        let actual = time.minute();
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_value(Time {scale: Tai, seconds: 0, subsecond: Subsecond::default() }, 0)]
    #[case::one_femtosecond_less_than_one_second(Time {scale: Tai, seconds: 0, subsecond: MAX_FEMTOSECONDS, }, 0)]
    #[case::one_second(Time {scale: Tai, seconds: 1, subsecond: Subsecond::default() }, 1)]
    #[case::one_femtosecond_less_than_a_minute(Time {scale: Tai, seconds: SECONDS_PER_MINUTE - 1, subsecond: MAX_FEMTOSECONDS, }, 59)]
    #[case::exactly_one_minute(Time {scale: Tai, seconds: SECONDS_PER_MINUTE, subsecond: Subsecond::default() }, 0)]
    #[case::one_minute_and_one_second(Time {scale: Tai, seconds: SECONDS_PER_MINUTE + 1, subsecond: Subsecond::default() }, 1)]
    #[case::one_femtosecond_less_than_the_epoch(Time {scale: Tai, seconds: - 1, subsecond: MAX_FEMTOSECONDS, }, 59)]
    #[case::one_second_less_than_the_epoch(Time {scale: Tai, seconds: - 1, subsecond: Subsecond::default() }, 59)]
    #[case::one_second_and_one_femtosecond_less_than_the_epoch(Time {scale: Tai, seconds: - 2, subsecond: MAX_FEMTOSECONDS, }, 58)]
    #[case::one_minute_less_than_the_epoch(Time {scale: Tai, seconds: - SECONDS_PER_MINUTE, subsecond: Subsecond::default() }, 0)]
    fn test_time_civil_time_second(#[case] time: Time<Tai>, #[case] expected: u8) {
        let actual = time.second();
        assert_eq!(expected, actual);
    }

    const POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE: Time<Tai> = Time {
        scale: Tai,
        seconds: 0,
        subsecond: Subsecond(0.123_456_789_012_345),
    };

    const NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE: Time<Tai> = Time {
        scale: Tai,
        seconds: -1,
        subsecond: Subsecond(0.123_456_789_012_345),
    };

    #[rstest]
    #[case::positive_time_millisecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::millisecond,
        123
    )]
    #[case::positive_time_microsecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::microsecond,
        456
    )]
    #[case::positive_time_nanosecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::nanosecond,
        789
    )]
    #[case::positive_time_picosecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::picosecond,
        12
    )]
    #[case::positive_time_femtosecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::femtosecond,
        345
    )]
    #[case::negative_time_millisecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::millisecond,
        123
    )]
    #[case::negative_time_microsecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::microsecond,
        456
    )]
    #[case::negative_time_nanosecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::nanosecond,
        789
    )]
    #[case::negative_time_picosecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::picosecond,
        12
    )]
    #[case::negative_time_femtosecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::femtosecond,
        345
    )]
    fn test_time_subseconds(
        #[case] time: Time<Tai>,
        #[case] f: fn(&Time<Tai>) -> i64,
        #[case] expected: i64,
    ) {
        let actual = f(&time);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_delta(Time::default(), TimeDelta::default(), Time::default())]
    #[case::pos_delta_no_carry(Time {scale: Tai, seconds: 1, subsecond: Subsecond(0.3) }, TimeDelta { seconds: 1, subsecond: Subsecond(0.6) }, Time {scale: Tai, seconds: 2, subsecond: Subsecond(0.9) })]
    #[case::pos_delta_with_carry(Time {scale: Tai, seconds: 1, subsecond: Subsecond(0.3) }, TimeDelta { seconds: 1, subsecond: Subsecond(0.9) }, Time {scale: Tai, seconds: 3, subsecond: Subsecond(0.2) })]
    #[case::neg_delta_no_carry(Time {scale: Tai, seconds: 1, subsecond: Subsecond(0.6) }, TimeDelta { seconds: -2, subsecond: Subsecond(0.7) }, Time {scale: Tai, seconds: 0, subsecond: Subsecond(0.3) })]
    #[case::neg_delta_with_carry(Time {scale: Tai, seconds: 1, subsecond: Subsecond(0.6) }, TimeDelta { seconds: -2, subsecond: Subsecond(0.3) }, Time { scale: Tai,seconds: -1, subsecond: Subsecond(0.9) })]
    fn test_time_add_time_delta(
        #[case] time: Time<Tai>,
        #[case] delta: TimeDelta,
        #[case] expected: Time<Tai>,
    ) {
        let actual = time + delta;
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_delta(Time::default(), TimeDelta::default(), Time::default())]
    #[case::pos_delta_no_carry(Time {scale: Tai, seconds: 1, subsecond: Subsecond(0.9) }, TimeDelta { seconds: 1, subsecond: Subsecond(0.3) }, Time {scale: Tai,  seconds: 0, subsecond: Subsecond(0.6) })]
    #[case::pos_delta_with_carry(Time {scale: Tai, seconds: 1, subsecond: Subsecond(0.3) }, TimeDelta { seconds: 1, subsecond: Subsecond(0.4) }, Time {scale: Tai,  seconds: -1, subsecond: Subsecond(0.9) })]
    #[case::neg_delta_no_carry(Time {scale: Tai, seconds: 1, subsecond: Subsecond(0.6) }, TimeDelta { seconds: -1, subsecond: Subsecond(0.7) }, Time {scale: Tai, seconds: 1, subsecond: Subsecond(0.9) })]
    #[case::neg_delta_with_carry(Time {scale: Tai, seconds: 1, subsecond: Subsecond(0.9) }, TimeDelta { seconds: -1, subsecond: Subsecond(0.3) }, Time {scale: Tai, seconds: 2, subsecond: Subsecond(0.6) })]
    fn test_time_sub_time_delta(
        #[case] time: Time<Tai>,
        #[case] delta: TimeDelta,
        #[case] expected: Time<Tai>,
    ) {
        let actual = time - delta;
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(Time::default(), Time::default())]
    #[case(Time::default(), Time::new(Tai, 1, Subsecond(0.9)))]
    #[case(Time::new(Tai, 0, Subsecond(0.9)), Time::new(Tai, 1, Subsecond(0.6)))]
    #[case(Time::new(Tai, 1, Subsecond(0.9)), Time::default())]
    #[case(Time::new(Tai, 1, Subsecond(0.6)), Time::new(Tai, 0, Subsecond(0.9)))]
    #[case(Time::new(Tai, 1, Subsecond(0.6)), Time::new(Tai, -1, Subsecond(0.9)), )]
    #[case(Time::new(Tai, -1, Subsecond(0.9)), Time::new(Tai, 1, Subsecond(0.6)), )]
    #[case(Time::new(Tai, 1, Subsecond(0.9)), Time::new(Tai, -1, Subsecond(0.6)), )]
    #[case(Time::new(Tai, -1, Subsecond(0.6)), Time::new(Tai, 1, Subsecond(0.9)), )]
    fn test_time_sub_time(#[case] time1: Time<Tai>, #[case] time2: Time<Tai>) {
        let delta = time2 - time1;
        let actual = time1 + delta;
        assert_eq!(actual, time2);
    }

    #[rstest]
    #[case::at_the_epoch(Time::default(), 0.0)]
    #[case::exactly_one_day_after_the_epoch(
    Time {
        scale: Tai,
    seconds: SECONDS_PER_DAY,
    subsecond: Subsecond::default(),
    },
    1.0
    )]
    #[case::exactly_one_day_before_the_epoch(
    Time {
        scale   : Tai,
    seconds: - SECONDS_PER_DAY,
    subsecond: Subsecond::default(),
    },
    - 1.0
    )]
    #[case::a_partial_number_of_days_after_the_epoch(
    Time {
        scale   : Tai,
    seconds: (SECONDS_PER_DAY / 2) * 3,
    subsecond: Subsecond(0.5),
    },
    1.5000057870370371
    )]
    fn test_time_days_since_j2000(#[case] time: Time<Tai>, #[case] expected: f64) {
        let actual = time.days_since_j2000();
        assert_float_eq!(expected, actual, abs <= 1e-12);
    }

    #[rstest]
    #[case::at_the_epoch(Time::default(), 0.0)]
    #[case::exactly_one_century_after_the_epoch(
    Time {
        scale   : Tai,
    seconds: SECONDS_PER_JULIAN_CENTURY,
    subsecond: Subsecond::default(),
    },
    1.0
    )]
    #[case::exactly_one_century_before_the_epoch(
    Time {
        scale   : Tai,
    seconds: - SECONDS_PER_JULIAN_CENTURY,
    subsecond: Subsecond::default(),
    },
    - 1.0
    )]
    #[case::a_partial_number_of_centuries_after_the_epoch(
    Time {
        scale   : Tai,
    seconds: (SECONDS_PER_JULIAN_CENTURY / 2) * 3,
    subsecond: Subsecond(0.5),
    },
    1.5000000001584404
    )]
    fn test_time_centuries_since_j2000(#[case] time: Time<Tai>, #[case] expected: f64) {
        let actual = time.centuries_since_j2000();
        assert_float_eq!(expected, actual, abs <= 1e-12,);
    }

    #[rstest]
    #[case::j2000(Time::default(), Date::new(2000, 1, 1).unwrap())]
    #[case::next_day(Time {scale: Tai, seconds: SECONDS_PER_DAY, subsecond: Subsecond::default()}, Date::new(2000, 1, 2).unwrap())]
    #[case::leap_year(Time {scale: Tai, seconds: SECONDS_PER_DAY * 366, subsecond: Subsecond::default()}, Date::new(2001, 1, 1).unwrap())]
    #[case::non_leap_year(Time {scale: Tai, seconds: SECONDS_PER_DAY * (366 + 365), subsecond: Subsecond::default()}, Date::new(2002, 1, 1).unwrap())]
    #[case::negative_time(Time {scale: Tai, seconds: -SECONDS_PER_DAY, subsecond: Subsecond::default()}, Date::new(1999, 12, 31).unwrap())]
    fn test_time_calendar_date(#[case] time: Time<Tai>, #[case] expected: Date) {
        assert_eq!(expected, time.date());
        assert_eq!(expected.year(), time.year());
        assert_eq!(expected.month(), time.month());
        assert_eq!(expected.day(), time.day());
    }

    #[test]
    fn test_time_scale() {
        let time: Time<Tai> = Time::default();
        assert_eq!(time.scale(), Tai);
    }

    #[test]
    fn test_time_override_scale() {
        let time: Time<Tai> = Time::default();
        let time = time.with_scale(Tt);
        assert_eq!(time.scale(), Tt);
    }

    #[test]
    fn test_time_leap_second_outside_utc() {
        let actual = time!(Tai, 2000, 1, 1, 23, 59, 60.0);
        let expected = Err(TimeError::LeapSecondOutsideUtc);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_julian_date_out_of_range_ord() {
        let actual = JulianDateOutOfRange(-f64::NAN).partial_cmp(&JulianDateOutOfRange(f64::NAN));
        let expected = Some(Ordering::Less);
        assert_eq!(actual, expected);
        let actual = JulianDateOutOfRange(-f64::NAN).cmp(&JulianDateOutOfRange(f64::NAN));
        let expected = Ordering::Less;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_time_to_delta() {
        let time = time!(Tai, 2000, 1, 1, 12, 0, 0.0).unwrap();
        let actual = time.to_delta();
        let expected = TimeDelta::from_seconds(0);
        assert_eq!(actual, expected);
    }
}
