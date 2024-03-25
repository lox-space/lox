/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashSet;
use std::sync::OnceLock;

use num::ToPrimitive;

use lox_utils::types::Seconds;

use crate::base_time::BaseTime;
use crate::constants::f64::SECONDS_PER_DAY;
use crate::deltas::TimeDelta;
use crate::julian_dates::Epoch::ModifiedJulianDate;
use crate::julian_dates::JulianDate;
use crate::julian_dates::Unit::Days;
use crate::time_scales::TAI;
use crate::utc::{UTCDateTime, UTCUndefinedError};
use crate::wall_clock::WallClock;
use crate::Time;

impl From<UTCDateTime> for Time<TAI> {
    fn from(utc: UTCDateTime) -> Self {
        let base = BaseTime::from_utc_datetime(utc);
        let idx = ls_epochs_j2000()
            .iter()
            .rposition(|item| *item <= base.seconds);

        let delta = if let Some(idx) = idx {
            // 1972-01-01 and after.
            let mut delta = LEAP_SECONDS[idx] as i64;
            if utc.time.second() == 60 {
                delta -= 1;
            }
            TimeDelta::from_seconds(delta)
        } else {
            // Before 1972-01-01.
            let mjd = base.julian_date(ModifiedJulianDate, Days);
            let negative_delta = delta_utc_tai(mjd).unwrap_or_else(|err| {
                // Impossible, since UTCDateTime objects are always in range.
                panic!("{}", err)
            });
            -negative_delta
        };

        Time::from_base_time(TAI, base + delta)
    }
}

impl TryFrom<Time<TAI>> for UTCDateTime {
    type Error = UTCUndefinedError;

    fn try_from(tai: Time<TAI>) -> Result<Self, Self::Error> {
        let idx = tai_leap_second_instants()
            .iter()
            .rposition(|item| *item <= tai.seconds());

        let delta = if let Some(idx) = idx {
            // 1972-01-01 and after.
            TimeDelta::from_seconds(LEAP_SECONDS[idx] as i64) // TODO: Do these need to be f64?
        } else {
            // Before 1972-01-01.
            let mjd = tai.julian_date(ModifiedJulianDate, Days);
            delta_tai_utc(mjd)?
        };

        let base_time = tai.base_time() - delta;
        let mut utc = UTCDateTime::from_base_time(base_time)?;
        if tai.is_leap_second() {
            utc.time.second += 1;
        }

        Ok(utc)
    }
}

impl Time<TAI> {
    fn is_leap_second(&self) -> bool {
        tai_leap_second_instants()
            .binary_search(&self.seconds())
            .is_ok()
    }
}

/// MJDs corresponding to the start of each leap second epoch.
const LS_EPOCHS: [u64; 28] = [
    41317, 41499, 41683, 42048, 42413, 42778, 43144, 43509, 43874, 44239, 44786, 45151, 45516,
    46247, 47161, 47892, 48257, 48804, 49169, 49534, 50083, 50630, 51179, 53736, 54832, 56109,
    57204, 57754,
];

/// TODO: Hoist.
const MJD_J2000: f64 = 51544.5;

/// Leap second epochs relative to J2000 UTC.
fn ls_epochs_j2000() -> &'static [i64; 28] {
    static LS_EPOCHS_J2000: OnceLock<[i64; 28]> = OnceLock::new();
    LS_EPOCHS_J2000.get_or_init(|| {
        let mut j2000_epochs = [0i64; 28];
        LS_EPOCHS.iter().enumerate().for_each(|(i, epoch)| {
            let j2000_epoch = ((*epoch as f64 - MJD_J2000) * SECONDS_PER_DAY).to_i64()
                .unwrap_or_else(|| {
                    panic!("cannot express leap second epoch `{}` relative to J2000 in seconds as an i64", epoch)
                });
            j2000_epochs[i] = j2000_epoch;
        });

        debug_assert!(is_sorted(&j2000_epochs));

        j2000_epochs
    })
}

fn tai_leap_second_instants() -> &'static [i64; 28] {
    static TAI_LS_INSTANTS: OnceLock<[i64; 28]> = OnceLock::new();
    TAI_LS_INSTANTS.get_or_init(|| {
        let mut instants = [0i64; 28];
        ls_epochs_j2000()
            .iter()
            .enumerate()
            .for_each(|(i, epoch)| instants[i] = epoch + LEAP_SECONDS[i] as i64 - 1);
        instants
    })
}

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

fn is_sorted<T: Ord>(array: &[T]) -> bool {
    array.windows(2).all(|x| x[0] <= x[1])
}

/// Since 1972, the difference between TAI and UTC is always a whole number of leap seconds.
fn get_tabulated_leap_seconds(mjd: f64) -> f64 {
    // Invariant: LS_EPOCHS must be sorted for the search below to work
    debug_assert!(is_sorted(&LS_EPOCHS));

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

// 1960-01-01
const MJD_UTC_DEFINED: f64 = 36934.0;

fn utc_is_defined_for(mjd: f64) -> bool {
    mjd >= MJD_UTC_DEFINED
}

fn is_before_1972(mjd: f64) -> bool {
    mjd < LS_EPOCHS[0] as f64
}

/// Given an input UTC datetime expressed as a pseudo-MJD, returns the difference between UTC and
/// TAI. The result is always negative, as TAI is ahead of UTC.
fn delta_utc_tai(mjd: f64) -> Result<TimeDelta, UTCUndefinedError> {
    if !utc_is_defined_for(mjd) {
        return Err(UTCUndefinedError);
    }

    let raw_delta = if is_before_1972(mjd) {
        interpolate_delta_utc_tai(mjd)
    } else {
        approximate_delta_utc_tai(mjd)
    };

    let delta = TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|_| {
        panic!(
            "calculation of UTC-TAI delta produced an invalid TimeDelta: raw_delta={}",
            raw_delta,
        )
    });

    Ok(-delta)
}

/// For the internal 1960 to 1972, there are 10 leap seconds distributed by a linear function.
fn interpolate_delta_utc_tai(mjd: f64) -> Seconds {
    // Invariant: EPOCHS must be sorted for the search below to work
    debug_assert!(is_sorted(&EPOCHS));

    let threshold = mjd.floor() as u64;
    let position = EPOCHS
        .iter()
        .rposition(|item| item <= &threshold)
        // Thanks to the 1960 check, rustc knows this result is always Some statically.
        .expect("EPOCHS contains no epoch less than or equal to MJD");

    OFFSETS[position] + (mjd - DRIFT_EPOCHS[position] as f64) * DRIFT_RATES[position]
}

/// Arrive at the correct leap second count for dates after 1972 by successive approximation using
/// tabular leap second data.
fn approximate_delta_utc_tai(mjd: f64) -> Seconds {
    let mut delta = 0.0;
    for _ in 1..=3 {
        delta = get_tabulated_leap_seconds(mjd + delta / SECONDS_PER_DAY);
    }
    delta
}

/// Returns the difference between TAI and UTC for a non-leap-second UTC datetime expressed as a
/// pseudo-MJD.
fn delta_tai_utc(mjd: f64) -> Result<TimeDelta, UTCUndefinedError> {
    if !utc_is_defined_for(mjd) {
        return Err(UTCUndefinedError);
    }

    let raw_delta = if is_before_1972(mjd) {
        interpolate_delta_tai_utc(mjd)
    } else {
        get_tabulated_leap_seconds(mjd)
    };

    let delta = TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|_| {
        panic!(
            "calculation of TAI-UTC delta produced an invalid TimeDelta: raw_delta={}",
            raw_delta,
        )
    });
    Ok(delta)
}

/// There are 10 leap seconds in the span 1960 to 1972, distributed by a linear function.
fn interpolate_delta_tai_utc(mjd: f64) -> f64 {
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
}

#[cfg(test)]
pub mod test {
    use rstest::rstest;

    use crate::base_time::BaseTime;
    use crate::calendar_dates::Date;
    use crate::subsecond::Subsecond;
    use crate::utc::UTCDateTime;
    use crate::utc::UTC;

    use super::*;

    #[rstest]
    #[case::before_1972(utc_1971_01_01(), tai_at_utc_1971_01_01())]
    #[case::before_leap_second(utc_1s_before_2016_leap_second(), tai_1s_before_2016_leap_second())]
    #[case::during_leap_second(utc_during_2016_leap_second(), tai_during_2016_leap_second())]
    #[case::after_leap_second(utc_1s_after_2016_leap_second(), tai_1s_after_2016_leap_second())]
    #[should_panic]
    #[case::illegal_utc_datetime(unconstructable_utc_datetime(), &Time::new(TAI, 0, Subsecond::default()))]
    fn test_tai_from_utc(#[case] utc: &UTCDateTime, #[case] expected: &Time<TAI>) {
        let actual = (*utc).into();
        assert_eq!(*expected, actual);
    }

    #[rstest]
    #[case::before_1972(tai_at_utc_1971_01_01(), Ok(*utc_1971_01_01()))]
    #[case::before_leap_second(tai_1s_before_2016_leap_second(), Ok(*utc_1s_before_2016_leap_second()))]
    #[case::during_leap_second(tai_during_2016_leap_second(), Ok(*utc_during_2016_leap_second()))]
    #[case::after_leap_second(tai_1s_after_2016_leap_second(), Ok(*utc_1s_after_2016_leap_second()))]
    #[case::illegal_utc_datetime(tai_before_utc_defined(), Err(UTCUndefinedError))]
    fn test_utc_try_from_tai(
        #[case] tai: &Time<TAI>,
        #[case] expected: Result<UTCDateTime, UTCUndefinedError>,
    ) {
        let actual = UTCDateTime::try_from(*tai);
        assert_eq!(expected, actual);
    }

    /*
        The following fixtures are derived from a mixture of direct calculation and, in the case
        where inherent rounding errors prevent exact calculation, by cross-referencing with the
        observed outputs. The latter case is marked with a comment.
    */

    fn utc_1971_01_01() -> &'static UTCDateTime {
        static UTC_1971: OnceLock<UTCDateTime> = OnceLock::new();
        UTC_1971.get_or_init(|| {
            UTCDateTime::new(Date::new(1971, 1, 1).unwrap(), UTC::default()).unwrap()
        })
    }

    fn tai_at_utc_1971_01_01() -> &'static Time<TAI> {
        const DELTA: TimeDelta = TimeDelta {
            seconds: 8,
            // Note the substantial rounding error inherent in converting between single-f64 MJDs.
            // For dates prior to 1972, this algorithm achieves microsecond precision at best.
            subsecond: Subsecond(0.9461620000000011),
        };

        static TAI_AT_UTC_1971_01_01: OnceLock<Time<TAI>> = OnceLock::new();
        TAI_AT_UTC_1971_01_01.get_or_init(|| {
            let utc = utc_1971_01_01();
            let base = BaseTime::from_utc_datetime(*utc);
            Time::from_base_time(TAI, base + DELTA)
        })
    }

    // 2016-12-31T23:59:59.000 UTC
    fn utc_1s_before_2016_leap_second() -> &'static UTCDateTime {
        static BEFORE_LEAP_SECOND: OnceLock<UTCDateTime> = OnceLock::new();
        BEFORE_LEAP_SECOND.get_or_init(|| {
            UTCDateTime::new(
                Date::new(2016, 12, 31).unwrap(),
                UTC::new(23, 59, 59, Subsecond(0.0)).unwrap(),
            )
            .unwrap()
        })
    }

    // 2017-01-01T00:00:35.000 TAI
    fn tai_1s_before_2016_leap_second() -> &'static Time<TAI> {
        static BEFORE_LEAP_SECOND: OnceLock<Time<TAI>> = OnceLock::new();
        BEFORE_LEAP_SECOND.get_or_init(|| Time::new(TAI, 536500835, Subsecond::default()))
    }

    // 2016-12-31T23:59:60.000 UTC
    fn utc_during_2016_leap_second() -> &'static UTCDateTime {
        static DURING_LEAP_SECOND: OnceLock<UTCDateTime> = OnceLock::new();
        DURING_LEAP_SECOND.get_or_init(|| {
            UTCDateTime::new(
                Date::new(2016, 12, 31).unwrap(),
                UTC::new(23, 59, 60, Subsecond(0.0)).unwrap(),
            )
            .unwrap()
        })
    }

    // 2017-01-01T00:00:36.000 TAI
    fn tai_during_2016_leap_second() -> &'static Time<TAI> {
        static DURING_LEAP_SECOND: OnceLock<Time<TAI>> = OnceLock::new();
        DURING_LEAP_SECOND.get_or_init(|| Time::new(TAI, 536500836, Subsecond::default()))
    }

    // 2017-01-01T00:00:00.000 UTC
    fn utc_1s_after_2016_leap_second() -> &'static UTCDateTime {
        static AFTER_LEAP_SECOND: OnceLock<UTCDateTime> = OnceLock::new();
        AFTER_LEAP_SECOND.get_or_init(|| {
            UTCDateTime::new(
                Date::new(2017, 1, 1).unwrap(),
                UTC::new(0, 0, 0, Subsecond(0.0)).unwrap(),
            )
            .unwrap()
        })
    }

    // 2017-01-01T00:00:37.000 TAI
    fn tai_1s_after_2016_leap_second() -> &'static Time<TAI> {
        static AFTER_LEAP_SECOND: OnceLock<Time<TAI>> = OnceLock::new();
        AFTER_LEAP_SECOND.get_or_init(|| Time::new(TAI, 536500837, Subsecond::default()))
    }

    // Bypasses the UTCDateTime constructor's range check to create an illegal UTCDateTime.
    // Used for testing panics.
    fn unconstructable_utc_datetime() -> &'static UTCDateTime {
        static ILLEGAL_UTC: OnceLock<UTCDateTime> = OnceLock::new();
        ILLEGAL_UTC.get_or_init(|| UTCDateTime {
            date: Date::new(1959, 12, 31).unwrap(),
            time: UTC::default(),
        })
    }

    // 1959-12-31T23:59:59.000 TAI
    fn tai_before_utc_defined() -> &'static Time<TAI> {
        static TAI_BEFORE_UTC_DEFINED: OnceLock<Time<TAI>> = OnceLock::new();
        TAI_BEFORE_UTC_DEFINED.get_or_init(|| Time::new(TAI, -1_262_347_201, Subsecond::default()))
    }
}
