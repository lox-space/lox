// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Units of [`TimeDelta`] selected at runtime.

use core::fmt::{Display, Formatter, Result};
use core::str::FromStr;

use super::{InvalidFloatSeconds, TimeDelta};
use crate::time::constants::{
    ATTOSECONDS_IN_FEMTOSECOND, ATTOSECONDS_IN_MICROSECOND, ATTOSECONDS_IN_MILLISECOND,
    ATTOSECONDS_IN_NANOSECOND, ATTOSECONDS_IN_PICOSECOND, ATTOSECONDS_IN_SECOND, SECONDS_PER_DAY,
    SECONDS_PER_HOUR, SECONDS_PER_JULIAN_CENTURY, SECONDS_PER_JULIAN_YEAR, SECONDS_PER_MINUTE,
};
use crate::units::{DisplayIn, ParseUnitError, parse_value_and_unit};

/// A unit of [`TimeDelta`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TimeDeltaUnit {
    /// Attoseconds (as).
    Attoseconds,
    /// Femtoseconds (fs).
    Femtoseconds,
    /// Picoseconds (ps).
    Picoseconds,
    /// Nanoseconds (ns).
    Nanoseconds,
    /// Microseconds (µs).
    Microseconds,
    /// Milliseconds (ms).
    Milliseconds,
    /// Seconds (s).
    Seconds,
    /// Minutes (min).
    Minutes,
    /// Hours (h).
    Hours,
    /// Days of 86400 s (d).
    Days,
    /// Julian years of 365.25 days (a).
    JulianYears,
    /// Julian centuries of 36525 days (cy).
    JulianCenturies,
}

/// How a unit relates to the second.
enum Scale {
    /// The number of attoseconds in one unit, for units shorter than a second.
    Subsecond(i64),
    /// The number of seconds in one unit, for units of a second or longer.
    Seconds(i64),
}

impl TimeDeltaUnit {
    /// All units.
    pub const ALL: &[Self] = &[
        Self::Attoseconds,
        Self::Femtoseconds,
        Self::Picoseconds,
        Self::Nanoseconds,
        Self::Microseconds,
        Self::Milliseconds,
        Self::Seconds,
        Self::Minutes,
        Self::Hours,
        Self::Days,
        Self::JulianYears,
        Self::JulianCenturies,
    ];

    /// Returns the unit symbol.
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Attoseconds => "as",
            Self::Femtoseconds => "fs",
            Self::Picoseconds => "ps",
            Self::Nanoseconds => "ns",
            Self::Microseconds => "µs",
            Self::Milliseconds => "ms",
            Self::Seconds => "s",
            Self::Minutes => "min",
            Self::Hours => "h",
            Self::Days => "d",
            Self::JulianYears => "a",
            Self::JulianCenturies => "cy",
        }
    }

    /// Returns the unit for a symbol, or `None` if the symbol is unknown.
    ///
    /// Besides the symbols returned by [`symbol`](Self::symbol), `μs` (Greek mu) and `us` are
    /// accepted for microseconds.
    pub fn from_symbol(symbol: &str) -> Option<Self> {
        match symbol {
            "μs" | "us" => Some(Self::Microseconds),
            _ => Self::ALL
                .iter()
                .copied()
                .find(|unit| unit.symbol() == symbol),
        }
    }

    /// Returns the exact number of attoseconds in one unit.
    const fn attoseconds(self) -> i128 {
        match self.scale() {
            Scale::Subsecond(attoseconds) => attoseconds as i128,
            Scale::Seconds(seconds) => seconds as i128 * ATTOSECONDS_IN_SECOND as i128,
        }
    }

    const fn scale(self) -> Scale {
        match self {
            Self::Attoseconds => Scale::Subsecond(1),
            Self::Femtoseconds => Scale::Subsecond(ATTOSECONDS_IN_FEMTOSECOND),
            Self::Picoseconds => Scale::Subsecond(ATTOSECONDS_IN_PICOSECOND),
            Self::Nanoseconds => Scale::Subsecond(ATTOSECONDS_IN_NANOSECOND),
            Self::Microseconds => Scale::Subsecond(ATTOSECONDS_IN_MICROSECOND),
            Self::Milliseconds => Scale::Subsecond(ATTOSECONDS_IN_MILLISECOND),
            Self::Seconds => Scale::Seconds(1),
            Self::Minutes => Scale::Seconds(SECONDS_PER_MINUTE),
            Self::Hours => Scale::Seconds(SECONDS_PER_HOUR),
            Self::Days => Scale::Seconds(SECONDS_PER_DAY),
            Self::JulianYears => Scale::Seconds(SECONDS_PER_JULIAN_YEAR),
            Self::JulianCenturies => Scale::Seconds(SECONDS_PER_JULIAN_CENTURY),
        }
    }
}

/// Rounds to the nearest integer, with ties away from zero, saturating at the `i128` bounds.
const fn round_half_away_from_zero(value: f64) -> i128 {
    let truncated = value as i128;
    let remainder = value - truncated as f64;
    if remainder >= 0.5 {
        truncated.saturating_add(1)
    } else if remainder <= -0.5 {
        truncated.saturating_sub(1)
    } else {
        truncated
    }
}

impl Display for TimeDeltaUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.pad(self.symbol())
    }
}

impl FromStr for TimeDeltaUnit {
    type Err = ParseUnitError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Self::from_symbol(s).ok_or(ParseUnitError::UnknownUnit)
    }
}

impl TimeDelta {
    /// Creates a `TimeDelta` from a value in the given unit.
    ///
    /// The whole part of `value` is converted exactly, so that e.g. an integral number of
    /// nanoseconds equals the result of [`TimeDelta::from_nanoseconds`]. The fractional part is
    /// rounded.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not finite or the result is out of range.
    pub fn try_new_in(
        value: f64,
        unit: TimeDeltaUnit,
    ) -> core::result::Result<Self, InvalidFloatSeconds> {
        let error = || {
            InvalidFloatSeconds(match unit.scale() {
                Scale::Subsecond(attoseconds) => {
                    value / (ATTOSECONDS_IN_SECOND / attoseconds) as f64
                }
                Scale::Seconds(seconds) => value * seconds as f64,
            })
        };
        Self::checked_from_f64_in(value, unit).ok_or_else(error)
    }

    /// Converts a value in the given unit, or returns `None` if it is not finite or out of
    /// range.
    ///
    /// Every `f64` constructor of [`TimeDelta`] goes through this function, so that they all
    /// round the same way.
    pub(super) const fn checked_from_f64_in(value: f64, unit: TimeDeltaUnit) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        let per_unit = unit.attoseconds();
        // Casting truncates towards zero and saturates, so an out-of-range whole part fails the
        // checked arithmetic or the range check below.
        let whole = value as i128;
        let fraction = round_half_away_from_zero((value - whole as f64) * per_unit as f64);
        match whole.checked_mul(per_unit) {
            Some(whole) => match whole.checked_add(fraction) {
                Some(attoseconds) => Self::from_total_attoseconds(attoseconds),
                None => None,
            },
            None => None,
        }
    }

    /// Creates a `TimeDelta` from a whole number of the given unit, or returns `None` if the
    /// result is out of range.
    ///
    /// Unlike [`try_new_in`](Self::try_new_in), the conversion is exact for any count.
    pub const fn checked_new_in(count: i128, unit: TimeDeltaUnit) -> Option<Self> {
        match count.checked_mul(unit.attoseconds()) {
            Some(attoseconds) => Self::from_total_attoseconds(attoseconds),
            None => None,
        }
    }

    const fn from_total_attoseconds(attoseconds: i128) -> Option<Self> {
        let seconds = attoseconds.div_euclid(ATTOSECONDS_IN_SECOND as i128);
        if seconds < i64::MIN as i128 || seconds > i64::MAX as i128 {
            return None;
        }
        // The remainder lies in [0, 10¹⁸), which fits an i64.
        let subsecond = attoseconds.rem_euclid(ATTOSECONDS_IN_SECOND as i128) as i64;
        Some(Self::new(seconds as i64, subsecond))
    }

    /// Returns the value in the given unit.
    ///
    /// The conversion is lossy for large values. Do arithmetic on [`TimeDelta`] and convert at
    /// the end to preserve precision.
    pub const fn to(self, unit: TimeDeltaUnit) -> f64 {
        // Give both parts the sign of the delta so that they do not cancel.
        let (whole, attos) = if self.seconds() < 0 && self.attoseconds() > 0 {
            (
                self.seconds() + 1,
                self.attoseconds() - ATTOSECONDS_IN_SECOND,
            )
        } else {
            (self.seconds(), self.attoseconds())
        };
        match unit.scale() {
            Scale::Subsecond(attoseconds) => {
                let per_second = ATTOSECONDS_IN_SECOND / attoseconds;
                whole as f64 * per_second as f64 + attos as f64 / attoseconds as f64
            }
            Scale::Seconds(seconds) => {
                (whole as f64 + attos as f64 / ATTOSECONDS_IN_SECOND as f64) / seconds as f64
            }
        }
    }

    /// Returns a value that displays the delta in the given unit.
    pub const fn display_in(self, unit: TimeDeltaUnit) -> DisplayIn {
        DisplayIn::new(self.to(unit), unit.symbol())
    }
}

impl FromStr for TimeDelta {
    type Err = ParseUnitError;

    /// Parses a finite value followed by a unit, e.g. `"90 min"` or `"1.5d"`.
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let (value, unit) = parse_value_and_unit(s, TimeDeltaUnit::from_symbol)?;
        Self::try_new_in(value, unit).map_err(|_| ParseUnitError::OutOfRange)
    }
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_try_new_in_and_to() {
        let d = TimeDelta::try_new_in(90.0, TimeDeltaUnit::Minutes).unwrap();
        assert_eq!(d, TimeDelta::from_seconds(5400));
        assert_eq!(d.to(TimeDeltaUnit::Hours), 1.5);
        assert_eq!(d.to(TimeDeltaUnit::Milliseconds), 5.4e6);
        let ms = TimeDelta::try_new_in(1.5, TimeDeltaUnit::Milliseconds).unwrap();
        assert_eq!(ms, TimeDelta::from_attoseconds(1_500_000_000_000_000));
        assert_eq!(ms.to(TimeDeltaUnit::Microseconds), 1500.0);
        let y = TimeDelta::try_new_in(1.0, TimeDeltaUnit::JulianYears).unwrap();
        assert_eq!(y.to(TimeDeltaUnit::Days), 365.25);
        assert!(TimeDelta::try_new_in(f64::NAN, TimeDeltaUnit::Seconds).is_err());
        assert!(TimeDelta::try_new_in(1e300, TimeDeltaUnit::Days).is_err());
    }

    #[test]
    fn test_try_new_in_is_exact_for_whole_values() {
        let ns = 123_456_789_012;
        assert_eq!(
            TimeDelta::try_new_in(ns as f64, TimeDeltaUnit::Nanoseconds),
            Ok(TimeDelta::from_nanoseconds(ns))
        );
        let fs = -987_654_321_123_456;
        assert_eq!(
            TimeDelta::try_new_in(fs as f64, TimeDeltaUnit::Femtoseconds),
            Ok(TimeDelta::from_femtoseconds(fs))
        );
        assert_eq!(
            TimeDelta::try_new_in(-1.5, TimeDeltaUnit::Days),
            Ok(TimeDelta::from_seconds(-129_600))
        );
    }

    #[test]
    fn test_f64_constructors_agree() {
        for value in [0.7, -0.7, 1.5, -2.25, 123_456.789, 1e-18, 0.1 + 0.2] {
            assert_eq!(
                Ok(TimeDelta::from_seconds_f64(value)),
                TimeDelta::try_new_in(value, TimeDeltaUnit::Seconds)
            );
            assert_eq!(
                Ok(TimeDelta::from_seconds_f64(value)),
                TimeDelta::try_from_seconds_f64(value)
            );
            assert_eq!(
                Ok(TimeDelta::from_days_f64(value)),
                TimeDelta::try_new_in(value, TimeDeltaUnit::Days)
            );
            assert_eq!(
                value * TimeDelta::from_seconds(1),
                TimeDelta::from_seconds_f64(value)
            );
        }
    }

    #[test]
    fn test_checked_new_in() {
        let ns = 1_700_000_000_123_456_789;
        assert_eq!(
            TimeDelta::checked_new_in(ns, TimeDeltaUnit::Nanoseconds),
            Some(TimeDelta::from_nanoseconds(ns as i64))
        );
        assert_eq!(
            TimeDelta::checked_new_in(-3, TimeDeltaUnit::JulianCenturies),
            Some(TimeDelta::from_seconds(-3 * SECONDS_PER_JULIAN_CENTURY))
        );
        assert_eq!(
            TimeDelta::checked_new_in(10_000_000_000_000_000_001, TimeDeltaUnit::Attoseconds),
            Some(TimeDelta::new(10, 1))
        );
        assert_eq!(
            TimeDelta::checked_new_in(i64::MAX as i128, TimeDeltaUnit::JulianCenturies),
            None
        );
        assert_eq!(
            TimeDelta::checked_new_in(i128::MAX, TimeDeltaUnit::Seconds),
            None
        );
    }

    #[test]
    fn test_try_new_in_reports_seconds() {
        assert_eq!(
            TimeDelta::try_new_in(1e300, TimeDeltaUnit::Days),
            Err(InvalidFloatSeconds(8.64e304))
        );
        assert!(TimeDelta::try_new_in(1e300, TimeDeltaUnit::Attoseconds).is_err());
        assert!(TimeDelta::try_new_in(-1e20, TimeDeltaUnit::Seconds).is_err());
    }

    #[test]
    fn test_negative_subsecond_units() {
        let d = TimeDelta::try_new_in(-1.5, TimeDeltaUnit::Milliseconds).unwrap();
        assert_eq!(d.to(TimeDeltaUnit::Milliseconds), -1.5);
        assert_eq!(d.to(TimeDeltaUnit::Seconds), -0.0015);
    }

    #[test]
    fn test_symbols_round_trip() {
        for unit in TimeDeltaUnit::ALL {
            assert_eq!(unit.symbol().parse::<TimeDeltaUnit>(), Ok(*unit));
            assert_eq!(unit.to_string(), unit.symbol());
        }
        assert_eq!("us".parse(), Ok(TimeDeltaUnit::Microseconds));
        assert_eq!("μs".parse(), Ok(TimeDeltaUnit::Microseconds));
    }

    #[test]
    fn test_from_str() {
        assert_eq!("90 min".parse(), Ok(TimeDelta::from_seconds(5400)));
        assert_eq!("1.5d".parse(), Ok(TimeDelta::from_seconds(129_600)));
        assert_eq!(
            "5 km".parse::<TimeDelta>(),
            Err(ParseUnitError::UnknownUnit)
        );
        assert_eq!(
            "1e300 a".parse::<TimeDelta>(),
            Err(ParseUnitError::OutOfRange)
        );
    }

    #[test]
    fn test_display_in() {
        let d = TimeDelta::from_seconds(5400);
        assert_eq!(
            format!("{:.1}", d.display_in(TimeDeltaUnit::Hours)),
            "1.5 h"
        );
    }
}
