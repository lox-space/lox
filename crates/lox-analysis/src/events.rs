// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Lazy zero-crossing event detection over time intervals.

use lox_core::{
    error::LoxError,
    math::{
        callback::{Callback, CallbackWithDerivative},
        roots::{Brent, FindBracketedRoot, FindBracketedRootWithDerivative, RootFinderError},
    },
    sync::CancellationToken,
};
use lox_time::{Time, deltas::TimeDelta, intervals::TimeInterval};
use thiserror::Error;

pub use lox_core::math::zero_crossing::ZeroCrossing;

/// A zero-crossing event at a specific time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Event {
    crossing: ZeroCrossing,
    time: Time,
}

impl Event {
    /// Creates a new event at the given time with the specified crossing direction.
    pub fn new(time: Time, crossing: ZeroCrossing) -> Self {
        Self { crossing, time }
    }

    /// Returns the time of the event.
    pub fn time(&self) -> Time {
        self.time
    }

    /// Returns the crossing direction.
    pub fn crossing(&self) -> ZeroCrossing {
        self.crossing
    }
}

/// Scalar function whose zero-crossings define events.
pub trait DetectFn {
    /// The error type returned by [`eval`](Self::eval).
    type Error: Into<LoxError>;
    /// Evaluates the detection function at the given time.
    fn eval(&self, time: Time) -> Result<f64, Self::Error>;
}

impl<F> DetectFn for &F
where
    F: DetectFn,
{
    type Error = F::Error;

    fn eval(&self, time: Time) -> Result<f64, Self::Error> {
        (**self).eval(time)
    }
}

/// Wraps an infallible closure into a [`DetectFn`].
pub struct FnDetect<F>(pub F);

impl<F> DetectFn for FnDetect<F>
where
    F: Fn(Time) -> f64,
{
    type Error = std::convert::Infallible;

    fn eval(&self, time: Time) -> Result<f64, Self::Error> {
        Ok((self.0)(time))
    }
}

/// Wraps a fallible closure into a [`DetectFn`].
pub struct TryFnDetect<F>(pub F);

impl<F, E> DetectFn for TryFnDetect<F>
where
    F: Fn(Time) -> Result<f64, E>,
    E: std::error::Error + Send + Sync + 'static,
{
    type Error = LoxError;

    fn eval(&self, time: Time) -> Result<f64, Self::Error> {
        (self.0)(time).map_err(LoxError::new)
    }
}

/// Scalar function that can also evaluate its time derivative.
pub trait Differentiable: DetectFn {
    /// Evaluates the detection function and its time derivative (per second)
    /// at the given time.
    fn eval_derivative(&self, time: Time) -> Result<(f64, f64), Self::Error>;
}

impl<F> Differentiable for &F
where
    F: Differentiable,
{
    fn eval_derivative(&self, time: Time) -> Result<(f64, f64), Self::Error> {
        (**self).eval_derivative(time)
    }
}

struct DetectCallback<'f, F> {
    f: &'f F,
    start: Time,
}

impl<'f, F> Callback for DetectCallback<'f, F>
where
    F: DetectFn,
{
    fn call(&self, t: f64) -> Result<f64, LoxError> {
        let time = self.start + TimeDelta::from_seconds_f64(t);
        self.f.eval(time).map_err(Into::into)
    }
}

struct DifferentiableCallback<'f, F> {
    f: &'f F,
    start: Time,
}

impl<'f, F> CallbackWithDerivative for DifferentiableCallback<'f, F>
where
    F: Differentiable,
{
    fn call(&self, t: f64) -> Result<(f64, f64), LoxError> {
        let time = self.start + TimeDelta::from_seconds_f64(t);
        self.f.eval_derivative(time).map_err(Into::into)
    }
}

/// Refines a bracketed crossing of a [`DetectFn`] to a root.
///
/// The bracket is expressed in seconds relative to `start`, and `values` are
/// the function values at the bracket endpoints. Implemented for every
/// [`FindBracketedRoot`]; wrap a [`FindBracketedRootWithDerivative`] in
/// [`WithDerivative`] to refine using the derivative of a [`Differentiable`]
/// detect function.
pub trait Refine<F: DetectFn> {
    /// Locates the root within `bracket`.
    fn refine(
        &self,
        f: &F,
        start: Time,
        bracket: (f64, f64),
        values: (f64, f64),
    ) -> Result<f64, RootFinderError>;
}

impl<F, R> Refine<F> for R
where
    F: DetectFn,
    R: FindBracketedRoot,
{
    fn refine(
        &self,
        f: &F,
        start: Time,
        bracket: (f64, f64),
        values: (f64, f64),
    ) -> Result<f64, RootFinderError> {
        self.find_in_bracket_with_values(DetectCallback { f, start }, bracket, values)
    }
}

/// Adapts a derivative-based root finder into a [`Refine`] implementation for
/// [`Differentiable`] detect functions.
pub struct WithDerivative<R>(pub R);

impl<F, R> Refine<F> for WithDerivative<R>
where
    F: Differentiable,
    R: FindBracketedRootWithDerivative,
{
    fn refine(
        &self,
        f: &F,
        start: Time,
        bracket: (f64, f64),
        values: (f64, f64),
    ) -> Result<f64, RootFinderError> {
        self.0.find_in_bracket_with_derivative_values(
            DifferentiableCallback { f, start },
            bracket,
            values,
        )
    }
}

/// Scalar function that also reports an upper bound on the magnitude of its
/// time derivative at each point.
pub trait RateBounded: DetectFn {
    /// Evaluates the detection function and an upper bound on `|d/dt eval|`
    /// (per second) at the given time.
    ///
    /// The bound must hold over the step taken from this point, not just
    /// instantaneously; [`AdaptiveSampler`] scales strides by a safety factor
    /// to absorb slack in locally derived bounds.
    fn eval_bounded(&self, time: Time) -> Result<(f64, f64), Self::Error>;
}

impl<F> RateBounded for &F
where
    F: RateBounded,
{
    fn eval_bounded(&self, time: Time) -> Result<(f64, f64), Self::Error> {
        (**self).eval_bounded(time)
    }
}

/// Determines the coarse sampling cadence of an event scan.
///
/// A sampler owns the evaluation of the detect function at sample points:
/// [`sample`](Self::sample) returns the value at the current point together
/// with the step to the next one. Fusing evaluation and stepping lets
/// adaptive samplers derive the step from per-point information (e.g. a
/// [`RateBounded`] rate bound) from a single evaluation.
pub trait Sampler<F: DetectFn> {
    /// Evaluates `f` at `time` and returns the value and the step to the
    /// next sample point.
    ///
    /// The step must be positive; clipping to the scan interval is the
    /// caller's responsibility.
    fn sample(&mut self, f: &F, time: Time) -> Result<(f64, TimeDelta), F::Error>;
}

/// Samples with a fixed step.
pub struct UniformSampler {
    step: TimeDelta,
}

impl UniformSampler {
    /// Creates a sampler with the given step.
    ///
    /// Events whose duration is shorter than `step` may be missed; prefer
    /// [`from_min_duration`](Self::from_min_duration) to state that contract
    /// explicitly.
    pub fn new(step: TimeDelta) -> Self {
        Self { step }
    }

    /// Creates a sampler that detects every event lasting at least `duration`.
    pub fn from_min_duration(duration: TimeDelta) -> Self {
        Self::new(0.5 * duration)
    }
}

impl<F> Sampler<F> for UniformSampler
where
    F: DetectFn,
{
    fn sample(&mut self, f: &F, time: Time) -> Result<(f64, TimeDelta), F::Error> {
        Ok((f.eval(time)?, self.step))
    }
}

/// Samples with a step proportional to the distance from zero.
///
/// `max_slope` is a bound on the magnitude of the detect function's time
/// derivative (units per second). A function bounded this way cannot reach
/// zero from value `v` in less than `|v| / max_slope` seconds, so stepping by
/// that amount — clamped to `[min_step, max_step]` — cannot step past a
/// crossing while `|v| > max_slope · min_step`. Near zero, where the ideal
/// step vanishes, the scan advances by `min_step`; as with [`UniformSampler`],
/// excursions shorter than `min_step` may be missed.
///
/// If `max_slope` does not actually bound the derivative, crossings may be
/// missed anywhere.
pub struct LipschitzSampler {
    max_slope: f64,
    min_step: TimeDelta,
    max_step: TimeDelta,
}

impl LipschitzSampler {
    /// Creates a sampler for a detect function whose derivative magnitude is
    /// bounded by `max_slope` (units per second).
    ///
    /// # Panics
    ///
    /// Panics if `max_slope` is not finite and positive, if `min_step` is not
    /// positive, or if `max_step < min_step`.
    pub fn new(max_slope: f64, min_step: TimeDelta, max_step: TimeDelta) -> Self {
        assert!(
            max_slope.is_finite() && max_slope > 0.0,
            "max_slope must be finite and positive"
        );
        assert!(min_step.is_positive(), "min_step must be positive");
        assert!(max_step >= min_step, "max_step must not be below min_step");
        Self {
            max_slope,
            min_step,
            max_step,
        }
    }
}

impl<F> Sampler<F> for LipschitzSampler
where
    F: DetectFn,
{
    fn sample(&mut self, f: &F, time: Time) -> Result<(f64, TimeDelta), F::Error> {
        let value = f.eval(time)?;
        let step = TimeDelta::from_seconds_f64(value.abs() / self.max_slope)
            .clamp(self.min_step, self.max_step);
        Ok((value, step))
    }
}

/// Fraction of the rate-bound-derived stride actually taken. A value below 1
/// keeps each step strictly short of the earliest possible crossing,
/// absorbing the difference between the local bound and the true supremum of
/// `|f′|` over the step.
const ADAPTIVE_SAFETY: f64 = 0.9;

/// Samples with a stride derived from a per-point rate bound.
///
/// At a sample with value `v` and rate bound `L` from
/// [`RateBounded::eval_bounded`], the function cannot reach zero in less than
/// `|v| / L` seconds, so that distance (scaled by a safety factor) is jumped,
/// clamped to `[min_step, max_step]`. A non-finite or non-positive bound
/// falls back to `min_step`, matching the uniform-grid behaviour for that
/// stride; as with [`UniformSampler`], excursions shorter than `min_step`
/// may be missed.
pub struct AdaptiveSampler {
    min_step: TimeDelta,
    max_step: TimeDelta,
}

impl AdaptiveSampler {
    /// Creates an adaptive sampler with the given stride bounds.
    ///
    /// # Panics
    ///
    /// Panics if `min_step` is not positive or if `max_step < min_step`.
    pub fn new(min_step: TimeDelta, max_step: TimeDelta) -> Self {
        assert!(min_step.is_positive(), "min_step must be positive");
        assert!(max_step >= min_step, "max_step must not be below min_step");
        Self { min_step, max_step }
    }
}

impl<F> Sampler<F> for AdaptiveSampler
where
    F: RateBounded,
{
    fn sample(&mut self, f: &F, time: Time) -> Result<(f64, TimeDelta), F::Error> {
        let (value, bound) = f.eval_bounded(time)?;
        let step = if bound.is_finite() && bound > 0.0 && value.is_finite() {
            TimeDelta::from_seconds_f64(ADAPTIVE_SAFETY * value.abs() / bound)
                .clamp(self.min_step, self.max_step)
        } else {
            self.min_step
        };
        Ok((value, step))
    }
}

/// Event-detection entry points, provided for every [`DetectFn`].
pub trait DetectFnExt: DetectFn {
    /// Returns a lazy iterator over the zero-crossing events of this function
    /// within `interval`.
    fn iter_events<'a, S>(&'a self, sampler: S, interval: TimeInterval) -> Events<&'a Self, S>
    where
        Self: Sized,
        S: Sampler<&'a Self>,
    {
        Events::new(self, sampler, interval)
    }

    /// Returns a lazy iterator over the zero-crossing events of this function
    /// within `interval`, taking ownership of the function.
    fn into_events<S>(self, sampler: S, interval: TimeInterval) -> Events<Self, S>
    where
        Self: Sized,
        S: Sampler<Self>,
    {
        Events::new(self, sampler, interval)
    }

    /// Detects all zero-crossing events of this function within `interval`.
    fn events<S>(&self, sampler: S, interval: TimeInterval) -> Result<Vec<Event>, DetectError>
    where
        Self: Sized,
        for<'a> S: Sampler<&'a Self>,
    {
        self.iter_events(sampler, interval).collect()
    }

    /// Returns a lazy iterator over the sub-intervals of `interval` where this
    /// function is non-negative.
    fn iter_intervals<'a, S>(&'a self, sampler: S, interval: TimeInterval) -> Intervals<&'a Self, S>
    where
        Self: Sized,
        S: Sampler<&'a Self>,
    {
        Intervals::new(self, sampler, interval)
    }

    /// Returns a lazy iterator over the sub-intervals of `interval` where this
    /// function is non-negative, taking ownership of the function.
    fn into_intervals<S>(self, sampler: S, interval: TimeInterval) -> Intervals<Self, S>
    where
        Self: Sized,
        S: Sampler<Self>,
    {
        Intervals::new(self, sampler, interval)
    }

    /// Detects all sub-intervals of `interval` where this function is
    /// non-negative.
    fn intervals<S>(
        &self,
        sampler: S,
        interval: TimeInterval,
    ) -> Result<Vec<TimeInterval>, DetectError>
    where
        Self: Sized,
        for<'a> S: Sampler<&'a Self>,
    {
        self.iter_intervals(sampler, interval).collect()
    }
}

impl<T> DetectFnExt for T where T: DetectFn {}

/// A lazy iterator over the zero-crossing events of a [`DetectFn`].
pub struct Events<F, S, R = Brent> {
    f: F,
    sampler: S,
    refiner: R,
    interval: TimeInterval,
    current: Time,
    /// The sample at `current` (value and step to the next point), carried
    /// over from the previous bracket so each point is only evaluated once.
    cached: Option<(f64, TimeDelta)>,
    token: Option<CancellationToken>,
}

impl<F, S> Events<F, S>
where
    F: DetectFn,
    S: Sampler<F>,
{
    /// Creates an event iterator for `f` over `interval`.
    pub fn new(f: F, sampler: S, interval: TimeInterval) -> Self {
        let current = interval.start();
        Self {
            f,
            sampler,
            refiner: Brent::default(),
            interval,
            current,
            cached: None,
            token: None,
        }
    }
}

impl<F, S, R> Events<F, S, R> {
    /// Replaces the root-refinement algorithm used to locate crossings within
    /// a bracket.
    pub fn with_refiner<R2>(self, refiner: R2) -> Events<F, S, R2> {
        Events {
            f: self.f,
            sampler: self.sampler,
            refiner,
            interval: self.interval,
            current: self.current,
            cached: self.cached,
            token: self.token,
        }
    }

    /// Makes the scan cancellable via `token`.
    ///
    /// The token is checked once per sample step, so cancellation also
    /// interrupts long event-free stretches, not just the gaps between yielded
    /// items. A cancelled scan yields a single [`DetectError::Cancelled`] and
    /// is exhausted afterwards.
    pub fn with_cancellation(mut self, token: CancellationToken) -> Self {
        self.token = Some(token);
        self
    }

    /// Poisons the iterator so that subsequent calls to `next` return `None`.
    fn poison(&mut self) {
        self.current = self.interval.end();
        self.cached = None;
    }
}

/// Errors that can occur during event detection.
#[derive(Debug, Error)]
pub enum DetectError {
    /// The root-finding algorithm failed.
    #[error(transparent)]
    RootFinder(#[from] RootFinderError),
    /// The detect function returned an error.
    #[error("detect function failed: {0}")]
    DetectFn(#[source] LoxError),
    /// The scan was cancelled via a [`CancellationToken`].
    #[error("event detection was cancelled")]
    Cancelled,
}

impl<F, S, R> Iterator for Events<F, S, R>
where
    F: DetectFn,
    S: Sampler<F>,
    R: Refine<F>,
{
    type Item = Result<Event, DetectError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current >= self.interval.end() {
                return None;
            }
            if self.token.as_ref().is_some_and(|t| t.is_cancelled()) {
                self.poison();
                return Some(Err(DetectError::Cancelled));
            }
            let t0 = self.current;
            // On error, poison the iterator and return.
            let (f0, step) = match self.cached {
                Some(cached) => cached,
                None => match self.sampler.sample(&self.f, t0) {
                    Ok(sample) => sample,
                    Err(e) => {
                        self.poison();
                        return Some(Err(DetectError::DetectFn(e.into())));
                    }
                },
            };
            assert!(
                step.is_positive(),
                "sampler returned a non-positive step at {t0}"
            );
            let mut next = t0 + step;
            if next > self.interval.end() {
                next = self.interval.end();
            }
            self.current = next;
            let f1 = match self.sampler.sample(&self.f, next) {
                Ok(sample) => {
                    self.cached = Some(sample);
                    sample.0
                }
                Err(e) => {
                    self.poison();
                    return Some(Err(DetectError::DetectFn(e.into())));
                }
            };
            let crossing = match ZeroCrossing::new(f0, f1) {
                Some(crossing) => crossing,
                None => continue,
            };
            let bracket = (0.0, (next - t0).to_seconds().to_f64());
            return match self.refiner.refine(&self.f, t0, bracket, (f0, f1)) {
                Ok(t) => {
                    let time = t0 + TimeDelta::from_seconds_f64(t);
                    Some(Ok(Event::new(time, crossing)))
                }
                Err(err) => {
                    self.poison();
                    Some(Err(err.into()))
                }
            };
        }
    }
}

/// A lazy iterator over the sub-intervals where a [`DetectFn`] is non-negative.
///
/// Windows that are already open at the start of the scan interval or still
/// open at its end are clipped to the interval boundaries.
pub struct Intervals<F, S, R = Brent> {
    events: Events<F, S, R>,
    /// Start of the currently open window, if the condition holds.
    open: Option<Time>,
    initialized: bool,
}

impl<F, S> Intervals<F, S>
where
    F: DetectFn,
    S: Sampler<F>,
{
    /// Creates an interval iterator for `f` over `interval`.
    pub fn new(f: F, sampler: S, interval: TimeInterval) -> Self {
        Self {
            events: Events::new(f, sampler, interval),
            open: None,
            initialized: false,
        }
    }
}

impl<F, S, R> Intervals<F, S, R> {
    /// Replaces the root-refinement algorithm used to locate crossings within
    /// a bracket.
    pub fn with_refiner<R2>(self, refiner: R2) -> Intervals<F, S, R2> {
        Intervals {
            events: self.events.with_refiner(refiner),
            open: self.open,
            initialized: self.initialized,
        }
    }

    /// Makes the scan cancellable via `token`.
    ///
    /// See [`Events::with_cancellation`].
    pub fn with_cancellation(mut self, token: CancellationToken) -> Self {
        self.events = self.events.with_cancellation(token);
        self
    }
}

impl<F, S, R> Iterator for Intervals<F, S, R>
where
    F: DetectFn,
    S: Sampler<F>,
    R: Refine<F>,
{
    type Item = Result<TimeInterval, DetectError>;

    fn next(&mut self) -> Option<Self::Item> {
        // The sign at the interval start decides whether a window is already
        // open when the scan begins.
        if !self.initialized {
            self.initialized = true;
            if !self.events.interval.is_empty() {
                let start = self.events.interval.start();
                match self.events.sampler.sample(&self.events.f, start) {
                    Ok(sample) => {
                        // Seed the scan so the start point is only evaluated once.
                        self.events.cached = Some(sample);
                        if sample.0 >= 0.0 {
                            self.open = Some(start);
                        }
                    }
                    Err(e) => {
                        self.events.poison();
                        return Some(Err(DetectError::DetectFn(e.into())));
                    }
                }
            }
        }
        loop {
            return match self.events.next() {
                Some(Ok(event)) => match event.crossing() {
                    ZeroCrossing::Up => {
                        debug_assert!(self.open.is_none(), "crossings must alternate");
                        self.open = Some(event.time());
                        continue;
                    }
                    ZeroCrossing::Down => {
                        let open = self.open.take().expect("crossings must alternate");
                        Some(Ok(TimeInterval::new(open, event.time())))
                    }
                },
                Some(Err(err)) => {
                    self.open = None;
                    Some(Err(err))
                }
                // The scan is exhausted; a still-open window is clipped to the
                // interval end.
                None => self
                    .open
                    .take()
                    .map(|open| Ok(TimeInterval::new(open, self.events.interval.end()))),
            };
        }
    }
}

/// A buffered operand of an interval-algebra combinator.
///
/// Holds at most one window of lookahead and checks the input contract
/// (sorted by start, pairwise disjoint, non-empty) in debug builds.
struct Operand<I> {
    iter: I,
    current: Option<TimeInterval>,
    prev_end: Option<Time>,
}

impl<I> Operand<I>
where
    I: Iterator<Item = Result<TimeInterval, DetectError>>,
{
    fn new(iter: I) -> Self {
        Self {
            iter,
            current: None,
            prev_end: None,
        }
    }

    /// Returns the buffered window, pulling from the underlying iterator if
    /// necessary. `Ok(None)` means the operand is exhausted.
    fn peek(&mut self) -> Result<Option<TimeInterval>, DetectError> {
        if self.current.is_none() {
            match self.iter.next() {
                Some(Ok(window)) => {
                    debug_assert!(
                        !window.is_empty(),
                        "operand yielded an empty window: {window:?}"
                    );
                    debug_assert!(
                        self.prev_end.is_none_or(|end| window.start() >= end),
                        "operand windows must be sorted and disjoint"
                    );
                    self.prev_end = Some(window.end());
                    self.current = Some(window);
                }
                Some(Err(err)) => return Err(err),
                None => {}
            }
        }
        Ok(self.current)
    }

    fn advance(&mut self) {
        self.current = None;
    }
}

/// A lazy iterator over the windows where both operands hold.
///
/// See [`IntervalIterExt::intersect`].
pub struct Intersect<A, B> {
    a: Operand<A>,
    b: Operand<B>,
    fused: bool,
}

impl<A, B> Iterator for Intersect<A, B>
where
    A: Iterator<Item = Result<TimeInterval, DetectError>>,
    B: Iterator<Item = Result<TimeInterval, DetectError>>,
{
    type Item = Result<TimeInterval, DetectError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.fused {
            return None;
        }
        loop {
            let (a, b) = match (self.a.peek(), self.b.peek()) {
                (Ok(Some(a)), Ok(Some(b))) => (a, b),
                (Ok(_), Ok(_)) => {
                    self.fused = true;
                    return None;
                }
                (Err(err), _) | (_, Err(err)) => {
                    self.fused = true;
                    return Some(Err(err));
                }
            };
            let intersection = a.intersect(b);
            // The window that ends first cannot intersect anything later on
            // the other side.
            if a.end() <= b.end() {
                self.a.advance();
            } else {
                self.b.advance();
            }
            if !intersection.is_empty() {
                return Some(Ok(intersection));
            }
        }
    }
}

/// A lazy iterator over the windows where either operand holds.
///
/// Overlapping and touching windows are coalesced, so a window is not yielded
/// before a gap after it has been observed.
///
/// See [`IntervalIterExt::union`].
pub struct Union<A, B> {
    a: Operand<A>,
    b: Operand<B>,
    /// The window currently being coalesced.
    pending: Option<TimeInterval>,
    fused: bool,
}

impl<A, B> Iterator for Union<A, B>
where
    A: Iterator<Item = Result<TimeInterval, DetectError>>,
    B: Iterator<Item = Result<TimeInterval, DetectError>>,
{
    type Item = Result<TimeInterval, DetectError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.fused {
            return None;
        }
        loop {
            let next = match (self.a.peek(), self.b.peek()) {
                (Ok(Some(a)), Ok(Some(b))) => {
                    if a.start() <= b.start() {
                        self.a.advance();
                        a
                    } else {
                        self.b.advance();
                        b
                    }
                }
                (Ok(Some(a)), Ok(None)) => {
                    self.a.advance();
                    a
                }
                (Ok(None), Ok(Some(b))) => {
                    self.b.advance();
                    b
                }
                (Ok(None), Ok(None)) => {
                    self.fused = true;
                    return self.pending.take().map(Ok);
                }
                (Err(err), _) | (_, Err(err)) => {
                    self.fused = true;
                    self.pending = None;
                    return Some(Err(err));
                }
            };
            match self.pending {
                None => self.pending = Some(next),
                Some(pending) if next.start() <= pending.end() => {
                    self.pending = Some(TimeInterval::new(
                        pending.start(),
                        pending.end().max(next.end()),
                    ));
                }
                Some(pending) => {
                    self.pending = Some(next);
                    return Some(Ok(pending));
                }
            }
        }
    }
}

/// A lazy iterator over the windows within a bounding interval where the
/// operand does not hold.
///
/// See [`IntervalIterExt::complement`].
pub struct Complement<I> {
    inner: Operand<I>,
    bound: TimeInterval,
    /// Start of the next candidate gap.
    cursor: Time,
    fused: bool,
}

impl<I> Iterator for Complement<I>
where
    I: Iterator<Item = Result<TimeInterval, DetectError>>,
{
    type Item = Result<TimeInterval, DetectError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.fused {
            return None;
        }
        loop {
            let window = match self.inner.peek() {
                Ok(window) => window,
                Err(err) => {
                    self.fused = true;
                    return Some(Err(err));
                }
            };
            let gap = match window {
                Some(window) => {
                    self.inner.advance();
                    // Clip to the bound; windows may extend past it.
                    let gap = TimeInterval::new(self.cursor, window.start().min(self.bound.end()));
                    self.cursor = self.cursor.max(window.end());
                    gap
                }
                None => {
                    self.fused = true;
                    TimeInterval::new(self.cursor, self.bound.end())
                }
            };
            if !gap.is_empty() {
                return Some(Ok(gap));
            }
            if self.fused {
                return None;
            }
        }
    }
}

/// Staged detection over a lazy interval stream.
///
/// See [`IntervalIterExt::then_within`].
pub struct ThenWithin<A, F, I> {
    outer: A,
    next: F,
    inner: Option<I>,
    fused: bool,
}

impl<A, F, I> Iterator for ThenWithin<A, F, I>
where
    A: Iterator<Item = Result<TimeInterval, DetectError>>,
    F: FnMut(TimeInterval) -> I,
    I: Iterator<Item = Result<TimeInterval, DetectError>>,
{
    type Item = Result<TimeInterval, DetectError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.fused {
                return None;
            }
            if let Some(inner) = self.inner.as_mut() {
                match inner.next() {
                    Some(Ok(window)) => return Some(Ok(window)),
                    Some(Err(e)) => {
                        self.fused = true;
                        return Some(Err(e));
                    }
                    // Inner exhausted: advance to the next outer window.
                    None => self.inner = None,
                }
                continue;
            }
            match self.outer.next() {
                Some(Ok(window)) => self.inner = Some((self.next)(window)),
                Some(Err(e)) => {
                    self.fused = true;
                    return Some(Err(e));
                }
                None => {
                    self.fused = true;
                    return None;
                }
            }
        }
    }
}

/// Set-algebra combinators over lazy interval streams, provided for every
/// iterator of interval results.
///
/// Operands must yield non-empty windows sorted by start and pairwise
/// disjoint — [`Intervals`] guarantees this by construction, and the
/// combinators preserve it, so they nest. The first error from either operand
/// is yielded once and exhausts the combined iterator.
pub trait IntervalIterExt: Iterator<Item = Result<TimeInterval, DetectError>> + Sized {
    /// Returns a lazy iterator over the windows where both `self` and `other`
    /// hold.
    fn intersect<B>(self, other: B) -> Intersect<Self, B>
    where
        B: Iterator<Item = Result<TimeInterval, DetectError>>,
    {
        Intersect {
            a: Operand::new(self),
            b: Operand::new(other),
            fused: false,
        }
    }

    /// Returns a lazy iterator over the windows where either `self` or
    /// `other` holds.
    fn union<B>(self, other: B) -> Union<Self, B>
    where
        B: Iterator<Item = Result<TimeInterval, DetectError>>,
    {
        Union {
            a: Operand::new(self),
            b: Operand::new(other),
            pending: None,
            fused: false,
        }
    }

    /// Returns a lazy iterator that runs `next` **only within** each window
    /// `self` yields, rather than over the whole scan interval.
    ///
    /// This is staged detection: put the cheap detector first and the
    /// expensive one second, so the expensive scan never sees the intervals
    /// the cheap one already ruled out. The result equals the intersection,
    /// but the second detector is only sampled inside surviving windows.
    ///
    /// Deliberately not called `chain` — that name is taken by
    /// [`Iterator::chain`], which concatenates rather than nests.
    fn then_within<F, I>(self, next: F) -> ThenWithin<Self, F, I>
    where
        F: FnMut(TimeInterval) -> I,
        I: Iterator<Item = Result<TimeInterval, DetectError>>,
    {
        ThenWithin {
            outer: self,
            next,
            inner: None,
            fused: false,
        }
    }

    /// Returns a lazy iterator over the windows within `bound` where `self`
    /// does not hold.
    fn complement(self, bound: TimeInterval) -> Complement<Self> {
        Complement {
            inner: Operand::new(self),
            cursor: bound.start(),
            bound,
            fused: false,
        }
    }
}

impl<I> IntervalIterExt for I where I: Iterator<Item = Result<TimeInterval, DetectError>> + Sized {}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use lox_core::math::roots::BracketedNewton;
    use lox_time::{Time, deltas::ToDelta};

    use super::*;

    struct SinDetector;

    impl DetectFn for SinDetector {
        type Error = std::convert::Infallible;

        fn eval(&self, time: Time) -> Result<f64, Self::Error> {
            Ok(time.to_delta().to_seconds().to_f64().sin())
        }
    }

    impl Differentiable for SinDetector {
        fn eval_derivative(&self, time: Time) -> Result<(f64, f64), Self::Error> {
            let t = time.to_delta().to_seconds().to_f64();
            Ok((t.sin(), t.cos()))
        }
    }

    fn seconds(t: f64) -> Time {
        Time::default() + TimeDelta::from_seconds_f64(t)
    }

    #[test]
    fn test_events_sin() {
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let events = SinDetector
            .events(UniformSampler::new(TimeDelta::from_seconds(1)), interval)
            .expect("detection should succeed");

        let expected = [
            (PI, ZeroCrossing::Down),
            (2.0 * PI, ZeroCrossing::Up),
            (3.0 * PI, ZeroCrossing::Down),
        ];
        assert_eq!(events.len(), expected.len());
        for (event, (time, crossing)) in events.iter().zip(expected) {
            let seconds = event.time().to_delta().to_seconds().to_f64();
            assert!(
                (seconds - time).abs() < 1e-5,
                "expected event near {time}, got {seconds}"
            );
            assert_eq!(event.crossing(), crossing);
        }
    }

    #[test]
    fn test_events_detector_is_reusable() {
        let detector = SinDetector;
        let interval = TimeInterval::new(seconds(0.0), seconds(4.0));
        let first = detector
            .events(UniformSampler::new(TimeDelta::from_seconds(1)), interval)
            .expect("detection should succeed");
        let second = detector
            .events(UniformSampler::new(TimeDelta::from_seconds(1)), interval)
            .expect("detection should succeed");
        assert_eq!(first, second);
    }

    #[test]
    fn test_events_empty_interval() {
        let interval = TimeInterval::new(seconds(0.0), seconds(0.0));
        let events = SinDetector
            .events(UniformSampler::new(TimeDelta::from_seconds(1)), interval)
            .expect("detection should succeed");
        assert!(events.is_empty());
    }

    struct FailingDetector;

    impl DetectFn for FailingDetector {
        type Error = LoxError;

        fn eval(&self, time: Time) -> Result<f64, Self::Error> {
            let t = time.to_delta().to_seconds().to_f64();
            if t > 2.5 {
                Err("out of domain".into())
            } else {
                Ok(t - 2.0)
            }
        }
    }

    struct ConstDetector(f64);

    impl DetectFn for ConstDetector {
        type Error = std::convert::Infallible;

        fn eval(&self, _time: Time) -> Result<f64, Self::Error> {
            Ok(self.0)
        }
    }

    fn assert_interval_approx(interval: TimeInterval, start: f64, end: f64) {
        let actual_start = interval.start().to_delta().to_seconds().to_f64();
        let actual_end = interval.end().to_delta().to_seconds().to_f64();
        assert!(
            (actual_start - start).abs() < 1e-5 && (actual_end - end).abs() < 1e-5,
            "expected interval [{start}, {end}), got [{actual_start}, {actual_end})"
        );
    }

    #[test]
    fn test_intervals_sin() {
        // sin is non-negative on [0, π] and [2π, 3π]; the first window is
        // already open at the scan start and clipped to it.
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let intervals = SinDetector
            .intervals(UniformSampler::new(TimeDelta::from_seconds(1)), interval)
            .expect("detection should succeed");
        assert_eq!(intervals.len(), 2);
        assert_interval_approx(intervals[0], 0.0, PI);
        assert_interval_approx(intervals[1], 2.0 * PI, 3.0 * PI);
    }

    #[test]
    fn test_intervals_clipped_at_end() {
        // The window opening at 2π is still open at the scan end.
        let interval = TimeInterval::new(seconds(0.0), seconds(8.0));
        let intervals = SinDetector
            .intervals(UniformSampler::new(TimeDelta::from_seconds(1)), interval)
            .expect("detection should succeed");
        assert_eq!(intervals.len(), 2);
        assert_interval_approx(intervals[0], 0.0, PI);
        assert_interval_approx(intervals[1], 2.0 * PI, 8.0);
    }

    #[test]
    fn test_intervals_no_crossings() {
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let sampler = || UniformSampler::new(TimeDelta::from_seconds(1));

        // An everywhere-positive function holds over the entire interval.
        let all = ConstDetector(1.0)
            .intervals(sampler(), interval)
            .expect("detection should succeed");
        assert_eq!(all, vec![interval]);

        // An everywhere-negative function never holds.
        let none = ConstDetector(-1.0)
            .intervals(sampler(), interval)
            .expect("detection should succeed");
        assert!(none.is_empty());
    }

    #[test]
    fn test_intervals_empty_interval() {
        let interval = TimeInterval::new(seconds(0.0), seconds(0.0));
        let intervals = ConstDetector(1.0)
            .intervals(UniformSampler::new(TimeDelta::from_seconds(1)), interval)
            .expect("detection should succeed");
        assert!(intervals.is_empty());
    }

    #[test]
    fn test_intervals_error_propagates() {
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let mut intervals = FailingDetector
            .into_intervals(UniformSampler::new(TimeDelta::from_seconds(1)), interval);
        // The condition becomes true at t = 2, but the detector fails at
        // t > 2.5 before the window can close.
        let first = intervals.next().expect("should yield the eval error");
        assert!(matches!(first, Err(DetectError::DetectFn(_))));
        assert!(intervals.next().is_none(), "iterator must be poisoned");
    }

    #[test]
    fn test_fn_detect_closure() {
        // A plain closure matches the SinDetector results exactly.
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let sampler = || UniformSampler::new(TimeDelta::from_seconds(1));
        let from_closure = FnDetect(|time: Time| time.to_delta().to_seconds().to_f64().sin())
            .events(sampler(), interval)
            .expect("detection should succeed");
        let from_struct = SinDetector
            .events(sampler(), interval)
            .expect("detection should succeed");
        assert_eq!(from_closure, from_struct);
    }

    #[test]
    fn test_try_fn_detect_closure() {
        #[derive(Debug, Error)]
        #[error("out of domain")]
        struct OutOfDomain;

        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let detector = TryFnDetect(|time: Time| {
            let t = time.to_delta().to_seconds().to_f64();
            if t > 2.5 {
                Err(OutOfDomain)
            } else {
                Ok(t - 2.0)
            }
        });
        let err = detector
            .events(UniformSampler::new(TimeDelta::from_seconds(1)), interval)
            .expect_err("detection should fail past t = 2.5");
        assert!(matches!(err, DetectError::DetectFn(_)));
    }

    #[test]
    fn test_events_with_refiner() {
        // A refiner with a much tighter tolerance still locates the crossings.
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let events: Vec<Event> = SinDetector
            .iter_events(UniformSampler::new(TimeDelta::from_seconds(1)), interval)
            .with_refiner(Brent::default().with_abs_tol(1e-12))
            .collect::<Result<_, _>>()
            .expect("detection should succeed");
        assert_eq!(events.len(), 3);
        let first = events[0].time().to_delta().to_seconds().to_f64();
        assert!(
            (first - PI).abs() < 1e-9,
            "expected tighter root near π, got {first}"
        );
    }

    #[test]
    fn test_derivative_refiner_matches_brent() {
        // Newton-refined events must agree with Brent-refined events on a
        // smooth oracle.
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let sampler = || UniformSampler::new(TimeDelta::from_seconds(1));
        let newton: Vec<Event> = SinDetector
            .iter_events(sampler(), interval)
            .with_refiner(WithDerivative(BracketedNewton::default()))
            .collect::<Result<_, _>>()
            .expect("detection should succeed");
        let brent = SinDetector
            .events(sampler(), interval)
            .expect("detection should succeed");

        assert_eq!(newton.len(), brent.len());
        for (n, b) in newton.iter().zip(&brent) {
            assert_eq!(n.crossing(), b.crossing());
            let tn = n.time().to_delta().to_seconds().to_f64();
            let tb = b.time().to_delta().to_seconds().to_f64();
            assert!(
                (tn - tb).abs() < 1e-5,
                "refiners disagree: newton {tn}, brent {tb}"
            );
        }
    }

    #[test]
    fn test_derivative_refiner_uses_derivative() {
        use std::cell::Cell;

        struct CountingSin(Cell<u32>);

        impl DetectFn for CountingSin {
            type Error = std::convert::Infallible;

            fn eval(&self, time: Time) -> Result<f64, Self::Error> {
                Ok(time.to_delta().to_seconds().to_f64().sin())
            }
        }

        impl Differentiable for CountingSin {
            fn eval_derivative(&self, time: Time) -> Result<(f64, f64), Self::Error> {
                self.0.set(self.0.get() + 1);
                let t = time.to_delta().to_seconds().to_f64();
                Ok((t.sin(), t.cos()))
            }
        }

        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let detector = CountingSin(Cell::new(0));
        let events: Vec<Event> = detector
            .iter_events(UniformSampler::new(TimeDelta::from_seconds(1)), interval)
            .with_refiner(WithDerivative(BracketedNewton::default()))
            .collect::<Result<_, _>>()
            .expect("detection should succeed");
        assert_eq!(events.len(), 3);
        assert!(
            detector.0.get() > 0,
            "the derivative must be evaluated during refinement"
        );
    }

    #[test]
    fn test_events_cancellation() {
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let token = CancellationToken::new();
        let mut events = SinDetector
            .into_events(UniformSampler::new(TimeDelta::from_seconds(1)), interval)
            .with_cancellation(token.clone());
        assert!(events.next().expect("first event").is_ok());
        token.cancel();
        let cancelled = events.next().expect("should yield the cancellation");
        assert!(matches!(cancelled, Err(DetectError::Cancelled)));
        assert!(events.next().is_none(), "iterator must be exhausted");
    }

    #[test]
    fn test_events_cancellation_interrupts_event_free_scan() {
        // No zero crossings anywhere: without the per-step token check a
        // single `next` call would scan the entire interval.
        let interval = TimeInterval::new(seconds(0.0), seconds(1e9));
        let token = CancellationToken::new();
        token.cancel();
        let mut events = ConstDetector(1.0)
            .into_events(UniformSampler::new(TimeDelta::from_seconds(1)), interval)
            .with_cancellation(token);
        let cancelled = events.next().expect("should yield the cancellation");
        assert!(matches!(cancelled, Err(DetectError::Cancelled)));
        assert!(events.next().is_none(), "iterator must be exhausted");
    }

    #[test]
    fn test_intervals_cancellation() {
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let token = CancellationToken::new();
        let mut intervals = SinDetector
            .into_intervals(UniformSampler::new(TimeDelta::from_seconds(1)), interval)
            .with_cancellation(token.clone());
        assert!(intervals.next().expect("first window").is_ok());
        token.cancel();
        let cancelled = intervals.next().expect("should yield the cancellation");
        assert!(matches!(cancelled, Err(DetectError::Cancelled)));
        assert!(intervals.next().is_none(), "iterator must be exhausted");
    }

    #[test]
    fn test_lipschitz_sampler_step_is_clamped() {
        // With max_slope = 1 the ideal step equals the distance from zero.
        let mut sampler = LipschitzSampler::new(
            1.0,
            TimeDelta::from_seconds_f64(0.1),
            TimeDelta::from_seconds_f64(10.0),
        );
        let t = seconds(0.0);
        let step = |sampler: &mut LipschitzSampler, value: f64| {
            let (v, step) = sampler.sample(&ConstDetector(value), t).unwrap();
            assert_eq!(v, value);
            step
        };
        assert_eq!(step(&mut sampler, 0.5), TimeDelta::from_seconds_f64(0.5));
        assert_eq!(step(&mut sampler, -0.5), TimeDelta::from_seconds_f64(0.5));
        assert_eq!(step(&mut sampler, 0.0), TimeDelta::from_seconds_f64(0.1));
        assert_eq!(step(&mut sampler, 100.0), TimeDelta::from_seconds_f64(10.0));
    }

    #[test]
    fn test_adaptive_sampler_matches_uniform_with_fewer_samples() {
        use std::cell::Cell;

        // sin with the true rate bound |cos| ≤ 1, counting evaluations.
        struct BoundedSin(Cell<u32>);

        impl DetectFn for BoundedSin {
            type Error = std::convert::Infallible;

            fn eval(&self, time: Time) -> Result<f64, Self::Error> {
                Ok(time.to_delta().to_seconds().to_f64().sin())
            }
        }

        impl RateBounded for BoundedSin {
            fn eval_bounded(&self, time: Time) -> Result<(f64, f64), Self::Error> {
                self.0.set(self.0.get() + 1);
                Ok((time.to_delta().to_seconds().to_f64().sin(), 1.0))
            }
        }

        let interval = TimeInterval::new(seconds(0.0), seconds(100.0));
        let min_step = TimeDelta::from_seconds_f64(0.01);
        let detector = BoundedSin(Cell::new(0));
        let adaptive = detector
            .events(
                AdaptiveSampler::new(min_step, TimeDelta::from_seconds(100)),
                interval,
            )
            .expect("detection should succeed");
        let uniform = SinDetector
            .events(UniformSampler::new(min_step), interval)
            .expect("detection should succeed");

        assert_eq!(adaptive.len(), uniform.len());
        for (a, u) in adaptive.iter().zip(&uniform) {
            assert_eq!(a.crossing(), u.crossing());
            let ta = a.time().to_delta().to_seconds().to_f64();
            let tu = u.time().to_delta().to_seconds().to_f64();
            assert!(
                (ta - tu).abs() < 1e-5,
                "samplers disagree: adaptive {ta}, uniform {tu}"
            );
        }
        // 100 s at a 0.01 s uniform grid is 10,000 samples; the rate bound
        // must cut that by orders of magnitude.
        assert!(
            detector.0.get() < 1000,
            "adaptive sampling took {} samples",
            detector.0.get()
        );
    }

    #[test]
    fn test_lipschitz_sampler_matches_uniform() {
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let adaptive = SinDetector
            .events(
                LipschitzSampler::new(
                    1.0,
                    TimeDelta::from_seconds_f64(0.1),
                    TimeDelta::from_seconds(5),
                ),
                interval,
            )
            .expect("detection should succeed");
        let uniform = SinDetector
            .events(UniformSampler::new(TimeDelta::from_seconds(1)), interval)
            .expect("detection should succeed");

        assert_eq!(adaptive.len(), uniform.len());
        for (a, u) in adaptive.iter().zip(&uniform) {
            assert_eq!(a.crossing(), u.crossing());
            let ta = a.time().to_delta().to_seconds().to_f64();
            let tu = u.time().to_delta().to_seconds().to_f64();
            assert!(
                (ta - tu).abs() < 1e-5,
                "samplers disagree: lipschitz {ta}, uniform {tu}"
            );
        }
    }

    struct CosDetector;

    impl DetectFn for CosDetector {
        type Error = std::convert::Infallible;

        fn eval(&self, time: Time) -> Result<f64, Self::Error> {
            Ok(time.to_delta().to_seconds().to_f64().cos())
        }
    }

    fn windows(
        spans: &[(f64, f64)],
    ) -> impl Iterator<Item = Result<TimeInterval, DetectError>> + use<> {
        spans
            .iter()
            .map(|&(start, end)| Ok(TimeInterval::new(seconds(start), seconds(end))))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn sampler() -> UniformSampler {
        UniformSampler::new(TimeDelta::from_seconds(1))
    }

    #[test]
    fn test_intersect_sin_cos() {
        // sin ≥ 0 on [0, π] ∪ [2π, 3π]; cos ≥ 0 on [0, π/2] ∪ [3π/2, 5π/2]
        // ∪ [7π/2, 10] — both hold on [0, π/2] and [2π, 5π/2].
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let intersection: Vec<TimeInterval> = SinDetector
            .iter_intervals(sampler(), interval)
            .intersect(CosDetector.iter_intervals(sampler(), interval))
            .collect::<Result<_, _>>()
            .expect("detection should succeed");
        assert_eq!(intersection.len(), 2);
        assert_interval_approx(intersection[0], 0.0, PI / 2.0);
        assert_interval_approx(intersection[1], 2.0 * PI, 2.5 * PI);
    }

    #[test]
    fn test_then_within_matches_intersect() {
        // Staged detection restricts the second scan to the first's windows, so
        // it must agree with the intersection while sampling far less.
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let staged: Vec<TimeInterval> = SinDetector
            .iter_intervals(sampler(), interval)
            .then_within(|window| CosDetector.iter_intervals(sampler(), window))
            .collect::<Result<_, _>>()
            .expect("detection should succeed");
        assert_eq!(staged.len(), 2);
        assert_interval_approx(staged[0], 0.0, PI / 2.0);
        assert_interval_approx(staged[1], 2.0 * PI, 2.5 * PI);
    }

    #[test]
    fn test_then_within_propagates_inner_error() {
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let mut it = SinDetector
            .iter_intervals(sampler(), interval)
            .then_within(|window| FailingDetector.iter_intervals(sampler(), window));
        assert!(matches!(it.next(), Some(Err(_))));
        // The stream fuses after an error.
        assert!(it.next().is_none());
    }

    #[test]
    fn test_union_sin_cos() {
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let union: Vec<TimeInterval> = SinDetector
            .iter_intervals(sampler(), interval)
            .union(CosDetector.iter_intervals(sampler(), interval))
            .collect::<Result<_, _>>()
            .expect("detection should succeed");
        // sin ≥ 0 on [0, π] ∪ [2π, 3π]; cos ≥ 0 on [0, π/2] ∪ [3π/2, 5π/2]
        // (7π/2 ≈ 10.996 lies outside the scan) — the second and third
        // windows coalesce.
        assert_eq!(union.len(), 2);
        assert_interval_approx(union[0], 0.0, PI);
        assert_interval_approx(union[1], 1.5 * PI, 3.0 * PI);
    }

    #[test]
    fn test_complement_sin() {
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let complement: Vec<TimeInterval> = SinDetector
            .iter_intervals(sampler(), interval)
            .complement(interval)
            .collect::<Result<_, _>>()
            .expect("detection should succeed");
        assert_eq!(complement.len(), 2);
        assert_interval_approx(complement[0], PI, 2.0 * PI);
        assert_interval_approx(complement[1], 3.0 * PI, 10.0);
    }

    #[test]
    fn test_union_coalesces_touching_windows() {
        let union: Vec<TimeInterval> = windows(&[(0.0, 1.0), (2.0, 3.0)])
            .union(windows(&[(1.0, 2.0)]))
            .collect::<Result<_, _>>()
            .expect("union should succeed");
        assert_eq!(union.len(), 1);
        assert_interval_approx(union[0], 0.0, 3.0);
    }

    #[test]
    fn test_algebra_with_empty_operand() {
        let empty = || windows(&[]);
        let intersection: Vec<TimeInterval> = windows(&[(0.0, 1.0)])
            .intersect(empty())
            .collect::<Result<_, _>>()
            .expect("intersection should succeed");
        assert!(intersection.is_empty());

        let union: Vec<TimeInterval> = windows(&[(0.0, 1.0)])
            .union(empty())
            .collect::<Result<_, _>>()
            .expect("union should succeed");
        assert_eq!(union.len(), 1);
        assert_interval_approx(union[0], 0.0, 1.0);

        let bound = TimeInterval::new(seconds(0.0), seconds(1.0));
        let complement: Vec<TimeInterval> = empty()
            .complement(bound)
            .collect::<Result<_, _>>()
            .expect("complement should succeed");
        assert_eq!(complement, vec![bound]);
    }

    #[test]
    fn test_complement_clips_to_bound() {
        // The operand extends past the bound on both sides.
        let bound = TimeInterval::new(seconds(0.0), seconds(10.0));
        let complement: Vec<TimeInterval> = windows(&[(-2.0, 1.0), (4.0, 12.0)])
            .complement(bound)
            .collect::<Result<_, _>>()
            .expect("complement should succeed");
        assert_eq!(complement.len(), 1);
        assert_interval_approx(complement[0], 1.0, 4.0);
    }

    #[test]
    fn test_algebra_nests() {
        // (sin ∧ cos) ∨ ¬sin within [0, 10].
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let nested: Vec<TimeInterval> = SinDetector
            .iter_intervals(sampler(), interval)
            .intersect(CosDetector.iter_intervals(sampler(), interval))
            .union(
                SinDetector
                    .iter_intervals(sampler(), interval)
                    .complement(interval),
            )
            .collect::<Result<_, _>>()
            .expect("detection should succeed");
        assert_eq!(nested.len(), 3);
        assert_interval_approx(nested[0], 0.0, PI / 2.0);
        assert_interval_approx(nested[1], PI, 2.5 * PI);
        assert_interval_approx(nested[2], 3.0 * PI, 10.0);
    }

    #[test]
    fn test_algebra_error_fuses() {
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let mut intersection = FailingDetector
            .into_intervals(sampler(), interval)
            .intersect(windows(&[(0.0, 10.0)]));
        let first = intersection.next().expect("should yield the eval error");
        assert!(matches!(first, Err(DetectError::DetectFn(_))));
        assert!(intersection.next().is_none(), "iterator must be fused");

        let mut union = FailingDetector
            .into_intervals(sampler(), interval)
            .union(windows(&[(0.0, 10.0)]));
        let first = union.next().expect("should yield the eval error");
        assert!(matches!(first, Err(DetectError::DetectFn(_))));
        assert!(union.next().is_none(), "iterator must be fused");

        let mut complement = FailingDetector
            .into_intervals(sampler(), interval)
            .complement(interval);
        let first = complement.next().expect("should yield the eval error");
        assert!(matches!(first, Err(DetectError::DetectFn(_))));
        assert!(complement.next().is_none(), "iterator must be fused");
    }

    #[test]
    fn test_events_error_poisons_iterator() {
        let interval = TimeInterval::new(seconds(0.0), seconds(10.0));
        let mut events =
            FailingDetector.into_events(UniformSampler::new(TimeDelta::from_seconds(1)), interval);
        // The crossing at t = 2 is found before the failure at t > 2.5.
        let first = events.next().expect("should yield the crossing at t = 2");
        assert!(first.is_ok());
        let second = events.next().expect("should yield the eval error");
        assert!(matches!(second, Err(DetectError::DetectFn(_))));
        assert!(events.next().is_none(), "iterator must be poisoned");
    }
}
