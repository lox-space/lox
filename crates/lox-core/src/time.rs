// SPDX-FileCopyrightText: 2023 Andrei Zisu <matzipan@gmail.com>
// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Durations: [`TimeDelta`](deltas::TimeDelta) and time constants.
//!
//! Calendar dates, Julian dates, time of day and instants in time scales live in `lox-time`.

/// Conversions between Lox and `chrono` types.
#[cfg(feature = "chrono")]
pub mod chrono;
pub mod constants;
pub mod deltas;
