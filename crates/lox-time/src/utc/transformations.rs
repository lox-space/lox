/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::constants::f64::SECONDS_PER_DAY;
use crate::constants::julian_dates::MJD_OFFSET;
use lox_eop::LoxEopError;
use thiserror::Error;

use crate::deltas::{TimeDelta, TimeDeltaError};
use crate::utc::UTCUndefinedError;

/// MJDs corresponding to the start of each leap second epoch.
const LS_EPOCHS: [u64; 28] = [
    41317, 41499, 41683, 42048, 42413, 42778, 43144, 43509, 43874, 44239, 44786, 45151, 45516,
    46247, 47161, 47892, 48257, 48804, 49169, 49534, 50083, 50630, 51179, 53736, 54832, 56109,
    57204, 57754,
];

/// The cumulative number of leap seconds at each epoch.
const LEAP_SECONDS: [f64; 28] = [
    10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0,
    26.0, 27.0, 28.0, 29.0, 30.0, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0,
];

// Constants for calculating the offset between TAI and UTC for dates between 1960-01-01 and
// 1972-01-01 See ftp://maia.usno.navy.mil/ser7/tai-utc.dat.
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

fn leap_seconds(mjd: f64) -> f64 {
    // Invariant: LS_EPOCHS must be sorted for the search below to work
    assert!(is_sorted(&LS_EPOCHS));

    let threshold = mjd.floor() as u64;
    let position = LS_EPOCHS
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

/// Returns the difference between UTC and TAI for a given date
///
/// Input is a two-part UTC Julian datetime.
pub fn offset_utc_tai(utc_date_time: &TwoPartDateTime) -> Result<f64, UTCUndefinedError> {
    // This function uses the [ERFA convention](https://github.com/liberfa/erfa/blob/master/src/dtf2d.c#L49)
    // for Julian day numbers representing UTC dates during leap seconds.
    let mjd = utc_date_time.day - MJD_OFFSET + utc_date_time.seconds_offset;

    // Before 1960-01-01
    if mjd < 36934.0 {
        return Err(UTCUndefinedError);
    }

    // Before 1972-01-01
    if mjd < LS_EPOCHS[1] as f64 {
        // Invariant: EPOCHS must be sorted for the search below to work
        debug_assert!(is_sorted(&EPOCHS));

        let threshold = mjd.floor() as u64;
        let position = EPOCHS
            .iter()
            .rposition(|item| item <= &threshold)
            // Thanks to the 1960 check, rustc knows this result is always Some statically.
            .expect("EPOCHS contains no epoch less than or equal to MJD");

        let offset =
            OFFSETS[position] + (mjd - DRIFT_EPOCHS[position] as f64) * DRIFT_RATES[position];

        return Ok(-offset);
    }

    let mut offset = 0.0;
    for _ in 1..=3 {
        offset = leap_seconds(mjd + offset / SECONDS_PER_DAY);
    }

    Ok(-offset)
}

/// Returns the difference between TAI and UTC for a non-leap-second UTC datetime expressed as a
/// pseudo-MJD.
///
/// It is _not_ suitable for calculating the TAI-UTC delta during a leap second, since
/// this information isn't obtainable from the MJD representation. Use [delta_tai_leap_second_utc]
/// to handle this case.
pub(crate) fn delta_tai_utc(mjd: f64) -> Result<TimeDelta, UTCUndefinedError> {
    // let mjd = tai_date_time.day - MJD_EPOCH + tai_date_time.seconds_offset;

    // Before 1960-01-01
    if mjd < 36934.0 {
        return Err(UTCUndefinedError);
    }

    // Before 1972-01-01
    let raw_delta = if mjd < LS_EPOCHS[1] as f64 {
        // Invariant: EPOCHS must be sorted for the search below to work
        debug_assert!(is_sorted(&EPOCHS));

        let threshold = mjd.floor() as u64;
        let position = EPOCHS
            .iter()
            .rposition(|item| item <= &threshold)
            // Thanks to the 1960 check, rustc knows this result is always Some statically.
            .expect("EPOCHS contains no epoch less than or equal to MJD");

        let rate_utc = DRIFT_RATES[position] / SECONDS_PER_DAY;
        let rate_tai = rate_utc / (1.0 + rate_utc) * SECONDS_PER_DAY;
        let offset = OFFSETS[position];
        let dt = mjd - DRIFT_EPOCHS[position] as f64 - offset / SECONDS_PER_DAY;

        offset + dt * rate_tai
    } else {
        leap_seconds(mjd)
    };

    let delta = TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|_| {
        panic!(
            "calculation of TAI-UTC delta produced an invalid TimeDelta: raw_delta={}",
            raw_delta,
        )
    });
    Ok(delta)
}

/// Returns the difference between TAI and UTC for a UTC datetime during a leap second, expressed as
/// a pseudo-MJD.
///
/// It is _not_ suitable for calculating the TAI-UTC delta during a leap second, since
/// this information isn't obtainable from the MJD representation. Use [delta_tai_leap_second_utc]
/// to handle this case.
pub(crate) fn delta_tai_leap_second_utc(mjd: f64) -> Result<TimeDelta, UTCUndefinedError> {
    delta_tai_utc(mjd).map(|delta| delta - TimeDelta::from_seconds(1))
}

#[cfg(test)]
pub mod test {
    use rstest::rstest;

    use crate::base_time::BaseTime;
    use crate::calendar_dates::Date;
    use crate::julian_dates::Epoch::ModifiedJulianDate;
    use crate::julian_dates::JulianDate;
    use crate::julian_dates::Unit::Days;
    use crate::subsecond::Subsecond;
    use crate::utc::UTCDateTime;
    use crate::utc::UTC;

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
        // 2016-12-31 23:59:60 UTC
        assert_eq!(
            offset_utc_tai(&TwoPartDateTime::from((2.4577545e6, 0f64))),
            Ok(-37.0)
        );
    }

    #[rstest]
    // Exercises the branch where mjd < LS_EPOCHS[1].
    #[case::y1971(
        UTCDateTime::new(
            Date::new(1971, 1, 1).unwrap(),
            UTC::default(),
        ).unwrap(),
        Ok(TimeDelta::from_decimal_seconds(8.946161731615149).unwrap())
    )]
    #[case::y1990(
        UTCDateTime::new(
            Date::new(1990, 1, 1).unwrap(),
            UTC::default(),
        ).unwrap(),
        Ok(TimeDelta::from_seconds(25))
    )]
    #[case::y2k(
        UTCDateTime::new(
            Date::new(2000, 1, 1).unwrap(),
            UTC::default(),
        ).unwrap(),
        Ok(TimeDelta::from_seconds(32))
    )]
    // delta_tai_utc is expected _not_ to adjust for the case where the input time is on a leap
    // second, and should return 37 rather than the correct offset of 36 for this leap second.
    #[case::leap_second(
        UTCDateTime::new(
            Date::new(2016, 12, 31).unwrap(),
            UTC::new(23, 59, 60, Subsecond::default()).unwrap(),
        ).unwrap(),
        Ok(TimeDelta::from_seconds(37))
    )]
    fn test_delta_tai_utc(
        #[case] utc: UTCDateTime,
        #[case] expected: Result<TimeDelta, UTCUndefinedError>,
    ) {
        let mjd = BaseTime::from_utc_datetime(utc).julian_date(ModifiedJulianDate, Days);
        let actual = delta_tai_utc(mjd);
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case::leap_second_adjustment(
        UTCDateTime::new(
            Date::new(2016, 12, 31).unwrap(),
            UTC::new(23, 59, 60, Subsecond::default()).unwrap(),
        ).unwrap(),
        Ok(TimeDelta::from_seconds(36))
    )]
    fn test_delta_tai_leap_second_utc(
        #[case] utc: UTCDateTime,
        #[case] expected: Result<TimeDelta, UTCUndefinedError>,
    ) {
        let mjd = BaseTime::from_utc_datetime(utc).julian_date(ModifiedJulianDate, Days);
        let actual = delta_tai_leap_second_utc(mjd);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_range_warnings() {
        // Values validated against LeapSeconds.jl

        // datetime2julian(DateTime(1959, 1, 1))
        assert_eq!(
            offset_utc_tai(&TwoPartDateTime::from((2.4365695e6, 0f64))),
            Err(UTCUndefinedError)
        );
    }

    // #[test]
    // fn test_during_leap_seconds() {
    //     let parameters: Vec<(TwoPartDateTime, TwoPartDateTime)> = vec![
    //         (
    //             // dt = (1963, 7, 23, 14, 12, 3.0)
    //             // utc = ERFA.dtf2d("UTC", dt...)
    //             TwoPartDateTime::from((2.4382335e6, 0.5917013888888889)),
    //             // tai = ERFA.utctai(utc...)
    //             TwoPartDateTime::from((2.4382335e6, 0.5917301446782292)),
    //         ),
    //         (
    //             // dt = (2012, 6, 30, 23, 59, 59.0)
    //             // utc = ERFA.dtf2d("UTC", dt...)
    //             TwoPartDateTime::from((2.4561085e6, 0.9999768521197672)),
    //             // tai = ERFA.utctai(utc...)
    //             TwoPartDateTime::from((2.4561085e6, 1.0003819444444444)),
    //         ),
    //         (
    //             // dt = (2012, 6, 30, 23, 59, 60.0)
    //             // utc = ERFA.dtf2d("UTC", dt...)
    //             TwoPartDateTime::from((2.4561085e6, 0.9999884260598836)),
    //             // tai = ERFA.utctai(utc...)
    //             TwoPartDateTime::from((2.4561085e6, 1.0003935185185184)),
    //         ),
    //         (
    //             // dt = (2012, 6, 30, 23, 59, 60.5)
    //             // utc = ERFA.dtf2d("UTC", dt...)
    //             TwoPartDateTime::from((2.4561085e6, 0.9999942130299417)),
    //             // tai = ERFA.utctai(utc...)
    //             TwoPartDateTime::from((2.4561085e6, 1.0003993055555553)),
    //         ),
    //         (
    //             // dt = (2012, 7, 1, 0, 0, 0.0)
    //             // utc = ERFA.dtf2d("UTC", dt...)
    //             TwoPartDateTime::from((2.4561095e6, 0.0)),
    //             // tai = ERFA.utctai(utc...)
    //             TwoPartDateTime::from((2.4561095e6, 0.0004050925925925926)),
    //         ),
    //     ];
    //
    //     for parameter in parameters {
    //         let diff_utc_tai = offset_utc_tai(&parameter.0).unwrap() / SECONDS_PER_DAY;
    //         let diff_tai_utc = delta_tai_utc(&parameter.1).unwrap() / SECONDS_PER_DAY;
    //
    //         assert!((diff_utc_tai.abs() - diff_tai_utc.abs()).abs() < 1e-14);
    //
    //         let utc_jd = parameter.0.day + parameter.0.seconds_offset;
    //         let tai_jd = parameter.1.day + parameter.1.seconds_offset;
    //
    //         assert_float_eq!(utc_jd - diff_utc_tai - tai_jd, 0.0, abs <= 1e-9);
    //         assert_float_eq!(tai_jd - diff_tai_utc - utc_jd, 0.0, abs <= 1e-9);
    //     }
    // }
}
