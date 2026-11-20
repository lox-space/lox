// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Checks that the conversions to and from other time libraries are reachable through
//! `lox-space`, including their error types.

#![cfg(feature = "time")]

use std::time::{Duration, SystemTime};

use lox_space::core::time::{FromDurationError, ToDurationError};
use lox_space::time::TimeDelta;
use lox_space::time::system_time::{FromSystemTimeError, ToSystemTimeError};
use lox_space::time::utc::Utc;

const SECONDS: i64 = 1_700_000_000;
const NANOSECONDS: u32 = 250_000_000;

fn utc() -> Utc {
    Utc::from_unix_timestamp(SECONDS, NANOSECONDS as i64).unwrap()
}

#[test]
fn test_std_duration() {
    let duration = Duration::new(SECONDS as u64, NANOSECONDS);
    let delta: Result<TimeDelta, FromDurationError> = duration.try_into();
    let back: Result<Duration, ToDurationError> = delta.unwrap().try_into();
    assert_eq!(back, Ok(duration));
}

#[test]
fn test_system_time() {
    let system_time: Result<SystemTime, ToSystemTimeError> = utc().try_into();
    let system_time = system_time.unwrap();
    assert_eq!(
        system_time,
        SystemTime::UNIX_EPOCH + Duration::new(SECONDS as u64, NANOSECONDS)
    );
    let back: Result<Utc, FromSystemTimeError> = system_time.try_into();
    assert_eq!(back, Ok(utc()));
}

#[cfg(feature = "jiff")]
#[test]
fn test_jiff() {
    use lox_space::core::time::FromJiffDurationError;
    use lox_space::time::Time;
    use lox_space::time::jiff::ToJiffTimestampError;
    use lox_space::time::time_scales::Tai;

    let timestamp = jiff::Timestamp::new(SECONDS, NANOSECONDS as i32).unwrap();
    let act: Result<jiff::Timestamp, ToJiffTimestampError> = utc().try_into();
    assert_eq!(act, Ok(timestamp));
    assert_eq!(Utc::from(timestamp), utc());
    assert_eq!(Time::<Tai>::from(timestamp), utc().to_time());

    let delta: Result<TimeDelta, FromJiffDurationError> =
        jiff::SignedDuration::from_millis(-1_500).try_into();
    assert_eq!(delta, Ok(TimeDelta::from_milliseconds(-1_500)));
}

#[cfg(feature = "time-rs")]
#[test]
fn test_time_rs() {
    use lox_space::core::time::FromTimeRsDurationError;
    use lox_space::time::Time;
    use lox_space::time::time_rs::ToTimeRsError;
    use lox_space::time::time_scales::Tai;

    let timestamp = time::Timestamp::new(SECONDS, NANOSECONDS).unwrap();
    let act: Result<time::OffsetDateTime, ToTimeRsError> = utc().try_into();
    assert_eq!(act, Ok(time::OffsetDateTime::from(timestamp)));
    assert_eq!(Utc::from(time::UtcDateTime::from(timestamp)), utc());
    assert_eq!(Time::<Tai>::from(timestamp), utc().to_time());

    let delta: Result<TimeDelta, FromTimeRsDurationError> =
        time::SignedDuration::milliseconds(-1_500).try_into();
    assert_eq!(delta, Ok(TimeDelta::from_milliseconds(-1_500)));
}
