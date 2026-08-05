// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Streams analysis pipelines from a rayon pool to an async consumer.
//!
//! This is the one reusable execution component (design §7.1). Everything else
//! about fan-out is the caller's business with plain iterators; only streaming
//! is non-trivial enough to warrant machinery, because it has to bridge
//! blocking CPU work to an async reader without stalling either.
//!
//! The shape is: rayon workers drive each target's [`Iterator`] → a **bounded**
//! [`tokio::sync::mpsc`] channel → an async [`Stream`] on the read side.
//!
//! ```text
//!   target A ─┐
//!   target B ─┼─ rayon ─ blocking_send ─▶ [bounded channel] ─▶ poll_recv ─ Stream
//!   target C ─┘                                  │
//!                     cancellation ◀─────────────┘ (channel closed, or token)
//! ```
//!
//! **Why bounded.** An unbounded channel makes a fast producer and a slow
//! consumer into a memory leak: a multi-week scan over hundreds of targets
//! would materialise every item before the client read the first one, which is
//! precisely what streaming exists to avoid. Bounding it converts that into
//! back-pressure — workers block in `blocking_send` until the consumer catches
//! up.
//!
//! **Why `blocking_send` and not `send().await`.** These are rayon worker
//! threads, not async tasks; there is no executor to yield to and nothing to
//! `.await` from. `blocking_send` parks the OS thread, which is the correct
//! thing to do when the work either side of it is synchronous and CPU-bound.
//! Calling the async `send` would require a runtime handle on every worker and
//! would block a runtime thread inside a future, starving the executor that is
//! supposed to be draining the channel — deadlock, not slowdown.
//!
//! **Two independent cancellation paths, and why one is not enough.** Dropping
//! the returned stream closes the channel, so the next `blocking_send` fails
//! and the worker returns; that alone bounds shutdown by *one item*. But a lazy
//! scan can run arbitrarily long before it produces its next item — a source
//! sweeping a quiet month emits nothing at all — so between-item checks are too
//! weak on their own. Hence the token: it is handed to `make`, so a source can
//! check it at its own sampling and root-refinement boundaries and bound
//! shutdown by one detector evaluation instead. A source that ignores the token
//! is still correct, just slower to stop.

use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;
use rayon::prelude::*;
use tokio::sync::mpsc;

use lox_core::sync::CancellationToken;

use crate::pipeline::AnalysisError;

/// The number of items that may sit in the channel before workers block.
///
/// Sized to amortise the per-item handoff without letting a fast producer build
/// a meaningful backlog. Tuning this against real item sizes is deferred
/// (design §10); [`stream_with_capacity`] exists so a caller — and the
/// back-pressure test — need not accept the default.
pub const DEFAULT_CAPACITY: usize = 64;

/// One event on the wire for one target.
///
/// Errors are ordinary items, so a failing target degrades to a partial result
/// rather than sinking the batch.
#[derive(Debug)]
pub enum StreamEvent<T> {
    /// An item the target's pipeline produced, or the error that ended it.
    Item(Result<T, AnalysisError>),
    /// The target's pipeline ran to exhaustion. Always the target's final
    /// event, and emitted even when it produced no items at all.
    ///
    /// This marker exists because a flat item stream cannot otherwise
    /// distinguish "this target completed and found nothing" — entirely
    /// normal — from "this target never ran". It carries no status of its own:
    /// `Item(Err(..))` neither suppresses nor replaces it.
    Completed,
}

/// The async read side of a [`stream`] run.
///
/// Holds a drop guard over the run's cancellation token, so letting this go —
/// a disconnected client, a `break` out of a consuming loop — stops the workers
/// rather than leaving them to finish a scan nobody will read.
pub struct AnalysisStream<K, T> {
    rx: mpsc::Receiver<(K, StreamEvent<T>)>,
    _guard: DropGuard,
}

impl<K, T> Stream for AnalysisStream<K, T> {
    type Item = (K, StreamEvent<T>);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().rx.poll_recv(cx)
    }
}

/// Cancels its token when dropped.
struct DropGuard(CancellationToken);

impl Drop for DropGuard {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

/// Streams every target's pipeline concurrently, interleaved in completion
/// order, with [`DEFAULT_CAPACITY`] items of buffering.
///
/// `make` builds one target's pipeline from its input and the run's
/// cancellation token. It runs on a worker thread, not on the caller's, so a
/// source that is expensive to *construct* costs nothing here.
///
/// `cancel` is additive: the run stops when either it or the returned stream's
/// own guard fires. Cancelling it does not disturb anything else the caller
/// attached to that token, and it does not itself observe stream drop — see
/// [`CancellationToken::child`].
///
/// Items arrive in completion order across targets, so **identity lives in the
/// key**, never in position.
///
/// # Example
///
/// ```
/// use futures_util::StreamExt;
/// use lox_analysis::pipeline::AnalysisError;
/// use lox_analysis::stream::{StreamEvent, stream};
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() {
/// let targets = [("a", 3usize), ("b", 0), ("c", 1)];
///
/// // Each target yields `n` items; `b` yields none.
/// let mut events = stream(
///     targets,
///     |n, _cancel| (0..n).map(Ok::<usize, AnalysisError>),
///     None,
/// );
///
/// let mut items = 0;
/// let mut completed = vec![];
/// while let Some((key, event)) = events.next().await {
///     match event {
///         StreamEvent::Item(item) => {
///             item.unwrap();
///             items += 1;
///         }
///         StreamEvent::Completed => completed.push(key),
///     }
/// }
///
/// assert_eq!(items, 4);
/// // Every target completes exactly once — including the one with no items.
/// completed.sort();
/// assert_eq!(completed, vec!["a", "b", "c"]);
/// # }
/// ```
pub fn stream<K, In, P, Item>(
    inputs: impl IntoIterator<Item = (K, In)>,
    make: impl Fn(In, CancellationToken) -> P + Send + Sync + 'static,
    cancel: Option<CancellationToken>,
) -> AnalysisStream<K, Item>
where
    P: Iterator<Item = Result<Item, AnalysisError>> + Send + 'static,
    K: Clone + Send + 'static,
    In: Send + 'static,
    Item: Send + 'static,
{
    stream_with_capacity(inputs, make, cancel, DEFAULT_CAPACITY)
}

/// [`stream`] with an explicit channel capacity.
///
/// # Panics
///
/// Panics if `capacity` is zero — a zero-capacity channel cannot make progress.
pub fn stream_with_capacity<K, In, P, Item>(
    inputs: impl IntoIterator<Item = (K, In)>,
    make: impl Fn(In, CancellationToken) -> P + Send + Sync + 'static,
    cancel: Option<CancellationToken>,
    capacity: usize,
) -> AnalysisStream<K, Item>
where
    P: Iterator<Item = Result<Item, AnalysisError>> + Send + 'static,
    K: Clone + Send + 'static,
    In: Send + 'static,
    Item: Send + 'static,
{
    assert!(capacity > 0, "channel capacity must be non-zero");

    // Drained here, on the caller's thread, so only the elements need
    // `Send + 'static` — not the container or the iterator producing them.
    let targets: Vec<(K, In)> = inputs.into_iter().collect();

    // A child rather than a clone: the guard below must be able to stop this
    // run without cancelling whatever else the caller hung off `cancel`.
    let token = match cancel {
        Some(external) => external.child(),
        None => CancellationToken::new(),
    };

    let (tx, rx) = mpsc::channel(capacity);
    let worker_token = token.clone();

    // Spawned, not run inline: `stream` returns a stream, so it must not block
    // the caller until the work is done.
    rayon::spawn(move || {
        targets.into_par_iter().for_each(|(key, input)| {
            // A target cancelled before its first item never appears on the
            // wire at all — deliberately, so the engine needs no `Started`
            // event. Callers reconcile against the input key set they own.
            if worker_token.is_cancelled() {
                return;
            }

            let pipeline = make(input, worker_token.clone());
            for item in pipeline {
                if worker_token.is_cancelled() {
                    return;
                }
                // A send failure means the consumer is gone; there is nobody
                // left to tell, so drop the target silently.
                if tx
                    .blocking_send((key.clone(), StreamEvent::Item(item)))
                    .is_err()
                {
                    return;
                }
            }

            // Reached only by exhausting the pipeline, which is what makes
            // `Completed` mean "ran to the end" and cancellation mean "no
            // marker". An `Err` item does not short-circuit this loop: a
            // pipeline that fuses after its error simply exhausts immediately.
            if worker_token.is_cancelled() {
                return;
            }
            let _ = tx.blocking_send((key, StreamEvent::Completed));
        });
    });

    AnalysisStream {
        rx,
        _guard: DropGuard(token),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Mutex, OnceLock};
    use std::time::Duration;

    use futures_util::StreamExt;
    use lox_core::error::LoxError;

    use super::*;

    /// Yields `Ok(i)` for `i in 0..n`, bumping `produced` as each item is
    /// *generated* — which is what makes back-pressure observable, since the
    /// count then reflects how far ahead of the consumer the worker ran.
    fn counted(
        n: usize,
        produced: Arc<AtomicUsize>,
    ) -> impl Iterator<Item = Result<usize, AnalysisError>> + Send + 'static {
        (0..n).map(move |i| {
            produced.fetch_add(1, Ordering::SeqCst);
            Ok(i)
        })
    }

    #[derive(Debug, thiserror::Error)]
    #[error("boom")]
    struct Boom;

    fn boom() -> AnalysisError {
        AnalysisError::Stage(LoxError::new(Boom))
    }

    /// Tallies `(items, errors, completed)` per key.
    async fn tally<K, T>(mut events: AnalysisStream<K, T>) -> Vec<(K, usize, usize, usize)>
    where
        K: Clone + Eq + std::hash::Hash + Ord,
    {
        let mut by_key: std::collections::HashMap<K, (usize, usize, usize)> = Default::default();
        while let Some((key, event)) = events.next().await {
            let entry = by_key.entry(key).or_default();
            match event {
                StreamEvent::Item(Ok(_)) => entry.0 += 1,
                StreamEvent::Item(Err(_)) => entry.1 += 1,
                StreamEvent::Completed => entry.2 += 1,
            }
        }
        let mut out: Vec<_> = by_key
            .into_iter()
            .map(|(k, (i, e, c))| (k, i, e, c))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    #[tokio::test]
    async fn every_started_target_completes_exactly_once() {
        let events = stream(
            [("a", 3usize), ("b", 1), ("c", 7)],
            |n, _| counted(n, Arc::new(AtomicUsize::new(0))),
            None,
        );
        assert_eq!(
            tally(events).await,
            vec![("a", 3, 0, 1), ("b", 1, 0, 1), ("c", 7, 0, 1)]
        );
    }

    #[tokio::test]
    async fn a_target_with_no_items_yields_a_bare_completed() {
        let events = stream([("empty", 0usize)], |n, _| counted(n, Arc::default()), None);
        assert_eq!(tally(events).await, vec![("empty", 0, 0, 1)]);
    }

    #[tokio::test]
    async fn an_error_mid_stream_does_not_suppress_completed() {
        // Two good items, then an error, then the pipeline fuses — exactly how
        // a real source behaves after a failed evaluation.
        let events = stream(
            [("a", ())],
            |_, _| {
                [Ok(1), Ok(2), Err(boom())]
                    .into_iter()
                    .collect::<Vec<_>>()
                    .into_iter()
            },
            None,
        );
        assert_eq!(tally(events).await, vec![("a", 2, 1, 1)]);
    }

    #[tokio::test]
    async fn a_target_failing_before_its_first_item_delivers_err_then_completed() {
        // The deferred-first-`Err` convention (design §4.1) on the wire: a
        // construction failure is indistinguishable from a mid-scan one.
        let events = stream(
            [("a", ())],
            |_, _| std::iter::once(Err::<usize, _>(boom())),
            None,
        );
        let mut events = std::pin::pin!(events);

        let (_, first) = events.next().await.expect("expected an error item");
        assert!(matches!(first, StreamEvent::Item(Err(_))));
        let (_, second) = events.next().await.expect("expected Completed");
        assert!(matches!(second, StreamEvent::Completed));
        assert!(events.next().await.is_none(), "stream should be exhausted");
    }

    #[tokio::test]
    async fn the_channel_bounds_how_far_the_producer_runs_ahead() {
        const CAPACITY: usize = 2;
        const READ: usize = 3;

        let produced = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&produced);
        let mut events = std::pin::pin!(stream_with_capacity(
            [("a", 10_000usize)],
            move |n, _| counted(n, Arc::clone(&counter)),
            None,
            CAPACITY,
        ));

        for _ in 0..READ {
            events.next().await.expect("expected an item");
        }
        // Give an unbounded producer every chance to run away.
        tokio::time::sleep(Duration::from_millis(100)).await;

        let ran_ahead = produced.load(Ordering::SeqCst);
        // Read items, plus a full channel, plus the one item the worker is
        // blocked trying to send.
        let ceiling = READ + CAPACITY + 1;
        assert!(
            ran_ahead <= ceiling,
            "producer ran {ran_ahead} items ahead of a {READ}-item consumer \
             (bound {ceiling}); the channel is not applying back-pressure"
        );
    }

    #[tokio::test]
    async fn dropping_the_stream_stops_the_workers() {
        let produced = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&produced);
        let mut events = stream_with_capacity(
            [("a", usize::MAX)],
            move |n, _| counted(n, Arc::clone(&counter)),
            None,
            4,
        );

        events.next().await.expect("expected an item");
        drop(events);

        // Let any in-flight send fail and the worker unwind.
        tokio::time::sleep(Duration::from_millis(50)).await;
        let after_drop = produced.load(Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(
            produced.load(Ordering::SeqCst),
            after_drop,
            "worker kept producing after the consumer went away"
        );
    }

    #[tokio::test]
    async fn dropping_the_stream_cancels_the_token_handed_to_make() {
        // The token is the path that bounds shutdown for a source which runs a
        // long way between items; closing the channel alone cannot.
        static SEEN: OnceLock<Mutex<Option<CancellationToken>>> = OnceLock::new();
        let seen = SEEN.get_or_init(|| Mutex::new(None));

        let mut events = stream(
            [("a", 8usize)],
            |n, cancel| {
                *SEEN.get().unwrap().lock().unwrap() = Some(cancel);
                counted(n, Arc::default())
            },
            None,
        );
        events.next().await.expect("expected an item");

        let token = seen.lock().unwrap().clone().expect("make was not called");
        assert!(!token.is_cancelled());
        drop(events);
        assert!(
            token.is_cancelled(),
            "stream drop must trip the token the source checks"
        );
    }

    #[tokio::test]
    async fn an_external_token_cancels_the_run() {
        let external = CancellationToken::new();
        let produced = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&produced);
        let mut events = std::pin::pin!(stream_with_capacity(
            [("a", usize::MAX)],
            move |n, _| counted(n, Arc::clone(&counter)),
            Some(external.clone()),
            4,
        ));

        events.next().await.expect("expected an item");
        external.cancel();

        // Drain whatever was already buffered; the stream must then end, and a
        // cancelled target gets no `Completed`.
        let mut completed = 0;
        while let Some((_, event)) = events.next().await {
            if matches!(event, StreamEvent::Completed) {
                completed += 1;
            }
        }
        assert_eq!(completed, 0, "a cancelled target must not report Completed");
    }

    #[tokio::test]
    async fn stream_drop_does_not_cancel_the_callers_token() {
        // The asymmetry that makes an external token safe to share.
        let external = CancellationToken::new();
        let events = stream(
            [("a", 4usize)],
            |n, _| counted(n, Arc::default()),
            Some(external.clone()),
        );
        drop(events);
        assert!(!external.is_cancelled());
    }

    #[tokio::test]
    async fn targets_interleave_rather_than_running_to_completion_in_order() {
        // Completion order, not input order — which is why the key travels with
        // every event instead of being inferred from position.
        if rayon::current_num_threads() < 2 {
            // With one worker `for_each` genuinely is sequential, so there is
            // no interleaving to observe and nothing this test could assert.
            return;
        }
        let events = stream(
            (0..8).map(|i| (i, i)),
            |i: usize, _| {
                (0..4).map(move |j| {
                    // Later targets are slower, so a strictly sequential engine
                    // would emit all of target 0 before any of target 7.
                    std::thread::sleep(Duration::from_millis(2 * i as u64));
                    Ok(j)
                })
            },
            None,
        );
        let mut events = std::pin::pin!(events);

        let mut order = vec![];
        while let Some((key, event)) = events.next().await {
            if matches!(event, StreamEvent::Item(_)) {
                order.push(key);
            }
        }

        assert_eq!(order.len(), 32);
        let sequential: Vec<usize> = (0..8).flat_map(|i| std::iter::repeat_n(i, 4)).collect();
        assert_ne!(order, sequential, "targets did not interleave");
    }

    #[test]
    #[should_panic(expected = "capacity must be non-zero")]
    fn zero_capacity_is_rejected() {
        let _ = stream_with_capacity([("a", 1usize)], |n, _| counted(n, Arc::default()), None, 0);
    }
}

/// Measures the engine's per-item overhead, which is the plan's go/no-go for
/// per-item sends versus chunked/coalesced ones:
///
/// ```text
/// cargo nextest run -p lox-analysis --features async \
///     -E 'test(measure_per_item_overhead)' --run-ignored only --no-capture --release
/// ```
#[cfg(test)]
mod measure {
    use std::hint::black_box;
    use std::time::Instant;

    use futures_util::StreamExt;

    use super::*;

    const ITEMS: usize = 200_000;

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "measurement, not an assertion"]
    async fn measure_per_item_overhead() {
        // Baseline: the same items, no engine. Whatever the difference is, it is
        // the channel handoff plus the rayon/async bridge — the producer itself
        // is the same trivial closure either way.
        let start = Instant::now();
        let mut n = 0usize;
        for item in (0..ITEMS).map(Ok::<usize, AnalysisError>) {
            n += black_box(item.unwrap());
        }
        let direct = start.elapsed();
        black_box(n);

        // One target: worst case for the engine, since there is no parallelism
        // to hide the handoff behind.
        let start = Instant::now();
        let mut events = std::pin::pin!(stream(
            [((), ITEMS)],
            |n: usize, _| (0..n).map(Ok::<usize, AnalysisError>),
            None,
        ));
        let mut items = 0usize;
        while let Some((_, event)) = events.next().await {
            if let StreamEvent::Item(item) = event {
                items += black_box(item.unwrap());
            }
        }
        let streamed = start.elapsed();
        black_box(items);

        let per_item = streamed.as_secs_f64() / ITEMS as f64;
        println!(
            "\n{ITEMS} items, single target, capacity {DEFAULT_CAPACITY}\n\
             \x20 direct iteration: {direct:>12.2?} ({:>8.1} ns/item)\n\
             \x20 through engine:   {streamed:>12.2?} ({:>8.1} ns/item)\n\
             \x20 engine overhead:  {:>12.2?} ({:>8.1} ns/item, {:.0}x)\n",
            direct.as_secs_f64() / ITEMS as f64 * 1e9,
            per_item * 1e9,
            streamed.saturating_sub(direct),
            (streamed.saturating_sub(direct)).as_secs_f64() / ITEMS as f64 * 1e9,
            streamed.as_secs_f64() / direct.as_secs_f64().max(f64::MIN_POSITIVE),
        );
        println!(
            "throughput: {:.0} items/s",
            ITEMS as f64 / streamed.as_secs_f64()
        );
    }
}
