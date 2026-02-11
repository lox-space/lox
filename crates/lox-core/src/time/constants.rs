// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::time::{calendar_dates::Date, deltas::TimeDelta};

/// The Unix Epoch (1970-01-01T00:00:00 TAI) as a [TimeDelta].
pub const UNIX_EPOCH: TimeDelta = Date::new_unchecked(1970, 1, 1).to_delta();
