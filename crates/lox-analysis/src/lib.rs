// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![warn(missing_docs)]

//! Visibility analysis, ground station and spacecraft asset modelling.

/// Asset definitions: ground stations, spacecraft, constellations, and scenarios.
pub mod assets;
/// Visibility analysis: line-of-sight, elevation masks, passes, and interval computation.
pub mod visibility;
