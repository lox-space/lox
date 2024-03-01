/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use num::ToPrimitive;
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::errors::LoxTimeError;

pub mod constants;
pub mod continuous;
pub mod dates;
pub mod errors;
pub mod intervals;
pub mod leap_seconds;
pub mod utc;

/// `WallClock` is the trait by which high-precision time representations expose human-readable time
/// components.
pub trait WallClock {
    fn hour(&self) -> i64;
    fn minute(&self) -> i64;
    fn second(&self) -> i64;
    fn millisecond(&self) -> i64;
    fn microsecond(&self) -> i64;
    fn nanosecond(&self) -> i64;
    fn picosecond(&self) -> i64;
    fn femtosecond(&self) -> i64;
}

/// An f64 value in the range [0.0, 1.0) representing a fraction of a second with femtosecond
/// resolution.
#[derive(Debug, Default, Copy, Clone, PartialOrd)]
pub struct Subsecond(f64);

/// Two Subseconds are considered equal if their difference is less than 1 femtosecond.
impl PartialEq for Subsecond {
    fn eq(&self, other: &Self) -> bool {
        (self.0 * 1e15).round() == (other.0 * 1e15).round()
    }
}

// The underlying f64 is guaranteed to be in the range [0.0, 1.0).
impl Eq for Subsecond {}

impl Subsecond {
    pub fn new(subsecond: f64) -> Result<Self, LoxTimeError> {
        if !(0.0..1.0).contains(&subsecond) {
            Err(LoxTimeError::InvalidSubsecond(subsecond))
        } else {
            Ok(Self(subsecond))
        }
    }

    /// The number of milliseconds in the subsecond.
    pub fn millisecond(&self) -> i64 {
        (self.0 * 1e3).trunc().to_i64().unwrap()
    }

    /// The number of microseconds since the last millisecond.
    pub fn microsecond(&self) -> i64 {
        (self.0 * 1e6).trunc().to_i64().unwrap() % 1_000
    }

    /// The number of nanoseconds since the last microsecond.
    pub fn nanosecond(&self) -> i64 {
        (self.0 * 1e9).trunc().to_i64().unwrap() % 1_000
    }

    /// The number of picoseconds since the last nanosecond.
    pub fn picosecond(&self) -> i64 {
        (self.0 * 1e12).trunc().to_i64().unwrap() % 1_000
    }

    /// The number of femtoseconds since the last picosecond.
    pub fn femtosecond(&self) -> i64 {
        (self.0 * 1e15).trunc().to_i64().unwrap() % 1_000
    }
}

impl Display for Subsecond {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:03}.{:03}.{:03}.{:03}.{:03}",
            self.millisecond(),
            self.microsecond(),
            self.nanosecond(),
            self.picosecond(),
            self.femtosecond()
        )
    }
}

#[allow(clippy::from_over_into)] // infallible in one direction only
impl Into<f64> for Subsecond {
    fn into(self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::below_lower_bound(-1e-15, Err(LoxTimeError::InvalidSubsecond(-1e-15)))]
    #[case::on_lower_bound(0.0, Ok(Subsecond(0.0)))]
    #[case::between_bounds(0.5, Ok(Subsecond(0.5)))]
    #[case::on_upper_bound(1.0, Err(LoxTimeError::InvalidSubsecond(1.0)))]
    #[case::above_upper_bound(1.5, Err(LoxTimeError::InvalidSubsecond(1.5)))]
    fn test_subsecond_new(#[case] raw: f64, #[case] expected: Result<Subsecond, LoxTimeError>) {
        assert_eq!(expected, Subsecond::new(raw));
    }

    #[test]
    fn test_subsecond_millisecond() {
        let subsecond = Subsecond(0.123456789876543);
        assert_eq!(123, subsecond.millisecond());
    }

    #[test]
    fn test_subsecond_microsecond() {
        let subsecond = Subsecond(0.123456789876543);
        assert_eq!(456, subsecond.microsecond());
    }

    #[test]
    fn test_subsecond_nanosecond() {
        let subsecond = Subsecond(0.123456789876543);
        assert_eq!(789, subsecond.nanosecond());
    }

    #[test]
    fn test_subsecond_picosecond() {
        let subsecond = Subsecond(0.123456789876543);
        assert_eq!(876, subsecond.picosecond());
    }

    #[test]
    fn test_subsecond_femtosecond() {
        let subsecond = Subsecond(0.123456789876543);
        assert_eq!(543, subsecond.femtosecond());
    }

    #[rstest]
    #[case::exactly_equal(Subsecond(0.0), Subsecond(0.0), true)]
    #[case::one_femtosecond_difference(Subsecond(0.0), Subsecond(1e-15), false)]
    #[case::more_than_one_femtosecond_difference(Subsecond(0.0), Subsecond(2e-15), false)]
    // Neither of the following values can be exactly represented as an f64, covering the edge case
    // where 1e-16 < δ < 1e-15, which `float_eq!` with `abs <= 1e-16` would consider unequal.
    #[case::less_than_one_femtosecond_difference(
        Subsecond(0.6),
        Subsecond(0.600_000_000_000_000_1),
        true
    )]
    fn test_subsecond_eq(#[case] lhs: Subsecond, #[case] rhs: Subsecond, #[case] expected: bool) {
        assert_eq!(expected, lhs == rhs);
    }

    #[test]
    fn test_subsecond_display() {
        let subsecond = Subsecond(0.123456789876543);
        assert_eq!("123.456.789.876.543", subsecond.to_string());
    }

    #[test]
    fn test_subsecond_into_f64() {
        let subsecond = Subsecond(0.0);
        assert_eq!(0.0, subsecond.into());
    }
}
