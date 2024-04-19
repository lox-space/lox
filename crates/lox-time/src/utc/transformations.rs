/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::error::Error;
use std::fmt::Display;
use std::sync::OnceLock;

use thiserror::Error;

use lox_eop::{DeltaUt1Utc, TargetDateError};

use crate::base_time::BaseTime;
use crate::calendar_dates::Date;
use crate::deltas::{TimeDelta, TimeDeltaError};
use crate::julian_dates::Epoch::ModifiedJulianDate;
use crate::julian_dates::JulianDate;
use crate::julian_dates::Unit::Days;
use crate::time_scales::{Tai, TimeScale, Ut1};
use crate::transformations::TransformTimeScale;
use crate::utc::{Utc, UtcDateTime, UtcUndefinedError};
use crate::Time;

mod before1972;
mod from1972;

impl From<UtcDateTime> for Time<Tai> {
    /// Converts a `UtcDateTime` to `TAI`, accounting for leap seconds. Infallible for all valid
    /// values of UTC.
    fn from(utc: UtcDateTime) -> Self {
        let delta = delta_utc_tai(utc);
        let base = BaseTime::from_utc_datetime(utc);
        Time::from_base_time(Tai, base - delta)
    }
}

fn delta_utc_tai(utc: UtcDateTime) -> TimeDelta {
    if utc < *utc_1972_01_01() {
        before1972::delta_utc_tai(utc)
    } else {
        from1972::delta_utc_tai(utc)
    }
    .unwrap_or_else(|| {
        // UtcDateTime objects are always in range.
        unreachable!(
            "failed to calculate UTC-TAI delta for UtcDateTime `{:?}`",
            utc
        );
    })
}

impl TryFrom<Time<Tai>> for UtcDateTime {
    type Error = UtcUndefinedError;

    /// Attempts to convert a `Time<TAI>` to a `UtcDateTime`, accounting for leap seconds. Returns
    /// [UtcUndefinedError] if the input `Time<TAI>` is before 1960-01-01 UTC, when UTC begins.
    fn try_from(tai: Time<Tai>) -> Result<Self, Self::Error> {
        let delta = delta_tai_utc(tai)?;
        let base_time = tai.base_time() - delta;
        let mut utc = UtcDateTime::from_base_time(base_time)?;
        if tai.is_leap_second() {
            utc.time.second = 60;
        }

        Ok(utc)
    }
}

fn delta_tai_utc(tai: Time<Tai>) -> Result<TimeDelta, UtcUndefinedError> {
    if tai < *tai_at_utc_1972_01_01() {
        before1972::delta_tai_utc(tai)
    } else {
        from1972::delta_tai_utc(tai)
    }
    .ok_or(UtcUndefinedError)
}

fn utc_1972_01_01() -> &'static UtcDateTime {
    static UTC_1972: OnceLock<UtcDateTime> = OnceLock::new();
    UTC_1972
        .get_or_init(|| UtcDateTime::new(Date::new(1972, 1, 1).unwrap(), Utc::default()).unwrap())
}

fn tai_at_utc_1972_01_01() -> &'static Time<Tai> {
    const LEAP_SECONDS_1972: i64 = 10;
    static TAI_AT_UTC_1972_01_01: OnceLock<Time<Tai>> = OnceLock::new();
    TAI_AT_UTC_1972_01_01.get_or_init(|| {
        let utc = utc_1972_01_01();
        let base_time = BaseTime::from_utc_datetime(*utc);
        let leap_seconds = TimeDelta::from_seconds(LEAP_SECONDS_1972);
        Time::from_base_time(Tai, base_time + leap_seconds)
    })
}

pub trait TryTransformTimeScale<T, U>
where
    T: TimeScale + Copy,
    U: TimeScale + Copy,
{
    type Error: Error;

    fn try_transform(&self, time: Time<T>) -> Result<Time<U>, Self::Error>;
}

/// Transform between [TimeScale]s which require observed data, namely the delta between UT1 and
/// UTC.
///
/// [ObservedDataTimeScaleTransformer] is suitable only for transformations from 1960-01-01 UTC,
/// when UTC was defined.
#[derive(Clone, Debug, PartialEq)]
pub struct ObservedDataTimeScaleTransformer<D: DeltaUt1Utc> {
    d_ut1_utc_provider: D,
}

impl<D: DeltaUt1Utc> ObservedDataTimeScaleTransformer<D> {
    pub fn new(d_ut1_utc_provider: D) -> Self {
        Self { d_ut1_utc_provider }
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum TransformationError {
    #[error(transparent)]
    TargetDateError(#[from] TargetDateError),
    #[error(transparent)]
    TimeDeltaError(#[from] TimeDeltaError),
}

impl<D: DeltaUt1Utc> TryTransformTimeScale<Tai, Ut1> for &ObservedDataTimeScaleTransformer<D> {
    type Error = TransformationError;

    fn try_transform(&self, time: Time<Tai>) -> Result<Time<Ut1>, Self::Error> {
        let mjd = time.julian_date(ModifiedJulianDate, Days);
        let d_ut1_utc = self.d_ut1_utc_provider.delta_ut1_utc(mjd)?;
        let d_ut1_utc = TimeDelta::from_decimal_seconds(d_ut1_utc)?;
        let d_tai_utc = delta_tai_utc(time).unwrap_or_else(|err| {
            // If the date is in range for `d_ut1_utc`, it will be in range for `d_tai_utc`.
            unreachable!(
                "failed to calculate TAI-UTC delta for Time<TAI> `{:?}`: {}",
                time, err
            )
        });
        let delta = d_ut1_utc - d_tai_utc;
        let ut1 = Time::from_base_time(Ut1, time.base_time() + delta);
        Ok(ut1)
    }
}

#[cfg(test)]
pub mod test {
    use std::path::Path;

    use lox_eop::EarthOrientationParams;
    use rstest::rstest;

    use lox_eop::iers::parse_finals_csv;

    use crate::base_time::BaseTime;
    use crate::calendar_dates::Date;
    use crate::julian_dates::Epoch;
    use crate::subsecond::Subsecond;
    use crate::utc::Utc;
    use crate::utc::UtcDateTime;

    use super::*;

    // #[rstest]
    // #[case::before_1972(utc_1971_01_01(), tai_at_utc_1971_01_01())]
    // #[case::before_leap_second(utc_1s_before_2016_leap_second(), tai_1s_before_2016_leap_second())]
    // #[case::during_leap_second(utc_during_2016_leap_second(), tai_during_2016_leap_second())]
    // #[case::after_leap_second(utc_1s_after_2016_leap_second(), tai_1s_after_2016_leap_second())]
    // #[should_panic]
    // #[case::illegal_utc_datetime(unconstructable_utc_datetime(), &Time::new(Tai, 0, Subsecond::default()))]
    // fn test_tai_from_utc(#[case] utc: &UtcDateTime, #[case] expected: &Time<Tai>) {
    //     let actual = (*utc).into();
    //     assert_eq!(*expected, actual);
    // }
    //
    // #[rstest]
    // #[case::before_utc_1972(tai_at_utc_1971_01_01(), Ok(*utc_1971_01_01()))]
    // #[case::utc_1972(tai_at_utc_1972_01_01(), Ok(*utc_1972_01_01()))]
    // #[case::before_leap_second(tai_1s_before_2016_leap_second(), Ok(*utc_1s_before_2016_leap_second()))]
    // #[case::during_leap_second(tai_during_2016_leap_second(), Ok(*utc_during_2016_leap_second()))]
    // #[case::after_leap_second(tai_1s_after_2016_leap_second(), Ok(*utc_1s_after_2016_leap_second()))]
    // #[case::utc_undefined(tai_before_utc_defined(), Err(UtcUndefinedError))]
    // fn test_utc_try_from_tai(
    //     #[case] tai: &Time<Tai>,
    //     #[case] expected: Result<UtcDateTime, UtcUndefinedError>,
    // ) {
    //     let actual = UtcDateTime::try_from(*tai);
    //     assert_eq!(expected, actual);
    // }

    #[rstest]
    #[case::j2000_tai(
        Time::j2000(Tai),
        Time::from_base_time(Ut1, BaseTime {
            seconds: -32,
            subsecond: Subsecond(1.0 - 0.644974644349812),
        })
    )]
    #[case::mjd_41684(
        Time::from_julian_day_number(
            Tai,
            41684,
            ModifiedJulianDate
        ),
        Time::from_base_time(Ut1, BaseTime {
            seconds: -851947212,
            subsecond: Subsecond(0.8084178),
        })
    )]
    #[case::mjd_60791(
        Time::from_julian_day_number(
            Tai,
            60791,
            ModifiedJulianDate
        ),
        Time::from_base_time(Ut1, BaseTime {
            seconds: 798897563,
            subsecond: Subsecond(0.0017368),
        })
    )]
    fn test_eop_transform_time_scale_tai_ut1_success(
        #[case] tai: Time<Tai>,
        #[case] expected: Time<Ut1>,
    ) {
        let transformer = observed_data_time_scale_transformer();
        let actual = transformer.try_transform(tai).unwrap_or_else(|err| {
            panic!("expected success, but got {:?}", err);
        });
        assert_eq!(expected, actual);
    }

    /*
        The following fixtures are derived from a mixture of direct calculation and, in the case
        where inherent rounding errors prevent exact calculation, by cross-referencing with the
        observed outputs. The latter case is marked with a comment.
    */

    fn utc_1971_01_01() -> &'static UtcDateTime {
        static UTC_1971: OnceLock<UtcDateTime> = OnceLock::new();
        UTC_1971.get_or_init(|| {
            UtcDateTime::new(Date::new(1971, 1, 1).unwrap(), Utc::default()).unwrap()
        })
    }

    fn tai_at_utc_1971_01_01() -> &'static Time<Tai> {
        const DELTA: TimeDelta = TimeDelta {
            seconds: 8,
            // Note the substantial rounding error inherent in converting between single-f64 MJDs.
            // For dates prior to 1972, this algorithm achieves microsecond precision at best.
            subsecond: Subsecond(0.9461620000000011),
        };

        static TAI_AT_UTC_1971_01_01: OnceLock<Time<Tai>> = OnceLock::new();
        TAI_AT_UTC_1971_01_01.get_or_init(|| {
            let utc = utc_1971_01_01();
            let base = BaseTime::from_utc_datetime(*utc);
            Time::from_base_time(Tai, base + DELTA)
        })
    }

    // 2016-12-31T23:59:59.000 UTC
    fn utc_1s_before_2016_leap_second() -> &'static UtcDateTime {
        static BEFORE_LEAP_SECOND: OnceLock<UtcDateTime> = OnceLock::new();
        BEFORE_LEAP_SECOND.get_or_init(|| {
            UtcDateTime::new(
                Date::new(2016, 12, 31).unwrap(),
                Utc::new(23, 59, 59, Subsecond(0.0)).unwrap(),
            )
            .unwrap()
        })
    }

    // 2017-01-01T00:00:35.000 TAI
    fn tai_1s_before_2016_leap_second() -> &'static Time<Tai> {
        static BEFORE_LEAP_SECOND: OnceLock<Time<Tai>> = OnceLock::new();
        BEFORE_LEAP_SECOND.get_or_init(|| Time::new(Tai, 536500835, Subsecond::default()))
    }

    // 2016-12-31T23:59:60.000 UTC
    fn utc_during_2016_leap_second() -> &'static UtcDateTime {
        static DURING_LEAP_SECOND: OnceLock<UtcDateTime> = OnceLock::new();
        DURING_LEAP_SECOND.get_or_init(|| {
            UtcDateTime::new(
                Date::new(2016, 12, 31).unwrap(),
                Utc::new(23, 59, 60, Subsecond(0.0)).unwrap(),
            )
            .unwrap()
        })
    }

    // 2017-01-01T00:00:36.000 TAI
    fn tai_during_2016_leap_second() -> &'static Time<Tai> {
        static DURING_LEAP_SECOND: OnceLock<Time<Tai>> = OnceLock::new();
        DURING_LEAP_SECOND.get_or_init(|| Time::new(Tai, 536500836, Subsecond::default()))
    }

    // 2017-01-01T00:00:00.000 UTC
    fn utc_1s_after_2016_leap_second() -> &'static UtcDateTime {
        static AFTER_LEAP_SECOND: OnceLock<UtcDateTime> = OnceLock::new();
        AFTER_LEAP_SECOND.get_or_init(|| {
            UtcDateTime::new(
                Date::new(2017, 1, 1).unwrap(),
                Utc::new(0, 0, 0, Subsecond(0.0)).unwrap(),
            )
            .unwrap()
        })
    }

    // 2017-01-01T00:00:37.000 TAI
    fn tai_1s_after_2016_leap_second() -> &'static Time<Tai> {
        static AFTER_LEAP_SECOND: OnceLock<Time<Tai>> = OnceLock::new();
        AFTER_LEAP_SECOND.get_or_init(|| Time::new(Tai, 536500837, Subsecond::default()))
    }

    // Bypasses the UtcDateTime constructor's range check to create an illegal UtcDateTime.
    // Used for testing panics.
    fn unconstructable_utc_datetime() -> &'static UtcDateTime {
        static ILLEGAL_UTC: OnceLock<UtcDateTime> = OnceLock::new();
        ILLEGAL_UTC.get_or_init(|| UtcDateTime {
            date: Date::new(1959, 12, 31).unwrap(),
            time: Utc::default(),
        })
    }

    // 1959-12-31T23:59:59.000 TAI
    fn tai_before_utc_defined() -> &'static Time<Tai> {
        static TAI_BEFORE_UTC_DEFINED: OnceLock<Time<Tai>> = OnceLock::new();
        TAI_BEFORE_UTC_DEFINED.get_or_init(|| Time::new(Tai, -1_262_347_201, Subsecond::default()))
    }

    fn any_time_delta_error() -> TimeDeltaError {
        TimeDeltaError {
            raw: f64::NAN,
            detail: String::default(),
        }
    }

    const TEST_DATA_DIR: &str = "../../data";

    fn observed_data_time_scale_transformer(
    ) -> &'static ObservedDataTimeScaleTransformer<&'static EarthOrientationParams> {
        static EOP: OnceLock<EarthOrientationParams> = OnceLock::new();
        static TRANSFORMER: OnceLock<
            ObservedDataTimeScaleTransformer<&'static EarthOrientationParams>,
        > = OnceLock::new();

        let eop = EOP.get_or_init(|| {
            parse_finals_csv(Path::new(TEST_DATA_DIR).join("finals.all.2024-04-19.csv")).unwrap()
        });
        TRANSFORMER.get_or_init(|| ObservedDataTimeScaleTransformer::new(eop))
    }
}
