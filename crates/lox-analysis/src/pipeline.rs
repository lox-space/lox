// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Lazy analysis pipelines and the generic ensemble runner.
//!
//! **Design prototype** validating the pipeline architecture:
//!
//! ```ignore
//! let passes = PassDetector::new(..).detect(interval).then(LinkDetector).try_collect()?;
//! ```
//!
//! The pieces, and the decisions behind them:
//!
//! - [`Stage`] — the one trait a *source* and every *transform* implement:
//!   `apply(input) -> Result<Vec<Out>, Error>`. Item types evolve stage to
//!   stage (`Pass -> Link -> ...`), and each item flat-maps to `0..n` outputs so
//!   filtering falls out for free.
//! - [`Source`] is the lazy scan that seeds a pipeline from a [`TimeInterval`],
//!   yielding items as the walk advances; [`PipelineExt::then`] chains the next
//!   stage. Both build on `std::iter::Iterator`, so the whole std/rayon ecosystem
//!   (`map`, `filter`, `take`, `try_collect`, `par_bridge`) is available for free.
//!   A scan cheap enough to compute up front can reuse an eager [`Stage`] via
//!   [`FromStage`].
//! - Every item is `Result<T, `[`AnalysisError`]`>` — one unified error flows the
//!   whole chain (each stage's error is `Into<AnalysisError>`), and `try_collect`
//!   short-circuits on the first failure.
//! - **Fan-out is the caller's**, with ordinary iterators — there is no runner
//!   type:
//!
//!   ```ignore
//!   let by_pair: Vec<_> = pairs
//!       .par_iter()                      // or .iter() — caller's choice, caller's pool
//!       .map(|p| (p.id(), make(p).try_collect::<Vec<_>>()))
//!       .collect();
//!   ```
//!
//!   Identity lives in the fan-out, not in the items: parallel and streaming
//!   results arrive in *completion* order, so keying by position would silently
//!   mis-associate them. `make: Fn(target) -> pipeline` is the one thing a user
//!   writes. Only streaming is non-trivial enough to warrant a component.
//!
//! Laziness here is Level 1 (composition-lazy): a stage's `apply` may compute its
//! whole output eagerly and hand back `vec.into_iter()`; the iterator is the
//! composition vehicle, not a promise of an incremental time-walk.
//!
//! Implementation note: the pipeline leans on `itertools` rather than bespoke
//! adapters — `detect` and `then` are `Either` + `flat_map` combinations, and the
//! constraint filter (`min_pass_duration` etc.) is just
//! `itertools::Itertools::filter_ok`. This deletes the hand-rolled `Detected`,
//! `Then`, and `KeepOk` adapters. A Level-1 [`Stage`] returns an owned
//! `Vec<Out>` from [`apply`](Stage::apply): it is eager internally anyway, and an
//! owned collection avoids the `&self` capture a return-position `impl Iterator`
//! would impose (which would make the stream borrow the stage and break both
//! building a pipeline from a temporary detector and `then`'s `flat_map`).
//! [`Source`] avoids it differently: its `type Stream` has no lifetime
//! parameter, so a stream structurally cannot borrow the `&self` handed to
//! `detect`. Level-2 within-stage laziness would trade a [`Stage`]'s `Vec` for a
//! named owning iterator type too.

use std::convert::Infallible;

use itertools::Either;
use lox_time::intervals::TimeInterval;
use thiserror::Error;

use lox_core::error::LoxError;

use crate::events::DetectError;
use crate::visibility::EvalError;

/// Pipeline-backed `*Analysis` types, coexisting with the eager ones until the
/// hard cut.
pub mod analyses;
mod sources;

pub use sources::{Eclipse, Window};

// ---------------------------------------------------------------------------
// Parallelism
// ---------------------------------------------------------------------------

/// How an `*Analysis::run` should fan out over its targets.
///
/// A **runtime** choice, not the rayon-free path: `Sequential` exists so a run
/// can be made deterministic for debugging while still linking rayon. Building
/// without the `parallel` feature is what removes rayon, and there `run` does
/// not exist at all — callers iterate themselves (design §9).
#[cfg(feature = "parallel")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parallelism {
    /// One target at a time, in input order.
    Sequential,
    /// Rayon: the global pool with `None`, or a local pool of `n` workers.
    ///
    /// `Some(n)` builds a **local** pool because the global one cannot be
    /// resized per call — a server sizing one request must not change the width
    /// of every other request in the process.
    Rayon(Option<usize>),
}

// ---------------------------------------------------------------------------
// Unified error
// ---------------------------------------------------------------------------

/// The single error type that flows through every analysis pipeline.
///
/// Each [`Stage`]'s own error is required to be `Into<AnalysisError>`, so a
/// pipeline is always `Iterator<Item = Result<T, AnalysisError>>` regardless of
/// how many differently-typed stages it chains.
#[derive(Debug, Error)]
pub enum AnalysisError {
    /// A root-finding / event-detection failure.
    #[error(transparent)]
    Detect(#[from] DetectError),
    /// A frame rotation failed.
    ///
    /// Constructed explicitly rather than via `#[from]`: the provider-specific
    /// error type varies, so the domain is known at the call site, not by type.
    #[error("rotation failed")]
    Rotation(#[source] LoxError),
    /// An ephemeris lookup failed.
    #[error("ephemeris lookup failed")]
    Ephemeris(#[source] LoxError),
    /// A stage failure that does not match a known domain variant.
    ///
    /// Wrapped in [`LoxError`] (the project's type-erased error) rather than
    /// stringified, so `Error::source()` and downcasting (`LoxError::downcast_ref`
    /// / `find_source`) survive to the wire — streaming clients need to
    /// distinguish e.g. an ephemeris gap from a rotation failure. Recurring domain
    /// errors get their own `#[from]` variants (rotation, ephemeris, …) as real
    /// stages are ported.
    #[error(transparent)]
    Stage(#[from] LoxError),
}

impl AnalysisError {
    /// Wraps a frame-rotation failure.
    pub fn rotation(e: impl Into<LoxError>) -> Self {
        Self::Rotation(e.into())
    }

    /// Wraps an ephemeris-lookup failure.
    pub fn ephemeris(e: impl Into<LoxError>) -> Self {
        Self::Ephemeris(e.into())
    }
}

impl From<Infallible> for AnalysisError {
    fn from(never: Infallible) -> Self {
        match never {}
    }
}

/// Routes a detector-evaluation failure to its domain variant.
///
/// This is what the named variants buy: a streaming client can tell an
/// ephemeris gap from a rotation failure without string-matching, and the
/// classification happens once here rather than at every call site.
impl From<EvalError> for AnalysisError {
    fn from(e: EvalError) -> Self {
        match e {
            EvalError::Rotation(_) => Self::Rotation(LoxError::new(e)),
            EvalError::Ephemeris(_) => Self::Ephemeris(LoxError::new(e)),
            EvalError::UndefinedProperty(_) => Self::Stage(LoxError::new(e)),
        }
    }
}

// ---------------------------------------------------------------------------
// Stage — the unified source/transform trait
// ---------------------------------------------------------------------------

/// A pipeline stage: maps one input into a stream of outputs.
///
/// The *source* of a pipeline is a `Stage<TimeInterval>` (see
/// [`Source::detect`]); every downstream *transform* is a `Stage<PrevOut>`. Each
/// input flat-maps to `0..n` outputs, so a stage that drops an item simply yields
/// an empty iterator.
pub trait Stage<In> {
    /// The item type this stage emits.
    type Out;
    /// This stage's own error type; unified into [`AnalysisError`] by the pipeline.
    type Error: Into<AnalysisError>;

    /// Applies the stage to a single input, returning all of its outputs.
    ///
    /// A Level-1 stage is eager internally — it computes every output for this
    /// input — so it returns an owned `Vec`, and the pipeline turns that into the
    /// lazy item stream. Returning an owned collection (rather than a
    /// return-position `impl Iterator`) is deliberate: an `impl Iterator` return
    /// would implicitly borrow `&self`, so the produced stream would borrow the
    /// stage — which breaks both building a pipeline from a temporary detector
    /// and `then`'s `flat_map`. (When Level-2 within-stage laziness is wanted,
    /// this becomes a named owning iterator type instead.)
    fn apply(&self, input: In) -> Result<Vec<Self::Out>, Self::Error>;
}

// ---------------------------------------------------------------------------
// Source::detect — seed a pipeline from an interval
// ---------------------------------------------------------------------------

/// A lazy scan over a [`TimeInterval`], yielding items as the scan advances.
///
/// `type Stream` deliberately carries **no lifetime parameter**, so a stream
/// cannot borrow the `&self` passed to [`detect`](Self::detect). That is what
/// lets a pipeline be built from a temporary detector and lets [`PipelineExt::then`]
/// flat-map over it. It does *not* force full ownership: a borrowing source may
/// set `Stream = MyStream<'a>` and carry scenario borrows, which is fine on sync
/// paths. Only the streaming engine additionally requires `Send + 'static`, and
/// that bound — not this one — is what forces owned or `Arc`-shared inputs there.
///
/// [`detect`](Self::detect) is **infallible in its signature**. A failure before
/// any item exists (an unresolvable ephemeris, say) is deferred into the stream:
/// the first `next()` yields `Err` and the stream then ends. "Failed to start"
/// and "failed mid-scan" therefore look identical to every consumer, so no
/// caller needs a second error path.
pub trait Source {
    /// The item type this source emits.
    type Out;
    /// The lazy stream of items produced by a scan.
    type Stream: Iterator<Item = Result<Self::Out, AnalysisError>>;

    /// Scans `interval`, returning the stream of items.
    fn detect(&self, interval: TimeInterval) -> Self::Stream;
}

/// Adapts an eager [`Stage`] into a [`Source`], for a scan cheap enough to
/// compute up front. The `Vec` is produced once and drained lazily; a source
/// failure becomes the stream's single `Err`.
///
/// The example is also the caller-owned fan-out pattern: identity is attached in
/// the `map` closure, never inferred from position, because parallel and
/// streaming results arrive in completion order.
///
/// ```
/// use lox_analysis::pipeline::{AnalysisError, FromStage, PipelineExt, Source, Stage};
/// use lox_time::deltas::TimeDelta;
/// use lox_time::intervals::TimeInterval;
/// use lox_time::time_scales::TimeScale;
/// use lox_time::Time;
/// use rayon::prelude::*;
/// use std::convert::Infallible;
///
/// // A toy source: chop the scan interval into fixed-length windows.
/// struct Windows {
///     step_secs: i64,
/// }
///
/// impl Stage<TimeInterval> for Windows {
///     type Out = TimeInterval;
///     type Error = Infallible;
///
///     fn apply(&self, interval: TimeInterval) -> Result<Vec<Self::Out>, Self::Error> {
///         let step = TimeDelta::from_seconds(self.step_secs);
///         let mut out = Vec::new();
///         let mut t = interval.start();
///         while t + step <= interval.end() {
///             out.push(TimeInterval::new(t, t + step));
///             t = t + step;
///         }
///         Ok(out)
///     }
/// }
///
/// let epoch = Time::j2000(TimeScale::Tai);
/// let interval = TimeInterval::new(epoch, epoch + TimeDelta::from_seconds(6));
/// let make = move |step| FromStage(Windows { step_secs: step }).detect(interval);
///
/// // Sequential: the caller's own loop.
/// let seq: Vec<(&str, usize)> = [("a", 2i64), ("b", 3), ("c", 6)]
///     .iter()
///     .map(|&(id, step)| {
///         let windows: Vec<_> = make(step).collect::<Result<_, AnalysisError>>().unwrap();
///         (id, windows.len())
///     })
///     .collect();
/// assert_eq!(seq, vec![("a", 3), ("b", 2), ("c", 1)]);
///
/// // Parallel: the caller's own pool. Same results, completion order.
/// let mut par: Vec<(&str, usize)> = [("a", 2i64), ("b", 3), ("c", 6)]
///     .par_iter()
///     .map(|&(id, step)| {
///         let windows: Vec<_> = make(step).collect::<Result<_, AnalysisError>>().unwrap();
///         (id, windows.len())
///     })
///     .collect();
/// par.sort();
/// let mut expected = seq.clone();
/// expected.sort();
/// assert_eq!(par, expected);
/// ```
pub struct FromStage<S>(pub S);

impl<S> Source for FromStage<S>
where
    S: Stage<TimeInterval>,
{
    type Out = S::Out;
    type Stream = StageStream<S::Out>;

    fn detect(&self, interval: TimeInterval) -> Self::Stream {
        match self.0.apply(interval) {
            Ok(items) => StageStream(Either::Left(items.into_iter().map(Ok))),
            Err(e) => StageStream(Either::Right(std::iter::once(Err(e.into())))),
        }
    }
}

/// The [`Source::Stream`] of a [`FromStage`] adapter.
pub struct StageStream<T>(
    #[allow(clippy::type_complexity)]
    Either<
        std::iter::Map<std::vec::IntoIter<T>, fn(T) -> Result<T, AnalysisError>>,
        std::iter::Once<Result<T, AnalysisError>>,
    >,
);

impl<T> Iterator for StageStream<T> {
    type Item = Result<T, AnalysisError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

// ---------------------------------------------------------------------------
// HasInterval — the one shipped capability
// ---------------------------------------------------------------------------

/// An analysis item that occupies a span of time.
///
/// This is deliberately the *only* capability trait: `Pass`, `Link` and friends
/// stay concrete structs with inherent accessors. A richer hierarchy
/// (`HasObservables` / `HasLinkBudget` / …) waits until some capability has a
/// second implementor — today there is one `Pass` and no `Link`, and the
/// streaming path erases capabilities to duck typing at the Python boundary
/// regardless.
pub trait HasInterval {
    /// The interval this item spans.
    fn interval(&self) -> TimeInterval;
}

// ---------------------------------------------------------------------------
// PipelineExt — the fluent `then` adapter
// ---------------------------------------------------------------------------

/// The fluent `then` adapter, blanket-implemented for any
/// `Iterator<Item = Result<T, AnalysisError>>`.
///
/// Item-preserving filters/maps are covered directly by `itertools` —
/// `filter_ok` (drop `Ok` items failing a predicate, e.g. a minimum pass
/// duration), `map_ok`, `filter_map_ok` — so this trait only adds the one thing
/// `itertools` cannot: chaining a type-changing [`Stage`].
pub trait PipelineExt<T>: Iterator<Item = Result<T, AnalysisError>> + Sized {
    /// Chains `stage`, feeding each upstream `Ok` item through it and flat-mapping
    /// the results. Upstream errors pass through untouched; a stage failure on one
    /// item becomes a single `Err` in the stream.
    fn then<S: Stage<T>>(self, stage: S) -> impl Iterator<Item = Result<S::Out, AnalysisError>> {
        self.flat_map(move |item| match item {
            Ok(input) => match stage.apply(input) {
                Ok(items) => Either::Left(items.into_iter().map(Ok)),
                Err(e) => Either::Right(std::iter::once(Err(e.into()))),
            },
            Err(e) => Either::Right(std::iter::once(Err(e))),
        })
    }
}

impl<T, I: Iterator<Item = Result<T, AnalysisError>>> PipelineExt<T> for I {}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_time::deltas::TimeDelta;
    use lox_time::time;
    use lox_time::time_scales::Tai;

    // -- Toy stages standing in for real detectors ------------------------------

    /// Source: splits an interval into fixed-length windows (toy "passes").
    struct Windows {
        step_secs: i64,
    }

    impl Stage<TimeInterval> for Windows {
        type Out = TimeInterval;
        type Error = Infallible;

        fn apply(&self, interval: TimeInterval) -> Result<Vec<Self::Out>, Self::Error> {
            let start = interval.start();
            let total = (interval.end() - start).to_seconds().to_f64() as i64;
            let mut windows = Vec::new();
            let mut t = 0;
            while t < total {
                let a = start + TimeDelta::from_seconds(t);
                let b = start + TimeDelta::from_seconds((t + self.step_secs).min(total));
                windows.push(TimeInterval::new(a, b));
                t += self.step_secs;
            }
            Ok(windows)
        }
    }

    /// Transform: scores each window by its duration — item type *evolves* from
    /// `TimeInterval` to `(TimeInterval, f64)`.
    struct Scored;

    impl Stage<TimeInterval> for Scored {
        type Out = (TimeInterval, f64);
        type Error = Infallible;

        fn apply(&self, window: TimeInterval) -> Result<Vec<Self::Out>, Self::Error> {
            let secs = (window.end() - window.start()).to_seconds().to_f64();
            Ok(vec![(window, secs)])
        }
    }

    /// Transform that always fails — exercises error unification + short-circuit.
    struct Boom;

    #[derive(Debug, Error)]
    #[error("boom")]
    struct BoomError;

    impl From<BoomError> for AnalysisError {
        fn from(e: BoomError) -> Self {
            AnalysisError::Stage(LoxError::new(e))
        }
    }

    impl Stage<TimeInterval> for Boom {
        type Out = TimeInterval;
        type Error = BoomError;

        fn apply(&self, _window: TimeInterval) -> Result<Vec<Self::Out>, Self::Error> {
            Err(BoomError)
        }
    }

    fn interval(secs: i64) -> TimeInterval {
        let start = time!(Tai, 2000, 1, 1, 12).unwrap().into_dynamic();
        TimeInterval::new(start, start + TimeDelta::from_seconds(secs))
    }

    #[test]
    fn test_detect_then_try_collect() {
        use itertools::Itertools;
        // 6 s interval, 2 s windows -> 3 windows, each scored with its duration.
        let scored: Vec<(TimeInterval, f64)> = FromStage(Windows { step_secs: 2 })
            .detect(interval(6))
            .then(Scored)
            .try_collect()
            .unwrap();

        assert_eq!(scored.len(), 3);
        for (_, secs) in &scored {
            assert_eq!(*secs, 2.0);
        }
    }

    #[test]
    fn test_error_short_circuits() {
        use itertools::Itertools;
        let result: Result<Vec<_>, _> = FromStage(Windows { step_secs: 2 })
            .detect(interval(6))
            .then(Boom)
            .try_collect();
        assert!(matches!(result, Err(AnalysisError::Stage(_))));
    }

    #[test]
    fn test_filter_ok_as_constraint_stage() {
        use itertools::Itertools;
        // 5 s interval, 2 s windows -> [2 s, 2 s, 1 s]; drop the short tail window,
        // exactly how a `min_pass_duration` constraint becomes a pipeline stage —
        // and it's just itertools' `filter_ok`, no bespoke adapter.
        let kept: Vec<(TimeInterval, f64)> = FromStage(Windows { step_secs: 2 })
            .detect(interval(5))
            .then(Scored)
            .filter_ok(|(_, secs)| *secs >= 2.0)
            .try_collect()
            .unwrap();
        assert_eq!(kept.len(), 2);
    }
}
