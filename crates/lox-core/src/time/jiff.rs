// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use jiff::SignedDuration;
use thiserror::Error;

use crate::time::constants::ATTOSECONDS_IN_NANOSECOND;
use crate::time::{TimeDelta, checked_from_seconds_and_signed_nanos};

/// Error returned when a `jiff::SignedDuration` is less than `i64::MIN` seconds
/// and therefore cannot be represented as a [`TimeDelta`].
#[derive(Debug, Error, PartialEq, Eq)]
#[error("{0:?} cannot be represented as a `TimeDelta`")]
pub struct FromJiffDurationError(pub SignedDuration);

impl TryFrom<SignedDuration> for TimeDelta {
    type Error = FromJiffDurationError;

    fn try_from(duration: SignedDuration) -> Result<Self, Self::Error> {
        checked_from_seconds_and_signed_nanos(duration.as_secs(), duration.subsec_nanos())
            .ok_or(FromJiffDurationError(duration))
    }
}

impl From<TimeDelta> for SignedDuration {
    /// Converts the duration, rounding it down to whole nanoseconds.
    fn from(delta: TimeDelta) -> Self {
        let nanos = (delta.attoseconds() / ATTOSECONDS_IN_NANOSECOND) as i32;
        SignedDuration::new(delta.seconds(), nanos)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(TimeDelta::ZERO, SignedDuration::ZERO)]
    #[case(TimeDelta::from_seconds(1), SignedDuration::from_secs(1))]
    #[case(TimeDelta::from_seconds(-1), SignedDuration::from_secs(-1))]
    #[case(
        TimeDelta::from_milliseconds(1_500),
        SignedDuration::from_millis(1_500)
    )]
    #[case(TimeDelta::from_milliseconds(-1_500), SignedDuration::from_millis(-1_500))]
    #[case(
        TimeDelta::from_nanoseconds(-123_456_789),
        SignedDuration::from_nanos(-123_456_789)
    )]
    #[case(
        TimeDelta::new(i64::MAX, 999_999_999 * ATTOSECONDS_IN_NANOSECOND),
        SignedDuration::MAX
    )]
    #[case(TimeDelta::new(i64::MIN, 0), SignedDuration::new(i64::MIN, 0))]
    fn test_jiff_signed_duration(#[case] delta: TimeDelta, #[case] duration: SignedDuration) {
        assert_eq!(SignedDuration::from(delta), duration);
        assert_eq!(TimeDelta::try_from(duration).unwrap(), delta);
    }

    #[rstest]
    #[case(
        TimeDelta::from_attoseconds(1_999_999_999),
        SignedDuration::from_nanos(1)
    )]
    #[case(TimeDelta::from_attoseconds(-1), SignedDuration::from_nanos(-1))]
    #[case(TimeDelta::from_attoseconds(-1_000_000_001), SignedDuration::from_nanos(-2))]
    fn test_jiff_signed_duration_rounds_down_to_nanoseconds(
        #[case] delta: TimeDelta,
        #[case] duration: SignedDuration,
    ) {
        assert_eq!(SignedDuration::from(delta), duration);
    }

    #[rstest]
    #[case(SignedDuration::MIN)]
    #[case(SignedDuration::new(i64::MIN, -1))]
    fn test_jiff_signed_duration_out_of_range(#[case] duration: SignedDuration) {
        assert_eq!(
            TimeDelta::try_from(duration),
            Err(FromJiffDurationError(duration))
        );
    }
}
