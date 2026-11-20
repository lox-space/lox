// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Conversions between `lox-time` types and [`std::time::SystemTime`].
//!
//! `SystemTime` counts Unix time, which ignores leap seconds. A leap second therefore has no
//! `SystemTime` of its own: 23:59:60 converts to the same instant as 00:00:00 of the next day.

use std::time::{Duration, SystemTime};

use thiserror::Error;

use crate::Time;
use crate::offsets::{DefaultOffsetProvider, Offset};
use crate::time_scales::{ContinuousTimeScale, Tai};
use crate::utc::Utc;
use crate::utc::transformations::ToUtc;

/// Error returned when a [`SystemTime`] is outside the range of [`Utc`].
#[derive(Debug, Error, PartialEq, Eq)]
#[error("{0:?} cannot be represented as a `Utc` instant")]
pub struct FromSystemTimeError(pub SystemTime);

/// Error returned when a UTC instant cannot be represented as a [`SystemTime`].
#[derive(Debug, Error, PartialEq, Eq)]
#[error("{0} cannot be represented as a `SystemTime`")]
pub struct ToSystemTimeError(pub Utc);

impl TryFrom<SystemTime> for Utc {
    type Error = FromSystemTimeError;

    fn try_from(time: SystemTime) -> Result<Self, Self::Error> {
        let utc = match time.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(after) => i64::try_from(after.as_secs())
                .ok()
                .and_then(|seconds| Utc::from_unix_timestamp(seconds, after.subsec_nanos() as i64)),
            Err(err) => {
                let before = err.duration();
                i64::try_from(before.as_secs()).ok().and_then(|seconds| {
                    Utc::from_unix_timestamp(-seconds, -(before.subsec_nanos() as i64))
                })
            }
        };
        utc.ok_or(FromSystemTimeError(time))
    }
}

impl TryFrom<Utc> for SystemTime {
    type Error = ToSystemTimeError;

    /// Converts the instant, rounding it down to whole nanoseconds.
    fn try_from(utc: Utc) -> Result<Self, Self::Error> {
        let Some((seconds, nanoseconds)) = utc.to_unix_timestamp() else {
            return Err(ToSystemTimeError(utc));
        };
        let whole = Duration::from_secs(seconds.unsigned_abs());
        let whole = if seconds < 0 {
            SystemTime::UNIX_EPOCH.checked_sub(whole)
        } else {
            SystemTime::UNIX_EPOCH.checked_add(whole)
        };
        whole
            .and_then(|time| time.checked_add(Duration::from_nanos(nanoseconds as u64)))
            .ok_or(ToSystemTimeError(utc))
    }
}

impl<T> TryFrom<Time<T>> for SystemTime
where
    T: ContinuousTimeScale + Copy,
    DefaultOffsetProvider: Offset<T, Tai>,
{
    type Error = ToSystemTimeError;

    fn try_from(time: Time<T>) -> Result<Self, Self::Error> {
        time.to_utc().try_into()
    }
}

impl TryFrom<SystemTime> for Time<Tai> {
    type Error = FromSystemTimeError;

    fn try_from(time: SystemTime) -> Result<Self, Self::Error> {
        Utc::try_from(time).map(|utc| utc.to_time())
    }
}

impl TryFrom<SystemTime> for Time {
    type Error = FromSystemTimeError;

    fn try_from(time: SystemTime) -> Result<Self, Self::Error> {
        Utc::try_from(time).map(|utc| utc.to_dynamic_time())
    }
}

impl Utc {
    /// Returns the current instant according to the system clock.
    ///
    /// # Panics
    ///
    /// Panics if the system clock is set outside the range of [`Utc`].
    pub fn now() -> Self {
        Utc::try_from(SystemTime::now()).expect("the system clock should be within range")
    }
}

impl<T: ContinuousTimeScale + Copy> Time<T> {
    /// Returns the current instant according to the system clock in the given time scale.
    ///
    /// # Panics
    ///
    /// Panics if the system clock is set outside the range of [`Utc`].
    pub fn now(scale: T) -> Self
    where
        DefaultOffsetProvider: Offset<Tai, T>,
    {
        Utc::now().to_time().to_scale(scale)
    }
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;
    use rstest::rstest;

    use crate::TimeDelta;
    use crate::calendar_dates::UNIX_EPOCH;
    use crate::time_scales::TimeScale;

    use super::*;

    fn unix(seconds: i64, nanos: u32) -> SystemTime {
        let whole = Duration::from_secs(seconds.unsigned_abs());
        let whole = if seconds < 0 {
            SystemTime::UNIX_EPOCH - whole
        } else {
            SystemTime::UNIX_EPOCH + whole
        };
        whole + Duration::from_nanos(nanos as u64)
    }

    #[rstest]
    #[case("1970-01-01T00:00:00", unix(0, 0))]
    #[case("2000-01-01T12:00:00", unix(946_728_000, 0))]
    #[case("2016-12-31T23:59:59.123456789", unix(1_483_228_799, 123_456_789))]
    #[case("2038-01-19T03:14:08", unix(2_147_483_648, 0))]
    #[case("1969-12-31T23:59:59.5", unix(-1, 500_000_000))]
    #[case("1900-01-01T00:00:00", unix(-2_208_988_800, 0))]
    fn test_system_time_utc(#[case] iso: &str, #[case] system_time: SystemTime) {
        let utc = Utc::from_iso(iso).unwrap();
        assert_eq!(SystemTime::try_from(utc).unwrap(), system_time);
        assert_eq!(Utc::try_from(system_time).unwrap(), utc);
    }

    #[test]
    fn test_system_time_leap_second_is_next_midnight() {
        let leap_second = Utc::from_iso("2016-12-31T23:59:60").unwrap();
        let midnight = Utc::from_iso("2017-01-01T00:00:00").unwrap();
        assert_eq!(
            SystemTime::try_from(leap_second).unwrap(),
            SystemTime::try_from(midnight).unwrap()
        );
    }

    #[rstest]
    #[case("2000-01-01T00:00:00.000000000999", unix(946_684_800, 0))]
    #[case("1969-12-31T23:59:59.999999999999", unix(-1, 999_999_999))]
    fn test_system_time_rounds_down_to_nanoseconds(
        #[case] iso: &str,
        #[case] system_time: SystemTime,
    ) {
        let utc = Utc::from_iso(iso).unwrap();
        assert_eq!(SystemTime::try_from(utc).unwrap(), system_time);
    }

    #[test]
    fn test_system_time_tai() {
        // TAI - UTC = 37 s since 2017.
        let system_time = unix(1_700_000_000, 0);
        let tai: Time<Tai> = system_time.try_into().unwrap();
        let exp = Time::from_delta(Tai, TimeDelta::from_seconds(1_700_000_037) + UNIX_EPOCH);
        assert_eq!(tai, exp);
        assert_eq!(SystemTime::try_from(tai).unwrap(), system_time);
    }

    #[test]
    fn test_system_time_dynamic_time() {
        let system_time = unix(1_700_000_000, 250_000_000);
        let time: Time = system_time.try_into().unwrap();
        assert_eq!(time.scale(), TimeScale::Tai);
        assert_eq!(SystemTime::try_from(time).unwrap(), system_time);
        let tt = time.to_scale(TimeScale::Tt);
        assert_eq!(SystemTime::try_from(tt).unwrap(), system_time);
    }

    #[test]
    fn test_system_time_now_roundtrip() {
        let now = SystemTime::now();
        let utc = Utc::try_from(now).unwrap();
        assert_eq!(SystemTime::try_from(utc).unwrap(), now);
    }

    // The wall clock may step backwards between two readings, so `now` is compared with a
    // second reading within a tolerance instead of bracketed by two readings.
    const NOW_TOLERANCE_SECONDS: f64 = 60.0;

    #[test]
    fn test_utc_now() {
        let now = Utc::now();
        let exp = Utc::try_from(SystemTime::now()).unwrap();
        assert_approx_eq!(now, exp, atol <= NOW_TOLERANCE_SECONDS);
    }

    #[test]
    fn test_time_now() {
        let now = Time::now(Tai);
        let exp: Time<Tai> = SystemTime::now().try_into().unwrap();
        assert_approx_eq!(now, exp, atol <= NOW_TOLERANCE_SECONDS);
    }

    #[rstest]
    #[case(TimeScale::Tai)]
    #[case(TimeScale::Tt)]
    #[case(TimeScale::Tdb)]
    fn test_time_now_dynamic(#[case] scale: TimeScale) {
        let now = Time::now(scale);
        let exp: Time = SystemTime::now().try_into().unwrap();
        assert_eq!(now.scale(), scale);
        assert_approx_eq!(
            now.to_scale(TimeScale::Tai),
            exp,
            atol <= NOW_TOLERANCE_SECONDS
        );
    }

    #[test]
    fn test_utc_out_of_range() {
        // Late enough that the offset from the Unix epoch overflows, early enough that
        // `Utc::from_delta` does not.
        let utc = Utc::from_delta(TimeDelta::from_seconds(i64::MAX - 2 * 86_400));
        assert_eq!(SystemTime::try_from(utc), Err(ToSystemTimeError(utc)));
    }

    #[cfg(unix)]
    #[test]
    fn test_system_time_out_of_range() {
        let system_time = SystemTime::UNIX_EPOCH
            .checked_sub(Duration::from_secs(i64::MAX as u64))
            .unwrap();
        assert_eq!(
            Utc::try_from(system_time),
            Err(FromSystemTimeError(system_time))
        );
    }
}
