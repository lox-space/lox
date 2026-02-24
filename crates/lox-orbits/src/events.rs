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

#[derive(Debug, Error)]
pub enum FindEventError {
    #[error("function is always negative")]
    AlwaysNegative,
    #[error("function is always positive")]
    AlwaysPositive,
    #[error(transparent)]
    RootFinder(#[from] RootFinderError),
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
struct DetectCallback<'a, T: TimeScale, F: DetectFn<T>> {
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
    func: F,
    root_finder: R,
    step: TimeDelta,
}

impl<F> RootFindingDetector<F, Brent> {
    pub fn new(func: F, step: TimeDelta) -> Self {
        Self {
            func,
            root_finder: Brent::default(),
            step,
        }
    }
}

impl<F, R> RootFindingDetector<F, R> {
    pub fn with_root_finder(func: F, step: TimeDelta, root_finder: R) -> Self {
        Self {
            func,
            root_finder,
            step,
        }
    }
}

impl<T, F, R> EventDetector<T> for RootFindingDetector<F, R>
where
    T: TimeScale + Copy,
    F: DetectFn<T>,
    for<'a> R: FindBracketedRoot<DetectCallback<'a, T, F>>,
{
    fn detect(&self, interval: TimeInterval<T>) -> Result<Vec<Event<T>>, DetectError> {
        let start = interval.start();
        let end = interval.end();
        let total_seconds = (end - start).to_seconds().to_f64();
        let step_seconds = self.step.to_seconds().to_f64();

        // Build time steps
        let mut steps = Vec::new();
        let mut t = 0.0;
        while t <= total_seconds {
            steps.push(t);
            t += step_seconds;
        }
        if steps.last().is_none_or(|&last| last < total_seconds) {
            steps.push(total_seconds);
        }

        // Evaluate function at each step
        let callback = DetectCallback::new(&self.func, start);
        let mut signs = Vec::with_capacity(steps.len());
        for &t in &steps {
            let v = callback
                .call(t)
                .map_err(|e| DetectError::RootFinder(RootFinderError::Callback(e)))?;
            signs.push(v.signum());
        }

        // All negative or all positive → no events
        if signs.iter().all(|&s| s < 0.0) || signs.iter().all(|&s| s > 0.0) {
            return Ok(vec![]);
        }

        // Find zero crossings
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

        Ok(events)
    }
}

// ---------------------------------------------------------------------------
// EventsToIntervals — converts EventDetector → IntervalDetector
// ---------------------------------------------------------------------------

/// Converts an `EventDetector` into an `IntervalDetector` by pairing
/// Up/Down crossings into intervals.
pub struct EventsToIntervals<D> {
    detector: D,
}

impl<D> EventsToIntervals<D> {
    pub fn new(detector: D) -> Self {
        Self { detector }
    }
}

impl<T, D> IntervalDetector<T> for EventsToIntervals<D>
where
    T: TimeScale + Copy,
    D: EventDetector<T>,
{
    fn detect(&self, interval: TimeInterval<T>) -> Result<Vec<TimeInterval<T>>, DetectError> {
        let start = interval.start();
        let end = interval.end();

        let events = self.detector.detect(interval)?;
        if events.is_empty() {
            return Ok(vec![]);
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
// Legacy free functions (kept for backward compat during migration)
// ---------------------------------------------------------------------------

pub fn find_events<F, R, T>(
    func: F,
    start: Time<T>,
    steps: &[f64],
    root_finder: R,
) -> Result<Vec<Event<T>>, FindEventError>
where
    F: Callback + Clone,
    T: TimeScale + Clone,
    R: FindBracketedRoot<F>,
{
    let mut signs = Vec::with_capacity(steps.len());
    for &t in steps {
        let v = func
            .call(t)
            .map_err(|e| FindEventError::RootFinder(RootFinderError::Callback(e)))?;
        signs.push(v.signum());
    }

    if signs.iter().all(|&s| s < 0.0) {
        return Err(FindEventError::AlwaysNegative);
    }

    if signs.iter().all(|&s| s > 0.0) {
        return Err(FindEventError::AlwaysPositive);
    }

    let mut events = vec![];

    for ((&t0, s0), (&t1, s1)) in std::iter::zip(steps, signs).tuple_windows() {
        if let Some(crossing) = ZeroCrossing::new(s0, s1) {
            let t = root_finder
                .find_in_bracket(func.clone(), (t0, t1))
                .map_err(FindEventError::RootFinder)?;
            let time = start.clone() + TimeDelta::from_seconds_f64(t);
            events.push(Event { crossing, time });
        }
    }

    Ok(events)
}

pub fn find_windows<F, T, R>(
    func: F,
    start: Time<T>,
    end: Time<T>,
    steps: &[f64],
    root_finder: R,
) -> Result<Vec<TimeInterval<T>>, RootFinderError>
where
    F: Callback + Clone,
    T: TimeScale + Clone,
    R: FindBracketedRoot<F>,
{
    match find_events(func, start.clone(), steps, root_finder) {
        Err(FindEventError::AlwaysNegative) => Ok(vec![]),
        Err(FindEventError::AlwaysPositive) => Ok(vec![TimeInterval::new(start, end)]),
        Err(FindEventError::RootFinder(err)) => Err(err),
        Ok(events) => {
            let mut events: VecDeque<Event<T>> = events.into();

            if events.is_empty() {
                return Ok(vec![]);
            }

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

            let mut windows = Vec::with_capacity(events.len() / 2);

            for (start, end) in events.into_iter().tuples() {
                debug_assert!(start.crossing == ZeroCrossing::Up);
                debug_assert!(end.crossing == ZeroCrossing::Down);
                windows.push(TimeInterval::new(start.time, end.time));
            }

            Ok(windows)
        }
    }
}

pub fn intersect_windows<T>(w1: &[TimeInterval<T>], w2: &[TimeInterval<T>]) -> Vec<TimeInterval<T>>
where
    T: TimeScale + Ord + Copy,
{
    lox_time::intervals::intersect_intervals(w1, w2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_math::roots::Brent;
    use lox_test_utils::assert_approx_eq;
    use lox_time::time_scales::Tai;
    use lox_time::{Time, time};
    use std::f64::consts::{PI, TAU};

    #[test]
    fn test_events() {
        let func = |t: f64| Ok(t.sin());
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let steps = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];

        let root_finder = Brent::default();

        let events = find_events(func, start, &steps, root_finder).unwrap();

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
        let func = |t: f64| Ok(t.sin());
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let steps = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let end = start + TimeDelta::from_seconds(7);

        let root_finder = Brent::default();

        let windows = find_windows(func, start, end, &steps, root_finder).unwrap();

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
        let func = |_: f64| Ok(-1.0);
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let steps = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let end = start + TimeDelta::from_seconds(7);

        let root_finder = Brent::default();

        let windows = find_windows(func, start, end, &steps, root_finder).unwrap();

        assert!(windows.is_empty());
    }

    #[test]
    fn test_windows_full_coverage() {
        let func = |_: f64| Ok(1.0);
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let steps = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let end = start + TimeDelta::from_seconds(7);

        let root_finder = Brent::default();

        let windows = find_windows(func, start, end, &steps, root_finder).unwrap();

        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].start(), start);
        assert_eq!(windows[0].end(), end);
    }
}
