// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Fan-out that degrades to sequential iteration when `parallel` is off.
//!
//! Rayon is behind the `parallel` feature (design §9), so every fan-out in the
//! crate goes through here rather than calling `par_iter` directly. That keeps
//! the `#[cfg]` in one place instead of at each call site, and makes the
//! rayon-free build a compile-time guarantee rather than something to remember.

/// Maps `f` over `items`, short-circuiting on the first error.
///
/// Runs in parallel only when the `parallel` feature is enabled *and* `parallel`
/// is `true`; otherwise iterates in order. The bounds are stated
/// unconditionally so that turning the feature off cannot make previously
/// rejected code compile — a rayon-free build should not be a *laxer* build.
pub(crate) fn try_map<T, U, E>(
    items: &[T],
    parallel: bool,
    f: impl Fn(&T) -> Result<U, E> + Send + Sync,
) -> Result<Vec<U>, E>
where
    T: Send + Sync,
    U: Send,
    E: Send,
{
    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        if parallel {
            return items.par_iter().map(f).collect();
        }
    }
    #[cfg(not(feature = "parallel"))]
    let _ = parallel;

    items.iter().map(f).collect()
}
