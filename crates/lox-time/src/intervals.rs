// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::{
    cmp::{max, min},
    fmt::Display,
    ops::{Add, Sub},
};

use lox_core::time::deltas::TimeDelta;
use lox_test_utils::approx_eq::ApproxEq;
use lox_test_utils::approx_eq::results::ApproxEqResults;

use crate::{
    Time,
    offsets::{DefaultOffsetProvider, Offset},
    time_scales::{Tai, TimeScale},
    utc::{Utc, transformations::ToUtc},
};

/// A half-open interval `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Interval<T> {
    start: T,
    end: T,
}

impl<T: ApproxEq + std::fmt::Debug> ApproxEq for Interval<T> {
    fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> ApproxEqResults {
        let mut results = ApproxEqResults::new();
        results.merge("start", self.start.approx_eq(&rhs.start, atol, rtol));
        results.merge("end", self.end.approx_eq(&rhs.end, atol, rtol));
        results
    }
}

impl<T> Interval<T> {
    /// Creates a new interval from `start` to `end`.
    pub fn new(start: T, end: T) -> Self {
        Interval { start, end }
    }

    /// Returns the start of the interval.
    pub fn start(&self) -> T
    where
        T: Copy,
    {
        self.start
    }

    /// Returns the end of the interval.
    pub fn end(&self) -> T
    where
        T: Copy,
    {
        self.end
    }

    /// Returns the duration of the interval as a [`TimeDelta`].
    pub fn duration(&self) -> TimeDelta
    where
        T: Sub<Output = TimeDelta> + Copy,
    {
        self.end - self.start
    }

    /// Returns `true` if the interval is empty (`start >= end`).
    pub fn is_empty(&self) -> bool
    where
        T: Ord,
    {
        self.start >= self.end
    }

    /// Returns `true` if `time` falls within `[start, end)`.
    pub fn contains_time(&self, time: T) -> bool
    where
        T: Ord,
    {
        self.start <= time && time < self.end
    }

    /// Returns the intersection of `self` and `other`.
    pub fn intersect(&self, other: Self) -> Self
    where
        T: Ord + Copy,
    {
        Interval {
            start: max(self.start, other.start),
            end: min(self.end, other.end),
        }
    }

    /// Returns `true` if `self` and `other` overlap.
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

    /// True if self fully contains other.
    pub fn contains(&self, other: &Self) -> bool
    where
        T: Ord,
    {
        self.start <= other.start && self.end >= other.end
    }
}

/// Iterator that steps through an interval with a fixed time step.
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

/// Intersect two sorted lists of intervals.
pub fn intersect_intervals<T: Ord + Copy>(
    a: &[Interval<T>],
    b: &[Interval<T>],
) -> Vec<Interval<T>> {
    let mut result = Vec::new();
    let mut i = 0;
    let mut j = 0;
    while i < a.len() && j < b.len() {
        let inter = a[i].intersect(b[j]);
        if !inter.is_empty() {
            result.push(inter);
        }
        // Advance the interval with the smaller end
        if a[i].end <= b[j].end {
            i += 1;
        } else {
            j += 1;
        }
    }
    result
}

/// Union two sorted lists of intervals (merge overlapping/adjacent).
pub fn union_intervals<T: Ord + Copy>(a: &[Interval<T>], b: &[Interval<T>]) -> Vec<Interval<T>> {
    // Merge the two sorted lists
    let mut all = Vec::with_capacity(a.len() + b.len());
    let mut i = 0;
    let mut j = 0;
    while i < a.len() && j < b.len() {
        if a[i].start <= b[j].start {
            all.push(a[i]);
            i += 1;
        } else {
            all.push(b[j]);
            j += 1;
        }
    }
    all.extend_from_slice(&a[i..]);
    all.extend_from_slice(&b[j..]);

    merge_intervals(all)
}

/// Complement intervals within a bounding interval.
pub fn complement_intervals<T: Ord + Copy>(
    intervals: &[Interval<T>],
    bound: Interval<T>,
) -> Vec<Interval<T>> {
    let mut result = Vec::new();
    let mut cursor = bound.start;
    for iv in intervals {
        if iv.start > cursor {
            let gap = Interval::new(cursor, iv.start);
            if !gap.is_empty() {
                result.push(gap);
            }
        }
        if iv.end > cursor {
            cursor = iv.end;
        }
    }
    if cursor < bound.end {
        result.push(Interval::new(cursor, bound.end));
    }
    result
}

fn merge_intervals<T: Ord + Copy>(sorted: Vec<Interval<T>>) -> Vec<Interval<T>> {
    let mut result: Vec<Interval<T>> = Vec::new();
    for iv in sorted {
        if iv.is_empty() {
            continue;
        }
        if let Some(last) = result.last_mut()
            && iv.start <= last.end
        {
            last.end = max(last.end, iv.end);
            continue;
        }
        result.push(iv);
    }
    result
}

/// An interval of [`TimeDelta`] values.
pub type TimeDeltaInterval = Interval<TimeDelta>;

impl TimeDeltaInterval {
    /// Converts this delta-based interval to a [`TimeInterval`] in the given time scale.
    pub fn to_scale<T: TimeScale + Copy>(&self, scale: T) -> TimeInterval<T> {
        Interval {
            start: Time::from_delta(scale, self.start),
            end: Time::from_delta(scale, self.end),
        }
    }
}

/// An interval of [`Time`] values in a given time scale.
pub type TimeInterval<T> = Interval<Time<T>>;

impl<T> TimeInterval<T>
where
    T: ToUtc + TimeScale + Copy,
    DefaultOffsetProvider: Offset<T, Tai>,
{
    /// Converts this time interval to a [`UtcInterval`].
    pub fn to_utc(&self) -> UtcInterval {
        Interval {
            start: self.start.to_utc(),
            end: self.end.to_utc(),
        }
    }
}

/// An interval of [`Utc`] values.
pub type UtcInterval = Interval<Utc>;

impl UtcInterval {
    /// Converts this UTC interval to a [`TimeInterval`] in TAI.
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
        assert_eq!(i.duration(), TimeDelta::from_hours(1));
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
        let step = TimeDelta::from_minutes(20);
        let times: Vec<_> = interval.step_by(step).collect();
        assert_eq!(times.len(), 4); // 0, 20, 40, 60 minutes
        assert_eq!(times[0], t0);
        assert_eq!(times[3], t1);
    }

    #[test]
    fn test_step_by_non_exact() {
        let t0 = time!(Tai, 2025, 11, 6).unwrap();
        let t1 = t0 + TimeDelta::from_minutes(50);
        let interval = TimeInterval::new(t0, t1);
        let step = TimeDelta::from_minutes(20);
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
        let dt = TimeDelta::from_minutes(15);
        assert_eq!(times[1], t0 + dt);
        assert_eq!(times[2], t0 + dt + dt);
    }

    #[test]
    fn test_timedelta_interval_step_by() {
        let td0 = TimeDelta::default();
        let td1 = TimeDelta::from_minutes(60);
        let interval = TimeDeltaInterval::new(td0, td1);
        let step = TimeDelta::from_minutes(20);
        let times: Vec<_> = interval.step_by(step).collect();
        assert_eq!(times.len(), 4);
    }

    #[test]
    fn test_step_by_backward() {
        let t0 = time!(Tai, 2025, 11, 6).unwrap();
        let t1 = time!(Tai, 2025, 11, 6, 1).unwrap();
        // Interval goes backward: start > end
        let interval = TimeInterval::new(t1, t0);
        let step = TimeDelta::from_minutes(20);
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
        let step = -TimeDelta::from_minutes(20);
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

    #[test]
    fn test_contains() {
        let outer = Interval::new(0, 10);
        let inner = Interval::new(2, 8);
        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn test_intersect_intervals() {
        let a = vec![Interval::new(0, 5), Interval::new(10, 15)];
        let b = vec![Interval::new(3, 12)];
        let result = intersect_intervals(&a, &b);
        assert_eq!(result, vec![Interval::new(3, 5), Interval::new(10, 12)]);
    }

    #[test]
    fn test_intersect_intervals_no_overlap() {
        let a = vec![Interval::new(0, 3)];
        let b = vec![Interval::new(5, 8)];
        let result = intersect_intervals(&a, &b);
        assert!(result.is_empty());
    }

    #[test]
    fn test_union_intervals() {
        let a = vec![Interval::new(0, 5)];
        let b = vec![Interval::new(3, 8)];
        let result = union_intervals(&a, &b);
        assert_eq!(result, vec![Interval::new(0, 8)]);
    }

    #[test]
    fn test_union_intervals_disjoint() {
        let a = vec![Interval::new(0, 3)];
        let b = vec![Interval::new(5, 8)];
        let result = union_intervals(&a, &b);
        assert_eq!(result, vec![Interval::new(0, 3), Interval::new(5, 8)]);
    }

    #[test]
    fn test_complement_intervals() {
        let intervals = vec![Interval::new(2, 4), Interval::new(6, 8)];
        let bound = Interval::new(0, 10);
        let result = complement_intervals(&intervals, bound);
        assert_eq!(
            result,
            vec![
                Interval::new(0, 2),
                Interval::new(4, 6),
                Interval::new(8, 10),
            ]
        );
    }

    #[test]
    fn test_complement_intervals_full_coverage() {
        let intervals = vec![Interval::new(0, 10)];
        let bound = Interval::new(0, 10);
        let result = complement_intervals(&intervals, bound);
        assert!(result.is_empty());
    }
}
