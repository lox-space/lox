/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Parse [EarthOrientationParams] from IERS CSV data.

#![cfg(test)]

use std::path::Path;

use serde::Deserialize;

use crate::{EarthOrientationParams, LoxEopError};

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "MJD")]
    modified_julian_date: i32,
    x_pole: Option<f64>,
    y_pole: Option<f64>,
    #[serde(rename = "UT1-UTC")]
    delta_ut1_utc: Option<f64>,
}

pub fn parse_finals_csv<P: AsRef<Path>>(path: P) -> Result<EarthOrientationParams, LoxEopError> {
    let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_path(path)?;
    let mut eop = EarthOrientationParams::default();
    for (i, result) in reader.deserialize().enumerate() {
        let record: Record = result?;
        if record.x_pole.is_none() {
            continue;
        }

        let x_pole = record.x_pole.unwrap();
        let y_pole = record.y_pole.unwrap_or_else(|| {
            panic!(
                "finals CSV record {} is missing y_pole despite present x_pole",
                i + 1,
            )
        });
        let delta_ut1_utc = record.delta_ut1_utc.unwrap_or_else(|| {
            panic!(
                "finals CSV record {} is missing delta_ut1_utc despite present x_pole",
                i + 1,
            )
        });

        eop.mjd.push(record.modified_julian_date as f64);
        eop.x_pole.push(x_pole);
        eop.y_pole.push(y_pole);
        eop.delta_ut1_utc.push(delta_ut1_utc);
    }

    Ok(eop)
}

#[cfg(test)]
mod tests {
    use lox_utils::types::julian_dates::ModifiedJulianDate;
    use rstest::rstest;
    use std::io;
    use std::path::Path;

    use crate::LoxEopError;

    use super::*;

    const TEST_DATA_DIR: &str = "../../data";

    #[derive(Default)]
    struct ExpectedRecord {
        mjd: ModifiedJulianDate,
        x_pole: f64,
        y_pole: f64,
        delta_ut1_utc: f64,
    }

    #[rstest]
    #[case::finals1980(
        "finals.all.csv",
        18933,
        ExpectedRecord {
            mjd: 41684.0,
            x_pole: 0.120733,
            y_pole: 0.136966,
            delta_ut1_utc: 0.8084178,
        },
        ExpectedRecord {
            mjd: 60616.0,
            x_pole: 0.236027,
            y_pole: 0.320683,
            delta_ut1_utc: 0.0428756,
        }
    )]
    #[case::finals2000a(
        "finals2000A.all.csv",
        19066,
        ExpectedRecord {
            mjd: 41684.0,
            x_pole: 0.120733,
            y_pole: 0.136966,
            delta_ut1_utc: 0.8084178,
        },
        ExpectedRecord {
            mjd: 60749.0,
            x_pole: 0.072472,
            y_pole: 0.295733,
            delta_ut1_utc: 0.0117914,
        }
    )]
    #[should_panic(expected = "finals CSV record 1 is missing y_pole despite present x_pole")]
    #[case::missing_y_pole(
        "iers_missing_y_pole.csv",
        0,
        ExpectedRecord::default(),
        ExpectedRecord::default()
    )]
    #[should_panic(
        expected = "finals CSV record 1 is missing delta_ut1_utc despite present x_pole"
    )]
    #[case::missing_delta_ut1_utc(
        "iers_missing_delta_ut1_utc.csv",
        0,
        ExpectedRecord::default(),
        ExpectedRecord::default()
    )]
    fn test_parse_finals_csv(
        #[case] path: &str,
        #[case] expected_count: usize,
        #[case] expected_first_record: ExpectedRecord,
        #[case] expected_last_record: ExpectedRecord,
    ) {
        let path = Path::new(TEST_DATA_DIR).join(path);
        let eop = parse_finals_csv(path).unwrap();
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

    #[test]
    fn test_lox_eop_error_from_csv_error() {
        // The csv::Error constructor is private, but we can create one using its implementation of
        // From<io::Error>.
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let csv_error = csv::Error::from(io_error);
        let lox_eop_error = LoxEopError::from(csv_error);
        let expected = LoxEopError::Csv("file not found".to_string());
        assert_eq!(lox_eop_error, expected);
    }

    #[test]
    fn test_lox_eop_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let lox_eop_error = LoxEopError::from(io_error);
        let expected = LoxEopError::Io("file not found".to_string());
        assert_eq!(lox_eop_error, expected);
    }
}
