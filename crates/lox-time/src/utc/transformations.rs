/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::sync::OnceLock;

use crate::base_time::BaseTime;
use crate::calendar_dates::Date;
use crate::deltas::TimeDelta;
use crate::time_scales::Tai;
use crate::utc::{Utc, UtcDateTime, UtcUndefinedError};
use crate::Time;

mod before1972;
mod from1972;

impl From<UtcDateTime> for Time<Tai> {
    /// Converts a `UTCDateTime` to `TAI`, accounting for leap seconds. Infallible for all valid
    /// values of UTC.
    fn from(utc: UtcDateTime) -> Self {
        let delta = if utc < *utc_1972_01_01() {
            before1972::delta_utc_tai(utc)
        } else {
            from1972::delta_utc_tai(utc)
        }
        .unwrap_or_else(|| {
            // UTCDateTime objects are always in range.
            unreachable!(
                "failed to calculate UTC-TAI delta for UTCDateTime `{:?}`",
                utc
            );
        });

        let base = BaseTime::from_utc_datetime(utc);
        Time::from_base_time(Tai, base - delta)
    }
}

impl TryFrom<Time<Tai>> for UtcDateTime {
    type Error = UtcUndefinedError;

    /// Attempts to convert a `Time<TAI>` to a `UTCDateTime`, accounting for leap seconds. Returns
    /// [UtcUndefinedError] if the input `Time<TAI>` is before 1960-01-01 UTC, when UTC begins.
    fn try_from(tai: Time<Tai>) -> Result<Self, Self::Error> {
        let delta = if tai < *tai_at_utc_1972_01_01() {
            before1972::delta_tai_utc(tai)
        } else {
            from1972::delta_tai_utc(tai)
        }
        .ok_or(UtcUndefinedError)?;

        let base_time = tai.base_time() - delta;
        let mut utc = UtcDateTime::from_base_time(base_time)?;
        if tai.is_leap_second() {
            utc.time.second = 60;
        }

        Ok(utc)
    }
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

#[cfg(test)]
pub mod test {
    use rstest::rstest;

    use crate::base_time::BaseTime;
    use crate::calendar_dates::Date;
    use crate::subsecond::Subsecond;
    use crate::utc::Utc;
    use crate::utc::UtcDateTime;

    use super::*;

    #[rstest]
    #[case::before_1972(utc_1971_01_01(), tai_at_utc_1971_01_01())]
    #[case::before_leap_second(utc_1s_before_2016_leap_second(), tai_1s_before_2016_leap_second())]
    #[case::during_leap_second(utc_during_2016_leap_second(), tai_during_2016_leap_second())]
    #[case::after_leap_second(utc_1s_after_2016_leap_second(), tai_1s_after_2016_leap_second())]
    #[should_panic]
    #[case::illegal_utc_datetime(unconstructable_utc_datetime(), &Time::new(TAI, 0, Subsecond::default()))]
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
    #[case::utc_undefined(tai_before_utc_defined(), Err(UTCUndefinedError))]
    fn test_utc_try_from_tai(
        #[case] tai: &Time<Tai>,
        #[case] expected: Result<UtcDateTime, UtcUndefinedError>,
    ) {
        let actual = UtcDateTime::try_from(*tai);
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

    // Bypasses the UTCDateTime constructor's range check to create an illegal UTCDateTime.
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
}
