/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use num::ToPrimitive;
use std::fmt;
use std::fmt::Formatter;

use crate::time::constants;
use crate::time::constants::i64::{SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE};

use crate::time::dates::Calendar::ProlepticJulian;
use crate::time::dates::{Date, DateTime, Time};

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

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
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

impl Epoch {
    pub fn from_date_and_time(scale: TimeScale, date: Date, time: Time) -> Self {
        let day_in_seconds = date.j2000() * SECONDS_PER_DAY - SECONDS_PER_DAY / 2;
        let hour_in_seconds = time.hour() * SECONDS_PER_HOUR;
        let minute_in_seconds = time.minute() * SECONDS_PER_MINUTE;
        let second = day_in_seconds + hour_in_seconds + minute_in_seconds + time.second();
        let attosecond = time.attosecond();
        let raw = RawEpoch { second, attosecond };
        Self::from_raw(scale, raw)
    }

    pub fn from_datetime(scale: TimeScale, dt: DateTime) -> Self {
        Self::from_date_and_time(scale, dt.date(), dt.time())
    }

    /// Returns the J2000 epoch in the given timescale.
    pub fn j2000(scale: TimeScale) -> Self {
        Self::from_raw(scale, RawEpoch::default())
    }

    /// Returns, as an epoch in the given timescale, midday on the first day of the proleptic Julian
    /// calendar.
    pub fn jd0(scale: TimeScale) -> Self {
        // This represents 4713 BC, since there is no year 0 separating BC and AD.
        let first_proleptic_day = Date::new_unchecked(ProlepticJulian, -4712, 1, 1);
        let midday = Time::new(12, 0, 0).expect("midday should be a valid time");
        Self::from_date_and_time(scale, first_proleptic_day, midday)
    }

    fn from_raw(scale: TimeScale, raw: RawEpoch) -> Self {
        match scale {
            TimeScale::TAI => Epoch::TAI(raw),
            TimeScale::TCB => Epoch::TCB(raw),
            TimeScale::TCG => Epoch::TCG(raw),
            TimeScale::TDB => Epoch::TDB(raw),
            TimeScale::TT => Epoch::TT(raw),
            TimeScale::UT1 => Epoch::UT1(raw),
        }
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

    pub fn days_since_j2000(&self) -> f64 {
        let d1 = self.second().to_f64().unwrap_or_default() / constants::f64::SECONDS_PER_DAY;
        let d2 = self.attosecond().to_f64().unwrap() / constants::f64::ATTOSECONDS_PER_DAY;
        d2 + d1
    }
}

impl fmt::Display for Epoch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "foo")
    }
}

#[cfg(test)]
mod tests {
    use crate::time::epochs::{Epoch, RawEpoch, TimeScale};

    #[test]
    fn test_epoch_j2000() {
        [
            (
                TimeScale::TAI,
                Epoch::TAI(RawEpoch {
                    second: 0,
                    attosecond: 0,
                }),
            ),
            (
                TimeScale::TCB,
                Epoch::TCB(RawEpoch {
                    second: 0,
                    attosecond: 0,
                }),
            ),
            (
                TimeScale::TCG,
                Epoch::TCG(RawEpoch {
                    second: 0,
                    attosecond: 0,
                }),
            ),
            (
                TimeScale::TDB,
                Epoch::TDB(RawEpoch {
                    second: 0,
                    attosecond: 0,
                }),
            ),
            (
                TimeScale::TT,
                Epoch::TT(RawEpoch {
                    second: 0,
                    attosecond: 0,
                }),
            ),
            (
                TimeScale::UT1,
                Epoch::UT1(RawEpoch {
                    second: 0,
                    attosecond: 0,
                }),
            ),
        ]
        .iter()
        .for_each(|(scale, expected)| {
            let actual = Epoch::j2000(*scale);
            assert_eq!(*expected, actual);
        });
    }

    #[test]
    fn test_epoch_jd0() {
        [
            (
                TimeScale::TAI,
                Epoch::TAI(RawEpoch {
                    second: -211813488000,
                    attosecond: 0,
                }),
            ),
            (
                TimeScale::TCB,
                Epoch::TCB(RawEpoch {
                    second: -211813488000,
                    attosecond: 0,
                }),
            ),
            (
                TimeScale::TCG,
                Epoch::TCG(RawEpoch {
                    second: -211813488000,
                    attosecond: 0,
                }),
            ),
            (
                TimeScale::TDB,
                Epoch::TDB(RawEpoch {
                    second: -211813488000,
                    attosecond: 0,
                }),
            ),
            (
                TimeScale::TT,
                Epoch::TT(RawEpoch {
                    second: -211813488000,
                    attosecond: 0,
                }),
            ),
            (
                TimeScale::UT1,
                Epoch::UT1(RawEpoch {
                    second: -211813488000,
                    attosecond: 0,
                }),
            ),
        ]
        .iter()
        .for_each(|(scale, expected)| {
            let actual = Epoch::jd0(*scale);
            assert_eq!(*expected, actual);
        });
    }
}
