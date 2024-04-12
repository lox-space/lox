/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use thiserror::Error;

use lox_utils::types::julian_dates::ModifiedJulianDayNumber;

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
    mjd: Vec<ModifiedJulianDayNumber>,
    x_pole: Vec<f64>,
    y_pole: Vec<f64>,
    delta_ut1_utc: Vec<f64>,
}

impl EarthOrientationParams {
    pub fn new(
        mjd: Vec<ModifiedJulianDayNumber>,
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

    pub fn mjd(&self) -> &[ModifiedJulianDayNumber] {
        &self.mjd
    }

    pub fn x_pole(&self) -> &[f64] {
        &self.x_pole
    }

    pub fn y_pole(&self) -> &[f64] {
        &self.y_pole
    }

    pub fn delta_ut1_utc(&self) -> &[f64] {
        &self.delta_ut1_utc
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    struct EopInputs {
        mjd: Vec<ModifiedJulianDayNumber>,
        x_pole: Vec<f64>,
        y_pole: Vec<f64>,
        delta_ut1_utc: Vec<f64>,
    }

    #[rstest]
    #[case::valid_inputs(
        EopInputs {
            mjd: vec![0],
            x_pole: vec![0.0],
            y_pole: vec![0.0],
            delta_ut1_utc: vec![0.0],
        },
        Ok(EarthOrientationParams {
            mjd: vec![0],
            x_pole: vec![0.0],
            y_pole: vec![0.0],
            delta_ut1_utc: vec![0.0],
        })
    )]
    #[case::extra_mjd(
        EopInputs {
            mjd: vec![0, 1],
            x_pole: vec![0.0],
            y_pole: vec![0.0],
            delta_ut1_utc: vec![0.0],
        },
        Err(EopError::DimensionMismatch {
            len_mjd: 2,
            len_x_pole: 1,
            len_y_pole: 1,
            len_delta_ut1_utc: 1,
        })
    )]
    #[case::extra_x_pole(
        EopInputs {
            mjd: vec![0],
            x_pole: vec![0.0, 1.0],
            y_pole: vec![0.0],
            delta_ut1_utc: vec![0.0],
        },
        Err(EopError::DimensionMismatch {
            len_mjd: 1,
            len_x_pole: 2,
            len_y_pole: 1,
            len_delta_ut1_utc: 1,
        })
    )]
    #[case::extra_y_pole(
        EopInputs {
            mjd: vec![0],
            x_pole: vec![0.0],
            y_pole: vec![0.0, 1.0],
            delta_ut1_utc: vec![0.0],
        },
        Err(EopError::DimensionMismatch {
            len_mjd: 1,
            len_x_pole: 1,
            len_y_pole: 2,
            len_delta_ut1_utc: 1,
        })
    )]
    #[case::extra_delta_ut1_utc(
        EopInputs {
            mjd: vec![0],
            x_pole: vec![0.0],
            y_pole: vec![0.0],
            delta_ut1_utc: vec![0.0, 1.0],
        },
        Err(EopError::DimensionMismatch {
            len_mjd: 1,
            len_x_pole: 1,
            len_y_pole: 1,
            len_delta_ut1_utc: 2,
        })
    )]
    #[case::empty_inputs(
        EopInputs {
            mjd: vec![],
            x_pole: vec![],
            y_pole: vec![],
            delta_ut1_utc: vec![],
        },
        Err(EopError::NoData)
    )]
    fn test_earth_orientation_params_new(
        #[case] inputs: EopInputs,
        #[case] expected: Result<EarthOrientationParams, EopError>,
    ) {
        let actual = EarthOrientationParams::new(
            inputs.mjd,
            inputs.x_pole,
            inputs.y_pole,
            inputs.delta_ut1_utc,
        );
        assert_eq!(actual, expected);
    }
}
