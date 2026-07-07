// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Scalar callback abstraction for numerical algorithms.

use crate::error::{BoxedError, LoxError};

/// A callable scalar function for numerical algorithms.
pub trait Callback {
    /// Evaluates the function at `v`.
    fn call(&self, v: f64) -> Result<f64, LoxError>;
}

impl<F> Callback for F
where
    F: Fn(f64) -> f64,
{
    fn call(&self, v: f64) -> Result<f64, LoxError> {
        Ok(self(v))
    }
}

/// Adapts a fallible closure into a [`Callback`].
///
/// Plain closures implement [`Callback`] directly; wrap a closure in
/// `Fallible` when its evaluation can fail.
pub struct Fallible<F>(pub F);

impl<F> Callback for Fallible<F>
where
    F: Fn(f64) -> Result<f64, BoxedError>,
{
    fn call(&self, v: f64) -> Result<f64, LoxError> {
        (self.0)(v).map_err(LoxError::from)
    }
}
