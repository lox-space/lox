/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

/*!
    Module `julian_dates` exposes constants related to standard Julian epochs and dates in a variety
    of formats.
*/

use crate::deltas::TimeDelta;
use crate::subsecond::Subsecond;

pub const SECONDS_BETWEEN_JD_AND_J2000: i64 = 211813488000;

pub const SECONDS_BETWEEN_MJD_AND_J2000: i64 = 4453444800;

pub const SECONDS_BETWEEN_J1950_AND_J2000: i64 = 1577880000;

pub const SECONDS_BETWEEN_J1977_AND_J2000: i64 = 725803200;

/// 4713 BC January 1 12:00
pub const J0: TimeDelta = TimeDelta {
    seconds: -SECONDS_BETWEEN_JD_AND_J2000,
    subsecond: Subsecond(0.0),
};

/// 1977 January 1 00:00, at which the following are equal:
/// * 1977-01-01T00:00:00.000 TAI
/// * 1977-01-01T00:00:32.184 TT
/// * 1977-01-01T00:00:32.184 TCG
/// * 1977-01-01T00:00:32.184 TCB
pub const J77: TimeDelta = TimeDelta {
    seconds: -SECONDS_BETWEEN_J1977_AND_J2000,
    subsecond: Subsecond(0.0),
};
