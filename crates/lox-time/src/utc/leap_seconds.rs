/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

/*!
    Module `leap_seconds` exposes the [LeapSecondsProvider] trait for defining sources of leap
    second data. Lox's standard implementation, [BuiltinLeapSeconds], is suitable for most
    applications.

    `leap_seconds` additionally exposes the lower-level [LeapSecondsKernel] for working directly
    with [NAIF Leap Seconds Kernel][LSK] data.

    [LSK]: https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/time.html#The%20Leapseconds%20Kernel%20LSK
*/

use crate::calendar_dates::{Date, DateError};
use crate::constants::i64::{SECONDS_PER_DAY, SECONDS_PER_HALF_DAY};
use crate::deltas::{TimeDelta, ToDelta};
use crate::prelude::{CivilTime, Tai};
use crate::Time;
use lox_io::spice::{Kernel, KernelError};
use std::convert::Infallible;
use std::fs::read_to_string;
use std::num::ParseIntError;
use std::path::Path;
use thiserror::Error;

use crate::transformations::{LeapSecondsProvider, OffsetProvider};
use crate::utc::Utc;

const LEAP_SECONDS_KERNEL_KEY: &str = "DELTET/DELTA_AT";

const LEAP_SECOND_EPOCHS_UTC: [i64; 28] = [
    -883656000, -867931200, -852033600, -820497600, -788961600, -757425600, -725803200, -694267200,
    -662731200, -631195200, -583934400, -552398400, -520862400, -457704000, -378734400, -315576000,
    -284040000, -236779200, -205243200, -173707200, -126273600, -79012800, -31579200, 189345600,
    284040000, 394372800, 488980800, 536500800,
];

const LEAP_SECOND_EPOCHS_TAI: [i64; 28] = [
    -883655991, -867931190, -852033589, -820497588, -788961587, -757425586, -725803185, -694267184,
    -662731183, -631195182, -583934381, -552398380, -520862379, -457703978, -378734377, -315575976,
    -284039975, -236779174, -205243173, -173707172, -126273571, -79012770, -31579169, 189345632,
    284040033, 394372834, 488980835, 536500836,
];

const LEAP_SECONDS: [i64; 28] = [
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33,
    34, 35, 36, 37,
];

/// `lox-time`'s default [LeapSecondsProvider], suitable for most applications.
///
/// `BuiltinLeapSeconds` relies on a hard-coded table of leap second data. As new leap seconds are
/// announced, `lox-time` will be updated to include the new data, reflected by a minor version
/// change. If this is unsuitable for your use case, we recommend implementing [LeapSecondsProvider]
/// manually.
#[derive(Debug)]
pub struct BuiltinLeapSeconds;

impl OffsetProvider for BuiltinLeapSeconds {
    type Error = Infallible;
}

impl LeapSecondsProvider for BuiltinLeapSeconds {
    fn delta_tai_utc(&self, tai: Time<Tai>) -> Option<TimeDelta> {
        find_leap_seconds_tai(&LEAP_SECOND_EPOCHS_TAI, &LEAP_SECONDS, tai)
    }

    fn delta_utc_tai(&self, utc: Utc) -> Option<TimeDelta> {
        find_leap_seconds_utc(&LEAP_SECOND_EPOCHS_UTC, &LEAP_SECONDS, utc)
    }

    fn is_leap_second_date(&self, date: Date) -> bool {
        is_leap_second_date(&LEAP_SECOND_EPOCHS_UTC, date)
    }

    fn is_leap_second(&self, tai: Time<Tai>) -> bool {
        is_leap_second(&LEAP_SECOND_EPOCHS_TAI, tai)
    }
}

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

impl OffsetProvider for LeapSecondsKernel {
    type Error = Infallible;
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

fn find_leap_seconds(epochs: &[i64], leap_seconds: &[i64], seconds: i64) -> Option<TimeDelta> {
    if seconds < epochs[0] {
        return None;
    }
    let idx = epochs.partition_point(|&epoch| epoch <= seconds) - 1;
    let seconds = leap_seconds[idx];
    Some(TimeDelta::from_seconds(seconds))
}

fn find_leap_seconds_tai(
    epochs: &[i64],
    leap_seconds: &[i64],
    tai: Time<Tai>,
) -> Option<TimeDelta> {
    find_leap_seconds(epochs, leap_seconds, tai.seconds())
}

fn find_leap_seconds_utc(epochs: &[i64], leap_seconds: &[i64], utc: Utc) -> Option<TimeDelta> {
    find_leap_seconds(epochs, leap_seconds, utc.to_delta().seconds).map(|mut ls| {
        if utc.second() == 60 {
            ls.seconds -= 1;
        }
        -ls
    })
}

fn is_leap_second_date(epochs: &[i64], date: Date) -> bool {
    let epochs: Vec<i64> = epochs
        .iter()
        .map(|&epoch| epoch / SECONDS_PER_DAY)
        .collect();
    let day_number = date.j2000_day_number();
    epochs.binary_search(&day_number).is_ok()
}

fn is_leap_second(epochs: &[i64], tai: Time<Tai>) -> bool {
    epochs.binary_search(&tai.seconds).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::sync::OnceLock;

    use crate::deltas::TimeDelta;
    use crate::time;
    use crate::time_scales::Tai;
    use crate::transformations::LeapSecondsProvider;
    use crate::utc;
    use crate::utc::Utc;
    use crate::Time;

    #[rstest]
    #[case::j2000(Time::default(), Utc::default(), 32)]
    #[case::new_year_1972(time!(Tai, 1972, 1, 1, 0, 0, 10.0).unwrap(), utc!(1972, 1, 1).unwrap(), 10)]
    #[case::new_year_2017(time!(Tai, 2017, 1, 1, 0, 0, 37.0).unwrap(), utc!(2017, 1, 1, 0, 0, 0.0).unwrap(), 37)]
    #[case::new_year_2024(time!(Tai, 2024, 1, 1).unwrap(), utc!(2024, 1, 1).unwrap(), 37)]
    fn test_builtin_leap_seconds(#[case] tai: Time<Tai>, #[case] utc: Utc, #[case] expected: i64) {
        let ls_tai = BuiltinLeapSeconds.delta_tai_utc(tai).unwrap();
        let ls_utc = BuiltinLeapSeconds.delta_utc_tai(utc).unwrap();
        assert_eq!(ls_tai, TimeDelta::from_seconds(expected));
        assert_eq!(ls_utc, TimeDelta::from_seconds(-expected));
    }

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
    #[case(&BuiltinLeapSeconds, Date::new(2000, 12, 31).unwrap(), false)]
    #[case(&BuiltinLeapSeconds, Date::new(2016, 12, 31).unwrap(), true)]
    #[case(kernel(), Date::new(2000, 12, 31).unwrap(), false)]
    #[case(kernel(), Date::new(2016, 12, 31).unwrap(), true)]
    fn test_is_leap_second_date(
        #[case] provider: &impl LeapSecondsProvider,
        #[case] date: Date,
        #[case] expected: bool,
    ) {
        let actual = provider.is_leap_second_date(date);
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case(&BuiltinLeapSeconds, time!(Tai, 2017, 1, 1, 0, 0, 35.0).unwrap(), false)]
    #[case(&BuiltinLeapSeconds, time!(Tai, 2017, 1, 1, 0, 0, 36.0).unwrap(), true)]
    #[case(kernel(), time!(Tai, 2017, 1, 1, 0, 0, 35.0).unwrap(), false)]
    #[case(kernel(), time!(Tai, 2017, 1, 1, 0, 0, 36.0).unwrap(), true)]
    fn test_is_leap_second(
        #[case] provider: &impl LeapSecondsProvider,
        #[case] tai: Time<Tai>,
        #[case] expected: bool,
    ) {
        let actual = provider.is_leap_second(tai);
        assert_eq!(actual, expected);
    }

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

    fn kernel() -> &'static LeapSecondsKernel {
        static LSK: OnceLock<LeapSecondsKernel> = OnceLock::new();
        LSK.get_or_init(|| LeapSecondsKernel::from_string(KERNEL).expect("file should be parsable"))
    }
}
