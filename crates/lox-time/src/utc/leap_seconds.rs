/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::calendar_dates::Date;
use crate::constants::i64::{SECONDS_PER_DAY, SECONDS_PER_HALF_DAY};
use lox_io::spice::{Kernel, KernelError};
use std::fs::read_to_string;
use std::num::ParseIntError;
use std::path::Path;
use thiserror::Error;

use crate::deltas::TimeDelta;
use crate::time_of_day::CivilTime;
use crate::time_scales::Tai;
use crate::utc::Utc;
use crate::Time;

const LEAP_SECONDS_KEY: &str = "DELTET/DELTA_AT";

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

pub trait LeapSecondsProvider {
    fn epochs_utc(&self) -> &[i64];
    fn epochs_tai(&self) -> &[i64];
    fn leap_seconds(&self) -> &[i64];

    fn find_leap_seconds(&self, epochs: &[i64], seconds: i64) -> Option<TimeDelta> {
        if seconds < epochs[0] {
            return None;
        }
        let idx = epochs.partition_point(|&epoch| epoch <= seconds) - 1;
        let seconds = self.leap_seconds()[idx];
        Some(TimeDelta::from_seconds(seconds))
    }

    fn delta_tai_utc(&self, tai: Time<Tai>) -> Option<TimeDelta> {
        self.find_leap_seconds(self.epochs_tai(), tai.seconds())
    }

    fn delta_utc_tai(&self, utc: Utc) -> Option<TimeDelta> {
        self.find_leap_seconds(self.epochs_utc(), utc.to_delta().seconds)
            .map(|mut ls| {
                if utc.second() == 60 {
                    ls.seconds -= 1;
                }
                -ls
            })
    }
}

pub struct BuiltinLeapSeconds;

impl LeapSecondsProvider for BuiltinLeapSeconds {
    fn epochs_utc(&self) -> &[i64] {
        &LEAP_SECOND_EPOCHS_UTC
    }

    fn epochs_tai(&self) -> &[i64] {
        &LEAP_SECOND_EPOCHS_TAI
    }

    fn leap_seconds(&self) -> &[i64] {
        &LEAP_SECONDS
    }
}

#[derive(Debug, Error)]
pub enum LskError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Kernel(#[from] KernelError),
    #[error("no leap seconds found in kernel under key `{}`", LEAP_SECONDS_KEY)]
    NoLeapSeconds,
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
}

pub struct Lsk {
    epochs_utc: Vec<i64>,
    epochs_tai: Vec<i64>,
    leap_seconds: Vec<i64>,
}

impl Lsk {
    pub fn from_string(kernel: impl AsRef<str>) -> Result<Self, LskError> {
        let kernel = Kernel::from_string(kernel.as_ref())?;
        let data = kernel
            .get_timestamp_array(LEAP_SECONDS_KEY)
            .ok_or(LskError::NoLeapSeconds)?;
        let mut epochs_utc: Vec<i64> = vec![];
        let mut epochs_tai: Vec<i64> = vec![];
        let mut leap_seconds: Vec<i64> = vec![];
        data.chunks(2).for_each(|chunk| {
            let ls = chunk[0].parse::<i64>();
            let date = Date::from_iso(
                &chunk[1]
                    .replace("JAN", "01")
                    .replace("JUL", "07")
                    .replace("-1", "-01"),
            );
            if let (Ok(ls), Ok(date)) = (ls, date) {
                let epoch = date.j2000_day_number() * SECONDS_PER_DAY - SECONDS_PER_HALF_DAY;
                epochs_utc.push(epoch);
                epochs_tai.push(epoch + ls - 1);
                leap_seconds.push(ls);
            }
        });
        Ok(Self {
            epochs_utc,
            epochs_tai,
            leap_seconds,
        })
    }
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, LskError> {
        let path = path.as_ref();
        let kernel = read_to_string(path)?;
        Self::from_string(kernel)
    }
}

impl LeapSecondsProvider for Lsk {
    fn epochs_utc(&self) -> &[i64] {
        &self.epochs_utc
    }

    fn epochs_tai(&self) -> &[i64] {
        &self.epochs_tai
    }

    fn leap_seconds(&self) -> &[i64] {
        &self.leap_seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::sync::OnceLock;

    use crate::time;
    use crate::utc;

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
    fn test_lsk_leap_seconds(
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

    #[test]
    fn test_lsk() {
        let lsk = kernel();
        assert_eq!(lsk.epochs_utc().len(), 28);
        assert_eq!(lsk.epochs_tai().len(), 28);
        assert_eq!(lsk.leap_seconds().len(), 28);
        assert_eq!(lsk.epochs_utc(), &LEAP_SECOND_EPOCHS_UTC);
        assert_eq!(lsk.epochs_tai(), &LEAP_SECOND_EPOCHS_TAI);
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

    fn kernel() -> &'static Lsk {
        static LSK: OnceLock<Lsk> = OnceLock::new();
        LSK.get_or_init(|| Lsk::from_string(KERNEL).expect("file should be parsable"))
    }
}
