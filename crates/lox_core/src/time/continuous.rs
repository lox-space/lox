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

use num::{abs, ToPrimitive};

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
/// `RawTime` is the base time representation for time scales without leap seconds. It is measured relative to
/// J2000. `RawTime::default()` represents the epoch itself.
///
/// `RawTime` has attosecond precision, and supports times within 292 billion years either side of the epoch.
pub struct RawTime {
    // The sign of the time is determined exclusively by the sign of the `second` field. `attoseconds` is always the
    // positive count of attoseconds since the last whole second. For example, one attosecond before the epoch is
    // represented as
    // ```
    // let time = RawTime {
    //     seconds: -1,
    //     attoseconds: ATTOSECONDS_PER_SECOND - 1,
    // };
    // ```
    seconds: i64,
    attoseconds: u64,
}

impl RawTime {
    fn is_negative(&self) -> bool {
        self.seconds < 0
    }

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

    /// The fractional number of Julian days since J2000.
    fn days_since_j2000(&self) -> f64 {
        self.seconds.to_f64().unwrap() / constants::f64::SECONDS_PER_DAY
            + self.attoseconds.to_f64().unwrap() / constants::f64::ATTOSECONDS_PER_DAY
    }

    /// The fractional number of Julian centuries since J2000.
    fn centuries_since_j2000(&self) -> f64 {
        self.days_since_j2000() / constants::f64::DAYS_PER_JULIAN_CENTURY
    }
}

impl Display for RawTime {
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

impl Add<TimeDelta> for RawTime {
    type Output = Self;

    /// The implementation of [Add] for [RawTime] follows the default Rust rules for integer overflow, which
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

impl Sub<TimeDelta> for RawTime {
    type Output = Self;

    /// The implementation of [Sub] for [RawTime] follows the default Rust rules for integer overflow, which
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

/// The continuous time scales supported by Lox.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimeScale {
    TAI,
    TCB,
    TCG,
    TDB,
    TT,
    UT1,
}

impl Display for TimeScale {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<&str>::into(*self))
    }
}

#[allow(clippy::from_over_into)] // Into is infallible, but From is not
impl Into<&str> for TimeScale {
    fn into(self) -> &'static str {
        match self {
            TimeScale::TAI => "TAI",
            TimeScale::TCB => "TCB",
            TimeScale::TCG => "TCG",
            TimeScale::TDB => "TDB",
            TimeScale::TT => "TT",
            TimeScale::UT1 => "UT1",
        }
    }
}

/// CalendarDate allows continuous time formats to report their date in their respective calendar.
pub trait CalendarDate {
    fn date(&self) -> Date;
}

/// International Atomic Time. Defaults to the J2000 epoch.
#[derive(Debug, Copy, Default, Clone, Eq, PartialEq)]
pub struct TAI(RawTime);

/// Barycentric Coordinate Time. Defaults to the J2000 epoch.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TCB(RawTime);

/// Geocentric Coordinate Time. Defaults to the J2000 epoch.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TCG(RawTime);

/// Barycentric Dynamical Time. Defaults to the J2000 epoch.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TDB(RawTime);

/// Terrestrial Time. Defaults to the J2000 epoch.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TT(RawTime);

/// Universal Time. Defaults to the J2000 epoch.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct UT1(RawTime);

/// Implements a time, [Display] and [WallClock] in terms of the underlying [RawTime].
macro_rules! delegate_to_raw_time {
    ($time_scale:ident, $test_module:ident) => {
        impl $time_scale {
            pub fn new(t: RawTime) -> Self {
                Self(t)
            }

            pub fn to_raw(&self) -> RawTime {
                self.0
            }

            /// The fractional number of Julian days since the J2000 epoch.
            pub fn days_since_j2000(&self) -> f64 {
                self.0.days_since_j2000()
            }

            /// The fractional number of Julian centuries since J2000.
            pub fn centuries_since_j2000(&self) -> f64 {
                self.0.centuries_since_j2000()
            }
        }

        impl Display for $time_scale {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "{} {}", self.0, stringify!($time_scale))
            }
        }

        impl WallClock for $time_scale {
            fn hour(&self) -> i64 {
                self.0.hour()
            }

            fn minute(&self) -> i64 {
                self.0.minute()
            }

            fn second(&self) -> i64 {
                self.0.second()
            }

            fn millisecond(&self) -> i64 {
                self.0.millisecond()
            }

            fn microsecond(&self) -> i64 {
                self.0.microsecond()
            }

            fn nanosecond(&self) -> i64 {
                self.0.nanosecond()
            }

            fn picosecond(&self) -> i64 {
                self.0.picosecond()
            }

            fn femtosecond(&self) -> i64 {
                self.0.femtosecond()
            }

            fn attosecond(&self) -> i64 {
                self.0.attosecond()
            }
        }

        #[cfg(test)]
        mod $test_module {
            use super::{$time_scale, RawTime};
            use float_eq::assert_float_eq;
            use $crate::time::WallClock;

            const RAW_TIME: RawTime = RawTime {
                seconds: 123456789,
                attoseconds: 123456789,
            };

            const TIME: $time_scale = $time_scale(RAW_TIME);

            #[test]
            fn test_new() {
                assert_eq!($time_scale::new(RAW_TIME), TIME);
            }

            #[test]
            fn test_to_raw() {
                assert_eq!(RAW_TIME, TIME.to_raw());
            }

            #[test]
            fn test_days_since_j2000_delegation() {
                let expected = RAW_TIME.days_since_j2000();
                let actual = TIME.days_since_j2000();
                assert_float_eq!(
                    expected,
                    actual,
                    rel <= 1e-15,
                    "expected {} Julian days, got {}",
                    expected,
                    actual,
                );
            }

            #[test]
            fn test_centuries_since_j2000_delegation() {
                let expected = RAW_TIME.centuries_since_j2000();
                let actual = TIME.centuries_since_j2000();
                assert_float_eq!(
                    expected,
                    actual,
                    rel <= 1e-15,
                    "expected {} Julian centuries, got {}",
                    expected,
                    actual,
                );
            }

            #[test]
            fn test_display() {
                let expected = format!("{} {}", RAW_TIME, stringify!($time_scale));
                let actual = format!("{}", TIME);
                assert_eq!(expected, actual);
            }

            #[test]
            fn test_hour_delegation() {
                assert_eq!(TIME.hour(), RAW_TIME.hour());
            }

            #[test]
            fn test_minute_delegation() {
                assert_eq!(TIME.minute(), RAW_TIME.minute());
            }

            #[test]
            fn test_second_delegation() {
                assert_eq!(TIME.second(), RAW_TIME.second());
            }

            #[test]
            fn test_millisecond_delegation() {
                assert_eq!(TIME.millisecond(), RAW_TIME.millisecond());
            }

            #[test]
            fn test_microsecond_delegation() {
                assert_eq!(TIME.microsecond(), RAW_TIME.microsecond());
            }

            #[test]
            fn test_nanosecond_delegation() {
                assert_eq!(TIME.nanosecond(), RAW_TIME.nanosecond());
            }

            #[test]
            fn test_picosecond_delegation() {
                assert_eq!(TIME.picosecond(), RAW_TIME.picosecond());
            }

            #[test]
            fn test_femtosecond_delegation() {
                assert_eq!(TIME.femtosecond(), RAW_TIME.femtosecond());
            }

            #[test]
            fn test_attosecond_delegation() {
                assert_eq!(TIME.attosecond(), RAW_TIME.attosecond());
            }
        }
    };
}

// Implement continuous time scales and `WallClock` in terms of `RawTime`.
delegate_to_raw_time!(TAI, tai_wall_clock_tests);
delegate_to_raw_time!(TCB, tcb_wall_clock_tests);
delegate_to_raw_time!(TCG, tcg_wall_clock_tests);
delegate_to_raw_time!(TDB, tdb_wall_clock_tests);
delegate_to_raw_time!(TT, tt_wall_clock_tests);
delegate_to_raw_time!(UT1, ut1_wall_clock_tests);

/// `Time` represents a time in any of the supported continuous timescales.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Time {
    TAI(TAI),
    TCB(TCB),
    TCG(TCG),
    TDB(TDB),
    TT(TT),
    UT1(UT1),
}

impl Time {
    /// Instantiates a `Time` of the given scale from a date and UTC timestamp.
    pub fn from_date_and_utc_timestamp(scale: TimeScale, date: Date, time: UTC) -> Self {
        let day_in_seconds = date.j2000() * SECONDS_PER_DAY - SECONDS_PER_DAY / 2;
        let hour_in_seconds = time.hour() * SECONDS_PER_HOUR;
        let minute_in_seconds = time.minute() * SECONDS_PER_MINUTE;
        let seconds = day_in_seconds + hour_in_seconds + minute_in_seconds + time.second();
        let attoseconds = time.subsecond_as_attoseconds();
        let raw = RawTime {
            seconds,
            attoseconds,
        };
        Self::from_raw(scale, raw)
    }

    /// Instantiates a `Time` of the given scale from a UTC datetime.
    pub fn from_utc_datetime(scale: TimeScale, dt: UTCDateTime) -> Self {
        Self::from_date_and_utc_timestamp(scale, dt.date(), dt.time())
    }

    pub fn scale(&self) -> TimeScale {
        match &self {
            Time::TAI(_) => TimeScale::TAI,
            Time::TCB(_) => TimeScale::TCB,
            Time::TCG(_) => TimeScale::TCG,
            Time::TDB(_) => TimeScale::TDB,
            Time::TT(_) => TimeScale::TT,
            Time::UT1(_) => TimeScale::UT1,
        }
    }

    /// Returns the J2000 epoch in the given timescale.
    pub fn j2000(scale: TimeScale) -> Self {
        Self::from_raw(scale, RawTime::default())
    }

    /// Returns, as an epoch in the given timescale, midday on the first day of the proleptic Julian
    /// calendar.
    pub fn jd0(scale: TimeScale) -> Self {
        // This represents 4713 BC, since there is no year 0 separating BC and AD.
        let first_proleptic_day = Date::new_unchecked(ProlepticJulian, -4712, 1, 1);
        let midday = UTC::new(12, 0, 0).expect("midday should be a valid time");
        Self::from_date_and_utc_timestamp(scale, first_proleptic_day, midday)
    }

    fn from_raw(scale: TimeScale, raw: RawTime) -> Self {
        match scale {
            TimeScale::TAI => Time::TAI(TAI(raw)),
            TimeScale::TCB => Time::TCB(TCB(raw)),
            TimeScale::TCG => Time::TCG(TCG(raw)),
            TimeScale::TDB => Time::TDB(TDB(raw)),
            TimeScale::TT => Time::TT(TT(raw)),
            TimeScale::UT1 => Time::UT1(UT1(raw)),
        }
    }

    fn raw(&self) -> RawTime {
        match self {
            Time::TAI(tai) => tai.0,
            Time::TCB(tcb) => tcb.0,
            Time::TCG(tcg) => tcg.0,
            Time::TDB(tdb) => tdb.0,
            Time::TT(tt) => tt.0,
            Time::UT1(ut1) => ut1.0,
        }
    }

    /// The number of whole seconds since J2000.
    pub fn seconds(&self) -> i64 {
        self.raw().seconds
    }

    /// The number of attoseconds from the last whole second.
    pub fn attoseconds(&self) -> u64 {
        self.raw().attoseconds
    }

    /// The fractional number of Julian days since J2000.
    pub fn days_since_j2000(&self) -> f64 {
        self.raw().days_since_j2000()
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Time::TAI(t) => t.fmt(f),
            Time::TCB(t) => t.fmt(f),
            Time::TCG(t) => t.fmt(f),
            Time::TDB(t) => t.fmt(f),
            Time::TT(t) => t.fmt(f),
            Time::UT1(t) => t.fmt(f),
        }
    }
}

impl WallClock for Time {
    fn hour(&self) -> i64 {
        match self {
            Time::TAI(t) => t.hour(),
            Time::TCB(t) => t.hour(),
            Time::TCG(t) => t.hour(),
            Time::TDB(t) => t.hour(),
            Time::TT(t) => t.hour(),
            Time::UT1(t) => t.hour(),
        }
    }

    fn minute(&self) -> i64 {
        match self {
            Time::TAI(t) => t.minute(),
            Time::TCB(t) => t.minute(),
            Time::TCG(t) => t.minute(),
            Time::TDB(t) => t.minute(),
            Time::TT(t) => t.minute(),
            Time::UT1(t) => t.minute(),
        }
    }

    fn second(&self) -> i64 {
        match self {
            Time::TAI(t) => t.second(),
            Time::TCB(t) => t.second(),
            Time::TCG(t) => t.second(),
            Time::TDB(t) => t.second(),
            Time::TT(t) => t.second(),
            Time::UT1(t) => t.second(),
        }
    }

    fn millisecond(&self) -> i64 {
        match self {
            Time::TAI(t) => t.millisecond(),
            Time::TCB(t) => t.millisecond(),
            Time::TCG(t) => t.millisecond(),
            Time::TDB(t) => t.millisecond(),
            Time::TT(t) => t.millisecond(),
            Time::UT1(t) => t.millisecond(),
        }
    }

    fn microsecond(&self) -> i64 {
        match self {
            Time::TAI(t) => t.microsecond(),
            Time::TCB(t) => t.microsecond(),
            Time::TCG(t) => t.microsecond(),
            Time::TDB(t) => t.microsecond(),
            Time::TT(t) => t.microsecond(),
            Time::UT1(t) => t.microsecond(),
        }
    }

    fn nanosecond(&self) -> i64 {
        match self {
            Time::TAI(t) => t.nanosecond(),
            Time::TCB(t) => t.nanosecond(),
            Time::TCG(t) => t.nanosecond(),
            Time::TDB(t) => t.nanosecond(),
            Time::TT(t) => t.nanosecond(),
            Time::UT1(t) => t.nanosecond(),
        }
    }

    fn picosecond(&self) -> i64 {
        match self {
            Time::TAI(t) => t.picosecond(),
            Time::TCB(t) => t.picosecond(),
            Time::TCG(t) => t.picosecond(),
            Time::TDB(t) => t.picosecond(),
            Time::TT(t) => t.picosecond(),
            Time::UT1(t) => t.picosecond(),
        }
    }

    fn femtosecond(&self) -> i64 {
        match self {
            Time::TAI(t) => t.femtosecond(),
            Time::TCB(t) => t.femtosecond(),
            Time::TCG(t) => t.femtosecond(),
            Time::TDB(t) => t.femtosecond(),
            Time::TT(t) => t.femtosecond(),
            Time::UT1(t) => t.femtosecond(),
        }
    }

    fn attosecond(&self) -> i64 {
        match self {
            Time::TAI(t) => t.attosecond(),
            Time::TCB(t) => t.attosecond(),
            Time::TCG(t) => t.attosecond(),
            Time::TDB(t) => t.attosecond(),
            Time::TT(t) => t.attosecond(),
            Time::UT1(t) => t.attosecond(),
        }
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use crate::time::dates::Calendar::Gregorian;

    use super::*;

    const TIME_SCALES: [TimeScale; 6] = [
        TimeScale::TAI,
        TimeScale::TCB,
        TimeScale::TCG,
        TimeScale::TDB,
        TimeScale::TT,
        TimeScale::UT1,
    ];

    #[test]
    fn test_raw_time_is_negative() {
        assert!(RawTime {
            seconds: -1,
            attoseconds: 0
        }
        .is_negative());
        assert!(!RawTime {
            seconds: 0,
            attoseconds: 0
        }
        .is_negative());
        assert!(!RawTime {
            seconds: 1,
            attoseconds: 0
        }
        .is_negative());
    }

    #[test]
    fn test_raw_time_hour() {
        struct TestCase {
            desc: &'static str,
            time: RawTime,
            expected_hour: i64,
        }

        let test_cases = [
            TestCase {
                desc: "zero value",
                time: RawTime {
                    seconds: 0,
                    attoseconds: 0,
                },
                expected_hour: 12,
            },
            TestCase {
                desc: "one attosecond less than an hour",
                time: RawTime {
                    seconds: SECONDS_PER_HOUR - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_hour: 12,
            },
            TestCase {
                desc: "exactly one hour",
                time: RawTime {
                    seconds: SECONDS_PER_HOUR,
                    attoseconds: 0,
                },
                expected_hour: 13,
            },
            TestCase {
                desc: "one day and one hour",
                time: RawTime {
                    seconds: SECONDS_PER_HOUR * 25,
                    attoseconds: 0,
                },
                expected_hour: 13,
            },
            TestCase {
                desc: "one attosecond less than the epoch",
                time: RawTime {
                    seconds: -1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_hour: 11,
            },
            TestCase {
                desc: "one hour less than the epoch",
                time: RawTime {
                    seconds: -SECONDS_PER_HOUR,
                    attoseconds: 0,
                },
                expected_hour: 11,
            },
            TestCase {
                desc: "one hour and one attosecond less than the epoch",
                time: RawTime {
                    seconds: -SECONDS_PER_HOUR - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_hour: 10,
            },
            TestCase {
                desc: "one day less than the epoch",
                time: RawTime {
                    seconds: -SECONDS_PER_DAY,
                    attoseconds: 0,
                },
                expected_hour: 12,
            },
            TestCase {
                // Exercises the case where the number of seconds exceeds the number of seconds in a day.
                desc: "two days less than the epoch",
                time: RawTime {
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
    fn test_raw_time_minute() {
        struct TestCase {
            desc: &'static str,
            time: RawTime,
            expected_minute: i64,
        }

        let test_cases = [
            TestCase {
                desc: "zero value",
                time: RawTime {
                    seconds: 0,
                    attoseconds: 0,
                },
                expected_minute: 0,
            },
            TestCase {
                desc: "one attosecond less than one minute",
                time: RawTime {
                    seconds: SECONDS_PER_MINUTE - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_minute: 0,
            },
            TestCase {
                desc: "one minute",
                time: RawTime {
                    seconds: SECONDS_PER_MINUTE,
                    attoseconds: 0,
                },
                expected_minute: 1,
            },
            TestCase {
                desc: "one attosecond less than an hour",
                time: RawTime {
                    seconds: SECONDS_PER_HOUR - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_minute: 59,
            },
            TestCase {
                desc: "exactly one hour",
                time: RawTime {
                    seconds: SECONDS_PER_HOUR,
                    attoseconds: 0,
                },
                expected_minute: 0,
            },
            TestCase {
                desc: "one hour and one minute",
                time: RawTime {
                    seconds: SECONDS_PER_HOUR + SECONDS_PER_MINUTE,
                    attoseconds: 0,
                },
                expected_minute: 1,
            },
            TestCase {
                desc: "one attosecond less than the epoch",
                time: RawTime {
                    seconds: -1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_minute: 59,
            },
            TestCase {
                desc: "one minute less than the epoch",
                time: RawTime {
                    seconds: -SECONDS_PER_MINUTE,
                    attoseconds: 0,
                },
                expected_minute: 59,
            },
            TestCase {
                desc: "one minute and one attosecond less than the epoch",
                time: RawTime {
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
    fn test_raw_time_second() {
        struct TestCase {
            desc: &'static str,
            time: RawTime,
            expected_second: i64,
        }

        let test_cases = [
            TestCase {
                desc: "zero value",
                time: RawTime {
                    seconds: 0,
                    attoseconds: 0,
                },
                expected_second: 0,
            },
            TestCase {
                desc: "one attosecond less than one second",
                time: RawTime {
                    seconds: 0,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_second: 0,
            },
            TestCase {
                desc: "one second",
                time: RawTime {
                    seconds: 1,
                    attoseconds: 0,
                },
                expected_second: 1,
            },
            TestCase {
                desc: "one attosecond less than a minute",
                time: RawTime {
                    seconds: SECONDS_PER_MINUTE - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_second: 59,
            },
            TestCase {
                desc: "exactly one minute",
                time: RawTime {
                    seconds: SECONDS_PER_MINUTE,
                    attoseconds: 0,
                },
                expected_second: 0,
            },
            TestCase {
                desc: "one minute and one second",
                time: RawTime {
                    seconds: SECONDS_PER_MINUTE + 1,
                    attoseconds: 0,
                },
                expected_second: 1,
            },
            TestCase {
                desc: "one attosecond less than the epoch",
                time: RawTime {
                    seconds: -1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_second: 59,
            },
            TestCase {
                desc: "one second less than the epoch",
                time: RawTime {
                    seconds: -1,
                    attoseconds: 0,
                },
                expected_second: 59,
            },
            TestCase {
                desc: "one second and one attosecond less than the epoch",
                time: RawTime {
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
    fn test_raw_time_subseconds_with_positive_seconds() {
        let time = RawTime {
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
    fn test_raw_time_subseconds_with_negative_seconds() {
        let time = RawTime {
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
    fn test_raw_time_days_since_j2000() {
        struct TestCase {
            desc: &'static str,
            time: RawTime,
            expected: f64,
        }

        let test_cases = [
            TestCase {
                desc: "at the epoch",
                time: RawTime::default(),
                expected: 0.0,
            },
            TestCase {
                desc: "less than one day after the epoch",
                time: RawTime {
                    seconds: SECONDS_PER_DAY - 1,
                    // Test a sufficient number of attoseconds that the difference is representable by a float.
                    attoseconds: ATTOSECONDS_PER_SECOND / 2,
                },
                expected: 0.999994212962963,
            },
            TestCase {
                desc: "one day after the epoch",
                time: RawTime {
                    seconds: SECONDS_PER_DAY,
                    attoseconds: 0,
                },
                expected: 1.0,
            },
            TestCase {
                desc: "before the epoch",
                time: RawTime {
                    seconds: -1,
                    // Test a sufficient number of attoseconds that the difference is representable by a float.
                    attoseconds: ATTOSECONDS_PER_SECOND / 2,
                },
                expected: -0.000005787037037037037,
            },
        ];

        for tc in test_cases {
            let actual = tc.time.days_since_j2000();
            assert_float_eq!(
                actual,
                tc.expected,
                rel <= 1e-15,
                "{}: expected {}, got {}",
                tc.desc,
                tc.expected,
                actual
            );
        }
    }

    #[test]
    fn test_raw_time_add_time_delta() {
        struct TestCase {
            desc: &'static str,
            delta: TimeDelta,
            time: RawTime,
            expected: RawTime,
        }

        let test_cases = [
            TestCase {
                desc: "positive time with no attosecond wrap",
                delta: TimeDelta {
                    seconds: 1,
                    attoseconds: 1,
                },
                time: RawTime {
                    seconds: 1,
                    attoseconds: 0,
                },
                expected: RawTime {
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
                time: RawTime {
                    seconds: 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected: RawTime {
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
                time: RawTime {
                    seconds: -1,
                    attoseconds: 0,
                },
                expected: RawTime {
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
                time: RawTime {
                    seconds: -1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected: RawTime {
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
    fn test_raw_time_sub_time_delta() {
        struct TestCase {
            desc: &'static str,
            delta: TimeDelta,
            time: RawTime,
            expected: RawTime,
        }

        let test_cases = [
            TestCase {
                desc: "positive time with no attosecond wrap",
                delta: TimeDelta {
                    seconds: 1,
                    attoseconds: 1,
                },
                time: RawTime {
                    seconds: 2,
                    attoseconds: 2,
                },
                expected: RawTime {
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
                time: RawTime {
                    seconds: 2,
                    attoseconds: 1,
                },
                expected: RawTime {
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
                time: RawTime {
                    seconds: -1,
                    attoseconds: 2,
                },
                expected: RawTime {
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
                time: RawTime {
                    seconds: -1,
                    attoseconds: 1,
                },
                expected: RawTime {
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
                time: RawTime {
                    seconds: 0,
                    attoseconds: 1,
                },
                expected: RawTime {
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
    fn test_timescale_into_str() {
        let test_cases = [
            (TimeScale::TAI, "TAI"),
            (TimeScale::TCB, "TCB"),
            (TimeScale::TCG, "TCG"),
            (TimeScale::TDB, "TDB"),
            (TimeScale::TT, "TT"),
            (TimeScale::UT1, "UT1"),
        ];

        for (scale, expected) in test_cases {
            assert_eq!(Into::<&str>::into(scale), expected);
        }
    }

    #[test]
    fn test_time_from_date_and_utc_timestamp() {
        let date = Date::new_unchecked(Gregorian, 2021, 1, 1);
        let utc = UTC::new(12, 34, 56).expect("time should be valid");
        let datetime = UTCDateTime::new(date, utc);

        for scale in TIME_SCALES {
            let actual = Time::from_date_and_utc_timestamp(scale, date, utc);
            let expected = Time::from_utc_datetime(scale, datetime);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_time_display() {
        let tai = TAI::default();
        let time = Time::TAI(tai);
        assert_eq!(tai.to_string(), time.to_string());
    }

    #[test]
    fn test_time_j2000() {
        [
            (TimeScale::TAI, Time::TAI(TAI::default())),
            (TimeScale::TCB, Time::TCB(TCB::default())),
            (TimeScale::TCG, Time::TCG(TCG::default())),
            (TimeScale::TDB, Time::TDB(TDB::default())),
            (TimeScale::TT, Time::TT(TT::default())),
            (TimeScale::UT1, Time::UT1(UT1::default())),
        ]
        .iter()
        .for_each(|(scale, expected)| {
            let actual = Time::j2000(*scale);
            assert_eq!(*expected, actual);
        });
    }

    #[test]
    fn test_time_jd0() {
        [
            (
                TimeScale::TAI,
                Time::TAI(TAI(RawTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
            (
                TimeScale::TCB,
                Time::TCB(TCB(RawTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
            (
                TimeScale::TCG,
                Time::TCG(TCG(RawTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
            (
                TimeScale::TDB,
                Time::TDB(TDB(RawTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
            (
                TimeScale::TT,
                Time::TT(TT(RawTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
            (
                TimeScale::UT1,
                Time::UT1(UT1(RawTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
        ]
        .iter()
        .for_each(|(scale, expected)| {
            let actual = Time::jd0(*scale);
            assert_eq!(*expected, actual);
        });
    }

    #[test]
    fn test_time_scale() {
        let test_cases = [
            (Time::TAI(TAI::default()), TimeScale::TAI),
            (Time::TCB(TCB::default()), TimeScale::TCB),
            (Time::TCG(TCG::default()), TimeScale::TCG),
            (Time::TDB(TDB::default()), TimeScale::TDB),
            (Time::TT(TT::default()), TimeScale::TT),
            (Time::UT1(UT1::default()), TimeScale::UT1),
        ];

        for (time, expected) in test_cases {
            assert_eq!(time.scale(), expected);
        }
    }

    #[test]
    fn test_time_wall_clock_hour() {
        let raw_time = RawTime::default();
        let expected = raw_time.hour();
        for scale in TIME_SCALES {
            let time = Time::from_raw(scale, raw_time);
            let actual = time.hour();
            assert_eq!(
                actual, expected,
                "expected time in scale {} to have hour {}, but got {}",
                scale, expected, actual
            );
        }
    }

    #[test]
    fn test_time_wall_clock_minute() {
        let raw_time = RawTime::default();
        let expected = raw_time.minute();
        for scale in TIME_SCALES {
            let time = Time::from_raw(scale, raw_time);
            let actual = time.minute();
            assert_eq!(
                actual, expected,
                "expected time in scale {} to have minute {}, but got {}",
                scale, expected, actual
            );
        }
    }

    #[test]
    fn test_time_wall_clock_second() {
        let raw_time = RawTime::default();
        let expected = raw_time.second();
        for scale in TIME_SCALES {
            let time = Time::from_raw(scale, raw_time);
            let actual = time.second();
            assert_eq!(
                actual, expected,
                "expected time in scale {} to have second {}, but got {}",
                scale, expected, actual
            );
        }
    }

    #[test]
    fn test_time_wall_clock_millisecond() {
        let raw_time = RawTime::default();
        let expected = raw_time.millisecond();
        for scale in TIME_SCALES {
            let time = Time::from_raw(scale, raw_time);
            let actual = time.millisecond();
            assert_eq!(
                actual, expected,
                "expected time in scale {} to have millisecond {}, but got {}",
                scale, expected, actual
            );
        }
    }

    #[test]
    fn test_time_wall_clock_microsecond() {
        let raw_time = RawTime::default();
        let expected = raw_time.microsecond();
        for scale in TIME_SCALES {
            let time = Time::from_raw(scale, raw_time);
            let actual = time.microsecond();
            assert_eq!(
                actual, expected,
                "expected time in scale {} to have microsecond {}, but got {}",
                scale, expected, actual
            );
        }
    }

    #[test]
    fn test_time_wall_clock_nanosecond() {
        let raw_time = RawTime::default();
        let expected = raw_time.nanosecond();
        for scale in TIME_SCALES {
            let time = Time::from_raw(scale, raw_time);
            let actual = time.nanosecond();
            assert_eq!(
                actual, expected,
                "expected time in scale {} to have nanosecond {}, but got {}",
                scale, expected, actual
            );
        }
    }

    #[test]
    fn test_time_wall_clock_picosecond() {
        let raw_time = RawTime::default();
        let expected = raw_time.picosecond();
        for scale in TIME_SCALES {
            let time = Time::from_raw(scale, raw_time);
            let actual = time.picosecond();
            assert_eq!(
                actual, expected,
                "expected time in scale {} to have picosecond {}, but got {}",
                scale, expected, actual
            );
        }
    }

    #[test]
    fn test_time_wall_clock_femtosecond() {
        let raw_time = RawTime::default();
        let expected = raw_time.femtosecond();
        for scale in TIME_SCALES {
            let time = Time::from_raw(scale, raw_time);
            let actual = time.femtosecond();
            assert_eq!(
                actual, expected,
                "expected time in scale {} to have femtosecond {}, but got {}",
                scale, expected, actual
            );
        }
    }

    #[test]
    fn test_time_wall_clock_attosecond() {
        let raw_time = RawTime::default();
        let expected = raw_time.attosecond();
        for scale in TIME_SCALES {
            let time = Time::from_raw(scale, raw_time);
            let actual = time.attosecond();
            assert_eq!(
                actual, expected,
                "expected time in scale {} to have attosecond {}, but got {}",
                scale, expected, actual
            );
        }
    }
}
