use crate::{
    calendar_dates::Date,
    time_of_day::{CivilTime, TimeOfDay, TimeOfDayError, TimeOfDayUtc},
};

use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, PartialOrd)]
pub enum UtcError {
    #[error(transparent)]
    TimeError(#[from] TimeOfDayError),
    #[error("no leap second on {0}")]
    NonLeapSecondDate(Date),
    #[error("UTC is not defined for dates before 1960-01-01")]
    UtcUndefined,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GenericDateTime<T: CivilTime> {
    date: Date,
    time: T,
}

pub type Utc = GenericDateTime<TimeOfDayUtc>;
pub type DateTime = GenericDateTime<TimeOfDay>;

impl Date {
    pub fn with_time(self, time: TimeOfDay) -> DateTime {
        DateTime::new(self, time)
    }

    pub fn with_utc(self, time: TimeOfDayUtc) -> Result<Utc, UtcError> {
        Utc::new(self, time)
    }

    pub fn with_hms(self, hour: u8, minute: u8, second: u8) -> Result<DateTime, TimeOfDayError> {
        let time = TimeOfDay::new(hour, minute, second)?;
        Ok(DateTime::new(self, time))
    }

    pub fn with_hms_decimal(
        self,
        hour: u8,
        minute: u8,
        seconds: f64,
    ) -> Result<DateTime, TimeOfDayError> {
        let time = TimeOfDay::from_hms_decimal(hour, minute, seconds)?;
        Ok(DateTime::new(self, time))
    }

    pub fn with_hms_utc(self, hour: u8, minute: u8, second: u8) -> Result<Utc, UtcError> {
        let time = TimeOfDayUtc::new(hour, minute, second)?;
        Utc::new(self, time)
    }

    pub fn with_hms_decimal_utc(self, hour: u8, minute: u8, seconds: f64) -> Result<Utc, UtcError> {
        let time = TimeOfDayUtc::from_hms_decimal(hour, minute, seconds)?;
        Utc::new(self, time)
    }
}

impl<T: CivilTime + Copy> GenericDateTime<T> {
    pub fn date(&self) -> Date {
        self.date
    }

    pub fn time(&self) -> T {
        self.time
    }
}

impl Utc {
    pub fn new(date: Date, time: TimeOfDayUtc) -> Result<Self, UtcError> {
        if date.year() < 1960 {
            return Err(UtcError::UtcUndefined);
        }
        if time.second() == 60 && !date.is_leap_second_date() {
            return Err(UtcError::NonLeapSecondDate(date));
        }
        Ok(Self { date, time })
    }

    pub fn is_leap_second(&self) -> bool {
        let hour = self.time.hour();
        let minute = self.time.minute();
        let second = self.time.second();
        hour == 23 && minute == 59 && second == 60
    }
}

impl DateTime {
    pub fn new(date: Date, time: TimeOfDay) -> Self {
        Self { date, time }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utc_leap_second() {
        let utc = Date::new(2016, 12, 31)
            .expect("date should be valid")
            .with_hms_utc(23, 59, 60)
            .expect("time should be valid");
        assert!(utc.is_leap_second())
    }

    #[test]
    fn test_utc_invalid_leap_second_date() {
        let date = Date::new(2024, 1, 1).expect("date should be valid");
        let utc = date.with_hms_utc(23, 59, 60);
        assert_eq!(utc, Err(UtcError::NonLeapSecondDate(date)));
    }

    #[test]
    fn test_utc_undefined() {
        let date = Date::new(1959, 12, 31).expect("date should be valid");
        let utc = date.with_hms_utc(23, 59, 60);
        assert_eq!(utc, Err(UtcError::UtcUndefined));
    }
}
