// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::{fs::read_to_string, num::ParseIntError, path::Path};

use lox_time::{
    Time,
    calendar_dates::{Date, DateError},
    deltas::TimeDelta,
    time_scales::Tai,
    utc::{
        Utc,
        leap_seconds::{
            LeapSecondsProvider, find_leap_seconds_tai, find_leap_seconds_utc, is_leap_second,
            is_leap_second_date,
        },
    },
};
use lox_units::i64::consts::{SECONDS_PER_DAY, SECONDS_PER_HALF_DAY};
use thiserror::Error;

use crate::spice::{Kernel, KernelError};

const LEAP_SECONDS_KERNEL_KEY: &str = "DELTET/DELTA_AT";

/// Error type related to parsing leap seconds data from a NAIF Leap Seconds Kernel.
#[derive(Debug, Error)]
pub enum LeapSecondsKernelError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Kernel(#[from] KernelError),
    #[error(
        "no leap seconds found in kernel under key `{}`",
        LEAP_SECONDS_KERNEL_KEY
    )]
    NoLeapSeconds,
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error(transparent)]
    DateError(#[from] DateError),
}

/// In-memory representation of a NAIF Leap Seconds Kernel.
///
/// Most users should prefer [BuiltinLeapSeconds] to implementing their own [LeapSecondsProvider]
/// using the kernel.
#[derive(Debug)]
pub struct LeapSecondsKernel {
    epochs_utc: Vec<i64>,
    epochs_tai: Vec<i64>,
    leap_seconds: Vec<i64>,
}

impl LeapSecondsKernel {
    /// Parse a NAIF Leap Seconds Kernel from a string.
    ///
    /// # Errors
    ///
    /// - [LeapSecondsKernelError::Kernel] if the kernel format is unparseable.
    /// - [LeapSecondsKernelError::NoLeapSeconds] if the kernel contains no leap second data.
    /// - [LeapSecondsKernelError::ParseInt] if a leap second entry in the kernel can't be
    ///   represented as an i64.
    /// - [LeapSecondsKernelError::DateError] if a date contained within the kernel is not
    ///   represented as a valid ISO 8601 string.
    pub fn from_string(kernel: impl AsRef<str>) -> Result<Self, LeapSecondsKernelError> {
        let kernel = Kernel::from_string(kernel.as_ref())?;
        let data = kernel
            .get_timestamp_array(LEAP_SECONDS_KERNEL_KEY)
            .ok_or(LeapSecondsKernelError::NoLeapSeconds)?;
        let mut epochs_utc: Vec<i64> = Vec::with_capacity(data.len() / 2);
        let mut epochs_tai: Vec<i64> = Vec::with_capacity(data.len() / 2);
        let mut leap_seconds: Vec<i64> = Vec::with_capacity(data.len() / 2);
        for chunk in data.chunks(2) {
            let ls = chunk[0].parse::<i64>()?;
            let date = Date::from_iso(
                &chunk[1]
                    .replace("JAN", "01")
                    .replace("JUL", "07")
                    .replace("-1", "-01"),
            )?;
            let epoch = date.j2000_day_number() * SECONDS_PER_DAY - SECONDS_PER_HALF_DAY;
            epochs_utc.push(epoch);
            epochs_tai.push(epoch + ls - 1);
            leap_seconds.push(ls);
        }
        Ok(Self {
            epochs_utc,
            epochs_tai,
            leap_seconds,
        })
    }

    /// Parse a NAIF Leap Seconds Kernel located at `path`.
    ///
    /// # Errors
    ///
    /// - [LeapSecondsKernelError::Io] if the file at `path` can't be read.
    /// - [LeapSecondsKernelError::Kernel] if the kernel format is unparseable.
    /// - [LeapSecondsKernelError::NoLeapSeconds] if the kernel contains no leap second data.
    /// - [LeapSecondsKernelError::ParseInt] if a leap second entry in the kernel can't be
    ///   represented as an i64.
    /// - [LeapSecondsKernelError::DateError] if a date contained within the kernel is not
    ///   represented as a valid ISO 8601 string.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, LeapSecondsKernelError> {
        let path = path.as_ref();
        let kernel = read_to_string(path)?;
        Self::from_string(kernel)
    }
}

impl LeapSecondsProvider for LeapSecondsKernel {
    fn delta_tai_utc(&self, tai: Time<Tai>) -> Option<TimeDelta> {
        find_leap_seconds_tai(&self.epochs_tai, &self.leap_seconds, tai)
    }

    fn delta_utc_tai(&self, utc: Utc) -> Option<TimeDelta> {
        find_leap_seconds_utc(&self.epochs_utc, &self.leap_seconds, utc)
    }

    fn is_leap_second_date(&self, date: Date) -> bool {
        is_leap_second_date(&self.epochs_tai, date)
    }

    fn is_leap_second(&self, tai: Time<Tai>) -> bool {
        is_leap_second(&self.epochs_tai, tai)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_time::utc;
    use lox_time::{
        time,
        utc::leap_seconds::{LEAP_SECOND_EPOCHS_TAI, LEAP_SECOND_EPOCHS_UTC, LEAP_SECONDS},
    };
    use rstest::rstest;
    use std::sync::OnceLock;

    #[test]
    fn test_leap_seconds_kernel() {
        let lsk = kernel();
        assert_eq!(lsk.leap_seconds.len(), 28);
        assert_eq!(lsk.epochs_utc.len(), 28);
        assert_eq!(lsk.epochs_tai.len(), 28);
        assert_eq!(lsk.leap_seconds, &LEAP_SECONDS);
        assert_eq!(lsk.epochs_utc, &LEAP_SECOND_EPOCHS_UTC);
        assert_eq!(lsk.epochs_tai, &LEAP_SECOND_EPOCHS_TAI);
    }

    const KERNEL: &str = "KPL/LSK

\\begindata

DELTET/DELTA_AT        = ( 10,   @1972-JAN-1
                           11,   @1972-JUL-1
                           12,   @1973-JAN-1
                           13,   @1974-JAN-1
                           14,   @1975-JAN-1
                           15,   @1976-JAN-1
                           16,   @1977-JAN-1
                           17,   @1978-JAN-1
                           18,   @1979-JAN-1
                           19,   @1980-JAN-1
                           20,   @1981-JUL-1
                           21,   @1982-JUL-1
                           22,   @1983-JUL-1
                           23,   @1985-JUL-1
                           24,   @1988-JAN-1
                           25,   @1990-JAN-1
                           26,   @1991-JAN-1
                           27,   @1992-JUL-1
                           28,   @1993-JUL-1
                           29,   @1994-JUL-1
                           30,   @1996-JAN-1
                           31,   @1997-JUL-1
                           32,   @1999-JAN-1
                           33,   @2006-JAN-1
                           34,   @2009-JAN-1
                           35,   @2012-JUL-1
                           36,   @2015-JUL-1
                           37,   @2017-JAN-1 )

\\begintext";

    #[rstest]
    #[case::before_utc(
        time!(Tai, 1971, 12, 31, 23, 59, 59.0).unwrap(),
        utc!(1971, 12, 31, 23, 59, 59.0).unwrap(),
        None,
    )]
    #[case::j2000(Time::default(), Utc::default(), Some(TimeDelta::from_seconds(32)))]
    #[case::new_year_1972(
        time!(Tai, 1972, 1, 1, 0, 0, 10.0).unwrap(),
        utc!(1972, 1, 1).unwrap(),
        Some(TimeDelta::from_seconds(10)),
    )]
    #[case::new_year_2017(
        time!(Tai, 2017, 1, 1, 0, 0, 37.0).unwrap(),
        utc!(2017, 1, 1, 0, 0, 0.0).unwrap(),
        Some(TimeDelta::from_seconds(37)),
    )]
    #[case::new_year_2024(
        time!(Tai, 2024, 1, 1).unwrap(),
        utc!(2024, 1, 1).unwrap(),
        Some(TimeDelta::from_seconds(37)),
    )]
    fn test_leap_seconds_kernel_leap_seconds(
        #[case] tai: Time<Tai>,
        #[case] utc: Utc,
        #[case] expected: Option<TimeDelta>,
    ) {
        let lsk = kernel();
        let ls_tai = lsk.delta_tai_utc(tai);
        let ls_utc = lsk.delta_utc_tai(utc);
        assert_eq!(ls_tai, expected);
        assert_eq!(ls_utc, expected.map(|ls| -ls));
    }

    #[rstest]
    #[case(Date::new(2000, 12, 31).unwrap(), false)]
    #[case(Date::new(2016, 12, 31).unwrap(), true)]
    fn test_is_leap_second_date(#[case] date: Date, #[case] expected: bool) {
        let actual = kernel().is_leap_second_date(date);
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case(time!(Tai, 2017, 1, 1, 0, 0, 35.0).unwrap(), false)]
    #[case(time!(Tai, 2017, 1, 1, 0, 0, 36.0).unwrap(), true)]
    fn test_is_leap_second(#[case] tai: Time<Tai>, #[case] expected: bool) {
        let actual = kernel().is_leap_second(tai);
        assert_eq!(actual, expected);
    }

    fn kernel() -> &'static LeapSecondsKernel {
        static LSK: OnceLock<LeapSecondsKernel> = OnceLock::new();
        LSK.get_or_init(|| LeapSecondsKernel::from_string(KERNEL).expect("file should be parsable"))
    }
}
