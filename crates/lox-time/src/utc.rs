// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Coordinated Universal Time (UTC) with leap-second support.
//!
//! [`Utc`] is a leap-second aware representation of UTC datetimes. Due to the complexity
//! inherent in working with leap seconds, it is segregated from the continuous time formats
//! and is used exclusively as an I/O format.

use alloc::borrow::ToOwned;
use alloc::string::String;
use core::fmt::{self, Display, Formatter};
use core::str::FromStr;

use lox_approx::{ApproxEq, ApproxEqResults};
use lox_core::time::constants::{
    ATTOSECONDS_IN_NANOSECOND, ATTOSECONDS_IN_SECOND, SECONDS_PER_DAY, SECONDS_PER_HALF_DAY,
};
use thiserror::Error;

use crate::TimeDelta;
use crate::calendar_dates::{CalendarDate, Date, DateError, UNIX_EPOCH};
use crate::julian_dates::JulianDate;
use crate::subsecond::Subsecond;
use crate::time_of_day::{CivilTime, TimeOfDay, TimeOfDayError};
use crate::utc::leap_seconds::{DefaultLeapSecondsProvider, LeapSecondsProvider};

const NANOSECONDS_IN_SECOND: i64 = ATTOSECONDS_IN_SECOND / ATTOSECONDS_IN_NANOSECOND;

pub mod leap_seconds;
pub mod transformations;

/// Error type returned when attempting to construct a [Utc] instance from invalid inputs.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum UtcError {
    /// Invalid date component.
    #[error(transparent)]
    DateError(#[from] DateError),
    /// Invalid time-of-day component.
    #[error(transparent)]
    TimeError(#[from] TimeOfDayError),
    /// Second 60 was specified on a date without a leap second.
    #[error("no leap second on {0}")]
    NonLeapSecondDate(Date),
    /// The input string is not valid ISO 8601.
    #[error("invalid ISO string `{0}`")]
    InvalidIsoString(String),
}

/// Coordinated Universal Time.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Utc {
    date: Date,
    time: TimeOfDay,
}

impl Utc {
    /// Creates a new [Utc] instance from the given [Date] and [TimeOfDay], with leap second
    /// validation provided by the [LeapSecondsProvider].
    ///
    /// # Errors
    ///
    /// - [UtcError::NonLeapSecondDate] if `time.seconds` is 60 seconds and the date is not a leap
    ///   second date.
    pub fn new(
        date: Date,
        time: TimeOfDay,
        provider: &impl LeapSecondsProvider,
    ) -> Result<Self, UtcError> {
        if time.second() == 60 && !provider.is_leap_second_date(date) {
            return Err(UtcError::NonLeapSecondDate(date));
        }
        Ok(Self { date, time })
    }

    /// Creates a [Utc] directly from a [Date] and [TimeOfDay] without
    /// leap-second validation. Use when the caller has already verified the
    /// inputs (e.g. for compile-time-known reference datetimes that aren't
    /// leap seconds).
    pub const fn new_unchecked(date: Date, time: TimeOfDay) -> Self {
        Self { date, time }
    }

    /// Returns a new [UtcBuilder].
    pub fn builder() -> UtcBuilder {
        UtcBuilder::default()
    }

    /// Constructs a new [Utc] instance from the given ISO 8601 string, with leap second validation
    /// provided by the [LeapSecondsProvider].
    ///
    /// # Errors
    ///
    /// - [UtcError::InvalidIsoString] if the input string is not a valid ISO 8601 string.
    /// - [UtcError::DateError] if the date component of the string is invalid.
    /// - [UtcError::TimeError] if the time component of the string is invalid.
    /// - [UtcError::NonLeapSecondDate] if the time component is 60 seconds and the date is not a
    ///   leap second date.
    pub fn from_iso_with_provider<T: LeapSecondsProvider>(
        iso: &str,
        provider: &T,
    ) -> Result<Self, UtcError> {
        let iso = iso.strip_suffix('Z').unwrap_or(iso);

        let Some((date, time_and_scale)) = iso.split_once('T') else {
            return Err(UtcError::InvalidIsoString(iso.to_owned()));
        };

        let mut parts = time_and_scale.split_whitespace();
        let (time, scale_abbrv) = match (parts.next(), parts.next(), parts.next()) {
            (Some(a), Some(b), None) => (a, b),
            _ => (time_and_scale, ""),
        };

        if !scale_abbrv.is_empty() && scale_abbrv != "UTC" {
            return Err(UtcError::InvalidIsoString(iso.to_owned()));
        }

        let date: Date = date.parse()?;
        let time: TimeOfDay = time.parse()?;

        Utc::new(date, time, provider)
    }

    /// Constructs a new [Utc] instance from the given ISO 8601 string, with leap second validation
    /// provided by [DefaultLeapSecondsProvider].
    pub fn from_iso(iso: &str) -> Result<Self, UtcError> {
        Self::from_iso_with_provider(iso, &DefaultLeapSecondsProvider)
    }

    /// Constructs a new [Utc] instance from a [TimeDelta] relative to J2000.
    ///
    /// Note that this constructor is not leap-second aware.
    pub fn from_delta(delta: TimeDelta) -> Self {
        let seconds = delta.seconds();
        let subsecond = Subsecond::from_attoseconds(delta.attoseconds());
        let date = Date::from_seconds_since_j2000(seconds);
        let time = TimeOfDay::from_seconds_since_j2000(seconds).with_subsecond(subsecond);
        Self { date, time }
    }
}

impl Utc {
    /// Returns the time since J2000, ignoring leap seconds.
    ///
    /// # Panics
    ///
    /// Panics if the number of seconds does not fit in an `i64`, which only happens for years
    /// beyond about ±2.9 × 10¹¹. Use [`Utc::checked_to_delta`] to handle that case.
    pub const fn to_delta(&self) -> TimeDelta {
        match self.checked_to_delta() {
            Some(delta) => delta,
            None => panic!("the time since J2000 is out of range"),
        }
    }

    /// Returns the time since J2000, ignoring leap seconds, or `None` if the number of seconds
    /// does not fit in an `i64`.
    pub const fn checked_to_delta(&self) -> Option<TimeDelta> {
        // The whole sum is formed in `i128`: for distant years the day number alone overflows
        // an `i64`, and within a day of `i64::MIN` the start of the day does even when the
        // instant does not.
        let seconds = self.date.j2000_day_number_i128() * SECONDS_PER_DAY as i128
            + self.time.second_of_day() as i128
            - SECONDS_PER_HALF_DAY as i128;
        if seconds < i64::MIN as i128 || seconds > i64::MAX as i128 {
            return None;
        }
        Some(TimeDelta::new(
            seconds as i64,
            self.time.subsecond().as_attoseconds(),
        ))
    }

    /// Constructs a [Utc] instance from a Unix timestamp.
    ///
    /// The instant is `seconds + nanoseconds × 10⁻⁹` seconds after 1970-01-01T00:00:00 UTC.
    /// `nanoseconds` may be negative or exceed one second; the excess is carried into
    /// `seconds`. Unix time ignores leap seconds, so a timestamp never resolves to 23:59:60.
    ///
    /// Returns `None` if the instant is outside the range of [Utc].
    pub fn from_unix_timestamp(seconds: i64, nanoseconds: i64) -> Option<Self> {
        let seconds = seconds.checked_add(nanoseconds.div_euclid(NANOSECONDS_IN_SECOND))?;
        let attoseconds = nanoseconds.rem_euclid(NANOSECONDS_IN_SECOND) * ATTOSECONDS_IN_NANOSECOND;
        TimeDelta::new(seconds, attoseconds)
            .checked_add(UNIX_EPOCH)
            .map(Self::from_delta)
    }

    /// Returns the Unix timestamp of this instant as whole seconds and a nanosecond remainder
    /// in `[0, 10⁹)`, rounded down to whole nanoseconds.
    ///
    /// Unix time ignores leap seconds, so 23:59:60 has the same timestamp as 00:00:00 of the
    /// next day.
    ///
    /// Returns `None` if the number of seconds overflows an `i64`.
    pub fn to_unix_timestamp(&self) -> Option<(i64, u32)> {
        let delta = self.checked_to_delta()?.checked_sub(UNIX_EPOCH)?;
        let nanoseconds = (delta.attoseconds() / ATTOSECONDS_IN_NANOSECOND) as u32;
        Some((delta.seconds(), nanoseconds))
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

impl JulianDate for Utc {
    fn to_delta(&self) -> TimeDelta {
        Utc::to_delta(self)
    }
}

impl ApproxEq for Utc {
    fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> ApproxEqResults {
        self.to_delta().approx_eq(&rhs.to_delta(), atol, rtol)
    }
}

/// A builder for constructing [Utc] instances piecewise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtcBuilder {
    date: Result<Date, DateError>,
    time: Result<TimeOfDay, TimeOfDayError>,
}

impl Default for UtcBuilder {
    /// Returns a new [UtcBuilder] at 2000-01-01T00:00:00.000 UTC.
    fn default() -> Self {
        Self {
            date: Ok(Date::default()),
            time: Ok(TimeOfDay::default()),
        }
    }
}

impl UtcBuilder {
    /// Sets the year, month and day fields of the [Utc] instance being built.
    pub fn with_ymd(self, year: i64, month: u8, day: u8) -> Self {
        Self {
            date: Date::new(year, month, day),
            ..self
        }
    }

    /// Sets the hour, minute, second and subsecond fields of the [Utc] instance being built.
    pub fn with_hms(self, hour: u8, minute: u8, seconds: f64) -> Self {
        Self {
            time: TimeOfDay::from_hms(hour, minute, seconds),
            ..self
        }
    }

    /// Constructs the [Utc] instance with leap second validation provided by the given
    /// [LeapSecondsProvider].
    pub fn build_with_provider(self, provider: &impl LeapSecondsProvider) -> Result<Utc, UtcError> {
        let date = self.date?;
        let time = self.time?;
        Utc::new(date, time, provider)
    }

    /// Constructs the [Utc] instance with leap second validation provided by [DefaultLeapSecondsProvider].
    pub fn build(self) -> Result<Utc, UtcError> {
        self.build_with_provider(&DefaultLeapSecondsProvider)
    }
}

/// The `utc` macro simplifies the creation of [Utc] instances.
///
/// # Examples
///
/// ```rust
/// use lox_time::utc;
/// use lox_time::utc::Utc;
///
/// utc!(2000, 1, 2); // 2000-01-02T00:00:00.000 UTC
/// utc!(2000, 1, 2, 3); // 2000-01-01T03:00:00.000 UTC
/// utc!(2000, 1, 2, 3, 4); // 2000-01-01T03:04:00.000 UTC
/// utc!(2000, 1, 2, 3, 4, 5.6); // 2000-01-01T03:04:05.600 UTC
/// ```
#[macro_export]
macro_rules! utc {
    ($year:literal, $month:literal, $day:literal) => {
        $crate::utc::Utc::builder()
            .with_ymd($year, $month, $day)
            .build()
    };
    ($year:literal, $month:literal, $day:literal, $hour:literal) => {
        $crate::utc::Utc::builder()
            .with_ymd($year, $month, $day)
            .with_hms($hour, 0, 0.0)
            .build()
    };
    ($year:literal, $month:literal, $day:literal, $hour:literal, $minute:literal) => {
        $crate::utc::Utc::builder()
            .with_ymd($year, $month, $day)
            .with_hms($hour, $minute, 0.0)
            .build()
    };
    ($year:literal, $month:literal, $day:literal, $hour:literal, $minute:literal, $second:literal) => {
        $crate::utc::Utc::builder()
            .with_ymd($year, $month, $day)
            .with_hms($hour, $minute, $second)
            .build()
    };
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString;

    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, 0, "1970-01-01T00:00:00")]
    #[case(946_728_000, 0, "2000-01-01T12:00:00")]
    #[case(1_483_228_799, 123_456_789, "2016-12-31T23:59:59.123456789")]
    #[case(-1, 500_000_000, "1969-12-31T23:59:59.5")]
    #[case(-2_208_988_800, 0, "1900-01-01T00:00:00")]
    fn test_utc_unix_timestamp(#[case] seconds: i64, #[case] nanoseconds: u32, #[case] iso: &str) {
        let utc = Utc::from_iso(iso).unwrap();
        assert_eq!(
            Utc::from_unix_timestamp(seconds, nanoseconds as i64),
            Some(utc)
        );
        assert_eq!(utc.to_unix_timestamp(), Some((seconds, nanoseconds)));
    }

    #[rstest]
    #[case(1, -500_000_000, "1970-01-01T00:00:00.5")]
    #[case(0, -1, "1969-12-31T23:59:59.999999999")]
    #[case(0, 2_500_000_000, "1970-01-01T00:00:02.5")]
    // A leap second as reported by e.g. `chrono`, with more than 10⁹ nanoseconds.
    #[case(1_483_228_799, 1_000_000_000, "2017-01-01T00:00:00")]
    fn test_utc_from_unix_timestamp_carries_nanoseconds(
        #[case] seconds: i64,
        #[case] nanoseconds: i64,
        #[case] iso: &str,
    ) {
        assert_eq!(
            Utc::from_unix_timestamp(seconds, nanoseconds),
            Some(Utc::from_iso(iso).unwrap())
        );
    }

    #[rstest]
    // The earliest timestamp whose offset from J2000 still fits in an `i64`.
    #[case(i64::MIN + 946_728_000)]
    #[case(i64::MAX)]
    fn test_utc_from_unix_timestamp_extremes(#[case] seconds: i64) {
        let utc = Utc::from_unix_timestamp(seconds, 0).unwrap();
        assert_eq!(utc.to_unix_timestamp(), Some((seconds, 0)));
    }

    #[test]
    fn test_utc_to_unix_timestamp_leap_second_is_next_midnight() {
        let leap_second = Utc::from_iso("2016-12-31T23:59:60").unwrap();
        assert_eq!(leap_second.to_unix_timestamp(), Some((1_483_228_800, 0)));
    }

    #[rstest]
    #[case("2000-01-01T00:00:00.000000000999", (946_684_800, 0))]
    #[case("1969-12-31T23:59:59.999999999999", (-1, 999_999_999))]
    fn test_utc_to_unix_timestamp_rounds_down(#[case] iso: &str, #[case] exp: (i64, u32)) {
        assert_eq!(Utc::from_iso(iso).unwrap().to_unix_timestamp(), Some(exp));
    }

    #[rstest]
    #[case(i64::MAX, 1_000_000_000)]
    #[case(i64::MIN, 0)]
    fn test_utc_from_unix_timestamp_out_of_range(#[case] seconds: i64, #[case] nanoseconds: i64) {
        assert_eq!(Utc::from_unix_timestamp(seconds, nanoseconds), None);
    }

    #[rstest]
    #[case(300_000_000_000)]
    #[case(-300_000_000_000)]
    fn test_utc_checked_to_delta_far_future_and_past(#[case] year: i64) {
        let utc = Utc::builder().with_ymd(year, 1, 1).build().unwrap();
        assert_eq!(utc.checked_to_delta(), None);
        assert_eq!(utc.to_unix_timestamp(), None);
    }

    #[test]
    fn test_utc_from_iso_distant_year() {
        let utc: Utc = "100000000000000-01-01T00:00:00".parse().unwrap();
        assert_eq!(utc.year(), 100_000_000_000_000);
        assert_eq!(utc.to_unix_timestamp(), None);
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn test_utc_to_delta_panics_out_of_range() {
        let utc = Utc::builder()
            .with_ymd(300_000_000_000, 1, 1)
            .build()
            .unwrap();
        let _ = utc.to_delta();
    }

    #[test]
    fn test_utc_to_unix_timestamp_out_of_range() {
        let utc = Utc::from_delta(TimeDelta::from_seconds(i64::MAX - 2 * SECONDS_PER_DAY));
        assert_eq!(utc.to_unix_timestamp(), None);
    }

    #[test]
    fn test_utc_display() {
        let utc = Utc::default();
        let expected = "2000-01-01T00:00:00.000 UTC".to_string();
        let actual = utc.to_string();
        assert_eq!(expected, actual);
        let expected = "2000-01-01T00:00:00.000000000000000 UTC".to_string();
        let actual = format!("{utc:.15}");
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
    fn test_utc_before_1960() {
        let actual = Utc::builder().with_ymd(1959, 12, 31).build();
        assert!(actual.is_ok());
    }

    #[test]
    fn test_utc_builder_with_provider() {
        let exp = utc!(2000, 1, 1).unwrap();
        let act = Utc::builder()
            .with_ymd(2000, 1, 1)
            .build_with_provider(&DefaultLeapSecondsProvider)
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
