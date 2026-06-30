// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::deltas::TimeDelta;
use crate::deltas::ToDelta;
use crate::offsets::DefaultOffsetProvider;
use crate::offsets::Offset;
use crate::time::DynTime;
use crate::time::Time;
use crate::time_of_day::CivilTime;
use crate::time_of_day::TimeOfDay;
use crate::time_scales::TimeScale;
use crate::time_scales::{DynTimeScale, Tai};
use lox_core::i64::consts::{SECONDS_PER_DAY, SECONDS_PER_HALF_DAY};
use lox_core::time::calendar_dates::Date;

use super::LeapSecondsProvider;
use super::Utc;
use super::leap_seconds::DefaultLeapSecondsProvider;

mod before1972;

impl Utc {
    /// Returns the TAI−UTC offset at this UTC instant.
    pub fn offset_tai(&self, provider: &impl LeapSecondsProvider) -> TimeDelta {
        if self < &UTC_1972_01_01 {
            before1972::delta_utc_tai(self)
        } else {
            provider.delta_utc_tai(*self)
        }
    }

    /// Converts this UTC instant to TAI using the given leap-seconds provider.
    pub fn to_time_with_provider(&self, provider: &impl LeapSecondsProvider) -> Time<Tai> {
        let offset = self.offset_tai(provider);
        Time::from_delta(Tai, self.to_delta() - offset)
    }

    /// Converts this UTC instant to TAI using the built-in leap-seconds table.
    pub fn to_time(&self) -> Time<Tai> {
        self.to_time_with_provider(&DefaultLeapSecondsProvider)
    }

    /// Converts this UTC instant to a [`DynTime`] in TAI using the given provider.
    pub fn to_dyn_time_with_provider(&self, provider: &impl LeapSecondsProvider) -> DynTime {
        let offset = self.offset_tai(provider);
        Time::from_delta(DynTimeScale::Tai, self.to_delta() - offset)
    }

    /// Converts this UTC instant to a [`DynTime`] in TAI using the built-in table.
    pub fn to_dyn_time(&self) -> DynTime {
        self.to_dyn_time_with_provider(&DefaultLeapSecondsProvider)
    }
}

/// Trait for types that can be converted to [`Utc`].
pub trait ToUtc {
    /// Converts to UTC using the given leap-seconds provider.
    fn to_utc_with_provider(&self, provider: &impl LeapSecondsProvider) -> Utc;

    /// Converts to UTC using the built-in leap-seconds table.
    fn to_utc(&self) -> Utc {
        self.to_utc_with_provider(&DefaultLeapSecondsProvider)
    }
}

impl ToUtc for Utc {
    fn to_utc_with_provider(&self, _provider: &impl LeapSecondsProvider) -> Utc {
        *self
    }
}

impl<T> ToUtc for Time<T>
where
    T: TimeScale + Copy,
    DefaultOffsetProvider: Offset<T, Tai>,
{
    fn to_utc_with_provider(&self, provider: &impl LeapSecondsProvider) -> Utc {
        let tai = self.to_scale(Tai);
        let delta = if tai < TAI_AT_UTC_1972_01_01 {
            before1972::delta_tai_utc(&tai)
        } else {
            provider.delta_tai_utc(tai)
        };
        let mut utc = Utc::from_delta(tai.to_delta() - delta);
        if provider.is_leap_second(tai) {
            utc.time = TimeOfDay::new(utc.hour(), utc.minute(), 60)
                .unwrap()
                .with_subsecond(utc.time.subsecond());
        }
        utc
    }
}

/// TAI−UTC at midnight on 1972-01-01.
const LEAP_SECONDS_1972: i64 = 10;

/// 1972-01-01T00:00:00 UTC, the start of the modern leap-second era.
const UTC_1972_01_01: Utc =
    Utc::new_unchecked(Date::new_unchecked(1972, 1, 1), TimeOfDay::MIDNIGHT);

/// TAI at UTC 1972-01-01T00:00:00 = J2000 - 28 years - 10 leap seconds.
/// Computed at compile time to avoid the OnceLock cache.
const TAI_AT_UTC_1972_01_01: Time<Tai> = {
    let seconds =
        Date::new_unchecked(1972, 1, 1).j2000_day_number() * SECONDS_PER_DAY - SECONDS_PER_HALF_DAY;
    let base = TimeDelta::from_seconds(seconds);
    let leap = TimeDelta::from_seconds(LEAP_SECONDS_1972);
    Time::from_delta(Tai, base.add_const(leap))
};

#[cfg(test)]
mod test {
    use crate::subsecond::Subsecond;
    use crate::time;
    use crate::time_of_day::TimeOfDay;
    use crate::time_scales::{Tcb, Tcg, Tdb, Tt};
    use crate::utc;
    use rstest::rstest;

    use super::*;

    #[test]
    fn test_utc_to_utc() {
        let utc0 = utc!(2000, 1, 1).unwrap();
        let utc1 = utc0.to_utc();
        assert_eq!(utc0, utc1);
    }

    #[rstest]
    #[case::before_1972(UTC_1971_01_01, TAI_AT_UTC_1971_01_01)]
    #[case::before_leap_second(UTC_1S_BEFORE_2016_LEAP_SECOND, TAI_1S_BEFORE_2016_LEAP_SECOND)]
    #[case::during_leap_second(UTC_DURING_2016_LEAP_SECOND, TAI_DURING_2016_LEAP_SECOND)]
    #[case::after_leap_second(UTC_1S_AFTER_2016_LEAP_SECOND, TAI_1S_AFTER_2016_LEAP_SECOND)]
    fn test_utc_to_tai(#[case] utc: Utc, #[case] expected: Time<Tai>) {
        let actual = utc.to_time();
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::before_utc_1972(TAI_AT_UTC_1971_01_01, UTC_1971_01_01)]
    #[case::utc_1972(TAI_AT_UTC_1972_01_01, UTC_1972_01_01)]
    #[case::before_leap_second(TAI_1S_BEFORE_2016_LEAP_SECOND, UTC_1S_BEFORE_2016_LEAP_SECOND)]
    #[case::during_leap_second(TAI_DURING_2016_LEAP_SECOND, UTC_DURING_2016_LEAP_SECOND)]
    #[case::after_leap_second(TAI_1S_AFTER_2016_LEAP_SECOND, UTC_1S_AFTER_2016_LEAP_SECOND)]
    #[case::before_1960(TAI_BEFORE_1960, UTC_BEFORE_1960)]
    fn test_tai_to_utc(#[case] tai: Time<Tai>, #[case] expected: Utc) {
        let actual = tai.to_utc();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_all_scales_to_utc() {
        use lox_approx::assert_approx_eq;

        let tai = time!(Tai, 2024, 5, 17, 12, 13, 14.0).unwrap();
        let exp = tai.to_utc();
        let tt = tai.to_scale(Tt);
        let act = tt.to_utc();
        assert_eq!(act, exp);
        let tcg = tai.to_scale(Tcg);
        let act = tcg.to_utc();
        assert_eq!(act, exp);
        // TCB conversions have lower precision due to the multi-step transformation
        let tcb = tai.to_scale(Tcb);
        let act = tcb.to_utc();
        assert_approx_eq!(act, exp);
        let tdb = tai.to_scale(Tdb);
        let act = tdb.to_utc();
        assert_eq!(act, exp);
    }

    /*
        The following fixtures are derived from a mixture of direct calculation and, in the case
        where inherent rounding errors prevent exact calculation, by cross-referencing with the
        observed outputs. The latter case is marked with a comment.
    */

    const UTC_1971_01_01: Utc =
        Utc::new_unchecked(Date::new_unchecked(1971, 1, 1), TimeOfDay::MIDNIGHT);

    const TAI_AT_UTC_1971_01_01: Time<Tai> = {
        const DELTA: TimeDelta = TimeDelta::from_seconds_and_subsecond_f64(8.0, 0.9461620000000011);
        let seconds = Date::new_unchecked(1971, 1, 1).j2000_day_number() * SECONDS_PER_DAY
            - SECONDS_PER_HALF_DAY;
        let base = TimeDelta::from_seconds(seconds);
        Time::from_delta(Tai, base.add_const(DELTA))
    };

    // 2016-12-31T23:59:59.000 UTC
    const UTC_1S_BEFORE_2016_LEAP_SECOND: Utc = Utc::new_unchecked(
        Date::new_unchecked(2016, 12, 31),
        TimeOfDay::new_unchecked(23, 59, 59),
    );

    // 2017-01-01T00:00:35.000 TAI
    const TAI_1S_BEFORE_2016_LEAP_SECOND: Time<Tai> = Time::new(Tai, 536500835, Subsecond::ZERO);

    // 2016-12-31T23:59:60.000 UTC
    const UTC_DURING_2016_LEAP_SECOND: Utc = Utc::new_unchecked(
        Date::new_unchecked(2016, 12, 31),
        TimeOfDay::new_unchecked(23, 59, 60),
    );

    // 2017-01-01T00:00:36.000 TAI
    const TAI_DURING_2016_LEAP_SECOND: Time<Tai> = Time::new(Tai, 536500836, Subsecond::ZERO);

    // 2017-01-01T00:00:00.000 UTC
    const UTC_1S_AFTER_2016_LEAP_SECOND: Utc =
        Utc::new_unchecked(Date::new_unchecked(2017, 1, 1), TimeOfDay::MIDNIGHT);

    // 2017-01-01T00:00:37.000 TAI
    const TAI_1S_AFTER_2016_LEAP_SECOND: Time<Tai> = Time::new(Tai, 536500837, Subsecond::ZERO);

    const UTC_BEFORE_1960: Utc =
        Utc::new_unchecked(Date::new_unchecked(1959, 12, 31), TimeOfDay::MIDNIGHT);

    // 1959-12-31T00:00:00.000 TAI (same as UTC since offset is zero pre-1960)
    const TAI_BEFORE_1960: Time<Tai> = {
        let seconds = Date::new_unchecked(1959, 12, 31).j2000_day_number() * SECONDS_PER_DAY
            - SECONDS_PER_HALF_DAY;
        Time::from_delta(Tai, TimeDelta::from_seconds(seconds))
    };
}
