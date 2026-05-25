// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Streaming primitive for parallel work.
//!
//! Provides a [`Stream`] type that delivers results from a rayon-driven
//! producer in completion order. The stream implements
//! [`futures_core::Stream`] for async consumers and offers
//! [`Stream::blocking_next`] for sync callers.
//!
//! See the [`par_stream`] builder for the entry point.

mod builder;
mod cancellation;
mod error;
mod stream;

// pub use builder::par_stream;
pub use cancellation::CancellationToken;
// pub use error::OnError;
// pub use stream::Stream;
