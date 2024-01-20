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

/// The time scales supported by Lox.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimeScale {
    TAI,
    TCB,
    TCG,
    TDB,
    TT,
    UT1,
    UTC,
}

impl Display for TimeScale {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TimeScale::TAI => write!(f, "TAI"),
            TimeScale::TCB => write!(f, "TCB"),
            TimeScale::TCG => write!(f, "TCG"),
            TimeScale::TDB => write!(f, "TDB"),
            TimeScale::TT => write!(f, "TT"),
            TimeScale::UT1 => write!(f, "UT1"),
            TimeScale::UTC => write!(f, "UTC"),
        }
    }
}

/// `WallClock` is the trait by which high-precision time representations expose human-readable time components.
///
/// The components returned by a `WallClock` must be interpreted in terms of its [TimeScale]. For example, a UTC
/// `WallClock` will have a `second` component of 60 during a leap second.
pub trait WallClock {
    fn scale(&self) -> TimeScale;
    fn hour(&self) -> i64;
    fn minute(&self) -> i64;
    fn second(&self) -> i64;
    fn millisecond(&self) -> i64;
    fn microsecond(&self) -> i64;
    fn nanosecond(&self) -> i64;
    fn picosecond(&self) -> i64;
    fn femtosecond(&self) -> i64;
    fn attosecond(&self) -> i64;
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
mod per_mille_tests {
    use crate::time::PerMille;

    #[test]
    fn test_new_valid() {
        assert!(PerMille::new(0).is_ok());
        assert!(PerMille::new(999).is_ok());
    }

    #[test]
    fn test_new_invalid() {
        assert!(PerMille::new(1000).is_err());
        assert!(PerMille::new(1001).is_err());
    }
}
