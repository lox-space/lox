// SPDX-FileCopyrightText: 2023 Andrei Zisu <matzipan@gmail.com>
// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
//
// SPDX-License-Identifier: MPL-2.0

#![warn(missing_docs)]

//! Orbit representations, propagators, ground stations, constellations, and event detection.

pub use glam::DVec3;

/// Satellite constellation builders (Walker, Flower, Street of Coverage).
pub mod constellations;
/// Root-finding event detection and interval computation.
pub mod events;
/// Ground location and observables computation.
pub mod ground;
/// Orbit types, trajectories, ensembles, and builders.
pub mod orbits;
/// Orbit propagators (Vallado, J2, SGP4, trajectory replay).
pub mod propagators;
