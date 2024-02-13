/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use num::traits::Inv;
use num::ToPrimitive;

use crate::time::constants::f64;
use crate::time::constants::u128;

/// An absolute continuous time difference with femtosecond precision.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TimeDelta {
    pub seconds: u64,
    pub femtoseconds: u64,
}

impl TimeDelta {
    pub fn int_seconds(seconds: u64) -> Self {
        Self {
            seconds,
            femtoseconds: 0,
        }
    }

    pub fn seconds(value: f64) -> Self {
        if value < f64::FEMTOSECONDS_PER_SECOND.inv() || value.is_nan() {
            return Self::default();
        }
        if value > u64::MAX as f64 {
            return Self {
                seconds: u64::MAX,
                femtoseconds: 0,
            };
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

    pub fn minutes(value: f64) -> Self {
        Self::seconds(value * f64::SECONDS_PER_MINUTE)
    }

    pub fn hours(value: f64) -> Self {
        Self::seconds(value * f64::SECONDS_PER_HOUR)
    }

    pub fn days(value: f64) -> Self {
        Self::seconds(value * f64::SECONDS_PER_DAY)
    }

    pub fn years(value: f64) -> Self {
        Self::seconds(value * f64::SECONDS_PER_JULIAN_YEAR)
    }

    pub fn centuries(value: f64) -> Self {
        Self::seconds(value * f64::SECONDS_PER_JULIAN_CENTURY)
    }

    pub fn to_seconds(&self) -> f64 {
        self.femtoseconds.to_f64().unwrap() / f64::FEMTOSECONDS_PER_SECOND
            + self.seconds.to_f64().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use proptest::num::f64::ANY;
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn test_int_seconds() {
        let dt = TimeDelta::int_seconds(60);
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.femtoseconds, 0);
    }

    #[test]
    fn test_seconds() {
        let dt = TimeDelta::seconds(60.3);
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.femtoseconds, 3 * 10u64.pow(14));
    }

    #[test]
    fn test_minutes() {
        let dt = TimeDelta::minutes(1.0);
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.femtoseconds, 0);
    }

    #[test]
    fn test_hours() {
        let dt = TimeDelta::hours(1.0);
        assert_eq!(dt.seconds, 3600);
        assert_eq!(dt.femtoseconds, 0);
    }

    #[test]
    fn test_days() {
        let dt = TimeDelta::days(1.0);
        assert_eq!(dt.seconds, 86400);
        assert_eq!(dt.femtoseconds, 0);
    }

    #[test]
    fn test_years() {
        let dt = TimeDelta::years(1.0);
        assert_eq!(dt.seconds, 31557600);
        assert_eq!(dt.femtoseconds, 0);
    }

    #[test]
    fn test_centuries() {
        let dt = TimeDelta::centuries(1.0);
        assert_eq!(dt.seconds, 3155760000);
        assert_eq!(dt.femtoseconds, 0);
    }

    #[test]
    fn test_attosecond() {
        let dt = TimeDelta::seconds(f64::FEMTOSECONDS_PER_SECOND.inv());
        assert_eq!(dt.seconds, 0);
        assert_eq!(dt.femtoseconds, 1);
    }

    proptest! {
        #[test]
        fn prop_seconds_roundtrip(s in ANY) {
            let min = f64::FEMTOSECONDS_PER_SECOND.inv();
            let max = u64::MAX as f64;
            let exp = if s < min || s.is_nan() {
                0.0
            } else if s > max {
                max
            } else {
                s
            };
            let delta = TimeDelta::seconds(s);
            if s > 1.0 {
                assert_float_eq!(delta.to_seconds(), exp, rel <= 1e-15);
            } else {
                assert_float_eq!(delta.to_seconds(), exp, abs <= 1e-15);
            }
        }
    }
}
