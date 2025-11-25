// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::sync::OnceLock;

use crate::deltas::TimeDelta;
use crate::deltas::ToDelta;
use crate::offsets::DefaultOffsetProvider;
use crate::offsets::TryOffset;
use crate::time::DynTime;
use crate::time_of_day::CivilTime;
use crate::time_of_day::TimeOfDay;
use crate::time_scales::TimeScale;
use crate::time_scales::{DynTimeScale, Tai};
use crate::{time::Time, utc};

use super::LeapSecondsProvider;
use super::leap_seconds::DefaultLeapSecondsProvider;
use super::{Utc, UtcError};

mod before1972;

impl Utc {
    pub fn offset_tai(&self, provider: &impl LeapSecondsProvider) -> TimeDelta {
        if self < utc_1972_01_01() {
            before1972::delta_utc_tai(self)
        } else {
            provider.delta_utc_tai(*self)
        }
        .expect("Utc objects should always be in range")
    }

    pub fn to_time_with_provider(&self, provider: &impl LeapSecondsProvider) -> Time<Tai> {
        let offset = self.offset_tai(provider);
        Time::from_delta(Tai, self.to_delta() - offset)
    }

    pub fn to_time(&self) -> Time<Tai> {
        self.to_time_with_provider(&DefaultLeapSecondsProvider)
    }

    pub fn to_dyn_time_with_provider(&self, provider: &impl LeapSecondsProvider) -> DynTime {
        let offset = self.offset_tai(provider);
        Time::from_delta(DynTimeScale::Tai, self.to_delta() - offset)
    }

    pub fn to_dyn_time(&self) -> DynTime {
        self.to_dyn_time_with_provider(&DefaultLeapSecondsProvider)
    }
}

pub trait TryToUtc {
    fn try_to_utc_with_provider(
        &self,
        provider: &impl LeapSecondsProvider,
    ) -> Result<Utc, UtcError>;

    fn try_to_utc(&self) -> Result<Utc, UtcError> {
        self.try_to_utc_with_provider(&DefaultLeapSecondsProvider)
    }
}

impl TryToUtc for Utc {
    fn try_to_utc_with_provider(
        &self,
        _provider: &impl LeapSecondsProvider,
    ) -> Result<Utc, UtcError> {
        Ok(*self)
    }
}

impl<T> TryToUtc for Time<T>
where
    T: TimeScale + Copy,
    DefaultOffsetProvider: TryOffset<T, Tai>,
{
    fn try_to_utc_with_provider(
        &self,
        provider: &impl LeapSecondsProvider,
    ) -> Result<Utc, UtcError> {
        let tai = self
            .try_to_scale(Tai, &DefaultOffsetProvider)
            .map_err(|err| UtcError::Ut1(err.to_string()))?;
        let delta = if &tai < tai_at_utc_1972_01_01() {
            before1972::delta_tai_utc(&tai)
        } else {
            provider.delta_tai_utc(tai)
        }
        .ok_or(UtcError::UtcUndefined)?;
        let mut utc = Utc::from_delta(tai.to_delta() - delta)?;
        if provider.is_leap_second(tai) {
            utc.time = TimeOfDay::new(utc.hour(), utc.minute(), 60)
                .unwrap()
                .with_subsecond(utc.time.subsecond());
        }
        Ok(utc)
    }
}

fn utc_1972_01_01() -> &'static Utc {
    static UTC_1972: OnceLock<Utc> = OnceLock::new();
    UTC_1972.get_or_init(|| utc!(1972, 1, 1).unwrap())
}

fn tai_at_utc_1972_01_01() -> &'static Time<Tai> {
    const LEAP_SECONDS_1972: i64 = 10;
    static TAI_AT_UTC_1972_01_01: OnceLock<Time<Tai>> = OnceLock::new();
    TAI_AT_UTC_1972_01_01.get_or_init(|| {
        let utc = utc_1972_01_01();
        let base_time = utc.to_delta();
        let leap_seconds = TimeDelta::from_seconds(LEAP_SECONDS_1972);
        Time::from_delta(Tai, base_time + leap_seconds)
    })
}

#[cfg(test)]
mod test {
    use crate::subsecond::Subsecond;
    use crate::time;
    use crate::time_scales::{Tcb, Tcg, Tdb, Tt};
    use rstest::rstest;

    use super::*;

    #[test]
    fn test_utc_to_utc() {
        let utc0 = utc!(2000, 1, 1).unwrap();
        let utc1 = utc0.try_to_utc().unwrap();
        assert_eq!(utc0, utc1);
    }

    #[rstest]
    #[case::before_1972(utc_1971_01_01(), tai_at_utc_1971_01_01())]
    #[case::before_leap_second(utc_1s_before_2016_leap_second(), tai_1s_before_2016_leap_second())]
    #[case::during_leap_second(utc_during_2016_leap_second(), tai_during_2016_leap_second())]
    #[case::after_leap_second(utc_1s_after_2016_leap_second(), tai_1s_after_2016_leap_second())]
    #[should_panic]
    #[case::illegal_utc_datetime(unconstructable_utc_datetime(), &Time::new(Tai, 0, Subsecond::default()))]
    fn test_utc_to_tai(#[case] utc: &Utc, #[case] expected: &Time<Tai>) {
        let actual = utc.to_time();
        assert_eq!(*expected, actual);
    }

    #[rstest]
    #[case::before_utc_1972(tai_at_utc_1971_01_01(), Ok(*utc_1971_01_01()))]
    #[case::utc_1972(tai_at_utc_1972_01_01(), Ok(*utc_1972_01_01()))]
    #[case::before_leap_second(tai_1s_before_2016_leap_second(), Ok(*utc_1s_before_2016_leap_second()))]
    #[case::during_leap_second(tai_during_2016_leap_second(), Ok(*utc_during_2016_leap_second()))]
    #[case::after_leap_second(tai_1s_after_2016_leap_second(), Ok(*utc_1s_after_2016_leap_second()))]
    #[case::utc_undefined(tai_before_utc_defined(), Err(UtcError::UtcUndefined))]
    fn test_tai_to_utc(#[case] tai: &Time<Tai>, #[case] expected: Result<Utc, UtcError>) {
        let actual = tai.try_to_utc();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_all_scales_to_utc() {
        use lox_test_utils::assert_approx_eq;

        let tai = time!(Tai, 2024, 5, 17, 12, 13, 14.0).unwrap();
        let exp = tai.try_to_utc().unwrap();
        let tt = tai.try_to_scale(Tt, &DefaultOffsetProvider).unwrap();
        let act = tt.try_to_utc().unwrap();
        assert_eq!(act, exp);
        let tcg = tai.try_to_scale(Tcg, &DefaultOffsetProvider).unwrap();
        let act = tcg.try_to_utc().unwrap();
        assert_eq!(act, exp);
        // TCB conversions have lower precision due to the multi-step transformation
        let tcb = tai.try_to_scale(Tcb, &DefaultOffsetProvider).unwrap();
        let act = tcb.try_to_utc().unwrap();
        assert_approx_eq!(act, exp);
        let tdb = tai.try_to_scale(Tdb, &DefaultOffsetProvider).unwrap();
        let act = tdb.try_to_utc().unwrap();
        assert_eq!(act, exp);
    }

    /*
        The following fixtures are derived from a mixture of direct calculation and, in the case
        where inherent rounding errors prevent exact calculation, by cross-referencing with the
        observed outputs. The latter case is marked with a comment.
    */

    fn utc_1971_01_01() -> &'static Utc {
        static UTC_1971: OnceLock<Utc> = OnceLock::new();
        UTC_1971.get_or_init(|| utc!(1971, 1, 1).unwrap())
    }

    fn tai_at_utc_1971_01_01() -> &'static Time<Tai> {
        // const DELTA: TimeDelta = TimeDelta::builder()
        //     .seconds(8)
        //     .milliseconds(946)
        //     .microseconds(162)
        //     .build();
        const DELTA: TimeDelta = TimeDelta::from_seconds_and_subsecond_f64(8.0, 0.9461620000000011);

        static TAI_AT_UTC_1971_01_01: OnceLock<Time<Tai>> = OnceLock::new();
        TAI_AT_UTC_1971_01_01.get_or_init(|| {
            let utc = utc_1971_01_01();
            let base = utc.to_delta();
            Time::from_delta(Tai, base + DELTA)
        })
    }

    // 2016-12-31T23:59:59.000 UTC
    fn utc_1s_before_2016_leap_second() -> &'static Utc {
        static BEFORE_LEAP_SECOND: OnceLock<Utc> = OnceLock::new();
        BEFORE_LEAP_SECOND.get_or_init(|| utc!(2016, 12, 31, 23, 59, 59.0).unwrap())
    }

    // 2017-01-01T00:00:35.000 TAI
    fn tai_1s_before_2016_leap_second() -> &'static Time<Tai> {
        static BEFORE_LEAP_SECOND: OnceLock<Time<Tai>> = OnceLock::new();
        BEFORE_LEAP_SECOND.get_or_init(|| Time::new(Tai, 536500835, Subsecond::default()))
    }

    // 2016-12-31T23:59:60.000 UTC
    fn utc_during_2016_leap_second() -> &'static Utc {
        static DURING_LEAP_SECOND: OnceLock<Utc> = OnceLock::new();
        DURING_LEAP_SECOND.get_or_init(|| utc!(2016, 12, 31, 23, 59, 60.0).unwrap())
    }

    // 2017-01-01T00:00:36.000 TAI
    fn tai_during_2016_leap_second() -> &'static Time<Tai> {
        static DURING_LEAP_SECOND: OnceLock<Time<Tai>> = OnceLock::new();
        DURING_LEAP_SECOND.get_or_init(|| Time::new(Tai, 536500836, Subsecond::default()))
    }

    // 2017-01-01T00:00:00.000 UTC
    fn utc_1s_after_2016_leap_second() -> &'static Utc {
        static AFTER_LEAP_SECOND: OnceLock<Utc> = OnceLock::new();
        AFTER_LEAP_SECOND.get_or_init(|| utc!(2017, 1, 1).unwrap())
    }

    // 2017-01-01T00:00:37.000 TAI
    fn tai_1s_after_2016_leap_second() -> &'static Time<Tai> {
        static AFTER_LEAP_SECOND: OnceLock<Time<Tai>> = OnceLock::new();
        AFTER_LEAP_SECOND.get_or_init(|| Time::new(Tai, 536500837, Subsecond::default()))
    }

    // Bypasses the Utc constructor's range check to create an illegal Utc.
    // Used for testing panics.
    fn unconstructable_utc_datetime() -> &'static Utc {
        static ILLEGAL_UTC: OnceLock<Utc> = OnceLock::new();
        ILLEGAL_UTC.get_or_init(|| utc!(1959, 12, 31).unwrap())
    }

    // 1959-12-31T23:59:59.000 TAI
    fn tai_before_utc_defined() -> &'static Time<Tai> {
        static TAI_BEFORE_UTC_DEFINED: OnceLock<Time<Tai>> = OnceLock::new();
        TAI_BEFORE_UTC_DEFINED.get_or_init(|| Time::new(Tai, -1_262_347_201, Subsecond::default()))
    }
}
