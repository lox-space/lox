/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::errors::LoxTimeError;
use num::ToPrimitive;
use std::fmt;
use std::fmt::{Display, Formatter};

pub mod constants;
pub mod continuous;
pub mod dates;
mod debug_panic;
pub mod errors;
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

/// An f64 value in the range [0.0, 1.0) representing a fraction of a second.
// Subsecond is an input format used to prevent users from accidentally violating time
// invariants.
#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd)]
pub struct Subsecond(f64);

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
        (self.0 * 1e9).trunc().to_i64().unwrap() % 1_000_000
    }

    /// The number of picoseconds since the last nanosecond.
    pub fn picosecond(&self) -> i64 {
        (self.0 * 1e12).trunc().to_i64().unwrap() % 1_000_000_000
    }

    /// The number of femtoseconds since the last picosecond.
    pub fn femtosecond(&self) -> i64 {
        (self.0 * 1e15).trunc().to_i64().unwrap() % 1_000_000_000_000
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
