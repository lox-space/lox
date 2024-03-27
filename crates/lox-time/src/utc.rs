use std::fmt::Display;

use crate::base_time::BaseTime;
use thiserror::Error;

use crate::calendar_dates::{CalendarDate, Date};
use crate::errors::LoxTimeError;
use crate::julian_dates::{Epoch, JulianDate, Unit};
use crate::subsecond::Subsecond;
use crate::wall_clock::WallClock;

pub mod transformations;

/// A UTC timestamp with additional support for fractional seconds represented with femtosecond
/// precision.
///
/// The `UTC` struct provides the ability to represent leap seconds by setting the `second`
/// component to 60. However, it has no awareness of whether a user-specified leap second is valid.
/// It is intended strictly as an IO time format which must be converted to a continuous time format
/// to be used in calculations.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
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
            self.hour, self.minute, self.second, self.subsecond
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
        self.subsecond.millisecond()
    }

    fn microsecond(&self) -> i64 {
        self.subsecond.microsecond()
    }

    fn nanosecond(&self) -> i64 {
        self.subsecond.nanosecond()
    }

    fn picosecond(&self) -> i64 {
        self.subsecond.picosecond()
    }

    fn femtosecond(&self) -> i64 {
        self.subsecond.femtosecond()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UTCDateTime {
    date: Date,
    time: UTC,
}

#[derive(Clone, Copy, Debug, Error, PartialEq)]
#[error("UTC is not defined for dates before 1960-01-01")]
/// UTC is not defined for dates before 1960-01-01. Attempting to create a `UTCDateTime` with such
/// a date results in this error.
pub struct UTCUndefinedError;

impl UTCDateTime {
    pub fn new(date: Date, time: UTC) -> Result<Self, UTCUndefinedError> {
        // TODO: This is a naïve check that assumes the input calendar is Gregorian. We need the
        // ability to convert dates between calendars to make this check more robust.
        if date.year() <= 1959 {
            Err(UTCUndefinedError)
        } else {
            Ok(Self { date, time })
        }
    }

    fn from_base_time(base_time: BaseTime) -> Result<Self, UTCUndefinedError> {
        let time = UTC {
            hour: base_time.hour() as u8,
            minute: base_time.minute() as u8,
            second: base_time.second() as u8,
            subsecond: base_time.subsecond,
        };
        let date = base_time.calendar_date();
        Self::new(date, time)
    }

    pub fn date(&self) -> Date {
        self.date
    }

    pub fn time(&self) -> UTC {
        self.time
    }
}

/// Since Julian dates are unable to represent leap seconds unambiguously, this implementation
/// returns pseudo-Julian dates following the ERFA convention such that, if the input is a leap
/// second, the Julian date is the same as the previous second.
impl JulianDate for UTCDateTime {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        let mut base_time = BaseTime::from_utc_datetime(*self);
        if self.time.second == 60 {
            base_time.seconds -= 1;
        }
        base_time.julian_date(epoch, unit)
    }

    fn two_part_julian_date(&self) -> (f64, f64) {
        let jd = self.julian_date(Epoch::JulianDate, Unit::Days);
        (jd.trunc(), jd.fract())
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use rstest::rstest;

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

    #[rstest]
    #[case::ok(
        Date::new(2021, 1, 1).unwrap(),
        Ok(UTCDateTime {
            date: Date::new(2021, 1, 1).unwrap(),
            time: UTC::default(),
        }),
    )]
    #[case::y1960(
        Date::new(1960, 1, 1).unwrap(),
        Ok(UTCDateTime {
            date: Date::new(1960, 1, 1).unwrap(),
            time: UTC::default(),
        }),
    )]
    #[case::before_1960(
        Date::new(1959, 12, 31).unwrap(),
        Err(UTCUndefinedError),
    )]
    fn test_utc_datetime_new(
        #[case] date: Date,
        #[case] expected: Result<UTCDateTime, UTCUndefinedError>,
    ) {
        let time = UTC::default();
        let actual = UTCDateTime::new(date, time);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::non_leap_second(
        UTCDateTime::new(Date::new(2000, 1, 1).unwrap(), UTC::default()).unwrap(),
        2451544.5,
    )]
    #[case::leap_second(
        UTCDateTime::new(Date::new(1999, 12, 31).unwrap(), UTC::new(23, 59, 60, Subsecond::default()).unwrap()).unwrap(),
        2451544.499988426,
    )]
    fn test_utc_datetime_julian_date(#[case] datetime: UTCDateTime, #[case] expected: f64) {
        let actual = datetime.julian_date(Epoch::JulianDate, Unit::Days);
        assert_float_eq!(expected, actual, rel <= 1e-9);
    }
}
