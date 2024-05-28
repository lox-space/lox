use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use itertools::Itertools;
use num::ToPrimitive;
use thiserror::Error;

use crate::calendar_dates::{CalendarDate, Date, DateError};
use crate::deltas::{TimeDelta, ToDelta};
use crate::julian_dates::JulianDate;
use crate::time_of_day::{CivilTime, TimeOfDay, TimeOfDayError};
use crate::transformations::LeapSecondsProvider;

use self::leap_seconds::BuiltinLeapSeconds;

pub mod leap_seconds;
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
    #[error("invalid ISO string `{0}`")]
    InvalidIsoString(String),
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Utc {
    date: Date,
    time: TimeOfDay,
}

impl Utc {
    pub fn new(
        date: Date,
        time: TimeOfDay,
        provider: &impl LeapSecondsProvider,
    ) -> Result<Self, UtcError> {
        if date.year() < 1960 {
            return Err(UtcError::UtcUndefined);
        }
        if time.second() == 60 && !provider.is_leap_second_date(date) {
            return Err(UtcError::NonLeapSecondDate(date));
        }
        Ok(Self { date, time })
    }

    pub fn builder() -> UtcBuilder {
        UtcBuilder::default()
    }

    pub fn from_iso_with_provider<T: LeapSecondsProvider>(
        iso: &str,
        provider: &T,
    ) -> Result<Self, UtcError> {
        let _ = iso.strip_suffix('Z');

        let Some((date, time_and_scale)) = iso.split_once('T') else {
            return Err(UtcError::InvalidIsoString(iso.to_owned()));
        };

        let (time, scale_abbrv) = time_and_scale
            .split_whitespace()
            .collect_tuple()
            .unwrap_or((time_and_scale, ""));

        if !scale_abbrv.is_empty() && scale_abbrv != "UTC" {
            return Err(UtcError::InvalidIsoString(iso.to_owned()));
        }

        let date: Date = date.parse()?;
        let time: TimeOfDay = time.parse()?;

        Utc::new(date, time, provider)
    }

    pub fn from_iso(iso: &str) -> Result<Self, UtcError> {
        Self::from_iso_with_provider(iso, &BuiltinLeapSeconds)
    }

    pub fn from_delta(delta: TimeDelta) -> Self {
        let date = Date::from_seconds_since_j2000(delta.seconds);
        let time =
            TimeOfDay::from_seconds_since_j2000(delta.seconds).with_subsecond(delta.subsecond);
        Self { date, time }
    }
}

impl ToDelta for Utc {
    fn to_delta(&self) -> TimeDelta {
        let seconds = self.date.seconds_since_j2000().to_i64().unwrap_or_else(|| {
            unreachable!(
                "seconds since J2000 for date {} are not representable as i64: {}",
                self,
                self.date.seconds_since_j2000()
            )
        }) + self.time.second_of_day();
        TimeDelta {
            seconds,
            subsecond: self.time.subsecond(),
        }
    }
}

impl Display for Utc {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let precision = f.precision().unwrap_or(3);
        write!(f, "{}T{:.*} UTC", self.date(), precision, self.time())
    }
}

impl FromStr for Utc {
    type Err = UtcError;

    fn from_str(iso: &str) -> Result<Self, Self::Err> {
        Self::from_iso(iso)
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UtcBuilder {
    date: Result<Date, DateError>,
    time: Result<TimeOfDay, TimeOfDayError>,
}

impl Default for UtcBuilder {
    fn default() -> Self {
        Self {
            date: Ok(Date::default()),
            time: Ok(TimeOfDay::default()),
        }
    }
}

impl UtcBuilder {
    pub fn with_ymd(self, year: i64, month: u8, day: u8) -> Self {
        Self {
            date: Date::new(year, month, day),
            ..self
        }
    }

    pub fn with_hms(self, hour: u8, minute: u8, seconds: f64) -> Self {
        Self {
            time: TimeOfDay::from_hms(hour, minute, seconds),
            ..self
        }
    }

    pub fn build_with_provider(self, provider: &impl LeapSecondsProvider) -> Result<Utc, UtcError> {
        let date = self.date?;
        let time = self.time?;
        Utc::new(date, time, provider)
    }

    pub fn build(self) -> Result<Utc, UtcError> {
        self.build_with_provider(&BuiltinLeapSeconds)
    }
}

#[macro_export]
macro_rules! utc {
    ($year:literal, $month:literal, $day:literal) => {
        Utc::builder().with_ymd($year, $month, $day).build()
    };
    ($year:literal, $month:literal, $day:literal, $hour:literal) => {
        Utc::builder()
            .with_ymd($year, $month, $day)
            .with_hms($hour, 0, 0.0)
            .build()
    };
    ($year:literal, $month:literal, $day:literal, $hour:literal, $minute:literal) => {
        Utc::builder()
            .with_ymd($year, $month, $day)
            .with_hms($hour, $minute, 0.0)
            .build()
    };
    ($year:literal, $month:literal, $day:literal, $hour:literal, $minute:literal, $second:literal) => {
        Utc::builder()
            .with_ymd($year, $month, $day)
            .with_hms($hour, $minute, $second)
            .build()
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

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

    #[rstest]
    #[case(utc!(2000, 1, 1), Utc::builder().with_ymd(2000, 1, 1).build())]
    #[case(utc!(2000, 1, 1, 12), Utc::builder().with_ymd(2000, 1, 1).with_hms(12, 0, 0.0).build())]
    #[case(utc!(2000, 1, 1, 12, 13), Utc::builder().with_ymd(2000, 1, 1).with_hms(12, 13, 0.0).build())]
    #[case(utc!(2000, 1, 1, 12, 13, 14.15), Utc::builder().with_ymd(2000, 1, 1).with_hms(12, 13, 14.15).build())]
    fn test_utc_macro(
        #[case] actual: Result<Utc, UtcError>,
        #[case] expected: Result<Utc, UtcError>,
    ) {
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_utc_non_leap_second_date() {
        let actual = Utc::builder()
            .with_ymd(2000, 1, 1)
            .with_hms(23, 59, 60.0)
            .build();
        let expected = Err(UtcError::NonLeapSecondDate(Date::new(2000, 1, 1).unwrap()));
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_utc_undefined() {
        let actual = Utc::builder().with_ymd(1959, 12, 31).build();
        let expected = Err(UtcError::UtcUndefined);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_utc_builder_with_provider() {
        let exp = utc!(2000, 1, 1).unwrap();
        let act = Utc::builder()
            .with_ymd(2000, 1, 1)
            .build_with_provider(&BuiltinLeapSeconds)
            .unwrap();
        assert_eq!(exp, act)
    }

    #[rstest]
    #[case("2000-01-01T00:00:00", Ok(utc!(2000, 1, 1).unwrap()))]
    #[case("2000-01-01T00:00:00 UTC", Ok(utc!(2000, 1, 1).unwrap()))]
    #[case("2000-01-01T00:00:00.000Z", Ok(utc!(2000, 1, 1).unwrap()))]
    #[case("2000-1-01T00:00:00", Err(UtcError::DateError(DateError::InvalidIsoString("2000-1-01".to_string()))))]
    #[case("2000-01-01T0:00:00", Err(UtcError::TimeError(TimeOfDayError::InvalidIsoString("0:00:00".to_string()))))]
    #[case("2000-01-01-00:00:00", Err(UtcError::InvalidIsoString("2000-01-01-00:00:00".to_string())))]
    #[case("2000-01-01T00:00:00 TAI", Err(UtcError::InvalidIsoString("2000-01-01T00:00:00 TAI".to_string())))]
    fn test_utc_from_str(#[case] iso: &str, #[case] expected: Result<Utc, UtcError>) {
        let actual: Result<Utc, UtcError> = iso.parse();
        assert_eq!(actual, expected)
    }
}
