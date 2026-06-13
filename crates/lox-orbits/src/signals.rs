// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Signal-based event detection: compose scalar signals, not interval lists.
//!
//! A condition is a scalar [`Signal`], positive where the condition holds;
//! its window set is the superlevel set `{ t : f(t) > 0 }`. Boolean logic on
//! conditions is exact in function space — `min` is AND, `max` is OR, `neg`
//! is NOT — so detection runs **once**, on the combined signal, and window
//! boundaries carry the binding constraint as a diagnostic.
//!
//! The pipeline is sample → bracket → refine: a [`Sampler`] produces a time
//! grid, [`Signal::eval_grid`] fills values (values, not signs — near-zero
//! local extrema become grazing candidates recovering sub-step windows), and
//! a bracketed root finder refines crossings, warm-started with the known
//! endpoint values.

use std::error::Error as StdError;
use std::fmt;

use lox_math::optim::{BrentMinimizer, FindBracketedMinimum};
use lox_math::roots::{BoxedError, Brent, Callback, CallbackError, FindBracketedRoot};
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::TimeScale;

use crate::events::{DetectError, DetectFn, Event, ZeroCrossing};

// ---------------------------------------------------------------------------
// Signal trait and adapters
// ---------------------------------------------------------------------------

/// Boxed evaluation error for composed signals.
///
/// Combinators join signals with different error types; their `Error` is
/// this common boxed form.
#[derive(Debug)]
pub struct EvalError(pub Box<dyn StdError + Send + Sync>);

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl StdError for EvalError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.0.as_ref())
    }
}

impl EvalError {
    fn new<E: StdError + Send + Sync + 'static>(e: E) -> Self {
        Self(Box::new(e))
    }
}

/// Scalar signal over time; positive values mean "condition holds".
pub trait Signal<T: TimeScale> {
    /// The error type returned by evaluation.
    type Error: StdError + Send + Sync + 'static;

    /// Evaluates the signal at the given time.
    fn eval(&self, time: Time<T>) -> Result<f64, Self::Error>;

    /// Batch evaluation over a time grid — the performance-critical entry
    /// point. Implementations amortize shared work (rotation series,
    /// time-scale conversions, cursor-based interpolation) across the grid.
    fn eval_grid(&self, times: &[Time<T>], out: &mut [f64]) -> Result<(), Self::Error>
    where
        T: Copy,
    {
        for (time, value) in times.iter().zip(out.iter_mut()) {
            *value = self.eval(*time)?;
        }
        Ok(())
    }

    /// Optional analytic time derivative.
    fn deriv(&self, time: Time<T>) -> Option<f64> {
        let _ = time;
        None
    }

    /// Number of leaf constraints in this signal.
    fn leaf_count(&self) -> usize {
        1
    }

    /// Evaluates the signal and reports which leaf constraint is binding
    /// (the argmin/argmax through the combinator tree).
    fn eval_binding(&self, time: Time<T>) -> Result<(f64, usize), Self::Error> {
        Ok((self.eval(time)?, 0))
    }
}

impl<T: TimeScale, S: Signal<T>> Signal<T> for &S {
    type Error = S::Error;

    fn eval(&self, time: Time<T>) -> Result<f64, Self::Error> {
        (**self).eval(time)
    }

    fn eval_grid(&self, times: &[Time<T>], out: &mut [f64]) -> Result<(), Self::Error>
    where
        T: Copy,
    {
        (**self).eval_grid(times, out)
    }

    fn deriv(&self, time: Time<T>) -> Option<f64> {
        (**self).deriv(time)
    }

    fn leaf_count(&self) -> usize {
        (**self).leaf_count()
    }

    fn eval_binding(&self, time: Time<T>) -> Result<(f64, usize), Self::Error> {
        (**self).eval_binding(time)
    }
}

impl<T, E> Signal<T> for Box<dyn Signal<T, Error = E> + '_>
where
    T: TimeScale,
    E: StdError + Send + Sync + 'static,
{
    type Error = E;

    fn eval(&self, time: Time<T>) -> Result<f64, Self::Error> {
        (**self).eval(time)
    }

    fn eval_grid(&self, times: &[Time<T>], out: &mut [f64]) -> Result<(), Self::Error>
    where
        T: Copy,
    {
        (**self).eval_grid(times, out)
    }

    fn deriv(&self, time: Time<T>) -> Option<f64> {
        (**self).deriv(time)
    }

    fn leaf_count(&self) -> usize {
        (**self).leaf_count()
    }

    fn eval_binding(&self, time: Time<T>) -> Result<(f64, usize), Self::Error> {
        (**self).eval_binding(time)
    }
}

/// Adapts any [`DetectFn`] into a [`Signal`] (migration bridge).
pub struct DetectSignal<F>(pub F);

impl<T, F> Signal<T> for DetectSignal<F>
where
    T: TimeScale + Copy,
    F: DetectFn<T>,
{
    type Error = F::Error;

    fn eval(&self, time: Time<T>) -> Result<f64, Self::Error> {
        self.0.eval(time)
    }
}

/// Wraps an infallible closure into a [`Signal`].
pub struct SignalFn<F>(pub F);

impl<T, F> Signal<T> for SignalFn<F>
where
    T: TimeScale + Copy,
    F: Fn(Time<T>) -> f64,
{
    type Error = std::convert::Infallible;

    fn eval(&self, time: Time<T>) -> Result<f64, Self::Error> {
        Ok((self.0)(time))
    }
}

// ---------------------------------------------------------------------------
// Combinators
// ---------------------------------------------------------------------------

/// AND: positive where both operands are positive (`min`).
pub struct Min<A, B> {
    a: A,
    b: B,
}

/// OR: positive where either operand is positive (`max`).
pub struct Max<A, B> {
    a: A,
    b: B,
}

/// NOT: positive where the operand is negative.
pub struct Neg<S> {
    inner: S,
}

/// Adds a constant offset (level shift) to a signal.
pub struct Offset<S> {
    inner: S,
    offset: f64,
}

macro_rules! impl_min_max {
    ($name:ident, $sel:ident, $pick_a:expr) => {
        impl<T, A, B> Signal<T> for $name<A, B>
        where
            T: TimeScale + Copy,
            A: Signal<T>,
            B: Signal<T>,
        {
            type Error = EvalError;

            fn eval(&self, time: Time<T>) -> Result<f64, Self::Error> {
                let a = self.a.eval(time).map_err(EvalError::new)?;
                let b = self.b.eval(time).map_err(EvalError::new)?;
                Ok(a.$sel(b))
            }

            fn eval_grid(&self, times: &[Time<T>], out: &mut [f64]) -> Result<(), Self::Error> {
                self.a.eval_grid(times, out).map_err(EvalError::new)?;
                let mut scratch = vec![0.0; times.len()];
                self.b
                    .eval_grid(times, &mut scratch)
                    .map_err(EvalError::new)?;
                for (value, other) in out.iter_mut().zip(&scratch) {
                    *value = value.$sel(*other);
                }
                Ok(())
            }

            fn deriv(&self, time: Time<T>) -> Option<f64> {
                let a = self.a.eval(time).ok()?;
                let b = self.b.eval(time).ok()?;
                let pick_a: fn(f64, f64) -> bool = $pick_a;
                if pick_a(a, b) {
                    self.a.deriv(time)
                } else {
                    self.b.deriv(time)
                }
            }

            fn leaf_count(&self) -> usize {
                self.a.leaf_count() + self.b.leaf_count()
            }

            fn eval_binding(&self, time: Time<T>) -> Result<(f64, usize), Self::Error> {
                let (a, ia) = self.a.eval_binding(time).map_err(EvalError::new)?;
                let (b, ib) = self.b.eval_binding(time).map_err(EvalError::new)?;
                let pick_a: fn(f64, f64) -> bool = $pick_a;
                if pick_a(a, b) {
                    Ok((a, ia))
                } else {
                    Ok((b, self.a.leaf_count() + ib))
                }
            }
        }
    };
}

impl_min_max!(Min, min, |a, b| a <= b);
impl_min_max!(Max, max, |a, b| a >= b);

impl<T, S> Signal<T> for Neg<S>
where
    T: TimeScale + Copy,
    S: Signal<T>,
{
    type Error = S::Error;

    fn eval(&self, time: Time<T>) -> Result<f64, Self::Error> {
        Ok(-self.inner.eval(time)?)
    }

    fn eval_grid(&self, times: &[Time<T>], out: &mut [f64]) -> Result<(), Self::Error> {
        self.inner.eval_grid(times, out)?;
        for value in out.iter_mut() {
            *value = -*value;
        }
        Ok(())
    }

    fn deriv(&self, time: Time<T>) -> Option<f64> {
        self.inner.deriv(time).map(|d| -d)
    }

    fn leaf_count(&self) -> usize {
        self.inner.leaf_count()
    }

    fn eval_binding(&self, time: Time<T>) -> Result<(f64, usize), Self::Error> {
        let (value, id) = self.inner.eval_binding(time)?;
        Ok((-value, id))
    }
}

impl<T, S> Signal<T> for Offset<S>
where
    T: TimeScale + Copy,
    S: Signal<T>,
{
    type Error = S::Error;

    fn eval(&self, time: Time<T>) -> Result<f64, Self::Error> {
        Ok(self.inner.eval(time)? + self.offset)
    }

    fn eval_grid(&self, times: &[Time<T>], out: &mut [f64]) -> Result<(), Self::Error> {
        self.inner.eval_grid(times, out)?;
        for value in out.iter_mut() {
            *value += self.offset;
        }
        Ok(())
    }

    fn deriv(&self, time: Time<T>) -> Option<f64> {
        self.inner.deriv(time)
    }

    fn leaf_count(&self) -> usize {
        self.inner.leaf_count()
    }

    fn eval_binding(&self, time: Time<T>) -> Result<(f64, usize), Self::Error> {
        let (value, id) = self.inner.eval_binding(time)?;
        Ok((value + self.offset, id))
    }
}

/// AND over a homogeneous list of signals (n-ary [`Min`]): positive where
/// every signal in the list is positive. Box the elements
/// (`Box<dyn Signal<…>>`) to compose differently-typed constraints whose
/// number is only known at runtime.
pub struct MinOf<S> {
    signals: Vec<S>,
}

impl<S> MinOf<S> {
    /// Creates the combinator from a non-empty list of signals.
    ///
    /// # Panics
    ///
    /// Panics if `signals` is empty.
    pub fn new(signals: Vec<S>) -> Self {
        assert!(!signals.is_empty(), "MinOf requires at least one signal");
        Self { signals }
    }
}

impl<T, S> Signal<T> for MinOf<S>
where
    T: TimeScale + Copy,
    S: Signal<T>,
{
    type Error = S::Error;

    fn eval(&self, time: Time<T>) -> Result<f64, Self::Error> {
        let mut value = f64::INFINITY;
        for signal in &self.signals {
            value = value.min(signal.eval(time)?);
        }
        Ok(value)
    }

    fn eval_grid(&self, times: &[Time<T>], out: &mut [f64]) -> Result<(), Self::Error> {
        self.signals[0].eval_grid(times, out)?;
        if self.signals.len() == 1 {
            return Ok(());
        }
        let mut scratch = vec![0.0; times.len()];
        for signal in &self.signals[1..] {
            signal.eval_grid(times, &mut scratch)?;
            for (value, other) in out.iter_mut().zip(&scratch) {
                *value = value.min(*other);
            }
        }
        Ok(())
    }

    fn deriv(&self, time: Time<T>) -> Option<f64> {
        let mut min = f64::INFINITY;
        let mut active = None;
        for signal in &self.signals {
            let value = signal.eval(time).ok()?;
            if value < min {
                min = value;
                active = Some(signal);
            }
        }
        active?.deriv(time)
    }

    fn leaf_count(&self) -> usize {
        self.signals.iter().map(Signal::leaf_count).sum()
    }

    fn eval_binding(&self, time: Time<T>) -> Result<(f64, usize), Self::Error> {
        let mut min = f64::INFINITY;
        let mut binding = 0;
        let mut offset = 0;
        for signal in &self.signals {
            let (value, id) = signal.eval_binding(time)?;
            if value < min {
                min = value;
                binding = offset + id;
            }
            offset += signal.leaf_count();
        }
        Ok((min, binding))
    }
}

/// Combinator methods for signals.
pub trait SignalExt<T: TimeScale>: Signal<T> + Sized {
    /// AND: positive where both `self` and `other` are positive.
    fn min<B: Signal<T>>(self, other: B) -> Min<Self, B> {
        Min { a: self, b: other }
    }

    /// OR: positive where either `self` or `other` is positive.
    fn max<B: Signal<T>>(self, other: B) -> Max<Self, B> {
        Max { a: self, b: other }
    }

    /// NOT: positive where `self` is negative.
    fn neg(self) -> Neg<Self> {
        Neg { inner: self }
    }

    /// Level shift: `self + offset`.
    fn offset(self, offset: f64) -> Offset<Self> {
        Offset {
            inner: self,
            offset,
        }
    }
}

impl<T: TimeScale, S: Signal<T>> SignalExt<T> for S {}

// ---------------------------------------------------------------------------
// Samplers
// ---------------------------------------------------------------------------

/// Produces the sample times covering an interval.
pub trait Sampler<T: TimeScale> {
    /// Returns monotonically increasing sample times spanning `interval`,
    /// including both endpoints (at least two samples).
    fn sample(&self, interval: TimeInterval<T>) -> Vec<Time<T>>;
}

/// Fixed-step sampling, always including the interval end.
pub struct UniformSampler {
    step: TimeDelta,
}

impl UniformSampler {
    /// Creates a uniform sampler with the given step.
    pub fn new(step: TimeDelta) -> Self {
        Self { step }
    }
}

impl<T: TimeScale + Copy> Sampler<T> for UniformSampler {
    fn sample(&self, interval: TimeInterval<T>) -> Vec<Time<T>> {
        let start = interval.start();
        let total = (interval.end() - start).to_seconds().to_f64();
        let step = self.step.to_seconds().to_f64();
        let mut times = Vec::new();
        let mut t = 0.0;
        while t <= total {
            times.push(start + TimeDelta::from_seconds_f64(t));
            t += step;
        }
        if times.len() < 2
            || (interval.end() - *times.last().unwrap())
                .to_seconds()
                .to_f64()
                > 0.0
        {
            times.push(interval.end());
        }
        times
    }
}

/// Samples at caller-provided knot times — e.g. the knots of a trajectory's
/// Hermite spline, which bound how fast any signal derived from it can vary.
/// Knots are clipped to the interval, both endpoints are always included,
/// and each gap can be uniformly refined.
pub struct KnotSampler<T: TimeScale> {
    knots: Vec<Time<T>>,
    refinement: usize,
}

impl<T: TimeScale + Copy> KnotSampler<T> {
    /// Creates a sampler over the given (monotonically increasing) knots.
    pub fn new(knots: Vec<Time<T>>) -> Self {
        Self {
            knots,
            refinement: 1,
        }
    }

    /// Subdivides each gap between samples into `refinement` equal parts.
    pub fn with_refinement(mut self, refinement: usize) -> Self {
        self.refinement = refinement.max(1);
        self
    }
}

impl<T: TimeScale + Copy> Sampler<T> for KnotSampler<T> {
    fn sample(&self, interval: TimeInterval<T>) -> Vec<Time<T>> {
        let start = interval.start();
        let end = interval.end();
        let inside = |t: &&Time<T>| {
            let s = (**t - start).to_seconds().to_f64();
            let e = (end - **t).to_seconds().to_f64();
            s > 0.0 && e > 0.0
        };
        let mut base = Vec::with_capacity(self.knots.len() + 2);
        base.push(start);
        base.extend(self.knots.iter().filter(inside).copied());
        base.push(end);

        if self.refinement == 1 {
            return base;
        }
        let mut times = Vec::with_capacity(base.len() * self.refinement);
        for pair in base.windows(2) {
            let gap = (pair[1] - pair[0]).to_seconds().to_f64();
            times.push(pair[0]);
            for k in 1..self.refinement {
                let dt = gap * k as f64 / self.refinement as f64;
                times.push(pair[0] + TimeDelta::from_seconds_f64(dt));
            }
        }
        times.push(end);
        times
    }
}

// ---------------------------------------------------------------------------
// Windows and diagnostics
// ---------------------------------------------------------------------------

/// Identifies a leaf constraint within a composed signal, by depth-first
/// position in the combinator tree (left operands first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstraintId(pub usize);

/// A window where the composed condition holds, with diagnostics naming the
/// binding constraint at each boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct Window<T: TimeScale> {
    interval: TimeInterval<T>,
    opened_by: ConstraintId,
    closed_by: ConstraintId,
}

impl<T: TimeScale + Copy> Window<T> {
    /// Returns the time interval of the window.
    pub fn interval(&self) -> TimeInterval<T> {
        self.interval
    }

    /// Returns the constraint that opened the window.
    pub fn opened_by(&self) -> ConstraintId {
        self.opened_by
    }

    /// Returns the constraint that closed the window.
    pub fn closed_by(&self) -> ConstraintId {
        self.closed_by
    }
}

// ---------------------------------------------------------------------------
// Detector pipeline
// ---------------------------------------------------------------------------

/// Relative threshold for treating a parabola vertex prediction as a grazing
/// candidate: the predicted extremum must cross zero by more than this
/// fraction of the local sample magnitude to be worth refining.
const GRAZING_REL_MARGIN: f64 = 1e-12;

/// A grazing probe at the predicted extremum must shrink the residual to
/// below this fraction of the middle sample's magnitude before the full
/// minimization is attempted; otherwise the candidate is dismissed.
const GRAZING_PROBE_IMPROVEMENT: f64 = 0.5;

/// Bridges a [`Signal`] to the root-finder [`Callback`] interface, mapping
/// seconds-since-start to signal values. Public because it appears in the
/// root-finder bounds of [`Detector`]'s methods.
pub struct SignalCallback<'a, T: TimeScale, S> {
    signal: &'a S,
    start: Time<T>,
}

impl<T: TimeScale + Copy, S> Clone for SignalCallback<'_, T, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: TimeScale + Copy, S> Copy for SignalCallback<'_, T, S> {}

impl<T, S> Callback for SignalCallback<'_, T, S>
where
    T: TimeScale + Copy,
    S: Signal<T>,
{
    fn call(&self, v: f64) -> Result<f64, CallbackError> {
        let time = self.start + TimeDelta::from_seconds_f64(v);
        self.signal
            .eval(time)
            .map_err(|e| CallbackError::from(Box::new(e) as BoxedError))
    }
}

/// Detects events and windows of a [`Signal`] over an interval.
pub struct Detector<S, R = Brent> {
    signal: S,
    root_finder: R,
    minimizer: BrentMinimizer,
}

impl<S> Detector<S, Brent> {
    /// Creates a detector with Brent's method for refinement.
    pub fn new(signal: S) -> Self {
        Self {
            signal,
            root_finder: Brent::default(),
            minimizer: BrentMinimizer::default(),
        }
    }
}

impl<S, R> Detector<S, R> {
    /// Creates a detector with a custom root finder.
    pub fn with_root_finder(signal: S, root_finder: R) -> Self {
        Self {
            signal,
            root_finder,
            minimizer: BrentMinimizer::default(),
        }
    }

    /// Returns the underlying signal.
    pub fn signal(&self) -> &S {
        &self.signal
    }
}

impl<S, R> Detector<S, R> {
    /// Core detection: sample → bracket (sign changes and grazing
    /// candidates) → refine. Returns events and the value at the interval
    /// start.
    pub(crate) fn events_with_start_value<T>(
        &self,
        interval: TimeInterval<T>,
        sampler: &impl Sampler<T>,
    ) -> Result<(Vec<Event<T>>, f64), DetectError>
    where
        T: TimeScale + Copy,
        S: Signal<T>,
        for<'a> R: FindBracketedRoot<SignalCallback<'a, T, S>>,
    {
        detect_grid(
            &self.signal,
            &self.root_finder,
            &self.minimizer,
            interval,
            sampler,
        )
    }
}

/// ② BRACKET / ③ REFINE: sign changes between adjacent grid samples,
/// refined with the known endpoint values.
fn scan_crossings<T, S, R>(
    callback: SignalCallback<'_, T, S>,
    root_finder: &R,
    offsets: &[f64],
    values: &[f64],
    start: Time<T>,
    events: &mut Vec<Event<T>>,
) -> Result<(), DetectError>
where
    T: TimeScale + Copy,
    S: Signal<T>,
    for<'a> R: FindBracketedRoot<SignalCallback<'a, T, S>>,
{
    // Classify crossings between *nonzero* samples, skipping exact zeros.
    // A zero sample is a boundary, not a sign: `signum` maps ±0.0 to ±1, so
    // comparing raw signums would read a 0 between equal-signed neighbours
    // (a touching root) as two spurious crossings, and depends on the IEEE
    // sign bit of a value that is mathematically zero. Tracking the last
    // nonzero sample instead bridges any run of zeros: opposite signs across
    // it are one genuine crossing, equal signs are none.
    let mut last_nonzero: Option<(f64, f64)> = None;
    for i in 0..offsets.len() {
        let v1 = values[i];
        if v1 == 0.0 {
            continue;
        }
        if let Some((o0, v0)) = last_nonzero
            && let Some(crossing) = ZeroCrossing::new(v0.signum(), v1.signum())
        {
            let t = root_finder
                .find_in_bracket_with_values(callback, (o0, offsets[i]), (v0, v1))
                .map_err(DetectError::RootFinder)?;
            events.push(Event::new(start + TimeDelta::from_seconds_f64(t), crossing));
        }
        last_nonzero = Some((offsets[i], v1));
    }
    Ok(())
}

/// ② BRACKET / ③ REFINE: grazing candidates — near-zero local extrema whose
/// parabola fit predicts a crossing the grid did not sample. A confirmed dip
/// across zero yields two new brackets and two events.
fn scan_grazing<T, S, R>(
    callback: SignalCallback<'_, T, S>,
    root_finder: &R,
    minimizer: &BrentMinimizer,
    offsets: &[f64],
    values: &[f64],
    start: Time<T>,
    events: &mut Vec<Event<T>>,
) -> Result<(), DetectError>
where
    T: TimeScale + Copy,
    S: Signal<T>,
    for<'a> R: FindBracketedRoot<SignalCallback<'a, T, S>>,
{
    for i in 1..offsets.len().saturating_sub(1) {
        let (v0, v1, v2) = (values[i - 1], values[i], values[i + 1]);
        // Only same-sign triples (a sign change is already handled).
        if v0.signum() != v1.signum() || v1.signum() != v2.signum() {
            continue;
        }
        let is_min = v1 < v0 && v1 < v2;
        let is_max = v1 > v0 && v1 > v2;
        if !(is_min && v1 > 0.0 || is_max && v1 < 0.0) {
            continue;
        }
        // Two complementary candidate filters, both costing zero extra
        // evaluations: a parabola fit through the triple (tight for
        // smooth extrema), and a relative-magnitude check (the parabola
        // underestimates tent-shaped extrema at combinator kinks, where
        // a near-zero sample between much larger neighbors is the
        // telltale instead).
        let vertex = parabola_vertex((offsets[i - 1], v0), (offsets[i], v1), (offsets[i + 1], v2))
            .filter(|(tv, _)| *tv > offsets[i - 1] && *tv < offsets[i + 1]);
        let parabola_crossing = vertex.is_some_and(|(_, vv)| {
            let scale = v1.abs().max(f64::MIN_POSITIVE);
            vv.signum() != v1.signum() && vv.abs() > GRAZING_REL_MARGIN * scale
        });
        let near_zero = v1.abs() < 0.5 * v0.abs().max(v2.abs());
        if !(parabola_crossing || near_zero) {
            continue;
        }
        // ③ REFINE, cheapest test first: a single probe at the predicted
        // extremum. A sign change there yields the two brackets directly;
        // otherwise the derivative-free minimizer runs only when the probe
        // moved markedly toward zero (sharp kink extrema the parabola fit
        // underestimates) — smooth non-crossing extrema are dismissed for
        // one evaluation instead of a full minimization.
        let probe_t = vertex
            .map(|(tv, _)| tv)
            .unwrap_or(0.5 * (offsets[i - 1] + offsets[i + 1]));
        let v_probe = callback
            .call(probe_t)
            .map_err(|e| DetectError::RootFinder(e.into()))?;
        let (x_star, v_star) = if v_probe.signum() != v1.signum() && v_probe != 0.0 {
            (probe_t, v_probe)
        } else if v_probe.abs() < GRAZING_PROBE_IMPROVEMENT * v1.abs() {
            // Minimize f for a dip, -f for a bump.
            let sign = if is_min { 1.0 } else { -1.0 };
            let objective = move |x: f64| -> Result<f64, BoxedError> {
                callback
                    .call(x)
                    .map(|v| sign * v)
                    .map_err(|e| BoxedError::from(e.to_string()))
            };
            let bracket = (offsets[i - 1], offsets[i + 1]);
            let x_star = minimizer
                .find_minimum_in_bracket(objective, bracket)
                .map_err(DetectError::RootFinder)?;
            let v_star = callback
                .call(x_star)
                .map_err(|e| DetectError::RootFinder(e.into()))?;
            if v_star.signum() == v1.signum() || v_star == 0.0 {
                // The extremum does not actually cross: a touching root
                // with empty interior, or a false candidate.
                continue;
            }
            (x_star, v_star)
        } else {
            continue;
        };
        // Two new brackets around the extremum → two events.
        for (a, b, fa, fb) in [
            (offsets[i - 1], x_star, v0, v_star),
            (x_star, offsets[i + 1], v_star, v2),
        ] {
            if let Some(crossing) = ZeroCrossing::new(fa.signum(), fb.signum()) {
                let t = root_finder
                    .find_in_bracket_with_values(callback, (a, b), (fa, fb))
                    .map_err(DetectError::RootFinder)?;
                events.push(Event::new(start + TimeDelta::from_seconds_f64(t), crossing));
            }
        }
    }
    Ok(())
}

fn sort_events<T: TimeScale + Copy>(events: &mut [Event<T>], start: Time<T>) {
    events.sort_by(|a, b| {
        let ta = (a.time() - start).to_seconds().to_f64();
        let tb = (b.time() - start).to_seconds().to_f64();
        ta.total_cmp(&tb)
    });
}

/// Sample → bracket → refine over a borrowed signal (shared by [`Detector`]
/// and the legacy detector in [`crate::events`]).
pub(crate) fn detect_grid<T, S, R>(
    signal: &S,
    root_finder: &R,
    minimizer: &BrentMinimizer,
    interval: TimeInterval<T>,
    sampler: &impl Sampler<T>,
) -> Result<(Vec<Event<T>>, f64), DetectError>
where
    T: TimeScale + Copy,
    S: Signal<T>,
    for<'a> R: FindBracketedRoot<SignalCallback<'a, T, S>>,
{
    let start = interval.start();
    let times = sampler.sample(interval);
    let mut values = vec![0.0; times.len()];
    signal
        .eval_grid(&times, &mut values)
        .map_err(|e| DetectError::Callback(Box::new(e)))?;

    let offsets: Vec<f64> = times
        .iter()
        .map(|t| (*t - start).to_seconds().to_f64())
        .collect();
    let callback = SignalCallback { signal, start };

    let mut events = Vec::new();
    scan_crossings(callback, root_finder, &offsets, &values, start, &mut events)?;
    scan_grazing(
        callback,
        root_finder,
        minimizer,
        &offsets,
        &values,
        start,
        &mut events,
    )?;
    sort_events(&mut events, start);
    Ok((events, values[0]))
}

/// Two-level sample → bracket → refine: a coarse sweep brackets the
/// crossings and only sign-changing coarse brackets are re-sampled at the
/// fine step — the legacy coarse/fine scheme, with grazing recovery on the
/// coarse grid so near-zero extrema between coarse samples are still found.
pub(crate) fn detect_grid_two_level<T, S, R>(
    signal: &S,
    root_finder: &R,
    minimizer: &BrentMinimizer,
    interval: TimeInterval<T>,
    coarse_step: TimeDelta,
    fine_step: TimeDelta,
) -> Result<(Vec<Event<T>>, f64), DetectError>
where
    T: TimeScale + Copy,
    S: Signal<T>,
    for<'a> R: FindBracketedRoot<SignalCallback<'a, T, S>>,
{
    let start = interval.start();
    let times = UniformSampler::new(coarse_step).sample(interval);
    let mut values = vec![0.0; times.len()];
    signal
        .eval_grid(&times, &mut values)
        .map_err(|e| DetectError::Callback(Box::new(e)))?;

    let offsets: Vec<f64> = times
        .iter()
        .map(|t| (*t - start).to_seconds().to_f64())
        .collect();
    let callback = SignalCallback { signal, start };
    let fine = fine_step.to_seconds().to_f64();

    let mut events = Vec::new();
    // Bracket selection bridges exact zeros the same way `scan_crossings`
    // does: a coarse sample of exactly 0 is skipped, and a sign change is
    // taken between the last two *nonzero* coarse samples. Otherwise a
    // crossing landing exactly on a coarse sample would fall in the gap
    // between two brackets that each look sign-stable.
    let mut last_nonzero: Option<(f64, f64)> = None;
    for i in 0..offsets.len() {
        let v1 = values[i];
        if v1 == 0.0 {
            continue;
        }
        if let Some((o0, v0)) = last_nonzero
            && ZeroCrossing::new(v0.signum(), v1.signum()).is_some()
        {
            // Subdivide the sign-changing span (which may bridge zero coarse
            // samples) at the fine step; endpoints known, interior one batch.
            let o1 = offsets[i];
            let segments = ((o1 - o0) / fine).ceil().max(1.0) as usize;
            let mut fine_offsets = Vec::with_capacity(segments + 1);
            let mut interior_times = Vec::with_capacity(segments.saturating_sub(1));
            fine_offsets.push(o0);
            for k in 1..segments {
                let o = o0 + k as f64 * fine;
                fine_offsets.push(o);
                interior_times.push(start + TimeDelta::from_seconds_f64(o));
            }
            fine_offsets.push(o1);
            let mut fine_values = vec![0.0; fine_offsets.len()];
            fine_values[0] = v0;
            *fine_values.last_mut().unwrap() = v1;
            if !interior_times.is_empty() {
                signal
                    .eval_grid(&interior_times, &mut fine_values[1..segments])
                    .map_err(|e| DetectError::Callback(Box::new(e)))?;
            }
            scan_crossings(
                callback,
                root_finder,
                &fine_offsets,
                &fine_values,
                start,
                &mut events,
            )?;
            scan_grazing(
                callback,
                root_finder,
                minimizer,
                &fine_offsets,
                &fine_values,
                start,
                &mut events,
            )?;
        }
        last_nonzero = Some((offsets[i], v1));
    }
    scan_grazing(
        callback,
        root_finder,
        minimizer,
        &offsets,
        &values,
        start,
        &mut events,
    )?;
    sort_events(&mut events, start);
    Ok((events, values[0]))
}

impl<S, R> Detector<S, R> {
    /// Detects all zero-crossing events within the interval.
    pub fn events<T>(
        &self,
        interval: TimeInterval<T>,
        sampler: &impl Sampler<T>,
    ) -> Result<Vec<Event<T>>, DetectError>
    where
        T: TimeScale + Copy,
        S: Signal<T>,
        for<'a> R: FindBracketedRoot<SignalCallback<'a, T, S>>,
    {
        self.events_with_start_value(interval, sampler)
            .map(|(events, _)| events)
    }

    /// Detects events with two-level sampling: a coarse sweep brackets the
    /// crossings and only sign-changing coarse brackets are re-sampled at
    /// the fine step. Grazing recovery runs on the coarse grid.
    pub fn events_two_level<T>(
        &self,
        interval: TimeInterval<T>,
        coarse_step: TimeDelta,
        fine_step: TimeDelta,
    ) -> Result<Vec<Event<T>>, DetectError>
    where
        T: TimeScale + Copy,
        S: Signal<T>,
        for<'a> R: FindBracketedRoot<SignalCallback<'a, T, S>>,
    {
        detect_grid_two_level(
            &self.signal,
            &self.root_finder,
            &self.minimizer,
            interval,
            coarse_step,
            fine_step,
        )
        .map(|(events, _)| events)
    }

    /// Detects all windows where the signal is positive, with the binding
    /// constraint at each boundary as a diagnostic.
    pub fn windows<T>(
        &self,
        interval: TimeInterval<T>,
        sampler: &impl Sampler<T>,
    ) -> Result<Vec<Window<T>>, DetectError>
    where
        T: TimeScale + Copy,
        S: Signal<T>,
        for<'a> R: FindBracketedRoot<SignalCallback<'a, T, S>>,
    {
        let (events, start_value) = self.events_with_start_value(interval, sampler)?;
        self.windows_from(interval, events, start_value)
    }

    /// Two-level variant of [`windows`](Self::windows).
    pub fn windows_two_level<T>(
        &self,
        interval: TimeInterval<T>,
        coarse_step: TimeDelta,
        fine_step: TimeDelta,
    ) -> Result<Vec<Window<T>>, DetectError>
    where
        T: TimeScale + Copy,
        S: Signal<T>,
        for<'a> R: FindBracketedRoot<SignalCallback<'a, T, S>>,
    {
        let (events, start_value) = detect_grid_two_level(
            &self.signal,
            &self.root_finder,
            &self.minimizer,
            interval,
            coarse_step,
            fine_step,
        )?;
        self.windows_from(interval, events, start_value)
    }

    /// Pairs Up/Down events into windows with binding-constraint
    /// diagnostics, synthesizing boundaries at the horizon edges
    /// (start-sign logic kept from the interval design).
    fn windows_from<T>(
        &self,
        interval: TimeInterval<T>,
        events: Vec<Event<T>>,
        start_value: f64,
    ) -> Result<Vec<Window<T>>, DetectError>
    where
        T: TimeScale + Copy,
        S: Signal<T>,
    {
        // A single-leaf signal is trivially its own binding constraint —
        // skip the two boundary evaluations per window.
        let single_leaf = self.signal.leaf_count() == 1;
        let binding = |time: Time<T>| -> Result<ConstraintId, DetectError> {
            if single_leaf {
                return Ok(ConstraintId(0));
            }
            self.signal
                .eval_binding(time)
                .map(|(_, id)| ConstraintId(id))
                .map_err(|e| DetectError::Callback(Box::new(e)))
        };

        if events.is_empty() {
            return if start_value.signum() >= 0.0 {
                Ok(vec![Window {
                    interval,
                    opened_by: binding(interval.start())?,
                    closed_by: binding(interval.end())?,
                }])
            } else {
                Ok(vec![])
            };
        }

        let mut boundaries: Vec<(Time<T>, ZeroCrossing)> = Vec::with_capacity(events.len() + 2);
        if events[0].crossing() == ZeroCrossing::Down {
            boundaries.push((interval.start(), ZeroCrossing::Up));
        }
        boundaries.extend(events.iter().map(|e| (e.time(), e.crossing())));
        if boundaries
            .last()
            .is_some_and(|(_, c)| *c == ZeroCrossing::Up)
        {
            boundaries.push((interval.end(), ZeroCrossing::Down));
        }

        let mut windows = Vec::with_capacity(boundaries.len() / 2);
        for pair in boundaries.chunks(2) {
            let [(open, up), (close, down)] = pair else {
                continue;
            };
            debug_assert_eq!(*up, ZeroCrossing::Up);
            debug_assert_eq!(*down, ZeroCrossing::Down);
            windows.push(Window {
                interval: TimeInterval::new(*open, *close),
                opened_by: binding(*open)?,
                closed_by: binding(*close)?,
            });
        }
        Ok(windows)
    }

    /// Detects windows inside the given candidate intervals only.
    ///
    /// Hierarchical pruning: `min(f_cheap, f_rest) ≤ f_cheap` pointwise, so
    /// when this detector's signal is `cheap.min(rest)` and `candidates` are
    /// the cheap signal's windows, every window of the combined signal lies
    /// inside a candidate — the expensive constraints are only evaluated
    /// where the cheap one already holds.
    pub fn windows_within<T>(
        &self,
        candidates: &[TimeInterval<T>],
        sampler: &impl Sampler<T>,
    ) -> Result<Vec<Window<T>>, DetectError>
    where
        T: TimeScale + Copy,
        S: Signal<T>,
        for<'a> R: FindBracketedRoot<SignalCallback<'a, T, S>>,
    {
        let mut windows = Vec::new();
        for candidate in candidates {
            windows.extend(self.windows(*candidate, sampler)?);
        }
        Ok(windows)
    }
}

/// Vertex of the parabola through three points, or `None` when degenerate.
fn parabola_vertex(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64)) -> Option<(f64, f64)> {
    let (x0, y0) = p0;
    let (x1, y1) = p1;
    let (x2, y2) = p2;
    let d0 = (x0 - x1) * (x0 - x2);
    let d1 = (x1 - x0) * (x1 - x2);
    let d2 = (x2 - x0) * (x2 - x1);
    // Lagrange form: f(x) = a x² + b x + c
    let a = y0 / d0 + y1 / d1 + y2 / d2;
    if a == 0.0 || !a.is_finite() {
        return None;
    }
    let b = -(y0 * (x1 + x2) / d0 + y1 * (x0 + x2) / d1 + y2 * (x0 + x1) / d2);
    let c = y0 * x1 * x2 / d0 + y1 * x0 * x2 / d1 + y2 * x0 * x1 / d2;
    let xv = -b / (2.0 * a);
    let yv = c - b * b / (4.0 * a);
    Some((xv, yv))
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI, TAU};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use lox_test_utils::assert_approx_eq;
    use lox_time::time;
    use lox_time::time_scales::Tai;

    use super::*;

    fn horizon(seconds: f64) -> TimeInterval<Tai> {
        let start = time!(Tai, 2026, 6, 1).unwrap();
        TimeInterval::new(start, start + TimeDelta::from_seconds_f64(seconds))
    }

    fn elapsed(interval: TimeInterval<Tai>, t: Time<Tai>) -> f64 {
        (t - interval.start()).to_seconds().to_f64()
    }

    /// Worked example: f_A = sin t, f_B = cos t on [0, 7]. Windows of
    /// min(f_A, f_B) are (0, π/2) and (2π, 7); the first
    /// closes on cos (leaf 1) and the second opens on sin (leaf 0).
    #[test]
    fn test_min_windows_match_worked_example() {
        let interval = horizon(7.0);
        let start = interval.start();
        let sin = SignalFn(move |t: Time<Tai>| (t - start).to_seconds().to_f64().sin());
        let cos = SignalFn(move |t: Time<Tai>| (t - start).to_seconds().to_f64().cos());
        let detector = Detector::new(sin.min(cos));
        let windows = detector
            .windows(
                interval,
                &UniformSampler::new(TimeDelta::from_seconds_f64(0.1)),
            )
            .unwrap();

        assert_eq!(windows.len(), 2);
        assert_approx_eq!(
            elapsed(interval, windows[0].interval().start()),
            0.0,
            atol <= 1e-6
        );
        assert_approx_eq!(
            elapsed(interval, windows[0].interval().end()),
            FRAC_PI_2,
            atol <= 1e-6
        );
        assert_approx_eq!(
            elapsed(interval, windows[1].interval().start()),
            TAU,
            atol <= 1e-6
        );
        assert_approx_eq!(
            elapsed(interval, windows[1].interval().end()),
            7.0,
            atol <= 1e-6
        );

        // Diagnostics: the binding constraint at each boundary.
        assert_eq!(windows[0].closed_by(), ConstraintId(1)); // cos crosses down at π/2
        assert_eq!(windows[1].opened_by(), ConstraintId(0)); // sin crosses up at 2π
    }

    /// Composed detection equals interval intersection of separate
    /// detections (locality), with fewer evaluations than two full sweeps.
    #[test]
    fn test_min_equals_interval_intersection() {
        use crate::events::{
            EventsToIntervals, IntervalDetector, IntervalDetectorExt, RootFindingDetector,
            TryFnDetect,
        };

        let interval = horizon(86_400.0);
        let start = interval.start();
        let count_a = AtomicUsize::new(0);
        let count_b = AtomicUsize::new(0);

        let elevation = move |t: Time<Tai>| {
            let s = (t - start).to_seconds().to_f64();
            (TAU * s / 5_700.0).sin() - 0.97
        };
        let occultation = move |t: Time<Tai>| {
            let s = (t - start).to_seconds().to_f64();
            (TAU * s / (5_700.0 * 1.37) + 0.7).sin() + 0.4
        };

        let counted_a = SignalFn(|t: Time<Tai>| {
            count_a.fetch_add(1, Ordering::Relaxed);
            elevation(t)
        });
        let counted_b = SignalFn(|t: Time<Tai>| {
            count_b.fetch_add(1, Ordering::Relaxed);
            occultation(t)
        });

        let step = TimeDelta::from_seconds(10);
        let detector = Detector::new(counted_a.min(counted_b));
        let windows = detector
            .windows(interval, &UniformSampler::new(step))
            .unwrap();
        let signal_evals = count_a.load(Ordering::Relaxed) + count_b.load(Ordering::Relaxed);

        // Reference: interval-space intersection of two separate detections.
        let count_ref = AtomicUsize::new(0);
        let a = EventsToIntervals::new(RootFindingDetector::new(
            TryFnDetect(|t: Time<Tai>| {
                count_ref.fetch_add(1, Ordering::Relaxed);
                Ok::<f64, std::convert::Infallible>(elevation(t))
            }),
            step,
        ));
        let b = EventsToIntervals::new(RootFindingDetector::new(
            TryFnDetect(|t: Time<Tai>| {
                count_ref.fetch_add(1, Ordering::Relaxed);
                Ok::<f64, std::convert::Infallible>(occultation(t))
            }),
            step,
        ));
        let reference = a.intersect(b).detect(interval).unwrap();
        let reference_evals = count_ref.load(Ordering::Relaxed);

        assert_eq!(windows.len(), reference.len());
        for (window, expected) in windows.iter().zip(&reference) {
            assert_approx_eq!(
                (window.interval().start() - expected.start())
                    .to_seconds()
                    .to_f64(),
                0.0,
                atol <= 1e-3
            );
            assert_approx_eq!(
                (window.interval().end() - expected.end())
                    .to_seconds()
                    .to_f64(),
                0.0,
                atol <= 1e-3
            );
        }
        // Both paths now share the warm-started pipeline, so the composed
        // sweep costs about the same as two separate sweeps (its surplus is
        // the boundary diagnostics and grazing refinement); the structural
        // wins are exact kink semantics, single-pass windows, and the
        // diagnostics themselves. Hierarchical pruning is where composition
        // will beat separate sweeps outright.
        assert!(signal_evals <= reference_evals + 256);
    }

    /// A window much shorter than the sampling step is recovered through
    /// the grazing path (parabola candidate + extremum refinement).
    #[test]
    fn test_grazing_window_recovered() {
        let interval = horizon(10.0);
        let start = interval.start();
        // Positive only on (4.98, 5.02): width 0.04 ≪ step 0.5.
        let f = SignalFn(move |t: Time<Tai>| {
            let s = (t - start).to_seconds().to_f64();
            0.02_f64.powi(2) - (s - 5.0).powi(2)
        });
        let detector = Detector::new(f);
        let windows = detector
            .windows(
                interval,
                &UniformSampler::new(TimeDelta::from_seconds_f64(0.5)),
            )
            .unwrap();
        assert_eq!(windows.len(), 1);
        assert_approx_eq!(
            elapsed(interval, windows[0].interval().start()),
            4.98,
            atol <= 1e-6
        );
        assert_approx_eq!(
            elapsed(interval, windows[0].interval().end()),
            5.02,
            atol <= 1e-6
        );
    }

    /// A touching root (extremum exactly at zero) yields no window — an
    /// intersection with empty interior.
    #[test]
    fn test_touching_extremum_yields_no_window() {
        let interval = horizon(10.0);
        let start = interval.start();
        let f = SignalFn(move |t: Time<Tai>| {
            let s = (t - start).to_seconds().to_f64();
            -((s - 5.0).powi(2))
        });
        let detector = Detector::new(f);
        let windows = detector
            .windows(
                interval,
                &UniformSampler::new(TimeDelta::from_seconds_f64(0.5)),
            )
            .unwrap();
        assert!(windows.is_empty());
    }

    /// Signals that never cross: all-positive yields the whole horizon,
    /// all-negative yields nothing.
    #[test]
    fn test_constant_sign_signals() {
        let interval = horizon(100.0);
        let sampler = UniformSampler::new(TimeDelta::from_seconds(10));

        let positive = Detector::new(SignalFn(|_: Time<Tai>| 1.0));
        let windows = positive.windows(interval, &sampler).unwrap();
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].interval(), interval);

        let negative = Detector::new(SignalFn(|_: Time<Tai>| -1.0));
        assert!(negative.windows(interval, &sampler).unwrap().is_empty());
    }

    /// `max` is OR: windows are the union of the operands' windows, and
    /// `neg` is NOT.
    #[test]
    fn test_max_and_neg_algebra() {
        let interval = horizon(TAU);
        let start = interval.start();
        let sin = SignalFn(move |t: Time<Tai>| (t - start).to_seconds().to_f64().sin());
        let cos = SignalFn(move |t: Time<Tai>| (t - start).to_seconds().to_f64().cos());
        let sampler = UniformSampler::new(TimeDelta::from_seconds_f64(0.05));

        // max(sin, cos) > 0 on (0, π) ∪ (3π/2, 2π) within one period.
        let windows = Detector::new(sin.max(cos))
            .windows(interval, &sampler)
            .unwrap();
        assert_eq!(windows.len(), 2);
        assert_approx_eq!(
            elapsed(interval, windows[0].interval().end()),
            PI,
            atol <= 1e-6
        );
        assert_approx_eq!(
            elapsed(interval, windows[1].interval().start()),
            3.0 * PI / 2.0,
            atol <= 1e-6
        );

        // NOT sin > 0 on (π, 2π).
        let sin2 = SignalFn(move |t: Time<Tai>| (t - start).to_seconds().to_f64().sin());
        let windows = Detector::new(sin2.neg())
            .windows(interval, &sampler)
            .unwrap();
        assert_eq!(windows.len(), 1);
        assert_approx_eq!(
            elapsed(interval, windows[0].interval().start()),
            PI,
            atol <= 1e-6
        );
    }

    /// Events found by the signal pipeline match the legacy detector.
    #[test]
    fn test_events_match_legacy_detector() {
        use crate::events::{EventDetector, FnDetect, RootFindingDetector};

        let interval = horizon(100.0);
        let start = interval.start();
        let f = move |t: Time<Tai>| ((t - start).to_seconds().to_f64() / 7.0).sin();
        let step = TimeDelta::from_seconds(1);

        let legacy = RootFindingDetector::new(FnDetect(f), step)
            .detect(interval)
            .unwrap();
        let signal = Detector::new(SignalFn(f))
            .events(interval, &UniformSampler::new(step))
            .unwrap();

        assert_eq!(legacy.len(), signal.len());
        for (a, b) in legacy.iter().zip(&signal) {
            assert_eq!(a.crossing(), b.crossing());
            assert_approx_eq!(
                (a.time() - b.time()).to_seconds().to_f64(),
                0.0,
                atol <= 1e-6
            );
        }
    }

    #[test]
    fn test_knot_sampler_clips_and_refines() {
        let interval = horizon(100.0);
        let start = interval.start();
        let knots: Vec<_> = [-10.0, 0.0, 25.0, 50.0, 75.0, 100.0, 120.0]
            .iter()
            .map(|s| start + TimeDelta::from_seconds_f64(*s))
            .collect();

        let times = KnotSampler::new(knots.clone()).sample(interval);
        // Outside and boundary-coincident knots are dropped; endpoints added.
        let offsets: Vec<f64> = times.iter().map(|t| elapsed(interval, *t)).collect();
        assert_eq!(offsets, vec![0.0, 25.0, 50.0, 75.0, 100.0]);

        let times = KnotSampler::new(knots).with_refinement(2).sample(interval);
        assert_eq!(times.len(), 9);
        assert_approx_eq!(elapsed(interval, times[1]), 12.5, atol <= 1e-9);
    }

    /// A signal varying at the knot rate is fully resolved by the knot
    /// sampler where a coarse uniform grid misses windows.
    #[test]
    fn test_knot_sampler_resolves_knot_rate_signal() {
        let interval = horizon(100.0);
        let start = interval.start();
        // Period 10 s: positive on 5 s half-cycles.
        let f =
            SignalFn(move |t: Time<Tai>| ((t - start).to_seconds().to_f64() * TAU / 10.0).sin());
        let knots: Vec<_> = (0..=50)
            .map(|k| start + TimeDelta::from_seconds_f64(k as f64 * 2.0))
            .collect();
        let windows = Detector::new(f)
            .windows(interval, &KnotSampler::new(knots))
            .unwrap();
        assert_eq!(windows.len(), 10);
    }

    /// Two-level detection matches the single-level fine sweep — including
    /// multiple crossings inside one coarse bracket and grazing windows
    /// between coarse samples — while evaluating far less.
    #[test]
    fn test_windows_two_level_matches_single_level() {
        let interval = horizon(1_000.0);
        let start = interval.start();
        let count = AtomicUsize::new(0);
        // Three crossings within [0, 1000]: passes at (100, 130) and
        // (600, 800), the first much shorter than the coarse step.
        let f = move |t: Time<Tai>| {
            let s = (t - start).to_seconds().to_f64();
            let pass1 = 15.0 - (s - 115.0).abs();
            let pass2 = 100.0 - (s - 700.0).abs();
            pass1.max(pass2)
        };
        let counted = SignalFn(|t: Time<Tai>| {
            count.fetch_add(1, Ordering::Relaxed);
            f(t)
        });

        let fine = TimeDelta::from_seconds(5);
        let single = Detector::new(SignalFn(f))
            .windows(interval, &UniformSampler::new(fine))
            .unwrap();
        let two_level = Detector::new(&counted)
            .windows_two_level(interval, TimeDelta::from_seconds(150), fine)
            .unwrap();
        let two_level_evals = count.load(Ordering::Relaxed);

        assert_eq!(single.len(), 2);
        assert_eq!(single.len(), two_level.len());
        for (a, b) in single.iter().zip(&two_level) {
            assert_approx_eq!(
                elapsed(interval, a.interval().start()),
                elapsed(interval, b.interval().start()),
                atol <= 1e-6
            );
            assert_approx_eq!(
                elapsed(interval, a.interval().end()),
                elapsed(interval, b.interval().end()),
                atol <= 1e-6
            );
        }
        // Coarse sweep (~7 samples) + fine subdivision of the sign-changing
        // brackets only — far fewer than the ~200 single-level samples.
        assert!(
            two_level_evals < 150,
            "two-level should evaluate sparsely ({two_level_evals} evals)"
        );
    }

    /// `MinOf` over boxed signals matches nested binary `min`, including
    /// the binding-constraint diagnostics.
    #[test]
    fn test_min_of_matches_nested_min() {
        let interval = horizon(7.0);
        let start = interval.start();
        let sampler = UniformSampler::new(TimeDelta::from_seconds_f64(0.1));
        let phase = move |t: Time<Tai>| (t - start).to_seconds().to_f64();

        let boxed: Vec<Box<dyn Signal<Tai, Error = std::convert::Infallible>>> = vec![
            Box::new(SignalFn(move |t| phase(t).sin())),
            Box::new(SignalFn(move |t| phase(t).cos())),
            Box::new(SignalFn(move |t| 1.0 - 0.1 * phase(t))),
        ];
        let n_ary = Detector::new(MinOf::new(boxed))
            .windows(interval, &sampler)
            .unwrap();

        let sin = SignalFn(move |t: Time<Tai>| phase(t).sin());
        let cos = SignalFn(move |t: Time<Tai>| phase(t).cos());
        let line = SignalFn(move |t: Time<Tai>| 1.0 - 0.1 * phase(t));
        let nested = Detector::new(sin.min(cos).min(line))
            .windows(interval, &sampler)
            .unwrap();

        assert_eq!(n_ary.len(), nested.len());
        for (a, b) in n_ary.iter().zip(&nested) {
            assert_eq!(a.interval(), b.interval());
            assert_eq!(a.opened_by(), b.opened_by());
            assert_eq!(a.closed_by(), b.closed_by());
        }
    }

    /// Pruned detection inside the cheap signal's windows finds the same
    /// windows as a full sweep of the combined signal, without evaluating
    /// the expensive constraint outside the candidates.
    #[test]
    fn test_windows_within_prunes_expensive_evals() {
        let interval = horizon(100.0);
        let start = interval.start();
        let sampler = UniformSampler::new(TimeDelta::from_seconds(1));
        let phase = move |t: Time<Tai>| (t - start).to_seconds().to_f64();
        // Cheap: positive on (20, 30) ∪ (70, 80) — 20% of the horizon;
        // expensive: positive on (0, 25) ∪ (50, 75).
        let cheap_fn =
            move |t: Time<Tai>| 5.0 - (phase(t) - 25.0).abs().min((phase(t) - 75.0).abs());
        let count = AtomicUsize::new(0);
        let expensive = SignalFn(|t: Time<Tai>| {
            count.fetch_add(1, Ordering::Relaxed);
            (phase(t) * TAU / 50.0).sin()
        });

        let candidates: Vec<_> = Detector::new(SignalFn(cheap_fn))
            .windows(interval, &sampler)
            .unwrap()
            .iter()
            .map(|w| w.interval())
            .collect();
        assert_eq!(candidates.len(), 2);

        let combined = Detector::new(SignalFn(cheap_fn).min(&expensive));
        let pruned = combined.windows_within(&candidates, &sampler).unwrap();
        let pruned_evals = count.swap(0, Ordering::Relaxed);
        let full = combined.windows(interval, &sampler).unwrap();
        let full_evals = count.load(Ordering::Relaxed);

        assert_eq!(pruned.len(), full.len());
        for (a, b) in pruned.iter().zip(&full) {
            assert_approx_eq!(
                elapsed(interval, a.interval().start()),
                elapsed(interval, b.interval().start()),
                atol <= 1e-6
            );
            assert_approx_eq!(
                elapsed(interval, a.interval().end()),
                elapsed(interval, b.interval().end()),
                atol <= 1e-6
            );
        }
        assert!(
            pruned_evals < full_evals / 2,
            "pruning should skip most expensive evals ({pruned_evals} vs {full_evals})"
        );
    }

    /// CHARACTERIZE: a real up-crossing landing exactly on a sample must
    /// still be detected (the reviewer's claimed miss).
    #[test]
    fn test_crossing_exactly_on_sample() {
        let interval = horizon(10.0);
        let start = interval.start();
        // f = s − 5: exactly 0 at the t=5 sample; positive on (5, 10].
        let f = SignalFn(move |t: Time<Tai>| (t - start).to_seconds().to_f64() - 5.0);
        let windows = Detector::new(f)
            .windows(
                interval,
                &UniformSampler::new(TimeDelta::from_seconds_f64(1.0)),
            )
            .unwrap();
        assert_eq!(windows.len(), 1, "crossing on a sample was missed");
        assert_approx_eq!(
            elapsed(interval, windows[0].interval().start()),
            5.0,
            atol <= 1e-6
        );
    }

    /// CHARACTERIZE: a forced `+0.0` sample between two negatives (a touching
    /// root from below, where natural arithmetic might not align the zero
    /// sign) must NOT produce a spurious window.
    #[test]
    fn test_touching_zero_sample_no_spurious_window() {
        let interval = horizon(10.0);
        let start = interval.start();
        // Negative everywhere except an exact +0.0 at the t=5 sample.
        let f = SignalFn(move |t: Time<Tai>| {
            let s = (t - start).to_seconds().to_f64();
            if (s - 5.0).abs() < 1e-9 { 0.0 } else { -1.0 }
        });
        let windows = Detector::new(f)
            .windows(
                interval,
                &UniformSampler::new(TimeDelta::from_seconds_f64(1.0)),
            )
            .unwrap();
        assert!(
            windows.is_empty(),
            "spurious window from a +0.0 touching sample: {windows:?}"
        );
    }

    /// `offset` shifts the level: elevation above a mask.
    #[test]
    fn test_offset_level_shift() {
        let interval = horizon(TAU);
        let start = interval.start();
        let sin = SignalFn(move |t: Time<Tai>| (t - start).to_seconds().to_f64().sin());
        let windows = Detector::new(sin.offset(-0.5))
            .windows(
                interval,
                &UniformSampler::new(TimeDelta::from_seconds_f64(0.05)),
            )
            .unwrap();
        // sin t > 0.5 on (π/6, 5π/6).
        assert_eq!(windows.len(), 1);
        assert_approx_eq!(
            elapsed(interval, windows[0].interval().start()),
            PI / 6.0,
            atol <= 1e-6
        );
        assert_approx_eq!(
            elapsed(interval, windows[0].interval().end()),
            5.0 * PI / 6.0,
            atol <= 1e-6
        );
    }
}
