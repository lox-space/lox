/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt;
use std::fmt::Formatter;
use std::ops::{Add, Sub};

use num::ToPrimitive;

use crate::time::constants;
use crate::time::constants::i64::{SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE};
use crate::time::constants::u64::ATTOSECONDS_PER_SECOND;
use crate::time::dates::Calendar::ProlepticJulian;
use crate::time::dates::Date;
use crate::time::utc::{UTCDateTime, UTC};

/// The continuous time scales supported by Lox.
#[derive(Debug, Copy, Clone)]
pub enum TimeScale {
    TAI,
    TCB,
    TCG,
    TDB,
    TT,
    UT1,
}

impl fmt::Display for TimeScale {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TimeScale::TAI => write!(f, "TAI"),
            TimeScale::TCB => write!(f, "TCB"),
            TimeScale::TCG => write!(f, "TCG"),
            TimeScale::TDB => write!(f, "TDB"),
            TimeScale::TT => write!(f, "TT"),
            TimeScale::UT1 => write!(f, "UT1"),
        }
    }
}

/// `ContinuousTime` is the base time representation for time scales without leap seconds. It is measured relative to an
/// arbitrary epoch. `RawTime::default()` represents the epoch itself.
///
/// `ContinuousTime` has attosecond precision, and supports times within 292 billion years either side of the epoch. The
/// sign of the time is determined by the sign of the `second` component.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct ContinuousTime {
    pub seconds: i64,
    pub attoseconds: u64,
}

impl Add<Delta> for ContinuousTime {
    type Output = Self;

    fn add(self, rhs: Delta) -> Self::Output {
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

impl Sub<Delta> for ContinuousTime {
    type Output = Self;

    fn sub(self, rhs: Delta) -> Self::Output {
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

/// An absolute continuous time difference with attosecond precision.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Delta {
    seconds: u64,
    attoseconds: u64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TAI(ContinuousTime);

impl TAI {
    pub fn to_ut1(&self, _dut: Delta, _dat: Delta) -> UT1 {
        todo!()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TCB(ContinuousTime);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TCG(ContinuousTime);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TDB(ContinuousTime);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TT(ContinuousTime);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UT1(ContinuousTime);

/// `Time` represents a time in any of the supported continuous timescales with attosecond precision.
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
    pub fn from_date_and_utc_timestamp(scale: TimeScale, date: Date, time: UTC) -> Self {
        let day_in_seconds = date.j2000() * SECONDS_PER_DAY - SECONDS_PER_DAY / 2;
        let hour_in_seconds = time.hour() * SECONDS_PER_HOUR;
        let minute_in_seconds = time.minute() * SECONDS_PER_MINUTE;
        let seconds = day_in_seconds + hour_in_seconds + minute_in_seconds + time.second();
        let attoseconds = time.subsecond_as_attoseconds();
        let raw = ContinuousTime {
            seconds,
            attoseconds,
        };
        Self::from_raw(scale, raw)
    }

    pub fn from_datetime(scale: TimeScale, dt: UTCDateTime) -> Self {
        Self::from_date_and_utc_timestamp(scale, dt.date(), dt.time())
    }

    /// Returns the J2000 epoch in the given timescale.
    pub fn j2000(scale: TimeScale) -> Self {
        Self::from_raw(scale, ContinuousTime::default())
    }

    /// Returns, as an epoch in the given timescale, midday on the first day of the proleptic Julian
    /// calendar.
    pub fn jd0(scale: TimeScale) -> Self {
        // This represents 4713 BC, since there is no year 0 separating BC and AD.
        let first_proleptic_day = Date::new_unchecked(ProlepticJulian, -4712, 1, 1);
        let midday = UTC::new(12, 0, 0).expect("midday should be a valid time");
        Self::from_date_and_utc_timestamp(scale, first_proleptic_day, midday)
    }

    fn from_raw(scale: TimeScale, raw_time: ContinuousTime) -> Self {
        match scale {
            TimeScale::TAI => Time::TAI(TAI(raw_time)),
            TimeScale::TCB => Time::TCB(TCB(raw_time)),
            TimeScale::TCG => Time::TCG(TCG(raw_time)),
            TimeScale::TDB => Time::TDB(TDB(raw_time)),
            TimeScale::TT => Time::TT(TT(raw_time)),
            TimeScale::UT1 => Time::UT1(UT1(raw_time)),
        }
    }

    fn raw(&self) -> ContinuousTime {
        match self {
            Time::TAI(tai) => tai.0,
            Time::TCB(tcb) => tcb.0,
            Time::TCG(tcg) => tcg.0,
            Time::TDB(tdb) => tdb.0,
            Time::TT(tt) => tt.0,
            Time::UT1(ut1) => ut1.0,
        }
    }

    pub fn seconds(&self) -> i64 {
        self.raw().seconds
    }

    pub fn attoseconds(&self) -> u64 {
        self.raw().attoseconds
    }

    pub fn days_since_j2000(&self) -> f64 {
        let d1 = self.seconds().to_f64().unwrap_or_default() / constants::f64::SECONDS_PER_DAY;
        let d2 = self.attoseconds().to_f64().unwrap() / constants::f64::ATTOSECONDS_PER_DAY;
        d2 + d1
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "foo")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_j2000() {
        [
            (
                TimeScale::TAI,
                Time::TAI(TAI(ContinuousTime {
                    seconds: 0,
                    attoseconds: 0,
                })),
            ),
            (
                TimeScale::TCB,
                Time::TCB(TCB(ContinuousTime {
                    seconds: 0,
                    attoseconds: 0,
                })),
            ),
            (
                TimeScale::TCG,
                Time::TCG(TCG(ContinuousTime {
                    seconds: 0,
                    attoseconds: 0,
                })),
            ),
            (
                TimeScale::TDB,
                Time::TDB(TDB(ContinuousTime {
                    seconds: 0,
                    attoseconds: 0,
                })),
            ),
            (
                TimeScale::TT,
                Time::TT(TT(ContinuousTime {
                    seconds: 0,
                    attoseconds: 0,
                })),
            ),
            (
                TimeScale::UT1,
                Time::UT1(UT1(ContinuousTime {
                    seconds: 0,
                    attoseconds: 0,
                })),
            ),
        ]
        .iter()
        .for_each(|(scale, expected)| {
            let actual = Time::j2000(*scale);
            assert_eq!(*expected, actual);
        });
    }

    #[test]
    fn test_epoch_jd0() {
        [
            (
                TimeScale::TAI,
                Time::TAI(TAI(ContinuousTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
            (
                TimeScale::TCB,
                Time::TCB(TCB(ContinuousTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
            (
                TimeScale::TCG,
                Time::TCG(TCG(ContinuousTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
            (
                TimeScale::TDB,
                Time::TDB(TDB(ContinuousTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
            (
                TimeScale::TT,
                Time::TT(TT(ContinuousTime {
                    seconds: -211813488000,
                    attoseconds: 0,
                })),
            ),
            (
                TimeScale::UT1,
                Time::UT1(UT1(ContinuousTime {
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
}
