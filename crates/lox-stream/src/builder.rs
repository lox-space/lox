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
}
