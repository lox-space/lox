/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use thiserror::Error;

#[derive(Clone, Error, Debug)]
pub enum LoxTimeError {
    #[error("invalid date `{0}-{1}-{2}`")]
    InvalidDate(i64, i64, i64),
    #[error("invalid time `{0}:{1}:{2}`")]
    InvalidTime(u8, u8, u8),
    #[error("seconds must be in the range [0.0, 60.0], but was `{0}`")]
    InvalidSeconds(f64),
    #[error("subsecond must be in the range [0.0, 1.0), but was `{0}`")]
    InvalidSubsecond(f64),
    #[error("day of year cannot be 366 for a non-leap year")]
    NonLeapYear,
}

// Manual implementation of PartialEq to handle NaNs, which are not equal to themselves, whereas
// errors resulting from NaNs should be.
impl PartialEq for LoxTimeError {
    fn eq(&self, other: &Self) -> bool {
        use LoxTimeError::*;

        match (self, other) {
            (InvalidDate(y1, m1, d1), InvalidDate(y2, m2, d2)) => y1 == y2 && m1 == m2 && d1 == d2,
            (InvalidTime(h1, m1, s1), InvalidTime(h2, m2, s2)) => h1 == h2 && m1 == m2 && s1 == s2,
            (InvalidSeconds(f1), InvalidSeconds(f2))
            | (InvalidSubsecond(f1), InvalidSubsecond(f2)) => {
                if f1.is_nan() && f2.is_nan() {
                    return true;
                }

                f1 == f2
            }
            (NonLeapYear, NonLeapYear) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::LoxTimeError::*;
    use super::*;

    #[rstest]
    #[case(InvalidDate(2024, 2, 30), InvalidDate(2024, 2, 30), true)]
    #[case(InvalidDate(2024, 2, 30), InvalidDate(2024, 2, 29), false)]
    #[case(InvalidDate(2024, 2, 30), NonLeapYear, false)]
    #[case(InvalidTime(23, 59, 60), InvalidTime(23, 59, 60), true)]
    #[case(InvalidTime(23, 59, 60), InvalidTime(23, 59, 59), false)]
    #[case(InvalidTime(23, 59, 60), NonLeapYear, false)]
    #[case(NonLeapYear, NonLeapYear, true)]
    #[case(NonLeapYear, InvalidDate(2024, 2, 30), false)]
    #[case(InvalidSeconds(60.0), InvalidSeconds(60.0), true)]
    #[case(InvalidSeconds(f64::NAN), InvalidSeconds(f64::NAN), true)]
    #[case(InvalidSeconds(60.0), InvalidSeconds(59.0), false)]
    #[case(InvalidSeconds(60.0), NonLeapYear, false)]
    #[case(InvalidSubsecond(1.0), InvalidSubsecond(1.0), true)]
    #[case(InvalidSubsecond(f64::NAN), InvalidSubsecond(f64::NAN), true)]
    #[case(InvalidSubsecond(1.0), InvalidSubsecond(0.0), false)]
    #[case(InvalidSubsecond(1.0), NonLeapYear, false)]
    fn test_lox_time_error_eq(
        #[case] lhs: LoxTimeError,
        #[case] rhs: LoxTimeError,
        #[case] expected: bool,
    ) {
        assert_eq!(expected, lhs == rhs);
    }
}
