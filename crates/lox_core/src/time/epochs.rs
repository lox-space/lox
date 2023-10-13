use std::fmt;
use std::fmt::Formatter;

use num::ToPrimitive;

use crate::time::dates::{Date, DateTime, Time};

const SECONDS_PER_DAY_F64: f64 = 24.0 * 60.0 * 60.0;

const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: i64 = 24 * SECONDS_PER_HOUR;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RawEpoch {
    second: i64,
    attosecond: i64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Epoch {
    TAI(RawEpoch),
    TCB(RawEpoch),
    TCG(RawEpoch),
    TDB(RawEpoch),
    TT(RawEpoch),
    UT1(RawEpoch),
}

impl fmt::Display for Epoch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "foo")
    }
}

impl Epoch {
    pub fn from_date_and_time(scale: TimeScale, date: Date, time: Time) -> Self {
        let day_in_seconds = date.j2000() * SECONDS_PER_DAY - SECONDS_PER_DAY / 2;
        let hour_in_seconds = time.hour() * SECONDS_PER_HOUR;
        let minute_in_seconds = time.minute() * SECONDS_PER_MINUTE;
        let second = day_in_seconds + hour_in_seconds + minute_in_seconds + time.second();
        let attosecond = time.attosecond();
        let raw = RawEpoch { second, attosecond };
        match scale {
            TimeScale::TAI => Epoch::TAI(raw),
            TimeScale::TCB => Epoch::TCB(raw),
            TimeScale::TCG => Epoch::TCG(raw),
            TimeScale::TDB => Epoch::TDB(raw),
            TimeScale::TT => Epoch::TT(raw),
            TimeScale::UT1 => Epoch::UT1(raw),
        }
    }

    pub fn from_datetime(scale: TimeScale, dt: DateTime) -> Self {
        Self::from_date_and_time(scale, dt.date(), dt.time())
    }

    fn raw(&self) -> &RawEpoch {
        match self {
            Epoch::TAI(raw)
            | Epoch::TCB(raw)
            | Epoch::TCG(raw)
            | Epoch::TDB(raw)
            | Epoch::TT(raw)
            | Epoch::UT1(raw) => raw,
        }
    }

    pub fn second(&self) -> i64 {
        self.raw().second
    }

    pub fn attosecond(&self) -> i64 {
        self.raw().attosecond
    }

    pub fn j2000(&self) -> f64 {
        let d1 = self.second().to_f64().unwrap_or_default() / SECONDS_PER_DAY_F64;
        let d2 = self.attosecond().to_f64().unwrap_or_default() / 1e18 / SECONDS_PER_DAY_F64;
        d2 + d1
    }
}
