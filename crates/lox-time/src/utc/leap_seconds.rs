// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

/*!
    Module `leap_seconds` exposes the [LeapSecondsProvider] trait for defining sources of leap
    second data. Lox's standard implementation, [BuiltinLeapSeconds], is suitable for most
    applications.

    `leap_seconds` additionally exposes the lower-level [LeapSecondsKernel] for working directly
    with [NAIF Leap Seconds Kernel][LSK] data.

    [LSK]: https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/time.html#The%20Leapseconds%20Kernel%20LSK
*/

use crate::calendar_dates::Date;
use crate::deltas::{TimeDelta, ToDelta};
use crate::time::Time;
use crate::time_of_day::CivilTime;
use crate::time_scales::Tai;
use crate::utc::Utc;
use lox_units::i64::consts::SECONDS_PER_DAY;

pub const LEAP_SECOND_EPOCHS_UTC: [i64; 28] = [
    -883656000, -867931200, -852033600, -820497600, -788961600, -757425600, -725803200, -694267200,
    -662731200, -631195200, -583934400, -552398400, -520862400, -457704000, -378734400, -315576000,
    -284040000, -236779200, -205243200, -173707200, -126273600, -79012800, -31579200, 189345600,
    284040000, 394372800, 488980800, 536500800,
];

pub const LEAP_SECOND_EPOCHS_TAI: [i64; 28] = [
    -883655991, -867931190, -852033589, -820497588, -788961587, -757425586, -725803185, -694267184,
    -662731183, -631195182, -583934381, -552398380, -520862379, -457703978, -378734377, -315575976,
    -284039975, -236779174, -205243173, -173707172, -126273571, -79012770, -31579169, 189345632,
    284040033, 394372834, 488980835, 536500836,
];

pub const LEAP_SECONDS: [i64; 28] = [
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33,
    34, 35, 36, 37,
];

/// Implementers of `LeapSecondsProvider` provide the offset between TAI and UTC in leap seconds at
/// an instant in either time scale.
pub trait LeapSecondsProvider {
    /// The difference in leap seconds between TAI and UTC at the given TAI instant.
    fn delta_tai_utc(&self, tai: Time<Tai>) -> Option<TimeDelta>;

    /// The difference in leap seconds between UTC and TAI at the given UTC instant.
    fn delta_utc_tai(&self, utc: Utc) -> Option<TimeDelta>;

    /// Returns `true` if a leap second occurs on `date`.
    fn is_leap_second_date(&self, date: Date) -> bool;

    /// Returns `true` if a leap second occurs at `tai`.
    fn is_leap_second(&self, tai: Time<Tai>) -> bool;
}

/// `lox-time`'s default [LeapSecondsProvider], suitable for most applications.
///
/// `BuiltinLeapSeconds` relies on a hard-coded table of leap second data. As new leap seconds are
/// announced, `lox-time` will be updated to include the new data, reflected by a minor version
/// change. If this is unsuitable for your use case, we recommend implementing [LeapSecondsProvider]
/// manually.
#[derive(Debug)]
pub struct BuiltinLeapSeconds;

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

fn find_leap_seconds(epochs: &[i64], leap_seconds: &[i64], seconds: i64) -> Option<TimeDelta> {
    if seconds < epochs[0] {
        return None;
    }
    let idx = epochs.partition_point(|&epoch| epoch <= seconds) - 1;
    let seconds = leap_seconds[idx];
    Some(TimeDelta::from_seconds(seconds))
}

pub fn find_leap_seconds_tai(
    epochs: &[i64],
    leap_seconds: &[i64],
    tai: Time<Tai>,
) -> Option<TimeDelta> {
    find_leap_seconds(epochs, leap_seconds, tai.seconds()?)
}

pub fn find_leap_seconds_utc(epochs: &[i64], leap_seconds: &[i64], utc: Utc) -> Option<TimeDelta> {
    find_leap_seconds(epochs, leap_seconds, utc.to_delta().seconds()?).map(|mut ls| {
        if utc.second() == 60 {
            ls -= TimeDelta::from_seconds(1);
        }
        -ls
    })
}

pub fn is_leap_second_date(epochs: &[i64], date: Date) -> bool {
    let epochs: Vec<i64> = epochs
        .iter()
        .map(|&epoch| epoch / SECONDS_PER_DAY)
        .collect();
    let day_number = date.j2000_day_number();
    epochs.binary_search(&day_number).is_ok()
}

pub fn is_leap_second(epochs: &[i64], tai: Time<Tai>) -> bool {
    match tai.seconds() {
        Some(seconds) => epochs.binary_search(&seconds).is_ok(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    use crate::deltas::TimeDelta;
    use crate::time;
    use crate::time::Time;
    use crate::time_scales::Tai;
    use crate::utc;
    use crate::utc::LeapSecondsProvider;
    use crate::utc::Utc;

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
    #[case(Date::new(2000, 12, 31).unwrap(), false)]
    #[case(Date::new(2016, 12, 31).unwrap(), true)]
    fn test_is_leap_second_date(#[case] date: Date, #[case] expected: bool) {
        let actual = BuiltinLeapSeconds.is_leap_second_date(date);
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case(time!(Tai, 2017, 1, 1, 0, 0, 35.0).unwrap(), false)]
    #[case(time!(Tai, 2017, 1, 1, 0, 0, 36.0).unwrap(), true)]
    fn test_is_leap_second(#[case] tai: Time<Tai>, #[case] expected: bool) {
        let actual = BuiltinLeapSeconds.is_leap_second(tai);
        assert_eq!(actual, expected);
    }
}
