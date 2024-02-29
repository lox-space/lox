use std::fmt::Display;

use num::ToPrimitive;

use crate::dates::Date;
use crate::errors::LoxTimeError;
use crate::{Subsecond, WallClock};

/// A UTC timestamp with additional support for fractional seconds represented with femtosecond
/// precision.
///
/// The `UTC` struct provides the ability to represent leap seconds by setting the `second`
/// component to 60. However, it has no awareness of whether a user-specified leap second is valid.
/// It is intended strictly as an IO time format which must be converted to a continuous time format
/// to be used in calculations.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct UTC {
    hour: u8,
    minute: u8,
    second: u8,
    subsecond: Subsecond,
}

impl UTC {
    pub fn new(
        hour: u8,
        minute: u8,
        second: u8,
        subsecond: Subsecond,
    ) -> Result<Self, LoxTimeError> {
        if !(0..24).contains(&hour) || !(0..60).contains(&minute) || !(0..61).contains(&second) {
            Err(LoxTimeError::InvalidTime(hour, minute, second))
        } else {
            Ok(Self {
                hour,
                minute,
                second,
                subsecond,
            })
        }
    }

    pub fn subsecond(&self) -> Subsecond {
        self.subsecond
    }
}

impl Display for UTC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}.{} UTC",
            self.hour,
            self.minute,
            self.second,
            self.subsecond
        )
    }
}

impl WallClock for UTC {
    fn hour(&self) -> i64 {
        self.hour as i64
    }

    fn minute(&self) -> i64 {
        self.minute as i64
    }

    fn second(&self) -> i64 {
        self.second as i64
    }

    fn millisecond(&self) -> i64 {
        self.millisecond()
    }

    fn microsecond(&self) -> i64 {
        self.microsecond()
    }

    fn nanosecond(&self) -> i64 {
        self.nanosecond()
    }

    fn picosecond(&self) -> i64 {
        self.picosecond()
    }

    fn femtosecond(&self) -> i64 {
        self.femtosecond()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UTCDateTime {
    date: Date,
    time: UTC,
}

impl UTCDateTime {
    pub fn new(date: Date, time: UTC) -> Self {
        Self { date, time }
    }

    pub fn date(&self) -> Date {
        self.date
    }

    pub fn time(&self) -> UTC {
        self.time
    }
}

#[cfg(test)]
mod tests {
    use crate::dates::Calendar::Gregorian;

    use super::*;

    const TIME: UTC = UTC {
        hour: 12,
        minute: 34,
        second: 56,
        subsecond: Subsecond(0.789_123_456_789_123),
    };

    #[test]
    fn test_time_display() {
        assert_eq!("12:34:56.789.123.456.789.123 UTC", TIME.to_string());
    }

    #[test]
    fn test_utc_wall_clock_hour() {
        assert_eq!(TIME.hour(), TIME.hour as i64);
    }

    #[test]
    fn test_utc_wall_clock_minute() {
        assert_eq!(TIME.minute(), TIME.minute as i64);
    }

    #[test]
    fn test_utc_wall_clock_second() {
        assert_eq!(TIME.second(), TIME.second as i64);
    }

    #[test]
    fn test_utc_wall_clock_millisecond() {
        assert_eq!(TIME.millisecond(), 789);
    }

    #[test]
    fn test_utc_wall_clock_microsecond() {
        assert_eq!(TIME.microsecond(), 123);
    }

    #[test]
    fn test_utc_wall_clock_nanosecond() {
        assert_eq!(TIME.nanosecond(), 456);
    }

    #[test]
    fn test_utc_wall_clock_picosecond() {
        assert_eq!(TIME.picosecond(), 789);
    }

    #[test]
    fn test_utc_wall_clock_femtosecond() {
        assert_eq!(TIME.femtosecond(), 123);
    }

    #[test]
    fn test_utc_datetime_new() {
        let date = Date::new_unchecked(Gregorian, 2021, 1, 1);
        let time = UTC::new(12, 34, 56, Subsecond::default()).expect("time should be valid");
        let expected = UTCDateTime { date, time };
        let actual = UTCDateTime::new(date, time);
        assert_eq!(expected, actual);
    }
}
