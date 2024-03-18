/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::leap_seconds::gen::{LEAP_SECONDS, LS_EPOCHS};
use thiserror::Error;

const MJD_EPOCH: f64 = 2400000.5;
const SECONDS_PER_DAY: f64 = 86400.0;

// Constants for calculating the offset between TAI and UTC for
// dates between 1960-01-01 and 1972-01-01
// See ftp://maia.usno.navy.mil/ser7/tai-utc.dat
// Section taken from
// https://github.com/JuliaTime/LeapSeconds.jl/blob/master/src/LeapSeconds.jl#L16

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

#[derive(Clone, Debug, Error, PartialEq)]
pub enum LeapSecondError {
    #[error("UTC is not defined for dates before 1960-01-01")]
    UTCDateBefore1960,
    #[error("UTC date is out of range")]
    UTCDateOutOfRange,
}

/// This type is used for increased precision
///
/// The first part is the UTC day number, and the second part is the second offset.
pub struct TwoPartDateTime {
    pub day: f64,
    pub seconds_offset: f64,
}

impl From<(f64, f64)> for TwoPartDateTime {
    fn from(item: (f64, f64)) -> Self {
        TwoPartDateTime {
            day: item.0,
            seconds_offset: item.1,
        }
    }
}

fn is_sorted(array: &[u64]) -> bool {
    array.windows(2).all(|x| x[0] <= x[1])
}

fn leap_seconds(mjd: f64) -> Result<f64, LeapSecondError> {
    // Invariant: LS_EPOCHS must be sorted for the search below to work
    assert!(is_sorted(&LS_EPOCHS));

    let threshold = mjd.floor() as u64;
    let position = LS_EPOCHS
        .iter()
        .rposition(|item| item <= &threshold)
        .ok_or(LeapSecondError::UTCDateOutOfRange)?;

    Ok(LEAP_SECONDS[position])
}

/// Returns the difference between UTC and TAI for a given date
///
/// Input is a two-part UTC Julian datetime.
pub fn offset_utc_tai(utc_date_time: &TwoPartDateTime) -> Result<f64, LeapSecondError> {
    // This function uses the [ERFA convention](https://github.com/liberfa/erfa/blob/master/src/dtf2d.c#L49)
    // for Julian day numbers representing UTC dates during leap seconds.
    let mjd = utc_date_time.day - MJD_EPOCH + utc_date_time.seconds_offset;

    // Before 1960-01-01
    if mjd < 36934.0 {
        return Err(LeapSecondError::UTCDateBefore1960);
    }

    // Before 1972-01-01
    if mjd < LS_EPOCHS[1] as f64 {
        // Invariant: EPOCHS must be sorted for the search below to work
        debug_assert!(is_sorted(&EPOCHS));

        let threshold = mjd.floor() as u64;
        let position = EPOCHS
            .iter()
            .rposition(|item| item <= &threshold)
            .ok_or(LeapSecondError::UTCDateOutOfRange)?;

        let offset =
            OFFSETS[position] + (mjd - DRIFT_EPOCHS[position] as f64) * DRIFT_RATES[position];

        return Ok(-offset);
    }

    let mut offset = 0.0;
    for _ in 1..=3 {
        offset = leap_seconds(mjd + offset / SECONDS_PER_DAY)?;
    }

    Ok(-offset)
}

/// Returns the difference between TAI and UTC for a given date
///
/// Input is a two-part TAI Julian datetime.
pub fn offset_tai_utc(tai_date_time: &TwoPartDateTime) -> Result<f64, LeapSecondError> {
    let mjd = tai_date_time.day - MJD_EPOCH + tai_date_time.seconds_offset;

    // Before 1960-01-01
    if mjd < 36934.0 {
        return Err(LeapSecondError::UTCDateBefore1960);
    }

    // Before 1972-01-01
    if mjd < LS_EPOCHS[1] as f64 {
        // Invariant: EPOCHS must be sorted for the search below to work
        debug_assert!(is_sorted(&EPOCHS));

        let threshold = mjd.floor() as u64;
        let position = EPOCHS
            .iter()
            .rposition(|item| item <= &threshold)
            .ok_or(LeapSecondError::UTCDateOutOfRange)?;

        let rate_utc = DRIFT_RATES[position] / SECONDS_PER_DAY;
        let rate_tai = rate_utc / (1.0 + rate_utc) * SECONDS_PER_DAY;
        let offset = OFFSETS[position];
        let dt = mjd - DRIFT_EPOCHS[position] as f64 - offset / SECONDS_PER_DAY;

        return Ok(offset + dt * rate_tai);
    }

    leap_seconds(mjd)
}

#[cfg(test)]
pub mod test {
    use float_eq::assert_float_eq;

    use super::*;

    #[test]
    fn test_offset_utc_tai() {
        // Values validated against LeapSeconds.jl

        // datetime2julian(DateTime(1990, 1, 1))
        assert_eq!(
            offset_utc_tai(&TwoPartDateTime::from((2.4478925e6, 0f64))),
            Ok(-25.0)
        );
        // datetime2julian(DateTime(2000, 1, 1))
        assert_eq!(
            offset_utc_tai(&TwoPartDateTime::from((2.4515445e6, 0f64))),
            Ok(-32.0)
        );
        // datetime2julian(DateTime(2020, 1, 1))
        assert_eq!(
            offset_utc_tai(&TwoPartDateTime::from((2.4577545e6, 0f64))),
            Ok(-37.0)
        );
    }

    #[test]
    fn test_offset_tai_utc() {
        // Values validated against LeapSeconds.jl

        // datetime2julian(DateTime(1990, 1, 1))
        assert_eq!(
            offset_tai_utc(&TwoPartDateTime::from((2.4478925e6, 0f64))),
            Ok(25.0)
        );
        // datetime2julian(DateTime(2000, 1, 1))
        assert_eq!(
            offset_tai_utc(&TwoPartDateTime::from((2.4515445e6, 0f64))),
            Ok(32.0)
        );
        // datetime2julian(DateTime(2020, 1, 1))
        assert_eq!(
            offset_tai_utc(&TwoPartDateTime::from((2.4577545e6, 0f64))),
            Ok(37.0)
        );
    }

    #[test]
    fn test_range_warnings() {
        // Values validated against LeapSeconds.jl

        // datetime2julian(DateTime(1959, 1, 1))
        assert_eq!(
            offset_tai_utc(&TwoPartDateTime::from((2.4365695e6, 0f64))),
            Err(LeapSecondError::UTCDateBefore1960)
        );
        // datetime2julian(DateTime(1959, 1, 1))
        assert_eq!(
            offset_utc_tai(&TwoPartDateTime::from((2.4365695e6, 0f64))),
            Err(LeapSecondError::UTCDateBefore1960)
        );
    }

    #[test]
    fn test_during_leap_seconds() {
        let parameters: Vec<(TwoPartDateTime, TwoPartDateTime)> = vec![
            (
                // dt = (1963, 7, 23, 14, 12, 3.0)
                // utc = ERFA.dtf2d("UTC", dt...)
                TwoPartDateTime::from((2.4382335e6, 0.5917013888888889)),
                // tai = ERFA.utctai(utc...)
                TwoPartDateTime::from((2.4382335e6, 0.5917301446782292)),
            ),
            (
                // dt = (2012, 6, 30, 23, 59, 59.0)
                // utc = ERFA.dtf2d("UTC", dt...)
                TwoPartDateTime::from((2.4561085e6, 0.9999768521197672)),
                // tai = ERFA.utctai(utc...)
                TwoPartDateTime::from((2.4561085e6, 1.0003819444444444)),
            ),
            (
                // dt = (2012, 6, 30, 23, 59, 60.0)
                // utc = ERFA.dtf2d("UTC", dt...)
                TwoPartDateTime::from((2.4561085e6, 0.9999884260598836)),
                // tai = ERFA.utctai(utc...)
                TwoPartDateTime::from((2.4561085e6, 1.0003935185185184)),
            ),
            (
                // dt = (2012, 6, 30, 23, 59, 60.5)
                // utc = ERFA.dtf2d("UTC", dt...)
                TwoPartDateTime::from((2.4561085e6, 0.9999942130299417)),
                // tai = ERFA.utctai(utc...)
                TwoPartDateTime::from((2.4561085e6, 1.0003993055555553)),
            ),
            (
                // dt = (2012, 7, 1, 0, 0, 0.0)
                // utc = ERFA.dtf2d("UTC", dt...)
                TwoPartDateTime::from((2.4561095e6, 0.0)),
                // tai = ERFA.utctai(utc...)
                TwoPartDateTime::from((2.4561095e6, 0.0004050925925925926)),
            ),
        ];

        for parameter in parameters {
            let diff_utc_tai = offset_utc_tai(&parameter.0).unwrap() / SECONDS_PER_DAY;
            let diff_tai_utc = offset_tai_utc(&parameter.1).unwrap() / SECONDS_PER_DAY;

            assert!((diff_utc_tai.abs() - diff_tai_utc.abs()).abs() < 1e-14);

            let utc_jd = parameter.0.day + parameter.0.seconds_offset;
            let tai_jd = parameter.1.day + parameter.1.seconds_offset;

            assert_float_eq!(utc_jd - diff_utc_tai - tai_jd, 0.0, abs <= 1e-9);
            assert_float_eq!(tai_jd - diff_tai_utc - utc_jd, 0.0, abs <= 1e-9);
        }
    }
}
