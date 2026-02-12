// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::{
    cmp::{max, min},
    fmt::Display,
    ops::Sub,
};

use lox_core::time::deltas::TimeDelta;

use crate::{
    Time,
    offsets::{DefaultOffsetProvider, TryOffset},
    time_scales::{Tai, TimeScale},
    utc::{Utc, UtcError, transformations::TryToUtc},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Interval<T> {
    start: T,
    end: T,
}

impl<T> Interval<T> {
    pub fn new(start: T, end: T) -> Self {
        Interval { start, end }
    }

    pub fn start(&self) -> T
    where
        T: Copy,
    {
        self.start
    }

    pub fn end(&self) -> T
    where
        T: Copy,
    {
        self.end
    }

    pub fn duration(&self) -> TimeDelta
    where
        T: Sub<Output = TimeDelta> + Copy,
    {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool
    where
        T: Ord,
    {
        self.start >= self.end
    }

    pub fn contains_time(&self, time: T) -> bool
    where
        T: Ord,
    {
        self.start <= time && time < self.end
    }

    pub fn intersect(&self, other: Self) -> Self
    where
        T: Ord + Copy,
    {
        Interval {
            start: max(self.start, other.start),
            end: min(self.end, other.end),
        }
    }

    pub fn overlaps(&self, other: Self) -> bool
    where
        T: Ord + Copy,
    {
        !self.intersect(other).is_empty()
    }
}

pub type TimeDeltaInterval = Interval<TimeDelta>;

impl TimeDeltaInterval {
    pub fn to_scale<T: TimeScale + Copy>(&self, scale: T) -> TimeInterval<T> {
        Interval {
            start: Time::from_delta(scale, self.start),
            end: Time::from_delta(scale, self.end),
        }
    }
}

pub type TimeInterval<T> = Interval<Time<T>>;

impl<T> TimeInterval<T>
where
    T: TryToUtc + TimeScale + Copy,
    DefaultOffsetProvider: TryOffset<T, Tai>,
{
    pub fn to_utc(&self) -> Result<UtcInterval, UtcError> {
        Ok(Interval {
            start: self.start.try_to_utc()?,
            end: self.end.try_to_utc()?,
        })
    }
}

pub type UtcInterval = Interval<Utc>;

impl UtcInterval {
    pub fn to_time(&self) -> TimeInterval<Tai> {
        Interval {
            start: self.start.to_time(),
            end: self.end.to_time(),
        }
    }
}

impl<T> Display for Interval<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.start.fmt(f)?;
        write!(f, " – ")?;
        self.end.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::{time, time_scales::Tai};

    use super::*;

    #[test]
    fn test_time_interval() {
        let t0 = time!(Tai, 2025, 11, 6).unwrap();
        let t1 = time!(Tai, 2025, 11, 6, 1).unwrap();
        let i = TimeInterval::new(t0, t1);
        assert_eq!(i.start(), t0);
        assert_eq!(i.end(), t1);
        assert_eq!(i.duration(), TimeDelta::from_hours(1.0));
        assert_eq!(
            format!("{}", i),
            "2025-11-06T00:00:00.000 TAI – 2025-11-06T01:00:00.000 TAI"
        );
    }
}
