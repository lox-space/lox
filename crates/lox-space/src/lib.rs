// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![warn(missing_docs)]

//! Lox is an MPLv2-licensed Rust astrodynamics library with first-class Python bindings for
//! orbital mechanics, mission analysis, and telecommunications.
//!
//! `lox-space` is the main entry point for both the Rust and Python APIs, re-exporting all
//! functionality from the Lox ecosystem through a unified interface and providing a
//! [`prelude`] of the most commonly used types. Each module mirrors the corresponding
//! standalone crate; enable the matching cargo feature to pull it in.
//!
//! # Python Quick Start
//!
//! ```python
//! import lox_space as lox
//!
//! # Parse a UTC epoch and convert to the TDB time scale
//! epoch = lox.UTC.from_iso("2025-01-01T12:00:00").to_scale("TDB")
//!
//! # Load Earth orientation parameters
//! provider = lox.EOPProvider("finals2000A.all.csv")
//!
//! # Design a sun-synchronous orbit at 800 km altitude with a 10:30 LTAN
//! sso = lox.Keplerian.sso(
//!     epoch, altitude=800 * lox.km, ltan=(10, 30), provider=provider
//! )
//!
//! # Convert to Cartesian state and propagate with J2 perturbations
//! state = sso.to_cartesian()
//! j2 = lox.J2(state)
//! trajectory = j2.propagate(epoch, end=epoch + 100 * lox.minutes)
//! ```
//!
//! # Rust Quick Start
//!
//! ```no_run
//! use lox_space::prelude::*;
//!
//! let epoch = Utc::from_iso("2025-01-01T12:00:00").unwrap().to_time().to_scale(TimeScale::Tdb);
//! let provider = EopParser::new().from_path("finals2000A.all.csv").parse().unwrap();
//!
//! let sso = SsoBuilder::default()
//!     .with_provider(&provider)
//!     .with_time(epoch)
//!     .with_altitude(800.0.km())
//!     .with_ltan(10, 30)
//!     .build()
//!     .unwrap();
//!
//! // Convert to Cartesian state and propagate with J2 perturbations
//! let state = sso.to_cartesian();
//! let j2 = J2Propagator::try_new(state).unwrap();
//! let end = epoch + TimeDelta::from_minutes(100);
//! let trajectory = j2.propagate(Interval::new(epoch, end)).unwrap();
//! ```
//!
//! # Installation
//!
//! ## Python
//!
//! ```sh
//! uv add lox-space
//! # or
//! pip install lox-space
//! ```
//!
//! ## Rust
//!
//! ```sh
//! cargo add lox-space
//! ```
//!
//! Or add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! lox-space = "0.1"
//! ```
//!
//! # Features
//!
//! - **Orbital Mechanics** — Keplerian elements, state vectors, SSO design, Vallado/J2/SGP4 propagation, TLE parsing
//! - **Time Systems** — TAI, TT, TDB, TCB, TCG, UTC, UT1; femtosecond precision, leap-second aware
//! - **Reference Frames** — ICRF, ITRF, TEME; CIO and equinox-based transformation chains
//! - **Ground Stations** — Visibility windows, elevation masks, pass prediction
//! - **Constellation Design** — Walker Delta/Star, Street-of-Coverage, Flower
//! - **RF Link Budgets** — Antenna patterns, modulation schemes, path loss
//! - **Python Bindings** — Full API with type stubs and NumPy interop
//!
//! # Status
//!
//! Lox is pre-1.0. The API may change between releases.
//!
//! # Documentation
//!
//! - Python: <https://python.lox.rs>
//! - Rust: <https://docs.rs/lox-space>

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
