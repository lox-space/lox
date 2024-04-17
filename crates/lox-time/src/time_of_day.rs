use crate::subsecond::{InvalidSubsecond, Subsecond};

use thiserror::Error;

#[derive(Debug, Copy, Clone, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimeOfDayError {
    #[error("hour must be in the range [0..24) but was {0}")]
    InvalidHour(u8),
    #[error("minute must be in the range [0..60) but was {0}")]
    InvalidMinute(u8),
    #[error("second must be in the range [0..60) but was {0}")]
    InvalidSecond(u8),
    #[error("second must be in the range [0.0..60.0) but was {0}")]
    InvalidSecondUtc(u8),
    #[error("second must be in the range [0..86400) but was {0}")]
    InvalidSecondOfDay(u64),
    #[error("second must be in the range [0.0..86401) but was {0}")]
    InvalidSecondOfDayUtc(u64),
    #[error("leap seconds are only valid at the end of the day")]
    InvalidLeapSecond,
    #[error(transparent)]
    InvalidSubsecond(#[from] InvalidSubsecond),
}

/// `CivilTime` is the trait by which high-precision time representations expose human-readable time
/// components.
pub trait CivilTime {
    fn hour(&self) -> u8;
    fn minute(&self) -> u8;
    fn second(&self) -> u8;
    fn millisecond(&self) -> i64;
    fn microsecond(&self) -> i64;
    fn nanosecond(&self) -> i64;
    fn picosecond(&self) -> i64;
    fn femtosecond(&self) -> i64;
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeOfDay {
    hour: u8,
    minute: u8,
    second: u8,
    subsecond: Subsecond,
}

impl TimeOfDay {
    pub fn new(hour: u8, minute: u8, second: u8) -> Result<Self, TimeOfDayError> {
        if !(0..24).contains(&hour) {
            return Err(TimeOfDayError::InvalidHour(hour));
        }
        if !(0..60).contains(&minute) {
            return Err(TimeOfDayError::InvalidMinute(minute));
        }
        if !(0..60).contains(&second) {
            return Err(TimeOfDayError::InvalidSecond(second));
        }
        Ok(Self {
            hour,
            minute,
            second,
            subsecond: Subsecond::default(),
        })
    }

    pub fn from_second_of_day(second_of_day: u64) -> Result<Self, TimeOfDayError> {
        if second_of_day > 86399 {
            return Err(TimeOfDayError::InvalidSecondOfDay(second_of_day));
        }
        let (hour, minute, second) = hms_from_second_of_day(second_of_day);
        Self::new(hour, minute, second)
    }

    pub fn from_hms_decimal(hour: u8, minute: u8, seconds: f64) -> Result<Self, TimeOfDayError> {
        let (second, fraction) = split_decimal_seconds(seconds);
        let subsecond = Subsecond::new(fraction).unwrap();
        Ok(Self::new(hour, minute, second)?.with_subsecond(subsecond))
    }

    pub fn with_subsecond(&mut self, subsecond: Subsecond) -> Self {
        self.subsecond = subsecond;
        *self
    }
}

impl CivilTime for TimeOfDay {
    fn hour(&self) -> u8 {
        self.hour
    }

    fn minute(&self) -> u8 {
        self.minute
    }

    fn second(&self) -> u8 {
        self.second
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

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeOfDayUtc {
    hour: u8,
    minute: u8,
    second: u8,
    subsecond: Subsecond,
}

impl TimeOfDayUtc {
    pub fn new(hour: u8, minute: u8, second: u8) -> Result<Self, TimeOfDayError> {
        if !(0..24).contains(&hour) {
            return Err(TimeOfDayError::InvalidHour(hour));
        }
        if !(0..60).contains(&minute) {
            return Err(TimeOfDayError::InvalidMinute(minute));
        }
        if !(0..61).contains(&second) {
            return Err(TimeOfDayError::InvalidSecond(second));
        }
        if second == 60 && (hour != 23 || minute != 59) {
            return Err(TimeOfDayError::InvalidLeapSecond);
        }
        Ok(Self {
            hour,
            minute,
            second,
            subsecond: Subsecond::default(),
        })
    }

    pub fn from_hms_decimal(hour: u8, minute: u8, seconds: f64) -> Result<Self, TimeOfDayError> {
        let (second, fraction) = split_decimal_seconds(seconds);
        let subsecond = Subsecond::new(fraction)
            .unwrap_or_else(|_| unreachable!("fraction should be in the range [0.0..1.0)"));
        Ok(Self::new(hour, minute, second)?.with_subsecond(subsecond))
    }

    pub fn from_second_of_day(second_of_day: u64) -> Result<Self, TimeOfDayError> {
        if second_of_day > 86400 {
            return Err(TimeOfDayError::InvalidSecondOfDayUtc(second_of_day));
        }
        let (hour, minute, second) = hms_from_second_of_day(second_of_day);
        Self::new(hour, minute, second)
    }

    pub fn with_subsecond(&mut self, subsecond: Subsecond) -> Self {
        self.subsecond = subsecond;
        *self
    }
}

impl CivilTime for TimeOfDayUtc {
    fn hour(&self) -> u8 {
        self.hour
    }

    fn minute(&self) -> u8 {
        self.minute
    }

    fn second(&self) -> u8 {
        self.second
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

fn split_decimal_seconds(value: f64) -> (u8, f64) {
    let whole = value.trunc() as u8;
    let fraction = value.fract();
    (whole, fraction)
}

fn hms_from_second_of_day(second_of_day: u64) -> (u8, u8, u8) {
    if second_of_day == 86400 {
        return (23, 59, 60);
    }
    let hour = (second_of_day / 3600) as u8;
    let minute = ((second_of_day % 3600) / 60) as u8;
    let second = (second_of_day % 60) as u8;
    (hour, minute, second)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hms_from_second_of_day() {
        assert_eq!(hms_from_second_of_day(43201), (12, 0, 1));
        assert_eq!(hms_from_second_of_day(86399), (23, 59, 59));
        assert_eq!(hms_from_second_of_day(86400), (23, 59, 60));
    }
}
