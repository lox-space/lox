// SPDX-FileCopyrightText: 2023 Andrei Zisu <matzipan@gmail.com>
// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! The [Subsecond] newtype for working with fractions of seconds.
//!
//! This module provides a high-precision representation of subsecond time values
//! with attosecond (10⁻¹⁸ second) resolution. The representation uses six components
//! to store milliseconds, microseconds, nanoseconds, picoseconds, femtoseconds, and
//! attoseconds, each normalized to the range [0, 999].
//!
//! # Examples
//!
//! ```
//! use lox_time::subsecond::Subsecond;
//!
//! // Create from individual components
//! let s = Subsecond::new()
//!     .set_milliseconds(123)
//!     .set_microseconds(456)
//!     .set_nanoseconds(789);
//!
//! assert_eq!(s.as_attoseconds(), 123456789000000000);
//!
//! // Create from total attoseconds
//! let s = Subsecond::from_attoseconds(123456789123456789);
//! assert_eq!(s.milliseconds(), 123);
//! assert_eq!(s.microseconds(), 456);
//!
//! // Parse from string
//! let s: Subsecond = "123456".parse().unwrap();
//! assert_eq!(s.milliseconds(), 123);
//! assert_eq!(s.microseconds(), 456);
//! ```

use std::fmt::Display;
use std::str::FromStr;

use lox_core::f64::consts::SECONDS_PER_ATTOSECOND;
use lox_core::i64::consts::{
    ATTOSECONDS_IN_FEMTOSECOND, ATTOSECONDS_IN_MICROSECOND, ATTOSECONDS_IN_MILLISECOND,
    ATTOSECONDS_IN_NANOSECOND, ATTOSECONDS_IN_PICOSECOND, ATTOSECONDS_IN_SECOND,
};
use thiserror::Error;

const FACTORS: [i64; 6] = [
    ATTOSECONDS_IN_MILLISECOND,
    ATTOSECONDS_IN_MICROSECOND,
    ATTOSECONDS_IN_NANOSECOND,
    ATTOSECONDS_IN_PICOSECOND,
    ATTOSECONDS_IN_FEMTOSECOND,
    1,
];

/// A high-precision representation of subsecond time with attosecond resolution.
///
/// `Subsecond` stores time values less than one second using six components:
/// milliseconds, microseconds, nanoseconds, picoseconds, femtoseconds, and attoseconds.
/// Each component is normalized to the range [0, 999].
///
/// The total precision is 10⁻¹⁸ seconds (one attosecond), providing sufficient accuracy
/// for astronomical and high-precision timing applications.
#[derive(Debug, Default, Clone, Copy)]
pub struct Subsecond([u32; 6]);

impl Subsecond {
    /// A constant representing zero subsecond time.
    pub const ZERO: Self = Self::new();

    /// Creates a new `Subsecond` with all components set to zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_time::subsecond::Subsecond;
    ///
    /// let s = Subsecond::new();
    /// assert_eq!(s.as_attoseconds(), 0);
    /// ```
    pub const fn new() -> Self {
        Self([0; 6])
    }

    /// Creates a `Subsecond` from a total number of attoseconds.
    ///
    /// The input value is automatically normalized to the range [0, 10¹⁸) attoseconds
    /// (i.e., [0, 1) seconds). Values greater than or equal to one second wrap around,
    /// and negative values wrap from the top.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_time::subsecond::Subsecond;
    ///
    /// let s = Subsecond::from_attoseconds(123456789123456789);
    /// assert_eq!(s.milliseconds(), 123);
    /// assert_eq!(s.microseconds(), 456);
    /// assert_eq!(s.nanoseconds(), 789);
    ///
    /// // Negative values wrap around
    /// let s = Subsecond::from_attoseconds(-1);
    /// assert_eq!(s.as_attoseconds(), 999999999999999999);
    /// ```
    pub const fn from_attoseconds(attoseconds: i64) -> Self {
        let attoseconds_normalized = if attoseconds < 0 {
            ATTOSECONDS_IN_SECOND + attoseconds
        } else {
            attoseconds
        } as i128;
        let mut this = Self::new();
        this.0[0] = ((attoseconds_normalized / ATTOSECONDS_IN_MILLISECOND as i128) % 1000) as u32;
        this.0[1] = ((attoseconds_normalized / ATTOSECONDS_IN_MICROSECOND as i128) % 1000) as u32;
        this.0[2] = ((attoseconds_normalized / ATTOSECONDS_IN_NANOSECOND as i128) % 1000) as u32;
        this.0[3] = ((attoseconds_normalized / ATTOSECONDS_IN_PICOSECOND as i128) % 1000) as u32;
        this.0[4] = ((attoseconds_normalized / ATTOSECONDS_IN_FEMTOSECOND as i128) % 1000) as u32;
        this.0[5] = (attoseconds_normalized % 1000) as u32;
        this
    }

    pub const fn from_f64(value: f64) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        let rem = value % 1.0;
        // Ensure remainder is in [0, 1) range (Rust's % can return negative values)
        let rem = if rem < 0.0 { rem + 1.0 } else { rem };
        // Convert to attoseconds with rounding to handle floating-point precision issues
        let attoseconds = (rem / lox_core::f64::consts::SECONDS_PER_ATTOSECOND).round() as i64;
        Some(Self::from_attoseconds(attoseconds))
    }

    /// Sets the millisecond component (10⁻³ seconds).
    ///
    /// Values are automatically normalized to [0, 999] using modulo arithmetic.
    /// In debug builds, values >= 1000 trigger an assertion.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_time::subsecond::Subsecond;
    ///
    /// let s = Subsecond::new().set_milliseconds(123);
    /// assert_eq!(s.milliseconds(), 123);
    /// ```
    pub const fn set_milliseconds(mut self, milliseconds: u32) -> Self {
        debug_assert!(milliseconds < 1000);
        self.0[0] = milliseconds % 1000;
        self
    }

    /// Sets the microsecond component (10⁻⁶ seconds).
    ///
    /// Values are automatically normalized to [0, 999] using modulo arithmetic.
    /// In debug builds, values >= 1000 trigger an assertion.
    pub const fn set_microseconds(mut self, microseconds: u32) -> Self {
        debug_assert!(microseconds < 1000);
        self.0[1] = microseconds % 1000;
        self
    }

    /// Sets the nanosecond component (10⁻⁹ seconds).
    ///
    /// Values are automatically normalized to [0, 999] using modulo arithmetic.
    /// In debug builds, values >= 1000 trigger an assertion.
    pub const fn set_nanoseconds(mut self, nanoseconds: u32) -> Self {
        debug_assert!(nanoseconds < 1000);
        self.0[2] = nanoseconds % 1000;
        self
    }

    /// Sets the picosecond component (10⁻¹² seconds).
    ///
    /// Values are automatically normalized to [0, 999] using modulo arithmetic.
    /// In debug builds, values >= 1000 trigger an assertion.
    pub const fn set_picoseconds(mut self, picoseconds: u32) -> Self {
        debug_assert!(picoseconds < 1000);
        self.0[3] = picoseconds % 1000;
        self
    }

    /// Sets the femtosecond component (10⁻¹⁵ seconds).
    ///
    /// Values are automatically normalized to [0, 999] using modulo arithmetic.
    /// In debug builds, values >= 1000 trigger an assertion.
    pub const fn set_femtoseconds(mut self, femtoseconds: u32) -> Self {
        debug_assert!(femtoseconds < 1000);
        self.0[4] = femtoseconds % 1000;
        self
    }

    /// Sets the attosecond component (10⁻¹⁸ seconds).
    ///
    /// Values are automatically normalized to [0, 999] using modulo arithmetic.
    /// In debug builds, values >= 1000 trigger an assertion.
    pub const fn set_attoseconds(mut self, attoseconds: u32) -> Self {
        debug_assert!(attoseconds < 1000);
        self.0[5] = attoseconds % 1000;
        self
    }

    /// Converts the subsecond value to total attoseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_time::subsecond::Subsecond;
    ///
    /// let s = Subsecond::new().set_milliseconds(123).set_microseconds(456);
    /// assert_eq!(s.as_attoseconds(), 123456000000000000);
    /// ```
    pub const fn as_attoseconds(&self) -> i64 {
        self.0[0] as i64 * FACTORS[0]
            + self.0[1] as i64 * FACTORS[1]
            + self.0[2] as i64 * FACTORS[2]
            + self.0[3] as i64 * FACTORS[3]
            + self.0[4] as i64 * FACTORS[4]
            + self.0[5] as i64 * FACTORS[5]
    }

    /// Converts the subsecond value to seconds as an `f64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_time::subsecond::Subsecond;
    ///
    /// let s = Subsecond::new().set_milliseconds(500);
    /// assert_eq!(s.as_seconds_f64(), 0.5);
    /// ```
    pub const fn as_seconds_f64(&self) -> f64 {
        self.as_attoseconds() as f64 * SECONDS_PER_ATTOSECOND
    }

    /// Returns the millisecond component (10⁻³ seconds).
    ///
    /// The returned value is always in the range [0, 999].
    pub const fn milliseconds(&self) -> u32 {
        self.0[0]
    }

    /// Returns the microsecond component (10⁻⁶ seconds).
    ///
    /// The returned value is always in the range [0, 999].
    pub const fn microseconds(&self) -> u32 {
        self.0[1]
    }

    /// Returns the nanosecond component (10⁻⁹ seconds).
    ///
    /// The returned value is always in the range [0, 999].
    pub const fn nanoseconds(&self) -> u32 {
        self.0[2]
    }

    /// Returns the picosecond component (10⁻¹² seconds).
    ///
    /// The returned value is always in the range [0, 999].
    pub const fn picoseconds(&self) -> u32 {
        self.0[3]
    }

    /// Returns the femtosecond component (10⁻¹⁵ seconds).
    ///
    /// The returned value is always in the range [0, 999].
    pub const fn femtoseconds(&self) -> u32 {
        self.0[4]
    }

    /// Returns the attosecond component (10⁻¹⁸ seconds).
    ///
    /// The returned value is always in the range [0, 999].
    pub const fn attoseconds(&self) -> u32 {
        self.0[5]
    }
}

impl Ord for Subsecond {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_attoseconds().cmp(&other.as_attoseconds())
    }
}

impl PartialOrd for Subsecond {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Subsecond {
    fn eq(&self, other: &Self) -> bool {
        self.as_attoseconds() == other.as_attoseconds()
    }
}

impl Eq for Subsecond {}

const DIGITS: usize = 18;

impl Display for Subsecond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "0.".fmt(f)?;
        let mut s = self.as_attoseconds().to_string();
        if s.len() < DIGITS {
            s = format!("{:0>width$}", s, width = DIGITS);
        }
        let p = f.precision().unwrap_or(3).clamp(0, DIGITS);
        s[0..p].fmt(f)
    }
}

/// Error returned when parsing a `Subsecond` from a string fails.
///
/// This error occurs when the input string contains non-numeric characters
/// or exceeds the maximum length of 18 digits.
#[derive(Debug, Error)]
#[error("could not parse subsecond from {0}")]
pub struct SubsecondParseError(String);

impl FromStr for Subsecond {
    type Err = SubsecondParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut this = Self::default();

        if s.is_empty() {
            return Ok(this);
        }

        if s.chars().any(|c| !c.is_numeric()) {
            return Err(SubsecondParseError(s.to_owned()));
        }
        let n = s.len();
        if n > DIGITS {
            return Err(SubsecondParseError(s.to_owned()));
        }

        let rem = n % 3;
        let s = if rem != 0 {
            let width = n + 3 - rem;
            format!("{:0<width$}", s)
        } else {
            s.to_owned()
        };

        for i in (0..s.len()).step_by(3) {
            this.0[i / 3] = s[i..i + 3].parse().unwrap();
        }

        Ok(this)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subsecond() {
        let s = Subsecond::new()
            .set_milliseconds(123)
            .set_microseconds(456)
            .set_nanoseconds(789)
            .set_picoseconds(123)
            .set_femtoseconds(456)
            .set_attoseconds(789);

        assert_eq!(s.as_attoseconds(), 123456789123456789);
        assert_eq!(s.as_seconds_f64(), 0.1234567891234568);
        assert_eq!(s.milliseconds(), 123);
        assert_eq!(s.microseconds(), 456);
        assert_eq!(s.nanoseconds(), 789);
        assert_eq!(s.picoseconds(), 123);
        assert_eq!(s.femtoseconds(), 456);
        assert_eq!(s.attoseconds(), 789);
    }

    #[test]
    fn test_subsecond_from_attoseconds() {
        let s = Subsecond::from_attoseconds(123456789123456789);

        assert_eq!(s.as_attoseconds(), 123456789123456789);
        assert_eq!(s.as_seconds_f64(), 0.1234567891234568);
        assert_eq!(s.milliseconds(), 123);
        assert_eq!(s.microseconds(), 456);
        assert_eq!(s.nanoseconds(), 789);
        assert_eq!(s.picoseconds(), 123);
        assert_eq!(s.femtoseconds(), 456);
        assert_eq!(s.attoseconds(), 789);
    }

    #[test]
    fn test_subsecond_display() {
        let s = Subsecond::new()
            .set_milliseconds(123)
            .set_microseconds(456)
            .set_nanoseconds(789)
            .set_picoseconds(123)
            .set_femtoseconds(456)
            .set_attoseconds(789);

        assert_eq!(format!("{}", s), "0.123");
        assert_eq!(format!("{:.6}", s), "0.123456");
        assert_eq!(format!("{:.18}", s), "0.123456789123456789");
    }

    #[test]
    fn test_subsecond_parse() {
        let exp = Subsecond::new().set_milliseconds(123).set_microseconds(400);
        let act: Subsecond = "1234".parse().unwrap();
        assert_eq!(act, exp);

        let exp = Subsecond::new()
            .set_milliseconds(123)
            .set_microseconds(456)
            .set_nanoseconds(789)
            .set_picoseconds(123)
            .set_femtoseconds(456)
            .set_attoseconds(789);
        let act: Subsecond = "123456789123456789".parse().unwrap();
        assert_eq!(act, exp);
    }

    #[test]
    #[should_panic]
    fn test_subsecond_parse_error() {
        "123foo".parse::<Subsecond>().unwrap();
    }

    #[test]
    fn test_subsecond_from_attoseconds_negative() {
        // -1 attosecond should wrap to 999999999999999999
        let s = Subsecond::from_attoseconds(-1);
        assert_eq!(s.as_attoseconds(), 999999999999999999);
        assert_eq!(s.milliseconds(), 999);
        assert_eq!(s.microseconds(), 999);
        assert_eq!(s.nanoseconds(), 999);
        assert_eq!(s.picoseconds(), 999);
        assert_eq!(s.femtoseconds(), 999);
        assert_eq!(s.attoseconds(), 999);
    }

    #[test]
    fn test_subsecond_from_attoseconds_zero() {
        let s = Subsecond::from_attoseconds(0);
        assert_eq!(s.as_attoseconds(), 0);
        assert_eq!(s, Subsecond::ZERO);
    }

    #[test]
    fn test_subsecond_from_attoseconds_max() {
        // Maximum subsecond value: 999999999999999999
        let max = ATTOSECONDS_IN_SECOND - 1;
        let s = Subsecond::from_attoseconds(max);
        assert_eq!(s.as_attoseconds(), max);
        assert_eq!(s.milliseconds(), 999);
        assert_eq!(s.microseconds(), 999);
        assert_eq!(s.nanoseconds(), 999);
        assert_eq!(s.picoseconds(), 999);
        assert_eq!(s.femtoseconds(), 999);
        assert_eq!(s.attoseconds(), 999);
    }

    #[test]
    fn test_subsecond_from_attoseconds_overflow() {
        // Values >= 1 second should wrap around
        let s = Subsecond::from_attoseconds(ATTOSECONDS_IN_SECOND);
        assert_eq!(s.as_attoseconds(), 0);

        let s = Subsecond::from_attoseconds(ATTOSECONDS_IN_SECOND + 123);
        assert_eq!(s.as_attoseconds(), 123);
    }

    #[test]
    fn test_subsecond_set_methods_max_value() {
        // Test that 999 is preserved correctly
        let s = Subsecond::new().set_milliseconds(999);
        assert_eq!(s.milliseconds(), 999);

        let s = Subsecond::new().set_microseconds(999);
        assert_eq!(s.microseconds(), 999);

        let s = Subsecond::new().set_nanoseconds(999);
        assert_eq!(s.nanoseconds(), 999);
    }

    #[test]
    fn test_subsecond_display_edge_cases() {
        // Test zero
        assert_eq!(format!("{}", Subsecond::ZERO), "0.000");

        // Test precision of 0
        let s = Subsecond::new().set_milliseconds(123);
        assert_eq!(format!("{:.0}", s), "");

        // Test precision larger than value
        assert_eq!(format!("{:.25}", s), "0.123000000000000000");
    }

    #[test]
    fn test_subsecond_parse_edge_cases() {
        // Empty string
        let s: Subsecond = "".parse().unwrap();
        assert_eq!(s, Subsecond::ZERO);

        // Single digit
        let s: Subsecond = "1".parse().unwrap();
        assert_eq!(s.milliseconds(), 100);

        // Two digits
        let s: Subsecond = "12".parse().unwrap();
        assert_eq!(s.milliseconds(), 120);

        // Maximum length (18 digits)
        let s: Subsecond = "999999999999999999".parse().unwrap();
        assert_eq!(s.as_attoseconds(), 999999999999999999);
    }

    #[test]
    fn test_subsecond_parse_too_long() {
        // 19 digits should fail
        let result = "1234567890123456789".parse::<Subsecond>();
        assert!(result.is_err());
    }

    #[test]
    fn test_subsecond_ordering() {
        let a = Subsecond::from_attoseconds(100);
        let b = Subsecond::from_attoseconds(200);
        let c = Subsecond::from_attoseconds(200);

        assert!(a < b);
        assert!(b > a);
        assert_eq!(b, c);
        assert!(b <= c);
        assert!(b >= c);
    }

    #[test]
    fn test_subsecond_equality() {
        let a = Subsecond::new().set_milliseconds(123);
        let b = Subsecond::from_attoseconds(123000000000000000);

        assert_eq!(a, b);
        assert_eq!(a.as_attoseconds(), b.as_attoseconds());
    }
}
