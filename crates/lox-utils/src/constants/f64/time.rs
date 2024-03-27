/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module `time` exposes constants time-related constants shared between Lox crates.

/*
   Days
*/

pub const DAYS_PER_JULIAN_CENTURY: f64 = 36525.0;

/*
   Seconds
*/
pub const SECONDS_PER_MINUTE: f64 = 60.0;

pub const SECONDS_PER_HOUR: f64 = SECONDS_PER_MINUTE * 60.0;

pub const SECONDS_PER_DAY: f64 = SECONDS_PER_HOUR * 24.0;

pub const SECONDS_PER_JULIAN_YEAR: f64 = SECONDS_PER_DAY * 365.25;

pub const SECONDS_PER_JULIAN_CENTURY: f64 = SECONDS_PER_JULIAN_YEAR * 100.0;

/*
   Julian dates
*/

pub const MJD_J2000: f64 = 51544.5;
