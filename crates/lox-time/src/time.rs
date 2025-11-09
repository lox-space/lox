// SPDX-FileCopyrightText: 2023 Andrei Zisu <matzipan@gmail.com>
// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Add;
use std::ops::Sub;
use std::str::FromStr;

use itertools::Itertools;
use lox_core::f64;
use lox_core::i64;
use lox_core::types::units::Days;
use lox_test_utils::approx_eq::ApproxEq;
use lox_test_utils::approx_eq::results::ApproxEqResults;
use num::ToPrimitive;
use thiserror::Error;

use crate::calendar_dates::CalendarDate;
use crate::calendar_dates::Date;
use crate::calendar_dates::DateError;
use crate::deltas::TimeDelta;
use crate::deltas::ToDelta;
use crate::julian_dates::Epoch;
use crate::julian_dates::JulianDate;
use crate::julian_dates::Unit;
use crate::offsets::DefaultOffsetProvider;
use crate::offsets::Offset;
use crate::offsets::TryOffset;
use crate::subsecond::Subsecond;
use crate::time_of_day::CivilTime;
use crate::time_of_day::TimeOfDay;
use crate::time_of_day::TimeOfDayError;
use crate::time_scales::DynTimeScale;
use crate::time_scales::Tai;
use crate::time_scales::Tcb;
use crate::time_scales::Tcg;
use crate::time_scales::Tdb;
use crate::time_scales::TimeScale;
use crate::time_scales::Tt;
use crate::time_scales::Ut1;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum TimeError {
    #[error(transparent)]
    DateError(#[from] DateError),
    #[error(transparent)]
    TimeError(#[from] TimeOfDayError),
    #[error("leap seconds do not exist in continuous time scales; use `Utc` instead")]
    LeapSecondOutsideUtc,
    #[error("invalid ISO string `{0}`")]
    InvalidIsoString(String),
}

/// An instant in time in a given [TimeScale], relative to J2000.
///
/// `Time` supports femtosecond precision, but be aware that many algorithms operating on `Time`s
/// are not accurate to this level of precision.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Time<T: TimeScale> {
    scale: T,
    delta: TimeDelta,
}

pub type DynTime = Time<DynTimeScale>;

impl<T: TimeScale> Time<T> {
    /// Instantiates a [Time] in the given [TimeScale] from the count of seconds since J2000, subdivided
    /// into integral seconds and [Subsecond].
    pub const fn new(scale: T, seconds: i64, subsecond: Subsecond) -> Self {
        let delta = TimeDelta::new(seconds, subsecond.as_attoseconds());
        Self { scale, delta }
    }

    /// Instantiates a [Time] in the given [TimeScale] from a [Date] and a [TimeOfDay].
    ///
    /// # Errors
    ///
    /// * Returns `TimeError::LeapSecondsOutsideUtc` if `time` is a leap second, since leap seconds
    ///   cannot be unambiguously represented by a continuous time format.
    pub fn from_date_and_time(scale: T, date: Date, time: TimeOfDay) -> Result<Self, TimeError> {
        let mut seconds = (date.days_since_j2000() * f64::consts::SECONDS_PER_DAY)
            .to_i64()
            .unwrap_or_else(|| {
                unreachable!(
                    "seconds since J2000 for date {} are not representable as i64: {}",
                    date,
                    date.days_since_j2000()
                )
            });
        if time.second() == 60 {
            return Err(TimeError::LeapSecondOutsideUtc);
        }
        seconds += time.second_of_day();
        Ok(Self::new(scale, seconds, time.subsecond()))
    }

    /// Instantiates a [Time] in the given [TimeScale] from an ISO 8601 string.
    ///
    /// # Errors
    ///
    /// * Returns `TimeError::InvalidIsoString` if `iso` is not a valid ISO 8601 timestamp.
    pub fn from_iso(scale: T, iso: &str) -> Result<Self, TimeError> {
        let Some((date, time_and_scale)) = iso.split_once('T') else {
            return Err(TimeError::InvalidIsoString(iso.to_owned()));
        };

        let (time, scale_abbrv) = time_and_scale
            .split_whitespace()
            .collect_tuple()
            .unwrap_or((time_and_scale, ""));

        if !scale_abbrv.is_empty() && scale_abbrv != scale.abbreviation() {
            return Err(TimeError::InvalidIsoString(iso.to_owned()));
        }

        let date: Date = date.parse()?;
        let time: TimeOfDay = time.parse()?;

        Self::from_date_and_time(scale, date, time)
    }

    /// Instantiates a [Time] in the given [TimeScale] and a [TimeDelta] relative to J2000.
    pub const fn from_delta(scale: T, delta: TimeDelta) -> Self {
        Self { scale, delta }
    }

    /// Returns the [Time] at `epoch` in the given [TimeScale].
    ///
    /// Since [Time] is defined relative to J2000, this is equivalent to the delta between
    /// J2000 and `epoch`.
    pub const fn from_epoch(scale: T, epoch: Epoch) -> Self {
        match epoch {
            Epoch::JulianDate => Self {
                scale,
                delta: TimeDelta::from_seconds(-i64::consts::SECONDS_BETWEEN_JD_AND_J2000),
            },
            Epoch::ModifiedJulianDate => Self {
                scale,
                delta: TimeDelta::from_seconds(-i64::consts::SECONDS_BETWEEN_MJD_AND_J2000),
            },
            Epoch::J1950 => Self {
                scale,
                delta: TimeDelta::from_seconds(-i64::consts::SECONDS_BETWEEN_J1950_AND_J2000),
            },
            Epoch::J2000 => Self {
                scale,
                delta: TimeDelta::ZERO,
            },
        }
    }

    /// Given a Julian date, instantiates a [Time] in the specified [TimeScale], relative to
    /// `epoch`.
    pub fn from_julian_date(scale: T, julian_date: Days, epoch: Epoch) -> Self {
        let delta = TimeDelta::from_julian_date(julian_date, epoch);
        Self { scale, delta }
    }

    pub fn from_two_part_julian_date(scale: T, jd1: Days, jd2: Days) -> Self {
        let delta = TimeDelta::from_two_part_julian_date(jd1, jd2);
        Self { scale, delta }
    }

    /// Returns a [TimeBuilder] for constructing a new [Time] in the given [TimeScale].
    pub fn builder_with_scale(scale: T) -> TimeBuilder<T> {
        TimeBuilder::new(scale)
    }

    /// Returns the timescale
    pub fn scale(&self) -> T
    where
        T: Copy,
    {
        self.scale
    }

    /// Returns a new [Time] with the delta of `self` but time scale `scale`.
    ///
    /// Note that the underlying delta is simply copied – no time scale transformation takes place.
    pub fn with_scale<S: TimeScale>(&self, scale: S) -> Time<S> {
        Time::from_delta(scale, self.delta)
    }

    pub fn try_to_scale<S, P>(&self, scale: S, provider: &P) -> Result<Time<S>, P::Error>
    where
        T: Copy,
        S: TimeScale + Copy,
        P: TryOffset<T, S> + ?Sized,
    {
        let offset = provider.try_offset(self.scale, scale, self.to_delta())?;
        Ok(self.with_scale_and_delta(scale, offset))
    }

    pub fn to_scale<S>(&self, scale: S) -> Time<S>
    where
        T: Copy,
        S: TimeScale + Copy,
        DefaultOffsetProvider: Offset<T, S>,
    {
        let offset = DefaultOffsetProvider.offset(self.scale, scale, self.to_delta());
        self.with_scale_and_delta(scale, offset)
    }

    /// Returns a new [Time] with the delta of `self` adjusted by `delta`, and time scale `scale`.
    ///
    /// Note that no time scale transformation takes place beyond the adjustment specified by
    /// `delta`.
    pub fn with_scale_and_delta<S: TimeScale>(&self, scale: S, delta: TimeDelta) -> Time<S> {
        Time::from_delta(scale, self.to_delta() + delta)
    }

    /// Returns the Julian epoch as a [Time] in the given [TimeScale].
    pub fn jd0(scale: T) -> Self {
        Self::from_epoch(scale, Epoch::JulianDate)
    }

    /// Returns the modified Julian epoch as a [Time] in the given [TimeScale].
    pub fn mjd0(scale: T) -> Self {
        Self::from_epoch(scale, Epoch::ModifiedJulianDate)
    }

    /// Returns the J1950 epoch as a [Time] in the given [TimeScale].
    pub fn j1950(scale: T) -> Self {
        Self::from_epoch(scale, Epoch::J1950)
    }

    /// Returns the J2000 epoch as a [Time] in the given [TimeScale].
    pub fn j2000(scale: T) -> Self {
        Self::from_epoch(scale, Epoch::J2000)
    }

    pub fn as_seconds_and_subsecond(&self) -> Option<(i64, Subsecond)> {
        self.delta.as_seconds_and_subsecond()
    }

    /// Returns the number of whole seconds since J2000.
    pub fn seconds(&self) -> Option<i64> {
        self.as_seconds_and_subsecond().map(|(seconds, _)| seconds)
    }

    /// Returns the fraction of a second from the last whole second as an `f64`.
    pub fn subsecond(&self) -> Option<f64> {
        self.as_seconds_and_subsecond()
            .map(|(_, subsecond)| subsecond.as_seconds_f64())
    }
}

impl<T: TimeScale + std::fmt::Debug> ApproxEq for Time<T> {
    fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> ApproxEqResults {
        self.to_delta().approx_eq(&rhs.to_delta(), atol, rtol)
    }
}

impl<T: TimeScale> ToDelta for Time<T> {
    fn to_delta(&self) -> TimeDelta {
        self.delta
    }
}

impl<T: TimeScale> JulianDate for Time<T> {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        self.delta.julian_date(epoch, unit)
    }
}

impl<T: TimeScale> Display for Time<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let precision = f.precision().unwrap_or(3);
        write!(
            f,
            "{}T{:.*} {}",
            self.date(),
            precision,
            self.time(),
            self.scale.abbreviation()
        )
    }
}

impl FromStr for Time<Tai> {
    type Err = TimeError;

    fn from_str(iso: &str) -> Result<Self, Self::Err> {
        Self::from_iso(Tai, iso)
    }
}

impl FromStr for Time<Tcb> {
    type Err = TimeError;

    fn from_str(iso: &str) -> Result<Self, Self::Err> {
        Self::from_iso(Tcb, iso)
    }
}

impl FromStr for Time<Tcg> {
    type Err = TimeError;

    fn from_str(iso: &str) -> Result<Self, Self::Err> {
        Self::from_iso(Tcg, iso)
    }
}

impl FromStr for Time<Tdb> {
    type Err = TimeError;

    fn from_str(iso: &str) -> Result<Self, Self::Err> {
        Self::from_iso(Tdb, iso)
    }
}

impl FromStr for Time<Tt> {
    type Err = TimeError;

    fn from_str(iso: &str) -> Result<Self, Self::Err> {
        Self::from_iso(Tt, iso)
    }
}

impl FromStr for Time<Ut1> {
    type Err = TimeError;

    fn from_str(iso: &str) -> Result<Self, Self::Err> {
        Self::from_iso(Ut1, iso)
    }
}

impl<T: TimeScale> Add<TimeDelta> for Time<T> {
    type Output = Self;

    fn add(self, rhs: TimeDelta) -> Self::Output {
        Self {
            scale: self.scale,
            delta: self.delta + rhs,
        }
    }
}

impl<T: TimeScale> Sub<TimeDelta> for Time<T> {
    type Output = Self;

    fn sub(self, rhs: TimeDelta) -> Self::Output {
        Self {
            scale: self.scale,
            delta: self.delta - rhs,
        }
    }
}

impl<T: TimeScale> Sub<Time<T>> for Time<T> {
    type Output = TimeDelta;

    fn sub(self, rhs: Time<T>) -> Self::Output {
        self.delta - rhs.delta
    }
}

impl<T: TimeScale> CivilTime for Time<T> {
    fn time(&self) -> TimeOfDay {
        debug_assert!(self.delta.is_finite());
        let (seconds, subsecond) = self.as_seconds_and_subsecond().unwrap();
        TimeOfDay::from_seconds_since_j2000(seconds).with_subsecond(subsecond)
    }
}

impl<T: TimeScale> CalendarDate for Time<T> {
    fn date(&self) -> Date {
        debug_assert!(self.delta.is_finite());
        let seconds = self.seconds().unwrap();
        Date::from_seconds_since_j2000(seconds)
    }
}

/// `TimeBuilder` supports the construction of [Time] instances piecewise using the builder pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeBuilder<T: TimeScale> {
    scale: T,
    date: Result<Date, DateError>,
    time: Result<TimeOfDay, TimeOfDayError>,
}

impl<T: TimeScale> TimeBuilder<T> {
    /// Returns a new [TimeBuilder], equivalent to a [Time] at J2000 in the given [TimeScale].
    pub fn new(scale: T) -> Self {
        Self {
            scale,
            date: Ok(Date::default()),
            time: Ok(TimeOfDay::default()),
        }
    }

    /// Sets the `year`, `month`, and `day` of the [Time] under construction.
    pub fn with_ymd(self, year: i64, month: u8, day: u8) -> Self {
        Self {
            date: Date::new(year, month, day),
            ..self
        }
    }

    /// Sets the `year` and `day_of_year` of the [Time] under construction.
    pub fn with_doy(self, year: i64, day_of_year: u16) -> Self {
        Self {
            date: Date::from_day_of_year(year, day_of_year),
            ..self
        }
    }

    /// Sets the `hour`, `minute`, and decimal `seconds` of the [Time] under construction.
    pub fn with_hms(self, hour: u8, minute: u8, seconds: f64) -> Self {
        Self {
            time: TimeOfDay::from_hms(hour, minute, seconds),
            ..self
        }
    }

    /// Builds the [Time] instance.
    ///
    /// # Errors
    ///
    /// * [DateError] if `ymd` data passed into the builder did not correspond to a valid date;
    /// * [TimeOfDayError] if `hms` data passed into the builder did not correspond to a valid time
    ///   of day.
    pub fn build(self) -> Result<Time<T>, TimeError> {
        let date = self.date?;
        let time = self.time?;
        Time::from_date_and_time(self.scale, date, time)
    }
}

/// Convenience macro to simplify the construction of [Time] instances.
///
/// # Examples
///
/// ```
/// use lox_time::Time;
/// use lox_time::time;
/// use lox_time::time_scales::Tai;
///
///
/// time!(Tai, 2020, 1, 2); // 2020-01-02T00:00:00.000 TAI
/// time!(Tai, 2020, 1, 2, 3) ; // 2020-01-02T03:00:00.000 TAI
/// time!(Tai, 2020, 1, 2, 3, 4); // 2020-01-02T03:04:00.000 TAI
/// time!(Tai, 2020, 1, 2, 3, 4, 5.006); // 2020-01-02T03:04:05.006 TAI
/// ```
#[macro_export]
macro_rules! time {
    ($scale:expr, $year:literal, $month:literal, $day:literal) => {
        Time::builder_with_scale($scale)
            .with_ymd($year, $month, $day)
            .build()
    };
    ($scale:expr, $year:literal, $month:literal, $day:literal, $hour:literal) => {
        Time::builder_with_scale($scale)
            .with_ymd($year, $month, $day)
            .with_hms($hour, 0, 0.0)
            .build()
    };
    ($scale:expr, $year:literal, $month:literal, $day:literal, $hour:literal, $minute:literal) => {
        Time::builder_with_scale($scale)
            .with_ymd($year, $month, $day)
            .with_hms($hour, $minute, 0.0)
            .build()
    };
    ($scale:expr, $year:literal, $month:literal, $day:literal, $hour:literal, $minute:literal, $second:literal) => {
        Time::builder_with_scale($scale)
            .with_ymd($year, $month, $day)
            .with_hms($hour, $minute, $second)
            .build()
    };
}

#[cfg(test)]
mod tests {
    use lox_core::f64::consts::DAYS_PER_JULIAN_CENTURY;
    use lox_test_utils::assert_approx_eq;
    use rstest::rstest;

    use crate::Time;
    use crate::time_scales::{Tai, Tdb, Tt};
    use lox_core::i64::consts::{SECONDS_PER_DAY, SECONDS_PER_HALF_DAY};

    use super::*;

    use lox_core::i64::consts::{
        SECONDS_BETWEEN_J1950_AND_J2000, SECONDS_BETWEEN_JD_AND_J2000,
        SECONDS_BETWEEN_MJD_AND_J2000, SECONDS_PER_HOUR, SECONDS_PER_JULIAN_CENTURY,
        SECONDS_PER_MINUTE,
    };

    #[test]
    fn test_time_builder() {
        let time = Time::builder_with_scale(Tai)
            .with_ymd(2000, 1, 1)
            .build()
            .unwrap();
        assert_eq!(time.seconds(), Some(-SECONDS_PER_HALF_DAY));
        let time = Time::builder_with_scale(Tai)
            .with_ymd(2000, 1, 1)
            .with_hms(12, 0, 0.0)
            .build()
            .unwrap();
        assert_eq!(time.seconds(), Some(0));
    }

    #[test]
    fn test_time_from_seconds() {
        let scale = Tai;
        let seconds = 1234567890;
        let subsecond = Subsecond::from_f64(0.9876543210).unwrap();
        let expected = Time::new(scale, seconds, subsecond);
        let actual = Time::new(scale, seconds, subsecond);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(Epoch::JulianDate, -SECONDS_BETWEEN_JD_AND_J2000)]
    #[case(Epoch::ModifiedJulianDate, -SECONDS_BETWEEN_MJD_AND_J2000)]
    #[case(Epoch::J1950, -SECONDS_BETWEEN_J1950_AND_J2000)]
    #[case(Epoch::J2000, 0)]
    fn test_time_from_julian_date(#[case] epoch: Epoch, #[case] seconds: i64) {
        let time = Time::from_julian_date(Tai, 0.0, epoch);
        assert_eq!(time.seconds(), Some(seconds));
    }

    #[test]
    fn test_time_from_julian_date_subsecond() {
        let time = Time::from_julian_date(Tai, 0.3 / f64::consts::SECONDS_PER_DAY, Epoch::J2000);
        assert_approx_eq!(time.subsecond().unwrap(), 0.3, atol <= 1e-15);
    }

    #[test]
    fn test_time_from_two_part_julian_date() {
        let t0 = time!(Tai, 2024, 7, 11, 8, 2, 14.0).unwrap();
        let (jd1, jd2) = t0.two_part_julian_date();
        let t1 = Time::from_two_part_julian_date(Tai, jd1, jd2);
        assert_approx_eq!(t0, t1);
    }

    #[rstest]
    #[case(i64::MAX as f64, 1.0)]
    #[case(i64::MIN as f64, -1.0)]
    fn test_time_from_two_part_julian_date_edge_cases(#[case] jd1: f64, #[case] jd2: f64) {
        let time = Time::from_two_part_julian_date(Tai, jd1, jd2);
        // Edge cases now result in non-finite TimeDelta variants (NaN, PosInf, NegInf)
        assert!(!time.to_delta().is_finite());
    }

    #[rstest]
    #[case(
        (SECONDS_BETWEEN_JD_AND_J2000 as f64) / f64::consts::SECONDS_PER_DAY,
        0.0,
        0,
    )]
    #[case(
        (SECONDS_BETWEEN_JD_AND_J2000 as f64 + 0.5) / f64::consts::SECONDS_PER_DAY,
        0.6 / f64::consts::SECONDS_PER_DAY,
        1,
    )]
    #[case(
        (SECONDS_BETWEEN_JD_AND_J2000 as f64 + 0.5) / f64::consts::SECONDS_PER_DAY,
        -0.6 / f64::consts::SECONDS_PER_DAY,
        -1,
    )]
    fn test_time_from_two_part_julian_date_adjustments(
        #[case] jd1: f64,
        #[case] jd2: f64,
        #[case] expected: i64,
    ) {
        let time = Time::from_two_part_julian_date(Tai, jd1, jd2);
        assert_eq!(time.seconds(), Some(expected));
    }

    #[test]
    fn test_time_with_scale_and_delta() {
        let tai: Time<Tai> = Time::default();
        let delta = TimeDelta::from_seconds(20);
        let tdb = tai.with_scale_and_delta(Tdb, delta);
        assert_eq!(tdb.scale(), Tdb);
        assert_eq!(tdb.seconds(), Some(tai.seconds().unwrap() + 20));
    }

    #[rstest]
    #[case(f64::INFINITY)]
    #[case(-f64::INFINITY)]
    #[case(f64::NAN)]
    #[case(-f64::NAN)]
    #[case(i64::MAX as f64 / f64::consts::SECONDS_PER_DAY + 1.0)]
    #[case(i64::MIN as f64 / f64::consts::SECONDS_PER_DAY - 1.0)]
    fn test_time_from_julian_date_special_values(#[case] julian_date: f64) {
        let time = Time::from_julian_date(Tai, julian_date, Epoch::J2000);
        // Special values (NaN, Infinity) result in non-finite TimeDelta
        assert!(!time.to_delta().is_finite());
    }

    #[rstest]
    #[case("2000-01-01T00:00:00", Ok(time!(Tai, 2000, 1, 1).unwrap()))]
    #[case("2000-01-01T00:00:00 TAI", Ok(time!(Tai, 2000, 1, 1).unwrap()))]
    #[case("2000-1-01T00:00:00", Err(TimeError::DateError(DateError::InvalidIsoString("2000-1-01".to_string()))))]
    #[case("2000-01-01T0:00:00", Err(TimeError::TimeError(TimeOfDayError::InvalidIsoString("0:00:00".to_string()))))]
    #[case("2000-01-01-00:00:00", Err(TimeError::InvalidIsoString("2000-01-01-00:00:00".to_string())))]
    #[case("2000-01-01T00:00:00 UTC", Err(TimeError::InvalidIsoString("2000-01-01T00:00:00 UTC".to_string())))]
    fn test_time_from_str_tai(#[case] iso: &str, #[case] expected: Result<Time<Tai>, TimeError>) {
        let actual: Result<Time<Tai>, TimeError> = iso.parse();
        assert_eq!(actual, expected)
    }

    #[rstest]
    #[case("2000-01-01T00:00:00", Ok(time!(Tcb, 2000, 1, 1).unwrap()))]
    #[case("2000-01-01T00:00:00 TCB", Ok(time!(Tcb, 2000, 1, 1).unwrap()))]
    #[case("2000-1-01T00:00:00", Err(TimeError::DateError(DateError::InvalidIsoString("2000-1-01".to_string()))))]
    #[case("2000-01-01T0:00:00", Err(TimeError::TimeError(TimeOfDayError::InvalidIsoString("0:00:00".to_string()))))]
    #[case("2000-01-01-00:00:00", Err(TimeError::InvalidIsoString("2000-01-01-00:00:00".to_string())))]
    #[case("2000-01-01T00:00:00 UTC", Err(TimeError::InvalidIsoString("2000-01-01T00:00:00 UTC".to_string())))]
    fn test_time_from_str_tcb(#[case] iso: &str, #[case] expected: Result<Time<Tcb>, TimeError>) {
        let actual: Result<Time<Tcb>, TimeError> = iso.parse();
        assert_eq!(actual, expected)
    }

    #[rstest]
    #[case("2000-01-01T00:00:00", Ok(time!(Tcg, 2000, 1, 1).unwrap()))]
    #[case("2000-01-01T00:00:00 TCG", Ok(time!(Tcg, 2000, 1, 1).unwrap()))]
    #[case("2000-1-01T00:00:00", Err(TimeError::DateError(DateError::InvalidIsoString("2000-1-01".to_string()))))]
    #[case("2000-01-01T0:00:00", Err(TimeError::TimeError(TimeOfDayError::InvalidIsoString("0:00:00".to_string()))))]
    #[case("2000-01-01-00:00:00", Err(TimeError::InvalidIsoString("2000-01-01-00:00:00".to_string())))]
    #[case("2000-01-01T00:00:00 UTC", Err(TimeError::InvalidIsoString("2000-01-01T00:00:00 UTC".to_string())))]
    fn test_time_from_str_tcg(#[case] iso: &str, #[case] expected: Result<Time<Tcg>, TimeError>) {
        let actual: Result<Time<Tcg>, TimeError> = iso.parse();
        assert_eq!(actual, expected)
    }

    #[rstest]
    #[case("2000-01-01T00:00:00", Ok(time!(Tdb, 2000, 1, 1).unwrap()))]
    #[case("2000-01-01T00:00:00 TDB", Ok(time!(Tdb, 2000, 1, 1).unwrap()))]
    #[case("2000-1-01T00:00:00", Err(TimeError::DateError(DateError::InvalidIsoString("2000-1-01".to_string()))))]
    #[case("2000-01-01T0:00:00", Err(TimeError::TimeError(TimeOfDayError::InvalidIsoString("0:00:00".to_string()))))]
    #[case("2000-01-01-00:00:00", Err(TimeError::InvalidIsoString("2000-01-01-00:00:00".to_string())))]
    #[case("2000-01-01T00:00:00 UTC", Err(TimeError::InvalidIsoString("2000-01-01T00:00:00 UTC".to_string())))]
    fn test_time_from_str_tdb(#[case] iso: &str, #[case] expected: Result<Time<Tdb>, TimeError>) {
        let actual: Result<Time<Tdb>, TimeError> = iso.parse();
        assert_eq!(actual, expected)
    }

    #[rstest]
    #[case("2000-01-01T00:00:00", Ok(time!(Tt, 2000, 1, 1).unwrap()))]
    #[case("2000-01-01T00:00:00 TT", Ok(time!(Tt, 2000, 1, 1).unwrap()))]
    #[case("2000-1-01T00:00:00", Err(TimeError::DateError(DateError::InvalidIsoString("2000-1-01".to_string()))))]
    #[case("2000-01-01T0:00:00", Err(TimeError::TimeError(TimeOfDayError::InvalidIsoString("0:00:00".to_string()))))]
    #[case("2000-01-01-00:00:00", Err(TimeError::InvalidIsoString("2000-01-01-00:00:00".to_string())))]
    #[case("2000-01-01T00:00:00 UTC", Err(TimeError::InvalidIsoString("2000-01-01T00:00:00 UTC".to_string())))]
    fn test_time_from_str_tt(#[case] iso: &str, #[case] expected: Result<Time<Tt>, TimeError>) {
        let actual: Result<Time<Tt>, TimeError> = iso.parse();
        assert_eq!(actual, expected)
    }

    #[rstest]
    #[case("2000-01-01T00:00:00", Ok(time!(Ut1, 2000, 1, 1).unwrap()))]
    #[case("2000-01-01T00:00:00 UT1", Ok(time!(Ut1, 2000, 1, 1).unwrap()))]
    #[case("2000-1-01T00:00:00", Err(TimeError::DateError(DateError::InvalidIsoString("2000-1-01".to_string()))))]
    #[case("2000-01-01T0:00:00", Err(TimeError::TimeError(TimeOfDayError::InvalidIsoString("0:00:00".to_string()))))]
    #[case("2000-01-01-00:00:00", Err(TimeError::InvalidIsoString("2000-01-01-00:00:00".to_string())))]
    #[case("2000-01-01T00:00:00 UTC", Err(TimeError::InvalidIsoString("2000-01-01T00:00:00 UTC".to_string())))]
    fn test_time_from_str_ut1(#[case] iso: &str, #[case] expected: Result<Time<Ut1>, TimeError>) {
        let actual: Result<Time<Ut1>, TimeError> = iso.parse();
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_time_display() {
        let time = Time::j2000(Tai);
        let expected = "2000-01-01T12:00:00.000 TAI".to_string();
        let actual = time.to_string();
        assert_eq!(expected, actual);
        let expected = "2000-01-01T12:00:00.000000000000000 TAI".to_string();
        let actual = format!("{time:.15}");
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_j2000() {
        let actual = Time::j2000(Tai);
        let expected = Time {
            scale: Tai,
            ..Default::default()
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_jd0() {
        let actual = Time::jd0(Tai);
        let expected = Time::new(Tai, -211813488000, Subsecond::default());
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_time_seconds() {
        let time = Time::new(Tai, 1234567890, Subsecond::from_f64(0.9876543210).unwrap());
        let expected = Some(1234567890);
        let actual = time.seconds();
        assert_eq!(
            expected, actual,
            "expected Time to have {expected:?} seconds, but got {actual:?}"
        );
    }

    #[test]
    fn test_julian_date() {
        let time = Time::jd0(Tdb);
        assert_eq!(time.julian_date(Epoch::JulianDate, Unit::Days), 0.0);
        assert_eq!(time.seconds_since_julian_epoch(), 0.0);
        assert_eq!(time.days_since_julian_epoch(), 0.0);
        assert_eq!(time.centuries_since_julian_epoch(), 0.0);
    }

    #[test]
    fn test_modified_julian_date() {
        let time = Time::mjd0(Tdb);
        assert_eq!(time.julian_date(Epoch::ModifiedJulianDate, Unit::Days), 0.0);
        assert_eq!(time.seconds_since_modified_julian_epoch(), 0.0);
        assert_eq!(time.days_since_modified_julian_epoch(), 0.0);
        assert_eq!(time.centuries_since_modified_julian_epoch(), 0.0);
    }

    #[test]
    fn test_j1950() {
        let time = Time::j1950(Tdb);
        assert_eq!(time.julian_date(Epoch::J1950, Unit::Days), 0.0);
        assert_eq!(time.seconds_since_j1950(), 0.0);
        assert_eq!(time.days_since_j1950(), 0.0);
        assert_eq!(time.centuries_since_j1950(), 0.0);
    }

    #[test]
    fn test_j2000() {
        let time = Time::j2000(Tdb);
        assert_eq!(time.julian_date(Epoch::J2000, Unit::Days), 0.0);
        assert_eq!(time.seconds_since_j2000(), 0.0);
        assert_eq!(time.days_since_j2000(), 0.0);
        assert_eq!(time.centuries_since_j2000(), 0.0);
    }

    #[test]
    fn test_j2100() {
        let time = time!(Tdb, 2100, 1, 1, 12).unwrap();
        assert_eq!(
            time.julian_date(Epoch::J2000, Unit::Days),
            DAYS_PER_JULIAN_CENTURY
        );
        assert_eq!(time.seconds_since_j2000(), 3155760000.0);
        assert_eq!(time.days_since_j2000(), DAYS_PER_JULIAN_CENTURY);
        assert_eq!(time.centuries_since_j2000(), 1.0);
    }

    #[test]
    fn test_two_part_julian_date() {
        let time = time!(Tdb, 2100, 1, 2).unwrap();
        let (jd1, jd2) = time.two_part_julian_date();
        assert_eq!(jd1, 2451545.0 + DAYS_PER_JULIAN_CENTURY);
        assert_eq!(jd2, 0.5);
    }

    #[test]
    fn test_time_macro() {
        let time = time!(Tai, 2000, 1, 1).unwrap();
        assert_eq!(time.seconds(), Some(-SECONDS_PER_HALF_DAY));
        let time = time!(Tai, 2000, 1, 1, 12).unwrap();
        assert_eq!(time.seconds(), Some(0));
        let time = time!(Tai, 2000, 1, 1, 12, 0).unwrap();
        assert_eq!(time.seconds(), Some(0));
        let time = time!(Tai, 2000, 1, 1, 12, 0, 0.0).unwrap();
        assert_eq!(time.seconds(), Some(0));
        // TODO: Fix subsecond handling in TimeOfDay::from_hms or time builder
        // let time = time!(Tai, 2000, 1, 1, 12, 0, 0.123).unwrap();
        // assert_eq!(time.seconds(), Some(0));
        // assert_approx_eq!(time.subsecond().unwrap(), 0.123, atol <= 1e-12);
    }

    #[test]
    fn test_time_subsecond() {
        let time = Time::new(Tai, 0, Subsecond::from_f64(0.123).unwrap());
        assert_eq!(time.subsecond(), Some(0.123));
    }

    #[rstest]
    #[case::zero_delta(Time::default(), Time::default(), TimeDelta::default())]
    #[case::positive_delta(Time::default(), Time::new(Tai, 1, Subsecond::default()), TimeDelta::from_seconds(-1))]
    #[case::negative_delta(Time::default(), Time::new(Tai, -1, Subsecond::default()), TimeDelta::from_seconds(1))]
    fn test_time_delta(
        #[case] lhs: Time<Tai>,
        #[case] rhs: Time<Tai>,
        #[case] expected: TimeDelta,
    ) {
        assert_eq!(expected, lhs - rhs);
    }

    const MAX_FEMTOSECONDS: Subsecond = Subsecond::from_attoseconds(999_999_999_999_999);

    #[rstest]
    #[case::zero_value(Time::new(Tai, 0, Subsecond::default()), 12)]
    #[case::one_femtosecond_less_than_an_hour(Time::new(Tai, SECONDS_PER_HOUR - 1, MAX_FEMTOSECONDS), 12)]
    #[case::exactly_one_hour(Time::new(Tai, SECONDS_PER_HOUR, Subsecond::default()), 13)]
    #[case::half_day(Time::new(Tai, SECONDS_PER_DAY / 2, Subsecond::default()), 0)]
    #[case::negative_half_day(Time::new(Tai, -SECONDS_PER_DAY / 2, Subsecond::default()), 0)]
    #[case::one_day_and_one_hour(Time::new(Tai, SECONDS_PER_HOUR * 25, Subsecond::default()), 13)]
    #[case::one_femtosecond_less_than_the_epoch(Time::new(Tai, -1, MAX_FEMTOSECONDS), 11)]
    #[case::one_hour_less_than_the_epoch(Time::new(Tai, -SECONDS_PER_HOUR, Subsecond::default()), 11)]
    #[case::one_hour_and_one_femtosecond_less_than_the_epoch(Time::new(Tai, -SECONDS_PER_HOUR - 1, MAX_FEMTOSECONDS), 10)]
    #[case::one_day_less_than_the_epoch(Time::new(Tai, -SECONDS_PER_DAY, Subsecond::default()), 12)]
    #[case::one_day_and_one_hour_less_than_the_epoch(Time::new(Tai, -SECONDS_PER_DAY - SECONDS_PER_HOUR, Subsecond::default()), 11)]
    #[case::two_days_less_than_the_epoch(Time::new(Tai, -SECONDS_PER_DAY * 2, Subsecond::default()), 12)]
    fn test_time_civil_time_hour(#[case] time: Time<Tai>, #[case] expected: u8) {
        let actual = time.hour();
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_value(Time::new(Tai, 0, Subsecond::default()), 0)]
    #[case::one_femtosecond_less_than_one_minute(Time::new(Tai, SECONDS_PER_MINUTE - 1, MAX_FEMTOSECONDS), 0)]
    #[case::one_minute(Time::new(Tai, SECONDS_PER_MINUTE, Subsecond::default()), 1)]
    #[case::one_femtosecond_less_than_an_hour(Time::new(Tai, SECONDS_PER_HOUR - 1, MAX_FEMTOSECONDS), 59)]
    #[case::exactly_one_hour(Time::new(Tai, SECONDS_PER_HOUR, Subsecond::default()), 0)]
    #[case::one_hour_and_one_minute(Time::new(Tai, SECONDS_PER_HOUR + SECONDS_PER_MINUTE, Subsecond::default()), 1)]
    #[case::one_hour_less_than_the_epoch(Time::new(Tai, -SECONDS_PER_HOUR, Subsecond::default()), 0)]
    #[case::one_femtosecond_less_than_the_epoch(Time::new(Tai, -1, MAX_FEMTOSECONDS), 59)]
    #[case::one_minute_less_than_the_epoch(Time::new(Tai, -SECONDS_PER_MINUTE, Subsecond::default()), 59)]
    #[case::one_minute_and_one_femtosecond_less_than_the_epoch(Time::new(Tai, -SECONDS_PER_MINUTE - 1, MAX_FEMTOSECONDS), 58)]
    fn test_time_civil_time_minute(#[case] time: Time<Tai>, #[case] expected: u8) {
        let actual = time.minute();
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_value(Time::new(Tai, 0, Subsecond::default()), 0)]
    #[case::one_femtosecond_less_than_one_second(Time::new(Tai, 0, MAX_FEMTOSECONDS), 0)]
    #[case::one_second(Time::new(Tai, 1, Subsecond::default()), 1)]
    #[case::one_femtosecond_less_than_a_minute(Time::new(Tai, SECONDS_PER_MINUTE - 1, MAX_FEMTOSECONDS), 59)]
    #[case::exactly_one_minute(Time::new(Tai, SECONDS_PER_MINUTE, Subsecond::default()), 0)]
    #[case::one_minute_and_one_second(Time::new(Tai, SECONDS_PER_MINUTE + 1, Subsecond::default()), 1)]
    #[case::one_femtosecond_less_than_the_epoch(Time::new(Tai, -1, MAX_FEMTOSECONDS), 59)]
    #[case::one_second_less_than_the_epoch(Time::new(Tai, -1, Subsecond::default()), 59)]
    #[case::one_second_and_one_femtosecond_less_than_the_epoch(Time::new(Tai, -2, MAX_FEMTOSECONDS), 58)]
    #[case::one_minute_less_than_the_epoch(Time::new(Tai, -SECONDS_PER_MINUTE, Subsecond::default()), 0)]
    fn test_time_civil_time_second(#[case] time: Time<Tai>, #[case] expected: u8) {
        let actual = time.second();
        assert_eq!(expected, actual);
    }

    const POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE: Time<Tai> = Time::new(
        Tai,
        0,
        Subsecond::new()
            .set_milliseconds(123)
            .set_microseconds(456)
            .set_nanoseconds(789)
            .set_picoseconds(12)
            .set_femtoseconds(345),
    );

    const NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE: Time<Tai> = Time::new(
        Tai,
        -1,
        Subsecond::new()
            .set_milliseconds(123)
            .set_microseconds(456)
            .set_nanoseconds(789)
            .set_picoseconds(12)
            .set_femtoseconds(345),
    );

    #[rstest]
    #[case::positive_time_millisecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::millisecond,
        123
    )]
    #[case::positive_time_microsecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::microsecond,
        456
    )]
    #[case::positive_time_nanosecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::nanosecond,
        789
    )]
    #[case::positive_time_picosecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::picosecond,
        12
    )]
    #[case::positive_time_femtosecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::femtosecond,
        345
    )]
    #[case::negative_time_millisecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::millisecond,
        123
    )]
    #[case::negative_time_microsecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::microsecond,
        456
    )]
    #[case::negative_time_nanosecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::nanosecond,
        789
    )]
    #[case::negative_time_picosecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::picosecond,
        12
    )]
    #[case::negative_time_femtosecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        CivilTime::femtosecond,
        345
    )]
    fn test_time_subseconds(
        #[case] time: Time<Tai>,
        #[case] f: fn(&Time<Tai>) -> u32,
        #[case] expected: u32,
    ) {
        let actual = f(&time);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_delta(Time::default(), TimeDelta::default(), Time::default())]
    #[case::pos_delta_no_carry(Time::new(Tai, 1, Subsecond::new().set_milliseconds(300)), TimeDelta::from_seconds_and_subsecond(1, Subsecond::new().set_milliseconds(600)), Time::new(Tai, 2, Subsecond::new().set_milliseconds(900)))]
    #[case::pos_delta_with_carry(Time::new(Tai, 1, Subsecond::new().set_milliseconds(300)), TimeDelta::from_seconds_and_subsecond(1, Subsecond::new().set_milliseconds(900)), Time::new(Tai, 3, Subsecond::new().set_milliseconds(200)))]
    #[case::neg_delta_no_carry(Time::new(Tai, 1, Subsecond::new().set_milliseconds(600)), TimeDelta::from_seconds_and_subsecond(-2, Subsecond::new().set_milliseconds(700)), Time::new(Tai, 0, Subsecond::new().set_milliseconds(300)))]
    #[case::neg_delta_with_carry(Time::new(Tai, 1, Subsecond::new().set_milliseconds(600)), TimeDelta::from_seconds_and_subsecond(-2, Subsecond::new().set_milliseconds(300)), Time::new(Tai, -1, Subsecond::new().set_milliseconds(900)))]
    fn test_time_add_time_delta(
        #[case] time: Time<Tai>,
        #[case] delta: TimeDelta,
        #[case] expected: Time<Tai>,
    ) {
        let actual = time + delta;
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_delta(Time::default(), TimeDelta::default(), Time::default())]
    #[case::pos_delta_no_carry(Time::new(Tai, 1, Subsecond::new().set_milliseconds(900)), TimeDelta::from_seconds_and_subsecond(1, Subsecond::new().set_milliseconds(300)), Time::new(Tai, 0, Subsecond::new().set_milliseconds(600)))]
    #[case::pos_delta_with_carry(Time::new(Tai, 1, Subsecond::new().set_milliseconds(300)), TimeDelta::from_seconds_and_subsecond(1, Subsecond::new().set_milliseconds(400)), Time::new(Tai, -1, Subsecond::new().set_milliseconds(900)))]
    #[case::neg_delta_no_carry(Time::new(Tai, 1, Subsecond::new().set_milliseconds(600)), TimeDelta::from_seconds_and_subsecond(-1, Subsecond::new().set_milliseconds(700)), Time::new(Tai, 1, Subsecond::new().set_milliseconds(900)))]
    #[case::neg_delta_with_carry(Time::new(Tai, 1, Subsecond::new().set_milliseconds(900)), TimeDelta::from_seconds_and_subsecond(-1, Subsecond::new().set_milliseconds(300)), Time::new(Tai, 2, Subsecond::new().set_milliseconds(600)))]
    fn test_time_sub_time_delta(
        #[case] time: Time<Tai>,
        #[case] delta: TimeDelta,
        #[case] expected: Time<Tai>,
    ) {
        let actual = time - delta;
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(Time::default(), Time::default())]
    #[case(Time::default(), Time::new(Tai, 1, Subsecond::new().set_milliseconds(900)))]
    #[case(
        Time::new(Tai, 0, Subsecond::new().set_milliseconds(900)),
        Time::new(Tai, 1, Subsecond::new().set_milliseconds(600))
    )]
    #[case(Time::new(Tai, 1, Subsecond::new().set_milliseconds(900)), Time::default())]
    #[case(
        Time::new(Tai, 1, Subsecond::new().set_milliseconds(600)),
        Time::new(Tai, 0, Subsecond::new().set_milliseconds(900))
    )]
    #[case(Time::new(Tai, 1, Subsecond::new().set_milliseconds(600)), Time::new(Tai, -1, Subsecond::new().set_milliseconds(900)), )]
    #[case(Time::new(Tai, -1, Subsecond::new().set_milliseconds(900)), Time::new(Tai, 1, Subsecond::new().set_milliseconds(600)), )]
    #[case(Time::new(Tai, 1, Subsecond::new().set_milliseconds(900)), Time::new(Tai, -1, Subsecond::new().set_milliseconds(600)), )]
    #[case(Time::new(Tai, -1, Subsecond::new().set_milliseconds(600)), Time::new(Tai, 1, Subsecond::new().set_milliseconds(900)), )]
    fn test_time_sub_time(#[case] time1: Time<Tai>, #[case] time2: Time<Tai>) {
        let delta = time2 - time1;
        let actual = time1 + delta;
        assert_eq!(actual, time2);
    }

    #[rstest]
    #[case::at_the_epoch(Time::default(), 0.0)]
    #[case::exactly_one_day_after_the_epoch(
        Time::new(Tai, SECONDS_PER_DAY, Subsecond::default()),
        1.0
    )]
    #[case::exactly_one_day_before_the_epoch(
        Time::new(Tai, -SECONDS_PER_DAY, Subsecond::default()),
        -1.0
    )]
    #[case::a_partial_number_of_days_after_the_epoch(
        Time::new(Tai, (SECONDS_PER_DAY / 2) * 3, Subsecond::new().set_milliseconds(500)),
        1.5000057870370371
    )]
    fn test_time_days_since_j2000(#[case] time: Time<Tai>, #[case] expected: f64) {
        let actual = time.days_since_j2000();
        assert_approx_eq!(expected, actual, atol <= 1e-12);
    }

    #[rstest]
    #[case::at_the_epoch(Time::default(), 0.0)]
    #[case::exactly_one_century_after_the_epoch(
        Time::new(Tai, SECONDS_PER_JULIAN_CENTURY, Subsecond::default()),
        1.0
    )]
    #[case::exactly_one_century_before_the_epoch(
        Time::new(Tai, -SECONDS_PER_JULIAN_CENTURY, Subsecond::default()),
        -1.0
    )]
    #[case::a_partial_number_of_centuries_after_the_epoch(
        Time::new(Tai, (SECONDS_PER_JULIAN_CENTURY / 2) * 3, Subsecond::new().set_milliseconds(500)),
        1.5000000001584404
    )]
    fn test_time_centuries_since_j2000(#[case] time: Time<Tai>, #[case] expected: f64) {
        let actual = time.centuries_since_j2000();
        assert_approx_eq!(expected, actual, atol <= 1e-12);
    }

    #[rstest]
    #[case::j2000(Time::default(), Date::new(2000, 1, 1).unwrap())]
    #[case::next_day(Time::new(Tai, SECONDS_PER_DAY, Subsecond::default()), Date::new(2000, 1, 2).unwrap())]
    #[case::leap_year(Time::new(Tai, SECONDS_PER_DAY * 366, Subsecond::default()), Date::new(2001, 1, 1).unwrap())]
    #[case::non_leap_year(Time::new(Tai, SECONDS_PER_DAY * (366 + 365), Subsecond::default()), Date::new(2002, 1, 1).unwrap())]
    #[case::negative_time(Time::new(Tai, -SECONDS_PER_DAY, Subsecond::default()), Date::new(1999, 12, 31).unwrap())]
    fn test_time_calendar_date(#[case] time: Time<Tai>, #[case] expected: Date) {
        assert_eq!(expected, time.date());
        assert_eq!(expected.year(), time.year());
        assert_eq!(expected.month(), time.month());
        assert_eq!(expected.day(), time.day());
    }

    #[test]
    fn test_time_scale() {
        let time: Time<Tai> = Time::default();
        assert_eq!(time.scale(), Tai);
    }

    #[test]
    fn test_time_override_scale() {
        let time: Time<Tai> = Time::default();
        let time = time.with_scale(Tt);
        assert_eq!(time.scale(), Tt);
    }

    #[test]
    fn test_time_leap_second_outside_utc() {
        let actual = time!(Tai, 2000, 1, 1, 23, 59, 60.0);
        let expected = Err(TimeError::LeapSecondOutsideUtc);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_time_to_delta() {
        let time = time!(Tai, 2000, 1, 1, 12, 0, 0.0).unwrap();
        let actual = time.to_delta();
        let expected = TimeDelta::from_seconds(0);
        assert_eq!(actual, expected);
    }
}
