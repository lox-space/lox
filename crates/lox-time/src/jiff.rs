// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Conversions between `lox-time` and `jiff` types.
//!
//! `jiff::Timestamp` counts Unix time, which ignores leap seconds. A leap second therefore has
//! no timestamp of its own: 23:59:60 converts to the same instant as 00:00:00 of the next day.

use jiff::Timestamp;
use thiserror::Error;

pub use lox_core::time::FromJiffDurationError;

use crate::Time;
use crate::offsets::{DefaultOffsetProvider, Offset};
use crate::time_scales::{ContinuousTimeScale, Tai};
use crate::utc::Utc;
use crate::utc::transformations::ToUtc;

/// Error returned when a UTC instant cannot be represented as a `jiff::Timestamp`.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("{0} cannot be represented as a `jiff::Timestamp`")]
pub struct ToJiffTimestampError(pub Utc);

impl TryFrom<Utc> for Timestamp {
    type Error = ToJiffTimestampError;

    /// Converts the instant, rounding it down to whole nanoseconds.
    fn try_from(utc: Utc) -> Result<Self, Self::Error> {
        utc.to_unix_timestamp()
            .and_then(|(seconds, nanoseconds)| Timestamp::new(seconds, nanoseconds as i32).ok())
            .ok_or(ToJiffTimestampError(utc))
    }
}

impl From<Timestamp> for Utc {
    fn from(timestamp: Timestamp) -> Self {
        Utc::from_unix_timestamp(timestamp.as_second(), timestamp.subsec_nanosecond() as i64)
            .expect("a `jiff::Timestamp` should be within the range of `Utc`")
    }
}

impl<T> TryFrom<Time<T>> for Timestamp
where
    T: ContinuousTimeScale + Copy,
    DefaultOffsetProvider: Offset<T, Tai>,
{
    type Error = ToJiffTimestampError;

    fn try_from(time: Time<T>) -> Result<Self, Self::Error> {
        time.to_utc().try_into()
    }
}

impl From<Timestamp> for Time<Tai> {
    fn from(timestamp: Timestamp) -> Self {
        Utc::from(timestamp).to_time()
    }
}

impl From<Timestamp> for Time {
    fn from(timestamp: Timestamp) -> Self {
        Utc::from(timestamp).to_dynamic_time()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::TimeDelta;
    use crate::calendar_dates::UNIX_EPOCH;
    use crate::time_scales::TimeScale;

    use super::*;

    fn ts(second: i64, nanosecond: i32) -> Timestamp {
        Timestamp::new(second, nanosecond).unwrap()
    }

    #[rstest]
    #[case("1970-01-01T00:00:00", ts(0, 0))]
    #[case("2000-01-01T12:00:00", ts(946_728_000, 0))]
    #[case("2016-12-31T23:59:59.123456789", ts(1_483_228_799, 123_456_789))]
    #[case("1969-12-31T23:59:59.5", ts(0, -500_000_000))]
    #[case("1900-01-01T00:00:00", ts(-2_208_988_800, 0))]
    fn test_jiff_timestamp_utc(#[case] iso: &str, #[case] timestamp: Timestamp) {
        let utc = Utc::from_iso(iso).unwrap();
        assert_eq!(Timestamp::try_from(utc).unwrap(), timestamp);
        assert_eq!(Utc::from(timestamp), utc);
    }

    #[test]
    fn test_jiff_timestamp_leap_second_is_next_midnight() {
        let leap_second = Utc::from_iso("2016-12-31T23:59:60").unwrap();
        let midnight = Utc::from_iso("2017-01-01T00:00:00").unwrap();
        assert_eq!(
            Timestamp::try_from(leap_second).unwrap(),
            Timestamp::try_from(midnight).unwrap()
        );
    }

    #[rstest]
    #[case("2000-01-01T00:00:00.000000000999", ts(946_684_800, 0))]
    #[case("1969-12-31T23:59:59.999999999999", ts(-1, 999_999_999))]
    fn test_jiff_timestamp_rounds_down_to_nanoseconds(
        #[case] iso: &str,
        #[case] timestamp: Timestamp,
    ) {
        let utc = Utc::from_iso(iso).unwrap();
        assert_eq!(Timestamp::try_from(utc).unwrap(), timestamp);
    }

    #[test]
    fn test_jiff_timestamp_tai() {
        // TAI - UTC = 37 s since 2017.
        let timestamp = ts(1_700_000_000, 0);
        let tai = Time::<Tai>::from(timestamp);
        let exp = Time::from_delta(Tai, TimeDelta::from_seconds(1_700_000_037) + UNIX_EPOCH);
        assert_eq!(tai, exp);
        assert_eq!(Timestamp::try_from(tai).unwrap(), timestamp);
    }

    #[test]
    fn test_jiff_timestamp_dynamic_time() {
        let timestamp = ts(1_700_000_000, 250_000_000);
        let time: Time = timestamp.into();
        assert_eq!(time.scale(), TimeScale::Tai);
        assert_eq!(Timestamp::try_from(time).unwrap(), timestamp);
        let tt = time.to_scale(TimeScale::Tt);
        assert_eq!(Timestamp::try_from(tt).unwrap(), timestamp);
    }

    #[rstest]
    #[case(Timestamp::MIN)]
    #[case(Timestamp::MAX)]
    fn test_jiff_timestamp_limits(#[case] timestamp: Timestamp) {
        let utc = Utc::from(timestamp);
        assert_eq!(Timestamp::try_from(utc).unwrap(), timestamp);
    }

    #[test]
    fn test_jiff_timestamp_just_after_max() {
        let utc =
            Utc::from_delta(Utc::from(Timestamp::MAX).to_delta() + TimeDelta::from_nanoseconds(1));
        assert_eq!(Timestamp::try_from(utc), Err(ToJiffTimestampError(utc)));
    }

    #[rstest]
    #[case(10_000)]
    #[case(-10_000)]
    fn test_jiff_timestamp_out_of_range(#[case] year: i64) {
        let utc = Utc::builder().with_ymd(year, 1, 1).build().unwrap();
        assert_eq!(Timestamp::try_from(utc), Err(ToJiffTimestampError(utc)));
    }
}
