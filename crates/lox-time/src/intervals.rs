// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use alloc::vec::Vec;
use core::{
    cmp::{max, min},
    fmt::Display,
    ops::{Add, Sub},
};

use lox_approx::ApproxEq;
use lox_approx::ApproxEqResults;
use lox_core::time::deltas::TimeDelta;

use crate::{Time, time::TimeScaleMismatch, time_scales::ContinuousTimeScale};

/// A half-open interval `[start, start + duration)`.
///
/// Stored as an epoch plus a [`TimeDelta`] rather than a pair of bounds, so an
/// interval cannot straddle two time scales: the scale lives on the epoch alone.
/// A negative duration denotes a backwards interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Interval<T> {
    epoch: T,
    duration: TimeDelta,
}

impl<T> ApproxEq for Interval<T>
where
    T: ApproxEq + core::fmt::Debug + Copy + Add<TimeDelta, Output = T>,
{
    /// Compares the observable bounds rather than the stored epoch and duration,
    /// so the comparison does not depend on the internal representation.
    fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> ApproxEqResults {
        let mut results = ApproxEqResults::new();
        results.merge("start", self.start().approx_eq(&rhs.start(), atol, rtol));
        results.merge("end", self.end().approx_eq(&rhs.end(), atol, rtol));
        results
    }
}

impl<T> Interval<T> {
    /// Creates an interval from an epoch and a duration.
    pub fn from_duration(epoch: T, duration: TimeDelta) -> Self {
        Interval { epoch, duration }
    }

    /// Returns the start of the interval.
    pub fn start(&self) -> T
    where
        T: Copy,
    {
        self.epoch
    }

    /// Returns the duration of the interval.
    pub fn duration(&self) -> TimeDelta {
        self.duration
    }

    /// Returns `true` if the interval is empty (its duration is not positive).
    pub fn is_empty(&self) -> bool {
        !self.duration.is_positive()
    }
}

impl<T> Interval<T>
where
    T: Copy + Sub<Output = TimeDelta>,
{
    /// Creates a new interval from `start` to `end`.
    ///
    /// # Panics
    ///
    /// For [`TimeInterval`], panics if the bounds are in different time scales,
    /// since the duration is derived by subtracting them. Use
    /// [`TimeInterval::try_new`] to handle that case.
    pub fn new(start: T, end: T) -> Self {
        Interval {
            epoch: start,
            duration: end - start,
        }
    }
}

impl<T> Interval<T>
where
    T: Copy + Add<TimeDelta, Output = T>,
{
    /// Returns the end of the interval.
    pub fn end(&self) -> T {
        self.epoch + self.duration
    }
}

impl<T> Interval<T>
where
    T: Copy + Ord + Add<TimeDelta, Output = T> + Sub<Output = TimeDelta>,
{
    /// Returns `true` if `time` falls within `[start, end)`.
    pub fn contains_time(&self, time: T) -> bool {
        self.start() <= time && time < self.end()
    }

    /// Returns the intersection of `self` and `other`.
    pub fn intersect(&self, other: Self) -> Self {
        Interval::new(
            max(self.start(), other.start()),
            min(self.end(), other.end()),
        )
    }

    /// Returns `true` if `self` and `other` overlap.
    pub fn overlaps(&self, other: Self) -> bool {
        !self.intersect(other).is_empty()
    }

    /// True if self fully contains other.
    pub fn contains(&self, other: &Self) -> bool {
        self.start() <= other.start() && self.end() >= other.end()
    }
}

impl<T> Interval<T>
where
    T: Copy + Add<TimeDelta, Output = T> + PartialOrd,
{
    /// Returns an iterator of evenly-spaced points from start to end (inclusive)
    /// with the given step size.
    ///
    /// The step sign is automatically adjusted to match the interval direction:
    /// forward for a positive duration, backward for a negative one.
    ///
    /// # Panics
    ///
    /// Panics if `step` is zero.
    pub fn step_by(&self, step: TimeDelta) -> IntervalStepIter<T> {
        assert!(
            step.is_positive() || step.is_negative(),
            "step must be non-zero"
        );
        let forward = !self.duration.is_negative();
        let step = if forward == step.is_positive() {
            step
        } else {
            -step
        };
        IntervalStepIter {
            current: self.epoch,
            end: self.end(),
            step,
            forward,
        }
    }

    /// Returns `n` evenly-spaced points from start to end (inclusive).
    ///
    /// Panics if `n < 2`.
    pub fn linspace(&self, n: usize) -> Vec<T> {
        assert!(n >= 2, "linspace requires at least 2 points");
        let step_secs = self.duration.to_seconds().to_f64() / (n - 1) as f64;
        (0..n)
            .map(|i| self.epoch + TimeDelta::from_seconds_f64(step_secs * i as f64))
            .collect()
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
pub fn intersect_intervals<T: Copy + Ord + Add<TimeDelta, Output = T> + Sub<Output = TimeDelta>>(
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
        if a[i].end() <= b[j].end() {
            i += 1;
        } else {
            j += 1;
        }
    }
    result
}

/// Union two sorted lists of intervals (merge overlapping/adjacent).
pub fn union_intervals<T: Copy + Ord + Add<TimeDelta, Output = T> + Sub<Output = TimeDelta>>(
    a: &[Interval<T>],
    b: &[Interval<T>],
) -> Vec<Interval<T>> {
    // Merge the two sorted lists
    let mut all = Vec::with_capacity(a.len() + b.len());
    let mut i = 0;
    let mut j = 0;
    while i < a.len() && j < b.len() {
        if a[i].start() <= b[j].start() {
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
pub fn complement_intervals<
    T: Copy + Ord + Add<TimeDelta, Output = T> + Sub<Output = TimeDelta>,
>(
    intervals: &[Interval<T>],
    bound: Interval<T>,
) -> Vec<Interval<T>> {
    let mut result = Vec::new();
    let mut cursor = bound.start();
    for iv in intervals {
        if iv.start() > cursor {
            let gap = Interval::new(cursor, iv.start());
            if !gap.is_empty() {
                result.push(gap);
            }
        }
        if iv.end() > cursor {
            cursor = iv.end();
        }
    }
    if cursor < bound.end() {
        result.push(Interval::new(cursor, bound.end()));
    }
    result
}

fn merge_intervals<T: Copy + Ord + Add<TimeDelta, Output = T> + Sub<Output = TimeDelta>>(
    sorted: Vec<Interval<T>>,
) -> Vec<Interval<T>> {
    let mut result: Vec<Interval<T>> = Vec::new();
    for iv in sorted {
        if iv.is_empty() {
            continue;
        }
        if let Some(last) = result.last_mut()
            && iv.start() <= last.end()
        {
            // Extend in place: the epoch is unchanged, only the duration grows.
            let end = max(last.end(), iv.end());
            *last = Interval::new(last.start(), end);
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
    pub fn to_scale<T: ContinuousTimeScale + Copy>(&self, scale: T) -> TimeInterval<T> {
        Interval::from_duration(Time::from_delta(scale, self.start()), self.duration())
    }
}

/// An interval of [`Time`] values in a given time scale.
///
/// The scale defaults to the runtime-determined [`TimeScale`](crate::time_scales::TimeScale).
pub type TimeInterval<T = crate::time_scales::TimeScale> = Interval<Time<T>>;

impl<T> TimeInterval<T>
where
    T: ContinuousTimeScale + Copy + Into<crate::time_scales::TimeScale>,
{
    /// Converts this interval into one whose bounds carry their time scale at runtime.
    pub fn into_dynamic(self) -> TimeInterval {
        Interval::from_duration(self.epoch.into_dynamic(), self.duration)
    }
}

impl TimeInterval {
    /// Creates a time interval from a pair of bounds, returning an error if they
    /// are in different time scales.
    ///
    /// [`Interval::new`] panics in that case, because it derives the duration by
    /// subtracting the bounds. Prefer this constructor when they come from
    /// external input.
    pub fn try_new(start: Time, end: Time) -> Result<Self, TimeScaleMismatch> {
        let duration = start.checked_sub(&end).map(|d| -d)?;
        Ok(Interval::from_duration(start, duration))
    }
}

impl<T> Display for Interval<T>
where
    T: Display + Copy + Add<TimeDelta, Output = T>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.epoch.fmt(f)?;
        write!(f, " – ")?;
        self.end().fmt(f)
    }
}

#[cfg(test)]
mod tests {

    /// Interval over `TimeDelta` seconds, for the pure interval-algebra tests.
    fn iv(start: i64, end: i64) -> TimeDeltaInterval {
        Interval::new(TimeDelta::from_seconds(start), TimeDelta::from_seconds(end))
    }
    use alloc::format;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::{
        time,
        time_scales::{Tai, Tdb, Tt},
    };

    use super::*;

    #[test]
    fn test_try_new_same_scale() {
        let t0 = time!(Tai, 2025, 11, 6).unwrap().into_dynamic();
        let t1 = time!(Tai, 2025, 11, 7).unwrap().into_dynamic();
        let iv = TimeInterval::try_new(t0, t1).expect("same scale");
        assert_eq!(iv.duration(), TimeDelta::from_days(1));
    }

    #[test]
    fn test_try_new_different_scale_returns_err() {
        let t0 = time!(Tai, 2025, 11, 6).unwrap().into_dynamic();
        let t1 = time!(Tt, 2025, 11, 7).unwrap().into_dynamic();
        assert!(TimeInterval::try_new(t0, t1).is_err());
    }

    #[test]
    #[should_panic(expected = "cannot subtract `Time` objects with different time scales")]
    fn test_mismatched_bounds_panic_at_construction() {
        let t0 = time!(Tai, 2025, 11, 6).unwrap().into_dynamic();
        let t1 = time!(Tt, 2025, 11, 7).unwrap().into_dynamic();
        // The duration is derived by subtracting the bounds, so the mismatch is
        // caught here rather than on first use.
        let _ = TimeInterval::new(t0, t1);
    }

    #[test]
    fn test_interval_carries_a_single_scale() {
        // An interval holds one epoch, so it cannot straddle two scales at all.
        let epoch = time!(Tdb, 2025, 11, 6).unwrap().into_dynamic();
        let iv = TimeInterval::from_duration(epoch, TimeDelta::from_hours(2));
        assert_eq!(iv.start().scale(), iv.end().scale());
        assert_eq!(iv.duration(), TimeDelta::from_hours(2));
    }

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
        let outer = iv(0, 10);
        let inner = iv(2, 8);
        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn test_intersect_intervals() {
        let a = vec![iv(0, 5), iv(10, 15)];
        let b = vec![iv(3, 12)];
        let result = intersect_intervals(&a, &b);
        assert_eq!(result, vec![iv(3, 5), iv(10, 12)]);
    }

    #[test]
    fn test_intersect_intervals_no_overlap() {
        let a = vec![iv(0, 3)];
        let b = vec![iv(5, 8)];
        let result = intersect_intervals(&a, &b);
        assert!(result.is_empty());
    }

    #[test]
    fn test_union_intervals() {
        let a = vec![iv(0, 5)];
        let b = vec![iv(3, 8)];
        let result = union_intervals(&a, &b);
        assert_eq!(result, vec![iv(0, 8)]);
    }

    #[test]
    fn test_union_intervals_disjoint() {
        let a = vec![iv(0, 3)];
        let b = vec![iv(5, 8)];
        let result = union_intervals(&a, &b);
        assert_eq!(result, vec![iv(0, 3), iv(5, 8)]);
    }

    #[test]
    fn test_complement_intervals() {
        let intervals = vec![iv(2, 4), iv(6, 8)];
        let bound = iv(0, 10);
        let result = complement_intervals(&intervals, bound);
        assert_eq!(result, vec![iv(0, 2), iv(4, 6), iv(8, 10),]);
    }

    #[test]
    fn test_complement_intervals_full_coverage() {
        let intervals = vec![iv(0, 10)];
        let bound = iv(0, 10);
        let result = complement_intervals(&intervals, bound);
        assert!(result.is_empty());
    }
}
