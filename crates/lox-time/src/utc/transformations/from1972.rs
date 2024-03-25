/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Mod `from1972` exposes functions that return the UTC-TAI deltas for dates from 1972-01-01
//! onwards, which are always a whole number of leap seconds.

use crate::constants::f64::SECONDS_PER_DAY;
use crate::deltas::TimeDelta;
use crate::time_scales::TAI;
use crate::Time;
use lox_utils::slices::is_sorted_asc;
use lox_utils::types::Seconds;
use num::ToPrimitive;
use std::sync::OnceLock;

/// TODO: Hoist.
const MJD_J2000: f64 = 51544.5;

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
                    panic!("cannot express leap second epoch `{}` relative to J2000 in seconds as an i64", epoch)
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
            .for_each(|(i, epoch)| instants[i] = epoch + LEAP_SECONDS[i] as i64 - 1);
        instants
    })
}

/// The cumulative number of leap seconds at each epoch.
pub const LEAP_SECONDS: [f64; 28] = [
    10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0,
    26.0, 27.0, 28.0, 29.0, 30.0, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0,
];

/// Since 1972, the difference between TAI and UTC is always a whole number of leap seconds.
pub fn leap_seconds_for_mjd(mjd: f64) -> f64 {
    // Invariant: LS_EPOCHS must be sorted for the search below to work
    debug_assert!(is_sorted_asc(&MJD_LEAP_SECOND_EPOCHS));

    let threshold = mjd.floor() as u64;
    let position = MJD_LEAP_SECOND_EPOCHS
        .iter()
        .rposition(|item| item <= &threshold)
        .unwrap_or_else(|| {
            // Should never happen, since the public input is always a valid UTC datetime.
            panic!(
                "LS_EPOCHS contains no epoch less than or equal to MJD {}",
                threshold
            )
        });

    LEAP_SECONDS[position]
}

/// For dates from 1972-01-01, returns a [TimeDelta] representing the count of leap seconds between
/// TAI and UTC. Returns `None` for dates before 1972.
pub fn delta_tai_utc(tai: Time<TAI>) -> Option<TimeDelta> {
    j2000_tai_leap_second_epochs()
        .iter()
        .rev()
        .zip(LEAP_SECONDS.iter().rev())
        .find_map(|(&epoch, &leap_seconds)| {
            if epoch <= tai.seconds() {
                let delta = TimeDelta::from_decimal_seconds(leap_seconds).unwrap_or_else(|_| {
                    panic!(
                        "calculation of TAI-UTC delta produced an invalid TimeDelta: leap_seconds={}",
                        leap_seconds,
                    )
                });
                Some(delta)
            } else {
                None
            }
        })
}

/// UTC minus TAI. Calculates the correct leap second count for dates after 1972 by successive
/// approximation.
pub fn delta_utc_tai(mjd: f64) -> Seconds {
    let mut delta = 0.0;
    for _ in 1..=3 {
        delta = leap_seconds_for_mjd(mjd + delta / SECONDS_PER_DAY);
    }
    delta
}

impl Time<TAI> {
    /// Returns true if the TAI timestamp corresponds to a UTC leap second.
    pub fn is_leap_second(&self) -> bool {
        j2000_tai_leap_second_epochs()
            .binary_search(&self.seconds())
            .is_ok()
    }
}
