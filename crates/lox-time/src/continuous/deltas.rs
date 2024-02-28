/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::debug_panic;
use num::ToPrimitive;

use crate::constants::f64;
use crate::constants::u128;

/// An absolute continuous time difference with femtosecond precision.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TimeDelta {
    pub seconds: u64,
    pub femtoseconds: u64,
}

impl TimeDelta {
    pub fn from_seconds(seconds: u64) -> Self {
        Self {
            seconds,
            femtoseconds: 0,
        }
    }

    pub fn from_decimal_seconds(value: f64) -> Self {
        if value < 0.0 {
            debug_panic!(
                "TimeDelta seconds component was negative, which will be set to zero in production"
            );
            return Self::default();
        }
        if value.is_nan() {
            debug_panic!(
                "TimeDelta seconds component was NaN, which will be set to zero in production"
            );
            return Self::default();
        }
        if value > u64::MAX as f64 {
            debug_panic!(
                "TimeDelta seconds component exceeds u64::MAX, which will saturate in production"
            );
            return Self {
                seconds: u64::MAX,
                femtoseconds: 0,
            };
        }
        if value < f64::SECONDS_PER_FEMTOSECOND {
            return Self::default();
        }
        if value.fract() == 0.0 {
            let seconds = value.to_u64().unwrap();
            return Self {
                seconds,
                femtoseconds: 0,
            };
        }
        let value = (value * f64::FEMTOSECONDS_PER_SECOND).to_u128().unwrap();
        let seconds = value / u128::FEMTOSECONDS_PER_SECOND;
        let femtoseconds = value - seconds * u128::FEMTOSECONDS_PER_SECOND;
        let seconds = seconds.to_u64().unwrap();
        let femtoseconds = femtoseconds.to_u64().unwrap();
        Self {
            seconds,
            femtoseconds,
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
        self.femtoseconds.to_f64().unwrap() / f64::FEMTOSECONDS_PER_SECOND
            + self.seconds.to_f64().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn test_seconds() {
        let dt = TimeDelta::from_seconds(60);
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.femtoseconds, 0);
    }

    #[test]
    fn test_decimal_seconds() {
        let dt = TimeDelta::from_decimal_seconds(60.3);
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.femtoseconds, 3 * 10u64.pow(14));
    }

    #[test]
    fn test_decimal_seconds_without_fraction() {
        let dt = TimeDelta::from_decimal_seconds(60.0);
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.femtoseconds, 0);
    }

    #[test]
    fn test_decimal_seconds_below_resolution() {
        let dt = TimeDelta::from_decimal_seconds(1e-18);
        assert_eq!(dt.seconds, 0);
        assert_eq!(dt.femtoseconds, 0);
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
        assert_eq!(dt.femtoseconds, 0);
    }

    #[test]
    fn test_hours() {
        let dt = TimeDelta::from_hours(1.0);
        assert_eq!(dt.seconds, 3600);
        assert_eq!(dt.femtoseconds, 0);
    }

    #[test]
    fn test_days() {
        let dt = TimeDelta::from_days(1.0);
        assert_eq!(dt.seconds, 86400);
        assert_eq!(dt.femtoseconds, 0);
    }

    #[test]
    fn test_years() {
        let dt = TimeDelta::from_julian_years(1.0);
        assert_eq!(dt.seconds, 31557600);
        assert_eq!(dt.femtoseconds, 0);
    }

    #[test]
    fn test_centuries() {
        let dt = TimeDelta::from_julian_centuries(1.0);
        assert_eq!(dt.seconds, 3155760000);
        assert_eq!(dt.femtoseconds, 0);
    }

    #[test]
    fn test_attosecond() {
        let dt = TimeDelta::from_decimal_seconds(f64::SECONDS_PER_FEMTOSECOND);
        assert_eq!(dt.seconds, 0);
        assert_eq!(dt.femtoseconds, 1);
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
}
