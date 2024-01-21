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
//! The supported timescales are specified by [ContinuousTimeScale].

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
use crate::time::{constants, TimeScale, WallClock};

/// An absolute continuous time difference with attosecond precision.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TimeDelta {
    seconds: u64,
    attoseconds: u64,
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
/// `RawContinuousTime` is the base time representation for time scales without leap seconds. It is measured relative to
/// J2000. `RawTime::default()` represents the epoch itself.
///
/// `RawContinuousTime` has attosecond precision, and supports times within 292 billion years either side of the epoch.
pub struct RawContinuousTime {
    // The sign of the time is determined exclusively by the sign of the `second` field. `attoseconds` is always the
    // positive count of attoseconds since the last whole second. For example, one attosecond before the epoch is
    // represented as
    // ```
    // let time = RawContinuousTime {
    //     seconds: -1,
    //     attoseconds: ATTOSECONDS_PER_SECOND - 1,
    // };
    // ```
    seconds: i64,
    attoseconds: u64,
}

impl RawContinuousTime {
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
}

impl Add<TimeDelta> for RawContinuousTime {
    type Output = Self;

    /// The implementation of [Add] for [RawContinuousTime] follows the default Rust rules for integer overflow, which
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

impl Sub<TimeDelta> for RawContinuousTime {
    type Output = Self;

    /// The implementation of [Sub] for [RawContinuousTime] follows the default Rust rules for integer overflow, which
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
pub enum ContinuousTimeScale {
    TAI,
    TCB,
    TCG,
    TDB,
    TT,
    UT1,
}

/// CalendarDate allows continuous time formats to report their date in their respective calendar.
pub trait CalendarDate {
    fn date(&self) -> Date;
}

/// International Atomic Time. Defaults to the J2000 epoch.
#[derive(Debug, Copy, Default, Clone, Eq, PartialEq)]
pub struct TAI(RawContinuousTime);

impl TAI {
    pub fn to_ut1(&self, _dut: TimeDelta, _dat: TimeDelta) -> UT1 {
        todo!()
    }
}

impl CalendarDate for TAI {
    fn date(&self) -> Date {
        todo!()
    }
}

/// Barycentric Coordinate Time. Defaults to the J2000 epoch.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TCB(RawContinuousTime);

/// Geocentric Coordinate Time. Defaults to the J2000 epoch.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TCG(RawContinuousTime);

/// Barycentric Dynamical Time. Defaults to the J2000 epoch.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TDB(RawContinuousTime);

/// Terrestrial Time. Defaults to the J2000 epoch.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TT(RawContinuousTime);

/// Universal Time. Defaults to the J2000 epoch.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct UT1(RawContinuousTime);

/// Implements the `WallClock` trait for the a time scale based on [RawContinuousTime] in terms of the underlying
/// raw time.
macro_rules! wall_clock {
    ($time_scale:ident, $test_module:ident) => {
        impl WallClock for $time_scale {
            fn scale(&self) -> TimeScale {
                TimeScale::$time_scale
            }

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
            use super::{$time_scale, RawContinuousTime};
            use crate::time::{TimeScale, WallClock};

            const RAW_TIME: RawContinuousTime = RawContinuousTime {
                seconds: 1234,
                attoseconds: 5678,
            };

            const TIME: $time_scale = $time_scale(RAW_TIME);

            #[test]
            fn test_scale() {
                assert_eq!(TIME.scale(), TimeScale::$time_scale);
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

// Implement WallClock for all continuous time scales.
wall_clock!(TAI, tai_wall_clock_tests);
wall_clock!(TCB, tcb_wall_clock_tests);
wall_clock!(TCG, tcg_wall_clock_tests);
wall_clock!(TDB, tdb_wall_clock_tests);
wall_clock!(TT, tt_wall_clock_tests);
wall_clock!(UT1, ut1_wall_clock_tests);

/// `ContinuousTime` represents a time in any of the supported continuous timescales.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ContinuousTime {
    TAI(TAI),
    TCB(TCB),
    TCG(TCG),
    TDB(TDB),
    TT(TT),
    UT1(UT1),
}

impl ContinuousTime {
    /// Instantiates a `ContinuousTime` of the given scale from a date and UTC timestamp.
    pub fn from_date_and_utc_timestamp(scale: ContinuousTimeScale, date: Date, time: UTC) -> Self {
        let day_in_seconds = date.j2000() * SECONDS_PER_DAY - SECONDS_PER_DAY / 2;
        let hour_in_seconds = time.hour() * SECONDS_PER_HOUR;
        let minute_in_seconds = time.minute() * SECONDS_PER_MINUTE;
        let seconds = day_in_seconds + hour_in_seconds + minute_in_seconds + time.second();
        let attoseconds = time.subsecond_as_attoseconds();
        let raw = RawContinuousTime {
            seconds,
            attoseconds,
        };
        Self::from_raw(scale, raw)
    }

    /// Instantiates a `ContinuousTime` of the given scale from a UTC datetime.
    pub fn from_utc_datetime(scale: ContinuousTimeScale, dt: UTCDateTime) -> Self {
        Self::from_date_and_utc_timestamp(scale, dt.date(), dt.time())
    }

    /// Returns the J2000 epoch in the given timescale.
    pub fn j2000(scale: ContinuousTimeScale) -> Self {
        Self::from_raw(scale, RawContinuousTime::default())
    }

    /// Returns, as an epoch in the given timescale, midday on the first day of the proleptic Julian
    /// calendar.
    pub fn jd0(scale: ContinuousTimeScale) -> Self {
        // This represents 4713 BC, since there is no year 0 separating BC and AD.
        let first_proleptic_day = Date::new_unchecked(ProlepticJulian, -4712, 1, 1);
        let midday = UTC::new(12, 0, 0).expect("midday should be a valid time");
        Self::from_date_and_utc_timestamp(scale, first_proleptic_day, midday)
    }

    fn from_raw(scale: ContinuousTimeScale, raw: RawContinuousTime) -> Self {
        match scale {
            ContinuousTimeScale::TAI => ContinuousTime::TAI(TAI(raw)),
            ContinuousTimeScale::TCB => ContinuousTime::TCB(TCB(raw)),
            ContinuousTimeScale::TCG => ContinuousTime::TCG(TCG(raw)),
            ContinuousTimeScale::TDB => ContinuousTime::TDB(TDB(raw)),
            ContinuousTimeScale::TT => ContinuousTime::TT(TT(raw)),
            ContinuousTimeScale::UT1 => ContinuousTime::UT1(UT1(raw)),
        }
    }

    fn raw(&self) -> RawContinuousTime {
        match self {
            ContinuousTime::TAI(tai) => tai.0,
            ContinuousTime::TCB(tcb) => tcb.0,
            ContinuousTime::TCG(tcg) => tcg.0,
            ContinuousTime::TDB(tdb) => tdb.0,
            ContinuousTime::TT(tt) => tt.0,
            ContinuousTime::UT1(ut1) => ut1.0,
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
    pub fn fractional_days(&self) -> f64 {
        let d1 = self.seconds().to_f64().unwrap_or_default() / constants::f64::SECONDS_PER_DAY;
        let d2 = self.attoseconds().to_f64().unwrap() / constants::f64::ATTOSECONDS_PER_DAY;
        d2 + d1
    }
}

impl Display for ContinuousTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "foo")
    }
}

impl WallClock for ContinuousTime {
    fn scale(&self) -> TimeScale {
        match self {
            ContinuousTime::TAI(_) => TimeScale::TAI,
            ContinuousTime::TCB(_) => TimeScale::TCB,
            ContinuousTime::TCG(_) => TimeScale::TCG,
            ContinuousTime::TDB(_) => TimeScale::TDB,
            ContinuousTime::TT(_) => TimeScale::TT,
            ContinuousTime::UT1(_) => TimeScale::UT1,
        }
    }

    fn hour(&self) -> i64 {
        match self {
            ContinuousTime::TAI(t) => t.hour(),
            ContinuousTime::TCB(t) => t.hour(),
            ContinuousTime::TCG(t) => t.hour(),
            ContinuousTime::TDB(t) => t.hour(),
            ContinuousTime::TT(t) => t.hour(),
            ContinuousTime::UT1(t) => t.hour(),
        }
    }

    fn minute(&self) -> i64 {
        match self {
            ContinuousTime::TAI(t) => t.minute(),
            ContinuousTime::TCB(t) => t.minute(),
            ContinuousTime::TCG(t) => t.minute(),
            ContinuousTime::TDB(t) => t.minute(),
            ContinuousTime::TT(t) => t.minute(),
            ContinuousTime::UT1(t) => t.minute(),
        }
    }

    fn second(&self) -> i64 {
        match self {
            ContinuousTime::TAI(t) => t.second(),
            ContinuousTime::TCB(t) => t.second(),
            ContinuousTime::TCG(t) => t.second(),
            ContinuousTime::TDB(t) => t.second(),
            ContinuousTime::TT(t) => t.second(),
            ContinuousTime::UT1(t) => t.second(),
        }
    }

    fn millisecond(&self) -> i64 {
        match self {
            ContinuousTime::TAI(t) => t.millisecond(),
            ContinuousTime::TCB(t) => t.millisecond(),
            ContinuousTime::TCG(t) => t.millisecond(),
            ContinuousTime::TDB(t) => t.millisecond(),
            ContinuousTime::TT(t) => t.millisecond(),
            ContinuousTime::UT1(t) => t.millisecond(),
        }
    }

    fn microsecond(&self) -> i64 {
        match self {
            ContinuousTime::TAI(t) => t.microsecond(),
            ContinuousTime::TCB(t) => t.microsecond(),
            ContinuousTime::TCG(t) => t.microsecond(),
            ContinuousTime::TDB(t) => t.microsecond(),
            ContinuousTime::TT(t) => t.microsecond(),
            ContinuousTime::UT1(t) => t.microsecond(),
        }
    }

    fn nanosecond(&self) -> i64 {
        match self {
            ContinuousTime::TAI(t) => t.nanosecond(),
            ContinuousTime::TCB(t) => t.nanosecond(),
            ContinuousTime::TCG(t) => t.nanosecond(),
            ContinuousTime::TDB(t) => t.nanosecond(),
            ContinuousTime::TT(t) => t.nanosecond(),
            ContinuousTime::UT1(t) => t.nanosecond(),
        }
    }

    fn picosecond(&self) -> i64 {
        match self {
            ContinuousTime::TAI(t) => t.picosecond(),
            ContinuousTime::TCB(t) => t.picosecond(),
            ContinuousTime::TCG(t) => t.picosecond(),
            ContinuousTime::TDB(t) => t.picosecond(),
            ContinuousTime::TT(t) => t.picosecond(),
            ContinuousTime::UT1(t) => t.picosecond(),
        }
    }

    fn femtosecond(&self) -> i64 {
        match self {
            ContinuousTime::TAI(t) => t.femtosecond(),
            ContinuousTime::TCB(t) => t.femtosecond(),
            ContinuousTime::TCG(t) => t.femtosecond(),
            ContinuousTime::TDB(t) => t.femtosecond(),
            ContinuousTime::TT(t) => t.femtosecond(),
            ContinuousTime::UT1(t) => t.femtosecond(),
        }
    }

    fn attosecond(&self) -> i64 {
        match self {
            ContinuousTime::TAI(t) => t.attosecond(),
            ContinuousTime::TCB(t) => t.attosecond(),
            ContinuousTime::TCG(t) => t.attosecond(),
            ContinuousTime::TDB(t) => t.attosecond(),
            ContinuousTime::TT(t) => t.attosecond(),
            ContinuousTime::UT1(t) => t.attosecond(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_continuous_time_is_negative() {
        assert!(RawContinuousTime {
            seconds: -1,
            attoseconds: 0
        }
        .is_negative());
        assert!(!RawContinuousTime {
            seconds: 0,
            attoseconds: 0
        }
        .is_negative());
        assert!(!RawContinuousTime {
            seconds: 1,
            attoseconds: 0
        }
        .is_negative());
    }

    #[test]
    fn test_raw_continuous_time_hour() {
        struct TestCase {
            desc: &'static str,
            time: RawContinuousTime,
            expected_hour: i64,
        }

        let test_cases = [
            TestCase {
                desc: "zero value",
                time: RawContinuousTime {
                    seconds: 0,
                    attoseconds: 0,
                },
                expected_hour: 12,
            },
            TestCase {
                desc: "one attosecond less than an hour",
                time: RawContinuousTime {
                    seconds: SECONDS_PER_HOUR - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_hour: 12,
            },
            TestCase {
                desc: "exactly one hour",
                time: RawContinuousTime {
                    seconds: SECONDS_PER_HOUR,
                    attoseconds: 0,
                },
                expected_hour: 13,
            },
            TestCase {
                desc: "one day and one hour",
                time: RawContinuousTime {
                    seconds: SECONDS_PER_HOUR * 25,
                    attoseconds: 0,
                },
                expected_hour: 13,
            },
            TestCase {
                desc: "one attosecond less than the epoch",
                time: RawContinuousTime {
                    seconds: -1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_hour: 11,
            },
            TestCase {
                desc: "one hour less than the epoch",
                time: RawContinuousTime {
                    seconds: -SECONDS_PER_HOUR,
                    attoseconds: 0,
                },
                expected_hour: 11,
            },
            TestCase {
                desc: "one hour and one attosecond less than the epoch",
                time: RawContinuousTime {
                    seconds: -SECONDS_PER_HOUR - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_hour: 10,
            },
            TestCase {
                desc: "one day less than the epoch",
                time: RawContinuousTime {
                    seconds: -SECONDS_PER_DAY,
                    attoseconds: 0,
                },
                expected_hour: 12,
            },
            TestCase {
                // Exercises the case where the number of seconds exceeds the number of seconds in a day.
                desc: "two days less than the epoch",
                time: RawContinuousTime {
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
    fn test_raw_continuous_time_minute() {
        struct TestCase {
            desc: &'static str,
            time: RawContinuousTime,
            expected_minute: i64,
        }

        let test_cases = [
            TestCase {
                desc: "zero value",
                time: RawContinuousTime {
                    seconds: 0,
                    attoseconds: 0,
                },
                expected_minute: 0,
            },
            TestCase {
                desc: "one attosecond less than one minute",
                time: RawContinuousTime {
                    seconds: SECONDS_PER_MINUTE - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_minute: 0,
            },
            TestCase {
                desc: "one minute",
                time: RawContinuousTime {
                    seconds: SECONDS_PER_MINUTE,
                    attoseconds: 0,
                },
                expected_minute: 1,
            },
            TestCase {
                desc: "one attosecond less than an hour",
                time: RawContinuousTime {
                    seconds: SECONDS_PER_HOUR - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_minute: 59,
            },
            TestCase {
                desc: "exactly one hour",
                time: RawContinuousTime {
                    seconds: SECONDS_PER_HOUR,
                    attoseconds: 0,
                },
                expected_minute: 0,
            },
            TestCase {
                desc: "one hour and one minute",
                time: RawContinuousTime {
                    seconds: SECONDS_PER_HOUR + SECONDS_PER_MINUTE,
                    attoseconds: 0,
                },
                expected_minute: 1,
            },
            TestCase {
                desc: "one attosecond less than the epoch",
                time: RawContinuousTime {
                    seconds: -1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_minute: 59,
            },
            TestCase {
                desc: "one minute less than the epoch",
                time: RawContinuousTime {
                    seconds: -SECONDS_PER_MINUTE,
                    attoseconds: 0,
                },
                expected_minute: 59,
            },
            TestCase {
                desc: "one minute and one attosecond less than the epoch",
                time: RawContinuousTime {
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
    fn test_raw_continuous_time_second() {
        struct TestCase {
            desc: &'static str,
            time: RawContinuousTime,
            expected_second: i64,
        }

        let test_cases = [
            TestCase {
                desc: "zero value",
                time: RawContinuousTime {
                    seconds: 0,
                    attoseconds: 0,
                },
                expected_second: 0,
            },
            TestCase {
                desc: "one attosecond less than one second",
                time: RawContinuousTime {
                    seconds: 0,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_second: 0,
            },
            TestCase {
                desc: "one second",
                time: RawContinuousTime {
                    seconds: 1,
                    attoseconds: 0,
                },
                expected_second: 1,
            },
            TestCase {
                desc: "one attosecond less than a minute",
                time: RawContinuousTime {
                    seconds: SECONDS_PER_MINUTE - 1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_second: 59,
            },
            TestCase {
                desc: "exactly one minute",
                time: RawContinuousTime {
                    seconds: SECONDS_PER_MINUTE,
                    attoseconds: 0,
                },
                expected_second: 0,
            },
            TestCase {
                desc: "one minute and one second",
                time: RawContinuousTime {
                    seconds: SECONDS_PER_MINUTE + 1,
                    attoseconds: 0,
                },
                expected_second: 1,
            },
            TestCase {
                desc: "one attosecond less than the epoch",
                time: RawContinuousTime {
                    seconds: -1,
                    attoseconds: ATTOSECONDS_PER_SECOND - 1,
                },
                expected_second: 59,
            },
            TestCase {
                desc: "one second less than the epoch",
                time: RawContinuousTime {
                    seconds: -1,
                    attoseconds: 0,
                },
                expected_second: 59,
            },
            TestCase {
                desc: "one second and one attosecond less than the epoch",
                time: RawContinuousTime {
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
    fn test_raw_continuous_time_subseconds_with_positive_seconds() {
        let time = RawContinuousTime {
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
    fn test_raw_continuous_time_subseconds_with_negative_seconds() {
        let time = RawContinuousTime {
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
    fn test_raw_continuous_time_add_time_delta_positive_time_no_attosecond_wrap() {
        let delta = TimeDelta {
            seconds: 1,
            attoseconds: 1,
        };
        let time = RawContinuousTime {
            seconds: 1,
            attoseconds: 0,
        };
        let expected = RawContinuousTime {
            seconds: 2,
            attoseconds: 1,
        };
        let actual = time + delta;
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_raw_continuous_time_add_time_delta_positive_time_attosecond_wrap() {
        let delta = TimeDelta {
            seconds: 1,
            attoseconds: 2,
        };
        let time = RawContinuousTime {
            seconds: 1,
            attoseconds: ATTOSECONDS_PER_SECOND - 1,
        };
        let expected = RawContinuousTime {
            seconds: 3,
            attoseconds: 1,
        };
        let actual = time + delta;
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_raw_continuous_time_add_time_delta_negative_time_no_attosecond_wrap() {
        let delta = TimeDelta {
            seconds: 1,
            attoseconds: 1,
        };
        let time = RawContinuousTime {
            seconds: -1,
            attoseconds: 0,
        };
        let expected = RawContinuousTime {
            seconds: 0,
            attoseconds: 1,
        };
        let actual = time + delta;
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_raw_continuous_time_add_time_delta_negative_time_attosecond_wrap() {
        let delta = TimeDelta {
            seconds: 1,
            attoseconds: 2,
        };
        let time = RawContinuousTime {
            seconds: -1,
            attoseconds: ATTOSECONDS_PER_SECOND - 1,
        };
        let expected = RawContinuousTime {
            seconds: 1,
            attoseconds: 1,
        };
        let actual = time + delta;
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_j2000() {
        [
            (
                ContinuousTimeScale::TAI,
                ContinuousTime::TAI(TAI::default()),
            ),
            (
                ContinuousTimeScale::TCB,
                ContinuousTime::TCB(TCB::default()),
            ),
            (
                ContinuousTimeScale::TCG,
                ContinuousTime::TCG(TCG::default()),
            ),
            (
                ContinuousTimeScale::TDB,
                ContinuousTime::TDB(TDB::default()),
            ),
            (ContinuousTimeScale::TT, ContinuousTime::TT(TT::default())),
            (
                ContinuousTimeScale::UT1,
                ContinuousTime::UT1(UT1::default()),
            ),
        ]
        .iter()
        .for_each(|(scale, expected)| {
            let actual = ContinuousTime::j2000(*scale);
            assert_eq!(*expected, actual);
        });
    }

    #[test]
    fn test_jd0() {
        [
            (
                ContinuousTimeScale::TAI,
                ContinuousTime::TAI(TAI(RawContinuousTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
            (
                ContinuousTimeScale::TCB,
                ContinuousTime::TCB(TCB(RawContinuousTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
            (
                ContinuousTimeScale::TCG,
                ContinuousTime::TCG(TCG(RawContinuousTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
            (
                ContinuousTimeScale::TDB,
                ContinuousTime::TDB(TDB(RawContinuousTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
            (
                ContinuousTimeScale::TT,
                ContinuousTime::TT(TT(RawContinuousTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
            (
                ContinuousTimeScale::UT1,
                ContinuousTime::UT1(UT1(RawContinuousTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
        ]
        .iter()
        .for_each(|(scale, expected)| {
            let actual = ContinuousTime::jd0(*scale);
            assert_eq!(*expected, actual);
        });
    }
}
