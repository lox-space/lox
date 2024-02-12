/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use num::traits::Inv;
use num::ToPrimitive;

use crate::time::constants::f64::{
    ATTOSECONDS_PER_SECOND, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_JULIAN_CENTURY,
    SECONDS_PER_JULIAN_YEAR, SECONDS_PER_MINUTE,
};

const ATTOSECONDS_PER_SECOND_U128: u128 = 10u128.pow(18);

/// An absolute continuous time difference with attosecond precision.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TimeDelta {
    pub seconds: u64,
    pub attoseconds: u64,
}

impl TimeDelta {
    pub fn int_seconds(seconds: u64) -> Self {
        Self {
            seconds,
            attoseconds: 0,
        }
    }

    pub fn seconds(value: f64) -> Option<Self> {
        if value < ATTOSECONDS_PER_SECOND.inv() {
            return Some(Self::default());
        }
        if value > u64::MAX as f64 {
            return None;
        }
        if value.fract() == 0.0 {
            let seconds = value.to_u64()?;
            return Some(Self {
                seconds,
                attoseconds: 0,
            });
        }
        let value = (value * ATTOSECONDS_PER_SECOND).to_u128()?;
        let seconds = value / ATTOSECONDS_PER_SECOND_U128;
        let attoseconds = value - seconds * ATTOSECONDS_PER_SECOND_U128;
        let seconds = seconds.to_u64()?;
        let attoseconds = attoseconds.to_u64()?;
        Some(Self {
            seconds,
            attoseconds,
        })
    }

    pub fn minutes(value: f64) -> Option<Self> {
        Self::seconds(value * SECONDS_PER_MINUTE)
    }

    pub fn hours(value: f64) -> Option<Self> {
        Self::seconds(value * SECONDS_PER_HOUR)
    }

    pub fn days(value: f64) -> Option<Self> {
        Self::seconds(value * SECONDS_PER_DAY)
    }

    pub fn years(value: f64) -> Option<Self> {
        Self::seconds(value * SECONDS_PER_JULIAN_YEAR)
    }

    pub fn centuries(value: f64) -> Option<Self> {
        Self::seconds(value * SECONDS_PER_JULIAN_CENTURY)
    }

    pub fn to_seconds(&self) -> f64 {
        self.attoseconds.to_f64().unwrap() / ATTOSECONDS_PER_SECOND + self.seconds.to_f64().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn test_int_seconds() {
        let dt = TimeDelta::int_seconds(60);
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.attoseconds, 0);
    }

    #[test]
    fn test_seconds() {
        let dt = TimeDelta::seconds(60.3).expect("should be valid");
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.attoseconds, 3 * 10u64.pow(17));
    }

    #[test]
    fn test_minutes() {
        let dt = TimeDelta::minutes(1.0).expect("should be valid");
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.attoseconds, 0);
    }

    #[test]
    fn test_hours() {
        let dt = TimeDelta::hours(1.0).expect("should be valid");
        assert_eq!(dt.seconds, 3600);
        assert_eq!(dt.attoseconds, 0);
    }

    #[test]
    fn test_days() {
        let dt = TimeDelta::days(1.0).expect("should be valid");
        assert_eq!(dt.seconds, 86400);
        assert_eq!(dt.attoseconds, 0);
    }

    #[test]
    fn test_years() {
        let dt = TimeDelta::years(1.0).expect("should be valid");
        assert_eq!(dt.seconds, 31557600);
        assert_eq!(dt.attoseconds, 0);
    }

    #[test]
    fn test_centuries() {
        let dt = TimeDelta::centuries(1.0).expect("should be valid");
        assert_eq!(dt.seconds, 3155760000);
        assert_eq!(dt.attoseconds, 0);
    }

    #[test]
    fn test_attosecond() {
        let dt = TimeDelta::seconds(1e-18).expect("should be valid");
        assert_eq!(dt.seconds, 0);
        assert_eq!(dt.attoseconds, 1);
    }

    proptest! {
        #[test]
        fn prop_seconds_roundtrip(s in 0.0..(u64::MAX as f64 / ATTOSECONDS_PER_SECOND)) {
            let exp = if s < ATTOSECONDS_PER_SECOND.inv() { 0.0 } else { s };
            assert_float_eq!(TimeDelta::seconds(s).expect("should be valid").to_seconds(), exp, rel <= 1e-8);
        }
    }
}
