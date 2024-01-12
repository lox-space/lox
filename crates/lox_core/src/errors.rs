/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoxError {
    #[error("invalid date `{0}-{1}-{2}`")]
    InvalidDate(i64, i64, i64),
    #[error("invalid time `{0}:{1}:{2}`")]
    InvalidTime(u8, u8, u8),
    #[error("seconds must be in the range [0.0, 60.0], but was `{0}`")]
    InvalidSeconds(f64),
    #[error("thousandths must be in the range [0, 999], but was `{0}`")]
    InvalidThousandths(u16),
    #[error("day of year cannot be 366 for a non-leap year")]
    NonLeapYear,
}
