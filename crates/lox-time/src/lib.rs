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

pub mod calendar_dates;
pub mod deltas;
pub mod intervals;
pub mod julian_dates;
pub mod offsets;
pub mod ranges;
pub mod subsecond;
pub mod time;
pub mod time_of_day;
pub mod time_scales;
pub mod utc;

pub use time::{DynTime, Time, TimeBuilder, TimeError};
