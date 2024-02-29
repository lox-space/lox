/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::ops::Neg;
use crate::{debug_panic, Subsecond};
use num::ToPrimitive;

use crate::constants::f64;

/// A signed, continuous time difference with at least femtosecond precision.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TimeDelta {
    // Like `BaseTime`, the sign of the delta is determined by the sign of the `seconds` field.
    pub seconds: i64,

    // The positive subseconds since the last whole second. For example, a delta of -0.5 s would be
    // represented as
    // ```
    // let delta = TimeDelta {
    //     seconds: -1,
    //     femtoseconds: Subsecond(0.5),
    // }
    // ```
    pub subsecond: Subsecond,
}

impl TimeDelta {
    pub fn from_seconds(seconds: i64) -> Self {
        Self {
            seconds,
            subsecond: Subsecond::default(),
        }
    }

    pub fn from_decimal_seconds(value: f64) -> Self {
        if value.is_nan() {
            debug_panic!(
                "TimeDelta seconds component was NaN, which will be set to zero in production"
            );
            return Self::default();
        }
        if value.is_infinite() {
            debug_panic!(
                "TimeDelta seconds component was infinite, which will return a TimeDelta with a saturated `seconds` component in production"
            );
            return if value.is_sign_positive() {
                TimeDelta {
                    seconds: i64::MAX,
                    subsecond: Subsecond::default(),
                }
            } else {
                TimeDelta {
                    seconds: i64::MIN,
                    subsecond: Subsecond::default(),
                }
            };
        }

        Self {
            seconds: value.trunc() as i64,
            subsecond: Subsecond(value.fract()),
        }
    }

    pub fn from_minutes(value: f64) -> Self {
        Self::from_decimal_seconds(value * f64::SECONDS_PER_MINUTE)
    }

    pub fn from_hours(value: f64) -> Self {
        Self::from_decimal_seconds(value * f64::SECONDS_PER_HOUR)
    }

    pub fn from_days(value: f64) -> Self {
        Self::from_decimal_seconds(value * f64::SECONDS_PER_DAY)
    }

    pub fn from_julian_years(value: f64) -> Self {
        Self::from_decimal_seconds(value * f64::SECONDS_PER_JULIAN_YEAR)
    }

    pub fn from_julian_centuries(value: f64) -> Self {
        Self::from_decimal_seconds(value * f64::SECONDS_PER_JULIAN_CENTURY)
    }

    pub fn to_decimal_seconds(&self) -> f64 {
        self.subsecond.0 + self.seconds.to_f64().unwrap()
    }
    
    pub fn is_negative(&self) -> bool {
        self.seconds < 0
    }
    
    pub fn is_zero(&self) -> bool {
        self.seconds == 0 && self.subsecond.0 == 0.0
    }
    
    pub fn is_positive(&self) -> bool {
        self.seconds > 0 || self.seconds == 0 && self.subsecond.0 > 0.0
    }
}

impl Neg for TimeDelta {
    type Output = Self;

    fn neg(self) -> Self::Output {
        if self.subsecond.0 == 0.0 {
            return Self {
                seconds: -self.seconds,
                subsecond: Subsecond::default(),
            };
        }
        
        Self {
            seconds: -self.seconds - 1,
            subsecond: Subsecond(1.0 - self.subsecond.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;

    #[test]
    fn test_seconds() {
        let dt = TimeDelta::from_seconds(60);
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.subsecond, Subsecond::default());
    }

    #[test]
    fn test_decimal_seconds() {
        let dt = TimeDelta::from_decimal_seconds(60.3);
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.subsecond, Subsecond(0.3));
    }

    #[test]
    fn test_decimal_seconds_without_fraction() {
        let dt = TimeDelta::from_decimal_seconds(60.0);
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.subsecond, Subsecond::default());
    }

    #[test]
    fn test_decimal_seconds_below_resolution() {
        let dt = TimeDelta::from_decimal_seconds(1e-18);
        assert_eq!(dt.seconds, 0);
        assert_eq!(dt.subsecond, Subsecond::default());
    }

    #[test]
    #[should_panic(expected = "saturate in production")]
    fn test_decimal_seconds_exceeds_max() {
        TimeDelta::from_decimal_seconds(f64::MAX);
    }

    #[test]
    #[should_panic(expected = "zero in production")]
    fn test_decimal_seconds_is_nan() {
        TimeDelta::from_decimal_seconds(f64::NAN);
    }

    #[test]
    fn test_minutes() {
        let dt = TimeDelta::from_minutes(1.0);
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.subsecond, Subsecond::default());
    }

    #[test]
    fn test_hours() {
        let dt = TimeDelta::from_hours(1.0);
        assert_eq!(dt.seconds, 3600);
        assert_eq!(dt.subsecond, Subsecond::default());
    }

    #[test]
    fn test_days() {
        let dt = TimeDelta::from_days(1.0);
        assert_eq!(dt.seconds, 86400);
        assert_eq!(dt.subsecond, Subsecond::default());
    }

    #[test]
    fn test_years() {
        let dt = TimeDelta::from_julian_years(1.0);
        assert_eq!(dt.seconds, 31557600);
        assert_eq!(dt.subsecond, Subsecond::default());
    }

    #[test]
    fn test_centuries() {
        let dt = TimeDelta::from_julian_centuries(1.0);
        assert_eq!(dt.seconds, 3155760000);
        assert_eq!(dt.subsecond, Subsecond::default());
    }

    #[test]
    fn test_attosecond() {
        let dt = TimeDelta::from_decimal_seconds(f64::SECONDS_PER_FEMTOSECOND);
        assert_eq!(dt.seconds, 0);
        assert_eq!(dt.subsecond, Subsecond(0.000_000_000_000_001));
    }

    proptest! {
        #[test]
        fn prop_seconds_roundtrip(s in 0.0..u64::MAX as f64) {
            let exp = if s < f64::SECONDS_PER_FEMTOSECOND {
                0.0
            } else {
                s
            };
            let delta = TimeDelta::from_decimal_seconds(s);
            if s > 1.0 {
                assert_float_eq!(delta.to_decimal_seconds(), exp, rel <= 1e-15);
            } else {
                assert_float_eq!(delta.to_decimal_seconds(), exp, abs <= 1e-15);
            }
        }
    }
    
    #[rstest]
    #[case::zero_subsecond(TimeDelta { seconds: 1, subsecond: Subsecond(0.0) }, TimeDelta { seconds: -1, subsecond: Subsecond(0.0) })]
    #[case::nonzero_subsecond(TimeDelta { seconds: 0, subsecond: Subsecond(0.3) }, TimeDelta { seconds: -1, subsecond: Subsecond(0.7) })]
    fn test_time_delta_neg(#[case] delta: TimeDelta, #[case] expected: TimeDelta) {
        assert_eq!(expected, -delta);
    }
}
