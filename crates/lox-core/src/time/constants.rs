// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Well-known time constants.

use crate::time::{calendar_dates::Date, deltas::TimeDelta};

/// The duration from J2000 (2000-01-01T12:00:00) to the Unix epoch (1970-01-01T00:00:00).
///
/// No time scale is implied; the value becomes an instant only when paired
/// with one.
pub const UNIX_EPOCH: TimeDelta = Date::new_unchecked(1970, 1, 1).to_delta();
