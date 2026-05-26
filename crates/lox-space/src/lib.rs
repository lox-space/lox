// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![warn(missing_docs)]

//! Umbrella crate for the lox astrodynamics toolkit.
//!
//! `lox-space` re-exports the individual `lox-*` crates under stable, stutter-free module
//! paths and provides a [`prelude`] of the most commonly used types. Each module mirrors the
//! corresponding standalone crate; enable the matching cargo feature to pull it in.

/// Mission analysis: access, visibility, link budgets, and ground-station modelling
/// (re-exports [`lox_analysis`]).
#[cfg(feature = "analysis")]
pub mod analysis;

/// Celestial bodies and their physical and rotational properties (re-exports [`lox_bodies`]).
#[cfg(feature = "bodies")]
pub mod bodies;

/// Satellite constellation builders and geometry (re-exports `lox_orbits::constellations`).
#[cfg(feature = "orbits")]
pub mod constellations;

/// Communications modelling: antennas, links, and signal chains (re-exports [`lox_comms`]).
#[cfg(feature = "comms")]
pub mod comms;

/// Core numeric types, units, and constants (re-exports [`lox_core`]).
#[cfg(feature = "core")]
pub mod core;

/// Earth-specific models such as geodesy and orientation (re-exports [`lox_earth`]).
#[cfg(feature = "earth")]
pub mod earth;

/// Planetary and lunar ephemerides (re-exports [`lox_ephem`]).
#[cfg(feature = "ephem")]
pub mod ephem;

/// Reference frames, rotations, and coordinate transformations (re-exports [`lox_frames`]).
#[cfg(feature = "frames")]
pub mod frames;

/// Parsers for astrodynamics data file formats (re-exports [`lox_io`]).
#[cfg(feature = "io")]
pub mod io;

/// ITU-R atmospheric propagation models (re-exports [`lox_itur`]).
#[cfg(feature = "itur")]
pub mod itur;

/// Math utilities used across the toolkit (re-exports [`lox_math`]).
#[cfg(feature = "math")]
pub mod math;

/// CCSDS Orbit Data Message types and (de-)serialization (re-exports [`lox_odm`]).
#[cfg(feature = "odm")]
pub mod odm;

/// Orbits, propagators, events, and ground tracks (re-exports [`lox_orbits`]).
#[cfg(feature = "orbits")]
pub mod orbits;

/// Time scales, epochs, and date arithmetic (re-exports [`lox_time`]).
#[cfg(feature = "time")]
pub mod time;

/// Construct a [`time::Time`] — re-export of the [`lox_time::time!`] macro.
#[cfg(feature = "time")]
#[macro_export]
macro_rules! time {
    ($($args:tt)*) => { $crate::__private_time!($($args)*) };
}
#[cfg(feature = "time")]
#[doc(hidden)]
pub use lox_time::time as __private_time;

/// Construct a UTC timestamp — re-export of the [`lox_time::utc!`] macro.
#[cfg(feature = "time")]
#[macro_export]
macro_rules! utc {
    ($($args:tt)*) => { $crate::__private_utc!($($args)*) };
}
#[cfg(feature = "time")]
#[doc(hidden)]
pub use lox_time::utc as __private_utc;

/// Physical units and quantities (re-exports [`lox_units`]).
#[cfg(feature = "units")]
pub mod units;

pub mod prelude;

/// Python bindings exposed via PyO3.
#[cfg(feature = "python")]
pub mod python;
