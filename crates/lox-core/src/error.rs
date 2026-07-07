// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Type-erased error handling for callback boundaries.
//!
//! User-provided callbacks (objective functions, detection functions, Python
//! callables) can fail with error types that `lox-core` cannot name. [`LoxError`]
//! erases those types so they can travel through solvers and detectors, while
//! [`find_source`] recovers them at the boundary via downcasting.

use alloc::boxed::Box;
use thiserror::Error;

/// A boxed, type-erased error.
pub type BoxedError = Box<dyn core::error::Error + Send + Sync + 'static>;

/// A type-erased error carrying an arbitrary failure across an abstraction
/// boundary.
///
/// Unlike a transparent wrapper, `LoxError` exposes the erased error as its
/// [`source`](core::error::Error::source), so the wrapped error remains
/// reachable when walking an error chain with [`find_source`].
#[derive(Debug, Error)]
#[error("{0}")]
pub struct LoxError(#[source] BoxedError);

impl LoxError {
    /// Wraps an arbitrary error.
    pub fn new(error: impl Into<BoxedError>) -> Self {
        LoxError(error.into())
    }

    /// Returns the wrapped error.
    pub fn into_inner(self) -> BoxedError {
        self.0
    }

    /// Searches the wrapped error and its source chain for an error of type `E`.
    pub fn downcast_ref<E: core::error::Error + 'static>(&self) -> Option<&E> {
        find_source(self.0.as_ref())
    }
}

impl From<&str> for LoxError {
    fn from(s: &str) -> Self {
        LoxError(s.into())
    }
}

impl From<BoxedError> for LoxError {
    fn from(e: BoxedError) -> Self {
        LoxError(e)
    }
}

/// Searches `err` and its source chain for an error of type `E`.
///
/// Transparent wrappers (thiserror's `#[error(transparent)]`) forward `source`
/// to the wrapped error's source and thus hide themselves from the chain;
/// [`LoxError`] deliberately appears as a real link so that errors it erases
/// stay reachable.
pub fn find_source<'a, E: core::error::Error + 'static>(
    err: &'a (dyn core::error::Error + 'static),
) -> Option<&'a E> {
    let mut current: Option<&(dyn core::error::Error + 'static)> = Some(err);
    while let Some(e) = current {
        if let Some(found) = e.downcast_ref::<E>() {
            return Some(found);
        }
        current = e.source();
    }
    None
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[derive(Debug, Error)]
    #[error("leaf failure")]
    struct LeafError;

    #[derive(Debug, Error)]
    #[error(transparent)]
    struct Transparent(#[from] LoxError);

    #[test]
    fn test_lox_error_display_is_inner() {
        let err = LoxError::new(LeafError);
        assert_eq!(err.to_string(), "leaf failure");

        let from_str: LoxError = "boom".into();
        assert_eq!(from_str.to_string(), "boom");

        let boxed: BoxedError = "kaboom".into();
        let from_boxed: LoxError = boxed.into();
        assert_eq!(from_boxed.to_string(), "kaboom");
    }

    #[test]
    fn test_into_inner_downcasts_to_original() {
        let err = LoxError::new(LeafError);
        let inner = err.into_inner();
        assert!(inner.downcast_ref::<LeafError>().is_some());
    }

    #[test]
    fn test_find_source_through_transparent_wrappers() {
        // A transparent wrapper hides itself from the chain, but the erased
        // error stays reachable because `LoxError` is a real chain link.
        let err = Transparent(LoxError::new(LeafError));
        let found = find_source::<LeafError>(&err).expect("leaf must be reachable");
        assert_eq!(found.to_string(), "leaf failure");
        assert!(find_source::<core::fmt::Error>(&err).is_none());
    }

    #[test]
    fn test_downcast_ref_finds_wrapped_error() {
        let err = LoxError::new(LeafError);
        assert!(err.downcast_ref::<LeafError>().is_some());
        assert!(err.downcast_ref::<core::fmt::Error>().is_none());
    }
}
