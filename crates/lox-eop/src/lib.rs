/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use lox_utils::types::julian_dates::ModifiedJulianDate;
use lox_utils::types::units::Seconds;
use thiserror::Error;

mod iers;
mod lagrange;

#[derive(Clone, Debug, Error, PartialEq)]
pub enum LoxEopError {
    #[error("{0}")]
    Csv(String),
    #[error("{0}")]
    Io(String),
}

// Neither csv::Error nor std::io::Error are clone due to some sophisticated inner workings
// involving trait objects (at least in the case of io::Error), but there's no good reason that Lox
// error types shouldn't be cloneable. Otherwise, the whole chain of errors based on LoxEopError
// become non-Clone.
impl From<csv::Error> for LoxEopError {
    fn from(err: csv::Error) -> Self {
        LoxEopError::Csv(err.to_string())
    }
}

impl From<std::io::Error> for LoxEopError {
    fn from(err: std::io::Error) -> Self {
        LoxEopError::Io(err.to_string())
    }
}

/// A representation of observed Earth orientation parameters, independent of input format.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EarthOrientationParams {
    mjd: Vec<ModifiedJulianDate>,
    x_pole: Vec<f64>,
    y_pole: Vec<f64>,
    delta_ut1_utc: Vec<f64>,
}
