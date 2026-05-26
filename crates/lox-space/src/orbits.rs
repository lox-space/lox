// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

// Re-export lox_orbits submodule contents at this level to avoid stutter
// (e.g. `lox_space::orbits::CartesianOrbit` instead of `lox_space::orbits::orbits::CartesianOrbit`).
pub use lox_orbits::DVec3;
pub use lox_orbits::constellations;
pub use lox_orbits::events;
pub use lox_orbits::ground;
pub use lox_orbits::orbits::*;
pub use lox_orbits::propagators;

#[cfg(feature = "python")]
/// Python bindings exposed via PyO3.
pub mod python;
