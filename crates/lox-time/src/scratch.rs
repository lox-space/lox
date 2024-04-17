/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::errors::LoxTimeError;
use crate::subsecond::Subsecond;
use crate::time_scales::TimeScale;
use crate::Time;
use thiserror::Error;

pub trait CivilTime {
    fn hour(&self) -> i64;
    fn minute(&self) -> i64;
    fn second(&self) -> i64;
    fn subsecond(&self) -> Subsecond;
}

/// ContinuousTime represents a civil clock without leap seconds.
pub trait ContinuousTime: CivilTime {}

pub trait CalendarDate {
    fn year(&self) -> i64;
    fn month(&self) -> i64;
    fn day(&self) -> i64;
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Utc {
    hour: u8,
    minute: u8,
    second: u8,
    subsecond: Subsecond,
}

/// The Utc constructor allows for leap seconds, while the Time constructor does not.
impl Utc {
    pub fn new(
        hour: u8,
        minute: u8,
        second: u8,
        subsecond: Subsecond,
    ) -> Result<Self, LoxTimeError> {
        if !(0..24).contains(&hour) || !(0..60).contains(&minute) || !(0..61).contains(&second) {
            Err(LoxTimeError::InvalidTime(hour, minute, second))
        } else {
            Ok(Self {
                hour,
                minute,
                second,
                subsecond,
            })
        }
    }
}

impl CivilTime for Utc {
    fn hour(&self) -> i64 {
        self.hour as i64
    }

    fn minute(&self) -> i64 {
        self.minute as i64
    }

    fn second(&self) -> i64 {
        self.second as i64
    }

    fn subsecond(&self) -> Subsecond {
        self.subsecond
    }
}

impl<T: TimeScale + Copy> CivilTime for Time<T> {
    fn hour(&self) -> i64 {
        todo!()
    }

    fn minute(&self) -> i64 {
        todo!()
    }

    fn second(&self) -> i64 {
        todo!()
    }

    fn subsecond(&self) -> Subsecond {
        todo!()
    }
}

/// Blanket implementation for all `Time` types. Library users don't have to think about this.
impl<T: TimeScale + Copy> ContinuousTime for Time<T> {}

/// DateTime uses a generic date, since a `Time` struct represent both a date and a time, already
/// implements `CalendarDate`, and we probably don't want to force users to create a new `Date`
/// instance just to create a `DateTime`.
pub struct DateTime<D: CalendarDate, T: CivilTime> {
    date: D,
    time: T,
}

/// No leap seconds to worry about – just a simple date and time.
impl<D, T> DateTime<D, T>
where
    D: CalendarDate,
    T: ContinuousTime,
{
    pub fn new(date: D, time: T) -> Self {
        Self { date, time }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
#[error("Non-leap second year")]
pub struct NonLeapSecondYearError;

/// This constructor and the associated error scenarios is exposed only for Utc.
impl<D> DateTime<D, Utc>
where
    D: CalendarDate,
{
    pub fn new(date: D, time: Utc) -> Result<Self, NonLeapSecondYearError> {
        // Validate that, if the time is a leap second, the date is a leap second year.
        todo!()
    }
}

/// Shared implementation for all `DateTime` types.
impl<D, T> DateTime<D, T>
where
    D: CalendarDate,
    T: CivilTime,
{
    pub fn date(&self) -> &D {
        &self.date
    }

    pub fn time(&self) -> &T {
        &self.time
    }
}
