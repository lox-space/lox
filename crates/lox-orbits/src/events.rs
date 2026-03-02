// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::VecDeque;
use std::fmt::Display;

use itertools::Itertools;
use lox_math::roots::{BoxedError, Callback, CallbackError, FindBracketedRoot, RootFinderError};
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::TimeScale;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Core event types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ZeroCrossing {
    Up,
    Down,
}

impl ZeroCrossing {
    fn new(s0: f64, s1: f64) -> Option<ZeroCrossing> {
        if s0 < 0.0 && s1 > 0.0 {
            Some(ZeroCrossing::Up)
        } else if s0 > 0.0 && s1 < 0.0 {
            Some(ZeroCrossing::Down)
        } else {
            None
        }
    }
}

impl Display for ZeroCrossing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZeroCrossing::Up => write!(f, "up"),
            ZeroCrossing::Down => write!(f, "down"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Event<T: TimeScale> {
    crossing: ZeroCrossing,
    time: Time<T>,
}

impl<T: TimeScale> Event<T> {
    pub fn new(time: Time<T>, crossing: ZeroCrossing) -> Self {
        Self { crossing, time }
    }

    pub fn time(&self) -> Time<T>
    where
        T: Copy,
    {
        self.time
    }

    pub fn crossing(&self) -> ZeroCrossing {
        self.crossing
    }
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum DetectError {
    #[error(transparent)]
    RootFinder(#[from] RootFinderError),
    #[error(transparent)]
    Callback(Box<dyn std::error::Error + Send + Sync>),
}

// ---------------------------------------------------------------------------
// Core traits
// ---------------------------------------------------------------------------

/// Scalar function whose zero-crossings define events.
pub trait DetectFn<T: TimeScale> {
    type Error: std::error::Error + Send + Sync + 'static;
    fn eval(&self, time: Time<T>) -> Result<f64, Self::Error>;
}

/// Detects instantaneous events (zero-crossings) within a time interval.
pub trait EventDetector<T: TimeScale> {
    fn detect(&self, interval: TimeInterval<T>) -> Result<Vec<Event<T>>, DetectError>;
}

/// Detects intervals where a condition holds within a time interval.
pub trait IntervalDetector<T: TimeScale> {
    fn detect(&self, interval: TimeInterval<T>) -> Result<Vec<TimeInterval<T>>, DetectError>;
}

// ---------------------------------------------------------------------------
// Callback wrapper for DetectFn → root finder bridge
// ---------------------------------------------------------------------------

/// A `Callback`-compatible wrapper that bridges `DetectFn` to the root-finder
/// interface.
pub(crate) struct DetectCallback<'a, T: TimeScale, F: DetectFn<T>> {
    func: &'a F,
    start: Time<T>,
}

impl<T: TimeScale + Copy, F: DetectFn<T>> Clone for DetectCallback<'_, T, F> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: TimeScale + Copy, F: DetectFn<T>> Copy for DetectCallback<'_, T, F> {}

impl<'a, T: TimeScale + Copy, F: DetectFn<T>> DetectCallback<'a, T, F> {
    fn new(func: &'a F, start: Time<T>) -> Self {
        Self { func, start }
    }
}

impl<T: TimeScale + Copy, F: DetectFn<T>> Callback for DetectCallback<'_, T, F> {
    fn call(&self, v: f64) -> Result<f64, CallbackError> {
        let time = self.start + TimeDelta::from_seconds_f64(v);
        self.func
            .eval(time)
            .map_err(|e| CallbackError::from(Box::new(e) as BoxedError))
    }
}

// ---------------------------------------------------------------------------
// RootFindingDetector — wraps DetectFn + root finder → EventDetector
// ---------------------------------------------------------------------------

use lox_math::roots::Brent;

/// Wraps a `DetectFn` with a root finder to produce an `EventDetector`.
pub struct RootFindingDetector<F, R = Brent> {
    pub(crate) func: F,
    root_finder: R,
    step: TimeDelta,
    coarse_step: Option<TimeDelta>,
}

impl<F> RootFindingDetector<F, Brent> {
    pub fn new(func: F, step: TimeDelta) -> Self {
        Self {
            func,
            root_finder: Brent::default(),
            step,
            coarse_step: None,
        }
    }
}

impl<F, R> RootFindingDetector<F, R> {
    pub fn with_root_finder(func: F, step: TimeDelta, root_finder: R) -> Self {
        Self {
            func,
            root_finder,
            step,
            coarse_step: None,
        }
    }

    pub fn with_coarse_step(mut self, coarse_step: TimeDelta) -> Self {
        self.coarse_step = Some(coarse_step);
        self
    }
}

/// Build a uniform time grid from 0 to `total` with the given `step`,
/// always including the endpoint.
fn build_time_grid(total: f64, step: f64) -> Vec<f64> {
    let mut grid = Vec::new();
    let mut t = 0.0;
    while t <= total {
        grid.push(t);
        t += step;
    }
    if grid.last().is_none_or(|&last| last < total) {
        grid.push(total);
    }
    grid
}

impl<F, R> RootFindingDetector<F, R> {
    /// Core detection returning events and the sign at the interval start.
    ///
    /// The start sign is needed by [`EventsToIntervals`] to determine whether
    /// the condition holds throughout when no zero-crossings are found.
    /// Returning it here avoids a redundant function evaluation.
    pub(crate) fn detect_with_start_sign<T>(
        &self,
        interval: TimeInterval<T>,
    ) -> Result<(Vec<Event<T>>, f64), DetectError>
    where
        T: TimeScale + Copy,
        F: DetectFn<T>,
        for<'a> R: FindBracketedRoot<DetectCallback<'a, T, F>>,
    {
        let start = interval.start();
        let end = interval.end();
        let total_seconds = (end - start).to_seconds().to_f64();
        let step_seconds = self.step.to_seconds().to_f64();
        let callback = DetectCallback::new(&self.func, start);

        match self.coarse_step {
            Some(coarse_step) => {
                let coarse_seconds = coarse_step.to_seconds().to_f64();
                self.detect_two_level(callback, start, total_seconds, step_seconds, coarse_seconds)
            }
            None => self.detect_single_level(callback, start, total_seconds, step_seconds),
        }
    }

    /// Single-level detection: evaluate at every fine step then root-find.
    fn detect_single_level<T>(
        &self,
        callback: DetectCallback<'_, T, F>,
        start: Time<T>,
        total_seconds: f64,
        step_seconds: f64,
    ) -> Result<(Vec<Event<T>>, f64), DetectError>
    where
        T: TimeScale + Copy,
        F: DetectFn<T>,
        for<'a> R: FindBracketedRoot<DetectCallback<'a, T, F>>,
    {
        let steps = build_time_grid(total_seconds, step_seconds);

        let mut signs = Vec::with_capacity(steps.len());
        for &t in &steps {
            let v = callback
                .call(t)
                .map_err(|e| DetectError::RootFinder(RootFinderError::Callback(e)))?;
            signs.push(v.signum());
        }

        let start_sign = signs[0];

        if signs.iter().all(|&s| s < 0.0) || signs.iter().all(|&s| s > 0.0) {
            return Ok((vec![], start_sign));
        }

        let mut events = Vec::new();
        for ((&t0, &s0), (&t1, &s1)) in std::iter::zip(&steps, &signs).tuple_windows() {
            if let Some(crossing) = ZeroCrossing::new(s0, s1) {
                let t = self
                    .root_finder
                    .find_in_bracket(callback, (t0, t1))
                    .map_err(DetectError::RootFinder)?;
                let time = start + TimeDelta::from_seconds_f64(t);
                events.push(Event { crossing, time });
            }
        }

        Ok((events, start_sign))
    }

    /// Two-level detection: coarse grid to find sign-change brackets, then
    /// fine grid within each bracket to locate precise crossings.
    fn detect_two_level<T>(
        &self,
        callback: DetectCallback<'_, T, F>,
        start: Time<T>,
        total_seconds: f64,
        step_seconds: f64,
        coarse_seconds: f64,
    ) -> Result<(Vec<Event<T>>, f64), DetectError>
    where
        T: TimeScale + Copy,
        F: DetectFn<T>,
        for<'a> R: FindBracketedRoot<DetectCallback<'a, T, F>>,
    {
        // 1. Build coarse grid and evaluate signs.
        let coarse_grid = build_time_grid(total_seconds, coarse_seconds);
        let mut coarse_signs = Vec::with_capacity(coarse_grid.len());
        for &t in &coarse_grid {
            let v = callback
                .call(t)
                .map_err(|e| DetectError::RootFinder(RootFinderError::Callback(e)))?;
            coarse_signs.push(v.signum());
        }

        let start_sign = coarse_signs[0];

        // 2. For each coarse bracket with a sign change, subdivide with fine steps.
        let mut events = Vec::new();
        for ((&tc0, &sc0), (&tc1, &sc1)) in
            std::iter::zip(&coarse_grid, &coarse_signs).tuple_windows()
        {
            if ZeroCrossing::new(sc0, sc1).is_none() {
                continue;
            }

            // Build fine grid within this coarse bracket.
            // Reuse the known sign at tc0 to avoid a redundant evaluation.
            let bracket_len = tc1 - tc0;
            let fine_grid = build_time_grid(bracket_len, step_seconds);

            let mut fine_times = Vec::with_capacity(fine_grid.len());
            let mut fine_signs = Vec::with_capacity(fine_grid.len());

            // First point: reuse coarse sign.
            fine_times.push(tc0);
            fine_signs.push(sc0);

            // Interior and last points: evaluate.
            for &ft in &fine_grid[1..] {
                let abs_t = tc0 + ft;
                fine_times.push(abs_t);
                let v = callback
                    .call(abs_t)
                    .map_err(|e| DetectError::RootFinder(RootFinderError::Callback(e)))?;
                fine_signs.push(v.signum());
            }

            // Root-find on fine-level sign changes.
            for ((&t0, &s0), (&t1, &s1)) in std::iter::zip(&fine_times, &fine_signs).tuple_windows()
            {
                if let Some(crossing) = ZeroCrossing::new(s0, s1) {
                    let t = self
                        .root_finder
                        .find_in_bracket(callback, (t0, t1))
                        .map_err(DetectError::RootFinder)?;
                    let time = start + TimeDelta::from_seconds_f64(t);
                    events.push(Event { crossing, time });
                }
            }
        }

        Ok((events, start_sign))
    }
}

impl<T, F, R> EventDetector<T> for RootFindingDetector<F, R>
where
    T: TimeScale + Copy,
    F: DetectFn<T>,
    for<'a> R: FindBracketedRoot<DetectCallback<'a, T, F>>,
{
    fn detect(&self, interval: TimeInterval<T>) -> Result<Vec<Event<T>>, DetectError> {
        self.detect_with_start_sign(interval)
            .map(|(events, _)| events)
    }
}

// ---------------------------------------------------------------------------
// EventsToIntervals — converts EventDetector → IntervalDetector
// ---------------------------------------------------------------------------

/// Converts a [`RootFindingDetector`] into an [`IntervalDetector`] by pairing
/// Up/Down crossings into intervals.
///
/// When no events are found, the sign of the detect function at the interval
/// start is checked: if positive, the entire interval is returned; if
/// negative, an empty list is returned.
pub struct EventsToIntervals<F, R = Brent> {
    detector: RootFindingDetector<F, R>,
}

impl<F> EventsToIntervals<F, Brent> {
    pub fn new(detector: RootFindingDetector<F>) -> Self {
        Self { detector }
    }
}

impl<F, R> EventsToIntervals<F, R> {
    pub fn with_root_finder(detector: RootFindingDetector<F, R>) -> Self {
        Self { detector }
    }
}

impl<T, F, R> IntervalDetector<T> for EventsToIntervals<F, R>
where
    T: TimeScale + Copy,
    F: DetectFn<T>,
    for<'a> R: FindBracketedRoot<DetectCallback<'a, T, F>>,
{
    fn detect(&self, interval: TimeInterval<T>) -> Result<Vec<TimeInterval<T>>, DetectError> {
        let start = interval.start();
        let end = interval.end();

        let (events, start_sign) = self.detector.detect_with_start_sign(interval)?;
        if events.is_empty() {
            // No zero crossings — use the sign at the start (already computed
            // during step evaluation) to determine if the condition holds
            // throughout or not at all.
            return if start_sign >= 0.0 {
                Ok(vec![interval])
            } else {
                Ok(vec![])
            };
        }

        let mut events: VecDeque<Event<T>> = events.into();

        if events.front().unwrap().crossing == ZeroCrossing::Down {
            events.push_front(Event {
                crossing: ZeroCrossing::Up,
                time: start,
            });
        }

        if events.back().unwrap().crossing == ZeroCrossing::Up {
            events.push_back(Event {
                crossing: ZeroCrossing::Down,
                time: end,
            });
        }

        let mut intervals = Vec::with_capacity(events.len() / 2);
        for (up, down) in events.into_iter().tuples() {
            debug_assert!(up.crossing == ZeroCrossing::Up);
            debug_assert!(down.crossing == ZeroCrossing::Down);
            intervals.push(TimeInterval::new(up.time, down.time));
        }

        Ok(intervals)
    }
}

// ---------------------------------------------------------------------------
// Combinators
// ---------------------------------------------------------------------------

/// Intervals where BOTH A and B are active (intersection).
pub struct Intersection<A, B> {
    a: A,
    b: B,
}

impl<T, A, B> IntervalDetector<T> for Intersection<A, B>
where
    T: TimeScale + Ord + Copy,
    A: IntervalDetector<T>,
    B: IntervalDetector<T>,
{
    fn detect(&self, interval: TimeInterval<T>) -> Result<Vec<TimeInterval<T>>, DetectError> {
        let a = self.a.detect(interval)?;
        let b = self.b.detect(interval)?;
        Ok(lox_time::intervals::intersect_intervals(&a, &b))
    }
}

/// Intervals where EITHER A or B is active (union).
pub struct Union<A, B> {
    a: A,
    b: B,
}

impl<T, A, B> IntervalDetector<T> for Union<A, B>
where
    T: TimeScale + Ord + Copy,
    A: IntervalDetector<T>,
    B: IntervalDetector<T>,
{
    fn detect(&self, interval: TimeInterval<T>) -> Result<Vec<TimeInterval<T>>, DetectError> {
        let a = self.a.detect(interval)?;
        let b = self.b.detect(interval)?;
        Ok(lox_time::intervals::union_intervals(&a, &b))
    }
}

/// Intervals where D is NOT active (complement within the search interval).
pub struct Complement<D> {
    detector: D,
}

impl<T, D> IntervalDetector<T> for Complement<D>
where
    T: TimeScale + Ord + Copy,
    D: IntervalDetector<T>,
{
    fn detect(&self, interval: TimeInterval<T>) -> Result<Vec<TimeInterval<T>>, DetectError> {
        let inner = self.detector.detect(interval)?;
        Ok(lox_time::intervals::complement_intervals(&inner, interval))
    }
}

/// Optimization: B only evaluates within A's detected intervals.
pub struct Chain<A, B> {
    a: A,
    b: B,
}

impl<T, A, B> IntervalDetector<T> for Chain<A, B>
where
    T: TimeScale + Copy,
    A: IntervalDetector<T>,
    B: IntervalDetector<T>,
{
    fn detect(&self, interval: TimeInterval<T>) -> Result<Vec<TimeInterval<T>>, DetectError> {
        let a_intervals = self.a.detect(interval)?;
        let mut result = Vec::new();
        for sub in a_intervals {
            result.extend(self.b.detect(sub)?);
        }
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Extension trait for IntervalDetector combinators
// ---------------------------------------------------------------------------

pub trait IntervalDetectorExt<T: TimeScale>: IntervalDetector<T> + Sized {
    fn intersect<B>(self, other: B) -> Intersection<Self, B> {
        Intersection { a: self, b: other }
    }

    fn union<B>(self, other: B) -> Union<Self, B> {
        Union { a: self, b: other }
    }

    fn complement(self) -> Complement<Self> {
        Complement { detector: self }
    }

    fn chain<B>(self, other: B) -> Chain<Self, B> {
        Chain { a: self, b: other }
    }
}

impl<T: TimeScale, D: IntervalDetector<T>> IntervalDetectorExt<T> for D {}

// ---------------------------------------------------------------------------
// IntervalDetector impls for boxed trait objects
// ---------------------------------------------------------------------------

impl<T: TimeScale> IntervalDetector<T> for Box<dyn IntervalDetector<T> + '_> {
    fn detect(&self, interval: TimeInterval<T>) -> Result<Vec<TimeInterval<T>>, DetectError> {
        (**self).detect(interval)
    }
}

impl<T: TimeScale> IntervalDetector<T> for Box<dyn IntervalDetector<T> + Send + '_> {
    fn detect(&self, interval: TimeInterval<T>) -> Result<Vec<TimeInterval<T>>, DetectError> {
        (**self).detect(interval)
    }
}

// ---------------------------------------------------------------------------
// Closure-based DetectFn adapters
// ---------------------------------------------------------------------------

/// Wraps an infallible closure into a `DetectFn`.
pub struct FnDetect<F>(pub F);

impl<T, F> DetectFn<T> for FnDetect<F>
where
    T: TimeScale + Copy,
    F: Fn(Time<T>) -> f64,
{
    type Error = std::convert::Infallible;
    fn eval(&self, time: Time<T>) -> Result<f64, Self::Error> {
        Ok((self.0)(time))
    }
}

/// Wraps a fallible closure into a `DetectFn`.
pub struct TryFnDetect<F>(pub F);

impl<T, F, E> DetectFn<T> for TryFnDetect<F>
where
    T: TimeScale + Copy,
    F: Fn(Time<T>) -> Result<f64, E>,
    E: std::error::Error + Send + Sync + 'static,
{
    type Error = E;
    fn eval(&self, time: Time<T>) -> Result<f64, Self::Error> {
        (self.0)(time)
    }
}

// ---------------------------------------------------------------------------
// Convenience functions
// ---------------------------------------------------------------------------

/// Find zero-crossing events for an infallible closure over a time interval.
pub fn find_events<T, F>(
    func: F,
    interval: TimeInterval<T>,
    step: TimeDelta,
) -> Result<Vec<Event<T>>, DetectError>
where
    T: TimeScale + Copy,
    F: Fn(Time<T>) -> f64,
{
    RootFindingDetector::new(FnDetect(func), step).detect(interval)
}

/// Find zero-crossing events for a fallible closure over a time interval.
pub fn try_find_events<T, F, E>(
    func: F,
    interval: TimeInterval<T>,
    step: TimeDelta,
) -> Result<Vec<Event<T>>, DetectError>
where
    T: TimeScale + Copy,
    F: Fn(Time<T>) -> Result<f64, E>,
    E: std::error::Error + Send + Sync + 'static,
{
    RootFindingDetector::new(TryFnDetect(func), step).detect(interval)
}

/// Find intervals where an infallible closure is positive.
pub fn find_windows<T, F>(
    func: F,
    interval: TimeInterval<T>,
    step: TimeDelta,
) -> Result<Vec<TimeInterval<T>>, DetectError>
where
    T: TimeScale + Copy,
    F: Fn(Time<T>) -> f64,
{
    let detector = RootFindingDetector::new(FnDetect(func), step);
    EventsToIntervals::new(detector).detect(interval)
}

/// Find intervals where a fallible closure is positive.
pub fn try_find_windows<T, F, E>(
    func: F,
    interval: TimeInterval<T>,
    step: TimeDelta,
) -> Result<Vec<TimeInterval<T>>, DetectError>
where
    T: TimeScale + Copy,
    F: Fn(Time<T>) -> Result<f64, E>,
    E: std::error::Error + Send + Sync + 'static,
{
    let detector = RootFindingDetector::new(TryFnDetect(func), step);
    EventsToIntervals::new(detector).detect(interval)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_test_utils::assert_approx_eq;
    use lox_time::time;
    use lox_time::time_scales::Tai;
    use std::f64::consts::{PI, TAU};
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A `DetectFn` wrapper that counts evaluations via an `AtomicUsize`.
    struct CountingDetectFn<'a, F> {
        inner: F,
        counter: &'a AtomicUsize,
    }

    impl<'a, T, F> DetectFn<T> for CountingDetectFn<'a, F>
    where
        T: TimeScale + Copy,
        F: Fn(Time<T>) -> f64,
    {
        type Error = std::convert::Infallible;
        fn eval(&self, time: Time<T>) -> Result<f64, Self::Error> {
            self.counter.fetch_add(1, Ordering::Relaxed);
            Ok((self.inner)(time))
        }
    }

    #[test]
    fn test_events() {
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let end = start + TimeDelta::from_seconds(7);
        let interval = TimeInterval::new(start, end);

        let detect_fn = FnDetect(|t: Time<Tai>| (t - start).to_seconds().to_f64().sin());
        let detector = RootFindingDetector::new(detect_fn, TimeDelta::from_seconds(1));
        let events = detector.detect(interval).unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].crossing, ZeroCrossing::Down);
        assert_approx_eq!(
            events[0].time,
            start + TimeDelta::from_seconds_f64(PI),
            rtol <= 1e-6
        );
        assert_eq!(events[1].crossing, ZeroCrossing::Up);
        assert_approx_eq!(
            events[1].time,
            start + TimeDelta::from_seconds_f64(TAU),
            rtol <= 1e-6
        );
    }

    #[test]
    fn test_windows() {
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let end = start + TimeDelta::from_seconds(7);
        let interval = TimeInterval::new(start, end);

        let detect_fn = FnDetect(|t: Time<Tai>| (t - start).to_seconds().to_f64().sin());
        let detector = RootFindingDetector::new(detect_fn, TimeDelta::from_seconds(1));
        let intervals_detector = EventsToIntervals::new(detector);
        let windows = intervals_detector.detect(interval).unwrap();

        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].start(), start);
        assert_approx_eq!(
            windows[0].end(),
            start + TimeDelta::from_seconds_f64(PI),
            rtol <= 1e-6
        );
    }

    #[test]
    fn test_windows_no_windows() {
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let end = start + TimeDelta::from_seconds(7);
        let interval = TimeInterval::new(start, end);

        let detect_fn = FnDetect(|_t: Time<Tai>| -1.0);
        let detector = RootFindingDetector::new(detect_fn, TimeDelta::from_seconds(1));
        let intervals_detector = EventsToIntervals::new(detector);
        let windows = intervals_detector.detect(interval).unwrap();

        assert!(windows.is_empty());
    }

    #[test]
    fn test_windows_full_coverage() {
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let end = start + TimeDelta::from_seconds(7);
        let interval = TimeInterval::new(start, end);

        let detect_fn = FnDetect(|_t: Time<Tai>| 1.0);
        let detector = RootFindingDetector::new(detect_fn, TimeDelta::from_seconds(1));
        let intervals_detector = EventsToIntervals::new(detector);
        let windows = intervals_detector.detect(interval).unwrap();

        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].start(), start);
        assert_eq!(windows[0].end(), end);
    }

    // -----------------------------------------------------------------------
    // Two-level stepping tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_two_level_matches_single_level() {
        // sin(t) over [0, 7]: zero crossings at PI and TAU.
        // Two-level with coarse_step=3s, fine step=1s should find the same events.
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let end = start + TimeDelta::from_seconds(7);
        let interval = TimeInterval::new(start, end);

        let single = RootFindingDetector::new(
            FnDetect(move |t: Time<Tai>| (t - start).to_seconds().to_f64().sin()),
            TimeDelta::from_seconds(1),
        )
        .detect(interval)
        .unwrap();

        let two_level = RootFindingDetector::new(
            FnDetect(move |t: Time<Tai>| (t - start).to_seconds().to_f64().sin()),
            TimeDelta::from_seconds(1),
        )
        .with_coarse_step(TimeDelta::from_seconds(3))
        .detect(interval)
        .unwrap();

        assert_eq!(single.len(), two_level.len());
        for (s, tl) in single.iter().zip(&two_level) {
            assert_eq!(s.crossing, tl.crossing);
            assert_approx_eq!(s.time, tl.time, rtol <= 1e-6);
        }
    }

    #[test]
    fn test_two_level_multiple_crossings_in_bracket() {
        // sin(t + 0.5) over [0, 10]: one coarse bracket [0, 10] contains 3
        // zero crossings (at t ≈ 2.64, 5.78, 8.92). The bracket has a sign
        // change (sin(0.5) > 0, sin(10.5) < 0) so the fine grid is applied
        // and all 3 crossings are found.
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let end = start + TimeDelta::from_seconds(10);
        let interval = TimeInterval::new(start, end);

        let func = move |t: Time<Tai>| ((t - start).to_seconds().to_f64() + 0.5).sin();

        let single = RootFindingDetector::new(FnDetect(func), TimeDelta::from_seconds(1))
            .detect(interval)
            .unwrap();

        let two_level = RootFindingDetector::new(FnDetect(func), TimeDelta::from_seconds(1))
            .with_coarse_step(TimeDelta::from_seconds(10))
            .detect(interval)
            .unwrap();

        assert_eq!(single.len(), 3, "expected 3 crossings");
        assert_eq!(single.len(), two_level.len());
        for (s, tl) in single.iter().zip(&two_level) {
            assert_eq!(s.crossing, tl.crossing);
            assert_approx_eq!(s.time, tl.time, rtol <= 1e-6);
        }
    }

    #[test]
    fn test_two_level_no_events() {
        // Constant negative function — no events, correct start_sign.
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let end = start + TimeDelta::from_seconds(10);
        let interval = TimeInterval::new(start, end);

        let det =
            RootFindingDetector::new(FnDetect(|_t: Time<Tai>| -1.0), TimeDelta::from_seconds(1))
                .with_coarse_step(TimeDelta::from_seconds(3));

        let (events, start_sign) = det.detect_with_start_sign(interval).unwrap();
        assert!(events.is_empty());
        assert!(start_sign < 0.0);
    }

    #[test]
    fn test_two_level_windows_roundtrip() {
        // EventsToIntervals with a coarse-stepped detector produces the same
        // windows as without for sin(t) over [0, 7].
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let end = start + TimeDelta::from_seconds(7);
        let interval = TimeInterval::new(start, end);

        let single_windows = EventsToIntervals::new(RootFindingDetector::new(
            FnDetect(move |t: Time<Tai>| (t - start).to_seconds().to_f64().sin()),
            TimeDelta::from_seconds(1),
        ))
        .detect(interval)
        .unwrap();

        let two_level_windows = EventsToIntervals::new(
            RootFindingDetector::new(
                FnDetect(move |t: Time<Tai>| (t - start).to_seconds().to_f64().sin()),
                TimeDelta::from_seconds(1),
            )
            .with_coarse_step(TimeDelta::from_seconds(3)),
        )
        .detect(interval)
        .unwrap();

        assert_eq!(single_windows.len(), two_level_windows.len());
        for (s, tl) in single_windows.iter().zip(&two_level_windows) {
            assert_approx_eq!(s.start(), tl.start(), rtol <= 1e-6);
            assert_approx_eq!(s.end(), tl.end(), rtol <= 1e-6);
        }
    }

    #[test]
    fn test_two_level_eval_count_reduction() {
        // Verify that two-level stepping uses fewer evaluations than single-level
        // for a long interval with sparse events.
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let end = start + TimeDelta::from_seconds(1000);
        let interval = TimeInterval::new(start, end);

        // sin(t/100) — two crossings in [0, 1000] at ~314s and ~628s.
        let func = move |t: Time<Tai>| ((t - start).to_seconds().to_f64() / 100.0).sin();

        let counter_single = AtomicUsize::new(0);
        let single = RootFindingDetector::new(
            CountingDetectFn {
                inner: func,
                counter: &counter_single,
            },
            TimeDelta::from_seconds(1),
        )
        .detect(interval)
        .unwrap();

        let counter_two = AtomicUsize::new(0);
        // min_pass_duration = 300s → coarse_step = 150s
        let two_level = RootFindingDetector::new(
            CountingDetectFn {
                inner: func,
                counter: &counter_two,
            },
            TimeDelta::from_seconds(1),
        )
        .with_coarse_step(TimeDelta::from_seconds(150))
        .detect(interval)
        .unwrap();

        // Both should find the same events.
        assert_eq!(single.len(), two_level.len());

        let single_evals = counter_single.load(Ordering::Relaxed);
        let two_level_evals = counter_two.load(Ordering::Relaxed);

        // Two-level should use significantly fewer evals.
        assert!(
            two_level_evals < single_evals,
            "two-level ({two_level_evals}) should use fewer evals than single-level ({single_evals})"
        );
    }

    // -----------------------------------------------------------------------
    // Combinator and extension-trait tests
    // -----------------------------------------------------------------------

    /// Helper: build an `EventsToIntervals` detector from an infallible closure.
    fn make_window_detector<F: Fn(Time<Tai>) -> f64>(
        func: F,
        step: TimeDelta,
    ) -> EventsToIntervals<FnDetect<F>> {
        EventsToIntervals::new(RootFindingDetector::new(FnDetect(func), step))
    }

    /// sin(t) is positive on (0, PI) within [0, 7].
    fn sin_detector(start: Time<Tai>) -> EventsToIntervals<FnDetect<impl Fn(Time<Tai>) -> f64>> {
        make_window_detector(
            move |t: Time<Tai>| (t - start).to_seconds().to_f64().sin(),
            TimeDelta::from_seconds(1),
        )
    }

    /// cos(t) is positive on [0, PI/2) and (3PI/2, 7] within [0, 7].
    fn cos_detector(start: Time<Tai>) -> EventsToIntervals<FnDetect<impl Fn(Time<Tai>) -> f64>> {
        make_window_detector(
            move |t: Time<Tai>| (t - start).to_seconds().to_f64().cos(),
            TimeDelta::from_seconds(1),
        )
    }

    fn test_interval() -> (Time<Tai>, TimeInterval<Tai>) {
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let end = start + TimeDelta::from_seconds(7);
        (start, TimeInterval::new(start, end))
    }

    #[test]
    fn test_intersect() {
        let (start, interval) = test_interval();
        // sin > 0 on (0, PI), cos > 0 on [0, PI/2) ∪ (3PI/2, 7]
        // intersection: (0, PI/2) — both positive only here (within [0, PI])
        let det = sin_detector(start).intersect(cos_detector(start));
        let windows = det.detect(interval).unwrap();
        assert_eq!(windows.len(), 2);
        // First window: start..PI/2
        assert_approx_eq!(windows[0].start(), start, rtol <= 1e-6);
        assert_approx_eq!(
            windows[0].end(),
            start + TimeDelta::from_seconds_f64(PI / 2.0),
            rtol <= 1e-4
        );
    }

    #[test]
    fn test_union() {
        let (start, interval) = test_interval();
        // sin > 0 on (0, PI), cos > 0 on [0, PI/2) ∪ (3PI/2, 7]
        // union covers most of the interval
        let det = sin_detector(start).union(cos_detector(start));
        let windows = det.detect(interval).unwrap();
        // The union should cover [0, PI] ∪ [3PI/2, 7]
        assert_eq!(windows.len(), 2);
        // First window spans from start to PI (sin covers 0..PI, cos covers 0..PI/2)
        assert_approx_eq!(windows[0].start(), start, rtol <= 1e-6);
        assert_approx_eq!(
            windows[0].end(),
            start + TimeDelta::from_seconds_f64(PI),
            rtol <= 1e-4
        );
    }

    #[test]
    fn test_complement() {
        let (start, interval) = test_interval();
        // sin > 0 on [start, PI] ∪ [TAU, end] within [0, 7]
        // complement: [PI, TAU]
        let det = sin_detector(start).complement();
        let windows = det.detect(interval).unwrap();
        assert_eq!(windows.len(), 1);
        assert_approx_eq!(
            windows[0].start(),
            start + TimeDelta::from_seconds_f64(PI),
            rtol <= 1e-4
        );
        assert_approx_eq!(
            windows[0].end(),
            start + TimeDelta::from_seconds_f64(TAU),
            rtol <= 1e-4
        );
    }

    #[test]
    fn test_chain() {
        let (start, interval) = test_interval();
        // Chain: first find sin > 0 windows, then within those evaluate cos > 0.
        // sin > 0 on [start, PI] ∪ [TAU, end].
        // Within [start, PI]: cos > 0 on [start, PI/2].
        // Within [TAU, end]: cos(TAU..7) > 0 throughout, so [TAU, end].
        let det = sin_detector(start).chain(cos_detector(start));
        let windows = det.detect(interval).unwrap();
        assert_eq!(windows.len(), 2);
        // First window: [start, PI/2]
        assert_approx_eq!(windows[0].start(), start, rtol <= 1e-6);
        assert_approx_eq!(
            windows[0].end(),
            start + TimeDelta::from_seconds_f64(PI / 2.0),
            rtol <= 1e-4
        );
        // Second window: [TAU, end]
        assert_approx_eq!(
            windows[1].start(),
            start + TimeDelta::from_seconds_f64(TAU),
            rtol <= 1e-4
        );
        assert_approx_eq!(
            windows[1].end(),
            start + TimeDelta::from_seconds(7),
            rtol <= 1e-6
        );
    }

    #[test]
    fn test_chain_restricts_evaluation() {
        // Chain should only evaluate B within A's windows.
        // Use a constant-negative A to prove B is never called.
        let (start, interval) = test_interval();
        let counter = AtomicUsize::new(0);
        let a = make_window_detector(|_t: Time<Tai>| -1.0, TimeDelta::from_seconds(1));
        let b = EventsToIntervals::new(RootFindingDetector::new(
            CountingDetectFn {
                inner: move |t: Time<Tai>| (t - start).to_seconds().to_f64().sin(),
                counter: &counter,
            },
            TimeDelta::from_seconds(1),
        ));
        let windows = a.chain(b).detect(interval).unwrap();
        assert!(windows.is_empty());
        assert_eq!(counter.load(Ordering::Relaxed), 0);
    }

    // -----------------------------------------------------------------------
    // Boxed IntervalDetector tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_boxed_interval_detector() {
        let (start, interval) = test_interval();
        let det: Box<dyn IntervalDetector<Tai>> = Box::new(sin_detector(start));
        let windows = det.detect(interval).unwrap();
        // sin > 0 on (0, PI) and (TAU, 7)
        assert_eq!(windows.len(), 2);
        assert_approx_eq!(
            windows[0].end(),
            start + TimeDelta::from_seconds_f64(PI),
            rtol <= 1e-4
        );
    }

    #[test]
    fn test_boxed_send_interval_detector() {
        let (start, interval) = test_interval();
        let det: Box<dyn IntervalDetector<Tai> + Send> = Box::new(sin_detector(start));
        let windows = det.detect(interval).unwrap();
        assert_eq!(windows.len(), 2);
    }

    #[test]
    fn test_boxed_dynamic_fold() {
        // Fold multiple detectors via Box<dyn IntervalDetector>, simulating
        // the pattern used in VisibilityAnalysis for occulting bodies.
        let (start, interval) = test_interval();

        // sin > 0 on (0, PI) ∪ (TAU, 7)
        // cos > 0 on [0, PI/2) ∪ (3PI/2, 7]
        // intersection: [0, PI/2) ∪ (TAU, 7] (approximately)
        let detectors: Vec<Box<dyn IntervalDetector<Tai>>> =
            vec![Box::new(sin_detector(start)), Box::new(cos_detector(start))];

        let mut combined: Box<dyn IntervalDetector<Tai>> = detectors.into_iter().next().unwrap();

        let det = Box::new(cos_detector(start)) as Box<dyn IntervalDetector<Tai>>;
        combined = Box::new(combined.intersect(det));

        let windows = combined.detect(interval).unwrap();
        assert_eq!(windows.len(), 2);
        // First window should end around PI/2
        assert_approx_eq!(
            windows[0].end(),
            start + TimeDelta::from_seconds_f64(PI / 2.0),
            rtol <= 1e-4
        );
    }

    // -----------------------------------------------------------------------
    // Convenience function tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_find_events() {
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let end = start + TimeDelta::from_seconds(7);
        let interval = TimeInterval::new(start, end);
        let events = find_events(
            |t: Time<Tai>| (t - start).to_seconds().to_f64().sin(),
            interval,
            TimeDelta::from_seconds(1),
        )
        .unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_try_find_events() {
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let end = start + TimeDelta::from_seconds(7);
        let interval = TimeInterval::new(start, end);
        let events = try_find_events(
            |t: Time<Tai>| {
                Ok::<f64, std::convert::Infallible>((t - start).to_seconds().to_f64().sin())
            },
            interval,
            TimeDelta::from_seconds(1),
        )
        .unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_find_windows() {
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let end = start + TimeDelta::from_seconds(7);
        let interval = TimeInterval::new(start, end);
        let windows = find_windows(
            |t: Time<Tai>| (t - start).to_seconds().to_f64().sin(),
            interval,
            TimeDelta::from_seconds(1),
        )
        .unwrap();
        assert_eq!(windows.len(), 2);
        assert_approx_eq!(
            windows[0].end(),
            start + TimeDelta::from_seconds_f64(PI),
            rtol <= 1e-4
        );
    }

    #[test]
    fn test_try_find_windows() {
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let end = start + TimeDelta::from_seconds(7);
        let interval = TimeInterval::new(start, end);
        let windows = try_find_windows(
            |t: Time<Tai>| {
                Ok::<f64, std::convert::Infallible>((t - start).to_seconds().to_f64().sin())
            },
            interval,
            TimeDelta::from_seconds(1),
        )
        .unwrap();
        assert_eq!(windows.len(), 2);
    }
}
