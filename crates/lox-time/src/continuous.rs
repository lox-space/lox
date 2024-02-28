/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module continuous provides representations and functions for working with time scales without leap seconds.
//!
//! Continuous times are represented with femtosecond precision.
//!
//! The supported timescales are specified by [TimeScale].

use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};

use num::{abs, Num, ToPrimitive};

use deltas::TimeDelta;

use crate::constants::i64::{
    SECONDS_PER_DAY, SECONDS_PER_HALF_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE,
};
use crate::constants::julian_dates::{
    SECONDS_BETWEEN_J1950_AND_J2000, SECONDS_BETWEEN_JD_AND_J2000, SECONDS_BETWEEN_MJD_AND_J2000,
};
use crate::constants::u64::{
    FEMTOSECONDS_PER_MICROSECOND, FEMTOSECONDS_PER_MILLISECOND, FEMTOSECONDS_PER_NANOSECOND,
    FEMTOSECONDS_PER_PICOSECOND, FEMTOSECONDS_PER_SECOND,
};
use crate::continuous::julian_dates::{Epoch, JulianDate, Unit};
use crate::continuous::transform::TransformTimeScale;
use crate::dates::Date;
use crate::utc::{UTCDateTime, UTC};
use crate::{constants, WallClock};

pub mod transform;

pub mod deltas;
pub mod julian_dates;

pub trait Subsecond: Num + ToPrimitive {}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
/// `BaseTime` is the base time representation for time scales without leap seconds. It is measured relative to
/// J2000. `BaseTime::default()` represents the epoch itself.
///
/// `BaseTime` has femtosecond precision, and supports times within 292 billion years either side of the epoch.
pub struct BaseTime<S: Subsecond> {
    // The sign of the time is determined exclusively by the sign of the `second` field. `femtoseconds` is always the
    // positive count of femtoseconds since the last whole second. For example, one femtosecond before the epoch is
    // represented as
    // ```
    // let time = BaseTime {
    //     seconds: -1,
    //     femtoseconds: FEMTOSECONDS_PER_SECOND - 1,
    // };
    // ```
    seconds: i64,
    subsecond: S,
}

impl BaseTime<u64> {
    pub fn new(seconds: i64, femtoseconds: u64) -> Self {
        let seconds = seconds + (femtoseconds / FEMTOSECONDS_PER_SECOND) as i64;
        let femtoseconds = femtoseconds % FEMTOSECONDS_PER_SECOND;
        Self {
            seconds,
            subsecond: femtoseconds,
        }
    }

    pub fn from_epoch(epoch: Epoch) -> Self {
        match epoch {
            Epoch::JulianDate => Self::new(-SECONDS_BETWEEN_JD_AND_J2000, 0),
            Epoch::ModifiedJulianDate => Self::new(-SECONDS_BETWEEN_MJD_AND_J2000, 0),
            Epoch::J1950 => Self::new(-SECONDS_BETWEEN_J1950_AND_J2000, 0),
            Epoch::J2000 => Self::new(0, 0),
        }
    }

    fn is_negative(&self) -> bool {
        self.seconds < 0
    }

    pub fn seconds(&self) -> i64 {
        self.seconds
    }

    pub fn femtoseconds(&self) -> u64 {
        self.subsecond
    }

    pub fn seconds_from_epoch(&self, epoch: Epoch) -> i64 {
        match epoch {
            Epoch::JulianDate => self.seconds + SECONDS_BETWEEN_JD_AND_J2000,
            Epoch::ModifiedJulianDate => self.seconds + SECONDS_BETWEEN_MJD_AND_J2000,
            Epoch::J1950 => self.seconds + SECONDS_BETWEEN_J1950_AND_J2000,
            Epoch::J2000 => self.seconds,
        }
    }
}

impl<T: Num> Display for BaseTime<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}.{:03}.{:03}.{:03}.{:03}.{:03}.{:03}",
            self.hour(),
            self.minute(),
            self.second(),
            self.millisecond(),
            self.microsecond(),
            self.nanosecond(),
            self.picosecond(),
            self.femtosecond(),
            self.femtosecond(),
        )
    }
}

impl Add<TimeDelta> for BaseTime<u64> {
    type Output = Self;

    /// The implementation of [Add] for [BaseTime] follows the default Rust rules for integer overflow, which
    /// should be sufficient for all practical purposes.
    fn add(self, rhs: TimeDelta) -> Self::Output {
        let mut femtoseconds = self.subsecond + rhs.femtoseconds;
        let mut seconds = self.seconds + rhs.seconds as i64;
        if femtoseconds >= FEMTOSECONDS_PER_SECOND {
            seconds += 1;
            femtoseconds -= FEMTOSECONDS_PER_SECOND;
        }
        Self {
            seconds,
            subsecond: femtoseconds,
        }
    }
}

impl Sub<TimeDelta> for BaseTime<u64> {
    type Output = Self;

    /// The implementation of [Sub] for [BaseTime] follows the default Rust rules for integer overflow, which
    /// should be sufficient for all practical purposes.
    fn sub(self, rhs: TimeDelta) -> Self::Output {
        let mut seconds = self.seconds - rhs.seconds as i64;
        let mut femtoseconds = self.subsecond;
        if rhs.femtoseconds > self.subsecond {
            seconds -= 1;
            femtoseconds = FEMTOSECONDS_PER_SECOND - (rhs.femtoseconds - self.subsecond);
        } else {
            femtoseconds -= rhs.femtoseconds;
        }
        Self {
            seconds,
            subsecond: femtoseconds,
        }
    }
}

impl WallClock for BaseTime<u64> {
    fn hour(&self) -> i64 {
        // Since J2000 is taken from midday, we offset by half a day to get the wall clock hour.
        let day_seconds: i64 = if self.is_negative() {
            SECONDS_PER_DAY - (abs(self.seconds) + SECONDS_PER_HALF_DAY) % SECONDS_PER_DAY
        } else {
            (self.seconds + SECONDS_PER_HALF_DAY) % SECONDS_PER_DAY
        };
        day_seconds / SECONDS_PER_HOUR
    }

    fn minute(&self) -> i64 {
        let hour_seconds: i64 = if self.is_negative() {
            SECONDS_PER_HOUR - abs(self.seconds) % SECONDS_PER_HOUR
        } else {
            self.seconds % SECONDS_PER_HOUR
        };
        hour_seconds / SECONDS_PER_MINUTE
    }

    fn second(&self) -> i64 {
        if self.is_negative() {
            SECONDS_PER_MINUTE - abs(self.seconds) % SECONDS_PER_MINUTE
        } else {
            self.seconds % SECONDS_PER_MINUTE
        }
    }

    fn millisecond(&self) -> i64 {
        (self.subsecond / FEMTOSECONDS_PER_MILLISECOND) as i64
    }

    fn microsecond(&self) -> i64 {
        (self.subsecond / FEMTOSECONDS_PER_MICROSECOND % 1000) as i64
    }

    fn nanosecond(&self) -> i64 {
        (self.subsecond / FEMTOSECONDS_PER_NANOSECOND % 1000) as i64
    }

    fn picosecond(&self) -> i64 {
        (self.subsecond / FEMTOSECONDS_PER_PICOSECOND % 1000) as i64
    }

    fn femtosecond(&self) -> i64 {
        (self.subsecond % 1000) as i64
    }
}

impl<T: Num> JulianDate for BaseTime<T> {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        let mut seconds = self.seconds_from_epoch(epoch).to_f64().unwrap();
        let fraction = self.subsecond.to_f64().unwrap() / constants::f64::FEMTOSECONDS_PER_SECOND;
        seconds += fraction;
        match unit {
            Unit::Seconds => seconds,
            Unit::Days => seconds / constants::f64::SECONDS_PER_DAY,
            Unit::Centuries => seconds / constants::f64::SECONDS_PER_JULIAN_CENTURY,
        }
    }

    fn two_part_julian_date(&self) -> (f64, f64) {
        let days = self.julian_date(Epoch::JulianDate, Unit::Days);
        (days.trunc(), days.fract())
    }
}

/// Marker trait with associated constants denoting a continuous astronomical time scale.
pub trait TimeScale {
    const ABBREVIATION: &'static str;
    const NAME: &'static str;
}

/// International Atomic Time.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TAI;

impl TimeScale for TAI {
    const ABBREVIATION: &'static str = "TAI";
    const NAME: &'static str = "International Atomic Time";
}

/// Barycentric Coordinate Time.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TCB;

impl TimeScale for TCB {
    const ABBREVIATION: &'static str = "TCB";
    const NAME: &'static str = "Barycentric Coordinate Time";
}

/// Geocentric Coordinate Time.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TCG;

impl TimeScale for TCG {
    const ABBREVIATION: &'static str = "TCG";
    const NAME: &'static str = "Geocentric Coordinate Time";
}

/// Barycentric Dynamical Time.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TDB;

impl TimeScale for TDB {
    const ABBREVIATION: &'static str = "TDB";
    const NAME: &'static str = "Barycentric Dynamical Time";
}

/// Terrestrial Time.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TT;

impl TimeScale for TT {
    const ABBREVIATION: &'static str = "TT";
    const NAME: &'static str = "Terrestrial Time";
}

/// Universal Time.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UT1;

impl TimeScale for UT1 {
    const ABBREVIATION: &'static str = "UT1";
    const NAME: &'static str = "Universal Time";
}

/// An instant in time in a given time scale.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Time<S, T>
where
    S: Subsecond,
    T: TimeScale + Copy,
{
    scale: T,
    timestamp: BaseTime<S>,
}

impl<T: TimeScale + Copy> Time<u64, T> {
    /// Instantiates a [Time] in the given scale from seconds and femtoseconds since the epoch.
    pub fn new(scale: T, seconds: i64, femtoseconds: u64) -> Self {
        Self {
            scale,
            timestamp: BaseTime::new(seconds, femtoseconds),
        }
    }

    /// Instantiates a [Time] in the given scale from a [BaseTime].
    pub fn from_base_time(scale: T, timestamp: BaseTime<S>) -> Self {
        Self { scale, timestamp }
    }

    /// Instantiates a [Time] in the given scale from a date and UTC timestamp.
    pub fn from_date_and_utc_timestamp(scale: T, date: Date, time: UTC) -> Self {
        let day_in_seconds = date.j2000() * SECONDS_PER_DAY - SECONDS_PER_DAY / 2;
        let hour_in_seconds = time.hour() * SECONDS_PER_HOUR;
        let minute_in_seconds = time.minute() * SECONDS_PER_MINUTE;
        let seconds = day_in_seconds + hour_in_seconds + minute_in_seconds + time.second();
        let femtoseconds = time.subsecond_as_femtoseconds();
        let base_time = BaseTime {
            seconds,
            subsecond: femtoseconds,
        };
        Self::from_base_time(scale, base_time)
    }

    /// Instantiates a [Time] in the given scale from a UTC datetime.
    pub fn from_utc_datetime(scale: T, dt: UTCDateTime) -> Self {
        Self::from_date_and_utc_timestamp(scale, dt.date(), dt.time())
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
    pub fn femtoseconds(&self) -> u64 {
        self.timestamp.subsecond
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

/// CalendarDate allows continuous time formats to report their date in their respective calendar.
pub trait CalendarDate {
    fn date(&self) -> Date;
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use mockall::predicate;
    use rstest::rstest;

    use crate::constants::i64::SECONDS_PER_JULIAN_CENTURY;
    use crate::continuous::transform::MockTransformTimeScale;
    use crate::dates::Calendar::Gregorian;

    use super::*;

    #[rstest]
    #[case::no_femtosecond_wrap(1, 1, BaseTime { seconds: 1, femtoseconds: 1 })]
    #[case::femtosecond_wrap(1, FEMTOSECONDS_PER_SECOND, BaseTime { seconds: 2, femtoseconds: 0 })]
    fn test_base_time_new(
        #[case] seconds: i64,
        #[case] femtoseconds: u64,
        #[case] expected: BaseTime,
    ) {
        let actual = BaseTime::new(seconds, femtoseconds);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_base_time_is_negative() {
        assert!(BaseTime {
            seconds: -1,
            subsecond: 0,
        }
        .is_negative());
        assert!(!BaseTime {
            seconds: 0,
            subsecond: 0,
        }
        .is_negative());
        assert!(!BaseTime {
            seconds: 1,
            subsecond: 0,
        }
        .is_negative());
    }

    #[test]
    fn test_base_time_seconds() {
        let time = BaseTime {
            seconds: 123,
            subsecond: 0,
        };
        assert_eq!(time.seconds(), 123);
    }

    #[test]
    fn test_base_time_femtoseconds() {
        let time = BaseTime {
            seconds: 0,
            subsecond: 123,
        };
        assert_eq!(time.femtoseconds(), 123);
    }

    #[rstest]
    #[case::zero_value(BaseTime { seconds: 0, femtoseconds: 0 }, 12)]
    #[case::one_femtosecond_less_than_an_hour(BaseTime { seconds: SECONDS_PER_HOUR - 1, femtoseconds: FEMTOSECONDS_PER_SECOND - 1 }, 12)]
    #[case::exactly_one_hour(BaseTime { seconds: SECONDS_PER_HOUR, femtoseconds: 0 }, 13)]
    #[case::one_day_and_one_hour(BaseTime { seconds: SECONDS_PER_HOUR * 25, femtoseconds: 0 }, 13)]
    #[case::one_femtosecond_less_than_the_epoch(BaseTime { seconds: - 1, femtoseconds: FEMTOSECONDS_PER_SECOND - 1 }, 11)]
    #[case::one_hour_less_than_the_epoch(BaseTime { seconds: - SECONDS_PER_HOUR, femtoseconds: 0 }, 11)]
    #[case::one_hour_and_one_femtosecond_less_than_the_epoch(BaseTime { seconds: - SECONDS_PER_HOUR - 1, femtoseconds: FEMTOSECONDS_PER_SECOND - 1 }, 10)]
    #[case::one_day_less_than_the_epoch(BaseTime { seconds: - SECONDS_PER_DAY, femtoseconds: 0 }, 12)]
    #[case::two_days_less_than_the_epoch(BaseTime { seconds: - SECONDS_PER_DAY * 2, femtoseconds: 0 }, 12)]
    fn test_base_time_wall_clock_hour(#[case] time: BaseTime, #[case] expected: i64) {
        let actual = time.hour();
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_value(BaseTime { seconds: 0, femtoseconds: 0 }, 0)]
    #[case::one_femtosecond_less_than_one_minute(BaseTime { seconds: SECONDS_PER_MINUTE - 1, femtoseconds: FEMTOSECONDS_PER_SECOND - 1 }, 0)]
    #[case::one_minute(BaseTime { seconds: SECONDS_PER_MINUTE, femtoseconds: 0 }, 1)]
    #[case::one_femtosecond_less_than_an_hour(BaseTime { seconds: SECONDS_PER_HOUR - 1, femtoseconds: FEMTOSECONDS_PER_SECOND - 1 }, 59)]
    #[case::exactly_one_hour(BaseTime { seconds: SECONDS_PER_HOUR, femtoseconds: 0 }, 0)]
    #[case::one_hour_and_one_minute(BaseTime { seconds: SECONDS_PER_HOUR + SECONDS_PER_MINUTE, femtoseconds: 0 }, 1)]
    #[case::one_femtosecond_less_than_the_epoch(BaseTime { seconds: - 1, femtoseconds: FEMTOSECONDS_PER_SECOND - 1 }, 59)]
    #[case::one_minute_less_than_the_epoch(BaseTime { seconds: - SECONDS_PER_MINUTE, femtoseconds: 0 }, 59)]
    #[case::one_minute_and_one_femtosecond_less_than_the_epoch(BaseTime { seconds: - SECONDS_PER_MINUTE - 1, femtoseconds: FEMTOSECONDS_PER_SECOND - 1 }, 58)]
    fn test_base_time_wall_clock_minute(#[case] time: BaseTime, #[case] expected: i64) {
        let actual = time.minute();
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_value(BaseTime { seconds: 0, femtoseconds: 0 }, 0)]
    #[case::one_femtosecond_less_than_one_second(BaseTime { seconds: 0, femtoseconds: FEMTOSECONDS_PER_SECOND - 1 }, 0)]
    #[case::one_second(BaseTime { seconds: 1, femtoseconds: 0 }, 1)]
    #[case::one_femtosecond_less_than_a_minute(BaseTime { seconds: SECONDS_PER_MINUTE - 1, femtoseconds: FEMTOSECONDS_PER_SECOND - 1 }, 59)]
    #[case::exactly_one_minute(BaseTime { seconds: SECONDS_PER_MINUTE, femtoseconds: 0 }, 0)]
    #[case::one_minute_and_one_second(BaseTime { seconds: SECONDS_PER_MINUTE + 1, femtoseconds: 0 }, 1)]
    #[case::one_femtosecond_less_than_the_epoch(BaseTime { seconds: - 1, femtoseconds: FEMTOSECONDS_PER_SECOND - 1 }, 59)]
    #[case::one_second_less_than_the_epoch(BaseTime { seconds: - 1, femtoseconds: 0 }, 59)]
    #[case::one_second_and_one_femtosecond_less_than_the_epoch(BaseTime { seconds: - 2, femtoseconds: FEMTOSECONDS_PER_SECOND - 1 }, 58)]
    fn test_base_time_wall_clock_second(#[case] time: BaseTime, #[case] expected: i64) {
        let actual = time.second();
        assert_eq!(expected, actual);
    }

    const POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE: BaseTime = BaseTime {
        seconds: 0,
        subsecond: 123_456_789_012_345,
    };

    const NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE: BaseTime = BaseTime {
        seconds: -1,
        subsecond: 123_456_789_012_345,
    };

    #[rstest]
    #[case::positive_time_millisecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::millisecond,
        123
    )]
    #[case::positive_time_microsecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::microsecond,
        456
    )]
    #[case::positive_time_nanosecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::nanosecond,
        789
    )]
    #[case::positive_time_picosecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::picosecond,
        12
    )]
    #[case::positive_time_femtosecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::femtosecond,
        345
    )]
    #[case::negative_time_millisecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::millisecond,
        123
    )]
    #[case::negative_time_microsecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::microsecond,
        456
    )]
    #[case::negative_time_nanosecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::nanosecond,
        789
    )]
    #[case::negative_time_picosecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::picosecond,
        12
    )]
    #[case::negative_time_femtosecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::femtosecond,
        345
    )]
    fn test_base_time_subseconds(
        #[case] time: BaseTime,
        #[case] f: fn(&BaseTime) -> i64,
        #[case] expected: i64,
    ) {
        let actual = f(&time);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::positive_time_no_femtosecond_wrap(
    TimeDelta {
    seconds: 1,
    femtoseconds: 1
    },
    BaseTime {
    seconds: 1,
    femtoseconds: 0
    },
    BaseTime {
    seconds: 2,
    femtoseconds: 1
    }
    )]
    #[case::positive_time_femtosecond_wrap(
    TimeDelta {
    seconds: 1,
    femtoseconds: 2
    },
    BaseTime {
    seconds: 1,
    femtoseconds: FEMTOSECONDS_PER_SECOND - 1
    },
    BaseTime {
    seconds: 3,
    femtoseconds: 1
    }
    )]
    #[case::negative_time_no_femtosecond_wrap(
    TimeDelta {
    seconds: 1,
    femtoseconds: 1
    },
    BaseTime {
    seconds: - 1,
    femtoseconds: 0
    },
    BaseTime {
    seconds: 0,
    femtoseconds: 1
    }
    )]
    #[case::negative_time_femtosecond_wrap(
    TimeDelta {
    seconds: 1,
    femtoseconds: 2
    },
    BaseTime {
    seconds: - 1,
    femtoseconds: FEMTOSECONDS_PER_SECOND - 1
    },
    BaseTime {
    seconds: 1,
    femtoseconds: 1
    }
    )]
    fn test_base_time_add_time_delta(
        #[case] delta: TimeDelta,
        #[case] time: BaseTime,
        #[case] expected: BaseTime,
    ) {
        let actual = time + delta;
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::positive_time_no_femtosecond_wrap(
    TimeDelta {
    seconds: 1,
    femtoseconds: 1
    },
    BaseTime {
    seconds: 2,
    femtoseconds: 2
    },
    BaseTime {
    seconds: 1,
    femtoseconds: 1
    }
    )]
    #[case::positive_time_femtosecond_wrap(
    TimeDelta {
    seconds: 1,
    femtoseconds: 2
    },
    BaseTime {
    seconds: 2,
    femtoseconds: 1
    },
    BaseTime {
    seconds: 0,
    femtoseconds: FEMTOSECONDS_PER_SECOND - 1
    }
    )]
    #[case::negative_time_no_femtosecond_wrap(
    TimeDelta {
    seconds: 1,
    femtoseconds: 1
    },
    BaseTime {
    seconds: - 1,
    femtoseconds: 2
    },
    BaseTime {
    seconds: - 2,
    femtoseconds: 1
    }
    )]
    #[case::negative_time_femtosecond_wrap(
    TimeDelta {
    seconds: 1,
    femtoseconds: 2
    },
    BaseTime {
    seconds: - 1,
    femtoseconds: 1
    },
    BaseTime {
    seconds: - 3,
    femtoseconds: FEMTOSECONDS_PER_SECOND - 1
    }
    )]
    #[case::transition_from_positive_to_negative_time(
    TimeDelta {
    seconds: 1,
    femtoseconds: 2
    },
    BaseTime {
    seconds: 0,
    femtoseconds: 1
    },
    BaseTime {
    seconds: - 2,
    femtoseconds: FEMTOSECONDS_PER_SECOND - 1
    }
    )]
    fn test_base_time_sub_time_delta(
        #[case] delta: TimeDelta,
        #[case] time: BaseTime,
        #[case] expected: BaseTime,
    ) {
        let actual = time - delta;
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::at_the_epoch(BaseTime::default(), 0.0)]
    #[case::exactly_one_day_after_the_epoch(
    BaseTime {
    seconds: SECONDS_PER_DAY,
    femtoseconds: 0
    },
    1.0
    )]
    #[case::exactly_one_day_before_the_epoch(
    BaseTime {
    seconds: - SECONDS_PER_DAY,
    femtoseconds: 0
    },
    - 1.0
    )]
    #[case::a_partial_number_of_days_after_the_epoch(
    BaseTime {
    seconds: (SECONDS_PER_DAY / 2) * 3,
    femtoseconds: FEMTOSECONDS_PER_SECOND / 2
    },
    1.5000057870370371
    )]
    fn test_base_time_days_since_j2000(#[case] time: BaseTime, #[case] expected: f64) {
        let actual = time.days_since_j2000();
        assert_float_eq!(expected, actual, abs <= 1e-12);
    }

    #[rstest]
    #[case::at_the_epoch(BaseTime::default(), 0.0)]
    #[case::exactly_one_century_after_the_epoch(
    BaseTime {
    seconds: SECONDS_PER_JULIAN_CENTURY,
    femtoseconds: 0
    },
    1.0
    )]
    #[case::exactly_one_century_before_the_epoch(
    BaseTime {
    seconds: - SECONDS_PER_JULIAN_CENTURY,
    femtoseconds: 0
    },
    - 1.0
    )]
    #[case::a_partial_number_of_centuries_after_the_epoch(
    BaseTime {
    seconds: (SECONDS_PER_JULIAN_CENTURY / 2) * 3,
    femtoseconds: FEMTOSECONDS_PER_SECOND / 2
    },
    1.5000000001584404
    )]
    fn test_base_time_centuries_since_j2000(#[case] time: BaseTime, #[case] expected: f64) {
        let actual = time.centuries_since_j2000();
        assert_float_eq!(expected, actual, abs <= 1e-12,);
    }

    #[test]
    fn test_time_new() {
        let scale = TAI;
        let seconds = 1234567890;
        let femtoseconds = 9876543210;
        let expected = Time {
            scale,
            timestamp: BaseTime {
                seconds,
                subsecond: femtoseconds,
            },
        };
        let actual = Time::new(scale, seconds, femtoseconds);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_from_date_and_utc_timestamp() {
        let date = Date::new_unchecked(Gregorian, 2021, 1, 1);
        let utc = UTC::new(12, 34, 56).expect("time should be valid");
        let datetime = UTCDateTime::new(date, utc);
        let actual = Time::from_date_and_utc_timestamp(TAI, date, utc);
        let expected = Time::from_utc_datetime(TAI, datetime);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_display() {
        let time = Time::j2000(TAI);
        let expected = "12:00:00.000.000.000.000.000.000 TAI".to_string();
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
                subsecond: 0,
            },
        );
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_seconds() {
        let time = Time::new(TAI, 1234567890, 9876543210);
        let expected = 1234567890;
        let actual = time.seconds();
        assert_eq!(
            expected, actual,
            "expected Time to have {} seconds, but got {}",
            expected, actual
        );
    }

    #[test]
    fn test_time_femtoseconds() {
        let time = Time::new(TAI, 1234567890, 9876543210);
        let expected = 9876543210;
        let actual = time.femtoseconds();
        assert_eq!(
            expected, actual,
            "expected Time to have {} femtoseconds, but got {}",
            expected, actual
        );
    }

    #[test]
    fn test_time_days_since_j2000() {
        let base_time = BaseTime {
            seconds: 1234567890,
            subsecond: 9876543210,
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
            subsecond: 9876543210,
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
            subsecond: 9876543210,
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
            subsecond: 9876543210,
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
            subsecond: 9876543210,
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
            subsecond: 9876543210,
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
            subsecond: 9876543210,
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
            subsecond: 9876543210,
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
            subsecond: 9876543210,
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
            subsecond: 9876543210,
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
        let utc = UTC::new(12, 0, 0).expect("should be valid");
        let time = Time::from_date_and_utc_timestamp(TDB, date, utc);
        assert_eq!(time.julian_date(Epoch::J2000, Unit::Days), 36525.0);
        assert_eq!(time.seconds_since_j2000(), 3155760000.0);
        assert_eq!(time.days_since_j2000(), 36525.0);
        assert_eq!(time.centuries_since_j2000(), 1.0);
    }

    #[test]
    fn test_two_part_julian_date() {
        let date = Date::new_unchecked(Gregorian, 2100, 1, 2);
        let utc = UTC::new(0, 0, 0).expect("should be valid");
        let time = Time::from_date_and_utc_timestamp(TDB, date, utc);
        let (jd1, jd2) = time.two_part_julian_date();
        assert_eq!(jd1, 2451545.0 + 36525.0);
        assert_eq!(jd2, 0.5);
    }
}
