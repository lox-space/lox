/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::errors::LoxError;
use std::fmt;
use std::fmt::{Display, Formatter};

pub mod constants;
pub mod continuous;
pub mod dates;
pub mod intervals;
pub mod leap_seconds;
pub mod utc;

/// `WallClock` is the trait by which high-precision time representations expose human-readable time components.
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

/// Newtype wrapper for thousandths of an SI-prefixed subsecond (milli, micro, nano, etc.).
#[repr(transparent)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PerMille(u16);

impl PerMille {
    pub fn new(per_mille: u16) -> Result<Self, LoxError> {
        if !(0..1000).contains(&per_mille) {
            Err(LoxError::InvalidPerMille(per_mille))
        } else {
            Ok(Self(per_mille))
        }
    }
}

impl Display for PerMille {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:03}", self.0)
    }
}

impl TryFrom<u16> for PerMille {
    type Error = LoxError;

    fn try_from(per_mille: u16) -> Result<Self, Self::Error> {
        Self::new(per_mille)
    }
}

#[allow(clippy::from_over_into)] // the Into conversion is infallible, but From is not
impl Into<i64> for PerMille {
    fn into(self) -> i64 {
        self.0 as i64
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::LoxError;
    use crate::time::PerMille;
    use rstest::rstest;

    #[rstest]
    #[case::on_lower_bound(0, Ok(PerMille(0)))]
    #[case::between_bounds(1, Ok(PerMille(1)))]
    #[case::on_upper_bound(999, Ok(PerMille(999)))]
    #[case::above_upper_bound(1000, Err(LoxError::InvalidPerMille(1000)))]
    fn test_per_mille_new(#[case] input: u16, #[case] expected: Result<PerMille, LoxError>) {
        let actual = PerMille::new(input);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero(PerMille(0), "000")]
    #[case::one_digit(PerMille(1), "001")]
    #[case::two_digits(PerMille(11), "011")]
    #[case::three_digts(PerMille(111), "111")]
    fn test_per_mille_display(#[case] input: PerMille, #[case] expected: &str) {
        let actual = input.to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_per_mille_try_from() {
        assert_eq!(PerMille::try_from(0), Ok(PerMille(0)));
        assert_eq!(
            PerMille::try_from(1000),
            Err(LoxError::InvalidPerMille(1000))
        );
    }

    #[test]
    fn test_per_mille_into_i64() {
        assert_eq!(Into::<i64>::into(PerMille(0)), 0i64);
    }
}
