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
//! [UTC] and [Date] are used strictly as an I/O formats, avoiding much of the complexity inherent
//! in working with leap seconds.

use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};

use crate::base_time::BaseTime;
use crate::calendar_dates::{CalendarDate, Date};
use crate::deltas::TimeDelta;
use crate::julian_dates::{Epoch, JulianDate, Unit};
use crate::subsecond::Subsecond;
use crate::time_scales::TimeScale;
use crate::transformations::TransformTimeScale;
use crate::utc::{UTCDateTime, UTC};
use crate::wall_clock::WallClock;

pub mod base_time;
pub mod calendar_dates;
pub mod constants;
pub mod deltas;
pub mod errors;
pub mod julian_dates;
pub mod subsecond;
pub mod time_scales;
pub mod transformations;
pub mod utc;
pub mod wall_clock;

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
    /// Instantiates a [Time] in the given scale from seconds and femtoseconds since the epoch.
    pub fn new(scale: T, seconds: i64, subsecond: Subsecond) -> Self {
        Self {
            scale,
            timestamp: BaseTime::new(seconds, subsecond),
        }
    }

    /// Instantiates a [Time] in the given scale from a [BaseTime].
    pub const fn from_base_time(scale: T, timestamp: BaseTime) -> Self {
        Self { scale, timestamp }
    }

    /// Instantiates a [Time] in the given scale from a [UTCDateTime].
    pub fn from_utc_datetime(scale: T, datetime: UTCDateTime) -> Self {
        let timestamp = BaseTime::from_utc_datetime(datetime);
        Self { scale, timestamp }
    }

    /// Instantiates a [Time] in the given scale from a [Date] and [UTC] instance.
    pub fn from_date_and_utc_timestamp(scale: T, date: Date, utc: UTC) -> Self {
        let timestamp = BaseTime::from_date_and_utc_timestamp(date, utc);
        Self { scale, timestamp }
    }

    /// Returns the epoch for the given [Epoch] in the given timescale.
    pub fn from_epoch(scale: T, epoch: Epoch) -> Self {
        let timestamp = BaseTime::from_epoch(epoch);
        Self { scale, timestamp }
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
        write!(f, "{} {}", self.timestamp, T::ABBREVIATION)
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

impl<T: TimeScale + Copy> WallClock for Time<T> {
    fn hour(&self) -> i64 {
        self.timestamp.hour()
    }

    fn minute(&self) -> i64 {
        self.timestamp.minute()
    }

    fn second(&self) -> i64 {
        self.timestamp.second()
    }

    fn millisecond(&self) -> i64 {
        self.timestamp.millisecond()
    }

    fn microsecond(&self) -> i64 {
        self.timestamp.microsecond()
    }

    fn nanosecond(&self) -> i64 {
        self.timestamp.nanosecond()
    }

    fn picosecond(&self) -> i64 {
        self.timestamp.picosecond()
    }

    fn femtosecond(&self) -> i64 {
        self.timestamp.femtosecond()
    }
}

impl<T: TimeScale + Copy> CalendarDate for Time<T> {
    fn calendar_date(&self) -> Date {
        self.timestamp.calendar_date()
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use lox_utils::constants::f64::time::DAYS_PER_JULIAN_CENTURY;
    use mockall::predicate;

    use crate::calendar_dates::Calendar::Gregorian;
    use crate::time_scales::{TAI, TDB, TT};
    use crate::transformations::MockTransformTimeScale;
    use crate::utc::UTC;
    use crate::Time;

    use super::*;

    #[test]
    fn test_time_new() {
        let scale = TAI;
        let seconds = 1234567890;
        let subsecond = Subsecond(0.9876543210);
        let expected = Time {
            scale,
            timestamp: BaseTime { seconds, subsecond },
        };
        let actual = Time::new(scale, seconds, subsecond);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_from_utc_datetime() {
        let scale = TAI;
        let datetime = UTCDateTime::new(Date::new(2023, 1, 1).unwrap(), UTC::default()).unwrap();
        let expected = Time {
            scale,
            timestamp: BaseTime::from_utc_datetime(datetime),
        };
        let actual = Time::from_utc_datetime(scale, datetime);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_from_date_and_utc_timestamp() {
        let scale = TAI;
        let date = Date::new(2023, 1, 1).unwrap();
        let utc = UTC::new(12, 0, 0, Subsecond::default()).unwrap();
        let expected = Time {
            scale,
            timestamp: BaseTime::from_date_and_utc_timestamp(date, utc),
        };
        let actual = Time::from_date_and_utc_timestamp(scale, date, utc);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_display() {
        let time = Time::j2000(TAI);
        let expected = "12:00:00.000.000.000.000.000 TAI".to_string();
        let actual = time.to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_j2000() {
        let actual = Time::j2000(TAI);
        let expected = Time {
            scale: TAI,
            timestamp: BaseTime::default(),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_jd0() {
        let actual = Time::jd0(TAI);
        let expected = Time::from_base_time(
            TAI,
            BaseTime {
                seconds: -211813488000,
                subsecond: Subsecond::default(),
            },
        );
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_seconds() {
        let time = Time::new(TAI, 1234567890, Subsecond(0.9876543210));
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
        let time = Time::new(TAI, 1234567890, Subsecond(0.9876543210));
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
        let actual = Time::from_base_time(TAI, base_time).days_since_j2000();
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
        let actual = Time::from_base_time(TAI, base_time).centuries_since_j2000();
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
    fn test_time_wall_clock_hour() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.hour();
        let actual = Time::from_base_time(TAI, base_time).hour();
        assert_eq!(
            expected, actual,
            "expected Time to have hour {}, but got {}",
            expected, actual
        );
    }

    #[test]
    fn test_time_wall_clock_minute() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.minute();
        let actual = Time::from_base_time(TAI, base_time).minute();
        assert_eq!(
            expected, actual,
            "expected Time to have minute {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_wall_clock_second() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.second();
        let actual = Time::from_base_time(TAI, base_time).second();
        assert_eq!(
            expected, actual,
            "expected Time to have second {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_wall_clock_millisecond() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.millisecond();
        let actual = Time::from_base_time(TAI, base_time).millisecond();
        assert_eq!(
            expected, actual,
            "expected Time to have millisecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_wall_clock_microsecond() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.microsecond();
        let actual = Time::from_base_time(TAI, base_time).microsecond();
        assert_eq!(
            expected, actual,
            "expected Time to have microsecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_wall_clock_nanosecond() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.nanosecond();
        let actual = Time::from_base_time(TAI, base_time).nanosecond();
        assert_eq!(
            expected, actual,
            "expected Time to have nanosecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_wall_clock_picosecond() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.picosecond();
        let actual = Time::from_base_time(TAI, base_time).picosecond();
        assert_eq!(
            expected, actual,
            "expected Time to have picosecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_wall_clock_femtosecond() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: Subsecond(0.9876543210),
        };
        let expected = base_time.femtosecond();
        let actual = Time::from_base_time(TAI, base_time).femtosecond();
        assert_eq!(
            expected, actual,
            "expected Time to have femtosecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_from_scale() {
        let time = Time::j2000(TAI);
        let mut transformer = MockTransformTimeScale::<TAI, TT>::new();
        let expected = Time::j2000(TT);

        transformer
            .expect_transform()
            .with(predicate::eq(time))
            .return_const(expected);

        let actual = Time::from_scale(time, transformer);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_into_scale() {
        let time = Time::j2000(TAI);
        let mut transformer = MockTransformTimeScale::<TAI, TT>::new();
        let expected = Time::j2000(TT);

        transformer
            .expect_transform()
            .with(predicate::eq(time))
            .return_const(expected);

        let actual = time.into_scale(transformer);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_julian_date() {
        let time = Time::jd0(TDB);
        assert_eq!(time.julian_date(Epoch::JulianDate, Unit::Days), 0.0);
        assert_eq!(time.seconds_since_julian_epoch(), 0.0);
        assert_eq!(time.days_since_julian_epoch(), 0.0);
        assert_eq!(time.centuries_since_julian_epoch(), 0.0);
    }

    #[test]
    fn test_modified_julian_date() {
        let time = Time::mjd0(TDB);
        assert_eq!(time.julian_date(Epoch::ModifiedJulianDate, Unit::Days), 0.0);
        assert_eq!(time.seconds_since_modified_julian_epoch(), 0.0);
        assert_eq!(time.days_since_modified_julian_epoch(), 0.0);
        assert_eq!(time.centuries_since_modified_julian_epoch(), 0.0);
    }

    #[test]
    fn test_j1950() {
        let time = Time::j1950(TDB);
        assert_eq!(time.julian_date(Epoch::J1950, Unit::Days), 0.0);
        assert_eq!(time.seconds_since_j1950(), 0.0);
        assert_eq!(time.days_since_j1950(), 0.0);
        assert_eq!(time.centuries_since_j1950(), 0.0);
    }

    #[test]
    fn test_j2000() {
        let time = Time::j2000(TDB);
        assert_eq!(time.julian_date(Epoch::J2000, Unit::Days), 0.0);
        assert_eq!(time.seconds_since_j2000(), 0.0);
        assert_eq!(time.days_since_j2000(), 0.0);
        assert_eq!(time.centuries_since_j2000(), 0.0);
    }

    #[test]
    fn test_j2100() {
        let date = Date::new_unchecked(Gregorian, 2100, 1, 1);
        let utc = UTC::new(12, 0, 0, Subsecond::default()).expect("should be valid");
        let time = Time {
            scale: TDB,
            timestamp: BaseTime::from_date_and_utc_timestamp(date, utc),
        };
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
        let date = Date::new_unchecked(Gregorian, 2100, 1, 2);
        let utc = UTC::new(0, 0, 0, Subsecond::default()).expect("should be valid");
        let time = Time {
            scale: TDB,
            timestamp: BaseTime::from_date_and_utc_timestamp(date, utc),
        };
        let (jd1, jd2) = time.two_part_julian_date();
        assert_eq!(jd1, 2451545.0 + DAYS_PER_JULIAN_CENTURY);
        assert_eq!(jd2, 0.5);
    }

    #[test]
    fn test_time_add_time_delta() {
        let time = Time::j2000(TAI);
        let delta = TimeDelta::from_decimal_seconds(1.5).unwrap();
        let expected = Time {
            scale: TAI,
            timestamp: time.timestamp + delta,
        };
        let actual = Time::j2000(TAI) + delta;
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_sub_time_delta() {
        let time = Time::j2000(TAI);
        let delta = TimeDelta::from_decimal_seconds(1.5).unwrap();
        let expected = Time {
            scale: TAI,
            timestamp: time.timestamp - delta,
        };
        let actual = Time::j2000(TAI) - delta;
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_calendar_date() {
        let base_time = BaseTime::default();
        let expected = base_time.calendar_date();
        let tai = Time::from_base_time(TAI, base_time);
        let actual = tai.calendar_date();
        assert_eq!(expected, actual);
    }
}
