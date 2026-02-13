// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use chrono::{DateTime, Utc};
use lox_core::time::deltas::ToDelta;

pub use lox_core::time::chrono::ChronoError;

use crate::{Time, time_scales::Tai, utc::UtcError};

impl TryFrom<Time<Tai>> for DateTime<Utc> {
    type Error = ChronoError;

    fn try_from(time: Time<Tai>) -> Result<Self, Self::Error> {
        time.to_delta().try_into()
    }
}

impl From<DateTime<Utc>> for Time<Tai> {
    fn from(dt: DateTime<Utc>) -> Self {
        Time::from_delta(Tai, dt.into())
    }
}

impl TryFrom<crate::utc::Utc> for DateTime<Utc> {
    type Error = ChronoError;

    fn try_from(utc: crate::utc::Utc) -> Result<Self, Self::Error> {
        utc.to_delta().try_into()
    }
}

impl TryFrom<DateTime<Utc>> for crate::utc::Utc {
    type Error = UtcError;

    fn try_from(dt: DateTime<Utc>) -> Result<Self, Self::Error> {
        crate::utc::Utc::from_delta(dt.into())
    }
}

#[cfg(test)]
mod tests {
    use lox_core::time::{constants::UNIX_EPOCH, deltas::TimeDelta};
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
