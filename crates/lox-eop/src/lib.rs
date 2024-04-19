/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use thiserror::Error;

use lox_utils::types::julian_dates::ModifiedJulianDate;
use lox_utils::types::units::Seconds;

use crate::lagrange::eop::{interpolate, Arguments};

pub mod iers;
mod lagrange;

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum EopError {
    #[error("input vectors for EarthOrientationParams must have equal lengths, but got mjd.len()={len_mjd}, x_pole.len()={len_x_pole}, y_pole.len()={len_y_pole}, delta_ut1_utc.len()={len_delta_ut1_utc}")]
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

    pub fn dates(&self) -> &[ModifiedJulianDate] {
        &self.mjd
    }

    pub fn x_pole_series(&self) -> &[f64] {
        &self.x_pole
    }

    pub fn y_pole_series(&self) -> &[f64] {
        &self.y_pole
    }

    pub fn delta_ut1_utc_series(&self) -> &[f64] {
        &self.delta_ut1_utc
    }
}

/// Provides the difference between UT1 and UTC for the given [ModifiedJulianDate].
pub trait DeltaUt1Utc {
    fn delta_ut1_utc(&self, mjd: ModifiedJulianDate) -> Result<Seconds, TargetDateError>;
}

#[derive(Copy, Clone, Debug, Error, PartialEq)]
pub enum TargetDateError {
    #[error("input MJD was {input}, which predates the earliest available data (MJD {earliest})")]
    BeforeEopData {
        input: ModifiedJulianDate,
        earliest: ModifiedJulianDate,
    },
    #[error("input MJD was {input}, which is after the latest available data (MJD {latest})")]
    AfterEopData {
        input: ModifiedJulianDate,
        latest: ModifiedJulianDate,
    },
}

/// Performs Lagrangian interpolation for the target MJD, returning the difference between UT1 and
/// UTC, or `[TargetDateError]` if the target MJD is outside the range of available data.
impl DeltaUt1Utc for &EarthOrientationParams {
    fn delta_ut1_utc(&self, mjd: ModifiedJulianDate) -> Result<Seconds, TargetDateError> {
        if mjd < self.mjd[0] {
            return Err(TargetDateError::BeforeEopData {
                input: mjd,
                earliest: self.mjd[0],
            });
        }

        if mjd > self.mjd[self.mjd.len() - 1] {
            return Err(TargetDateError::AfterEopData {
                input: mjd,
                latest: self.mjd[self.mjd.len() - 1],
            });
        }

        let lagrange_args = Arguments::new(
            &self.x_pole,
            &self.y_pole,
            &self.delta_ut1_utc,
            &self.mjd,
            mjd,
        )
        .unwrap_or_else(|err| {
            // Unreachable for properly constructed `EarthOrientationParams`.
            panic!(
                "failed to create `Arguments` from `EarthOrientationParams`: {}",
                err
            )
        });

        let interpolation = interpolate(&lagrange_args);
        Ok(interpolation.d_ut1_utc)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use rstest::rstest;

    use super::*;

    struct EopInputs {
        mjd: Vec<ModifiedJulianDate>,
        x_pole: Vec<f64>,
        y_pole: Vec<f64>,
        delta_ut1_utc: Vec<f64>,
    }

    #[rstest]
    #[case::valid_inputs(
        EopInputs {
            mjd: vec![0.0],
            x_pole: vec![0.0],
            y_pole: vec![0.0],
            delta_ut1_utc: vec![0.0],
        },
        Ok(EarthOrientationParams {
            mjd: vec![0.0],
            x_pole: vec![0.0],
            y_pole: vec![0.0],
            delta_ut1_utc: vec![0.0],
        })
    )]
    #[case::extra_mjd(
        EopInputs {
            mjd: vec![0.0, 1.0],
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
            mjd: vec![0.0],
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
            mjd: vec![0.0],
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
            mjd: vec![0.0],
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

    #[test]
    fn test_eop_dates() {
        let eop = eop_fixture();
        assert_eq!(eop.dates(), &[0.0, 1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_eop_x_pole_series() {
        let eop = eop_fixture();
        assert_eq!(eop.x_pole_series(), &[4.0, 5.0, 6.0, 7.0]);
    }

    #[test]
    fn test_eop_y_pole_series() {
        let eop = eop_fixture();
        assert_eq!(eop.y_pole_series(), &[8.0, 9.0, 10.0, 11.0]);
    }

    #[test]
    fn test_eop_delta_ut1_utc_series() {
        let eop = eop_fixture();
        assert_eq!(eop.delta_ut1_utc_series(), &[0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn test_eop_delta_ut1_utc_ok() {
        let eop = eop_fixture();
        let mjd = 1.0;
        let args = Arguments::new(
            eop.x_pole_series(),
            eop.y_pole_series(),
            eop.dates(),
            eop.delta_ut1_utc_series(),
            mjd,
        )
        .unwrap();
        let expected: Result<Seconds, TargetDateError> = Ok(interpolate(&args).d_ut1_utc);
        let actual = eop.delta_ut1_utc(mjd);
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case::before_data(-1.0, TargetDateError::BeforeEopData { input: -1.0, earliest: 0.0 })]
    #[case::after_data(4.0, TargetDateError::AfterEopData { input: 4.0, latest: 3.0 })]
    fn test_eop_delta_ut1_utc_errors(
        #[case] mjd: ModifiedJulianDate,
        #[case] expected: TargetDateError,
    ) {
        let eop = eop_fixture();
        let actual = eop.delta_ut1_utc(mjd);
        assert_eq!(actual, Err(expected));
    }

    fn eop_fixture() -> &'static EarthOrientationParams {
        static EOP: OnceLock<EarthOrientationParams> = OnceLock::new();
        EOP.get_or_init(|| {
            EarthOrientationParams::new(
                vec![0.0, 1.0, 2.0, 3.0],
                vec![4.0, 5.0, 6.0, 7.0],
                vec![8.0, 9.0, 10.0, 11.0],
                vec![0.1, 0.2, 0.3, 0.4],
            )
            .unwrap()
        })
    }
}
