// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::{
    cmp::{max, min},
    fmt::Display,
    ops::{Add, Sub},
};

use lox_core::time::deltas::TimeDelta;

use crate::{
    Time,
    offsets::{DefaultOffsetProvider, Offset},
    time_scales::{Tai, TimeScale},
    utc::{Utc, transformations::ToUtc},
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

    /// Returns an iterator of evenly-spaced points from start to end (inclusive)
    /// with the given step size.
    ///
    /// The step sign is automatically adjusted to match the interval direction:
    /// forward if `start <= end`, backward if `start > end`.
    ///
    /// # Panics
    ///
    /// Panics if `step` is zero.
    pub fn step_by(&self, step: TimeDelta) -> IntervalStepIter<T>
    where
        T: Copy + Add<TimeDelta, Output = T> + PartialOrd,
    {
        assert!(
            step.is_positive() || step.is_negative(),
            "step must be non-zero"
        );
        let forward = self.start <= self.end;
        let step = if forward == step.is_positive() {
            step
        } else {
            -step
        };
        IntervalStepIter {
            current: self.start,
            end: self.end,
            step,
            forward,
        }
    }

    /// Returns `n` evenly-spaced points from start to end (inclusive).
    ///
    /// Panics if `n < 2`.
    pub fn linspace(&self, n: usize) -> Vec<T>
    where
        T: Copy + Add<TimeDelta, Output = T> + Sub<Output = TimeDelta>,
    {
        assert!(n >= 2, "linspace requires at least 2 points");
        let duration = self.end - self.start;
        let step_secs = duration.to_seconds().to_f64() / (n - 1) as f64;
        (0..n)
            .map(|i| self.start + TimeDelta::from_seconds_f64(step_secs * i as f64))
            .collect()
    }
}

pub struct IntervalStepIter<T> {
    current: T,
    end: T,
    step: TimeDelta,
    forward: bool,
}

impl<T> Iterator for IntervalStepIter<T>
where
    T: Copy + Add<TimeDelta, Output = T> + PartialOrd,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let done = if self.forward {
            self.current > self.end
        } else {
            self.current < self.end
        };
        if done {
            return None;
        }
        let value = self.current;
        self.current = self.current + self.step;
        Some(value)
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
    T: ToUtc + TimeScale + Copy,
    DefaultOffsetProvider: Offset<T, Tai>,
{
    pub fn to_utc(&self) -> UtcInterval {
        Interval {
            start: self.start.to_utc(),
            end: self.end.to_utc(),
        }
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

    #[test]
    fn test_step_by() {
        let t0 = time!(Tai, 2025, 11, 6).unwrap();
        let t1 = time!(Tai, 2025, 11, 6, 1).unwrap();
        let interval = TimeInterval::new(t0, t1);
        let step = TimeDelta::from_minutes(20.0);
        let times: Vec<_> = interval.step_by(step).collect();
        assert_eq!(times.len(), 4); // 0, 20, 40, 60 minutes
        assert_eq!(times[0], t0);
        assert_eq!(times[3], t1);
    }

    #[test]
    fn test_step_by_non_exact() {
        let t0 = time!(Tai, 2025, 11, 6).unwrap();
        let t1 = t0 + TimeDelta::from_minutes(50.0);
        let interval = TimeInterval::new(t0, t1);
        let step = TimeDelta::from_minutes(20.0);
        let times: Vec<_> = interval.step_by(step).collect();
        assert_eq!(times.len(), 3); // 0, 20, 40 minutes (60 exceeds end)
    }

    #[test]
    fn test_linspace() {
        let t0 = time!(Tai, 2025, 11, 6).unwrap();
        let t1 = time!(Tai, 2025, 11, 6, 1).unwrap();
        let interval = TimeInterval::new(t0, t1);
        let times = interval.linspace(5);
        assert_eq!(times.len(), 5);
        assert_eq!(times[0], t0);
        assert_eq!(times[4], t1);
        // Equal spacing: 15 minutes apart
        let dt = TimeDelta::from_minutes(15.0);
        assert_eq!(times[1], t0 + dt);
        assert_eq!(times[2], t0 + dt + dt);
    }

    #[test]
    fn test_timedelta_interval_step_by() {
        let td0 = TimeDelta::default();
        let td1 = TimeDelta::from_minutes(60.0);
        let interval = TimeDeltaInterval::new(td0, td1);
        let step = TimeDelta::from_minutes(20.0);
        let times: Vec<_> = interval.step_by(step).collect();
        assert_eq!(times.len(), 4);
    }

    #[test]
    fn test_step_by_backward() {
        let t0 = time!(Tai, 2025, 11, 6).unwrap();
        let t1 = time!(Tai, 2025, 11, 6, 1).unwrap();
        // Interval goes backward: start > end
        let interval = TimeInterval::new(t1, t0);
        let step = TimeDelta::from_minutes(20.0);
        let times: Vec<_> = interval.step_by(step).collect();
        assert_eq!(times.len(), 4); // 60, 40, 20, 0 minutes
        assert_eq!(times[0], t1);
        assert_eq!(times[3], t0);
        // Monotonically decreasing
        for w in times.windows(2) {
            assert!(w[0] > w[1]);
        }
    }

    #[test]
    fn test_step_by_backward_auto_negates_step() {
        let t0 = time!(Tai, 2025, 11, 6).unwrap();
        let t1 = time!(Tai, 2025, 11, 6, 1).unwrap();
        // Backward interval with an already-negative step — should still work
        let interval = TimeInterval::new(t1, t0);
        let step = -TimeDelta::from_minutes(20.0);
        let times: Vec<_> = interval.step_by(step).collect();
        assert_eq!(times.len(), 4);
        assert_eq!(times[0], t1);
        assert_eq!(times[3], t0);
    }

    #[test]
    #[should_panic(expected = "step must be non-zero")]
    fn test_step_by_zero_panics() {
        let t0 = time!(Tai, 2025, 11, 6).unwrap();
        let t1 = time!(Tai, 2025, 11, 6, 1).unwrap();
        let interval = TimeInterval::new(t0, t1);
        let _ = interval.step_by(TimeDelta::default()).collect::<Vec<_>>();
    }
}
