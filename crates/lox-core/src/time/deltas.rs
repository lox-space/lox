// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

/*!
    Module `deltas` contains [TimeDelta], the key primitive of the `lox-time` crate.

    [TimeDelta] is a signed, unscaled delta relative to an arbitrary epoch. This forms the basis
    of instants in all continuous time scales.

    The [ToDelta] trait specifies the method by which such scaled time representations should
    expose their underlying [TimeDelta].
*/

use core::fmt::Display;
use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use crate::time::constants::{
    ATTOSECONDS_IN_FEMTOSECOND, ATTOSECONDS_IN_MICROSECOND, ATTOSECONDS_IN_MILLISECOND,
    ATTOSECONDS_IN_NANOSECOND, ATTOSECONDS_IN_PICOSECOND, ATTOSECONDS_IN_SECOND, SECONDS_PER_DAY,
    SECONDS_PER_HOUR, SECONDS_PER_MINUTE,
};
use lox_approx::ApproxEq;
use thiserror::Error;

use super::subsecond::Subsecond;

mod unit;

pub use unit::TimeDeltaUnit;

/// A signed finite time delta with whole seconds and an attosecond remainder in `[0, 10¹⁸)`.
///
/// `TimeDelta` represents a duration as whole seconds plus a fractional attosecond
/// component.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeDelta {
    /// Whole seconds component.
    seconds: i64,
    /// Attosecond remainder in `[0, 10¹⁸)`.
    attoseconds: i64,
}

impl TimeDelta {
    /// A zero-length time delta.
    pub const ZERO: Self = TimeDelta::from_seconds(0);

    /// Creates a new `TimeDelta` from seconds and attoseconds.
    ///
    /// The attoseconds value is automatically normalized to [0, 10¹⁸), with
    /// overflow/underflow carried into the seconds component.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_core::time::deltas::TimeDelta;
    ///
    /// let dt = TimeDelta::new(1, 500_000_000_000_000_000);
    /// assert_eq!(dt.seconds(), 1);
    /// assert_eq!(dt.attoseconds(), 500_000_000_000_000_000);
    /// ```
    pub const fn new(seconds: i64, attoseconds: i64) -> Self {
        let (seconds, attoseconds) = Self::normalize(seconds, attoseconds);
        Self {
            seconds,
            attoseconds,
        }
    }

    /// Normalizes attoseconds to [0, ATTOSECONDS_IN_SECOND) range.
    const fn normalize(mut seconds: i64, mut attoseconds: i64) -> (i64, i64) {
        // Handle overflow: attoseconds >= ATTOSECONDS_IN_SECOND
        if attoseconds >= ATTOSECONDS_IN_SECOND {
            let carry = attoseconds / ATTOSECONDS_IN_SECOND;
            seconds += carry;
            attoseconds %= ATTOSECONDS_IN_SECOND;
        }

        // Handle underflow: attoseconds < 0
        if attoseconds < 0 {
            // Calculate how many seconds to borrow
            let borrow = (-attoseconds + ATTOSECONDS_IN_SECOND - 1) / ATTOSECONDS_IN_SECOND;
            seconds -= borrow;
            attoseconds += borrow * ATTOSECONDS_IN_SECOND;
        }

        (seconds, attoseconds)
    }

    /// Creates a `TimeDelta` from a floating-point number of seconds.
    ///
    /// # Panics
    /// Panics if a non-finite or out-of-range value is encountered.
    #[track_caller]
    pub const fn from_seconds_f64(value: f64) -> Self {
        Self::new_in(value, TimeDeltaUnit::Seconds)
    }

    /// Tries to create a `TimeDelta` from a floating-point number of seconds.
    ///
    /// # Errors
    /// Returns an error if an out-of-range value is encountered.
    pub fn try_from_seconds_f64(value: f64) -> Result<Self, InvalidFloatSeconds> {
        Self::try_new_in(value, TimeDeltaUnit::Seconds)
    }

    /// Creates a `TimeDelta` from a whole number of seconds.
    pub const fn from_seconds(seconds: i64) -> Self {
        Self::new(seconds, 0)
    }

    /// Creates a `TimeDelta` from a whole number of minutes.
    pub const fn from_minutes(minutes: i64) -> Self {
        Self::from_seconds(minutes * SECONDS_PER_MINUTE)
    }

    /// Creates a `TimeDelta` from a whole number of hours.
    pub const fn from_hours(hours: i64) -> Self {
        Self::from_seconds(hours * SECONDS_PER_HOUR)
    }

    /// Creates a `TimeDelta` from a whole number of days.
    pub const fn from_days(days: i64) -> Self {
        Self::from_seconds(days * SECONDS_PER_DAY)
    }

    /// Creates a `TimeDelta` from a number of milliseconds.
    pub const fn from_milliseconds(ms: i64) -> Self {
        let seconds = ms / 1000;
        let remainder = ms % 1000;
        Self::new(seconds, remainder * ATTOSECONDS_IN_MILLISECOND)
    }

    /// Creates a `TimeDelta` from a number of microseconds.
    pub const fn from_microseconds(us: i64) -> Self {
        let seconds = us / 1_000_000;
        let remainder = us % 1_000_000;
        Self::new(seconds, remainder * ATTOSECONDS_IN_MICROSECOND)
    }

    /// Creates a `TimeDelta` from a number of nanoseconds.
    pub const fn from_nanoseconds(ns: i64) -> Self {
        let seconds = ns / 1_000_000_000;
        let remainder = ns % 1_000_000_000;
        Self::new(seconds, remainder * ATTOSECONDS_IN_NANOSECOND)
    }

    /// Creates a `TimeDelta` from a number of picoseconds.
    pub const fn from_picoseconds(ps: i64) -> Self {
        let seconds = ps / 1_000_000_000_000;
        let remainder = ps % 1_000_000_000_000;
        Self::new(seconds, remainder * ATTOSECONDS_IN_PICOSECOND)
    }

    /// Creates a `TimeDelta` from a number of femtoseconds.
    pub const fn from_femtoseconds(fs: i64) -> Self {
        let seconds = fs / 1_000_000_000_000_000;
        let remainder = fs % 1_000_000_000_000_000;
        Self::new(seconds, remainder * ATTOSECONDS_IN_FEMTOSECOND)
    }

    /// Creates a `TimeDelta` from a number of attoseconds.
    pub const fn from_attoseconds(atto: i64) -> Self {
        let seconds = atto / ATTOSECONDS_IN_SECOND;
        let remainder = atto % ATTOSECONDS_IN_SECOND;
        Self::new(seconds, remainder)
    }

    /// Creates a `TimeDelta` from whole seconds and a [`Subsecond`] fractional part.
    pub const fn from_seconds_and_subsecond(seconds: i64, subsecond: Subsecond) -> Self {
        Self::new(seconds, subsecond.as_attoseconds())
    }

    /// Returns the whole seconds and [`Subsecond`] components.
    pub const fn as_seconds_and_subsecond(&self) -> (i64, Subsecond) {
        (self.seconds, Subsecond::from_attoseconds(self.attoseconds))
    }

    /// Returns the time delta as a floating-point number of seconds.
    ///
    /// The conversion is lossy for large values. Do arithmetic on [`TimeDelta`]
    /// and convert at the end to preserve precision.
    pub const fn as_seconds_f64(&self) -> f64 {
        self.seconds as f64 + self.attoseconds as f64 / ATTOSECONDS_IN_SECOND as f64
    }

    /// Returns `true` if the time delta is negative.
    pub const fn is_negative(&self) -> bool {
        self.seconds < 0
    }

    /// Returns `true` if the time delta is exactly zero.
    pub const fn is_zero(&self) -> bool {
        self.seconds == 0 && self.attoseconds == 0
    }

    /// Returns `true` if the time delta is positive.
    pub const fn is_positive(&self) -> bool {
        self.seconds > 0 || self.seconds == 0 && self.attoseconds > 0
    }

    /// Returns the whole seconds component.
    pub const fn seconds(&self) -> i64 {
        self.seconds
    }

    /// Returns the subsecond fraction as an `f64`.
    pub const fn subsecond(&self) -> f64 {
        self.as_seconds_and_subsecond().1.as_seconds_f64()
    }

    /// Returns the attosecond component.
    pub const fn attoseconds(&self) -> i64 {
        self.attoseconds
    }

    const fn neg_const(self) -> Self {
        if self.attoseconds == 0 {
            return Self {
                seconds: -self.seconds,
                attoseconds: self.attoseconds,
            };
        }

        Self {
            seconds: -self.seconds - 1,
            attoseconds: ATTOSECONDS_IN_SECOND - self.attoseconds,
        }
    }

    /// Adds two `TimeDelta` values in a `const` context.
    ///
    /// # Panics
    /// Panics if the sum overflows the seconds component. Use
    /// [`checked_add`](Self::checked_add) for a fallible alternative.
    pub const fn add_const(self, rhs: Self) -> Self {
        let seconds = match self.seconds.checked_add(rhs.seconds) {
            Some(seconds) => seconds,
            None => panic!("overflow adding `TimeDelta` values"),
        };
        let attoseconds = self.attoseconds + rhs.attoseconds;
        let (seconds, attoseconds) = if attoseconds >= ATTOSECONDS_IN_SECOND {
            let carry = attoseconds / ATTOSECONDS_IN_SECOND;
            let seconds = match seconds.checked_add(carry) {
                Some(seconds) => seconds,
                None => panic!("overflow adding `TimeDelta` values"),
            };
            (seconds, attoseconds % ATTOSECONDS_IN_SECOND)
        } else {
            (seconds, attoseconds)
        };
        Self {
            seconds,
            attoseconds,
        }
    }

    /// Adds two `TimeDelta` values and returns `None` if overflow occurred.
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        let seconds = self.seconds.checked_add(rhs.seconds)?;
        let attoseconds = self.attoseconds.checked_add(rhs.attoseconds)?;
        let (seconds, attoseconds) = if attoseconds >= ATTOSECONDS_IN_SECOND {
            (
                seconds.checked_add(attoseconds / ATTOSECONDS_IN_SECOND)?,
                attoseconds % ATTOSECONDS_IN_SECOND,
            )
        } else {
            (seconds, attoseconds)
        };
        Some(Self {
            seconds,
            attoseconds,
        })
    }

    /// Subtracts `rhs` from `self` in a `const` context.
    ///
    /// # Panics
    /// Panics if the difference overflows the seconds component. Use
    /// [`checked_sub`](Self::checked_sub) for a fallible alternative.
    pub const fn sub_const(self, rhs: Self) -> Self {
        let mut seconds = match self.seconds.checked_sub(rhs.seconds) {
            Some(seconds) => seconds,
            None => panic!("overflow subtracting `TimeDelta` values"),
        };
        let attoseconds = if self.attoseconds >= rhs.attoseconds {
            self.attoseconds - rhs.attoseconds
        } else {
            seconds = match seconds.checked_sub(1) {
                Some(seconds) => seconds,
                None => panic!("overflow subtracting `TimeDelta` values"),
            };
            self.attoseconds + ATTOSECONDS_IN_SECOND - rhs.attoseconds
        };
        Self {
            seconds,
            attoseconds,
        }
    }

    /// Subtracts `rhs` from `self` and returns `None` if overflow occurred.
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        let mut seconds = self.seconds.checked_sub(rhs.seconds)?;
        let attoseconds = if self.attoseconds >= rhs.attoseconds {
            self.attoseconds - rhs.attoseconds
        } else {
            seconds = seconds.checked_sub(1)?;
            self.attoseconds + ATTOSECONDS_IN_SECOND - rhs.attoseconds
        };
        Some(Self {
            seconds,
            attoseconds,
        })
    }

    /// Multiplies the time delta by an `f64` scalar in a `const` context.
    pub const fn mul_const(self, rhs: f64) -> Self {
        let seconds_product = rhs * self.seconds as f64;
        let attoseconds_product = rhs * self.attoseconds as f64 / ATTOSECONDS_IN_SECOND as f64;

        Self::from_seconds_f64(attoseconds_product + seconds_product)
    }

    /// Tries to multiply the time delta by an `f64` scalar.
    ///
    /// # Errors
    /// Returns an error if an out-of-range value is encountered.
    pub fn try_mul(self, rhs: f64) -> Result<Self, InvalidFloatSeconds> {
        let seconds_product = rhs * self.seconds as f64;
        let attoseconds_product = rhs * self.attoseconds as f64 / ATTOSECONDS_IN_SECOND as f64;

        Self::try_from_seconds_f64(attoseconds_product + seconds_product)
    }
}

/// Error returned when trying to create a `TimeDelta` from an out-of-range `f64`.
#[derive(Debug, Error, PartialEq)]
#[error("value {0} is non-finite or out-of-range")]
pub struct InvalidFloatSeconds(f64);

impl InvalidFloatSeconds {
    /// Creates the error for a value in seconds.
    pub(crate) const fn new(seconds: f64) -> Self {
        Self(seconds)
    }
}

impl Default for TimeDelta {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl Ord for TimeDelta {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let Self {
            seconds: s1,
            attoseconds: a1,
        } = self;
        let Self {
            seconds: s2,
            attoseconds: a2,
        } = other;
        s1.cmp(s2).then_with(|| a1.cmp(a2))
    }
}

impl PartialOrd for TimeDelta {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for TimeDelta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} s", self.as_seconds_f64())
    }
}

impl Neg for TimeDelta {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.neg_const()
    }
}

impl Add for TimeDelta {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.add_const(rhs)
    }
}

impl AddAssign for TimeDelta {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sub for TimeDelta {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.sub_const(rhs)
    }
}

impl SubAssign for TimeDelta {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Mul<TimeDelta> for f64 {
    type Output = TimeDelta;

    /// Scales a `TimeDelta` by a scalar.
    ///
    /// # Panics
    /// Panics if a non-finite or out-of-range value is encountered. Use
    /// [`TimeDelta::try_mul`] for a fallible alternative.
    fn mul(self, rhs: TimeDelta) -> Self::Output {
        rhs.mul_const(self)
    }
}

impl ApproxEq for TimeDelta {
    fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> lox_approx::ApproxEqResults {
        self.as_seconds_f64()
            .approx_eq(&rhs.as_seconds_f64(), atol, rtol)
    }
}

impl Sum for TimeDelta {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum = TimeDelta::ZERO;

        for delta in iter {
            sum += delta;
        }

        sum
    }
}

impl<'a> Sum<&'a TimeDelta> for TimeDelta {
    fn sum<I: Iterator<Item = &'a TimeDelta>>(iter: I) -> Self {
        TimeDelta::sum(iter.copied())
    }
}

/// A unifying trait for types that can be converted into a [TimeDelta].
pub trait ToDelta {
    /// Transforms the value into a [TimeDelta].
    fn to_delta(&self) -> TimeDelta;
}

/// Extension trait for creating [`TimeDelta`] values from numeric types.
///
/// # Examples
///
/// ```
/// use lox_core::time::deltas::TimeDeltaLiterals;
///
/// let delta = 1.5.days();
/// let delta = 30.mins();
/// let delta = 500.millis();
/// ```
pub trait TimeDeltaLiterals {
    /// Creates a [`TimeDelta`] from a value in days.
    fn days(&self) -> TimeDelta;
    /// Creates a [`TimeDelta`] from a value in hours.
    fn hours(&self) -> TimeDelta;
    /// Creates a [`TimeDelta`] from a value in minutes.
    fn mins(&self) -> TimeDelta;
    /// Creates a [`TimeDelta`] from a value in seconds.
    fn secs(&self) -> TimeDelta;
    /// Creates a [`TimeDelta`] from a value in milliseconds.
    fn millis(&self) -> TimeDelta;
    /// Creates a [`TimeDelta`] from a value in microseconds.
    fn micros(&self) -> TimeDelta;
    /// Creates a [`TimeDelta`] from a value in nanoseconds.
    fn nanos(&self) -> TimeDelta;
    /// Creates a [`TimeDelta`] from a value in picoseconds.
    fn picos(&self) -> TimeDelta;
    /// Creates a [`TimeDelta`] from a value in femtoseconds.
    fn femtos(&self) -> TimeDelta;
    /// Creates a [`TimeDelta`] from a value in attoseconds.
    fn attos(&self) -> TimeDelta;
}

impl TimeDeltaLiterals for f64 {
    fn days(&self) -> TimeDelta {
        TimeDelta::new_in(*self, TimeDeltaUnit::Days)
    }
    fn hours(&self) -> TimeDelta {
        TimeDelta::new_in(*self, TimeDeltaUnit::Hours)
    }
    fn mins(&self) -> TimeDelta {
        TimeDelta::new_in(*self, TimeDeltaUnit::Minutes)
    }
    fn secs(&self) -> TimeDelta {
        TimeDelta::new_in(*self, TimeDeltaUnit::Seconds)
    }
    fn millis(&self) -> TimeDelta {
        TimeDelta::new_in(*self, TimeDeltaUnit::Milliseconds)
    }
    fn micros(&self) -> TimeDelta {
        TimeDelta::new_in(*self, TimeDeltaUnit::Microseconds)
    }
    fn nanos(&self) -> TimeDelta {
        TimeDelta::new_in(*self, TimeDeltaUnit::Nanoseconds)
    }
    fn picos(&self) -> TimeDelta {
        TimeDelta::new_in(*self, TimeDeltaUnit::Picoseconds)
    }
    fn femtos(&self) -> TimeDelta {
        TimeDelta::new_in(*self, TimeDeltaUnit::Femtoseconds)
    }
    fn attos(&self) -> TimeDelta {
        TimeDelta::new_in(*self, TimeDeltaUnit::Attoseconds)
    }
}

impl TimeDeltaLiterals for i64 {
    fn days(&self) -> TimeDelta {
        TimeDelta::from_days(*self)
    }
    fn hours(&self) -> TimeDelta {
        TimeDelta::from_hours(*self)
    }
    fn mins(&self) -> TimeDelta {
        TimeDelta::from_minutes(*self)
    }
    fn secs(&self) -> TimeDelta {
        TimeDelta::from_seconds(*self)
    }
    fn millis(&self) -> TimeDelta {
        TimeDelta::from_milliseconds(*self)
    }
    fn micros(&self) -> TimeDelta {
        TimeDelta::from_microseconds(*self)
    }
    fn nanos(&self) -> TimeDelta {
        TimeDelta::from_nanoseconds(*self)
    }
    fn picos(&self) -> TimeDelta {
        TimeDelta::from_picoseconds(*self)
    }
    fn femtos(&self) -> TimeDelta {
        TimeDelta::from_femtoseconds(*self)
    }
    fn attos(&self) -> TimeDelta {
        TimeDelta::from_attoseconds(*self)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::math::float::abs;
    use crate::time::constants::ATTOSECONDS_IN_SECOND;

    #[test]
    fn test_new_normalizes_attoseconds() {
        // Attoseconds >= ATTOSECONDS_IN_SECOND should carry into seconds
        let dt = TimeDelta::new(1, ATTOSECONDS_IN_SECOND + 500);
        assert_eq!(dt.seconds(), 2);
        assert_eq!(dt.attoseconds(), 500);

        // Negative attoseconds should borrow from seconds
        let dt = TimeDelta::new(1, -500);
        assert_eq!(dt.seconds(), 0);
        assert_eq!(dt.attoseconds(), ATTOSECONDS_IN_SECOND - 500);
    }

    #[test]
    fn test_from_seconds() {
        let dt = TimeDelta::from_seconds(60);
        assert_eq!(dt.seconds(), 60);
        assert_eq!(dt.attoseconds(), 0);
    }

    #[test]
    fn test_from_seconds_f64_positive() {
        let dt = TimeDelta::from_seconds_f64(1.5);
        assert_eq!(dt.seconds(), 1);
        assert_eq!(dt.attoseconds(), 500_000_000_000_000_000);
    }

    #[test]
    fn test_from_seconds_f64_negative() {
        let dt = TimeDelta::from_seconds_f64(-1.5);
        assert_eq!(dt.seconds(), -2);
        assert_eq!(dt.attoseconds(), 500_000_000_000_000_000);
    }

    #[rstest]
    #[should_panic]
    #[case::nan(f64::NAN)]
    #[should_panic]
    #[case::pos_inf(f64::INFINITY)]
    #[should_panic]
    #[case::neg_inf(f64::NEG_INFINITY)]
    #[should_panic]
    #[case::over((i64::MAX as f64).next_up())]
    #[should_panic]
    #[case::under((i64::MIN as f64).next_down())]
    fn test_from_seconds_f64_invalid_values(#[case] value: f64) {
        TimeDelta::from_seconds_f64(value);
    }

    #[test]
    fn test_from_minutes() {
        let dt = TimeDelta::from_minutes(1);
        assert_eq!(dt.seconds(), 60);
    }

    #[test]
    fn test_from_hours() {
        let dt = TimeDelta::from_hours(1);
        assert_eq!(dt.seconds(), 3600);
    }

    #[test]
    fn test_from_days() {
        let dt = TimeDelta::from_days(1);
        assert_eq!(dt.seconds(), 86400);
    }

    #[test]
    fn test_checked_add_carry() {
        assert_eq!(
            TimeDelta::new(1, 800_000_000_000_000_000)
                .checked_add(TimeDelta::new(2, 300_000_000_000_000_000)),
            Some(TimeDelta::new(4, 100_000_000_000_000_000))
        );
    }

    #[test]
    fn test_checked_add_carry_overflow() {
        assert!(
            TimeDelta::new(i64::MAX, 800_000_000_000_000_000)
                .checked_add(TimeDelta::new(0, 300_000_000_000_000_000))
                .is_none()
        );
    }

    #[test]
    #[should_panic(expected = "overflow adding `TimeDelta` values")]
    fn test_add_const_carry_overflow_panics() {
        let _ = TimeDelta::new(i64::MAX, 800_000_000_000_000_000)
            + TimeDelta::new(0, 300_000_000_000_000_000);
    }

    #[test]
    fn test_checked_sub_borrow() {
        assert_eq!(
            TimeDelta::new(1, 200_000_000_000_000_000)
                .checked_sub(TimeDelta::new(0, 500_000_000_000_000_000)),
            Some(TimeDelta::new(0, 700_000_000_000_000_000))
        );
    }

    #[test]
    fn test_checked_sub_borrow_underflow() {
        assert!(
            TimeDelta::new(i64::MIN, 200_000_000_000_000_000)
                .checked_sub(TimeDelta::new(0, 500_000_000_000_000_000))
                .is_none()
        );
    }

    #[test]
    #[should_panic(expected = "overflow subtracting `TimeDelta` values")]
    fn test_sub_const_borrow_underflow_panics() {
        let _ = TimeDelta::new(i64::MIN, 200_000_000_000_000_000)
            - TimeDelta::new(0, 500_000_000_000_000_000);
    }

    #[test]
    fn test_is_positive() {
        assert!(TimeDelta::from_seconds(1).is_positive());
        assert!(!TimeDelta::from_seconds(-1).is_positive());
        assert!(!TimeDelta::from_seconds(0).is_positive());
        assert!(TimeDelta::new(0, 1).is_positive());
    }

    #[test]
    fn test_is_negative() {
        assert!(TimeDelta::from_seconds(-1).is_negative());
        assert!(!TimeDelta::from_seconds(1).is_negative());
        assert!(!TimeDelta::from_seconds(0).is_negative());
    }

    #[test]
    fn test_is_zero() {
        assert!(TimeDelta::from_seconds(0).is_zero());
        assert!(TimeDelta::ZERO.is_zero());
        assert!(!TimeDelta::from_seconds(1).is_zero());
        assert!(!TimeDelta::new(0, 1).is_zero());
    }

    #[test]
    fn test_neg() {
        let dt = TimeDelta::new(1, 500_000_000_000_000_000);
        let neg = -dt;
        assert_eq!(neg.seconds(), -2);
        assert_eq!(neg.attoseconds(), 500_000_000_000_000_000);

        // Zero attoseconds
        let dt = TimeDelta::from_seconds(1);
        let neg = -dt;
        assert_eq!(neg.seconds(), -1);
        assert_eq!(neg.attoseconds(), 0);
    }

    #[test]
    fn test_add_positive() {
        let a = TimeDelta::new(1, 600_000_000_000_000_000);
        let b = TimeDelta::new(1, 600_000_000_000_000_000);
        let sum = a + b;
        assert_eq!(sum.seconds(), 3);
        assert_eq!(sum.attoseconds(), 200_000_000_000_000_000);
    }

    #[test]
    fn test_add_with_carry() {
        let a = TimeDelta::new(1, 700_000_000_000_000_000);
        let b = TimeDelta::new(0, 500_000_000_000_000_000);
        let sum = a + b;
        assert_eq!(sum.seconds(), 2);
        assert_eq!(sum.attoseconds(), 200_000_000_000_000_000);
    }

    #[test]
    fn test_sub_positive() {
        let a = TimeDelta::new(3, 200_000_000_000_000_000);
        let b = TimeDelta::new(1, 600_000_000_000_000_000);
        let diff = a - b;
        assert_eq!(diff.seconds(), 1);
        assert_eq!(diff.attoseconds(), 600_000_000_000_000_000);
    }

    #[test]
    fn test_sub_with_borrow() {
        let a = TimeDelta::new(2, 200_000_000_000_000_000);
        let b = TimeDelta::new(1, 500_000_000_000_000_000);
        let diff = a - b;
        assert_eq!(diff.seconds(), 0);
        assert_eq!(diff.attoseconds(), 700_000_000_000_000_000);
    }

    #[test]
    fn test_sub_to_negative() {
        let a = TimeDelta::from_seconds(1);
        let b = TimeDelta::from_seconds(2);
        let diff = a - b;
        assert_eq!(diff.seconds(), -1);
        assert_eq!(diff.attoseconds(), 0);
    }

    #[test]
    fn test_ord_valid() {
        let a = TimeDelta::new(1, 500_000_000_000_000_000);
        let b = TimeDelta::new(2, 300_000_000_000_000_000);
        let c = TimeDelta::new(1, 500_000_000_000_000_000);

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, c);
        assert!(a <= c);
        assert!(a >= c);
    }

    #[test]
    fn test_as_seconds_f64() {
        let dt = TimeDelta::new(1, 500_000_000_000_000_000);
        assert_eq!(dt.as_seconds_f64(), 1.5);

        let dt = TimeDelta::new(-2, 500_000_000_000_000_000);
        assert_eq!(dt.as_seconds_f64(), -1.5);
    }

    #[test]
    fn test_add_assign() {
        let mut dt = TimeDelta::from_seconds(1);
        dt += TimeDelta::from_seconds(2);
        assert_eq!(dt.seconds(), 3);
    }

    #[test]
    fn test_sub_assign() {
        let mut dt = TimeDelta::from_seconds(5);
        dt -= TimeDelta::from_seconds(2);
        assert_eq!(dt.seconds(), 3);
    }

    #[test]
    fn test_mul_precision() {
        // Test that multiplication preserves precision better than naive conversion
        let dt = TimeDelta::new(1000000, 123_456_789_012_345_678);
        let factor = 1e-10;

        let result = factor * dt;

        // Expected: 1000000.123456789012345678 * 1e-10 = 0.0001000000123456789...
        // With improved precision, we should preserve more digits
        let result_f64 = result.as_seconds_f64();
        let expected = 0.0001000000123456789;

        // Check within attosecond precision
        assert!(abs(result_f64 - expected) < 1e-17);
    }

    #[test]
    fn test_mul_small_factors() {
        // Test with very small factors like those used in time scale conversions
        let dt = TimeDelta::new(788000833, 145_000_000_000_000_000);
        let lg = 6.969290134e-10; // From TCG/TT conversions

        let result = lg * dt;

        // The result should be computed with higher precision than naive approach
        let result_seconds = result.as_seconds_f64();

        // Verify it's in the expected range (around 0.55 seconds for these values)
        assert!(result_seconds > 0.5 && result_seconds < 0.6);
    }

    #[test]
    fn test_mul_special_values() {
        let dt = TimeDelta::from_seconds(100);

        // Multiply by zero
        let result = 0.0 * dt;
        assert!(result.is_zero());
    }

    #[test]
    fn test_time_units_f64() {
        assert_eq!(1.5.days(), TimeDelta::from_hours(36));
        assert_eq!(2.0.hours(), TimeDelta::from_hours(2));
        assert_eq!(30.0.mins(), TimeDelta::from_minutes(30));
        assert_eq!(60.0.secs(), TimeDelta::from_seconds(60));
        assert_eq!(500.0.millis(), TimeDelta::from_milliseconds(500));
        assert_eq!(1000.0.micros(), TimeDelta::from_microseconds(1000));
        assert_eq!(
            123_456_789.0.nanos(),
            TimeDelta::from_nanoseconds(123_456_789)
        );
        assert_eq!(1000.0.picos(), TimeDelta::from_picoseconds(1000));
        assert_eq!(1000.0.femtos(), TimeDelta::from_femtoseconds(1000));
        assert_eq!(1000.0.attos(), TimeDelta::from_attoseconds(1000));
    }

    #[test]
    fn test_time_units_i64() {
        assert_eq!(1_i64.days(), TimeDelta::from_days(1));
        assert_eq!(2_i64.hours(), TimeDelta::from_hours(2));
        assert_eq!(30_i64.mins(), TimeDelta::from_minutes(30));
        assert_eq!(60_i64.secs(), TimeDelta::from_seconds(60));
        assert_eq!(500_i64.millis(), TimeDelta::from_milliseconds(500));
        assert_eq!(1000_i64.micros(), TimeDelta::from_microseconds(1000));
        assert_eq!(1000_i64.nanos(), TimeDelta::from_nanoseconds(1000));
        assert_eq!(1000_i64.picos(), TimeDelta::from_picoseconds(1000));
        assert_eq!(1000_i64.femtos(), TimeDelta::from_femtoseconds(1000));
        assert_eq!(1000_i64.attos(), TimeDelta::from_attoseconds(1000));
    }

    #[test]
    fn test_from_milliseconds() {
        let dt = TimeDelta::from_milliseconds(1500);
        assert_eq!(dt.seconds(), 1);
        assert_eq!(dt.attoseconds(), 500_000_000_000_000_000);

        let dt = TimeDelta::from_milliseconds(-1500);
        assert_eq!(dt.seconds(), -2);
        assert_eq!(dt.attoseconds(), 500_000_000_000_000_000);
    }

    #[test]
    fn test_from_microseconds() {
        let dt = TimeDelta::from_microseconds(1_000_000);
        assert_eq!(dt.seconds(), 1);
        assert_eq!(dt.attoseconds(), 0);

        let dt = TimeDelta::from_microseconds(1_500_000);
        assert_eq!(dt.seconds(), 1);
        assert_eq!(dt.attoseconds(), 500_000_000_000_000_000);
    }

    #[test]
    fn test_from_nanoseconds() {
        let dt = TimeDelta::from_nanoseconds(500_000_000);
        assert_eq!(dt.as_seconds_f64(), 0.5);

        let dt = TimeDelta::from_nanoseconds(1_500_000_000);
        assert_eq!(dt.seconds(), 1);
        assert_eq!(dt.attoseconds(), 500_000_000_000_000_000);
    }

    #[test]
    fn test_from_picoseconds() {
        let dt = TimeDelta::from_picoseconds(1_000_000_000_000);
        assert_eq!(dt.seconds(), 1);
        assert_eq!(dt.attoseconds(), 0);
    }

    #[test]
    fn test_from_femtoseconds() {
        let dt = TimeDelta::from_femtoseconds(1_000_000_000_000_000);
        assert_eq!(dt.seconds(), 1);
        assert_eq!(dt.attoseconds(), 0);
    }

    #[test]
    fn test_from_attoseconds() {
        let dt = TimeDelta::from_attoseconds(ATTOSECONDS_IN_SECOND);
        assert_eq!(dt.seconds(), 1);
        assert_eq!(dt.attoseconds(), 0);

        let dt = TimeDelta::from_attoseconds(ATTOSECONDS_IN_SECOND + 42);
        assert_eq!(dt.seconds(), 1);
        assert_eq!(dt.attoseconds(), 42);
    }

    #[test]
    fn test_sum_owned() {
        let deltas = [TimeDelta::from_seconds(42), TimeDelta::from_seconds(43)];
        let sum = deltas.into_iter().sum::<TimeDelta>();

        assert_eq!(sum, TimeDelta::from_seconds(85));
    }

    #[test]
    fn test_sum_borrowed() {
        let deltas = [TimeDelta::from_seconds(42), TimeDelta::from_seconds(43)];
        let sum = deltas.iter().sum::<TimeDelta>();

        assert_eq!(sum, TimeDelta::from_seconds(85));
    }
}
