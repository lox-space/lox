/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::fs;
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

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

#[derive(Debug, Deserialize)]
enum ValueType {
    #[serde(rename = "prediction")]
    Prediction,
    #[serde(rename = "final")]
    Final,
}

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "MJD")]
    modified_julian_date: i32,
    x_pole: Option<f64>,
    sigma_x_pole: Option<f64>,
    y_pole: Option<f64>,
    sigma_y_pole: Option<f64>,
    #[serde(rename = "UT1-UTC")]
    delta_ut1_utc: Option<f64>,
    #[serde(rename = "sigma_UT1-UTC")]
    sigma_delta_ut1_utc: Option<f64>,
    #[serde(rename = "LOD")]
    lod: Option<f64>,
    #[serde(rename = "sigma_LOD")]
    sigma_lod: Option<f64>,
    #[serde(rename = "dPsi")]
    delta_psi: Option<f64>,
    #[serde(rename = "sigma_dPsi")]
    sigma_delta_psi: Option<f64>,
    #[serde(rename = "dEpsilon")]
    delta_epsilon: Option<f64>,
    #[serde(rename = "sigma_dEpsilon")]
    sigma_delta_epsilon: Option<f64>,
    #[serde(rename = "dX")]
    delta_x: Option<f64>,
    #[serde(rename = "sigma_dX")]
    sigma_delta_x: Option<f64>,
    #[serde(rename = "dY")]
    delta_y: Option<f64>,
    #[serde(rename = "sigma_dY")]
    sigma_delta_y: Option<f64>,
}

// TODO: This is currently used only in tests, but will be used by the public interface in future.
#[allow(dead_code)]
fn read_records<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, LoxEopError> {
    let contents = fs::read_to_string(path)?
        // Replace duplicate `Type` headers
        .replacen("Type", "type_polar_motion", 1)
        .replacen("Type", "type_rotation", 1)
        .replacen("Type", "type_equator", 1)
        .replacen("Type", "type_polar_motion_b", 1)
        .replacen("Type", "type_rotation_b", 1)
        .replacen("Type", "type_equator_b", 1);
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(contents.as_bytes());
    let mut records: Vec<Record> = vec![];
    for result in rdr.deserialize() {
        records.push(result?);
    }
    Ok(records)
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Records {
    modified_julian_date: Vec<i32>,
    x_pole: Vec<Option<f64>>,
    sigma_x_pole: Vec<Option<f64>>,
    y_pole: Vec<Option<f64>>,
    sigma_y_pole: Vec<Option<f64>>,
    delta_ut1_utc: Vec<Option<f64>>,
    sigma_delta_ut1_utc: Vec<Option<f64>>,
    lod: Vec<Option<f64>>,
    sigma_lod: Vec<Option<f64>>,
    delta_psi: Vec<Option<f64>>,
    sigma_delta_psi: Vec<Option<f64>>,
    delta_epsilon: Vec<Option<f64>>,
    sigma_delta_epsilon: Vec<Option<f64>>,
    delta_x: Vec<Option<f64>>,
    sigma_delta_x: Vec<Option<f64>>,
    delta_y: Vec<Option<f64>>,
    sigma_delta_y: Vec<Option<f64>>,
}

impl Records {
    fn push(&mut self, record: &Record) {
        self.modified_julian_date.push(record.modified_julian_date);
        self.x_pole.push(record.x_pole);
        self.sigma_x_pole.push(record.sigma_x_pole);
        self.y_pole.push(record.y_pole);
        self.sigma_y_pole.push(record.sigma_y_pole);
        self.delta_ut1_utc.push(record.delta_ut1_utc);
        self.sigma_delta_ut1_utc.push(record.sigma_delta_ut1_utc);
        self.lod.push(record.lod);
        self.sigma_lod.push(record.sigma_lod);
        self.delta_psi.push(record.delta_psi);
        self.sigma_delta_psi.push(record.sigma_delta_psi);
        self.delta_epsilon.push(record.delta_epsilon);
        self.sigma_delta_epsilon.push(record.sigma_delta_epsilon);
        self.delta_x.push(record.delta_x);
        self.sigma_delta_x.push(record.sigma_delta_x);
        self.delta_y.push(record.delta_y);
        self.sigma_delta_y.push(record.sigma_delta_y);
    }
}

impl From<Vec<Record>> for Records {
    fn from(record_vec: Vec<Record>) -> Self {
        let mut records = Records::default();
        record_vec
            .iter()
            .filter(|record| record.x_pole.is_some())
            .for_each(|record| records.push(record));
        records
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::Path;

    use super::*;

    const TEST_DATA_DIR: &str = "../../data";

    #[test]
    fn test_records_push() {
        let record = Record {
            modified_julian_date: 0,
            x_pole: Some(0.0),
            sigma_x_pole: Some(0.0),
            y_pole: Some(0.0),
            sigma_y_pole: Some(0.0),
            delta_ut1_utc: Some(0.0),
            sigma_delta_ut1_utc: Some(0.0),
            lod: Some(0.0),
            sigma_lod: Some(0.0),
            delta_psi: Some(0.0),
            sigma_delta_psi: Some(0.0),
            delta_epsilon: Some(0.0),
            sigma_delta_epsilon: Some(0.0),
            delta_x: Some(0.0),
            sigma_delta_x: Some(0.0),
            delta_y: Some(0.0),
            sigma_delta_y: Some(0.0),
        };
        let mut records = Records::default();
        records.push(&record);
        let expected = Records {
            modified_julian_date: vec![0],
            x_pole: vec![Some(0.0)],
            sigma_x_pole: vec![Some(0.0)],
            y_pole: vec![Some(0.0)],
            sigma_y_pole: vec![Some(0.0)],
            delta_ut1_utc: vec![Some(0.0)],
            sigma_delta_ut1_utc: vec![Some(0.0)],
            lod: vec![Some(0.0)],
            sigma_lod: vec![Some(0.0)],
            delta_psi: vec![Some(0.0)],
            sigma_delta_psi: vec![Some(0.0)],
            delta_epsilon: vec![Some(0.0)],
            sigma_delta_epsilon: vec![Some(0.0)],
            delta_x: vec![Some(0.0)],
            sigma_delta_x: vec![Some(0.0)],
            delta_y: vec![Some(0.0)],
            sigma_delta_y: vec![Some(0.0)],
        };
        assert_eq!(records, expected);
    }

    #[test]
    fn test_records_from_vec_record() {
        let record = Record {
            modified_julian_date: 0,
            x_pole: Some(0.0),
            sigma_x_pole: Some(0.0),
            y_pole: Some(0.0),
            sigma_y_pole: Some(0.0),
            delta_ut1_utc: Some(0.0),
            sigma_delta_ut1_utc: Some(0.0),
            lod: Some(0.0),
            sigma_lod: Some(0.0),
            delta_psi: Some(0.0),
            sigma_delta_psi: Some(0.0),
            delta_epsilon: Some(0.0),
            sigma_delta_epsilon: Some(0.0),
            delta_x: Some(0.0),
            sigma_delta_x: Some(0.0),
            delta_y: Some(0.0),
            sigma_delta_y: Some(0.0),
        };
        let records = Records::from(vec![record]);
        let expected = Records {
            modified_julian_date: vec![0],
            x_pole: vec![Some(0.0)],
            sigma_x_pole: vec![Some(0.0)],
            y_pole: vec![Some(0.0)],
            sigma_y_pole: vec![Some(0.0)],
            delta_ut1_utc: vec![Some(0.0)],
            sigma_delta_ut1_utc: vec![Some(0.0)],
            lod: vec![Some(0.0)],
            sigma_lod: vec![Some(0.0)],
            delta_psi: vec![Some(0.0)],
            sigma_delta_psi: vec![Some(0.0)],
            delta_epsilon: vec![Some(0.0)],
            sigma_delta_epsilon: vec![Some(0.0)],
            delta_x: vec![Some(0.0)],
            sigma_delta_x: vec![Some(0.0)],
            delta_y: vec![Some(0.0)],
            sigma_delta_y: vec![Some(0.0)],
        };
        assert_eq!(records, expected);
    }

    #[test]
    fn test_csv() {
        let finals1980 = Path::new(TEST_DATA_DIR).join("finals.all.csv");
        let finals2000a = Path::new(TEST_DATA_DIR).join("finals2000A.all.csv");
        let records_1980 = read_records(finals1980).expect("file should be readable");
        let records_2000a = read_records(finals2000a).expect("file should be readable");
        let first_1980 = records_1980.first();
        assert!(first_1980.is_some());
        assert!(first_1980.unwrap().delta_psi.is_some());
        assert!(first_1980.unwrap().sigma_delta_psi.is_some());
        assert!(first_1980.unwrap().delta_epsilon.is_some());
        assert!(first_1980.unwrap().sigma_delta_epsilon.is_some());
        assert!(first_1980.unwrap().delta_x.is_none());
        assert!(first_1980.unwrap().sigma_delta_x.is_none());
        assert!(first_1980.unwrap().delta_y.is_none());
        assert!(first_1980.unwrap().sigma_delta_y.is_none());
        let first_2000a = records_2000a.first();
        assert!(first_2000a.is_some());
        assert!(first_2000a.unwrap().delta_x.is_some());
        assert!(first_2000a.unwrap().sigma_delta_x.is_some());
        assert!(first_2000a.unwrap().delta_y.is_some());
        assert!(first_2000a.unwrap().sigma_delta_y.is_some());
        assert!(first_2000a.unwrap().delta_psi.is_none());
        assert!(first_2000a.unwrap().sigma_delta_psi.is_none());
        assert!(first_2000a.unwrap().delta_epsilon.is_none());
        assert!(first_2000a.unwrap().sigma_delta_epsilon.is_none());
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
