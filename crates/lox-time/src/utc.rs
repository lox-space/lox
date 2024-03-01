use std::fmt::Display;

use num::ToPrimitive;

use crate::constants::u64::{
    FEMTOSECONDS_PER_MICROSECOND, FEMTOSECONDS_PER_MILLISECOND, FEMTOSECONDS_PER_NANOSECOND,
    FEMTOSECONDS_PER_PICOSECOND,
};
use crate::dates::Date;
use crate::errors::LoxTimeError;
use crate::{PerMille, WallClock};

/// A UTC timestamp with additional support for fractional seconds represented with femtosecond
/// precision.
///
/// The `UTC` struct provides the ability to represent leap seconds by setting the `second`
/// component to 60. However, it has no awareness of whether a user-specified leap second is valid.
/// It is intended strictly as an IO time format which must be converted to a continuous time format
/// to be used in calculations.
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub struct UTC {
    hour: u8,
    minute: u8,
    second: u8,
    pub milli: PerMille,
    pub micro: PerMille,
    pub nano: PerMille,
    pub pico: PerMille,
    pub femto: PerMille,
}

impl UTC {
    pub fn new(hour: u8, minute: u8, second: u8) -> Result<Self, LoxTimeError> {
        if !(0..24).contains(&hour) || !(0..60).contains(&minute) || !(0..61).contains(&second) {
            Err(LoxTimeError::InvalidTime(hour, minute, second))
        } else {
            Ok(Self {
                hour,
                minute,
                second,
                ..Default::default()
            })
        }
    }

    pub fn from_fractional_seconds(
        hour: u8,
        minute: u8,
        seconds: f64,
    ) -> Result<Self, LoxTimeError> {
        if !(0.0..61.0).contains(&seconds) {
            return Err(LoxTimeError::InvalidSeconds(seconds));
        }
        let sub = split_seconds(seconds.fract()).unwrap();
        let second = seconds.round().to_u8().unwrap();
        Self::new(hour, minute, second)?;
        Ok(Self {
            hour,
            minute,
            second,
            milli: PerMille(sub[0] as u16),
            micro: PerMille(sub[1] as u16),
            nano: PerMille(sub[2] as u16),
            pico: PerMille(sub[3] as u16),
            femto: PerMille(sub[4] as u16),
        })
    }

    pub fn subsecond_as_femtoseconds(&self) -> u64 {
        let mut femtoseconds = self.femto.0 as u64;
        femtoseconds += self.pico.0 as u64 * FEMTOSECONDS_PER_PICOSECOND;
        femtoseconds += self.nano.0 as u64 * FEMTOSECONDS_PER_NANOSECOND;
        femtoseconds += self.micro.0 as u64 * FEMTOSECONDS_PER_MICROSECOND;
        femtoseconds += self.milli.0 as u64 * FEMTOSECONDS_PER_MILLISECOND;
        femtoseconds
    }
}

impl Display for UTC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}.{}.{}.{}.{}.{} UTC",
            self.hour,
            self.minute,
            self.second,
            self.milli,
            self.micro,
            self.nano,
            self.pico,
            self.femto,
        )?;
        Ok(())
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
        self.milli.into()
    }

    fn microsecond(&self) -> i64 {
        self.micro.into()
    }

    fn nanosecond(&self) -> i64 {
        self.nano.into()
    }

    fn picosecond(&self) -> i64 {
        self.pico.into()
    }

    fn femtosecond(&self) -> i64 {
        self.femto.into()
    }
}

/// Split a floating-point second into SI-prefixed integer parts.
fn split_seconds(seconds: f64) -> Option<[i64; 5]> {
    if !(0.0..1.0).contains(&seconds) {
        return None;
    }
    let mut femto = (seconds * 1e15).to_i64()?;
    let mut parts: [i64; 5] = [0; 5];
    for (i, exponent) in (3..15).step_by(3).rev().enumerate() {
        let factor = i64::pow(10, exponent);
        parts[i] = femto / factor;
        femto -= parts[i] * factor;
    }
    parts[4] = femto;
    Some(parts)
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
    use proptest::{prop_assert, proptest};

    use crate::dates::Calendar::Gregorian;

    use super::*;

    const TIME: UTC = UTC {
        hour: 12,
        minute: 34,
        second: 56,
        milli: PerMille(789),
        micro: PerMille(123),
        nano: PerMille(456),
        pico: PerMille(789),
        femto: PerMille(123),
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
        assert_eq!(TIME.millisecond(), TIME.milli.into());
    }

    #[test]
    fn test_utc_wall_clock_microsecond() {
        assert_eq!(TIME.microsecond(), TIME.micro.into());
    }

    #[test]
    fn test_utc_wall_clock_nanosecond() {
        assert_eq!(TIME.nanosecond(), TIME.nano.into());
    }

    #[test]
    fn test_utc_wall_clock_picosecond() {
        assert_eq!(TIME.picosecond(), TIME.pico.into());
    }

    #[test]
    fn test_utc_wall_clock_femtosecond() {
        assert_eq!(TIME.femtosecond(), TIME.femto.into());
    }

    proptest! {
        #[test]
        fn prop_test_split_seconds(s in 0.0..1.0) {
            prop_assert!(split_seconds(s).is_some())
        }
    }

    #[test]
    fn test_split_seconds() {
        let s1 = split_seconds(0.123).expect("seconds should be valid");
        assert_eq!(123, s1[0]);
        assert_eq!(0, s1[1]);
        assert_eq!(0, s1[2]);
        assert_eq!(0, s1[3]);
        assert_eq!(0, s1[4]);
        let s2 = split_seconds(0.123_456).expect("seconds should be valid");
        assert_eq!(123, s2[0]);
        assert_eq!(456, s2[1]);
        assert_eq!(0, s2[2]);
        assert_eq!(0, s2[3]);
        assert_eq!(0, s2[4]);
        let s3 = split_seconds(0.123_456_789).expect("seconds should be valid");
        assert_eq!(123, s3[0]);
        assert_eq!(456, s3[1]);
        assert_eq!(789, s3[2]);
        assert_eq!(0, s3[3]);
        assert_eq!(0, s3[4]);
        let s4 = split_seconds(0.123_456_789_123).expect("seconds should be valid");
        assert_eq!(123, s4[0]);
        assert_eq!(456, s4[1]);
        assert_eq!(789, s4[2]);
        assert_eq!(123, s4[3]);
        assert_eq!(0, s4[4]);
        let s5 = split_seconds(0.123_456_789_123_456).expect("seconds should be valid");
        assert_eq!(123, s5[0]);
        assert_eq!(456, s5[1]);
        assert_eq!(789, s5[2]);
        assert_eq!(123, s5[3]);
        assert_eq!(456, s5[4]);
        let s6 = split_seconds(0.123_456_789_123_456_78).expect("seconds should be valid");
        assert_eq!(123, s6[0]);
        assert_eq!(456, s6[1]);
        assert_eq!(789, s6[2]);
        assert_eq!(123, s6[3]);
        assert_eq!(456, s6[4]);
        let s7 = split_seconds(0.000_000_000_000_001).expect("seconds should be valid");
        assert_eq!(0, s7[0]);
        assert_eq!(0, s7[1]);
        assert_eq!(0, s7[2]);
        assert_eq!(0, s7[3]);
        assert_eq!(1, s7[4]);
    }

    #[test]
    fn test_illegal_split_seconds() {
        assert!(split_seconds(2.0).is_none());
        assert!(split_seconds(-0.2).is_none());
    }

    #[test]
    fn test_utc_datetime_new() {
        let date = Date::new_unchecked(Gregorian, 2021, 1, 1);
        let time = UTC::new(12, 34, 56).expect("time should be valid");
        let expected = UTCDateTime { date, time };
        let actual = UTCDateTime::new(date, time);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_fractional_seconds() {
        let hour = 0;
        let minute = 0;
        let second = 0.123_456_789_123_456_78;
        let expected = UTC {
            hour: 0,
            minute: 0,
            second: 0,
            milli: PerMille(123),
            micro: PerMille(456),
            nano: PerMille(789),
            pico: PerMille(123),
            femto: PerMille(456),
        };
        let actual =
            UTC::from_fractional_seconds(hour, minute, second).expect("time should be valid");
        assert_eq!(expected, actual);
    }
}
