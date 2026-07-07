// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Module `python` aggregates the Python binding for `lox-time`.

/// Python bindings for time delta types.
pub mod deltas;
/// Python bindings for time interval types.
pub mod intervals;
/// Python bindings for the core `Time` type.
pub mod time;
/// Python bindings for time scale types.
pub mod time_scales;
/// Python bindings for time series types.
pub mod time_series;
/// Python bindings for UTC time types.
pub mod utc;
