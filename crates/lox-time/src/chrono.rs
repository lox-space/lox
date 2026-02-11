// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use chrono::{DateTime, Utc};
use lox_core::time::{
    calendar_dates::Date,
    deltas::{TimeDelta, ToDelta},
};
use thiserror::Error;

use crate::{Time, time_scales::Tai, utc::UtcError};

const ATTOS_TO_NANOS: i64 = 1_000_000_000;
const UNIX_EPOCH: TimeDelta = Date::new_unchecked(1970, 1, 1).to_delta();

#[derive(Debug, Error)]
#[error("{0} cannot be represented as a `chrono::DateTime`")]
pub struct ChronoError(TimeDelta);

impl TryFrom<Time<Tai>> for DateTime<Utc> {
    type Error = ChronoError;

    fn try_from(time: Time<Tai>) -> Result<Self, Self::Error> {
        delta_to_date_time(time.to_delta())
    }
}

impl From<DateTime<Utc>> for Time<Tai> {
    fn from(dt: DateTime<Utc>) -> Self {
        Time::from_delta(Tai, date_time_to_delta(dt))
    }
}

impl TryFrom<crate::utc::Utc> for DateTime<Utc> {
    type Error = ChronoError;

    fn try_from(time: crate::utc::Utc) -> Result<Self, Self::Error> {
        delta_to_date_time(time.to_delta())
    }
}

impl TryFrom<DateTime<Utc>> for crate::utc::Utc {
    type Error = UtcError;

    fn try_from(dt: DateTime<Utc>) -> Result<Self, Self::Error> {
        crate::utc::Utc::from_delta(date_time_to_delta(dt))
    }
}

fn delta_to_date_time(delta: TimeDelta) -> Result<DateTime<Utc>, ChronoError> {
    let dt = delta - UNIX_EPOCH;
    let (second, subsecond) = dt.as_seconds_and_subsecond().ok_or(ChronoError(delta))?;
    let nanos = subsecond.as_attoseconds() / ATTOS_TO_NANOS;
    DateTime::from_timestamp(second, nanos as u32).ok_or(ChronoError(delta))
}

fn date_time_to_delta(dt: DateTime<Utc>) -> TimeDelta {
    let delta = TimeDelta::new(
        dt.timestamp(),
        dt.timestamp_subsec_nanos() as i64 * ATTOS_TO_NANOS,
    );
    delta + UNIX_EPOCH
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(UNIX_EPOCH)]
    #[case(TimeDelta::default())]
    #[case(TimeDelta::from_seconds_f64(0.123456))]
    #[should_panic(expected = "NaN")]
    #[case(TimeDelta::NaN)]
    fn test_chrono_time_roundtrip(#[case] delta: TimeDelta) {
        let exp = Time::from_delta(Tai, delta);
        let dt: DateTime<Utc> = exp.try_into().unwrap();
        let act: Time<Tai> = dt.into();
        assert_eq!(act, exp)
    }

    #[rstest]
    #[case(UNIX_EPOCH)]
    #[case(TimeDelta::default())]
    #[case(TimeDelta::from_seconds_f64(0.123456))]
    fn test_chrono_utc_roundtrip(#[case] delta: TimeDelta) {
        let exp = crate::utc::Utc::from_delta(delta).unwrap();
        let dt: DateTime<Utc> = exp.try_into().unwrap();
        let act: crate::utc::Utc = dt.try_into().unwrap();
        assert_eq!(act, exp)
    }
}
