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
use crate::time_scales::TAI;
use crate::utc::{UTCDateTime, UTCUndefinedError, UTC};
use crate::Time;

mod before1972;
mod from1972;

impl From<UTCDateTime> for Time<TAI> {
    /// Converts a `UTCDateTime` to `TAI`, accounting for leap seconds. Infallible for all valid
    /// values of UTC.
    fn from(utc: UTCDateTime) -> Self {
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
        Time::from_base_time(TAI, base - delta)
    }
}

impl TryFrom<Time<TAI>> for UTCDateTime {
    type Error = UTCUndefinedError;

    /// Attempts to convert a `Time<TAI>` to a `UTCDateTime`, accounting for leap seconds. Returns
    /// [UTCUndefinedError] if the input `Time<TAI>` is before 1960-01-01 UTC, when UTC begins.
    fn try_from(tai: Time<TAI>) -> Result<Self, Self::Error> {
        let delta = if tai < *tai_at_utc_1972_01_01() {
            before1972::delta_tai_utc(tai)
        } else {
            from1972::delta_tai_utc(tai)
        }
        .ok_or(UTCUndefinedError)?;

        let base_time = tai.base_time() - delta;
        let mut utc = UTCDateTime::from_base_time(base_time)?;
        if tai.is_leap_second() {
            utc.time.second = 60;
        }

        Ok(utc)
    }
}

fn utc_1972_01_01() -> &'static UTCDateTime {
    static UTC_1972: OnceLock<UTCDateTime> = OnceLock::new();
    UTC_1972
        .get_or_init(|| UTCDateTime::new(Date::new(1972, 1, 1).unwrap(), UTC::default()).unwrap())
}

fn tai_at_utc_1972_01_01() -> &'static Time<TAI> {
    const LEAP_SECONDS_1972: i64 = 10;
    static TAI_AT_UTC_1972_01_01: OnceLock<Time<TAI>> = OnceLock::new();
    TAI_AT_UTC_1972_01_01.get_or_init(|| {
        let utc = utc_1972_01_01();
        let base_time = BaseTime::from_utc_datetime(*utc);
        let leap_seconds = TimeDelta::from_seconds(LEAP_SECONDS_1972);
        Time::from_base_time(TAI, base_time + leap_seconds)
    })
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
    #[case::before_utc_1972(tai_at_utc_1971_01_01(), Ok(*utc_1971_01_01()))]
    #[case::utc_1972(tai_at_utc_1972_01_01(), Ok(*utc_1972_01_01()))]
    #[case::before_leap_second(tai_1s_before_2016_leap_second(), Ok(*utc_1s_before_2016_leap_second()))]
    #[case::during_leap_second(tai_during_2016_leap_second(), Ok(*utc_during_2016_leap_second()))]
    #[case::after_leap_second(tai_1s_after_2016_leap_second(), Ok(*utc_1s_after_2016_leap_second()))]
    #[case::utc_undefined(tai_before_utc_defined(), Err(UTCUndefinedError))]
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
