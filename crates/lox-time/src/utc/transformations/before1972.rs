/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Mod `pre1972` exposes functions for interpolating the UTC-TAI delta for dates between
//! 1960-01-01 and 1972-01-01, during which there are 10 leap seconds distributed by a linear
//! function.
//!
//! Data sourced from ftp://maia.usno.navy.mil/ser7/tai-utc.dat.

use lox_math::constants::f64::time::SECONDS_PER_DAY;
use lox_math::slices::is_sorted_asc;

use crate::deltas::{TimeDelta, ToDelta};
use crate::julian_dates::Epoch::ModifiedJulianDate;
use crate::julian_dates::JulianDate;
use crate::julian_dates::Unit::Days;
use crate::time_scales::Tai;
use crate::utc::Utc;
use crate::Time;

const EPOCHS: [u64; 14] = [
    36934, 37300, 37512, 37665, 38334, 38395, 38486, 38639, 38761, 38820, 38942, 39004, 39126,
    39887,
];

const OFFSETS: [f64; 14] = [
    1.417818, 1.422818, 1.372818, 1.845858, 1.945858, 3.240130, 3.340130, 3.440130, 3.540130,
    3.640130, 3.740130, 3.840130, 4.313170, 4.213170,
];

const DRIFT_EPOCHS: [u64; 14] = [
    37300, 37300, 37300, 37665, 37665, 38761, 38761, 38761, 38761, 38761, 38761, 38761, 39126,
    39126,
];

const DRIFT_RATES: [f64; 14] = [
    0.0012960, 0.0012960, 0.0012960, 0.0011232, 0.0011232, 0.0012960, 0.0012960, 0.0012960,
    0.0012960, 0.0012960, 0.0012960, 0.0012960, 0.0025920, 0.0025920,
];

/// UTC minus TAI. Returns [None] if the input is before 1960-01-01, when UTC is defined from,
/// although this is impossible for all properly constructed [UtcDateTime] instances.
pub fn delta_utc_tai(utc: &Utc) -> Option<TimeDelta> {
    // Invariant: EPOCHS must be sorted for the search below to work
    debug_assert!(is_sorted_asc(&EPOCHS));

    let mjd = utc.to_delta().days_since_modified_julian_epoch();
    let threshold = mjd.floor() as u64;
    let position = EPOCHS.iter().rposition(|item| item <= &threshold)?;
    let raw_delta =
        OFFSETS[position] + (mjd - DRIFT_EPOCHS[position] as f64) * DRIFT_RATES[position];
    let delta = TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|_| {
        unreachable!(
            "calculation of UTC-TAI delta produced an invalid TimeDelta: raw_delta={}",
            raw_delta,
        )
    });

    Some(-delta)
}

/// TAI minus UTC.
pub fn delta_tai_utc(tai: &Time<Tai>) -> Option<TimeDelta> {
    // Invariant: EPOCHS must be sorted for the search below to work
    debug_assert!(is_sorted_asc(&EPOCHS));

    let mjd = tai.julian_date(ModifiedJulianDate, Days);
    let threshold = mjd.floor() as u64;
    let position = EPOCHS.iter().rposition(|item| item <= &threshold)?;
    let rate_utc = DRIFT_RATES[position] / SECONDS_PER_DAY;
    let rate_tai = rate_utc / (1.0 + rate_utc) * SECONDS_PER_DAY;
    let offset = OFFSETS[position];
    let dt = mjd - DRIFT_EPOCHS[position] as f64 - offset / SECONDS_PER_DAY;
    let raw_delta = offset + dt * rate_tai;
    let delta = TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|_| {
        unreachable!(
            "calculation of TAI-UTC delta produced an invalid TimeDelta: raw_delta={}",
            raw_delta,
        )
    });

    Some(delta)
}
