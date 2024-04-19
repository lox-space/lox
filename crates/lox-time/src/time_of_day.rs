use std::fmt::Display;

use crate::subsecond::{InvalidSubsecond, Subsecond};

use thiserror::Error;

#[derive(Debug, Copy, Clone, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimeOfDayError {
    #[error("hour must be in the range [0..24) but was {0}")]
    InvalidHour(u8),
    #[error("minute must be in the range [0..60) but was {0}")]
    InvalidMinute(u8),
    #[error("second must be in the range [0..61) but was {0}")]
    InvalidSecond(u8),
    #[error("second must be in the range [0..86401) but was {0}")]
    InvalidSecondOfDay(u64),
    #[error("leap seconds are only valid at the end of the day")]
    InvalidLeapSecond,
    #[error(transparent)]
    InvalidSubsecond(#[from] InvalidSubsecond),
}

/// `CivilTime` is the trait by which high-precision time representations expose human-readable time
/// components.
pub trait CivilTime {
    fn time(&self) -> TimeOfDay;

    fn hour(&self) -> u8 {
        self.time().hour()
    }

    fn minute(&self) -> u8 {
        self.time().minute()
    }

    fn second(&self) -> u8 {
        self.time().second()
    }

    fn millisecond(&self) -> i64 {
        self.time().subsecond().millisecond()
    }

    fn microsecond(&self) -> i64 {
        self.time().subsecond().microsecond()
    }

    fn nanosecond(&self) -> i64 {
        self.time().subsecond().nanosecond()
    }

    fn picosecond(&self) -> i64 {
        self.time().subsecond().picosecond()
    }

    fn femtosecond(&self) -> i64 {
        self.time().subsecond().femtosecond()
    }
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
        if !(0..61).contains(&second) {
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
        if !(0..86401).contains(&second_of_day) {
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

    pub fn hour(&self) -> u8 {
        self.hour
    }

    pub fn minute(&self) -> u8 {
        self.minute
    }

    pub fn second(&self) -> u8 {
        self.second
    }

    pub fn subsecond(&self) -> Subsecond {
        self.subsecond
    }
}

impl Display for TimeOfDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let precision = f.precision().unwrap_or(3);
        write!(
            f,
            "{:02}:{:02}:{:02}{}",
            self.hour,
            self.minute,
            self.second,
            format!("{:.*}", precision, self.subsecond).trim_start_matches('0')
        )
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

    #[test]
    fn test_time_of_day_display() {
        let subsecond = Subsecond::new(0.123456789123456).unwrap();
        let time = TimeOfDay::new(12, 0, 0).unwrap().with_subsecond(subsecond);
        assert_eq!(format!("{}", time), "12:00:00.123");
        assert_eq!(format!("{:.15}", time), "12:00:00.123456789123456");
    }
}
