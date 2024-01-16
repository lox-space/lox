/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::errors::LoxError;
use crate::time::dates::Date;
use crate::time::scales::TimeScale;
use std::fmt::Display;

pub mod constants;
pub mod continuous;
pub mod dates;
pub mod intervals;
pub mod leap_seconds;
pub mod scales;
pub mod utc;

/// `WallClock` is the trait by which high-precision time representations expose human-readable time components.
///
/// The components returned by a `WallClock` must be interpreted in terms of its [TimeScale]. For example, a UTC
/// `WallClock` will have a `second` component of 60 during a leap second.
pub trait WallClock {
    fn scale(&self) -> TimeScale;
    fn hour(&self) -> u8;
    fn minute(&self) -> u8;
    fn second(&self) -> u8;
    fn millisecond(&self) -> Thousandths;
    fn microsecond(&self) -> Thousandths;
    fn nanosecond(&self) -> Thousandths;
    fn picosecond(&self) -> Thousandths;
    fn femtosecond(&self) -> Thousandths;
    fn attosecond(&self) -> Thousandths;
}

/// Newtype wrapper for thousandths of an SI-prefixed subsecond (milli, micro, nano, etc.).
#[repr(transparent)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Thousandths(u16);

impl Thousandths {
    pub fn new(thousandths: u16) -> Result<Self, LoxError> {
        if !(0..1000).contains(&thousandths) {
            Err(LoxError::InvalidThousandths(thousandths))
        } else {
            Ok(Self(thousandths))
        }
    }
}

impl Display for Thousandths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:03}", self.0)
    }
}

impl TryFrom<u16> for Thousandths {
    type Error = LoxError;

    fn try_from(thousandths: u16) -> Result<Self, Self::Error> {
        Self::new(thousandths)
    }
}

#[allow(clippy::from_over_into)] // the Into conversion is infallible, but From is not
impl Into<u64> for Thousandths {
    fn into(self) -> u64 {
        self.0 as u64
    }
}

#[cfg(test)]
mod thousandths_tests {
    use crate::time::Thousandths;

    #[test]
    fn test_new_valid() {
        assert!(Thousandths::new(0).is_ok());
        assert!(Thousandths::new(999).is_ok());
    }

    #[test]
    fn test_new_invalid() {
        assert!(Thousandths::new(1000).is_err());
        assert!(Thousandths::new(1001).is_err());
    }
}
