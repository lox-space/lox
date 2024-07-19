/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Parse [EarthOrientationParams] from IERS CSV data.

use std::path::{Path, PathBuf};

use lox_math::types::julian_dates::ModifiedJulianDayNumber;
use serde::Deserialize;
use thiserror::Error;

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

    pub fn parse_finals_csv<P: AsRef<Path>>(path: P) -> Result<Self, ParseFinalsCsvError> {
        let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_path(&path)?;
        let mut mjd = Vec::new();
        let mut x_pole = Vec::new();
        let mut y_pole = Vec::new();
        let mut delta_ut1_utc = Vec::new();

        for (i, result) in reader.deserialize().enumerate() {
            let record: Record = result?;
            if record.x_pole.is_none() {
                continue;
            }

            let record_x_pole = record.x_pole.unwrap();
            let record_y_pole = record
                .y_pole
                .ok_or_else(|| ParseFinalsCsvError::MissingData {
                    path: path.as_ref().to_path_buf(),
                    row: i + 1,
                })?;
            let record_delta_ut1_utc =
                record
                    .delta_ut1_utc
                    .ok_or_else(|| ParseFinalsCsvError::MissingData {
                        path: path.as_ref().to_path_buf(),
                        row: i + 1,
                    })?;

            mjd.push(record.modified_julian_date);
            x_pole.push(record_x_pole);
            y_pole.push(record_y_pole);
            delta_ut1_utc.push(record_delta_ut1_utc);
        }

        Self::new(mjd, x_pole, y_pole, delta_ut1_utc).map_err(|e| ParseFinalsCsvError::InvalidEop {
            path: path.as_ref().to_path_buf(),
            source: e,
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

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ParseFinalsCsvError {
    #[error("{0}")]
    Csv(String),
    #[error("finals CSV at `{path}` is missing data from row {row}")]
    MissingData { path: PathBuf, row: usize },
    #[error("CSV file at `{path}` contains invalid data: {source}")]
    InvalidEop { path: PathBuf, source: EopError },
}

// csv::Error is not Clone, but there's no good reason that Lox error types shouldn't be
// cloneable. Otherwise, the whole chain of errors based on LoxEopError become non-Clone.
impl From<csv::Error> for ParseFinalsCsvError {
    fn from(err: csv::Error) -> Self {
        ParseFinalsCsvError::Csv(err.to_string())
    }
}

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "MJD")]
    modified_julian_date: i32,
    x_pole: Option<f64>,
    y_pole: Option<f64>,
    #[serde(rename = "UT1-UTC")]
    delta_ut1_utc: Option<f64>,
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use rstest::rstest;

    use lox_math::types::julian_dates::ModifiedJulianDayNumber;

    use super::*;

    const TEST_DATA_DIR: &str = "../../data";

    #[derive(Default)]
    struct ExpectedRecord {
        mjd: ModifiedJulianDayNumber,
        x_pole: f64,
        y_pole: f64,
        delta_ut1_utc: f64,
    }

    #[rstest]
    #[case::finals1980(
        "finals.all.csv",
        18933,
        ExpectedRecord {
            mjd: 41684,
            x_pole: 0.120733,
            y_pole: 0.136966,
            delta_ut1_utc: 0.8084178,
        },
        ExpectedRecord {
            mjd: 60616,
            x_pole: 0.236027,
            y_pole: 0.320683,
            delta_ut1_utc: 0.0428756,
        }
    )]
    #[case::finals2000a(
        "finals2000A.all.csv",
        19066,
        ExpectedRecord {
            mjd: 41684,
            x_pole: 0.120733,
            y_pole: 0.136966,
            delta_ut1_utc: 0.8084178,
        },
        ExpectedRecord {
            mjd: 60749,
            x_pole: 0.072472,
            y_pole: 0.295733,
            delta_ut1_utc: 0.0117914,
        }
    )]
    fn test_parse_finals_csv_success(
        #[case] path: &str,
        #[case] expected_count: usize,
        #[case] expected_first_record: ExpectedRecord,
        #[case] expected_last_record: ExpectedRecord,
    ) {
        let path = Path::new(TEST_DATA_DIR).join(path);
        let eop = EarthOrientationParams::parse_finals_csv(path).unwrap();
        assert_eq!(
            eop.mjd.len(),
            expected_count,
            "expected {} MJD values, but got {}",
            expected_count,
            eop.mjd.len()
        );
        assert_eq!(
            eop.x_pole.len(),
            expected_count,
            "expected {} x_pole values, but got {}",
            expected_count,
            eop.x_pole.len()
        );
        assert_eq!(
            eop.y_pole.len(),
            expected_count,
            "expected {} y_pole values, but got {}",
            expected_count,
            eop.y_pole.len()
        );
        assert_eq!(
            eop.delta_ut1_utc.len(),
            expected_count,
            "expected {} delta_ut1_utc values, but got {}",
            expected_count,
            eop.delta_ut1_utc.len()
        );

        let first_mjd = eop.mjd.first().unwrap();
        let first_x_pole = eop.x_pole.first().unwrap();
        let first_y_pole = eop.y_pole.first().unwrap();
        let first_delta_ut1_utc = eop.delta_ut1_utc.first().unwrap();
        assert_eq!(
            *first_mjd, expected_first_record.mjd,
            "expected first MJD to be {}, but was {}",
            expected_first_record.mjd, *first_mjd
        );
        assert_eq!(
            *first_x_pole, expected_first_record.x_pole,
            "expected first x_pole value to be {}, but was {}",
            expected_first_record.x_pole, *first_x_pole
        );
        assert_eq!(
            *first_y_pole, expected_first_record.y_pole,
            "expected first y_pole value to be {}, but was {}",
            expected_first_record.y_pole, *first_y_pole
        );
        assert_eq!(
            *first_delta_ut1_utc, expected_first_record.delta_ut1_utc,
            "expected first delta_ut1_utc value to be {}, but was {}",
            expected_first_record.delta_ut1_utc, *first_delta_ut1_utc
        );

        let last_mjd = eop.mjd.last().unwrap();
        let last_x_pole = eop.x_pole.last().unwrap();
        let last_y_pole = eop.y_pole.last().unwrap();
        let last_delta_ut1_utc = eop.delta_ut1_utc.last().unwrap();
        assert_eq!(
            *last_mjd, expected_last_record.mjd,
            "expected last MJD to be {}, but was {}",
            expected_last_record.mjd, *last_mjd
        );
        assert_eq!(
            *last_x_pole, expected_last_record.x_pole,
            "expected last x_pole value to be {}, but was {}",
            expected_last_record.x_pole, *last_x_pole
        );
        assert_eq!(
            *last_y_pole, expected_last_record.y_pole,
            "expected last y_pole value to be {}, but was {}",
            expected_last_record.y_pole, *last_y_pole
        );
        assert_eq!(
            *last_delta_ut1_utc, expected_last_record.delta_ut1_utc,
            "expected last delta_ut1_utc value to be {}, but was {}",
            expected_last_record.delta_ut1_utc, *last_delta_ut1_utc
        );
    }

    #[rstest]
    #[case::csv_no_such_file("missing.csv", ParseFinalsCsvError::Csv("No such file or directory (os error 2)".to_string()))]
    #[case::csv_parse_failure("finals_type_error.csv", ParseFinalsCsvError::Csv("CSV deserialize error: record 1 (line: 2, byte: 265): field 0: invalid digit found in string".to_string()))]
    #[case::missing_y_pole(
        "finals_missing_y_pole.csv",
        ParseFinalsCsvError::MissingData {
            path: Path::new(TEST_DATA_DIR).join(Path::new("finals_missing_y_pole.csv")),
            row: 1,
        },
    )]
    #[case::missing_delta_ut1_utc(
        "finals_missing_delta_ut1_utc.csv",
        ParseFinalsCsvError::MissingData {
            path: Path::new(TEST_DATA_DIR).join(Path::new("finals_missing_delta_ut1_utc.csv")),
            row: 1,
        },
    )]
    #[case::no_data(
        "finals_no_data.csv",
        ParseFinalsCsvError::InvalidEop {
            path: Path::new(TEST_DATA_DIR).join(Path::new("finals_no_data.csv")),
            source: EopError::NoData,
        },
    )]
    fn test_parse_finals_csv_errors(#[case] path: &str, #[case] expected: ParseFinalsCsvError) {
        let path = Path::new(TEST_DATA_DIR).join(path);
        let result = EarthOrientationParams::parse_finals_csv(path);
        assert_eq!(result, Err(expected));
    }
}
