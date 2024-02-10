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
pub struct UnscaledTime {
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

impl UnscaledTime {
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

impl Display for UnscaledTime {
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

impl Add<TimeDelta> for UnscaledTime {
    type Output = Self;

    /// The implementation of [Add] for [UnscaledTime] follows the default Rust rules for integer overflow, which
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

impl Sub<TimeDelta> for UnscaledTime {
    type Output = Self;

    /// The implementation of [Sub] for [UnscaledTime] follows the default Rust rules for integer overflow, which
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

impl WallClock for UnscaledTime {
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
    timestamp: UnscaledTime,
}

impl<T: TimeScale + Copy> Time<T> {
    /// Instantiates a [Time] in the given scale from seconds and attoseconds since the epoch.
    pub fn new(scale: T, seconds: i64, attoseconds: u64) -> Self {
        Self {
            scale,
            timestamp: UnscaledTime {
                seconds,
                attoseconds,
            },
        }
    }

    /// Instantiates a [Time] in the given scale from an [UnscaledTime].
    pub fn from_unscaled(scale: T, timestamp: UnscaledTime) -> Self {
        Self { scale, timestamp }
    }

    /// Instantiates a [Time] in the given scale from a date and UTC timestamp.
    pub fn from_date_and_utc_timestamp(scale: T, date: Date, time: UTC) -> Self {
        let day_in_seconds = date.j2000() * SECONDS_PER_DAY - SECONDS_PER_DAY / 2;
        let hour_in_seconds = time.hour() * SECONDS_PER_HOUR;
        let minute_in_seconds = time.minute() * SECONDS_PER_MINUTE;
        let seconds = day_in_seconds + hour_in_seconds + minute_in_seconds + time.second();
        let attoseconds = time.subsecond_as_attoseconds();
        let unscaled = UnscaledTime {
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
            timestamp: UnscaledTime::default(),
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
    pub fn unscaled(&self) -> UnscaledTime {
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
    use crate::time::constants::i64::SECONDS_PER_JULIAN_CENTURY;
    use crate::time::dates::Calendar::Gregorian;
    use float_eq::assert_float_eq;

    use super::*;

    #[test]
    fn test_unscaled_time_is_negative() {
        assert!(UnscaledTime {
            seconds: -1,
            attoseconds: 0
        }
        .is_negative());
        assert!(!UnscaledTime {
            seconds: 0,
            attoseconds: 0
        }
        .is_negative());
        assert!(!UnscaledTime {
            seconds: 1,
            attoseconds: 0
        }
        .is_negative());
    }

    #[test]
    fn test_unscaled_time_hour() {
        struct TestCase {
            desc: &'static str,
            time: UnscaledTime,
            expected_hour: i64,
        }

        let test_cases = [
            TestCase {
                desc: "zero value",
                time: UnscaledTime {
                    seconds: 0,
                    attoseconds: 0,
                },
                expected_hour: 12,
            },
            TestCase {
                desc: "one attosecond less than an hour",
                time: UnscaledTime {
                    seconds: SECONDS_PER_HOUR - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_hour: 12,
            },
            TestCase {
                desc: "exactly one hour",
                time: UnscaledTime {
                    seconds: SECONDS_PER_HOUR,
                    attoseconds: 0,
                },
                expected_hour: 13,
            },
            TestCase {
                desc: "one day and one hour",
                time: UnscaledTime {
                    seconds: SECONDS_PER_HOUR * 25,
                    attoseconds: 0,
                },
                expected_hour: 13,
            },
            TestCase {
                desc: "one attosecond less than the epoch",
                time: UnscaledTime {
                    seconds: -1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_hour: 11,
            },
            TestCase {
                desc: "one hour less than the epoch",
                time: UnscaledTime {
                    seconds: -SECONDS_PER_HOUR,
                    attoseconds: 0,
                },
                expected_hour: 11,
            },
            TestCase {
                desc: "one hour and one attosecond less than the epoch",
                time: UnscaledTime {
                    seconds: -SECONDS_PER_HOUR - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_hour: 10,
            },
            TestCase {
                desc: "one day less than the epoch",
                time: UnscaledTime {
                    seconds: -SECONDS_PER_DAY,
                    attoseconds: 0,
                },
                expected_hour: 12,
            },
            TestCase {
                // Exercises the case where the number of seconds exceeds the number of seconds in a day.
                desc: "two days less than the epoch",
                time: UnscaledTime {
                    seconds: -SECONDS_PER_DAY * 2,
                    attoseconds: 0,
                },
                expected_hour: 12,
            },
        ];

        for tc in test_cases {
            let actual = tc.time.hour();
            assert_eq!(
                actual, tc.expected_hour,
                "{}: expected {}, got {}",
                tc.desc, tc.expected_hour, actual
            );
        }
    }

    #[test]
    fn test_unscaled_time_minute() {
        struct TestCase {
            desc: &'static str,
            time: UnscaledTime,
            expected_minute: i64,
        }

        let test_cases = [
            TestCase {
                desc: "zero value",
                time: UnscaledTime {
                    seconds: 0,
                    attoseconds: 0,
                },
                expected_minute: 0,
            },
            TestCase {
                desc: "one attosecond less than one minute",
                time: UnscaledTime {
                    seconds: SECONDS_PER_MINUTE - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_minute: 0,
            },
            TestCase {
                desc: "one minute",
                time: UnscaledTime {
                    seconds: SECONDS_PER_MINUTE,
                    attoseconds: 0,
                },
                expected_minute: 1,
            },
            TestCase {
                desc: "one attosecond less than an hour",
                time: UnscaledTime {
                    seconds: SECONDS_PER_HOUR - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_minute: 59,
            },
            TestCase {
                desc: "exactly one hour",
                time: UnscaledTime {
                    seconds: SECONDS_PER_HOUR,
                    attoseconds: 0,
                },
                expected_minute: 0,
            },
            TestCase {
                desc: "one hour and one minute",
                time: UnscaledTime {
                    seconds: SECONDS_PER_HOUR + SECONDS_PER_MINUTE,
                    attoseconds: 0,
                },
                expected_minute: 1,
            },
            TestCase {
                desc: "one attosecond less than the epoch",
                time: UnscaledTime {
                    seconds: -1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_minute: 59,
            },
            TestCase {
                desc: "one minute less than the epoch",
                time: UnscaledTime {
                    seconds: -SECONDS_PER_MINUTE,
                    attoseconds: 0,
                },
                expected_minute: 59,
            },
            TestCase {
                desc: "one minute and one attosecond less than the epoch",
                time: UnscaledTime {
                    seconds: -SECONDS_PER_MINUTE - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_minute: 58,
            },
        ];

        for tc in test_cases {
            let actual = tc.time.minute();
            assert_eq!(
                actual, tc.expected_minute,
                "{}: expected {}, got {}",
                tc.desc, tc.expected_minute, actual
            );
        }
    }

    #[test]
    fn test_unscaled_time_second() {
        struct TestCase {
            desc: &'static str,
            time: UnscaledTime,
            expected_second: i64,
        }

        let test_cases = [
            TestCase {
                desc: "zero value",
                time: UnscaledTime {
                    seconds: 0,
                    attoseconds: 0,
                },
                expected_second: 0,
            },
            TestCase {
                desc: "one attosecond less than one second",
                time: UnscaledTime {
                    seconds: 0,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_second: 0,
            },
            TestCase {
                desc: "one second",
                time: UnscaledTime {
                    seconds: 1,
                    attoseconds: 0,
                },
                expected_second: 1,
            },
            TestCase {
                desc: "one attosecond less than a minute",
                time: UnscaledTime {
                    seconds: SECONDS_PER_MINUTE - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_second: 59,
            },
            TestCase {
                desc: "exactly one minute",
                time: UnscaledTime {
                    seconds: SECONDS_PER_MINUTE,
                    attoseconds: 0,
                },
                expected_second: 0,
            },
            TestCase {
                desc: "one minute and one second",
                time: UnscaledTime {
                    seconds: SECONDS_PER_MINUTE + 1,
                    attoseconds: 0,
                },
                expected_second: 1,
            },
            TestCase {
                desc: "one attosecond less than the epoch",
                time: UnscaledTime {
                    seconds: -1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_second: 59,
            },
            TestCase {
                desc: "one second less than the epoch",
                time: UnscaledTime {
                    seconds: -1,
                    attoseconds: 0,
                },
                expected_second: 59,
            },
            TestCase {
                desc: "one second and one attosecond less than the epoch",
                time: UnscaledTime {
                    seconds: -2,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_second: 58,
            },
        ];

        for tc in test_cases {
            let actual = tc.time.second();
            assert_eq!(
                actual, tc.expected_second,
                "{}: expected {}, got {}",
                tc.desc, tc.expected_second, actual
            );
        }
    }

    #[test]
    fn test_unscaled_time_subseconds_with_positive_seconds() {
        let time = UnscaledTime {
            seconds: 0,
            attoseconds: 123_456_789_012_345_678,
        };

        struct TestCase {
            unit: &'static str,
            expected: i64,
            actual: i64,
        }

        let test_cases = [
            TestCase {
                unit: "millisecond",
                expected: 123,
                actual: time.millisecond(),
            },
            TestCase {
                unit: "microsecond",
                expected: 456,
                actual: time.microsecond(),
            },
            TestCase {
                unit: "nanosecond",
                expected: 789,
                actual: time.nanosecond(),
            },
            TestCase {
                unit: "picosecond",
                expected: 12,
                actual: time.picosecond(),
            },
            TestCase {
                unit: "femtosecond",
                expected: 345,
                actual: time.femtosecond(),
            },
            TestCase {
                unit: "attosecond",
                expected: 678,
                actual: time.attosecond(),
            },
        ];

        for tc in test_cases {
            assert_eq!(
                tc.actual, tc.expected,
                "expected {} {}, got {}",
                tc.unit, tc.expected, tc.actual
            );
        }
    }

    #[test]
    fn test_unscaled_time_subseconds_with_negative_seconds() {
        let time = UnscaledTime {
            seconds: -1,
            attoseconds: 123_456_789_012_345_678,
        };

        struct TestCase {
            unit: &'static str,
            expected: i64,
            actual: i64,
        }

        let test_cases = [
            TestCase {
                unit: "millisecond",
                expected: 123,
                actual: time.millisecond(),
            },
            TestCase {
                unit: "microsecond",
                expected: 456,
                actual: time.microsecond(),
            },
            TestCase {
                unit: "nanosecond",
                expected: 789,
                actual: time.nanosecond(),
            },
            TestCase {
                unit: "picosecond",
                expected: 12,
                actual: time.picosecond(),
            },
            TestCase {
                unit: "femtosecond",
                expected: 345,
                actual: time.femtosecond(),
            },
            TestCase {
                unit: "attosecond",
                expected: 678,
                actual: time.attosecond(),
            },
        ];

        for tc in test_cases {
            assert_eq!(
                tc.actual, tc.expected,
                "expected {} {}, got {}",
                tc.unit, tc.expected, tc.actual
            );
        }
    }

    #[test]
    fn test_unscaled_time_add_time_delta() {
        struct TestCase {
            desc: &'static str,
            delta: TimeDelta,
            time: UnscaledTime,
            expected: UnscaledTime,
        }

        let test_cases = [
            TestCase {
                desc: "positive time with no attosecond wrap",
                delta: TimeDelta {
                    seconds: 1,
                    attoseconds: 1,
                },
                time: UnscaledTime {
                    seconds: 1,
                    attoseconds: 0,
                },
                expected: UnscaledTime {
                    seconds: 2,
                    attoseconds: 1,
                },
            },
            TestCase {
                desc: "positive time with attosecond wrap",
                delta: TimeDelta {
                    seconds: 1,
                    attoseconds: 2,
                },
                time: UnscaledTime {
                    seconds: 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected: UnscaledTime {
                    seconds: 3,
                    attoseconds: 1,
                },
            },
            TestCase {
                desc: "negative time with no attosecond wrap",
                delta: TimeDelta {
                    seconds: 1,
                    attoseconds: 1,
                },
                time: UnscaledTime {
                    seconds: -1,
                    attoseconds: 0,
                },
                expected: UnscaledTime {
                    seconds: 0,
                    attoseconds: 1,
                },
            },
            TestCase {
                desc: "negative time with attosecond wrap",
                delta: TimeDelta {
                    seconds: 1,
                    attoseconds: 2,
                },
                time: UnscaledTime {
                    seconds: -1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected: UnscaledTime {
                    seconds: 1,
                    attoseconds: 1,
                },
            },
        ];

        for tc in test_cases {
            let actual = tc.time + tc.delta;
            assert_eq!(
                actual, tc.expected,
                "{}: expected {:?}, got {:?}",
                tc.desc, tc.expected, actual
            );
        }
    }

    #[test]
    fn test_unscaled_time_sub_time_delta() {
        struct TestCase {
            desc: &'static str,
            delta: TimeDelta,
            time: UnscaledTime,
            expected: UnscaledTime,
        }

        let test_cases = [
            TestCase {
                desc: "positive time with no attosecond wrap",
                delta: TimeDelta {
                    seconds: 1,
                    attoseconds: 1,
                },
                time: UnscaledTime {
                    seconds: 2,
                    attoseconds: 2,
                },
                expected: UnscaledTime {
                    seconds: 1,
                    attoseconds: 1,
                },
            },
            TestCase {
                desc: "positive time with attosecond wrap",
                delta: TimeDelta {
                    seconds: 1,
                    attoseconds: 2,
                },
                time: UnscaledTime {
                    seconds: 2,
                    attoseconds: 1,
                },
                expected: UnscaledTime {
                    seconds: 0,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
            },
            TestCase {
                desc: "negative time with no attosecond wrap",
                delta: TimeDelta {
                    seconds: 1,
                    attoseconds: 1,
                },
                time: UnscaledTime {
                    seconds: -1,
                    attoseconds: 2,
                },
                expected: UnscaledTime {
                    seconds: -2,
                    attoseconds: 1,
                },
            },
            TestCase {
                desc: "negative time with attosecond wrap",
                delta: TimeDelta {
                    seconds: 1,
                    attoseconds: 2,
                },
                time: UnscaledTime {
                    seconds: -1,
                    attoseconds: 1,
                },
                expected: UnscaledTime {
                    seconds: -3,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
            },
            TestCase {
                desc: "transition from positive to negative time",
                delta: TimeDelta {
                    seconds: 1,
                    attoseconds: 2,
                },
                time: UnscaledTime {
                    seconds: 0,
                    attoseconds: 1,
                },
                expected: UnscaledTime {
                    seconds: -2,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
            },
        ];

        for tc in test_cases {
            let actual = tc.time - tc.delta;
            assert_eq!(
                actual, tc.expected,
                "{}: expected {:?}, got {:?}",
                tc.desc, tc.expected, actual
            );
        }
    }

    #[test]
    fn test_unscaled_time_days_since_j2000() {
        struct TestCase {
            desc: &'static str,
            time: UnscaledTime,
            expected: f64,
        }

        let test_cases = [
            TestCase {
                desc: "at the epoch",
                time: UnscaledTime::default(),
                expected: 0.0,
            },
            TestCase {
                desc: "exactly one day after the epoch",
                time: UnscaledTime {
                    seconds: SECONDS_PER_DAY,
                    attoseconds: 0,
                },
                expected: 1.0,
            },
            TestCase {
                desc: "exactly one day before the epoch",
                time: UnscaledTime {
                    seconds: -SECONDS_PER_DAY,
                    attoseconds: 0,
                },
                expected: -1.0,
            },
            TestCase {
                desc: "a partial number of days after the epoch",
                time: UnscaledTime {
                    seconds: (SECONDS_PER_DAY / 2) * 3,
                    attoseconds: ATTOSECONDS_PER_SECOND / 2,
                },
                expected: 1.5000057870370371,
            },
        ];

        for tc in test_cases {
            let actual = tc.time.days_since_j2000();
            assert_float_eq!(
                tc.expected,
                actual,
                abs <= 1e-12,
                "{}: expected {}, got {}",
                tc.desc,
                tc.expected,
                actual
            );
        }
    }

    #[test]
    fn test_unscaled_time_centuries_since_j2000() {
        struct TestCase {
            desc: &'static str,
            time: UnscaledTime,
            expected: f64,
        }

        let test_cases = [
            TestCase {
                desc: "at the epoch",
                time: UnscaledTime::default(),
                expected: 0.0,
            },
            TestCase {
                desc: "exactly one century after the epoch",
                time: UnscaledTime {
                    seconds: SECONDS_PER_JULIAN_CENTURY,
                    attoseconds: 0,
                },
                expected: 1.0,
            },
            TestCase {
                desc: "exactly one century before the epoch",
                time: UnscaledTime {
                    seconds: -SECONDS_PER_JULIAN_CENTURY,
                    attoseconds: 0,
                },
                expected: -1.0,
            },
            TestCase {
                desc: "a partial number of centuries after the epoch",
                time: UnscaledTime {
                    seconds: (SECONDS_PER_JULIAN_CENTURY / 2) * 3,
                    attoseconds: ATTOSECONDS_PER_SECOND / 2,
                },
                expected: 1.5000000001584404,
            },
        ];

        for tc in test_cases {
            let actual = tc.time.centuries_since_j2000();
            assert_float_eq!(
                tc.expected,
                actual,
                abs <= 1e-12,
                "{}: expected {}, got {}",
                tc.desc,
                tc.expected,
                actual
            );
        }
    }

    #[test]
    fn test_time_from_date_and_utc_timestamp() {
        let date = Date::new_unchecked(Gregorian, 2021, 1, 1);
        let utc = UTC::new(12, 34, 56).expect("time should be valid");
        let datetime = UTCDateTime::new(date, utc);
        let actual = Time::from_date_and_utc_timestamp(TAI, date, utc);
        let expected = Time::from_utc_datetime(TAI, datetime);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_time_display() {
        let time = Time::j2000(TAI);
        let expected = "12:00:00.000.000.000.000.000.000 TAI".to_string();
        let actual = time.to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_time_j2000() {
        let actual = Time::j2000(TAI);
        let expected = Time {
            scale: TAI,
            timestamp: UnscaledTime::default(),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_jd0() {
        let actual = Time::jd0(TAI);
        let expected = Time::from_unscaled(
            TAI,
            UnscaledTime {
                seconds: -211813488000,
                attoseconds: 0,
            },
        );
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_days_since_j2000() {
        let unscaled = UnscaledTime {
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
        let unscaled = UnscaledTime {
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
        let unscaled_time = UnscaledTime {
            seconds: 1234567890,
            attoseconds: 9876543210,
        };
        let expected = unscaled_time.hour();
        let actual = Time::from_unscaled(TAI, unscaled_time).hour();
        assert_eq!(
            actual, expected,
            "expected Time to have hour {}, but got {}",
            expected, actual
        );
    }

    #[test]
    fn test_time_wall_clock_minute() {
        let unscaled_time = UnscaledTime {
            seconds: 1234567890,
            attoseconds: 9876543210,
        };
        let expected = unscaled_time.minute();
        let actual = Time::from_unscaled(TAI, unscaled_time).minute();
        assert_eq!(
            actual, expected,
            "expected Time to have minute {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_wall_clock_second() {
        let unscaled_time = UnscaledTime {
            seconds: 1234567890,
            attoseconds: 9876543210,
        };
        let expected = unscaled_time.second();
        let actual = Time::from_unscaled(TAI, unscaled_time).second();
        assert_eq!(
            actual, expected,
            "expected Time to have second {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_wall_clock_millisecond() {
        let unscaled_time = UnscaledTime {
            seconds: 1234567890,
            attoseconds: 9876543210,
        };
        let expected = unscaled_time.millisecond();
        let actual = Time::from_unscaled(TAI, unscaled_time).millisecond();
        assert_eq!(
            actual, expected,
            "expected Time to have millisecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_wall_clock_microsecond() {
        let unscaled_time = UnscaledTime {
            seconds: 1234567890,
            attoseconds: 9876543210,
        };
        let expected = unscaled_time.microsecond();
        let actual = Time::from_unscaled(TAI, unscaled_time).microsecond();
        assert_eq!(
            actual, expected,
            "expected Time to have microsecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_wall_clock_nanosecond() {
        let unscaled_time = UnscaledTime {
            seconds: 1234567890,
            attoseconds: 9876543210,
        };
        let expected = unscaled_time.nanosecond();
        let actual = Time::from_unscaled(TAI, unscaled_time).nanosecond();
        assert_eq!(
            actual, expected,
            "expected Time to have nanosecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_wall_clock_picosecond() {
        let unscaled_time = UnscaledTime {
            seconds: 1234567890,
            attoseconds: 9876543210,
        };
        let expected = unscaled_time.picosecond();
        let actual = Time::from_unscaled(TAI, unscaled_time).picosecond();
        assert_eq!(
            actual, expected,
            "expected Time to have picosecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_wall_clock_femtosecond() {
        let unscaled_time = UnscaledTime {
            seconds: 1234567890,
            attoseconds: 9876543210,
        };
        let expected = unscaled_time.femtosecond();
        let actual = Time::from_unscaled(TAI, unscaled_time).femtosecond();
        assert_eq!(
            actual, expected,
            "expected Time to have femtosecond {}, but got {}",
            expected, actual,
        );
    }

    #[test]
    fn test_time_wall_clock_attosecond() {
        let unscaled_time = UnscaledTime {
            seconds: 1234567890,
            attoseconds: 9876543210,
        };
        let expected = unscaled_time.attosecond();
        let actual = Time::from_unscaled(TAI, unscaled_time).attosecond();
        assert_eq!(
            actual, expected,
            "expected Time to have attosecond {}, but got {}",
            expected, actual,
        );
    }
}
