// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![warn(missing_docs)]

//! Visibility analysis, ground station and spacecraft asset modelling.

/// Asset definitions: ground stations, spacecraft, constellations, and scenarios.
pub mod assets;
/// Root-finding event detection and interval computation.
pub mod events;
/// AOI imaging event detection: sub-satellite point, swath, and off-nadir coverage.
#[cfg(feature = "imaging")]
pub mod imaging;
/// The eager `*Analysis` implementations, staged for deletion (see the module
/// docs). Not part of the supported surface.
#[doc(hidden)]
pub mod legacy;
mod par;
/// Lazy analysis pipelines: the `Source` scan, `Stage` transforms, and caller-owned fan-out.
pub mod pipeline;
/// Power budget analysis: eclipse detection, beta angle, solar flux.
pub mod power;
/// Streaming execution: rayon workers to an async consumer over a bounded channel.
#[cfg(feature = "async")]
pub mod stream;
/// Visibility analysis: line-of-sight, elevation masks, passes, and interval computation.
pub mod visibility;
