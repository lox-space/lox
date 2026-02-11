// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{
    i64::consts::ATTOSECONDS_IN_NANOSECOND,
    time::{constants::UNIX_EPOCH, deltas::TimeDelta},
};

#[derive(Debug, Error)]
pub enum ChronoError {
    #[error("{0} cannot be represented as a `chrono::DateTime`")]
    DateTime(TimeDelta),
    #[error("{0} cannot be represented as a `chrono::TimeDelta`")]
    TimeDelta(TimeDelta),
}

impl TryFrom<TimeDelta> for DateTime<Utc> {
    type Error = ChronoError;

    fn try_from(delta: TimeDelta) -> Result<Self, Self::Error> {
        let (second, nanos) = delta
            .to_unix_second_and_nanos()
            .ok_or(ChronoError::DateTime(delta))?;
        DateTime::from_timestamp(second, nanos).ok_or(ChronoError::DateTime(delta))
    }
}

impl From<DateTime<Utc>> for TimeDelta {
    fn from(dt: DateTime<Utc>) -> Self {
        TimeDelta::from_unix_second_and_nanos(dt.timestamp(), dt.timestamp_subsec_nanos())
    }
}

impl TryFrom<TimeDelta> for chrono::TimeDelta {
    type Error = ChronoError;

    fn try_from(delta: TimeDelta) -> Result<Self, Self::Error> {
        let (second, nanos) = delta
            .to_unix_second_and_nanos()
            .ok_or(ChronoError::TimeDelta(delta))?;
        chrono::TimeDelta::new(second, nanos).ok_or(ChronoError::TimeDelta(delta))
    }
}

impl From<chrono::TimeDelta> for TimeDelta {
    fn from(delta: chrono::TimeDelta) -> Self {
        let mut second = delta.num_seconds();
        let mut nanos = delta.subsec_nanos();
        if nanos < 0 {
            second -= 1;
            nanos += 1_000_000_000;
        }
        TimeDelta::from_unix_second_and_nanos(second, nanos as u32)
    }
}

impl TimeDelta {
    fn to_unix_second_and_nanos(self) -> Option<(i64, u32)> {
        let delta = self - UNIX_EPOCH;
        delta.as_seconds_and_subsecond().map(|(second, subsecond)| {
            (
                second,
                (subsecond.as_attoseconds() / ATTOSECONDS_IN_NANOSECOND) as u32,
            )
        })
    }

    fn from_unix_second_and_nanos(second: i64, nanos: u32) -> Self {
        TimeDelta::new(second, nanos as i64 * ATTOSECONDS_IN_NANOSECOND) + UNIX_EPOCH
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(UNIX_EPOCH)]
    #[case(TimeDelta::default())]
    #[case(TimeDelta::from_seconds_f64(0.123456))]
    #[should_panic(expected = "NaN")]
    #[case(TimeDelta::NaN)]
    fn test_chrono_time_delta_roundtrip(#[case] exp: TimeDelta) {
        let dt: DateTime<Utc> = exp.try_into().unwrap();
        let act: TimeDelta = dt.into();
        assert_eq!(act, exp)
    }

    #[rstest]
    #[case(UNIX_EPOCH)]
    #[case(UNIX_EPOCH + TimeDelta::from_seconds(1))]
    #[case(UNIX_EPOCH + TimeDelta::from_seconds(-1))]
    #[case(UNIX_EPOCH + TimeDelta::from_seconds_f64(1.123456))]
    #[case(UNIX_EPOCH + TimeDelta::from_seconds_f64(-1.123456))]
    #[should_panic(expected = "NaN")]
    #[case(TimeDelta::NaN)]
    fn test_foo(#[case] exp: TimeDelta) {
        let delta: chrono::TimeDelta = exp.try_into().unwrap();
        let act: TimeDelta = delta.into();
        assert_approx_eq!(act, exp, atol <= 1e-8);
    }
}
