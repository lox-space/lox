// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use chrono::{DateTime, Utc};
use lox_core::time::constants::{ATTOSECONDS_IN_NANOSECOND, UNIX_EPOCH};
use lox_core::time::deltas::{TimeDelta, ToDelta};
use thiserror::Error;

pub use lox_core::time::chrono::ChronoTimeDeltaError;

use crate::Time;
use crate::offsets::{DefaultOffsetProvider, Offset};
use crate::time_scales::{ContinuousTimeScale, Tai};
use crate::utc::transformations::ToUtc;

/// Error returned when a UTC instant cannot be represented as a `chrono::DateTime`.
#[derive(Debug, Error)]
#[error("{0} cannot be represented as a `chrono::DateTime`")]
pub struct ChronoError(pub crate::utc::Utc);

/// Converts a UTC instant to seconds and nanoseconds since the Unix epoch.
fn to_unix_second_and_nanos(utc: crate::utc::Utc) -> (i64, u32) {
    let delta = utc.to_delta() - UNIX_EPOCH;
    let nanos = (delta.attoseconds() / ATTOSECONDS_IN_NANOSECOND) as u32;
    (delta.seconds(), nanos)
}

/// Converts seconds and nanoseconds since the Unix epoch to a UTC instant.
fn from_unix_second_and_nanos(second: i64, nanos: u32) -> crate::utc::Utc {
    let delta = TimeDelta::new(second, nanos as i64 * ATTOSECONDS_IN_NANOSECOND) + UNIX_EPOCH;
    crate::utc::Utc::from_delta(delta)
}

impl<T> TryFrom<Time<T>> for DateTime<Utc>
where
    T: ContinuousTimeScale + Copy,
    DefaultOffsetProvider: Offset<T, Tai>,
{
    type Error = ChronoError;

    fn try_from(time: Time<T>) -> Result<Self, Self::Error> {
        time.to_utc().try_into()
    }
}

impl From<DateTime<Utc>> for Time<Tai> {
    fn from(dt: DateTime<Utc>) -> Self {
        crate::utc::Utc::from(dt).to_time()
    }
}

impl From<DateTime<Utc>> for Time {
    fn from(dt: DateTime<Utc>) -> Self {
        crate::utc::Utc::from(dt).to_dynamic_time()
    }
}

impl TryFrom<crate::utc::Utc> for DateTime<Utc> {
    type Error = ChronoError;

    /// Converts the instant, rounding it down to whole nanoseconds.
    fn try_from(utc: crate::utc::Utc) -> Result<Self, Self::Error> {
        let (second, nanos) = to_unix_second_and_nanos(utc);
        DateTime::from_timestamp(second, nanos).ok_or(ChronoError(utc))
    }
}

impl From<DateTime<Utc>> for crate::utc::Utc {
    fn from(dt: DateTime<Utc>) -> Self {
        from_unix_second_and_nanos(dt.timestamp(), dt.timestamp_subsec_nanos())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::time_scales::TimeScale;

    use super::*;

    // Post-1972 delta where TAI-UTC offset is an exact integer, ensuring
    // lossless roundtrip through chrono's nanosecond precision.
    const POST_1972_DELTA: TimeDelta = TimeDelta::from_seconds(-86400);

    #[rstest]
    #[case(POST_1972_DELTA)]
    #[case(TimeDelta::default())]
    #[case(TimeDelta::from_seconds_f64(0.123456))]
    fn test_chrono_time_tai_roundtrip(#[case] delta: TimeDelta) {
        let exp = Time::from_delta(Tai, delta);
        let dt: DateTime<Utc> = exp.try_into().unwrap();
        let act: Time<Tai> = dt.into();
        assert_eq!(act, exp)
    }

    #[rstest]
    #[case(POST_1972_DELTA)]
    #[case(TimeDelta::default())]
    #[case(TimeDelta::from_seconds_f64(0.123456))]
    fn test_chrono_time_roundtrip(#[case] delta: TimeDelta) {
        let exp = Time::from_delta(TimeScale::Tai, delta);
        let dt: DateTime<Utc> = exp.try_into().unwrap();
        let act: Time = dt.into();
        assert_eq!(act, exp)
    }

    #[rstest]
    #[case(UNIX_EPOCH)]
    #[case(TimeDelta::default())]
    #[case(TimeDelta::from_seconds_f64(0.123456))]
    fn test_chrono_utc_roundtrip(#[case] delta: TimeDelta) {
        let exp = crate::utc::Utc::from_delta(delta);
        let dt: DateTime<Utc> = exp.try_into().unwrap();
        let act: crate::utc::Utc = dt.into();
        assert_eq!(act, exp)
    }
}
