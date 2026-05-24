// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![warn(missing_docs)]

//! Visibility analysis, ground station and spacecraft asset modelling.

/// Asset definitions: ground stations, spacecraft, constellations, and scenarios.
pub mod assets;
/// AOI imaging event detection: sub-satellite point, swath, and off-nadir coverage.
#[cfg(feature = "imaging")]
pub mod imaging;
/// Power budget analysis: eclipse detection, beta angle, solar flux.
pub mod power;
/// Visibility analysis: line-of-sight, elevation masks, passes, and interval computation.
pub mod visibility;
