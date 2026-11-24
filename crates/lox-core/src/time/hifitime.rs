// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use hifitime::Duration;
use thiserror::Error;

use crate::time::TimeDelta;
use crate::time::constants::ATTOSECONDS_IN_NANOSECOND;

const NANOSECONDS_IN_SECOND: i128 = 1_000_000_000;

/// Error returned when a [`TimeDelta`] is longer than ±32,768 centuries and therefore
/// cannot be represented as a `hifitime::Duration`.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("{0:?} cannot be represented as a `hifitime::Duration`")]
pub struct ToHifitimeDurationError(pub TimeDelta);

impl TryFrom<TimeDelta> for Duration {
    type Error = ToHifitimeDurationError;

    /// Converts the duration, rounding it down to whole nanoseconds.
    fn try_from(delta: TimeDelta) -> Result<Self, Self::Error> {
        let nanoseconds = delta.seconds() as i128 * NANOSECONDS_IN_SECOND
            + (delta.attoseconds() / ATTOSECONDS_IN_NANOSECOND) as i128;
        // `Duration::from_total_nanoseconds` saturates instead of failing.
        if nanoseconds < Duration::MIN.total_nanoseconds()
            || nanoseconds > Duration::MAX.total_nanoseconds()
        {
            return Err(ToHifitimeDurationError(delta));
        }
        Ok(Duration::from_total_nanoseconds(nanoseconds))
    }
}

impl From<Duration> for TimeDelta {
    fn from(duration: Duration) -> Self {
        let nanoseconds = duration.total_nanoseconds();
        let seconds = nanoseconds.div_euclid(NANOSECONDS_IN_SECOND);
        let nanoseconds = nanoseconds.rem_euclid(NANOSECONDS_IN_SECOND);
        // A `Duration` spans at most ±32,768 centuries, about ±10¹⁴ seconds.
        TimeDelta::new(
            seconds as i64,
            nanoseconds as i64 * ATTOSECONDS_IN_NANOSECOND,
        )
    }
}

#[cfg(test)]
mod tests {
    use hifitime::Unit;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(TimeDelta::ZERO, Duration::ZERO)]
    #[case(TimeDelta::from_seconds(1), Unit::Second * 1)]
    #[case(TimeDelta::from_seconds(-1), Unit::Second * -1)]
    #[case(TimeDelta::from_milliseconds(1_500), Unit::Millisecond * 1_500)]
    #[case(TimeDelta::from_milliseconds(-1_500), Unit::Millisecond * -1_500)]
    #[case(
        TimeDelta::from_nanoseconds(-123_456_789),
        Duration::from_total_nanoseconds(-123_456_789)
    )]
    #[case(TimeDelta::from_days(36_525), Unit::Century * 1)]
    #[case(TimeDelta::from_days(-36_525), Unit::Century * -1)]
    #[case(TimeDelta::from_days(32_768 * 36_525), Duration::MAX)]
    #[case(TimeDelta::from_days(-32_768 * 36_525), Duration::MIN)]
    fn test_hifitime_duration(#[case] delta: TimeDelta, #[case] duration: Duration) {
        assert_eq!(Duration::try_from(delta).unwrap(), duration);
        assert_eq!(TimeDelta::from(duration), delta);
    }

    #[rstest]
    #[case(
        TimeDelta::from_attoseconds(1_999_999_999),
        Duration::from_total_nanoseconds(1)
    )]
    #[case(
        TimeDelta::from_attoseconds(-1),
        Duration::from_total_nanoseconds(-1)
    )]
    #[case(
        TimeDelta::from_attoseconds(-1_000_000_001),
        Duration::from_total_nanoseconds(-2)
    )]
    fn test_hifitime_duration_rounds_down_to_nanoseconds(
        #[case] delta: TimeDelta,
        #[case] duration: Duration,
    ) {
        assert_eq!(Duration::try_from(delta).unwrap(), duration);
    }

    #[rstest]
    #[case(TimeDelta::from_days(32_768 * 36_525) + TimeDelta::from_nanoseconds(1))]
    #[case(TimeDelta::from_days(-32_768 * 36_525) - TimeDelta::from_attoseconds(1))]
    #[case(TimeDelta::new(i64::MAX, 0))]
    #[case(TimeDelta::new(i64::MIN, 0))]
    fn test_hifitime_duration_out_of_range(#[case] delta: TimeDelta) {
        assert_eq!(
            Duration::try_from(delta),
            Err(ToHifitimeDurationError(delta))
        );
    }
}
