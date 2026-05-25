// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::cancellation::CancellationToken;
use crate::error::OnError;
use crate::stream::Stream;

/// Spawns a parallel pipeline over `items`. Each input is passed to `work`
/// on a rayon worker; results are streamed back in completion order.
///
/// `capacity` is the bounded channel capacity — when the buffer is full,
/// producers block, applying backpressure to slow consumers.
///
/// `on_error` controls whether per-unit errors abort the remaining work or
/// are emitted as `Err` items while the stream continues.
pub fn par_stream<I, T, E, F>(items: I, capacity: usize, on_error: OnError, work: F) -> Stream<T, E>
where
    I: IntoParallelIterator + Send + 'static,
    I::Item: Send,
    T: Send + 'static,
    E: Send + 'static,
    F: Fn(I::Item, &CancellationToken) -> Result<T, E> + Send + Sync + 'static,
{
    let (tx, rx) = async_channel::bounded(capacity);
    let token = CancellationToken::new();
    let work_token = token.clone();

    rayon::spawn(move || {
        items.into_par_iter().for_each(|item| {
            if work_token.is_cancelled() {
                return;
            }
            let result = work(item, &work_token);
            let is_err = result.is_err();
            let _ = tx.send_blocking(result);
            if is_err && matches!(on_error, OnError::Abort) {
                work_token.cancel();
            }
        });
    });

    Stream { rx, token }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_yields_all_results() {
        let s = par_stream(0..100, 16, OnError::Continue, |i, _| Ok::<i32, ()>(i * 2));
        let collected: Vec<i32> = s
            .collect_blocking()
            .into_iter()
            .map(|r| r.unwrap())
            .collect();
        let mut sorted = collected;
        sorted.sort();
        let expected: Vec<i32> = (0..100).map(|i| i * 2).collect();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn empty_input_terminates_immediately() {
        let mut s = par_stream(0..0, 4, OnError::Continue, |i, _| Ok::<i32, ()>(i));
        assert_eq!(s.blocking_next(), None);
    }

    #[test]
    fn fast_units_arrive_before_slow_ones() {
        use std::time::Duration;

        // Index 0 sleeps 200ms; indices 1..=4 sleep 10ms each. We expect the
        // four fast indices to arrive before index 0.
        let s = par_stream(0..5, 8, OnError::Continue, |i, _| {
            let dur = if i == 0 {
                Duration::from_millis(200)
            } else {
                Duration::from_millis(10)
            };
            std::thread::sleep(dur);
            Ok::<i32, ()>(i)
        });

        let mut order = Vec::new();
        for item in s.collect_blocking() {
            order.push(item.unwrap());
        }
        // index 0 must not be the first item to arrive
        assert_ne!(order[0], 0, "slow unit arrived first: {order:?}");
    }

    #[test]
    fn continue_yields_every_error_then_finishes() {
        // Fail when i % 7 == 6. Inputs 0..100 → i ∈ {6,13,20,27,34,41,48,55,62,69,76,83,90,97}
        // = 14 errors, 86 oks.
        let s = par_stream(0..100, 16, OnError::Continue, |i, _| {
            if i % 7 == 6 {
                Err::<i32, i32>(i)
            } else {
                Ok::<i32, i32>(i)
            }
        });

        let items: Vec<_> = s.collect_blocking();
        let errs: Vec<_> = items.iter().filter_map(|r| r.as_ref().err()).collect();
        let oks: Vec<_> = items.iter().filter_map(|r| r.as_ref().ok()).collect();
        assert_eq!(errs.len(), 14);
        assert_eq!(oks.len(), 86);
    }

    #[test]
    fn abort_terminates_after_bounded_concurrent_errors() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let invocations = Arc::new(AtomicUsize::new(0));
        let c = invocations.clone();

        // One bad input triggers Abort; over 10_000 inputs, no more than
        // worker_count units should observably run.
        let s = par_stream(0..10_000_usize, 16, OnError::Abort, move |i, _| {
            c.fetch_add(1, Ordering::SeqCst);
            if i == 0 { Err::<usize, ()>(()) } else { Ok(i) }
        });

        let items: Vec<_> = s.collect_blocking();
        let errs = items.iter().filter(|r| r.is_err()).count();
        assert!(errs >= 1);
        // Upper bound: roughly the rayon worker count + a small slop. We
        // assert << 10_000 to catch a missing cancel().
        let total = invocations.load(Ordering::SeqCst);
        let max_expected = rayon::current_num_threads() * 4;
        assert!(
            total <= max_expected,
            "expected ≤ {max_expected} invocations after abort, got {total}"
        );
    }

    #[test]
    fn drop_cancels_in_flight_workers() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::time::Duration;

        let invocations = Arc::new(AtomicUsize::new(0));
        let c = invocations.clone();

        let mut s = par_stream(0..10_000_usize, 4, OnError::Continue, move |i, _| {
            c.fetch_add(1, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(20));
            Ok::<usize, ()>(i)
        });

        // Receive one item then drop.
        let _ = s.blocking_next();
        drop(s);

        // Give workers time to observe the cancellation.
        std::thread::sleep(Duration::from_millis(200));
        let after_drop = invocations.load(Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(500));
        let later = invocations.load(Ordering::SeqCst);
        // Bounded growth: at most one extra unit per worker.
        let max_growth = rayon::current_num_threads();
        assert!(
            later - after_drop <= max_growth,
            "growth {} > {} after drop",
            later - after_drop,
            max_growth,
        );
    }

    #[test]
    fn explicit_cancel_stops_workers() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::time::Duration;

        let invocations = Arc::new(AtomicUsize::new(0));
        let c = invocations.clone();

        let mut s = par_stream(0..10_000_usize, 4, OnError::Continue, move |i, _| {
            c.fetch_add(1, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(20));
            Ok::<usize, ()>(i)
        });

        let _ = s.blocking_next();
        s.cancel();

        std::thread::sleep(Duration::from_millis(200));
        let after_cancel = invocations.load(Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(500));
        let later = invocations.load(Ordering::SeqCst);
        let max_growth = rayon::current_num_threads();
        assert!(
            later - after_cancel <= max_growth,
            "growth {} > {} after cancel",
            later - after_cancel,
            max_growth,
        );

        // Subsequent poll terminates cleanly.
        let _drained: Vec<_> = (&mut s).collect_blocking_inplace();
    }

    // ----- helper -----

    trait CollectBlocking<T> {
        fn collect_blocking(self) -> Vec<T>;
    }

    impl<T, E> CollectBlocking<Result<T, E>> for Stream<T, E> {
        fn collect_blocking(mut self) -> Vec<Result<T, E>> {
            let mut v = Vec::new();
            while let Some(item) = self.blocking_next() {
                v.push(item);
            }
            v
        }
    }

    // ----- additional helper for in-place draining -----
    trait CollectBlockingInplace<T> {
        fn collect_blocking_inplace(self) -> Vec<T>;
    }
    impl<T, E> CollectBlockingInplace<Result<T, E>> for &mut Stream<T, E> {
        fn collect_blocking_inplace(self) -> Vec<Result<T, E>> {
            let mut v = Vec::new();
            while let Some(item) = self.blocking_next() {
                v.push(item);
            }
            v
        }
    }
}
