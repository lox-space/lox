use crate::errors::LoxError;
use crate::time::constants::u64::{
    ATTOSECONDS_PER_FEMTOSECOND, ATTOSECONDS_PER_MICROSECOND, ATTOSECONDS_PER_MILLISECOND,
    ATTOSECONDS_PER_NANOSECOND, ATTOSECONDS_PER_PICOSECOND,
};
use crate::time::dates::Date;
use crate::time::scales::TimeScale;
use crate::time::{Thousandths, WallClock};
use num::ToPrimitive;
use std::fmt::Display;

/// A UTC timestamp with additional support for fractional seconds represented with attosecond
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
    pub milli: Thousandths,
    pub micro: Thousandths,
    pub nano: Thousandths,
    pub pico: Thousandths,
    pub femto: Thousandths,
    pub atto: Thousandths,
}

impl UTC {
    pub fn new(hour: u8, minute: u8, second: u8) -> Result<Self, LoxError> {
        if !(0..24).contains(&hour) || !(0..60).contains(&minute) || !(0..61).contains(&second) {
            Err(LoxError::InvalidTime(hour, minute, second))
        } else {
            Ok(Self {
                hour,
                minute,
                second,
                ..Default::default()
            })
        }
    }

    pub fn from_fractional_seconds(hour: u8, minute: u8, seconds: f64) -> Result<Self, LoxError> {
        if !(0.0..61.0).contains(&seconds) {
            return Err(LoxError::InvalidSeconds(seconds));
        }
        let sub = split_seconds(seconds.fract()).unwrap();
        let second = seconds.round().to_u8().unwrap();
        Self::new(hour, minute, second)?;
        Ok(Self {
            hour,
            minute,
            second,
            milli: Thousandths(sub[0] as u16),
            micro: Thousandths(sub[1] as u16),
            nano: Thousandths(sub[2] as u16),
            pico: Thousandths(sub[3] as u16),
            femto: Thousandths(sub[4] as u16),
            atto: Thousandths(sub[5] as u16),
        })
    }

    /// Returns the `hour` component of the timestamp in a representation convenient for calculations.
    pub fn hour(&self) -> i64 {
        self.hour as i64
    }

    /// Returns the `minute` component of the timestamp in a representation convenient for calculations.
    pub fn minute(&self) -> i64 {
        self.minute as i64
    }

    /// Returns the `second` component of the timestamp in a representation convenient for calculations.
    pub fn second(&self) -> i64 {
        self.second as i64
    }

    pub fn subsecond_as_attoseconds(&self) -> u64 {
        let mut attoseconds = self.atto.0 as u64;
        attoseconds += self.femto.0 as u64 * ATTOSECONDS_PER_FEMTOSECOND;
        attoseconds += self.pico.0 as u64 * ATTOSECONDS_PER_PICOSECOND;
        attoseconds += self.nano.0 as u64 * ATTOSECONDS_PER_NANOSECOND;
        attoseconds += self.micro.0 as u64 * ATTOSECONDS_PER_MICROSECOND;
        attoseconds += self.milli.0 as u64 * ATTOSECONDS_PER_MILLISECOND;
        attoseconds
    }
}

impl Display for UTC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}.{}.{}.{}.{}.{}.{}",
            self.hour,
            self.minute,
            self.second,
            self.milli,
            self.micro,
            self.nano,
            self.pico,
            self.femto,
            self.atto
        )?;
        Ok(())
    }
}

impl WallClock for UTC {
    fn scale(&self) -> TimeScale {
        TimeScale::UTC
    }

    fn hour(&self) -> u8 {
        self.hour
    }

    fn minute(&self) -> u8 {
        self.minute
    }

    fn second(&self) -> u8 {
        self.second
    }

    fn millisecond(&self) -> Thousandths {
        self.milli
    }

    fn microsecond(&self) -> Thousandths {
        self.micro
    }

    fn nanosecond(&self) -> Thousandths {
        self.nano
    }

    fn picosecond(&self) -> Thousandths {
        self.pico
    }

    fn femtosecond(&self) -> Thousandths {
        self.femto
    }

    fn attosecond(&self) -> Thousandths {
        self.atto
    }
}

/// Split a floating-point second into SI-prefixed integer parts.
fn split_seconds(seconds: f64) -> Option<[i64; 6]> {
    if !(0.0..1.0).contains(&seconds) {
        return None;
    }
    let mut atto = (seconds * 1e18).to_i64()?;
    let mut parts: [i64; 6] = [0; 6];
    for (i, exponent) in (3..18).step_by(3).rev().enumerate() {
        let factor = i64::pow(10, exponent);
        parts[i] = atto / factor;
        atto -= parts[i] * factor;
    }
    parts[5] = atto / 10 * 10;
    Some(parts)
}

#[derive(Debug, Copy, Clone)]
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
    use crate::time::utc::split_seconds;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_test_split_seconds(s in 0.0..1.0) {
            prop_assert!(split_seconds(s).is_some())
        }
    }

    #[test]
    fn test_split_second() {
        let s1 = split_seconds(0.123).expect("seconds should be valid");
        assert_eq!(123, s1[0]);
        assert_eq!(0, s1[1]);
        assert_eq!(0, s1[2]);
        assert_eq!(0, s1[3]);
        assert_eq!(0, s1[4]);
        assert_eq!(0, s1[5]);
        let s2 = split_seconds(0.123_456).expect("seconds should be valid");
        assert_eq!(123, s2[0]);
        assert_eq!(456, s2[1]);
        assert_eq!(0, s2[2]);
        assert_eq!(0, s2[3]);
        assert_eq!(0, s2[4]);
        assert_eq!(0, s2[5]);
        let s3 = split_seconds(0.123_456_789).expect("seconds should be valid");
        assert_eq!(123, s3[0]);
        assert_eq!(456, s3[1]);
        assert_eq!(789, s3[2]);
        assert_eq!(0, s3[3]);
        assert_eq!(0, s3[4]);
        assert_eq!(0, s3[5]);
        let s4 = split_seconds(0.123_456_789_123).expect("seconds should be valid");
        assert_eq!(123, s4[0]);
        assert_eq!(456, s4[1]);
        assert_eq!(789, s4[2]);
        assert_eq!(123, s4[3]);
        assert_eq!(0, s4[4]);
        assert_eq!(0, s4[5]);
        let s5 = split_seconds(0.123_456_789_123_456).expect("seconds should be valid");
        assert_eq!(123, s5[0]);
        assert_eq!(456, s5[1]);
        assert_eq!(789, s5[2]);
        assert_eq!(123, s5[3]);
        assert_eq!(456, s5[4]);
        assert_eq!(0, s5[5]);
        let s6 = split_seconds(0.123_456_789_123_456_78).expect("seconds should be valid");
        assert_eq!(123, s6[0]);
        assert_eq!(456, s6[1]);
        assert_eq!(789, s6[2]);
        assert_eq!(123, s6[3]);
        assert_eq!(456, s6[4]);
        assert_eq!(780, s6[5]);
        let s7 = split_seconds(0.000_000_000_000_000_01).expect("seconds should be valid");
        assert_eq!(0, s7[0]);
        assert_eq!(0, s7[1]);
        assert_eq!(0, s7[2]);
        assert_eq!(0, s7[3]);
        assert_eq!(0, s7[4]);
        assert_eq!(10, s7[5]);
    }

    #[test]
    fn test_illegal_split_second() {
        assert!(split_seconds(2.0).is_none());
        assert!(split_seconds(-0.2).is_none());
    }
}
