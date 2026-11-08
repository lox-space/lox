// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use thiserror::Error;

use crate::time::{TimeDelta, constants::ATTOSECONDS_IN_NANOSECOND};

/// Error returned when a [`TimeDelta`] cannot be represented as a `chrono::TimeDelta`.
#[derive(Debug, Error)]
#[error("{0} cannot be represented as a `chrono::TimeDelta`")]
pub struct ChronoTimeDeltaError(pub TimeDelta);

impl TryFrom<TimeDelta> for chrono::TimeDelta {
    type Error = ChronoTimeDeltaError;

    /// Converts the duration, rounding it down to whole nanoseconds.
    fn try_from(delta: TimeDelta) -> Result<Self, Self::Error> {
        let nanos = (delta.attoseconds() / ATTOSECONDS_IN_NANOSECOND) as u32;
        chrono::TimeDelta::new(delta.seconds(), nanos).ok_or(ChronoTimeDeltaError(delta))
    }
}

impl From<chrono::TimeDelta> for TimeDelta {
    fn from(delta: chrono::TimeDelta) -> Self {
        // `subsec_nanos` has the sign of the duration; `TimeDelta::new` normalises it.
        TimeDelta::new(
            delta.num_seconds(),
            delta.subsec_nanos() as i64 * ATTOSECONDS_IN_NANOSECOND,
        )
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(TimeDelta::ZERO, chrono::TimeDelta::zero())]
    #[case(TimeDelta::from_seconds(1), chrono::TimeDelta::seconds(1))]
    #[case(TimeDelta::from_seconds(-1), chrono::TimeDelta::seconds(-1))]
    #[case(
        TimeDelta::from_milliseconds(1_500),
        chrono::TimeDelta::milliseconds(1_500)
    )]
    #[case(TimeDelta::from_milliseconds(-1_500), chrono::TimeDelta::milliseconds(-1_500))]
    #[case(
        TimeDelta::from_nanoseconds(123_456_789),
        chrono::TimeDelta::nanoseconds(123_456_789)
    )]
    fn test_chrono_time_delta(#[case] delta: TimeDelta, #[case] chrono: chrono::TimeDelta) {
        assert_eq!(chrono::TimeDelta::try_from(delta).unwrap(), chrono);
        assert_eq!(TimeDelta::from(chrono), delta);
    }

    #[test]
    fn test_chrono_time_delta_rounds_down_to_nanoseconds() {
        let delta = TimeDelta::from_attoseconds(1_999_999_999);
        assert_eq!(
            chrono::TimeDelta::try_from(delta).unwrap(),
            chrono::TimeDelta::nanoseconds(1)
        );
    }

    #[test]
    fn test_chrono_time_delta_out_of_range() {
        assert!(chrono::TimeDelta::try_from(TimeDelta::from_seconds(i64::MAX)).is_err());
    }
}
