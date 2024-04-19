use std::fmt::{self, Display, Formatter};

use num::ToPrimitive;
use thiserror::Error;

use crate::base_time::BaseTime;
use crate::calendar_dates::{CalendarDate, Date, DateError};
use crate::constants::i64::SECONDS_PER_HALF_DAY;
use crate::deltas::TimeDelta;
use crate::julian_dates::{Epoch, JulianDate, Unit};
use crate::subsecond::Subsecond;
use crate::time_of_day::{CivilTime, TimeOfDay, TimeOfDayError};

pub mod transformations;

#[derive(Debug, Clone, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum UtcError {
    #[error(transparent)]
    DateError(#[from] DateError),
    #[error(transparent)]
    TimeError(#[from] TimeOfDayError),
    #[error("no leap second on {0}")]
    NonLeapSecondDate(Date),
    #[error("UTC is not defined for dates before 1960-01-01")]
    UtcUndefined,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Utc {
    date: Date,
    time: TimeOfDay,
}

impl Utc {
    pub fn new(year: i64, month: u8, day: u8) -> Result<Self, UtcError> {
        if year < 1960 {
            return Err(UtcError::UtcUndefined);
        }
        let date = Date::new(year, month, day)?;
        Ok(Self {
            date,
            time: TimeOfDay::default(),
        })
    }

    pub fn from_delta(delta: TimeDelta) -> Self {
        let date = Date::from_seconds_since_j2000(delta.seconds);
        let time =
            TimeOfDay::from_seconds_since_j2000(delta.seconds).with_subsecond(delta.subsecond);
        Self { date, time }
    }

    pub fn with_time_of_day(mut self, time: TimeOfDay) -> Result<Self, UtcError> {
        if time.second() == 60 && !self.date.is_leap_second_date() {
            return Err(UtcError::NonLeapSecondDate(self.date));
        }
        self.time = time;
        Ok(self)
    }

    pub fn with_hms(self, hour: u8, minute: u8, seconds: f64) -> Result<Self, UtcError> {
        let time = TimeOfDay::from_hms(hour, minute, seconds)?;
        self.with_time_of_day(time)
    }

    pub fn to_delta(&self) -> TimeDelta {
        let seconds = self
            .date
            .seconds_since_j2000()
            .to_i64()
            .unwrap_or_else(|| unreachable!("should be representable as i64"))
            + self.time.second_of_day()
            - SECONDS_PER_HALF_DAY;
        TimeDelta {
            seconds,
            subsecond: self.time.subsecond(),
        }
    }
}

impl Display for Utc {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let precision = f.precision().unwrap_or(3);
        write!(f, "{}T{:.*} UTC", self.date(), precision, self.time(),)
    }
}

impl CalendarDate for Utc {
    fn date(&self) -> Date {
        self.date
    }
}

impl CivilTime for Utc {
    fn time(&self) -> TimeOfDay {
        self.time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utc_display() {
        let utc = Utc::default();
        let expected = "2000-01-01T00:00:00.000 UTC".to_string();
        let actual = utc.to_string();
        assert_eq!(expected, actual);
        let expected = "2000-01-01T00:00:00.000000000000000 UTC".to_string();
        let actual = format!("{:.15}", utc);
        assert_eq!(expected, actual);
    }
}
