// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

/// Per-call policy for how a streamed pipeline reacts to per-unit errors.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OnError {
    /// Emit per-unit errors as `Err(e)` items but keep producing
    /// subsequent items.
    Continue,
    /// On the first per-unit error, cancel all remaining work. Up to
    /// `worker_count` concurrent errors may be observed before the stream
    /// terminates.
    Abort,
}
