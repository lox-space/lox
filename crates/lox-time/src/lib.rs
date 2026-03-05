// SPDX-FileCopyrightText: 2023 Andrei Zisu <matzipan@gmail.com>
// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

/*!
    `lox-time` defines an API for working with instants in astronomical time scales.

    # Overview

    `lox_time` exposes:
    - the marker trait [TimeScale] and zero-sized implementations representing the most common,
      continuous astronomical time scales;
    - the concrete type [Time] representing an instant in a [TimeScale];
    - [Utc], the only discontinuous time representation supported by Lox;
    - the [TryToScale] and [ToScale] traits, supporting transformations between pairs of time
      scales;
    - standard implementations of the most common time scale transformations.

    # Continuous vs discontinuous timescales

    Internally, Lox uses only continuous time scales (i.e. time scales without leap seconds). An
    instance of [Time] represents an instant in time generic over a continuous [TimeScale].

    [Utc] is used strictly as an I/O time format, which must be transformed into a continuous time
    scale before use in the wider Lox ecosystem.

    This separation minimises the complexity in working with leap seconds, confining these
    transformations to the crate boundaries.
*/

#![warn(missing_docs)]

/// Calendar date types (re-exported from `lox-core`).
pub mod calendar_dates;
/// Conversions between `lox-time` and `chrono` types.
#[cfg(feature = "chrono")]
pub mod chrono;
/// Time deltas (re-exported from `lox-core`).
pub mod deltas;
/// Time intervals and set operations on them.
pub mod intervals;
/// Julian date types (re-exported from `lox-core`).
pub mod julian_dates;
/// Time scale offset computation.
pub mod offsets;
/// Sub-second precision types (re-exported from `lox-core`).
pub mod subsecond;
/// The [`Time`] type representing an instant in a continuous time scale.
pub mod time;
/// Time-of-day types (re-exported from `lox-core`).
pub mod time_of_day;
/// Astronomical time scale definitions.
pub mod time_scales;
/// Coordinated Universal Time (UTC) with leap-second support.
pub mod utc;

pub use time::{DynTime, Time, TimeBuilder, TimeError};
