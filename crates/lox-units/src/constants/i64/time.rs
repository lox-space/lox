/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module `i64` exposes time-related `i64` constants.

pub const SECONDS_PER_MINUTE: i64 = 60;

pub const SECONDS_PER_HOUR: i64 = 60 * SECONDS_PER_MINUTE;

pub const SECONDS_PER_DAY: i64 = 24 * SECONDS_PER_HOUR;

pub const SECONDS_PER_HALF_DAY: i64 = SECONDS_PER_DAY / 2;

pub const SECONDS_PER_JULIAN_YEAR: i64 = 31_557_600;

pub const SECONDS_PER_JULIAN_CENTURY: i64 = SECONDS_PER_JULIAN_YEAR * 100;
