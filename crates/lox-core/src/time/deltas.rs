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
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use lox_approx::ApproxEq;
use thiserror::Error;

use crate::math::float::mul_add;

use crate::f64;
use crate::i64::consts::{
    ATTOSECONDS_IN_FEMTOSECOND, ATTOSECONDS_IN_MICROSECOND, ATTOSECONDS_IN_MILLISECOND,
    ATTOSECONDS_IN_NANOSECOND, ATTOSECONDS_IN_PICOSECOND, ATTOSECONDS_IN_SECOND,
    SECONDS_BETWEEN_JD_AND_J2000, SECONDS_PER_DAY as I64_SECONDS_PER_DAY,
    SECONDS_PER_HOUR as I64_SECONDS_PER_HOUR, SECONDS_PER_MINUTE as I64_SECONDS_PER_MINUTE,
};
use crate::types::units::Days;

use super::julian_dates::{Epoch, JulianDate, Unit};
use super::subsecond::Subsecond;

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

    /// Returns a [`TimeDeltaBuilder`] for constructing a `TimeDelta` from individual components.
    pub const fn builder() -> TimeDeltaBuilder {
        TimeDeltaBuilder::new()
    }

    /// Creates a `TimeDelta` from a floating-point number of seconds.
    ///
    /// # Panics
    /// Panics if a non-finite or out-of-range value is encountered.
    #[track_caller]
    pub const fn from_seconds_f64(value: f64) -> Self {
        assert!(
            value.is_finite() && value >= i64::MIN as f64 && value < i64::MAX as f64,
            "non-finite or out-of-range value encountered"
        );

        // Inline const-compatible round_ties_even (banker's rounding). The
        // bounds check above guarantees `value` fits in `i64::MIN..=i64::MAX`.
        let i = value as i64 as f64;
        let frac = value - i;
        let seconds = if frac > 0.5 {
            i + 1.0
        } else if frac < -0.5 {
            i - 1.0
        } else if frac == 0.5 {
            if (i as i64) % 2 == 0 { i } else { i + 1.0 }
        } else if frac == -0.5 {
            if (i as i64) % 2 == 0 { i } else { i - 1.0 }
        } else {
            i
        };
        let subseconds = value - seconds;
        // Inline const-compatible round (half away from zero). The product is
        // bounded by `ATTOSECONDS_IN_SECOND` (1e18), within `i64` range.
        let scaled = subseconds * ATTOSECONDS_IN_SECOND as f64;
        let si = scaled as i64 as f64;
        let sfrac = scaled - si;
        let attos_rounded = if sfrac >= 0.5 {
            si + 1.0
        } else if sfrac <= -0.5 {
            si - 1.0
        } else {
            si
        };

        if subseconds.is_sign_negative() {
            Self {
                seconds: seconds as i64 - 1,
                attoseconds: attos_rounded as i64 + ATTOSECONDS_IN_SECOND,
            }
        } else {
            Self {
                seconds: seconds as i64,
                attoseconds: attos_rounded as i64,
            }
        }
    }

    /// Tries to create a `TimeDelta` from a floating-point number of seconds.
    ///
    /// # Errors
    /// Returns an error if an out-of-range value is encountered.
    pub fn try_from_seconds_f64(value: f64) -> Result<Self, InvalidFloatSeconds> {
        if !value.is_finite() || value < i64::MIN as f64 || value >= i64::MAX as f64 {
            return Err(InvalidFloatSeconds(value));
        }
        Ok(Self::from_seconds_f64(value))
    }

    /// Creates a `TimeDelta` from a whole number of seconds.
    pub const fn from_seconds(seconds: i64) -> Self {
        Self::new(seconds, 0)
    }

    /// Creates a `TimeDelta` from a whole number of minutes.
    pub const fn from_minutes(minutes: i64) -> Self {
        Self::from_seconds(minutes * I64_SECONDS_PER_MINUTE)
    }

    /// Creates a `TimeDelta` from a floating-point number of minutes.
    pub const fn from_minutes_f64(value: f64) -> Self {
        Self::from_seconds_f64(value * f64::consts::SECONDS_PER_MINUTE)
    }

    /// Tries to create a `TimeDelta` from a floating-point number of minutes.
    ///
    /// # Errors
    /// Returns an error if an out-of-range value is encountered.
    pub fn try_from_minutes_f64(value: f64) -> Result<Self, InvalidFloatSeconds> {
        Self::try_from_seconds_f64(value * f64::consts::SECONDS_PER_MINUTE)
    }

    /// Creates a `TimeDelta` from a whole number of hours.
    pub const fn from_hours(hours: i64) -> Self {
        Self::from_seconds(hours * I64_SECONDS_PER_HOUR)
    }

    /// Creates a `TimeDelta` from a floating-point number of hours.
    pub const fn from_hours_f64(value: f64) -> Self {
        Self::from_seconds_f64(value * f64::consts::SECONDS_PER_HOUR)
    }

    /// Tries to create a `TimeDelta` from a floating-point number of hours.
    ///
    /// # Errors
    /// Returns an error if an out-of-range value is encountered.
    pub fn try_from_hours_f64(value: f64) -> Result<Self, InvalidFloatSeconds> {
        Self::try_from_seconds_f64(value * f64::consts::SECONDS_PER_HOUR)
    }

    /// Creates a `TimeDelta` from a whole number of days.
    pub const fn from_days(days: i64) -> Self {
        Self::from_seconds(days * I64_SECONDS_PER_DAY)
    }

    /// Creates a `TimeDelta` from a floating-point number of days.
    pub const fn from_days_f64(value: f64) -> Self {
        Self::from_seconds_f64(value * f64::consts::SECONDS_PER_DAY)
    }

    /// Tries to create a `TimeDelta` from a floating-point number of days.
    ///
    /// # Errors
    /// Returns an error if an out-of-range value is encountered.
    pub fn try_from_days_f64(value: f64) -> Result<Self, InvalidFloatSeconds> {
        Self::try_from_seconds_f64(value * f64::consts::SECONDS_PER_DAY)
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

    /// Creates a `TimeDelta` from a floating-point number of Julian years (365.25 days each).
    pub const fn from_julian_years(value: f64) -> Self {
        Self::from_seconds_f64(value * f64::consts::SECONDS_PER_JULIAN_YEAR)
    }

    /// Tries to create a `TimeDelta` from a floating-point number of Julian years.
    ///
    /// # Errors
    /// Returns an error if an out-of-range value is encountered.
    pub fn try_from_julian_years(value: f64) -> Result<Self, InvalidFloatSeconds> {
        Self::try_from_seconds_f64(value * f64::consts::SECONDS_PER_JULIAN_YEAR)
    }

    /// Creates a `TimeDelta` from a floating-point number of Julian centuries (36525 days each).
    pub const fn from_julian_centuries(value: f64) -> Self {
        Self::from_seconds_f64(value * f64::consts::SECONDS_PER_JULIAN_CENTURY)
    }

    /// Tries to create a `TimeDelta` from a floating-point number of Julian centuries.
    ///
    /// # Errors
    /// Returns an error if an out-of-range value is encountered.
    pub fn try_from_julian_centuries(value: f64) -> Result<Self, InvalidFloatSeconds> {
        Self::try_from_seconds_f64(value * f64::consts::SECONDS_PER_JULIAN_CENTURY)
    }

    /// Creates a `TimeDelta` from whole seconds and a [`Subsecond`] fractional part.
    pub const fn from_seconds_and_subsecond(seconds: i64, subsecond: Subsecond) -> Self {
        Self::new(seconds, subsecond.as_attoseconds())
    }

    /// Creates a `TimeDelta` from floating-point seconds and a subsecond fraction.
    pub const fn from_seconds_and_subsecond_f64(seconds: f64, subsecond: f64) -> Self {
        Self::from_seconds_f64(subsecond).add_const(Self::from_seconds_f64(seconds))
    }

    /// Converts a Julian date relative to `epoch` into seconds since J2000.
    const fn julian_date_seconds(julian_date: Days, epoch: Epoch) -> f64 {
        let seconds = julian_date * f64::consts::SECONDS_PER_DAY;
        match epoch {
            Epoch::JulianDate => seconds - f64::consts::SECONDS_BETWEEN_JD_AND_J2000,
            Epoch::ModifiedJulianDate => seconds - f64::consts::SECONDS_BETWEEN_MJD_AND_J2000,
            Epoch::J1950 => seconds - f64::consts::SECONDS_BETWEEN_J1950_AND_J2000,
            Epoch::J2000 => seconds,
        }
    }

    /// Creates a `TimeDelta` from a Julian date relative to the given epoch.
    pub const fn from_julian_date(julian_date: Days, epoch: Epoch) -> Self {
        Self::from_seconds_f64(Self::julian_date_seconds(julian_date, epoch))
    }

    /// Tries to create a `TimeDelta` from a Julian date relative to the given epoch.
    ///
    /// # Errors
    /// Returns an error if an out-of-range value is encountered.
    pub fn try_from_julian_date(
        julian_date: Days,
        epoch: Epoch,
    ) -> Result<Self, InvalidFloatSeconds> {
        Self::try_from_seconds_f64(Self::julian_date_seconds(julian_date, epoch))
    }

    /// Creates a `TimeDelta` from a two-part Julian date (`jd1 + jd2`).
    pub const fn from_two_part_julian_date(jd1: Days, jd2: Days) -> Self {
        TimeDelta::from_seconds_f64(jd1 * f64::consts::SECONDS_PER_DAY)
            .add_const(TimeDelta::from_seconds_f64(
                jd2 * f64::consts::SECONDS_PER_DAY,
            ))
            .sub_const(TimeDelta::from_seconds(SECONDS_BETWEEN_JD_AND_J2000))
    }

    /// Tries to create a `TimeDelta` from a two-part Julian date (`jd1 + jd2`).
    ///
    /// # Errors
    /// Returns an error if an out-of-range value is encountered.
    pub fn try_from_two_part_julian_date(
        jd1: Days,
        jd2: Days,
    ) -> Result<Self, InvalidFloatSeconds> {
        let dt1 = TimeDelta::try_from_seconds_f64(jd1 * f64::consts::SECONDS_PER_DAY)?;
        let dt2 = TimeDelta::try_from_seconds_f64(jd2 * f64::consts::SECONDS_PER_DAY)?;
        let dt = dt1
            .checked_add(dt2)
            .and_then(|dt| dt.checked_sub(TimeDelta::from_seconds(SECONDS_BETWEEN_JD_AND_J2000)))
            .ok_or(InvalidFloatSeconds(
                jd1 + jd2 - SECONDS_BETWEEN_JD_AND_J2000 as f64,
            ))?;
        Ok(dt)
    }

    /// Returns the whole seconds and [`Subsecond`] components.
    pub const fn as_seconds_and_subsecond(&self) -> (i64, Subsecond) {
        (self.seconds, Subsecond::from_attoseconds(self.attoseconds))
    }

    /// Returns the time delta as a high-precision [`Seconds`] representation.
    ///
    /// The result is a [`Seconds`] where `hi` contains the whole seconds and `lo`
    /// contains the subsecond fraction. This preserves full precision even
    /// for large time values.
    ///
    /// For a lossy single f64, use `.to_seconds().to_f64()`.
    pub const fn to_seconds(&self) -> Seconds {
        Seconds::new(
            self.seconds as f64,
            self.attoseconds as f64 / ATTOSECONDS_IN_SECOND as f64,
        )
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

impl From<f64> for TimeDelta {
    fn from(value: f64) -> Self {
        Self::from_seconds_f64(value)
    }
}

impl Display for TimeDelta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} s", self.to_seconds().to_f64())
    }
}

impl JulianDate for TimeDelta {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        let tf = self.to_seconds();
        let epoch_offset = match epoch {
            Epoch::JulianDate => f64::consts::SECONDS_BETWEEN_JD_AND_J2000,
            Epoch::ModifiedJulianDate => f64::consts::SECONDS_BETWEEN_MJD_AND_J2000,
            Epoch::J1950 => f64::consts::SECONDS_BETWEEN_J1950_AND_J2000,
            Epoch::J2000 => 0.0,
        };
        let adjusted = tf + Seconds::from_f64(epoch_offset);
        let seconds = adjusted.to_f64();
        match unit {
            Unit::Seconds => seconds,
            Unit::Days => seconds / f64::consts::SECONDS_PER_DAY,
            Unit::Years => seconds / f64::consts::SECONDS_PER_JULIAN_YEAR,
            Unit::Centuries => seconds / f64::consts::SECONDS_PER_JULIAN_CENTURY,
        }
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

impl From<i64> for TimeDelta {
    fn from(value: i64) -> Self {
        TimeDelta::from_seconds(value)
    }
}

impl From<i32> for TimeDelta {
    fn from(value: i32) -> Self {
        TimeDelta::from_seconds(value as i64)
    }
}

impl ApproxEq for TimeDelta {
    fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> lox_approx::ApproxEqResults {
        self.to_seconds()
            .to_f64()
            .approx_eq(&rhs.to_seconds().to_f64(), atol, rtol)
    }
}

/// Extension trait for creating [`TimeDelta`] values from numeric types.
///
/// # Examples
///
/// ```
/// use lox_core::time::deltas::TimeUnits;
///
/// let delta = 1.5.days();
/// let delta = 30.mins();
/// let delta = 500.millis();
/// ```
pub trait TimeUnits {
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

impl TimeUnits for f64 {
    fn days(&self) -> TimeDelta {
        TimeDelta::from_days_f64(*self)
    }
    fn hours(&self) -> TimeDelta {
        TimeDelta::from_hours_f64(*self)
    }
    fn mins(&self) -> TimeDelta {
        TimeDelta::from_minutes_f64(*self)
    }
    fn secs(&self) -> TimeDelta {
        TimeDelta::from_seconds_f64(*self)
    }
    fn millis(&self) -> TimeDelta {
        TimeDelta::from_seconds_f64(*self * f64::consts::SECONDS_PER_MILLISECOND)
    }
    fn micros(&self) -> TimeDelta {
        TimeDelta::from_seconds_f64(*self * f64::consts::SECONDS_PER_MICROSECOND)
    }
    fn nanos(&self) -> TimeDelta {
        TimeDelta::from_seconds_f64(*self * f64::consts::SECONDS_PER_NANOSECOND)
    }
    fn picos(&self) -> TimeDelta {
        TimeDelta::from_seconds_f64(*self * f64::consts::SECONDS_PER_PICOSECOND)
    }
    fn femtos(&self) -> TimeDelta {
        TimeDelta::from_seconds_f64(*self * f64::consts::SECONDS_PER_FEMTOSECOND)
    }
    fn attos(&self) -> TimeDelta {
        TimeDelta::from_seconds_f64(*self * f64::consts::SECONDS_PER_ATTOSECOND)
    }
}

impl TimeUnits for i64 {
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

/// A unifying trait for types that can be converted into a [TimeDelta].
pub trait ToDelta {
    /// Transforms the value into a [TimeDelta].
    fn to_delta(&self) -> TimeDelta;
}

/// A high-precision representation of a duration in seconds.
///
/// Uses a two-part floating point representation where the value is `hi + lo`.
/// The `lo` component is a correction term that captures precision lost in the
/// `hi` component. This allows representing values with roughly double the
/// precision of a single f64.
///
/// This is used to preserve precision when converting [`TimeDelta`] to floating
/// point, especially for large second values combined with small subsecond values.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Seconds {
    /// The high-order component (primary value in seconds)
    pub hi: f64,
    /// The low-order component (correction term in seconds)
    pub lo: f64,
}

impl Seconds {
    /// Creates a new Seconds from high and low components.
    pub const fn new(hi: f64, lo: f64) -> Self {
        Self { hi, lo }
    }

    /// Creates a Seconds from a single f64 value.
    pub const fn from_f64(value: f64) -> Self {
        Self { hi: value, lo: 0.0 }
    }

    /// Converts to a single f64 (lossy for large values with small corrections).
    pub const fn to_f64(self) -> f64 {
        self.hi + self.lo
    }

    /// Returns true if either component is NaN.
    pub const fn is_nan(self) -> bool {
        self.hi.is_nan() || self.lo.is_nan()
    }

    /// Returns true if either component is infinite.
    pub const fn is_infinite(self) -> bool {
        self.hi.is_infinite() || self.lo.is_infinite()
    }

    /// Adds two Seconds values with error compensation (Knuth's algorithm).
    fn compensated_add(self, other: Self) -> Self {
        let (s, e) = two_sum(self.hi, other.hi);
        let e = e + self.lo + other.lo;
        let (hi, lo) = two_sum(s, e);
        Self { hi, lo }
    }

    /// Subtracts another Seconds from this one.
    fn compensated_sub(self, other: Self) -> Self {
        self.compensated_add(other.neg())
    }

    /// Negates this Seconds.
    pub const fn neg(self) -> Self {
        Self {
            hi: -self.hi,
            lo: -self.lo,
        }
    }

    /// Multiplies this Seconds by an f64 scalar.
    pub fn mul_f64(self, rhs: f64) -> Self {
        let (p, e) = two_prod(self.hi, rhs);
        let e = e + self.lo * rhs;
        let (hi, lo) = two_sum(p, e);
        Self { hi, lo }
    }

    /// Multiplies two Seconds values.
    fn compensated_mul(self, other: Self) -> Self {
        let (p, e) = two_prod(self.hi, other.hi);
        let e = e + self.hi * other.lo + self.lo * other.hi;
        let (hi, lo) = two_sum(p, e);
        Self { hi, lo }
    }
}

impl Add for Seconds {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        self.compensated_add(rhs)
    }
}

impl Sub for Seconds {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self.compensated_sub(rhs)
    }
}

impl Neg for Seconds {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Seconds::neg(self)
    }
}

impl Mul<f64> for Seconds {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        self.mul_f64(rhs)
    }
}

impl Mul for Seconds {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.compensated_mul(rhs)
    }
}

impl Mul<Seconds> for f64 {
    type Output = Seconds;
    fn mul(self, rhs: Seconds) -> Self::Output {
        rhs.mul_f64(self)
    }
}

impl From<f64> for Seconds {
    fn from(value: f64) -> Self {
        Self::from_f64(value)
    }
}

/// Knuth's two-sum algorithm for error-free addition.
///
/// Returns (sum, error) where sum + error = a + b exactly.
#[inline]
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let v = s - a;
    let e = (a - (s - v)) + (b - v);
    (s, e)
}

/// Two-product algorithm using FMA for error-free multiplication.
///
/// Returns (product, error) where product + error = a * b exactly.
#[inline]
fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    let e = mul_add(a, b, -p); // FMA: a * b - p
    (p, e)
}
/// A builder for constructing [`TimeDelta`] values from individual time components.
#[derive(Copy, Clone, Debug, Default)]
pub struct TimeDeltaBuilder {
    seconds: i64,
    subsecond: Subsecond,
    negative: bool,
}

impl TimeDeltaBuilder {
    /// Creates a new builder with all components set to zero.
    pub const fn new() -> Self {
        Self {
            seconds: 0,
            subsecond: Subsecond::new(),
            negative: false,
        }
    }

    /// Sets the whole seconds component.
    pub const fn seconds(mut self, seconds: i64) -> Self {
        self.seconds = seconds;
        self
    }

    /// Marks the resulting `TimeDelta` as negative.
    pub const fn negative(mut self) -> Self {
        self.negative = true;
        self
    }

    /// Sets the milliseconds component, carrying overflow into seconds.
    pub const fn milliseconds(mut self, milliseconds: u32) -> Self {
        let extra_seconds = milliseconds / 1000;
        let milliseconds = milliseconds % 1000;
        self.seconds += extra_seconds as i64;
        self.subsecond = self.subsecond.set_milliseconds(milliseconds);
        self
    }

    /// Sets the microseconds component, carrying overflow into milliseconds.
    pub const fn microseconds(mut self, microseconds: u32) -> Self {
        let extra_milliseconds = microseconds / 1000;
        let microseconds = microseconds % 1000;
        let current_millis = self.subsecond.milliseconds();
        self.subsecond = self
            .subsecond
            .set_milliseconds(current_millis + extra_milliseconds);
        self.subsecond = self.subsecond.set_microseconds(microseconds);
        self
    }

    /// Sets the nanoseconds component, carrying overflow into microseconds.
    pub const fn nanoseconds(mut self, nanoseconds: u32) -> Self {
        let extra_microseconds = nanoseconds / 1000;
        let nanoseconds = nanoseconds % 1000;
        let current_micros = self.subsecond.microseconds();
        self.subsecond = self
            .subsecond
            .set_microseconds(current_micros + extra_microseconds);
        self.subsecond = self.subsecond.set_nanoseconds(nanoseconds);
        self
    }

    /// Sets the picoseconds component, carrying overflow into nanoseconds.
    pub const fn picoseconds(mut self, picoseconds: u32) -> Self {
        let extra_nanoseconds = picoseconds / 1000;
        let picoseconds = picoseconds % 1000;
        let current_nanos = self.subsecond.nanoseconds();
        self.subsecond = self
            .subsecond
            .set_nanoseconds(current_nanos + extra_nanoseconds);
        self.subsecond = self.subsecond.set_picoseconds(picoseconds);
        self
    }

    /// Sets the femtoseconds component, carrying overflow into picoseconds.
    pub const fn femtoseconds(mut self, femtoseconds: u32) -> Self {
        let extra_picoseconds = femtoseconds / 1000;
        let femtoseconds = femtoseconds % 1000;
        let current_picos = self.subsecond.picoseconds();
        self.subsecond = self
            .subsecond
            .set_picoseconds(current_picos + extra_picoseconds);
        self.subsecond = self.subsecond.set_femtoseconds(femtoseconds);
        self
    }

    /// Sets the attoseconds component, carrying overflow into femtoseconds.
    pub const fn attoseconds(mut self, attoseconds: u32) -> Self {
        let extra_femtoseconds = attoseconds / 1000;
        let attoseconds = attoseconds % 1000;
        let current_femtos = self.subsecond.femtoseconds();
        self.subsecond = self
            .subsecond
            .set_femtoseconds(current_femtos + extra_femtoseconds);
        self.subsecond = self.subsecond.set_attoseconds(attoseconds);
        self
    }

    /// Builds the `TimeDelta` from the configured components.
    ///
    /// All subsecond components are automatically normalized and carried
    /// into the seconds component as needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_core::time::deltas::TimeDelta;
    ///
    /// let dt = TimeDelta::builder()
    ///     .seconds(1)
    ///     .milliseconds(500)
    ///     .build();
    /// assert_eq!(dt.seconds(), 1);
    /// ```
    pub const fn build(self) -> TimeDelta {
        let seconds = self.seconds;
        let attoseconds = self.subsecond.as_attoseconds();

        // Check if negative: explicit flag OR negative seconds
        let is_negative = self.negative || seconds < 0;

        if is_negative {
            // Use absolute value of seconds, then negate the whole thing
            let abs_seconds = if seconds < 0 { -seconds } else { seconds };
            let magnitude = TimeDelta::new(abs_seconds, attoseconds);
            magnitude.neg_const()
        } else {
            TimeDelta::new(seconds, attoseconds)
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::i64::consts::ATTOSECONDS_IN_SECOND;
    use crate::math::float::abs;

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
        let dt = TimeDelta::from_minutes_f64(1.0);
        assert_eq!(dt.seconds(), 60);
    }

    #[test]
    fn test_from_hours() {
        let dt = TimeDelta::from_hours_f64(1.0);
        assert_eq!(dt.seconds(), 3600);
    }

    #[test]
    fn test_from_days() {
        let dt = TimeDelta::from_days_f64(1.0);
        assert_eq!(dt.seconds(), 86400);
    }

    #[test]
    fn test_try_from_days_f64() {
        assert_eq!(
            TimeDelta::try_from_days_f64(1.5).unwrap(),
            TimeDelta::from_hours(36)
        );
        assert!(TimeDelta::try_from_days_f64(f64::INFINITY).is_err());
    }

    #[test]
    fn test_try_from_julian_years() {
        assert_eq!(
            TimeDelta::try_from_julian_years(1.0).unwrap(),
            TimeDelta::from_seconds_f64(crate::f64::consts::SECONDS_PER_JULIAN_YEAR)
        );
        assert!(TimeDelta::try_from_julian_years(f64::NAN).is_err());
    }

    #[test]
    fn test_try_from_julian_centuries() {
        assert_eq!(
            TimeDelta::try_from_julian_centuries(1.0).unwrap(),
            TimeDelta::from_seconds_f64(crate::f64::consts::SECONDS_PER_JULIAN_CENTURY)
        );
        assert!(TimeDelta::try_from_julian_centuries(f64::INFINITY).is_err());
    }

    #[test]
    fn test_try_from_two_part_julian_date() {
        assert_eq!(
            TimeDelta::try_from_two_part_julian_date(2451545.0, 0.0).unwrap(),
            TimeDelta::ZERO
        );
        assert!(TimeDelta::try_from_two_part_julian_date(2451545.0, f64::NAN).is_err());
        assert!(TimeDelta::try_from_two_part_julian_date(f64::INFINITY, 0.0).is_err());
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
    fn test_builder() {
        let dt = TimeDelta::builder()
            .seconds(1)
            .milliseconds(500)
            .microseconds(250)
            .build();

        assert_eq!(dt.seconds(), 1);
        assert_eq!(dt.attoseconds(), 500_250_000_000_000_000);
    }

    #[test]
    fn test_builder_overflow() {
        let dt = TimeDelta::builder().seconds(0).milliseconds(1500).build();

        assert_eq!(dt.seconds(), 1);
        assert_eq!(dt.attoseconds(), 500_000_000_000_000_000);
    }

    #[test]
    fn test_to_seconds() {
        let dt = TimeDelta::new(1, 500_000_000_000_000_000);
        assert_eq!(dt.to_seconds().to_f64(), 1.5);

        let dt = TimeDelta::new(-2, 500_000_000_000_000_000);
        assert_eq!(dt.to_seconds().to_f64(), -1.5);
    }

    #[test]
    fn test_from_integer() {
        let dt: TimeDelta = 42i32.into();
        assert_eq!(dt.seconds(), 42);

        let dt: TimeDelta = 42i64.into();
        assert_eq!(dt.seconds(), 42);
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
    fn test_time_delta_julian_date() {
        let dt = TimeDelta::builder()
            .seconds(-725803232)
            .milliseconds(184)
            .build();
        let exp = -725803232.184;
        let act = dt.julian_date(Epoch::J2000, Unit::Seconds);
        assert_eq!(act, exp);
    }

    #[test]
    fn test_mul_precision() {
        // Test that multiplication preserves precision better than naive conversion
        let dt = TimeDelta::new(1000000, 123_456_789_012_345_678);
        let factor = 1e-10;

        let result = factor * dt;

        // Expected: 1000000.123456789012345678 * 1e-10 = 0.0001000000123456789...
        // With improved precision, we should preserve more digits
        let result_f64 = result.to_seconds().to_f64();
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
        let result_seconds = result.to_seconds().to_f64();

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
    fn test_builder_negative_subsecond_only() {
        // Test that negative() works for sub-second values (the main fix)
        let dt = TimeDelta::builder()
            .microseconds(65)
            .nanoseconds(500)
            .negative()
            .build();

        // Should be -65.5 microseconds = -6.55e-5 seconds
        let expected = -65.5e-6;
        assert!(
            abs(dt.to_seconds().to_f64() - expected) < 1e-15,
            "expected {} but got {}",
            expected,
            dt.to_seconds().to_f64()
        );
        assert!(dt.is_negative());
    }

    #[test]
    fn test_builder_negative_with_seconds() {
        // Test negative() with whole seconds
        let dt = TimeDelta::builder()
            .seconds(1)
            .milliseconds(500)
            .negative()
            .build();

        assert_eq!(dt.to_seconds().to_f64(), -1.5);
        assert!(dt.is_negative());
    }

    #[test]
    fn test_builder_negative_seconds_without_flag() {
        // Existing behavior: negative seconds should still work
        let dt = TimeDelta::builder().seconds(-1).milliseconds(500).build();

        assert_eq!(dt.to_seconds().to_f64(), -1.5);
        assert!(dt.is_negative());
    }

    #[test]
    fn test_builder_negative_flag_with_negative_seconds() {
        // Both negative flag and negative seconds: should be negative (no double negation)
        let dt = TimeDelta::builder()
            .seconds(-1)
            .milliseconds(500)
            .negative()
            .build();

        assert_eq!(dt.to_seconds().to_f64(), -1.5);
        assert!(dt.is_negative());
    }

    #[test]
    fn test_seconds_precision() {
        // Verify that Seconds preserves precision for large values
        let dt = TimeDelta::builder()
            .seconds(-725803167)
            .milliseconds(816)
            .build();
        let tf = dt.to_seconds();

        // For negative times, internal representation stores one less second
        // and a positive subsecond fraction that adds up to the correct value.
        // -725803167.816 = -725803168 + 0.184
        assert_eq!(tf.hi, -725803168.0);
        // lo should be the subsecond fraction (1.0 - 0.816 = 0.184)
        assert!(abs(tf.lo - 0.184) < 1e-15);

        // Combined should give the correct value
        assert!(abs(tf.to_f64() - (-725803167.816)) < 1e-9);
    }

    #[test]
    fn test_seconds_arithmetic() {
        let a = Seconds::new(1e15, 0.5);
        let b = Seconds::new(1.0, 0.25);

        // Addition
        let sum = a + b;
        assert_eq!(sum.to_f64(), 1e15 + 1.75);

        // Subtraction
        let diff = a - b;
        assert_eq!(diff.to_f64(), 1e15 - 1.0 + 0.25);

        // Multiplication by scalar
        let prod = a * 2.0;
        assert_eq!(prod.to_f64(), 2e15 + 1.0);

        // Negation
        let neg = -a;
        assert_eq!(neg.hi, -1e15);
        assert_eq!(neg.lo, -0.5);
    }

    #[test]
    fn test_time_units_f64() {
        assert_eq!(1.0.days(), TimeDelta::from_days_f64(1.0));
        assert_eq!(2.0.hours(), TimeDelta::from_hours_f64(2.0));
        assert_eq!(30.0.mins(), TimeDelta::from_minutes_f64(30.0));
        assert_eq!(60.0.secs(), TimeDelta::from_seconds_f64(60.0));
        assert_eq!(500.0.millis(), TimeDelta::from_seconds_f64(0.5));
        assert_eq!(1000.0.micros(), TimeDelta::from_seconds_f64(1e-3));
        assert_eq!(1000.0.nanos(), TimeDelta::from_seconds_f64(1e-6));
        assert_eq!(1000.0.picos(), TimeDelta::from_seconds_f64(1e-9));
        assert_eq!(1000.0.femtos(), TimeDelta::from_seconds_f64(1e-12));
        assert_eq!(1000.0.attos(), TimeDelta::from_seconds_f64(1e-15));
    }

    #[test]
    fn test_time_units_i64() {
        assert_eq!(1_i64.days(), TimeDelta::from_days_f64(1.0));
        assert_eq!(2_i64.hours(), TimeDelta::from_hours_f64(2.0));
        assert_eq!(30_i64.mins(), TimeDelta::from_minutes_f64(30.0));
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
        assert_eq!(dt.to_seconds().to_f64(), 0.5);

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
}
