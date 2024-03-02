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
    #[error("`{raw:?}` cannot be represented as a `TimeDelta`")]
    InvalidTimeDelta { raw: f64, detail: String },
}

// Manual implementation of PartialEq to handle NaNs, which are not equal to themselves, whereas
// errors resulting from NaNs should be.
impl PartialEq for LoxTimeError {
    fn eq(&self, other: &Self) -> bool {
        use LoxTimeError::*;

        match (self, other) {
            (InvalidDate(_, _, _), InvalidDate(_, _, _))
            | (InvalidTime(_, _, _), InvalidTime(_, _, _))
            | (NonLeapYear, NonLeapYear) => self == other,
            (InvalidSeconds(f1), InvalidSeconds(f2))
            | (InvalidSubsecond(f1), InvalidSubsecond(f2)) => {
                if f1.is_nan() && f2.is_nan() {
                    return true;
                }

                f1 == f2
            }
            (
                InvalidTimeDelta {
                    raw: r1,
                    detail: d1,
                },
                InvalidTimeDelta {
                    raw: r2,
                    detail: d2,
                },
            ) => {
                if d1 != d2 {
                    return false;
                }

                if r1.is_nan() && r2.is_nan() {
                    return true;
                }

                r1 == r2
            }
            _ => false,
        }
    }
}
