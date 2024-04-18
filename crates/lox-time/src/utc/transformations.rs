/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt::Display;
use std::sync::OnceLock;

use thiserror::Error;

use lox_eop::EarthOrientationParams;

use crate::base_time::BaseTime;
use crate::calendar_dates::Date;
use crate::deltas::{TimeDelta, TimeDeltaError};
use crate::julian_dates::Epoch;
use crate::time_scales::Tai;
use crate::utc::{UtcDateTime, UtcOld, UtcUndefinedError};
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
    UTC_1972.get_or_init(|| {
        UtcDateTime::new(Date::new(1972, 1, 1).unwrap(), UtcOld::default()).unwrap()
    })
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

/// EarthOrientationParamsError indicates that provided [EarthOrientationParams] were invalid for
/// the construction of an [EopTimeScaleTransformer].
#[derive(Clone, Debug, Error, PartialEq)]
#[error("EarthOrientationParams contain invalid data at position {position}: {details}")]
pub struct EarthOrientationParamsError {
    position: usize,
    details: EopErrorDetails,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EopErrorDetails {
    /// Arises when a [ModifiedJulianDayNumber] in [EarthOrientationParams] is before
    /// 1960-01-01 UTC.
    InvalidMjd(UtcUndefinedError),
    /// Arises when a ΔUT1-UTC value in [EarthOrientationParams] cannot be represented as a
    /// [TimeDelta].
    InvalidDeltaUt1Utc(TimeDeltaError),
}

impl Display for EopErrorDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMjd(err) => write!(f, "invalid Modified Julian Day Number: {}", err),
            Self::InvalidDeltaUt1Utc(err) => write!(f, "invalid ΔUT1-UTC value: {}", err),
        }
    }
}

impl From<UtcUndefinedError> for EopErrorDetails {
    fn from(err: UtcUndefinedError) -> Self {
        Self::InvalidMjd(err)
    }
}

impl From<TimeDeltaError> for EopErrorDetails {
    fn from(err: TimeDeltaError) -> Self {
        Self::InvalidDeltaUt1Utc(err)
    }
}

/// Transform between [TimeScale]s which require observed data in the form of
/// [EarthOrientationParams], namely conversions involving [UT1].
///
/// [EopTimeScaleTransformer] is suitable only for transformations from 1960-01-01 UTC, when UTC
/// was defined.
#[derive(Clone, Debug, PartialEq)]
pub struct EopTimeScaleTransformer<'a> {
    eop: &'a EarthOrientationParams,
    delta_ut1_tai: Vec<TimeDelta>,
}

impl<'a> EopTimeScaleTransformer<'a> {
    /// Instantiates a new [EopTimeScaleTransformer] with the provided [EarthOrientationParams].
    ///
    /// Returns [EarthOrientationParamsError] if the input data contains a [ModifiedJulianDayNumber]
    /// before 1960-01-01 UTC, or a ΔUT1-UTC value which cannot be represented as a [TimeDelta].
    /// These errors should not arise in [EarthOrientationParams] derived from valid IERS data.
    pub fn new(eop: &'a EarthOrientationParams) -> Result<Self, EarthOrientationParamsError> {
        Ok(Self {
            eop,
            delta_ut1_tai: calculate_delta_ut1_tai_from_eop(eop)?,
        })
    }
}

fn calculate_delta_ut1_tai_from_eop(
    eop: &EarthOrientationParams,
) -> Result<Vec<TimeDelta>, EarthOrientationParamsError> {
    eop.mjd()
        .iter()
        .zip(eop.delta_ut1_utc())
        .enumerate()
        .map(|(i, (mjd, delta_ut1_utc))| {
            let tai = Time::from_julian_day_number(Tai, *mjd, Epoch::ModifiedJulianDate);
            let delta_ut1_utc = TimeDelta::from_decimal_seconds(*delta_ut1_utc).map_err(|err| {
                EarthOrientationParamsError {
                    position: i,
                    details: EopErrorDetails::InvalidDeltaUt1Utc(err),
                }
            })?;
            let delta_tai_utc = delta_tai_utc(tai).map_err(|err| EarthOrientationParamsError {
                position: i,
                details: EopErrorDetails::InvalidMjd(err),
            })?;

            Ok(delta_ut1_utc - delta_tai_utc)
        })
        .collect()
}

#[cfg(test)]
pub mod test {
    use rstest::rstest;

    use crate::base_time::BaseTime;
    use crate::calendar_dates::Date;
    use crate::subsecond::Subsecond;
    use crate::utc::UtcDateTime;
    use crate::utc::UtcOld;

    use super::*;

    #[rstest]
    #[case::before_1972(utc_1971_01_01(), tai_at_utc_1971_01_01())]
    #[case::before_leap_second(utc_1s_before_2016_leap_second(), tai_1s_before_2016_leap_second())]
    #[case::during_leap_second(utc_during_2016_leap_second(), tai_during_2016_leap_second())]
    #[case::after_leap_second(utc_1s_after_2016_leap_second(), tai_1s_after_2016_leap_second())]
    #[should_panic]
    #[case::illegal_utc_datetime(unconstructable_utc_datetime(), &Time::from_seconds(Tai, 0, Subsecond::default()))]
    fn test_tai_from_utc(#[case] utc: &UtcDateTime, #[case] expected: &Time<Tai>) {
        let actual = (*utc).into();
        assert_eq!(*expected, actual);
    }

    #[rstest]
    #[case::before_utc_1972(tai_at_utc_1971_01_01(), Ok(*utc_1971_01_01()))]
    #[case::utc_1972(tai_at_utc_1972_01_01(), Ok(*utc_1972_01_01()))]
    #[case::before_leap_second(tai_1s_before_2016_leap_second(), Ok(*utc_1s_before_2016_leap_second()))]
    #[case::during_leap_second(tai_during_2016_leap_second(), Ok(*utc_during_2016_leap_second()))]
    #[case::after_leap_second(tai_1s_after_2016_leap_second(), Ok(*utc_1s_after_2016_leap_second()))]
    #[case::utc_undefined(tai_before_utc_defined(), Err(UtcUndefinedError))]
    fn test_utc_try_from_tai(
        #[case] tai: &Time<Tai>,
        #[case] expected: Result<UtcDateTime, UtcUndefinedError>,
    ) {
        let actual = UtcDateTime::try_from(*tai);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::invalid_mjd(EopErrorDetails::InvalidMjd(UtcUndefinedError), format!("invalid Modified Julian Day Number: {}", UtcUndefinedError))]
    #[case::invalid_delta_ut1_utc(EopErrorDetails::InvalidDeltaUt1Utc(any_time_delta_error()), format!("invalid ΔUT1-UTC value: {}", any_time_delta_error()))]
    fn test_eop_error_details_display(#[case] variant: EopErrorDetails, #[case] expected: String) {
        assert_eq!(expected, variant.to_string());
    }

    #[test]
    fn test_eop_error_details_from_utc_undefined_error() {
        let expected = EopErrorDetails::InvalidMjd(UtcUndefinedError);
        let actual: EopErrorDetails = UtcUndefinedError.into();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eop_error_details_from_time_delta_error() {
        let expected = EopErrorDetails::InvalidDeltaUt1Utc(any_time_delta_error());
        let actual: EopErrorDetails = any_time_delta_error().into();
        assert_eq!(expected, actual);
    }

    type EopTransformResult = Result<EopTimeScaleTransformer<'static>, EarthOrientationParamsError>;

    #[rstest]
    #[case::valid_eop(eop_1972_01_02(), Ok(EopTimeScaleTransformer {
        eop: eop_1972_01_02(),
        delta_ut1_tai: vec![TimeDelta::from_decimal_seconds(-11.1915822).unwrap()],
    }))]
    #[case::invalid_mjd(eop_with_invalid_mjd(), Err(EarthOrientationParamsError {
        position: 0,
        details: EopErrorDetails::InvalidMjd(UtcUndefinedError),
    }))]
    #[case::invalid_delta_ut1_utc(eop_with_invalid_delta_ut1_utc(), Err(EarthOrientationParamsError {
        position: 0,
        details: EopErrorDetails::InvalidDeltaUt1Utc(TimeDeltaError {
            raw: f64::NAN,
            detail: "NaN is unrepresentable".to_string(),
        }),
    }))]
    fn test_eop_time_scale_transformer_new(
        #[case] eop: &EarthOrientationParams,
        #[case] expected: EopTransformResult,
    ) {
        let actual = EopTimeScaleTransformer::new(eop);
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
            UtcDateTime::new(Date::new(1971, 1, 1).unwrap(), UtcOld::default()).unwrap()
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
                UtcOld::new(23, 59, 59, Subsecond(0.0)).unwrap(),
            )
            .unwrap()
        })
    }

    // 2017-01-01T00:00:35.000 TAI
    fn tai_1s_before_2016_leap_second() -> &'static Time<Tai> {
        static BEFORE_LEAP_SECOND: OnceLock<Time<Tai>> = OnceLock::new();
        BEFORE_LEAP_SECOND.get_or_init(|| Time::from_seconds(Tai, 536500835, Subsecond::default()))
    }

    // 2016-12-31T23:59:60.000 UTC
    fn utc_during_2016_leap_second() -> &'static UtcDateTime {
        static DURING_LEAP_SECOND: OnceLock<UtcDateTime> = OnceLock::new();
        DURING_LEAP_SECOND.get_or_init(|| {
            UtcDateTime::new(
                Date::new(2016, 12, 31).unwrap(),
                UtcOld::new(23, 59, 60, Subsecond(0.0)).unwrap(),
            )
            .unwrap()
        })
    }

    // 2017-01-01T00:00:36.000 TAI
    fn tai_during_2016_leap_second() -> &'static Time<Tai> {
        static DURING_LEAP_SECOND: OnceLock<Time<Tai>> = OnceLock::new();
        DURING_LEAP_SECOND.get_or_init(|| Time::from_seconds(Tai, 536500836, Subsecond::default()))
    }

    // 2017-01-01T00:00:00.000 UTC
    fn utc_1s_after_2016_leap_second() -> &'static UtcDateTime {
        static AFTER_LEAP_SECOND: OnceLock<UtcDateTime> = OnceLock::new();
        AFTER_LEAP_SECOND.get_or_init(|| {
            UtcDateTime::new(
                Date::new(2017, 1, 1).unwrap(),
                UtcOld::new(0, 0, 0, Subsecond(0.0)).unwrap(),
            )
            .unwrap()
        })
    }

    // 2017-01-01T00:00:37.000 TAI
    fn tai_1s_after_2016_leap_second() -> &'static Time<Tai> {
        static AFTER_LEAP_SECOND: OnceLock<Time<Tai>> = OnceLock::new();
        AFTER_LEAP_SECOND.get_or_init(|| Time::from_seconds(Tai, 536500837, Subsecond::default()))
    }

    // Bypasses the UtcDateTime constructor's range check to create an illegal UtcDateTime.
    // Used for testing panics.
    fn unconstructable_utc_datetime() -> &'static UtcDateTime {
        static ILLEGAL_UTC: OnceLock<UtcDateTime> = OnceLock::new();
        ILLEGAL_UTC.get_or_init(|| UtcDateTime {
            date: Date::new(1959, 12, 31).unwrap(),
            time: UtcOld::default(),
        })
    }

    // 1959-12-31T23:59:59.000 TAI
    fn tai_before_utc_defined() -> &'static Time<Tai> {
        static TAI_BEFORE_UTC_DEFINED: OnceLock<Time<Tai>> = OnceLock::new();
        TAI_BEFORE_UTC_DEFINED
            .get_or_init(|| Time::from_seconds(Tai, -1_262_347_201, Subsecond::default()))
    }

    fn any_time_delta_error() -> TimeDeltaError {
        TimeDeltaError {
            raw: f64::NAN,
            detail: String::default(),
        }
    }

    fn eop_1972_01_02() -> &'static EarthOrientationParams {
        static EOP_1972_01_02: OnceLock<EarthOrientationParams> = OnceLock::new();
        EOP_1972_01_02.get_or_init(|| {
            EarthOrientationParams::new(
                vec![41684],
                vec![0.120733],
                vec![0.136966],
                vec![0.8084178],
            )
            .unwrap()
        })
    }

    fn eop_with_invalid_mjd() -> &'static EarthOrientationParams {
        static EOP_INVALID_MJD: OnceLock<EarthOrientationParams> = OnceLock::new();
        EOP_INVALID_MJD.get_or_init(|| {
            EarthOrientationParams::new(vec![0], vec![0.0], vec![0.0], vec![0.0]).unwrap()
        })
    }

    fn eop_with_invalid_delta_ut1_utc() -> &'static EarthOrientationParams {
        static EOP_INVALID_DELTA_UT1_UTC: OnceLock<EarthOrientationParams> = OnceLock::new();
        EOP_INVALID_DELTA_UT1_UTC.get_or_init(|| {
            EarthOrientationParams::new(vec![41684], vec![0.120733], vec![0.136966], vec![f64::NAN])
                .unwrap()
        })
    }
}
