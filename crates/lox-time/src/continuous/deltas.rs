/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::Subsecond;
use num::ToPrimitive;
use std::ops::Neg;

use lox_utils::debug_panic;

use crate::constants::f64;

/// A signed, continuous time difference supporting femtosecond precision.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TimeDelta {
    // Like `BaseTime`, the sign of the delta is determined by the sign of the `seconds` field.
    pub seconds: i64,

    // The positive subsecond since the last whole second. For example, a delta of -0.25 s would be
    // represented as
    // ```
    // let delta = TimeDelta {
    //     seconds: -1,
    //     femtoseconds: Subsecond(0.75),
    // }
    // ```
    pub subsecond: Subsecond,
}

impl TimeDelta {
    pub fn new(seconds: i64, subsecond: Subsecond) -> Self {
        Self { seconds, subsecond }
    }

    pub fn from_seconds(seconds: i64) -> Self {
        Self {
            seconds,
            subsecond: Subsecond::default(),
        }
    }

    /// Create a [TimeDelta] from a floating-point number of seconds.
    ///
    /// As the magnitude of the input's significand grows, the precision of the resulting
    /// `TimeDelta` falls. Applications requiring precision guarantees should use `TimeDelta::new`
    /// instead.
    pub fn from_decimal_seconds(value: f64) -> Self {
        if value.is_nan() {
            debug_panic!(
                "TimeDelta seconds component was NaN, which will be set to zero in production"
            );
            return Self::default();
        }
        if value >= (i64::MAX as f64) {
            debug_panic!(
                "TimeDelta seconds component was >= i64::MAX, which will saturate in production"
            );
            return TimeDelta {
                seconds: i64::MAX,
                subsecond: Subsecond(value.fract()),
            };
        }
        if value <= (i64::MIN as f64) {
            debug_panic!(
                "TimeDelta seconds component was <= i64::MIN, which will saturate in production"
            );
            return TimeDelta {
                seconds: i64::MIN,
                subsecond: Subsecond(value.fract()),
            };
        }

        if value.is_sign_negative() {
            Self {
                seconds: (value as i64) - 1,
                subsecond: Subsecond(1.0 + value.fract()),
            }
        } else {
            Self {
                seconds: value as i64,
                subsecond: Subsecond(value.fract()),
            }
        }
    }

    /// Create a [TimeDelta] from a floating-point number of minutes.
    ///
    /// As the magnitude of the input's significand grows, the resolution of the resulting
    /// `TimeDelta` falls. Applications requiring precision guarantees should use `TimeDelta::new`
    /// instead.
    pub fn from_minutes(value: f64) -> Self {
        Self::from_decimal_seconds(value * f64::SECONDS_PER_MINUTE)
    }

    /// Create a [TimeDelta] from a floating-point number of hours.
    ///
    /// As the magnitude of the input's significand grows, the resolution of the resulting
    /// `TimeDelta` falls. Applications requiring precision guarantees should use `TimeDelta::new`
    /// instead.
    pub fn from_hours(value: f64) -> Self {
        Self::from_decimal_seconds(value * f64::SECONDS_PER_HOUR)
    }

    /// Create a [TimeDelta] from a floating-point number of days.
    ///
    /// As the magnitude of the input's significand grows, the resolution of the resulting
    /// `TimeDelta` falls. Applications requiring precision guarantees should use `TimeDelta::new`
    /// instead.
    pub fn from_days(value: f64) -> Self {
        Self::from_decimal_seconds(value * f64::SECONDS_PER_DAY)
    }

    /// Create a [TimeDelta] from a floating-point number of Julian years.
    ///
    /// As the magnitude of the input's significand grows, the resolution of the resulting
    /// `TimeDelta` falls. Applications requiring precision guarantees should use `TimeDelta::new`
    /// instead.
    pub fn from_julian_years(value: f64) -> Self {
        Self::from_decimal_seconds(value * f64::SECONDS_PER_JULIAN_YEAR)
    }

    /// Create a [TimeDelta] from a floating-point number of Julian centuries.
    ///
    /// As the magnitude of the input's significand grows, the resolution of the resulting
    /// `TimeDelta` falls. Applications requiring precision guarantees should use `TimeDelta::new`
    /// instead.
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
    fn test_from_seconds() {
        let dt = TimeDelta::from_seconds(60);
        let expected = TimeDelta {
            seconds: 60,
            subsecond: Subsecond::default(),
        };
        assert_eq!(expected, dt);
    }

    #[rstest]
    #[case::simple(0.2, TimeDelta { seconds: 0, subsecond: Subsecond(0.2) })]
    #[case::no_fraction(60.0, TimeDelta { seconds: 60, subsecond: Subsecond::default() })]
    #[case::loss_of_precision(60.3, TimeDelta { seconds: 60, subsecond: Subsecond(0.29999999999999716) })]
    fn test_decimal_seconds(#[case] seconds: f64, #[case] expected: TimeDelta) {
        let actual = TimeDelta::from_decimal_seconds(seconds);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[should_panic(expected = "saturate in production")]
    #[case::positive_infinity(f64::INFINITY)]
    #[should_panic(expected = "saturate in production")]
    #[case::negative_infinity(f64::NEG_INFINITY)]
    #[should_panic(expected = "saturate in production")]
    #[case::greater_than_i64_max(i64::MAX as f64 + 1.0)]
    #[should_panic(expected = "saturate in production")]
    #[case::less_than_i64_min(i64::MIN as f64 - 1.0)]
    #[should_panic(expected = "zero in production")]
    #[case::nan(f64::NAN)]
    fn test_from_decimal_seconds_panics(#[case] value: f64) {
        TimeDelta::from_decimal_seconds(value);
    }

    #[test]
    fn test_from_minutes() {
        let dt = TimeDelta::from_minutes(1.0);
        let expected = TimeDelta {
            seconds: 60,
            subsecond: Subsecond::default(),
        };
        assert_eq!(expected, dt);
    }

    #[test]
    fn test_from_hours() {
        let dt = TimeDelta::from_hours(1.0);
        let expected = TimeDelta {
            seconds: 3600,
            subsecond: Subsecond::default(),
        };
        assert_eq!(expected, dt);
    }

    #[test]
    fn test_from_days() {
        let dt = TimeDelta::from_days(1.0);
        let expected = TimeDelta {
            seconds: 86400,
            subsecond: Subsecond::default(),
        };
        assert_eq!(expected, dt);
    }

    #[test]
    fn test_from_years() {
        let dt = TimeDelta::from_julian_years(1.0);
        let expected = TimeDelta {
            seconds: 31557600,
            subsecond: Subsecond::default(),
        };
        assert_eq!(expected, dt);
    }

    #[test]
    fn test_from_centuries() {
        let dt = TimeDelta::from_julian_centuries(1.0);
        let expected = TimeDelta {
            seconds: 3155760000,
            subsecond: Subsecond::default(),
        };
        assert_eq!(expected, dt);
    }

    #[test]
    fn test_second() {
        let dt = TimeDelta {
            seconds: 1,
            subsecond: Subsecond::default(),
        };
        assert_eq!(1, dt.seconds);
    }

    #[test]
    fn test_subsecond() {
        let dt = TimeDelta {
            seconds: 0,
            subsecond: Subsecond(0.3),
        };
        assert_eq!(Subsecond(0.3), dt.subsecond);
    }

    proptest! {
        #[test]
        fn prop_seconds_roundtrip(s in 0.0..i64::MAX as f64) {
            let exp = if s < f64::SECONDS_PER_FEMTOSECOND {
                0.0
            } else {
                s
            };
            let delta = TimeDelta::from_decimal_seconds(s);
            if s > 1.0 {
                assert_float_eq!(delta.to_decimal_seconds(), exp, rel <= 1e-15, "input {} was not roundtrippable", s);
            } else {
                assert_float_eq!(delta.to_decimal_seconds(), exp, abs <= 1e-15, "input {} was not roundtrippable", s);
            }
        }
    }

    #[rstest]
    #[case::positive(TimeDelta { seconds: 1, subsecond: Subsecond(0.0) }, true)]
    #[case::negative(TimeDelta { seconds: -1, subsecond: Subsecond(0.0) }, false)]
    #[case::zero(TimeDelta { seconds: 0, subsecond: Subsecond(0.0) }, false)]
    fn test_is_positive(#[case] delta: TimeDelta, #[case] expected: bool) {
        assert_eq!(expected, delta.is_positive());
    }

    #[rstest]
    #[case::positive(TimeDelta { seconds: 1, subsecond: Subsecond(0.0) }, false)]
    #[case::negative(TimeDelta { seconds: -1, subsecond: Subsecond(0.0) }, true)]
    #[case::zero(TimeDelta { seconds: 0, subsecond: Subsecond(0.0) }, false)]
    fn test_is_negative(#[case] delta: TimeDelta, #[case] expected: bool) {
        assert_eq!(expected, delta.is_negative());
    }

    #[rstest]
    #[case::positive(TimeDelta { seconds: 1, subsecond: Subsecond(0.0) }, false)]
    #[case::negative(TimeDelta { seconds: -1, subsecond: Subsecond(0.0) }, false)]
    #[case::zero(TimeDelta { seconds: 0, subsecond: Subsecond(0.0) }, true)]
    fn test_is_zero(#[case] delta: TimeDelta, #[case] expected: bool) {
        assert_eq!(expected, delta.is_zero());
    }

    #[rstest]
    #[case::zero_subsecond(TimeDelta { seconds: 1, subsecond: Subsecond(0.0) }, TimeDelta { seconds: -1, subsecond: Subsecond(0.0) })]
    #[case::nonzero_subsecond(TimeDelta { seconds: 0, subsecond: Subsecond(0.3) }, TimeDelta { seconds: -1, subsecond: Subsecond(0.7) })]
    fn test_time_delta_neg(#[case] delta: TimeDelta, #[case] expected: TimeDelta) {
        assert_eq!(expected, -delta);
    }
}
