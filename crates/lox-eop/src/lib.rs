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

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum EopError {
    #[error("input vectors for EarthOrientationParams must have equal lengths, but got mjd={len_mjd}, x_pole={len_x_pole}, y_pole={len_y_pole}, delta_ut1_utc={len_delta_ut1_utc}")]
    DimensionMismatch {
        len_mjd: usize,
        len_x_pole: usize,
        len_y_pole: usize,
        len_delta_ut1_utc: usize,
    },
    #[error("EarthOrientationParams cannot be empty, but empty input vectors were provided")]
    NoData,
}

/// A representation of observed Earth orientation parameters, independent of input format.
#[derive(Clone, Debug, PartialEq)]
pub struct EarthOrientationParams {
    mjd: Vec<ModifiedJulianDate>,
    x_pole: Vec<f64>,
    y_pole: Vec<f64>,
    delta_ut1_utc: Vec<f64>,
}

impl EarthOrientationParams {
    pub fn new(
        mjd: Vec<ModifiedJulianDate>,
        x_pole: Vec<f64>,
        y_pole: Vec<f64>,
        delta_ut1_utc: Vec<f64>,
    ) -> Result<Self, EopError> {
        if mjd.len() != x_pole.len()
            || mjd.len() != y_pole.len()
            || mjd.len() != delta_ut1_utc.len()
        {
            return Err(EopError::DimensionMismatch {
                len_mjd: mjd.len(),
                len_x_pole: x_pole.len(),
                len_y_pole: y_pole.len(),
                len_delta_ut1_utc: delta_ut1_utc.len(),
            });
        }

        if mjd.is_empty() {
            return Err(EopError::NoData);
        }

        Ok(EarthOrientationParams {
            mjd,
            x_pole,
            y_pole,
            delta_ut1_utc,
        })
    }
}

#[derive(Copy, Clone, Debug, Error, PartialEq)]
pub enum DateError {
    #[error("input MJD {input} is before earliest EOP data MJD {earliest}")]
    BeforeData {
        input: ModifiedJulianDate,
        earliest: ModifiedJulianDate,
    },
    #[error("input MJD {input} is after latest EOP data MJD {latest}")]
    AfterData {
        input: ModifiedJulianDate,
        latest: ModifiedJulianDate,
    },
}

/// Implementers of [DeltaUt1Tai] provide the difference between UT1 and TAI as a floating-point
/// number of seconds.  
pub trait DeltaUt1Tai {
    fn delta_ut1_tai(&self, mjd: ModifiedJulianDate) -> Result<Seconds, DateError>;
}

// impl DeltaUt1Tai for EarthOrientationParams {
//     fn delta_ut1_tai(&self, mjd: ModifiedJulianDate) -> Result<Seconds, DateError> {
//         // getΔUT1(eop, date; args...) = interpolate(eop, :ΔUT1, date; args...)
//         if mjd < *self.mjd.first().unwrap() {
//             return Err(DateError::BeforeData {
//                 input: mjd,
//                 earliest: self.mjd[0],
//             });
//         }
//
//         if mjd > *self.mjd.last().unwrap() {
//             || mjd > self.mjd[self.mjd.len() - 1] {
//             Err()
//         }
//         Ok(0.0)
//     }
// }
