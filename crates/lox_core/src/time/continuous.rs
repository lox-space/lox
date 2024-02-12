/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module continuous provides representations and functions for working with time scales without leap seconds.
//!
//! Continuous times are represented with attosecond precision.
//!
//! The supported timescales are specified by [TimeScale].

mod constants;
pub mod transform;

use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};

use num::abs;

use crate::time::constants::f64::DAYS_PER_JULIAN_CENTURY;
use crate::time::constants::i64::{
    SECONDS_PER_DAY, SECONDS_PER_HALF_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE,
};
use crate::time::constants::u64::{
    ATTOSECONDS_PER_FEMTOSECOND, ATTOSECONDS_PER_MICROSECOND, ATTOSECONDS_PER_MILLISECOND,
    ATTOSECONDS_PER_NANOSECOND, ATTOSECONDS_PER_PICOSECOND, ATTOSECONDS_PER_SECOND,
};
use crate::time::dates::Calendar::ProlepticJulian;
use crate::time::dates::Date;
use crate::time::utc::{UTCDateTime, UTC};
use crate::time::{constants, WallClock};

/// An absolute continuous time difference with attosecond precision.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TimeDelta {
    seconds: u64,
    attoseconds: u64,
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
/// `UnscaledTime` is the base time representation for time scales without leap seconds. It is measured relative to
/// J2000. `UnscaledTime::default()` represents the epoch itself.
///
/// `UnscaledTime` has attosecond precision, and supports times within 292 billion years either side of the epoch.
pub struct BaseTime {
    // The sign of the time is determined exclusively by the sign of the `second` field. `attoseconds` is always the
    // positive count of attoseconds since the last whole second. For example, one attosecond before the epoch is
    // represented as
    // ```
    // let time = UnscaledTime {
    //     seconds: -1,
    //     attoseconds: ATTOSECONDS_PER_SECOND - 1,
    // };
    // ```
    seconds: i64,
    attoseconds: u64,
}

impl BaseTime {
    fn is_negative(&self) -> bool {
        self.seconds < 0
    }

    pub fn seconds(&self) -> i64 {
        self.seconds
    }

    pub fn attoseconds(&self) -> u64 {
        self.attoseconds
    }

    /// The fractional number of Julian days since J2000.
    pub fn days_since_j2000(&self) -> f64 {
        let d1 = self.seconds as f64 / constants::f64::SECONDS_PER_DAY;
        let d2 = self.attoseconds as f64 / constants::f64::ATTOSECONDS_PER_DAY;
        d2 + d1
    }

    /// The fractional number of Julian centuries since J2000.
    pub fn centuries_since_j2000(&self) -> f64 {
        self.days_since_j2000() / DAYS_PER_JULIAN_CENTURY
    }
}

impl Display for BaseTime {
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
            self.attosecond(),
        )
    }
}

impl Add<TimeDelta> for BaseTime {
    type Output = Self;

    /// The implementation of [Add] for [BaseTime] follows the default Rust rules for integer overflow, which
    /// should be sufficient for all practical purposes.
    fn add(self, rhs: TimeDelta) -> Self::Output {
        let mut attoseconds = self.attoseconds + rhs.attoseconds;
        let mut seconds = self.seconds + rhs.seconds as i64;
        if attoseconds >= ATTOSECONDS_PER_SECOND {
            seconds += 1;
            attoseconds -= ATTOSECONDS_PER_SECOND;
        }
        Self {
            seconds,
            attoseconds,
        }
    }
}

impl Sub<TimeDelta> for BaseTime {
    type Output = Self;

    /// The implementation of [Sub] for [BaseTime] follows the default Rust rules for integer overflow, which
    /// should be sufficient for all practical purposes.
    fn sub(self, rhs: TimeDelta) -> Self::Output {
        let mut seconds = self.seconds - rhs.seconds as i64;
        let mut attoseconds = self.attoseconds;
        if rhs.attoseconds > self.attoseconds {
            seconds -= 1;
            attoseconds = ATTOSECONDS_PER_SECOND - (rhs.attoseconds - self.attoseconds);
        } else {
            attoseconds -= rhs.attoseconds;
        }
        Self {
            seconds,
            attoseconds,
        }
    }
}

impl WallClock for BaseTime {
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
        (self.attoseconds / ATTOSECONDS_PER_MILLISECOND) as i64
    }

    fn microsecond(&self) -> i64 {
        (self.attoseconds / ATTOSECONDS_PER_MICROSECOND % 1000) as i64
    }

    fn nanosecond(&self) -> i64 {
        (self.attoseconds / ATTOSECONDS_PER_NANOSECOND % 1000) as i64
    }

    fn picosecond(&self) -> i64 {
        (self.attoseconds / ATTOSECONDS_PER_PICOSECOND % 1000) as i64
    }

    fn femtosecond(&self) -> i64 {
        (self.attoseconds / ATTOSECONDS_PER_FEMTOSECOND % 1000) as i64
    }

    fn attosecond(&self) -> i64 {
        (self.attoseconds % 1000) as i64
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
pub struct Time<T: TimeScale + Copy> {
    scale: T,
    timestamp: BaseTime,
}

impl<T: TimeScale + Copy> Time<T> {
    /// Instantiates a [Time] in the given scale from seconds and attoseconds since the epoch.
    pub fn new(scale: T, seconds: i64, attoseconds: u64) -> Self {
        Self {
            scale,
            timestamp: BaseTime {
                seconds,
                attoseconds,
            },
        }
    }

    /// Instantiates a [Time] in the given scale from a [BaseTime].
    pub fn from_unscaled(scale: T, timestamp: BaseTime) -> Self {
        Self { scale, timestamp }
    }

    /// Instantiates a [Time] in the given scale from a date and UTC timestamp.
    pub fn from_date_and_utc_timestamp(scale: T, date: Date, time: UTC) -> Self {
        let day_in_seconds = date.j2000() * SECONDS_PER_DAY - SECONDS_PER_DAY / 2;
        let hour_in_seconds = time.hour() * SECONDS_PER_HOUR;
        let minute_in_seconds = time.minute() * SECONDS_PER_MINUTE;
        let seconds = day_in_seconds + hour_in_seconds + minute_in_seconds + time.second();
        let attoseconds = time.subsecond_as_attoseconds();
        let unscaled = BaseTime {
            seconds,
            attoseconds,
        };
        Self::from_unscaled(scale, unscaled)
    }

    /// Instantiates a [Time] in the given scale from a UTC datetime.
    pub fn from_utc_datetime(scale: T, dt: UTCDateTime) -> Self {
        Self::from_date_and_utc_timestamp(scale, dt.date(), dt.time())
    }

    /// Returns the J2000 epoch in the given timescale.
    pub fn j2000(scale: T) -> Self {
        Self {
            scale,
            timestamp: BaseTime::default(),
        }
    }

    /// Returns, as an epoch in the given timescale, midday on the first day of the proleptic Julian
    /// calendar.
    pub fn jd0(scale: T) -> Self {
        // This represents 4713 BC, since there is no year 0 separating BC and AD.
        let first_proleptic_day = Date::new_unchecked(ProlepticJulian, -4712, 1, 1);
        let midday = UTC::new(12, 0, 0).expect("midday should be a valid time");
        Self::from_date_and_utc_timestamp(scale, first_proleptic_day, midday)
    }

    /// The underlying unscaled timestamp.
    pub fn unscaled(&self) -> BaseTime {
        self.timestamp
    }

    /// The number of whole seconds since J2000.
    pub fn seconds(&self) -> i64 {
        self.timestamp.seconds
    }

    /// The number of attoseconds from the last whole second.
    pub fn attoseconds(&self) -> u64 {
        self.timestamp.attoseconds
    }

    /// The fractional number of Julian days since J2000.
    pub fn days_since_j2000(&self) -> f64 {
        self.timestamp.days_since_j2000()
    }

    /// The fractional number of Julian centuries since J2000.
    pub fn centuries_since_j2000(&self) -> f64 {
        self.timestamp.centuries_since_j2000()
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
        Self::from_unscaled(self.scale, self.timestamp + rhs)
    }
}

impl<T: TimeScale + Copy> Sub<TimeDelta> for Time<T> {
    type Output = Self;

    fn sub(self, rhs: TimeDelta) -> Self::Output {
        Self::from_unscaled(self.scale, self.timestamp - rhs)
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

    fn attosecond(&self) -> i64 {
        self.timestamp.attosecond()
    }
}

/// CalendarDate allows continuous time formats to report their date in their respective calendar.
pub trait CalendarDate {
    fn date(&self) -> Date;
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use rstest::rstest;

    use crate::time::constants::i64::SECONDS_PER_JULIAN_CENTURY;
    use crate::time::dates::Calendar::Gregorian;

    use super::*;

    #[test]
    fn test_base_time_is_negative() {
        assert!(BaseTime {
            seconds: -1,
            attoseconds: 0,
        }
        .is_negative());
        assert!(!BaseTime {
            seconds: 0,
            attoseconds: 0,
        }
        .is_negative());
        assert!(!BaseTime {
            seconds: 1,
            attoseconds: 0,
        }
        .is_negative());
    }

    #[test]
    fn test_base_time_seconds() {
        let time = BaseTime {
            seconds: 123,
            attoseconds: 0,
        };
        assert_eq!(time.seconds(), 123);
    }

    #[test]
    fn test_base_time_attoseconds() {
        let time = BaseTime {
            seconds: 0,
            attoseconds: 123,
        };
        assert_eq!(time.attoseconds(), 123);
    }

    #[rstest]
    #[case::zero_value(BaseTime { seconds: 0, attoseconds: 0 }, 12)]
    #[case::one_attosecond_less_than_an_hour(BaseTime { seconds: SECONDS_PER_HOUR - 1, attoseconds: ATTOSECONDS_PER_SECOND - 1 }, 12)]
    #[case::exactly_one_hour(BaseTime { seconds: SECONDS_PER_HOUR, attoseconds: 0 }, 13)]
    #[case::one_day_and_one_hour(BaseTime { seconds: SECONDS_PER_HOUR * 25, attoseconds: 0 }, 13)]
    #[case::one_attosecond_less_than_the_epoch(BaseTime { seconds: -1, attoseconds: ATTOSECONDS_PER_SECOND - 1 }, 11)]
    #[case::one_hour_less_than_the_epoch(BaseTime { seconds: -SECONDS_PER_HOUR, attoseconds: 0 }, 11)]
    #[case::one_hour_and_one_attosecond_less_than_the_epoch(BaseTime { seconds: -SECONDS_PER_HOUR - 1, attoseconds: ATTOSECONDS_PER_SECOND - 1 }, 10)]
    #[case::one_day_less_than_the_epoch(BaseTime { seconds: -SECONDS_PER_DAY, attoseconds: 0 }, 12)]
    #[case::two_days_less_than_the_epoch(BaseTime { seconds: -SECONDS_PER_DAY * 2, attoseconds: 0 }, 12)]
    fn test_base_time_wall_clock_hour(#[case] time: BaseTime, #[case] expected: i64) {
        let actual = time.hour();
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_value(BaseTime { seconds: 0, attoseconds: 0 }, 0)]
    #[case::one_attosecond_less_than_one_minute(BaseTime { seconds: SECONDS_PER_MINUTE - 1, attoseconds: ATTOSECONDS_PER_SECOND - 1 }, 0)]
    #[case::one_minute(BaseTime { seconds: SECONDS_PER_MINUTE, attoseconds: 0 }, 1)]
    #[case::one_attosecond_less_than_an_hour(BaseTime { seconds: SECONDS_PER_HOUR - 1, attoseconds: ATTOSECONDS_PER_SECOND - 1 }, 59)]
    #[case::exactly_one_hour(BaseTime { seconds: SECONDS_PER_HOUR, attoseconds: 0 }, 0)]
    #[case::one_hour_and_one_minute(BaseTime { seconds: SECONDS_PER_HOUR + SECONDS_PER_MINUTE, attoseconds: 0 }, 1)]
    #[case::one_attosecond_less_than_the_epoch(BaseTime { seconds: -1, attoseconds: ATTOSECONDS_PER_SECOND - 1 }, 59)]
    #[case::one_minute_less_than_the_epoch(BaseTime { seconds: -SECONDS_PER_MINUTE, attoseconds: 0 }, 59)]
    #[case::one_minute_and_one_attosecond_less_than_the_epoch(BaseTime { seconds: -SECONDS_PER_MINUTE - 1, attoseconds: ATTOSECONDS_PER_SECOND - 1 }, 58)]
    fn test_base_time_wall_clock_minute(#[case] time: BaseTime, #[case] expected: i64) {
        let actual = time.minute();
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_value(BaseTime { seconds: 0, attoseconds: 0 }, 0)]
    #[case::one_attosecond_less_than_one_second(BaseTime { seconds: 0, attoseconds: ATTOSECONDS_PER_SECOND - 1 }, 0)]
    #[case::one_second(BaseTime { seconds: 1, attoseconds: 0 }, 1)]
    #[case::one_attosecond_less_than_a_minute(BaseTime { seconds: SECONDS_PER_MINUTE - 1, attoseconds: ATTOSECONDS_PER_SECOND - 1 }, 59)]
    #[case::exactly_one_minute(BaseTime { seconds: SECONDS_PER_MINUTE, attoseconds: 0 }, 0)]
    #[case::one_minute_and_one_second(BaseTime { seconds: SECONDS_PER_MINUTE + 1, attoseconds: 0 }, 1)]
    #[case::one_attosecond_less_than_the_epoch(BaseTime { seconds: -1, attoseconds: ATTOSECONDS_PER_SECOND - 1 }, 59)]
    #[case::one_second_less_than_the_epoch(BaseTime { seconds: -1, attoseconds: 0 }, 59)]
    #[case::one_second_and_one_attosecond_less_than_the_epoch(BaseTime { seconds: -2, attoseconds: ATTOSECONDS_PER_SECOND - 1 }, 58)]
    fn test_base_time_wall_clock_second(#[case] time: BaseTime, #[case] expected: i64) {
        let actual = time.second();
        assert_eq!(expected, actual);
    }

    const POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE: BaseTime = BaseTime {
        seconds: 0,
        attoseconds: 123_456_789_012_345_678,
    };

    const NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE: BaseTime = BaseTime {
        seconds: -1,
        attoseconds: 123_456_789_012_345_678,
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
    #[case::positive_time_attosecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::attosecond,
        678
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
    #[case::negative_time_attosecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::attosecond,
        678
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
    #[case::positive_time_no_attosecond_wrap(
        TimeDelta {
            seconds: 1,
            attoseconds: 1
        },
        BaseTime {
            seconds: 1,
            attoseconds: 0
        },
        BaseTime {
            seconds: 2,
            attoseconds: 1
        }
    )]
    #[case::positive_time_attosecond_wrap(
        TimeDelta {
            seconds: 1,
            attoseconds: 2
        },
        BaseTime {
            seconds: 1,
            attoseconds: ATTOSECONDS_PER_SECOND - 1
        },
        BaseTime {
            seconds: 3,
            attoseconds: 1
        }
    )]
    #[case::negative_time_no_attosecond_wrap(
        TimeDelta {
            seconds: 1,
            attoseconds: 1
        },
        BaseTime {
            seconds: -1,
            attoseconds: 0
        },
        BaseTime {
            seconds: 0,
            attoseconds: 1
        }
    )]
    #[case::negative_time_attosecond_wrap(
        TimeDelta {
            seconds: 1,
            attoseconds: 2
        },
        BaseTime {
            seconds: -1,
            attoseconds: ATTOSECONDS_PER_SECOND - 1
        },
        BaseTime {
            seconds: 1,
            attoseconds: 1
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
    #[case::positive_time_no_attosecond_wrap(
        TimeDelta {
            seconds: 1,
            attoseconds: 1
        },
        BaseTime {
            seconds: 2,
            attoseconds: 2
        },
        BaseTime {
            seconds: 1,
            attoseconds: 1
        }
    )]
    #[case::positive_time_attosecond_wrap(
        TimeDelta {
            seconds: 1,
            attoseconds: 2
        },
        BaseTime {
            seconds: 2,
            attoseconds: 1
        },
        BaseTime {
            seconds: 0,
            attoseconds: ATTOSECONDS_PER_SECOND - 1
        }
    )]
    #[case::negative_time_no_attosecond_wrap(
        TimeDelta {
            seconds: 1,
            attoseconds: 1
        },
        BaseTime {
            seconds: -1,
            attoseconds: 2
        },
        BaseTime {
            seconds: -2,
            attoseconds: 1
        }
    )]
    #[case::negative_time_attosecond_wrap(
        TimeDelta {
            seconds: 1,
            attoseconds: 2
        },
        BaseTime {
            seconds: -1,
            attoseconds: 1
        },
        BaseTime {
            seconds: -3,
            attoseconds: ATTOSECONDS_PER_SECOND - 1
        }
    )]
    #[case::transition_from_positive_to_negative_time(
        TimeDelta {
            seconds: 1,
            attoseconds: 2
        },
        BaseTime {
            seconds: 0,
            attoseconds: 1
        },
        BaseTime {
            seconds: -2,
            attoseconds: ATTOSECONDS_PER_SECOND - 1
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
            attoseconds: 0
        },
        1.0
    )]
    #[case::exactly_one_day_before_the_epoch(
        BaseTime {
            seconds: -SECONDS_PER_DAY,
            attoseconds: 0
        },
        -1.0
    )]
    #[case::a_partial_number_of_days_after_the_epoch(
        BaseTime {
            seconds: (SECONDS_PER_DAY / 2) * 3,
            attoseconds: ATTOSECONDS_PER_SECOND / 2
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
            attoseconds: 0
        },
        1.0
    )]
    #[case::exactly_one_century_before_the_epoch(
        BaseTime {
            seconds: -SECONDS_PER_JULIAN_CENTURY,
            attoseconds: 0
        },
        -1.0
    )]
    #[case::a_partial_number_of_centuries_after_the_epoch(
        BaseTime {
            seconds: (SECONDS_PER_JULIAN_CENTURY / 2) * 3,
            attoseconds: ATTOSECONDS_PER_SECOND / 2
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
        let attoseconds = 9876543210;
        let expected = Time {
            scale,
            timestamp: BaseTime {
                seconds,
                attoseconds,
            },
        };
        let actual = Time::new(scale, seconds, attoseconds);
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
        let expected = Time::from_unscaled(
            TAI,
            BaseTime {
                seconds: -211813488000,
                attoseconds: 0,
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
    fn test_time_attoseconds() {
        let time = Time::new(TAI, 1234567890, 9876543210);
        let expected = 9876543210;
        let actual = time.attoseconds();
        assert_eq!(
            expected, actual,
            "expected Time to have {} attoseconds, but got {}",
            expected, actual
        );
    }

    #[test]
    fn test_time_days_since_j2000() {
        let unscaled = BaseTime {
            seconds: 1234567890,
            attoseconds: 9876543210,
        };
        let expected = unscaled.days_since_j2000();
        let actual = Time::from_unscaled(TAI, unscaled).days_since_j2000();
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
        let unscaled = BaseTime {
            seconds: 1234567890,
            attoseconds: 9876543210,
        };
        let expected = unscaled.centuries_since_j2000();
        let actual = Time::from_unscaled(TAI, unscaled).centuries_since_j2000();
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
            attoseconds: 9876543210,
        };
        let expected = base_time.hour();
        let actual = Time::from_unscaled(TAI, base_time).hour();
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
            attoseconds: 9876543210,
        };
        let expected = base_time.minute();
        let actual = Time::from_unscaled(TAI, base_time).minute();
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
            attoseconds: 9876543210,
        };
        let expected = base_time.second();
        let actual = Time::from_unscaled(TAI, base_time).second();
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
            attoseconds: 9876543210,
        };
        let expected = base_time.millisecond();
        let actual = Time::from_unscaled(TAI, base_time).millisecond();
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
            attoseconds: 9876543210,
        };
        let expected = base_time.microsecond();
        let actual = Time::from_unscaled(TAI, base_time).microsecond();
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
            attoseconds: 9876543210,
        };
        let expected = base_time.nanosecond();
        let actual = Time::from_unscaled(TAI, base_time).nanosecond();
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
            attoseconds: 9876543210,
        };
        let expected = base_time.picosecond();
        let actual = Time::from_unscaled(TAI, base_time).picosecond();
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
            attoseconds: 9876543210,
        };
        let expected = base_time.femtosecond();
        let actual = Time::from_unscaled(TAI, base_time).femtosecond();
        assert_eq!(
            expected, actual,
            "expected Time to have femtosecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_wall_clock_attosecond() {
        let base_time = BaseTime {
            seconds: 1234567890,
            attoseconds: 9876543210,
        };
        let expected = base_time.attosecond();
        let actual = Time::from_unscaled(TAI, base_time).attosecond();
        assert_eq!(
            expected, actual,
            "expected Time to have attosecond {}, but got {}",
            expected, actual,
        );
    }
}
