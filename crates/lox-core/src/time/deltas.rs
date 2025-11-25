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

use std::fmt::Display;
use std::ops::{Add, AddAssign, Mul, Neg, RangeInclusive, Sub, SubAssign};

use lox_test_utils::approx_eq::ApproxEq;

use crate::f64;
use crate::i64::consts::{ATTOSECONDS_IN_SECOND, SECONDS_BETWEEN_JD_AND_J2000};
use crate::types::units::Days;

use super::julian_dates::{Epoch, JulianDate, Unit};
use super::ranges::TimeDeltaRange;
use super::subsecond::Subsecond;

pub trait TimeUnits {
    fn days(&self) -> TimeDelta;
    fn hours(&self) -> TimeDelta;
    fn mins(&self) -> TimeDelta;
    fn secs(&self) -> TimeDelta;
    fn millis(&self) -> TimeDelta;
    fn micros(&self) -> TimeDelta;
    fn nanos(&self) -> TimeDelta;
    fn picos(&self) -> TimeDelta;
    fn femtos(&self) -> TimeDelta;
    fn attos(&self) -> TimeDelta;
}

/// A unifying trait for types that can be converted into a [TimeDelta].
pub trait ToDelta {
    /// Transforms the value into a [TimeDelta].
    fn to_delta(&self) -> TimeDelta;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TimeDelta {
    Valid { seconds: i64, attoseconds: i64 },
    NaN,
    PosInf,
    NegInf,
}

impl TimeDelta {
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
    /// assert_eq!(dt.seconds(), Some(1));
    /// assert_eq!(dt.attoseconds(), Some(500_000_000_000_000_000));
    /// ```
    pub const fn new(seconds: i64, attoseconds: i64) -> Self {
        let (seconds, attoseconds) = Self::normalize(seconds, attoseconds);
        Self::Valid {
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

    pub const fn builder() -> TimeDeltaBuilder {
        TimeDeltaBuilder::new()
    }

    pub const fn from_seconds_f64(value: f64) -> Self {
        if value.is_nan() {
            return TimeDelta::NaN;
        }
        if value < i64::MIN as f64 {
            return TimeDelta::NegInf;
        }
        if value > i64::MAX as f64 {
            return TimeDelta::PosInf;
        }
        let seconds = value.round_ties_even();
        let subseconds = value - seconds;
        if subseconds.is_sign_negative() {
            let seconds = seconds as i64 - 1;
            let attoseconds =
                (subseconds * ATTOSECONDS_IN_SECOND as f64).round() as i64 + ATTOSECONDS_IN_SECOND;
            TimeDelta::Valid {
                seconds,
                attoseconds,
            }
        } else {
            let seconds = seconds as i64;
            let attoseconds = (subseconds * ATTOSECONDS_IN_SECOND as f64).round() as i64;
            TimeDelta::Valid {
                seconds,
                attoseconds,
            }
        }
    }

    pub const fn from_seconds(seconds: i64) -> Self {
        Self::new(seconds, 0)
    }

    pub const fn from_minutes(value: f64) -> Self {
        Self::from_seconds_f64(value * f64::consts::SECONDS_PER_MINUTE)
    }

    pub const fn from_hours(value: f64) -> Self {
        Self::from_seconds_f64(value * f64::consts::SECONDS_PER_HOUR)
    }

    pub const fn from_days(value: f64) -> Self {
        Self::from_seconds_f64(value * f64::consts::SECONDS_PER_DAY)
    }

    pub const fn from_julian_years(value: f64) -> Self {
        Self::from_seconds_f64(value * f64::consts::SECONDS_PER_JULIAN_YEAR)
    }

    pub const fn from_julian_centuries(value: f64) -> Self {
        Self::from_seconds_f64(value * f64::consts::SECONDS_PER_JULIAN_CENTURY)
    }

    pub const fn from_seconds_and_subsecond(seconds: i64, subsecond: Subsecond) -> Self {
        Self::new(seconds, subsecond.as_attoseconds())
    }

    pub const fn from_seconds_and_subsecond_f64(seconds: f64, subsecond: f64) -> Self {
        Self::from_seconds_f64(subsecond).add_const(Self::from_seconds_f64(seconds))
    }

    pub const fn from_julian_date(julian_date: Days, epoch: Epoch) -> Self {
        let seconds = julian_date * f64::consts::SECONDS_PER_DAY;
        let seconds = match epoch {
            Epoch::JulianDate => seconds - f64::consts::SECONDS_BETWEEN_JD_AND_J2000,
            Epoch::ModifiedJulianDate => seconds - f64::consts::SECONDS_BETWEEN_MJD_AND_J2000,
            Epoch::J1950 => seconds - f64::consts::SECONDS_BETWEEN_J1950_AND_J2000,
            Epoch::J2000 => seconds,
        };
        Self::from_seconds_f64(seconds)
    }

    pub const fn from_two_part_julian_date(jd1: Days, jd2: Days) -> Self {
        TimeDelta::from_seconds_f64(jd1 * f64::consts::SECONDS_PER_DAY)
            .add_const(TimeDelta::from_seconds_f64(
                jd2 * f64::consts::SECONDS_PER_DAY,
            ))
            .sub_const(TimeDelta::from_seconds(SECONDS_BETWEEN_JD_AND_J2000))
    }

    pub const fn as_seconds_and_subsecond(&self) -> Option<(i64, Subsecond)> {
        match self {
            TimeDelta::Valid {
                seconds,
                attoseconds,
            } => Some((*seconds, Subsecond::from_attoseconds(*attoseconds))),
            _ => None,
        }
    }

    pub const fn as_seconds_and_subsecond_f64(self) -> (f64, f64) {
        let (seconds, attoseconds) = match self {
            TimeDelta::Valid {
                seconds,
                attoseconds,
            } => (seconds, attoseconds),
            TimeDelta::NaN => return (f64::NAN, f64::NAN),
            TimeDelta::PosInf => return (f64::INFINITY, f64::INFINITY),
            TimeDelta::NegInf => return (-f64::INFINITY, -f64::INFINITY),
        };
        (
            seconds as f64,
            attoseconds as f64 / ATTOSECONDS_IN_SECOND as f64,
        )
    }

    pub const fn as_seconds_f64(&self) -> f64 {
        let (seconds, subsecond) = self.as_seconds_and_subsecond_f64();
        subsecond + seconds
    }

    pub const fn is_negative(&self) -> bool {
        match self {
            TimeDelta::Valid { seconds, .. } => *seconds < 0,
            TimeDelta::NegInf => true,
            _ => false,
        }
    }

    pub const fn is_zero(&self) -> bool {
        match &self {
            TimeDelta::Valid {
                seconds,
                attoseconds,
            } => *seconds == 0 && *attoseconds == 0,
            _ => false,
        }
    }

    pub const fn is_positive(&self) -> bool {
        match self {
            TimeDelta::Valid {
                seconds,
                attoseconds,
            } => *seconds > 0 || *seconds == 0 && *attoseconds > 0,
            TimeDelta::PosInf => true,
            _ => false,
        }
    }

    pub const fn is_finite(&self) -> bool {
        matches!(self, Self::Valid { .. })
    }

    pub const fn is_nan(&self) -> bool {
        matches!(self, Self::NaN)
    }

    pub const fn is_infinite(&self) -> bool {
        matches!(self, Self::PosInf | Self::NegInf)
    }

    pub fn range(range: RangeInclusive<i64>) -> TimeDeltaRange {
        range.into()
    }

    pub const fn seconds(&self) -> Option<i64> {
        match self {
            Self::Valid { seconds, .. } => Some(*seconds),
            _ => None,
        }
    }

    pub const fn subsecond(&self) -> Option<f64> {
        match self.as_seconds_and_subsecond() {
            Some((_, subsecond)) => Some(subsecond.as_seconds_f64()),
            None => None,
        }
    }

    pub const fn attoseconds(&self) -> Option<i64> {
        match self {
            Self::Valid { attoseconds, .. } => Some(*attoseconds),
            _ => None,
        }
    }

    const fn neg_const(self) -> Self {
        let (seconds, attoseconds) = match self {
            TimeDelta::Valid {
                seconds,
                attoseconds,
            } => (seconds, attoseconds),
            TimeDelta::NaN => return Self::NaN,
            TimeDelta::PosInf => return Self::NegInf,
            TimeDelta::NegInf => return Self::PosInf,
        };
        if attoseconds == 0 {
            return Self::Valid {
                seconds: -seconds,
                attoseconds,
            };
        }

        Self::Valid {
            seconds: -seconds - 1,
            attoseconds: ATTOSECONDS_IN_SECOND - attoseconds,
        }
    }

    pub const fn add_const(self, rhs: Self) -> Self {
        let (secs_lhs, attos_lhs, secs_rhs, attos_rhs) = match (self, rhs) {
            (
                TimeDelta::Valid {
                    seconds: secs_lhs,
                    attoseconds: attos_lhs,
                },
                TimeDelta::Valid {
                    seconds: secs_rhs,
                    attoseconds: attos_rhs,
                },
            ) => (secs_lhs, attos_lhs, secs_rhs, attos_rhs),
            (TimeDelta::PosInf, TimeDelta::Valid { .. })
            | (TimeDelta::Valid { .. }, TimeDelta::PosInf)
            | (TimeDelta::PosInf, TimeDelta::PosInf) => return TimeDelta::PosInf,
            (TimeDelta::NegInf, TimeDelta::Valid { .. })
            | (TimeDelta::Valid { .. }, TimeDelta::NegInf)
            | (TimeDelta::NegInf, TimeDelta::NegInf) => return TimeDelta::NegInf,
            (TimeDelta::PosInf, TimeDelta::NegInf) | (TimeDelta::NegInf, TimeDelta::PosInf) => {
                return TimeDelta::NaN;
            }
            (_, TimeDelta::NaN) | (TimeDelta::NaN, _) => return TimeDelta::NaN,
        };

        let seconds = secs_lhs + secs_rhs;
        let attoseconds = attos_lhs + attos_rhs;
        Self::new(seconds, attoseconds)
    }

    pub const fn sub_const(self, rhs: Self) -> Self {
        let (secs_lhs, attos_lhs, secs_rhs, attos_rhs) = match (self, rhs) {
            (
                TimeDelta::Valid {
                    seconds: secs_lhs,
                    attoseconds: attos_lhs,
                },
                TimeDelta::Valid {
                    seconds: secs_rhs,
                    attoseconds: attos_rhs,
                },
            ) => (secs_lhs, attos_lhs, secs_rhs, attos_rhs),
            (TimeDelta::PosInf, TimeDelta::Valid { .. }) => return TimeDelta::PosInf,
            (TimeDelta::Valid { .. }, TimeDelta::PosInf) => return TimeDelta::NegInf,
            (TimeDelta::NegInf, TimeDelta::Valid { .. }) => return TimeDelta::NegInf,
            (TimeDelta::Valid { .. }, TimeDelta::NegInf) => return TimeDelta::PosInf,
            (TimeDelta::PosInf, TimeDelta::PosInf) | (TimeDelta::NegInf, TimeDelta::NegInf) => {
                return TimeDelta::NaN;
            }
            (TimeDelta::PosInf, TimeDelta::NegInf) => return TimeDelta::PosInf,
            (TimeDelta::NegInf, TimeDelta::PosInf) => return TimeDelta::NegInf,
            (_, TimeDelta::NaN) | (TimeDelta::NaN, _) => return TimeDelta::NaN,
        };

        let seconds = secs_lhs - secs_rhs;
        let attoseconds = attos_lhs - attos_rhs;
        Self::new(seconds, attoseconds)
    }

    pub const fn mul_const(self, rhs: f64) -> Self {
        let (seconds, attoseconds) = match self {
            TimeDelta::Valid {
                seconds,
                attoseconds,
            } => (seconds, attoseconds),
            TimeDelta::NaN => return TimeDelta::NaN,
            TimeDelta::PosInf => {
                return if rhs.is_nan() {
                    TimeDelta::NaN
                } else if rhs > 0.0 {
                    TimeDelta::PosInf
                } else if rhs < 0.0 {
                    TimeDelta::NegInf
                } else {
                    TimeDelta::NaN
                };
            }
            TimeDelta::NegInf => {
                return if rhs.is_nan() {
                    TimeDelta::NaN
                } else if rhs > 0.0 {
                    TimeDelta::NegInf
                } else if rhs < 0.0 {
                    TimeDelta::PosInf
                } else {
                    TimeDelta::NaN
                };
            }
        };

        if rhs.is_nan() {
            return TimeDelta::NaN;
        }
        if !rhs.is_finite() {
            return if rhs.is_sign_positive() {
                TimeDelta::PosInf
            } else {
                TimeDelta::NegInf
            };
        }

        // Multiply seconds component
        let seconds_product = rhs * seconds as f64;

        // Multiply attoseconds component (keeping high precision)
        // attoseconds * factor / ATTOSECONDS_IN_SECOND
        let attoseconds_product = rhs * attoseconds as f64 / ATTOSECONDS_IN_SECOND as f64;

        // Combine results
        TimeDelta::from_seconds_f64(seconds_product + attoseconds_product)
    }
}

impl Default for TimeDelta {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl Ord for TimeDelta {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match (self, other) {
            // NaN is incomparable, but we need total ordering for Ord
            // Define NaN as less than everything else
            (TimeDelta::NaN, TimeDelta::NaN) => Ordering::Equal,
            (TimeDelta::NaN, _) => Ordering::Less,
            (_, TimeDelta::NaN) => Ordering::Greater,

            // Infinities
            (TimeDelta::NegInf, TimeDelta::NegInf) => Ordering::Equal,
            (TimeDelta::NegInf, _) => Ordering::Less,
            (_, TimeDelta::NegInf) => Ordering::Greater,

            (TimeDelta::PosInf, TimeDelta::PosInf) => Ordering::Equal,
            (TimeDelta::PosInf, _) => Ordering::Greater,
            (_, TimeDelta::PosInf) => Ordering::Less,

            // Both Valid: compare (seconds, attoseconds) tuples
            (
                TimeDelta::Valid {
                    seconds: s1,
                    attoseconds: a1,
                },
                TimeDelta::Valid {
                    seconds: s2,
                    attoseconds: a2,
                },
            ) => s1.cmp(s2).then_with(|| a1.cmp(a2)),
        }
    }
}

impl PartialOrd for TimeDelta {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<f64> for TimeDelta {
    fn from(value: f64) -> Self {
        Self::from_seconds_f64(value)
    }
}

impl Display for TimeDelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} s", self.as_seconds_f64())
    }
}

impl JulianDate for TimeDelta {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        let (mut seconds, subsecond) = self.as_seconds_and_subsecond_f64();
        seconds = match epoch {
            Epoch::JulianDate => seconds + f64::consts::SECONDS_BETWEEN_JD_AND_J2000,
            Epoch::ModifiedJulianDate => seconds + f64::consts::SECONDS_BETWEEN_MJD_AND_J2000,
            Epoch::J1950 => seconds + f64::consts::SECONDS_BETWEEN_J1950_AND_J2000,
            Epoch::J2000 => seconds,
        };
        let seconds = subsecond + seconds;
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
    fn approx_eq(
        &self,
        rhs: &Self,
        atol: f64,
        rtol: f64,
    ) -> lox_test_utils::approx_eq::ApproxEqResults {
        self.as_seconds_f64()
            .approx_eq(&rhs.as_seconds_f64(), atol, rtol)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TimeDeltaBuilder {
    seconds: i64,
    subsecond: Subsecond,
}

impl TimeDeltaBuilder {
    pub const fn new() -> Self {
        Self {
            seconds: 0,
            subsecond: Subsecond::new(),
        }
    }

    pub const fn seconds(mut self, seconds: i64) -> Self {
        self.seconds = seconds;
        self
    }

    pub const fn milliseconds(mut self, milliseconds: u32) -> Self {
        let extra_seconds = milliseconds / 1000;
        let milliseconds = milliseconds % 1000;
        self.seconds += extra_seconds as i64;
        self.subsecond = self.subsecond.set_milliseconds(milliseconds);
        self
    }

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
    /// assert_eq!(dt.seconds(), Some(1));
    /// ```
    pub const fn build(self) -> TimeDelta {
        let mut seconds = self.seconds;
        let mut attoseconds = self.subsecond.as_attoseconds();
        if seconds.is_negative() {
            seconds -= 1;
            attoseconds = ATTOSECONDS_IN_SECOND - attoseconds;
        }
        TimeDelta::new(seconds, attoseconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i64::consts::ATTOSECONDS_IN_SECOND;

    #[test]
    fn test_new_normalizes_attoseconds() {
        // Attoseconds >= ATTOSECONDS_IN_SECOND should carry into seconds
        let dt = TimeDelta::new(1, ATTOSECONDS_IN_SECOND + 500);
        assert_eq!(dt.seconds(), Some(2));
        assert_eq!(dt.attoseconds(), Some(500));

        // Negative attoseconds should borrow from seconds
        let dt = TimeDelta::new(1, -500);
        assert_eq!(dt.seconds(), Some(0));
        assert_eq!(dt.attoseconds(), Some(ATTOSECONDS_IN_SECOND - 500));
    }

    #[test]
    fn test_from_seconds() {
        let dt = TimeDelta::from_seconds(60);
        assert_eq!(dt.seconds(), Some(60));
        assert_eq!(dt.attoseconds(), Some(0));
    }

    #[test]
    fn test_from_seconds_f64_positive() {
        let dt = TimeDelta::from_seconds_f64(1.5);
        assert_eq!(dt.seconds(), Some(1));
        assert_eq!(dt.attoseconds(), Some(500_000_000_000_000_000));
    }

    #[test]
    fn test_from_seconds_f64_negative() {
        let dt = TimeDelta::from_seconds_f64(-1.5);
        assert_eq!(dt.seconds(), Some(-2));
        assert_eq!(dt.attoseconds(), Some(500_000_000_000_000_000));
    }

    #[test]
    fn test_from_seconds_f64_special_values() {
        assert!(matches!(
            TimeDelta::from_seconds_f64(f64::NAN),
            TimeDelta::NaN
        ));
        assert!(matches!(
            TimeDelta::from_seconds_f64(f64::INFINITY),
            TimeDelta::PosInf
        ));
        assert!(matches!(
            TimeDelta::from_seconds_f64(f64::NEG_INFINITY),
            TimeDelta::NegInf
        ));
    }

    #[test]
    fn test_from_minutes() {
        let dt = TimeDelta::from_minutes(1.0);
        assert_eq!(dt.seconds(), Some(60));
    }

    #[test]
    fn test_from_hours() {
        let dt = TimeDelta::from_hours(1.0);
        assert_eq!(dt.seconds(), Some(3600));
    }

    #[test]
    fn test_from_days() {
        let dt = TimeDelta::from_days(1.0);
        assert_eq!(dt.seconds(), Some(86400));
    }

    #[test]
    fn test_is_positive() {
        assert!(TimeDelta::from_seconds(1).is_positive());
        assert!(!TimeDelta::from_seconds(-1).is_positive());
        assert!(!TimeDelta::from_seconds(0).is_positive());
        assert!(TimeDelta::new(0, 1).is_positive());
        assert!(TimeDelta::PosInf.is_positive());
    }

    #[test]
    fn test_is_negative() {
        assert!(TimeDelta::from_seconds(-1).is_negative());
        assert!(!TimeDelta::from_seconds(1).is_negative());
        assert!(!TimeDelta::from_seconds(0).is_negative());
        assert!(TimeDelta::NegInf.is_negative());
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
        assert_eq!(neg.seconds(), Some(-2));
        assert_eq!(neg.attoseconds(), Some(500_000_000_000_000_000));

        // Zero attoseconds
        let dt = TimeDelta::from_seconds(1);
        let neg = -dt;
        assert_eq!(neg.seconds(), Some(-1));
        assert_eq!(neg.attoseconds(), Some(0));

        // Infinities
        assert!(matches!(-TimeDelta::PosInf, TimeDelta::NegInf));
        assert!(matches!(-TimeDelta::NegInf, TimeDelta::PosInf));
        assert!(matches!(-TimeDelta::NaN, TimeDelta::NaN));
    }

    #[test]
    fn test_add_positive() {
        let a = TimeDelta::new(1, 600_000_000_000_000_000);
        let b = TimeDelta::new(1, 600_000_000_000_000_000);
        let sum = a + b;
        assert_eq!(sum.seconds(), Some(3));
        assert_eq!(sum.attoseconds(), Some(200_000_000_000_000_000));
    }

    #[test]
    fn test_add_with_carry() {
        let a = TimeDelta::new(1, 700_000_000_000_000_000);
        let b = TimeDelta::new(0, 500_000_000_000_000_000);
        let sum = a + b;
        assert_eq!(sum.seconds(), Some(2));
        assert_eq!(sum.attoseconds(), Some(200_000_000_000_000_000));
    }

    #[test]
    fn test_add_infinities() {
        assert!(matches!(
            TimeDelta::PosInf + TimeDelta::from_seconds(1),
            TimeDelta::PosInf
        ));
        assert!(matches!(
            TimeDelta::NegInf + TimeDelta::from_seconds(1),
            TimeDelta::NegInf
        ));
        assert!(matches!(
            TimeDelta::PosInf + TimeDelta::NegInf,
            TimeDelta::NaN
        ));
    }

    #[test]
    fn test_sub_positive() {
        let a = TimeDelta::new(3, 200_000_000_000_000_000);
        let b = TimeDelta::new(1, 600_000_000_000_000_000);
        let diff = a - b;
        assert_eq!(diff.seconds(), Some(1));
        assert_eq!(diff.attoseconds(), Some(600_000_000_000_000_000));
    }

    #[test]
    fn test_sub_with_borrow() {
        let a = TimeDelta::new(2, 200_000_000_000_000_000);
        let b = TimeDelta::new(1, 500_000_000_000_000_000);
        let diff = a - b;
        assert_eq!(diff.seconds(), Some(0));
        assert_eq!(diff.attoseconds(), Some(700_000_000_000_000_000));
    }

    #[test]
    fn test_sub_to_negative() {
        let a = TimeDelta::from_seconds(1);
        let b = TimeDelta::from_seconds(2);
        let diff = a - b;
        assert_eq!(diff.seconds(), Some(-1));
        assert_eq!(diff.attoseconds(), Some(0));
    }

    #[test]
    fn test_sub_infinities() {
        assert!(matches!(
            TimeDelta::PosInf - TimeDelta::from_seconds(1),
            TimeDelta::PosInf
        ));
        assert!(matches!(
            TimeDelta::from_seconds(1) - TimeDelta::PosInf,
            TimeDelta::NegInf
        ));
        assert!(matches!(
            TimeDelta::PosInf - TimeDelta::PosInf,
            TimeDelta::NaN
        ));
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
    fn test_ord_infinities() {
        let valid = TimeDelta::from_seconds(1);

        assert!(TimeDelta::NegInf < valid);
        assert!(valid < TimeDelta::PosInf);
        assert!(TimeDelta::NegInf < TimeDelta::PosInf);
        assert!(TimeDelta::NaN < TimeDelta::NegInf);
        assert!(TimeDelta::NaN < valid);
        assert!(TimeDelta::NaN < TimeDelta::PosInf);
    }

    #[test]
    fn test_builder() {
        let dt = TimeDelta::builder()
            .seconds(1)
            .milliseconds(500)
            .microseconds(250)
            .build();

        assert_eq!(dt.seconds(), Some(1));
        assert_eq!(dt.attoseconds(), Some(500_250_000_000_000_000));
    }

    #[test]
    fn test_builder_overflow() {
        let dt = TimeDelta::builder().seconds(0).milliseconds(1500).build();

        assert_eq!(dt.seconds(), Some(1));
        assert_eq!(dt.attoseconds(), Some(500_000_000_000_000_000));
    }

    #[test]
    fn test_as_seconds_f64() {
        let dt = TimeDelta::new(1, 500_000_000_000_000_000);
        assert_eq!(dt.as_seconds_f64(), 1.5);

        let dt = TimeDelta::new(-2, 500_000_000_000_000_000);
        assert_eq!(dt.as_seconds_f64(), -1.5);
    }

    #[test]
    fn test_from_integer() {
        let dt: TimeDelta = 42i32.into();
        assert_eq!(dt.seconds(), Some(42));

        let dt: TimeDelta = 42i64.into();
        assert_eq!(dt.seconds(), Some(42));
    }

    #[test]
    fn test_add_assign() {
        let mut dt = TimeDelta::from_seconds(1);
        dt += TimeDelta::from_seconds(2);
        assert_eq!(dt.seconds(), Some(3));
    }

    #[test]
    fn test_sub_assign() {
        let mut dt = TimeDelta::from_seconds(5);
        dt -= TimeDelta::from_seconds(2);
        assert_eq!(dt.seconds(), Some(3));
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
        let result_f64 = result.as_seconds_f64();
        let expected = 0.0001000000123456789;

        // Check within attosecond precision
        assert!((result_f64 - expected).abs() < 1e-17);
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
        assert!(result.is_finite());
    }

    #[test]
    fn test_mul_special_values() {
        let dt = TimeDelta::from_seconds(100);

        // Multiply by zero
        let result = 0.0 * dt;
        assert!(result.is_zero());

        // Multiply by NaN
        let result = f64::NAN * dt;
        assert!(result.is_nan());

        // Multiply by infinity
        let result = f64::INFINITY * dt;
        assert_eq!(result, TimeDelta::PosInf);

        let result = f64::NEG_INFINITY * dt;
        assert_eq!(result, TimeDelta::NegInf);

        // Multiply infinity by negative factor
        let result = -2.0 * TimeDelta::PosInf;
        assert_eq!(result, TimeDelta::NegInf);

        let result = -2.0 * TimeDelta::NegInf;
        assert_eq!(result, TimeDelta::PosInf);

        // Multiply infinity by zero
        let result = 0.0 * TimeDelta::PosInf;
        assert!(result.is_nan());
    }
}
