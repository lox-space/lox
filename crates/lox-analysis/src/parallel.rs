// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Fallible fan-out over a slice.
//!
//! On targets with threads the map runs on the `rayon` thread pool. On `wasm32`
//! targets it runs sequentially — `rayon` is not a dependency there, because
//! neither Pyodide's CPython nor a plain browser wasm module provides threads.
//! Both implementations share one signature, so call sites stay
//! target-agnostic.

#[cfg(not(target_family = "wasm"))]
use rayon::prelude::*;

/// Maps `f` over `items`, short-circuiting on the first error.
pub(crate) fn try_map<T, U, E, F>(items: &[T], f: F) -> Result<Vec<U>, E>
where
    T: Sync,
    U: Send,
    E: Send,
    F: Fn(&T) -> Result<U, E> + Send + Sync,
{
    try_map_above(items, 0, f)
}

/// Maps `f` over `items`, short-circuiting on the first error and fanning out
/// only when there are more than `threshold` items.
///
/// A threshold pays off where a single item is cheap enough that the thread pool
/// hop dominates; pass `0` — or use [`try_map`] — when each item is expensive
/// enough to always be worth fanning out.
#[cfg(not(target_family = "wasm"))]
pub(crate) fn try_map_above<T, U, E, F>(items: &[T], threshold: usize, f: F) -> Result<Vec<U>, E>
where
    T: Sync,
    U: Send,
    E: Send,
    F: Fn(&T) -> Result<U, E> + Send + Sync,
{
    if items.len() > threshold {
        items.par_iter().map(&f).collect()
    } else {
        items.iter().map(f).collect()
    }
}

/// Maps `f` over `items`, short-circuiting on the first error.
///
/// `threshold` is ignored: wasm targets have no threads to fan out to.
#[cfg(target_family = "wasm")]
pub(crate) fn try_map_above<T, U, E, F>(items: &[T], _threshold: usize, f: F) -> Result<Vec<U>, E>
where
    T: Sync,
    U: Send,
    E: Send,
    F: Fn(&T) -> Result<U, E> + Send + Sync,
{
    items.iter().map(f).collect()
}
