/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Mod `from1972` exposes functions that return the UTC-TAI deltas for dates from 1972-01-01
//! onwards, which are always a whole number of leap seconds.

use std::sync::OnceLock;

use num::ToPrimitive;

use lox_utils::constants::f64::time::{MJD_J2000, SECONDS_PER_DAY};
use lox_utils::slices::is_sorted_asc;

use crate::base_time::BaseTime;
use crate::deltas::TimeDelta;
use crate::time_scales::Tai;
use crate::utc::UtcDateTime;
use crate::Time;

/// MJDs corresponding to the start of each leap second epoch from 1972-01-01 onwards.
const MJD_LEAP_SECOND_EPOCHS: [u64; 28] = [
    41317, 41499, 41683, 42048, 42413, 42778, 43144, 43509, 43874, 44239, 44786, 45151, 45516,
    46247, 47161, 47892, 48257, 48804, 49169, 49534, 50083, 50630, 51179, 53736, 54832, 56109,
    57204, 57754,
];

/// Leap second epochs in seconds relative to J2000 UTC.
fn j2000_utc_leap_second_epochs() -> &'static [i64; 28] {
    static LS_EPOCHS_J2000: OnceLock<[i64; 28]> = OnceLock::new();
    LS_EPOCHS_J2000.get_or_init(|| {
        let mut j2000_epochs = [0i64; 28];
        MJD_LEAP_SECOND_EPOCHS.iter().enumerate().for_each(|(i, epoch)| {
            let j2000_epoch = ((*epoch as f64 - MJD_J2000) * SECONDS_PER_DAY).to_i64()
                .unwrap_or_else(|| {
                    unreachable!("cannot express leap second epoch `{}` relative to J2000 in seconds as an i64", epoch)
                });
            j2000_epochs[i] = j2000_epoch;
        });

        debug_assert!(is_sorted_asc(&j2000_epochs));

        j2000_epochs
    })
}

/// Leap second epochs in seconds relative to J2000 TAI.
pub fn j2000_tai_leap_second_epochs() -> &'static [i64; 28] {
    static TAI_LS_INSTANTS: OnceLock<[i64; 28]> = OnceLock::new();
    TAI_LS_INSTANTS.get_or_init(|| {
        let mut instants = [0i64; 28];
        j2000_utc_leap_second_epochs()
            .iter()
            .enumerate()
            .for_each(|(i, epoch)| instants[i] = epoch + LEAP_SECONDS[i] - 1);
        instants
    })
}

/// The cumulative number of leap seconds at each epoch.
pub const LEAP_SECONDS: [i64; 28] = [
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33,
    34, 35, 36, 37,
];

/// For dates from 1972-01-01, returns a [TimeDelta] representing the count of leap seconds between
/// TAI and UTC. Returns `None` for dates before 1972.
pub fn delta_tai_utc(tai: Time<Tai>) -> Option<TimeDelta> {
    j2000_tai_leap_second_epochs()
        .iter()
        .rev()
        .zip(LEAP_SECONDS.iter().rev())
        .find_map(|(&epoch, &leap_seconds)| {
            if epoch <= tai.seconds() {
                Some(TimeDelta::from_seconds(leap_seconds))
            } else {
                None
            }
        })
}

/// UTC minus TAI. Calculates the correct leap second count for dates after 1972 by simple lookup.
pub fn delta_utc_tai(utc: UtcDateTime) -> Option<TimeDelta> {
    let base_time = BaseTime::from_utc_datetime(utc);
    j2000_utc_leap_second_epochs()
        .iter()
        .rev()
        .zip(LEAP_SECONDS.iter().rev())
        .find_map(|(&epoch, &leap_seconds)| {
            if epoch <= base_time.seconds() {
                Some(TimeDelta::from_seconds(leap_seconds))
            } else {
                None
            }
        })
        .map(|mut delta| {
            if utc.time.second == 60 {
                delta.seconds -= 1;
            }
            -delta
        })
}

impl Time<Tai> {
    /// Returns true if the TAI timestamp corresponds to a UTC leap second since 1972.
    pub fn is_leap_second(&self) -> bool {
        j2000_tai_leap_second_epochs()
            .binary_search(&self.seconds())
            .is_ok()
    }
}
