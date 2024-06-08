/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module `subsecond` exposes the [Subsecond] newtype for working with fractions of seconds.

use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};

use num::ToPrimitive;

use thiserror::Error;

/// Error type returned when attempting to construct a [Subsecond] from an invalid `f64`.
#[derive(Debug, Copy, Clone, Error)]
#[error("subsecond must be in the range [0.0, 1.0), but was `{0}`")]
pub struct InvalidSubsecond(f64);

impl PartialOrd for InvalidSubsecond {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InvalidSubsecond {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialEq for InvalidSubsecond {
    fn eq(&self, other: &Self) -> bool {
        self.0.total_cmp(&other.0) == Ordering::Equal
    }
}

impl Eq for InvalidSubsecond {}

/// An `f64` value in the range `[0.0, 1.0)` representing a fraction of a second with femtosecond
/// precision.
#[derive(Debug, Default, Copy, Clone)]
pub struct Subsecond(pub(crate) f64);

/// Two Subseconds are considered equal if their absolute difference is less than 1 femtosecond.
impl PartialEq for Subsecond {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 || (self.0 - other.0).abs() < 1e-15
    }
}

// The underlying f64 is guaranteed to be in the range [0.0, 1.0).
impl Eq for Subsecond {}

impl PartialOrd for Subsecond {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// The underlying f64 is guaranteed to be in the range [0.0, 1.0), and hence has a total ordering.
impl Ord for Subsecond {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.partial_cmp(&other.0) {
            Some(ordering) => ordering,
            None => unreachable!(),
        }
    }
}

impl Subsecond {
    pub fn new(subsecond: f64) -> Result<Self, InvalidSubsecond> {
        if !(0.0..1.0).contains(&subsecond) {
            Err(InvalidSubsecond(subsecond))
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
        let precision = f.precision().unwrap_or(3);
        write!(f, "{:.*}", precision, self.0)
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
    #[case::below_lower_bound(-1e-15, Err(InvalidSubsecond(-1e-15)))]
    #[case::on_lower_bound(0.0, Ok(Subsecond(0.0)))]
    #[case::between_bounds(0.5, Ok(Subsecond(0.5)))]
    #[case::on_upper_bound(1.0, Err(InvalidSubsecond(1.0)))]
    #[case::above_upper_bound(1.5, Err(InvalidSubsecond(1.5)))]
    fn test_subsecond_new(#[case] raw: f64, #[case] expected: Result<Subsecond, InvalidSubsecond>) {
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
        assert_eq!("0.123", subsecond.to_string());
        assert_eq!(format!("{:.15}", subsecond), "0.123456789876543");
    }

    #[test]
    fn test_subsecond_into_f64() {
        let subsecond = Subsecond(0.0);
        let as_f64: f64 = subsecond.into();
        assert_eq!(0.0, as_f64);
    }

    #[test]
    fn test_invalid_subsecond_ord() {
        let actual = InvalidSubsecond(-f64::NAN).partial_cmp(&InvalidSubsecond(f64::NAN));
        let expected = Some(Ordering::Less);
        assert_eq!(actual, expected);
        let actual = InvalidSubsecond(-f64::NAN).cmp(&InvalidSubsecond(f64::NAN));
        let expected = Ordering::Less;
        assert_eq!(actual, expected);
    }
}
