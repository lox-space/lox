// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Bridge `lox_stream::Stream<T, E>` into a
//! `futures::Stream<Result<T, E>>` for connectrpc server-streaming handlers.
//!
//! `lox_stream::Stream` already implements `futures_core::Stream`, so the
//! bridge is a zero-overhead newtype wrapper that merely maps the error
//! type to `ConnectError` and pins the result in a `Box`.

use connectrpc::ConnectError;
use futures::StreamExt;
use lox_stream::Stream as LoxStream;

/// Wrap a `lox_stream::Stream<T, E>` as a connectrpc `ServiceStream<T>`.
///
/// `lox_stream::Stream` is already a `futures_core::Stream`, so no
/// thread-hopping is required. Items are yielded in completion order
/// (rayon's par_stream contract). Drop cancels the underlying workers via
/// the `CancellationToken` embedded in the `LoxStream`.
pub fn bridge<T, E>(lox: LoxStream<T, E>) -> connectrpc::ServiceStream<T>
where
    T: Send + 'static,
    E: std::fmt::Display + Send + 'static,
{
    Box::pin(lox.map(|res| {
        res.map_err(|e| ConnectError::internal(e.to_string()))
    }))
}
