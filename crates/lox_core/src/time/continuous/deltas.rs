/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use num::ToPrimitive;

use crate::time::constants::f64::{
    ATTOSECONDS_PER_SECOND, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_JULIAN_CENTURY,
    SECONDS_PER_JULIAN_YEAR, SECONDS_PER_MINUTE,
};

/// An absolute continuous time difference with attosecond precision.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

    pub fn seconds(value: f64) -> Self {
        let seconds = value.round().to_u64().unwrap();
        let attoseconds = (value.fract() * ATTOSECONDS_PER_SECOND).to_u64().unwrap();
        Self {
            seconds,
            attoseconds,
        }
    }

    pub fn minutes(value: f64) -> Self {
        Self::seconds(value * SECONDS_PER_MINUTE)
    }

    pub fn hours(value: f64) -> Self {
        Self::seconds(value * SECONDS_PER_HOUR)
    }

    pub fn days(value: f64) -> Self {
        Self::seconds(value * SECONDS_PER_DAY)
    }

    pub fn years(value: f64) -> Self {
        Self::seconds(value * SECONDS_PER_JULIAN_YEAR)
    }

    pub fn centuries(value: f64) -> Self {
        Self::seconds(value * SECONDS_PER_JULIAN_CENTURY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_seconds() {
        let dt = TimeDelta::int_seconds(60);
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.attoseconds, 0);
    }

    #[test]
    fn test_seconds() {
        let dt = TimeDelta::seconds(60.0);
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.attoseconds, 0);
    }

    #[test]
    fn test_minutes() {
        let dt = TimeDelta::minutes(1.0);
        assert_eq!(dt.seconds, 60);
        assert_eq!(dt.attoseconds, 0);
    }

    #[test]
    fn test_hours() {
        let dt = TimeDelta::hours(1.0);
        assert_eq!(dt.seconds, 3600);
        assert_eq!(dt.attoseconds, 0);
    }

    #[test]
    fn test_days() {
        let dt = TimeDelta::days(1.0);
        assert_eq!(dt.seconds, 86400);
        assert_eq!(dt.attoseconds, 0);
    }

    #[test]
    fn test_years() {
        let dt = TimeDelta::years(1.0);
        assert_eq!(dt.seconds, 31557600);
        assert_eq!(dt.attoseconds, 0);
    }

    #[test]
    fn test_centuries() {
        let dt = TimeDelta::centuries(1.0);
        assert_eq!(dt.seconds, 3155760000);
        assert_eq!(dt.attoseconds, 0);
    }
}
