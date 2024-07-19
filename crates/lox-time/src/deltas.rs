/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

/*!
    Module `deltas` contains [TimeDelta], the key primitive of the `lox-time` crate.

    [TimeDelta] is a signed, unscaled delta relative to an arbitrary epoch. This forms the basis
    of instants in all continuous time scales.

    The [ToDelta] trait specifies the method by which such scaled time representations should
    expose their underlying [TimeDelta].
*/

use std::fmt::Display;
use std::ops::{Add, Neg, RangeInclusive, Sub};

use num::ToPrimitive;
use thiserror::Error;

use lox_math::constants::f64::time::{
    SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_JULIAN_CENTURY, SECONDS_PER_JULIAN_YEAR,
    SECONDS_PER_MINUTE,
};

use crate::ranges::TimeDeltaRange;
use crate::{
    constants::julian_dates::{
        SECONDS_BETWEEN_J1950_AND_J2000, SECONDS_BETWEEN_JD_AND_J2000,
        SECONDS_BETWEEN_MJD_AND_J2000,
    },
    julian_dates::{Epoch, JulianDate, Unit},
    subsecond::Subsecond,
};

/// A unifying trait for types that can be converted into a [TimeDelta].
pub trait ToDelta {
    /// Transforms the value into a [TimeDelta].
    fn to_delta(&self) -> TimeDelta;
}

/// Error type returned when attempting to construct a [TimeDelta] from an invalid `f64`.
#[derive(Clone, Debug, Default, Error)]
#[error("`{raw}` cannot be represented as a `TimeDelta`: {detail}")]
pub struct TimeDeltaError {
    pub raw: f64,
    pub detail: String,
}

// Manual implementation of PartialEq to handle NaNs, which are not equal to themselves, but
// errors resulting from NaNs should be.
impl PartialEq for TimeDeltaError {
    fn eq(&self, other: &Self) -> bool {
        self.detail == other.detail
            && (self.raw.is_nan() && other.raw.is_nan() || self.raw == other.raw)
    }
}

/// A signed, continuous time difference supporting femtosecond precision.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TimeDelta {
    // The sign of the delta is determined by the sign of the `seconds` field.
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
    /// Construct a new [TimeDelta] from a number of seconds and a [Subsecond].
    pub fn new(seconds: i64, subsecond: Subsecond) -> Self {
        Self { seconds, subsecond }
    }

    /// Construct a [TimeDelta] from an integral number of seconds.
    pub fn from_seconds(seconds: i64) -> Self {
        Self {
            seconds,
            subsecond: Subsecond::default(),
        }
    }

    /// Construct a [TimeDelta] from a floating-point number of seconds.
    ///
    /// As the magnitude of the input's significand grows, the precision of the resulting
    /// `TimeDelta` falls. Applications requiring precision guarantees should use [TimeDelta::new]
    /// instead.
    ///
    /// # Errors
    ///
    /// - [TimeDeltaError] if the input is NaN or ±infinity.
    pub fn from_decimal_seconds(value: f64) -> Result<Self, TimeDeltaError> {
        if value.is_nan() {
            return Err(TimeDeltaError {
                raw: value,
                detail: "NaN is unrepresentable".to_string(),
            });
        }
        if value >= (i64::MAX as f64) {
            return Err(TimeDeltaError {
                raw: value,
                detail: "input seconds cannot exceed the maximum value of an i64".to_string(),
            });
        }
        if value <= (i64::MIN as f64) {
            return Err(TimeDeltaError {
                raw: value,
                detail: "input seconds cannot be less than the minimum value of an i64".to_string(),
            });
        }

        let result = if value.is_sign_negative() {
            if value.fract() == 0.0 {
                Self {
                    seconds: value as i64,
                    subsecond: Subsecond::default(),
                }
            } else {
                Self {
                    seconds: (value as i64) - 1,
                    subsecond: Subsecond(1.0 + value.fract()),
                }
            }
        } else {
            Self {
                seconds: value as i64,
                subsecond: Subsecond(value.fract()),
            }
        };

        Ok(result)
    }

    /// Construct a [TimeDelta] from a floating-point number of minutes.
    ///
    /// As the magnitude of the input's significand grows, the resolution of the resulting
    /// `TimeDelta` falls. Applications requiring precision guarantees should use [TimeDelta::new]
    /// instead.
    ///
    /// # Errors
    ///
    /// - [TimeDeltaError] if the input is NaN or ±infinity.
    pub fn from_minutes(value: f64) -> Result<Self, TimeDeltaError> {
        Self::from_decimal_seconds(value * SECONDS_PER_MINUTE)
    }

    /// Construct a [TimeDelta] from a floating-point number of hours.
    ///
    /// As the magnitude of the input's significand grows, the resolution of the resulting
    /// `TimeDelta` falls. Applications requiring precision guarantees should use [TimeDelta::new]
    /// instead.
    ///
    /// # Errors
    ///
    /// - [TimeDeltaError] if the input is NaN or ±infinity.
    pub fn from_hours(value: f64) -> Result<Self, TimeDeltaError> {
        Self::from_decimal_seconds(value * SECONDS_PER_HOUR)
    }

    /// Construct a [TimeDelta] from a floating-point number of days.
    ///
    /// As the magnitude of the input's significand grows, the resolution of the resulting
    /// `TimeDelta` falls. Applications requiring precision guarantees should use [TimeDelta::new]
    /// instead.
    ///
    /// # Errors
    ///
    /// - [TimeDeltaError] if the input is NaN or ±infinity.
    pub fn from_days(value: f64) -> Result<Self, TimeDeltaError> {
        Self::from_decimal_seconds(value * SECONDS_PER_DAY)
    }

    /// Construct a [TimeDelta] from a floating-point number of Julian years.
    ///
    /// As the magnitude of the input's significand grows, the resolution of the resulting
    /// `TimeDelta` falls. Applications requiring precision guarantees should use [TimeDelta::new]
    /// instead.
    ///
    /// # Errors
    ///
    /// - [TimeDeltaError] if the input is NaN or ±infinity.
    pub fn from_julian_years(value: f64) -> Result<Self, TimeDeltaError> {
        Self::from_decimal_seconds(value * SECONDS_PER_JULIAN_YEAR)
    }

    /// Construct a [TimeDelta] from a floating-point number of Julian centuries.
    ///
    /// As the magnitude of the input's significand grows, the resolution of the resulting
    /// `TimeDelta` falls. Applications requiring precision guarantees should use [TimeDelta::new]
    /// instead.
    ///
    /// # Errors
    ///
    /// - [TimeDeltaError] if the input is NaN or ±infinity.
    pub fn from_julian_centuries(value: f64) -> Result<Self, TimeDeltaError> {
        Self::from_decimal_seconds(value * SECONDS_PER_JULIAN_CENTURY)
    }

    /// Express `&self` as a floating-point number of seconds, with potential loss of precision.
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

    /// Scale the [TimeDelta] by `factor`, with possible loss of precision.
    pub fn scale(mut self, mut factor: f64) -> Self {
        // Treating both `Self` and `factor` as positive and then correcting the sign at the end
        // substantially simplifies the implementation.
        let mut sign = 1;
        if self.is_negative() {
            if factor.is_sign_negative() {
                self = -self;
                factor = factor.abs();
            } else {
                self = -self;
                sign = -sign;
            }
        } else if self.is_positive() && factor.is_sign_negative() {
            sign = -sign;
            factor = factor.abs();
        }

        // Various accuracy-preserving optimisations for floating-point algebra appear to have
        // no effect on the result of this function for the expected inputs. This is possibly
        // because we rarely scale beyond one order of magnitude's difference.
        let seconds_f64 = self.seconds as f64;
        let mut scaled_seconds = seconds_f64 * factor;
        let mut scaled_subsecond = self.subsecond.0.mul_add(factor, scaled_seconds.fract());
        if scaled_subsecond >= 1.0 {
            scaled_subsecond = scaled_subsecond.fract();
            scaled_seconds += scaled_subsecond.trunc();
        }

        let result = Self {
            seconds: scaled_seconds
                .to_i64()
                .expect("scaled seconds field was not representable as an i64"),
            subsecond: Subsecond(scaled_subsecond),
        };

        if sign < 0 {
            -result
        } else {
            result
        }
    }

    /// Express the [TimeDelta] as an integral number of seconds since the given [Epoch].
    pub fn seconds_from_epoch(&self, epoch: Epoch) -> i64 {
        match epoch {
            Epoch::JulianDate => self.seconds + SECONDS_BETWEEN_JD_AND_J2000,
            Epoch::ModifiedJulianDate => self.seconds + SECONDS_BETWEEN_MJD_AND_J2000,
            Epoch::J1950 => self.seconds + SECONDS_BETWEEN_J1950_AND_J2000,
            Epoch::J2000 => self.seconds,
        }
    }

    pub fn range(range: RangeInclusive<i64>) -> TimeDeltaRange {
        range.into()
    }
}

impl Display for TimeDelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} s", self.to_decimal_seconds())
    }
}

impl JulianDate for TimeDelta {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        let mut decimal_seconds = self.seconds_from_epoch(epoch).to_f64().unwrap();
        decimal_seconds += self.subsecond.0;
        match unit {
            Unit::Seconds => decimal_seconds,
            Unit::Days => decimal_seconds / SECONDS_PER_DAY,
            Unit::Centuries => decimal_seconds / SECONDS_PER_JULIAN_CENTURY,
        }
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

impl Add for TimeDelta {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if rhs.is_negative() {
            return self - (-rhs);
        }

        let mut sum_seconds = self.seconds + rhs.seconds;
        let mut sum_subsecond = self.subsecond.0 + rhs.subsecond.0;
        if sum_subsecond >= 1.0 {
            sum_subsecond = sum_subsecond.fract();
            sum_seconds += 1;
        }
        Self {
            seconds: sum_seconds,
            subsecond: Subsecond(sum_subsecond),
        }
    }
}

impl Sub for TimeDelta {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if rhs.is_negative() {
            return self + (-rhs);
        }

        let mut diff_seconds = self.seconds - rhs.seconds;
        let mut diff_subsecond = self.subsecond.0 - rhs.subsecond.0;
        if diff_subsecond.abs() > f64::EPSILON && diff_subsecond < 0.0 {
            diff_subsecond += 1.0;
            diff_seconds -= 1;
        }
        Self {
            seconds: diff_seconds,
            subsecond: Subsecond(diff_subsecond),
        }
    }
}

impl From<i64> for TimeDelta {
    fn from(value: i64) -> Self {
        TimeDelta::from_seconds(value)
    }
}

impl From<i32> for TimeDelta {
    fn from(value: i32) -> Self {
        TimeDelta::from_seconds(value as i64)
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use lox_math::constants::f64::time::DAYS_PER_JULIAN_CENTURY;
    use proptest::prelude::*;
    use rstest::rstest;

    use crate::constants::f64::SECONDS_PER_FEMTOSECOND;

    use super::*;

    #[test]
    fn test_new() {
        let dt = TimeDelta::new(1, Subsecond(0.3));
        let expected = TimeDelta {
            seconds: 1,
            subsecond: Subsecond(0.3),
        };
        assert_eq!(expected, dt);
    }

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
    #[case::pos_fraction(1.2, Ok(TimeDelta { seconds: 1, subsecond: Subsecond(0.2) }))]
    #[case::neg_fraction(-1.2, Ok(TimeDelta { seconds: -2, subsecond: Subsecond(0.8) }))]
    #[case::pos_no_fraction(60.0, Ok(TimeDelta { seconds: 60, subsecond: Subsecond::default() }))]
    #[case::neg_no_fraction(-60.0, Ok(TimeDelta { seconds: -60, subsecond: Subsecond::default() }))]
    #[case::loss_of_precision(60.3, Ok(TimeDelta { seconds: 60, subsecond: Subsecond(0.29999999999999716) }))]
    #[case::nan(f64::NAN, Err(TimeDeltaError { raw: f64::NAN, detail: "NaN is unrepresentable".to_string() }))]
    #[case::greater_than_i64_max(i64::MAX as f64 + 1.0, Err(TimeDeltaError { raw: i64::MAX as f64 + 1.0, detail: "input seconds cannot exceed the maximum value of an i64".to_string() }))]
    #[case::less_than_i64_min(i64::MIN as f64 - 1.0, Err(TimeDeltaError { raw: i64::MIN as f64 - 1.0, detail: "input seconds cannot be less than the minimum value of an i64".to_string() }))]
    fn test_from_decimal_seconds(
        #[case] seconds: f64,
        #[case] expected: Result<TimeDelta, TimeDeltaError>,
    ) {
        let actual = TimeDelta::from_decimal_seconds(seconds);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_minutes() {
        let dt = TimeDelta::from_minutes(1.0);
        let expected = Ok(TimeDelta {
            seconds: 60,
            subsecond: Subsecond::default(),
        });
        assert_eq!(expected, dt);
    }

    #[test]
    fn test_from_hours() {
        let dt = TimeDelta::from_hours(1.0);
        let expected = Ok(TimeDelta {
            seconds: 3600,
            subsecond: Subsecond::default(),
        });
        assert_eq!(expected, dt);
    }

    #[test]
    fn test_from_days() {
        let dt = TimeDelta::from_days(1.0);
        let expected = Ok(TimeDelta {
            seconds: 86400,
            subsecond: Subsecond::default(),
        });
        assert_eq!(expected, dt);
    }

    #[test]
    fn test_from_years() {
        let dt = TimeDelta::from_julian_years(1.0);
        let expected = Ok(TimeDelta {
            seconds: 31557600,
            subsecond: Subsecond::default(),
        });
        assert_eq!(expected, dt);
    }

    #[test]
    fn test_from_centuries() {
        let dt = TimeDelta::from_julian_centuries(1.0);
        let expected = Ok(TimeDelta {
            seconds: 3155760000,
            subsecond: Subsecond::default(),
        });
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
            let exp = if s < SECONDS_PER_FEMTOSECOND {
                0.0
            } else {
                s
            };
            let delta = TimeDelta::from_decimal_seconds(s).unwrap();
            if s > 1.0 {
                assert_float_eq!(delta.to_decimal_seconds(), exp, rel <= 1e-15, "input {} was not roundtrippable", s);
            } else {
                assert_float_eq!(delta.to_decimal_seconds(), exp, abs <= 1e-15, "input {} was not roundtrippable", s);
            }
        }
    }

    #[rstest]
    #[case::positive(TimeDelta { seconds: 1, subsecond: Subsecond(0.0) }, true)]
    #[case::negative(TimeDelta { seconds: - 1, subsecond: Subsecond(0.0) }, false)]
    #[case::zero(TimeDelta { seconds: 0, subsecond: Subsecond(0.0) }, false)]
    fn test_is_positive(#[case] delta: TimeDelta, #[case] expected: bool) {
        assert_eq!(expected, delta.is_positive());
    }

    #[rstest]
    #[case::positive(TimeDelta { seconds: 1, subsecond: Subsecond(0.0) }, false)]
    #[case::negative(TimeDelta { seconds: - 1, subsecond: Subsecond(0.0) }, true)]
    #[case::zero(TimeDelta { seconds: 0, subsecond: Subsecond(0.0) }, false)]
    fn test_is_negative(#[case] delta: TimeDelta, #[case] expected: bool) {
        assert_eq!(expected, delta.is_negative());
    }

    #[rstest]
    #[case::positive(TimeDelta { seconds: 1, subsecond: Subsecond(0.0) }, false)]
    #[case::negative(TimeDelta { seconds: - 1, subsecond: Subsecond(0.0) }, false)]
    #[case::zero(TimeDelta { seconds: 0, subsecond: Subsecond(0.0) }, true)]
    fn test_is_zero(#[case] delta: TimeDelta, #[case] expected: bool) {
        assert_eq!(expected, delta.is_zero());
    }

    #[rstest]
    #[case::zero_subsecond(TimeDelta { seconds: 1, subsecond: Subsecond(0.0) }, TimeDelta { seconds: - 1, subsecond: Subsecond(0.0) })]
    #[case::nonzero_subsecond(TimeDelta { seconds: 0, subsecond: Subsecond(0.3) }, TimeDelta { seconds: - 1, subsecond: Subsecond(0.7) })]
    fn test_time_delta_neg(#[case] delta: TimeDelta, #[case] expected: TimeDelta) {
        assert_eq!(expected, -delta);
    }

    #[rstest]
    #[case::zero(TimeDelta { seconds: 1, subsecond: Subsecond(0.0) }, 0.0, TimeDelta { seconds: 0, subsecond: Subsecond(0.0) })]
    #[case::one(TimeDelta { seconds: 1, subsecond: Subsecond(0.0) }, 1.0, TimeDelta { seconds: 1, subsecond: Subsecond(0.0) })]
    #[case::two(TimeDelta { seconds: 1, subsecond: Subsecond(0.0) }, 2.0, TimeDelta { seconds: 2, subsecond: Subsecond(0.0) })]
    #[case::half(TimeDelta { seconds: 1, subsecond: Subsecond(0.0) }, 0.5, TimeDelta { seconds: 0, subsecond: Subsecond(0.5) })]
    #[case::pos_delta_neg_factor(TimeDelta { seconds: 0, subsecond: Subsecond(0.3) }, - 1.0, TimeDelta { seconds: - 1, subsecond: Subsecond(0.7) })]
    #[case::neg_delta_pos_factor(TimeDelta { seconds: - 1, subsecond: Subsecond(0.3) }, 1.0, TimeDelta { seconds: - 1, subsecond: Subsecond(0.3) })]
    #[case::neg_delta_neg_factor(TimeDelta { seconds: - 1, subsecond: Subsecond(0.3) }, - 1.0, TimeDelta { seconds: 0, subsecond: Subsecond(0.7) })]
    fn test_time_delta_scale(
        #[case] delta: TimeDelta,
        #[case] factor: f64,
        #[case] expected: TimeDelta,
    ) {
        assert_eq!(expected, delta.scale(factor));
    }

    #[rstest]
    #[case::zero_zero(TimeDelta { seconds: 0, subsecond: Subsecond(0.0) }, TimeDelta { seconds: 0, subsecond: Subsecond(0.0) }, TimeDelta { seconds: 0, subsecond: Subsecond(0.0) })]
    #[case::pos_lhs_pos_rhs(TimeDelta { seconds: 1, subsecond: Subsecond(0.5) }, TimeDelta { seconds: 1, subsecond: Subsecond(0.5) }, TimeDelta { seconds: 3, subsecond: Subsecond(0.0) })]
    #[case::pos_lhs_neg_rhs(TimeDelta { seconds: 1, subsecond: Subsecond(0.2) }, TimeDelta { seconds: - 1, subsecond: Subsecond(0.5) }, TimeDelta { seconds: 0, subsecond: Subsecond(0.7) })]
    #[case::neg_lhs_pos_rhs(TimeDelta { seconds: - 1, subsecond: Subsecond(0.2) }, TimeDelta { seconds: 1, subsecond: Subsecond(0.5) }, TimeDelta { seconds: 0, subsecond: Subsecond(0.7) })]
    #[case::neg_lhs_neg_rhs(TimeDelta { seconds: - 1, subsecond: Subsecond(0.5) }, TimeDelta { seconds: - 1, subsecond: Subsecond(0.5) }, TimeDelta { seconds: - 1, subsecond: Subsecond(0.0) })]
    #[case::sign_change_pos_to_neg(TimeDelta { seconds: 0, subsecond: Subsecond(0.2) }, TimeDelta { seconds: - 1, subsecond: Subsecond(0.7) }, TimeDelta { seconds: - 1, subsecond: Subsecond(0.9) })]
    #[case::sign_change_neg_to_pos(TimeDelta { seconds: - 1, subsecond: Subsecond(0.7) }, TimeDelta { seconds: 0, subsecond: Subsecond(0.5) }, TimeDelta { seconds: 0, subsecond: Subsecond(0.2) })]
    fn test_time_delta_add(
        #[case] lhs: TimeDelta,
        #[case] rhs: TimeDelta,
        #[case] expected: TimeDelta,
    ) {
        assert_eq!(expected, lhs + rhs);
    }

    #[rstest]
    #[case::zero_zero(TimeDelta { seconds: 0, subsecond: Subsecond(0.0) }, TimeDelta { seconds: 0, subsecond: Subsecond(0.0) }, TimeDelta { seconds: 0, subsecond: Subsecond(0.0) })]
    #[case::pos_lhs_pos_rhs(TimeDelta { seconds: 1, subsecond: Subsecond(0.5) }, TimeDelta { seconds: 1, subsecond: Subsecond(0.5) }, TimeDelta { seconds: 0, subsecond: Subsecond(0.0) })]
    #[case::pos_lhs_neg_rhs(TimeDelta { seconds: 1, subsecond: Subsecond(0.2) }, TimeDelta { seconds: -1, subsecond: Subsecond(0.5) }, TimeDelta { seconds: 1, subsecond: Subsecond(0.7) })]
    #[case::neg_lhs_pos_rhs(TimeDelta { seconds: -1, subsecond: Subsecond(0.2) }, TimeDelta { seconds: 1, subsecond: Subsecond(0.5) }, TimeDelta { seconds: -3, subsecond: Subsecond(0.7) })]
    #[case::neg_lhs_neg_rhs(TimeDelta { seconds: -2, subsecond: Subsecond(0.5) }, TimeDelta { seconds: -1, subsecond: Subsecond(0.5) }, TimeDelta { seconds: -1, subsecond: Subsecond(0.0) })]
    #[case::sign_change_pos_to_neg(TimeDelta { seconds: 0, subsecond: Subsecond(0.2) }, TimeDelta { seconds: 0, subsecond: Subsecond(0.3) }, TimeDelta { seconds: -1, subsecond: Subsecond(0.9) })]
    #[case::sign_change_neg_to_pos(TimeDelta { seconds: -1, subsecond: Subsecond(0.7) }, TimeDelta { seconds: -1, subsecond: Subsecond(0.5) }, TimeDelta { seconds: 0, subsecond: Subsecond(0.2) })]
    fn test_time_delta_sub(
        #[case] lhs: TimeDelta,
        #[case] rhs: TimeDelta,
        #[case] expected: TimeDelta,
    ) {
        assert_eq!(expected, lhs - rhs);
    }

    #[test]
    fn test_delta_julian_date() {
        let delta = TimeDelta::new(
            crate::constants::i64::SECONDS_PER_JULIAN_CENTURY,
            Subsecond::default(),
        );
        assert_eq!(delta.seconds_since_j2000(), SECONDS_PER_JULIAN_CENTURY);
        assert_eq!(delta.days_since_j2000(), DAYS_PER_JULIAN_CENTURY);
        assert_eq!(delta.centuries_since_j2000(), 1.0);
        assert_eq!(delta.centuries_since_j1950(), 1.5);
        assert_eq!(
            delta.centuries_since_modified_julian_epoch(),
            2.411211498973306
        );
        assert_eq!(delta.centuries_since_julian_epoch(), 68.11964407939767);
    }

    #[test]
    fn test_delta_sub_epsilon() {
        let delta0 = TimeDelta::default();
        let delta1 = TimeDelta::new(0, Subsecond(1e-17));
        let delta = delta0 - delta1;
        assert_ne!(delta.subsecond.0, 1.0)
    }

    #[test]
    fn test_delta_from_integer() {
        let delta: TimeDelta = 4i32.into();
        assert_eq!(delta.seconds, 4);
        let delta: TimeDelta = 4i64.into();
        assert_eq!(delta.seconds, 4);
    }
}
