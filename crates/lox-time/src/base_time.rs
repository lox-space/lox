/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module base_time provides a scale-agnostic continuous time representation with femtosecond
//! precision.

use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};

use num::{abs, ToPrimitive};

use crate::constants;
use crate::constants::i64::{
    SECONDS_PER_DAY, SECONDS_PER_HALF_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE,
};
use crate::constants::julian_dates::{
    SECONDS_BETWEEN_J1950_AND_J2000, SECONDS_BETWEEN_JD_AND_J2000, SECONDS_BETWEEN_MJD_AND_J2000,
};
use crate::deltas::TimeDelta;
use crate::julian_dates::{Epoch, JulianDate, Unit};
use crate::subsecond::Subsecond;
use crate::wall_clock::WallClock;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
/// `BaseTime` is the base time representation for time scales without leap seconds. It is measured
/// relative to J2000. `BaseTime::default()` represents the epoch itself.
///
/// `BaseTime` guarantees femtosecond precision, and supports times within 292 billion years either
/// side of the epoch.
pub struct BaseTime {
    // The sign of the time is determined exclusively by the sign of the `seconds` field. `subsecond`
    // is always the positive fraction of a second since the last whole second. For example, one
    // femtosecond before the epoch is represented as
    // ```
    // let time = BaseTime {
    //     seconds: -1,
    //     subsecond: Subsecond(0.999_999_999_999_999),
    // };
    // ```
    pub seconds: i64,
    pub subsecond: Subsecond,
}

impl BaseTime {
    pub const fn new(seconds: i64, subsecond: Subsecond) -> Self {
        Self { seconds, subsecond }
    }

    pub fn from_epoch(epoch: Epoch) -> Self {
        match epoch {
            Epoch::JulianDate => BaseTime {
                seconds: -SECONDS_BETWEEN_JD_AND_J2000,
                subsecond: Subsecond::default(),
            },
            Epoch::ModifiedJulianDate => BaseTime {
                seconds: -SECONDS_BETWEEN_MJD_AND_J2000,
                subsecond: Subsecond::default(),
            },
            Epoch::J1950 => BaseTime {
                seconds: -SECONDS_BETWEEN_J1950_AND_J2000,
                subsecond: Subsecond::default(),
            },
            Epoch::J2000 => BaseTime {
                seconds: 0,
                subsecond: Subsecond::default(),
            },
        }
    }

    fn is_negative(&self) -> bool {
        self.seconds < 0
    }

    pub fn seconds(&self) -> i64 {
        self.seconds
    }

    pub fn subsecond(&self) -> f64 {
        self.subsecond.0
    }

    pub fn seconds_from_epoch(&self, epoch: Epoch) -> i64 {
        match epoch {
            Epoch::JulianDate => self.seconds + SECONDS_BETWEEN_JD_AND_J2000,
            Epoch::ModifiedJulianDate => self.seconds + SECONDS_BETWEEN_MJD_AND_J2000,
            Epoch::J1950 => self.seconds + SECONDS_BETWEEN_J1950_AND_J2000,
            Epoch::J2000 => self.seconds,
        }
    }

    /// Convert self to a single f64, potentially with loss of precision.
    pub fn to_f64(self) -> f64 {
        self.subsecond.0 + self.seconds as f64
    }

    /// Returns the `TimeDelta` between `self` and `other`.
    pub fn delta(&self, other: &Self) -> TimeDelta {
        let mut seconds = self.seconds - other.seconds;
        let subsecond = if self.subsecond < other.subsecond {
            seconds -= 1;
            self.subsecond.0 - other.subsecond.0 + 1.0
        } else {
            self.subsecond.0 - other.subsecond.0
        };

        TimeDelta {
            seconds,
            subsecond: Subsecond(subsecond),
        }
    }
}

impl Display for BaseTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}.{}",
            self.hour(),
            self.minute(),
            self.second(),
            self.subsecond
        )
    }
}

impl Add<TimeDelta> for BaseTime {
    type Output = Self;

    /// The implementation of [Add] for [BaseTime] follows the default Rust rules for integer overflow, which
    /// should be sufficient for all practical purposes.
    fn add(self, rhs: TimeDelta) -> Self::Output {
        if rhs.is_negative() {
            return self - (-rhs);
        }

        let subsec_and_carry = self.subsecond.0 + rhs.subsecond.0;
        let seconds = subsec_and_carry.trunc().to_i64().unwrap() + self.seconds + rhs.seconds;
        Self {
            seconds,
            subsecond: Subsecond(subsec_and_carry.fract()),
        }
    }
}

impl Sub<TimeDelta> for BaseTime {
    type Output = Self;

    /// The implementation of [Sub] for [BaseTime] follows the default Rust rules for integer overflow, which
    /// should be sufficient for all practical purposes.
    fn sub(self, rhs: TimeDelta) -> Self::Output {
        if rhs.is_negative() {
            return self + (-rhs);
        }

        let mut subsec = self.subsecond.0 - rhs.subsecond.0;
        let mut seconds = self.seconds - rhs.seconds;
        if subsec.is_sign_negative() {
            seconds -= 1;
            subsec += 1.0;
        }
        Self {
            seconds,
            subsecond: Subsecond(subsec),
        }
    }
}

impl WallClock for BaseTime {
    fn hour(&self) -> i64 {
        // Since J2000 is taken from midday, we offset by half a day to get the wall clock hour.
        let day_seconds: i64 = if self.is_negative() {
            SECONDS_PER_DAY - (abs(self.seconds) + SECONDS_PER_HALF_DAY) % SECONDS_PER_DAY
        } else {
            (self.seconds + SECONDS_PER_HALF_DAY) % SECONDS_PER_DAY
        };
        day_seconds / SECONDS_PER_HOUR
    }

    fn minute(&self) -> i64 {
        let hour_seconds: i64 = if self.is_negative() {
            SECONDS_PER_HOUR - abs(self.seconds) % SECONDS_PER_HOUR
        } else {
            self.seconds % SECONDS_PER_HOUR
        };
        hour_seconds / SECONDS_PER_MINUTE
    }

    fn second(&self) -> i64 {
        if self.is_negative() {
            SECONDS_PER_MINUTE - abs(self.seconds) % SECONDS_PER_MINUTE
        } else {
            self.seconds % SECONDS_PER_MINUTE
        }
    }

    fn millisecond(&self) -> i64 {
        self.subsecond.millisecond()
    }

    fn microsecond(&self) -> i64 {
        self.subsecond.microsecond()
    }

    fn nanosecond(&self) -> i64 {
        self.subsecond.nanosecond()
    }

    fn picosecond(&self) -> i64 {
        self.subsecond.picosecond()
    }

    fn femtosecond(&self) -> i64 {
        self.subsecond.femtosecond()
    }
}

impl JulianDate for BaseTime {
    fn julian_date(&self, epoch: Epoch, unit: Unit) -> f64 {
        let mut decimal_seconds = self.seconds_from_epoch(epoch).to_f64().unwrap();
        decimal_seconds += self.subsecond.0;
        match unit {
            Unit::Seconds => decimal_seconds,
            Unit::Days => decimal_seconds / constants::f64::SECONDS_PER_DAY,
            Unit::Centuries => decimal_seconds / constants::f64::SECONDS_PER_JULIAN_CENTURY,
        }
    }

    fn two_part_julian_date(&self) -> (f64, f64) {
        let days = self.julian_date(Epoch::JulianDate, Unit::Days);
        (days.trunc(), days.fract())
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use rstest::rstest;

    use crate::constants::i64::SECONDS_PER_JULIAN_CENTURY;

    use super::*;

    #[test]
    fn test_base_time_new() {
        let seconds = 123;
        let subsecond = Subsecond(0.123_456_789_012_345);
        let expected = BaseTime { seconds, subsecond };
        let actual = BaseTime::new(seconds, subsecond);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_base_time_is_negative() {
        assert!(BaseTime {
            seconds: -1,
            subsecond: Subsecond::default(),
        }
        .is_negative());
        assert!(!BaseTime {
            seconds: 0,
            subsecond: Subsecond::default(),
        }
        .is_negative());
        assert!(!BaseTime {
            seconds: 1,
            subsecond: Subsecond::default(),
        }
        .is_negative());
    }

    #[test]
    fn test_base_time_seconds() {
        let time = BaseTime {
            seconds: 123,
            subsecond: Subsecond::default(),
        };
        assert_eq!(time.seconds(), 123);
    }

    #[test]
    fn test_base_time_subsecond() {
        let time = BaseTime {
            seconds: 0,
            subsecond: Subsecond(0.123),
        };
        assert_eq!(time.subsecond(), 0.123);
    }

    #[rstest]
    #[case::zero_delta(BaseTime::default(), BaseTime::default(), TimeDelta::default())]
    #[case::positive_delta(BaseTime::default(), BaseTime { seconds: 1, subsecond: Subsecond::default() }, TimeDelta { seconds: -1, subsecond: Subsecond::default() })]
    #[case::negative_delta(BaseTime::default(), BaseTime { seconds: -1, subsecond: Subsecond::default() }, TimeDelta { seconds: 1, subsecond: Subsecond::default() })]
    fn test_base_time_delta(
        #[case] lhs: BaseTime,
        #[case] rhs: BaseTime,
        #[case] expected: TimeDelta,
    ) {
        assert_eq!(expected, lhs.delta(&rhs));
    }

    const MAX_FEMTOSECONDS: Subsecond = Subsecond(0.999_999_999_999_999);

    #[rstest]
    #[case::zero_value(BaseTime { seconds: 0, subsecond: Subsecond::default() }, 12)]
    #[case::one_femtosecond_less_than_an_hour(BaseTime { seconds: SECONDS_PER_HOUR - 1, subsecond: MAX_FEMTOSECONDS, }, 12)]
    #[case::exactly_one_hour(BaseTime { seconds: SECONDS_PER_HOUR, subsecond: Subsecond::default() }, 13)]
    #[case::one_day_and_one_hour(BaseTime { seconds: SECONDS_PER_HOUR * 25, subsecond: Subsecond::default() }, 13)]
    #[case::one_femtosecond_less_than_the_epoch(BaseTime { seconds: - 1, subsecond: MAX_FEMTOSECONDS, }, 11)]
    #[case::one_hour_less_than_the_epoch(BaseTime { seconds: - SECONDS_PER_HOUR, subsecond: Subsecond::default() }, 11)]
    #[case::one_hour_and_one_femtosecond_less_than_the_epoch(BaseTime { seconds: - SECONDS_PER_HOUR - 1, subsecond: MAX_FEMTOSECONDS, }, 10)]
    #[case::one_day_less_than_the_epoch(BaseTime { seconds: - SECONDS_PER_DAY, subsecond: Subsecond::default() }, 12)]
    #[case::two_days_less_than_the_epoch(BaseTime { seconds: - SECONDS_PER_DAY * 2, subsecond: Subsecond::default() }, 12)]
    fn test_base_time_wall_clock_hour(#[case] time: BaseTime, #[case] expected: i64) {
        let actual = time.hour();
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_value(BaseTime { seconds: 0, subsecond: Subsecond::default() }, 0)]
    #[case::one_femtosecond_less_than_one_minute(BaseTime { seconds: SECONDS_PER_MINUTE - 1, subsecond: MAX_FEMTOSECONDS, }, 0)]
    #[case::one_minute(BaseTime { seconds: SECONDS_PER_MINUTE, subsecond: Subsecond::default() }, 1)]
    #[case::one_femtosecond_less_than_an_hour(BaseTime { seconds: SECONDS_PER_HOUR - 1, subsecond: MAX_FEMTOSECONDS, }, 59)]
    #[case::exactly_one_hour(BaseTime { seconds: SECONDS_PER_HOUR, subsecond: Subsecond::default() }, 0)]
    #[case::one_hour_and_one_minute(BaseTime { seconds: SECONDS_PER_HOUR + SECONDS_PER_MINUTE, subsecond: Subsecond::default() }, 1)]
    #[case::one_femtosecond_less_than_the_epoch(BaseTime { seconds: - 1, subsecond: MAX_FEMTOSECONDS, }, 59)]
    #[case::one_minute_less_than_the_epoch(BaseTime { seconds: - SECONDS_PER_MINUTE, subsecond: Subsecond::default() }, 59)]
    #[case::one_minute_and_one_femtosecond_less_than_the_epoch(BaseTime { seconds: - SECONDS_PER_MINUTE - 1, subsecond: MAX_FEMTOSECONDS, }, 58)]
    fn test_base_time_wall_clock_minute(#[case] time: BaseTime, #[case] expected: i64) {
        let actual = time.minute();
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_value(BaseTime { seconds: 0, subsecond: Subsecond::default() }, 0)]
    #[case::one_femtosecond_less_than_one_second(BaseTime { seconds: 0, subsecond: MAX_FEMTOSECONDS, }, 0)]
    #[case::one_second(BaseTime { seconds: 1, subsecond: Subsecond::default() }, 1)]
    #[case::one_femtosecond_less_than_a_minute(BaseTime { seconds: SECONDS_PER_MINUTE - 1, subsecond: MAX_FEMTOSECONDS, }, 59)]
    #[case::exactly_one_minute(BaseTime { seconds: SECONDS_PER_MINUTE, subsecond: Subsecond::default() }, 0)]
    #[case::one_minute_and_one_second(BaseTime { seconds: SECONDS_PER_MINUTE + 1, subsecond: Subsecond::default() }, 1)]
    #[case::one_femtosecond_less_than_the_epoch(BaseTime { seconds: - 1, subsecond: MAX_FEMTOSECONDS, }, 59)]
    #[case::one_second_less_than_the_epoch(BaseTime { seconds: - 1, subsecond: Subsecond::default() }, 59)]
    #[case::one_second_and_one_femtosecond_less_than_the_epoch(BaseTime { seconds: - 2, subsecond: MAX_FEMTOSECONDS, }, 58)]
    fn test_base_time_wall_clock_second(#[case] time: BaseTime, #[case] expected: i64) {
        let actual = time.second();
        assert_eq!(expected, actual);
    }

    const POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE: BaseTime = BaseTime {
        seconds: 0,
        subsecond: Subsecond(0.123_456_789_012_345),
    };

    const NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE: BaseTime = BaseTime {
        seconds: -1,
        subsecond: Subsecond(0.123_456_789_012_345),
    };

    #[rstest]
    #[case::positive_time_millisecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::millisecond,
        123
    )]
    #[case::positive_time_microsecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::microsecond,
        456
    )]
    #[case::positive_time_nanosecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::nanosecond,
        789
    )]
    #[case::positive_time_picosecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::picosecond,
        12
    )]
    #[case::positive_time_femtosecond(
        POSITIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::femtosecond,
        345
    )]
    #[case::negative_time_millisecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::millisecond,
        123
    )]
    #[case::negative_time_microsecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::microsecond,
        456
    )]
    #[case::negative_time_nanosecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::nanosecond,
        789
    )]
    #[case::negative_time_picosecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::picosecond,
        12
    )]
    #[case::negative_time_femtosecond(
        NEGATIVE_BASE_TIME_SUBSECONDS_FIXTURE,
        WallClock::femtosecond,
        345
    )]
    fn test_base_time_subseconds(
        #[case] time: BaseTime,
        #[case] f: fn(&BaseTime) -> i64,
        #[case] expected: i64,
    ) {
        let actual = f(&time);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_delta(BaseTime::default(), TimeDelta::default(), BaseTime::default())]
    #[case::pos_delta_no_carry(BaseTime { seconds: 1, subsecond: Subsecond(0.3) }, TimeDelta { seconds: 1, subsecond: Subsecond(0.6) }, BaseTime { seconds: 2, subsecond: Subsecond(0.9) })]
    #[case::pos_delta_with_carry(BaseTime { seconds: 1, subsecond: Subsecond(0.3) }, TimeDelta { seconds: 1, subsecond: Subsecond(0.9) }, BaseTime { seconds: 3, subsecond: Subsecond(0.2) })]
    #[case::neg_delta_no_carry(BaseTime { seconds: 1, subsecond: Subsecond(0.6) }, TimeDelta { seconds: -2, subsecond: Subsecond(0.7) }, BaseTime { seconds: 0, subsecond: Subsecond(0.3) })]
    #[case::neg_delta_with_carry(BaseTime { seconds: 1, subsecond: Subsecond(0.6) }, TimeDelta { seconds: -2, subsecond: Subsecond(0.3) }, BaseTime { seconds: -1, subsecond: Subsecond(0.9) })]
    fn test_base_time_add_time_delta(
        #[case] time: BaseTime,
        #[case] delta: TimeDelta,
        #[case] expected: BaseTime,
    ) {
        let actual = time + delta;
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::zero_delta(BaseTime::default(), TimeDelta::default(), BaseTime::default())]
    #[case::pos_delta_no_carry(BaseTime { seconds: 1, subsecond: Subsecond(0.9) }, TimeDelta { seconds: 1, subsecond: Subsecond(0.3) }, BaseTime { seconds: 0, subsecond: Subsecond(0.6) })]
    #[case::pos_delta_with_carry(BaseTime { seconds: 1, subsecond: Subsecond(0.3) }, TimeDelta { seconds: 1, subsecond: Subsecond(0.4) }, BaseTime { seconds: -1, subsecond: Subsecond(0.9) })]
    #[case::neg_delta_no_carry(BaseTime { seconds: 1, subsecond: Subsecond(0.6) }, TimeDelta { seconds: -1, subsecond: Subsecond(0.7) }, BaseTime { seconds: 1, subsecond: Subsecond(0.9) })]
    #[case::neg_delta_with_carry(BaseTime { seconds: 1, subsecond: Subsecond(0.9) }, TimeDelta { seconds: -1, subsecond: Subsecond(0.3) }, BaseTime { seconds: 2, subsecond: Subsecond(0.6) })]
    fn test_base_time_sub_time_delta(
        #[case] time: BaseTime,
        #[case] delta: TimeDelta,
        #[case] expected: BaseTime,
    ) {
        let actual = time - delta;
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::at_the_epoch(BaseTime::default(), 0.0)]
    #[case::exactly_one_day_after_the_epoch(
    BaseTime {
    seconds: SECONDS_PER_DAY,
    subsecond: Subsecond::default(),
    },
    1.0
    )]
    #[case::exactly_one_day_before_the_epoch(
    BaseTime {
    seconds: - SECONDS_PER_DAY,
    subsecond: Subsecond::default(),
    },
    - 1.0
    )]
    #[case::a_partial_number_of_days_after_the_epoch(
    BaseTime {
    seconds: (SECONDS_PER_DAY / 2) * 3,
    subsecond: Subsecond(0.5),
    },
    1.5000057870370371
    )]
    fn test_base_time_days_since_j2000(#[case] time: BaseTime, #[case] expected: f64) {
        let actual = time.days_since_j2000();
        assert_float_eq!(expected, actual, abs <= 1e-12);
    }

    #[rstest]
    #[case::at_the_epoch(BaseTime::default(), 0.0)]
    #[case::exactly_one_century_after_the_epoch(
    BaseTime {
    seconds: SECONDS_PER_JULIAN_CENTURY,
    subsecond: Subsecond::default(),
    },
    1.0
    )]
    #[case::exactly_one_century_before_the_epoch(
    BaseTime {
    seconds: - SECONDS_PER_JULIAN_CENTURY,
    subsecond: Subsecond::default(),
    },
    - 1.0
    )]
    #[case::a_partial_number_of_centuries_after_the_epoch(
    BaseTime {
    seconds: (SECONDS_PER_JULIAN_CENTURY / 2) * 3,
    subsecond: Subsecond(0.5),
    },
    1.5000000001584404
    )]
    fn test_base_time_centuries_since_j2000(#[case] time: BaseTime, #[case] expected: f64) {
        let actual = time.centuries_since_j2000();
        assert_float_eq!(expected, actual, abs <= 1e-12,);
    }

    #[test]
    fn test_base_time_to_f64() {
        let time = BaseTime {
            seconds: 123,
            subsecond: Subsecond(0.123),
        };
        let expected = 123.123;
        let actual = time.to_f64();
        assert_float_eq!(expected, actual, abs <= 1e-15);
    }
}