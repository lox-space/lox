// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Conversions between `lox-time` and `time` types.
//!
//! `time::Timestamp`, `time::UtcDateTime` and `time::OffsetDateTime` count Unix time, which
//! ignores leap seconds. A leap second therefore has no representation of its own: 23:59:60
//! converts to the same instant as 00:00:00 of the next day. An `OffsetDateTime` created from
//! a lox type is always in UTC.

use thiserror::Error;
use time::{OffsetDateTime, Timestamp, UtcDateTime};

pub use lox_core::time::FromTimeRsDurationError;

use crate::Time;
use crate::offsets::{DefaultOffsetProvider, Offset};
use crate::time_scales::{ContinuousTimeScale, Tai};
use crate::utc::Utc;
use crate::utc::transformations::ToUtc;

/// Error returned when a UTC instant cannot be represented as a `time` type.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("{0} cannot be represented as a `time` date-time")]
pub struct ToTimeRsError(pub Utc);

/// Converts a UTC instant to a `time::Timestamp`, rounding it down to whole nanoseconds.
fn to_timestamp(utc: Utc) -> Result<Timestamp, ToTimeRsError> {
    utc.to_unix_timestamp()
        .and_then(|(seconds, nanoseconds)| Timestamp::new(seconds, nanoseconds).ok())
        .ok_or(ToTimeRsError(utc))
}

fn from_timestamp(timestamp: Timestamp) -> Utc {
    Utc::from_unix_timestamp(timestamp.as_seconds(), timestamp.nanosecond() as i64)
        .expect("a `time::Timestamp` should be within the range of `Utc`")
}

macro_rules! impl_time_rs_conversions {
    ($ty:ty) => {
        impl TryFrom<Utc> for $ty {
            type Error = ToTimeRsError;

            /// Converts the instant, rounding it down to whole nanoseconds.
            fn try_from(utc: Utc) -> Result<Self, Self::Error> {
                to_timestamp(utc).map(Self::from)
            }
        }

        impl From<$ty> for Utc {
            fn from(value: $ty) -> Self {
                from_timestamp(Timestamp::from(value))
            }
        }

        impl<T> TryFrom<Time<T>> for $ty
        where
            T: ContinuousTimeScale + Copy,
            DefaultOffsetProvider: Offset<T, Tai>,
        {
            type Error = ToTimeRsError;

            fn try_from(time: Time<T>) -> Result<Self, Self::Error> {
                time.to_utc().try_into()
            }
        }

        impl From<$ty> for Time<Tai> {
            fn from(value: $ty) -> Self {
                Utc::from(value).to_time()
            }
        }

        impl From<$ty> for Time {
            fn from(value: $ty) -> Self {
                Utc::from(value).to_dynamic_time()
            }
        }
    };
}

impl_time_rs_conversions!(Timestamp);
impl_time_rs_conversions!(UtcDateTime);
impl_time_rs_conversions!(OffsetDateTime);

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use time::UtcOffset;

    use crate::TimeDelta;
    use crate::calendar_dates::UNIX_EPOCH;
    use crate::time_scales::TimeScale;

    use super::*;

    fn ts(seconds: i64, nanoseconds: u32) -> Timestamp {
        Timestamp::new(seconds, nanoseconds).unwrap()
    }

    #[rstest]
    #[case("1970-01-01T00:00:00", ts(0, 0))]
    #[case("2000-01-01T12:00:00", ts(946_728_000, 0))]
    #[case("2016-12-31T23:59:59.123456789", ts(1_483_228_799, 123_456_789))]
    #[case("1969-12-31T23:59:59.5", ts(-1, 500_000_000))]
    #[case("1900-01-01T00:00:00", ts(-2_208_988_800, 0))]
    fn test_time_rs_utc(#[case] iso: &str, #[case] timestamp: Timestamp) {
        let utc = Utc::from_iso(iso).unwrap();

        assert_eq!(Timestamp::try_from(utc).unwrap(), timestamp);
        assert_eq!(Utc::from(timestamp), utc);

        let utc_date_time = UtcDateTime::from(timestamp);
        assert_eq!(UtcDateTime::try_from(utc).unwrap(), utc_date_time);
        assert_eq!(Utc::from(utc_date_time), utc);

        let offset_date_time = OffsetDateTime::from(timestamp);
        assert_eq!(OffsetDateTime::try_from(utc).unwrap(), offset_date_time);
        assert_eq!(Utc::from(offset_date_time), utc);
    }

    #[test]
    fn test_time_rs_offset_date_time_is_utc() {
        let utc = Utc::from_iso("2024-06-01T12:00:00").unwrap();
        let offset_date_time = OffsetDateTime::try_from(utc).unwrap();
        assert_eq!(offset_date_time.offset(), UtcOffset::UTC);
    }

    #[test]
    fn test_time_rs_offset_date_time_with_offset() {
        // 14:00 at UTC+2 is 12:00 UTC.
        let offset = UtcOffset::from_hms(2, 0, 0).unwrap();
        let offset_date_time = OffsetDateTime::from(ts(1_717_243_200, 0)).to_offset(offset);
        assert_eq!(offset_date_time.hour(), 14);
        let exp = Utc::from_iso("2024-06-01T12:00:00").unwrap();
        assert_eq!(Utc::from(offset_date_time), exp);
    }

    #[test]
    fn test_time_rs_leap_second_is_next_midnight() {
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
    fn test_time_rs_rounds_down_to_nanoseconds(#[case] iso: &str, #[case] timestamp: Timestamp) {
        let utc = Utc::from_iso(iso).unwrap();
        assert_eq!(Timestamp::try_from(utc).unwrap(), timestamp);
    }

    #[test]
    fn test_time_rs_tai() {
        // TAI - UTC = 37 s since 2017.
        let timestamp = ts(1_700_000_000, 0);
        let tai = Time::<Tai>::from(timestamp);
        let exp = Time::from_delta(Tai, TimeDelta::from_seconds(1_700_000_037) + UNIX_EPOCH);
        assert_eq!(tai, exp);
        assert_eq!(Timestamp::try_from(tai).unwrap(), timestamp);
        assert_eq!(
            UtcDateTime::try_from(tai).unwrap(),
            UtcDateTime::from(timestamp)
        );
    }

    #[test]
    fn test_time_rs_dynamic_time() {
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
    fn test_time_rs_limits(#[case] timestamp: Timestamp) {
        let utc = Utc::from(timestamp);
        assert_eq!(Timestamp::try_from(utc).unwrap(), timestamp);
    }

    #[test]
    fn test_time_rs_just_after_max() {
        let max = Utc::from(Timestamp::MAX).to_delta();
        let utc = Utc::from_delta(max + TimeDelta::from_nanoseconds(1));
        assert_eq!(Timestamp::try_from(utc), Err(ToTimeRsError(utc)));
        assert_eq!(UtcDateTime::try_from(utc), Err(ToTimeRsError(utc)));
        assert_eq!(OffsetDateTime::try_from(utc), Err(ToTimeRsError(utc)));
    }
}
