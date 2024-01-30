/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

use crate::akima::{Akima, AkimaError};

mod akima;

#[derive(Error, Debug)]
enum LoxEopError {
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Interpolation(#[from] AkimaError),
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
    // #[serde(rename = "Year")]
    // year: u64,
    // #[serde(rename = "Month")]
    // month: u64,
    // #[serde(rename = "Day")]
    // day: u64,
    // type_polar_motion: ValueType,
    x_pole: Option<f64>,
    sigma_x_pole: Option<f64>,
    y_pole: Option<f64>,
    sigma_y_pole: Option<f64>,
    // type_rotation: String,
    #[serde(rename = "UT1-UTC")]
    delta_ut1_utc: Option<f64>,
    #[serde(rename = "sigma_UT1-UTC")]
    sigma_delta_ut1_utc: Option<f64>,
    #[serde(rename = "LOD")]
    lod: Option<f64>,
    #[serde(rename = "sigma_LOD")]
    sigma_lod: Option<f64>,
    // type_equator: String,
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
    // type_polar_motion_b: String,
    // #[serde(rename = "bulB/x_pole")]
    // x_pole_b: Option<f64>,
    // #[serde(rename = "bulB/y_pole")]
    // y_pole_b: Option<f64>,
    // type_rotation_b: String,
    // #[serde(rename = "bulB/UT-UTC")]
    // delta_ut1_utc_b: Option<f64>,
    // type_equator_b: String,
    // #[serde(rename = "bulB/dPsi")]
    // delta_psi_b: Option<f64>,
    // #[serde(rename = "bulB/dEpsilon")]
    // delta_epsilon_b: Option<f64>,
    // #[serde(rename = "bulB/dX")]
    // delta_x_b: Option<f64>,
    // #[serde(rename = "bulB/dY")]
    // delta_y_b: Option<f64>,
}

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
    fn new() -> Self {
        Self {
            modified_julian_date: vec![],
            x_pole: vec![],
            sigma_x_pole: vec![],
            y_pole: vec![],
            sigma_y_pole: vec![],
            delta_ut1_utc: vec![],
            sigma_delta_ut1_utc: vec![],
            lod: vec![],
            sigma_lod: vec![],
            delta_psi: vec![],
            sigma_delta_psi: vec![],
            delta_epsilon: vec![],
            sigma_delta_epsilon: vec![],
            delta_x: vec![],
            sigma_delta_x: vec![],
            delta_y: vec![],
            sigma_delta_y: vec![],
        }
    }

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
        let mut records = Records::new();
        record_vec
            .iter()
            .filter(|record| record.x_pole.is_some())
            .for_each(|record| records.push(record));
        records
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum EopConvention {
    Iau1980,
    Iau2000a,
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
enum EopFields {
    Xpole,
    SigmaXpole,
    Ypole,
    SigmaYpole,
    DeltaUt1Utc,
    SigmaDeltaUt1Utc,
    Lod,
    SigmaLod,
    DeltaPsi,
    SigmaDeltaPsi,
    DeltaEpsilon,
    SigmaDeltaEpsilon,
    DeltaX,
    SigmaDeltaX,
    DeltaY,
    SigmaDeltaY,
}

struct EarthOrientation {
    convention: EopConvention,
    interpolators: HashMap<EopFields, Akima>,
}

impl EarthOrientation {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, LoxEopError> {
        let records: Records = read_records(path)?.into();
        Ok(records.try_into()?)
    }

    pub fn convention(&self) -> EopConvention {
        self.convention
    }

    pub fn interpolate(&self, field: EopFields, mjd: f64) -> f64 {
        self.interpolators[&field].interpolate(mjd)
    }

    pub fn polar_motion_x(&self, mjd: f64) -> f64 {
        self.interpolate(EopFields::Xpole, mjd)
    }

    pub fn polar_motion_x_error(&self, mjd: f64) -> f64 {
        self.interpolate(EopFields::SigmaXpole, mjd)
    }

    pub fn polar_motion_y(&self, mjd: f64) -> f64 {
        self.interpolate(EopFields::Ypole, mjd)
    }

    pub fn polar_motion_y_error(&self, mjd: f64) -> f64 {
        self.interpolate(EopFields::SigmaYpole, mjd)
    }

    pub fn polar_motion(&self, mjd: f64) -> (f64, f64) {
        (self.polar_motion_x(mjd), self.polar_motion_y(mjd))
    }
}

impl TryFrom<Records> for EarthOrientation {
    type Error = AkimaError;
    fn try_from(records: Records) -> Result<Self, Self::Error> {
        let convention = match records.delta_x[0] {
            Some(_) => EopConvention::Iau2000a,
            None => EopConvention::Iau1980,
        };

        let modified_julian_date = records.modified_julian_date;
        let mut interpolators: HashMap<EopFields, Akima> = HashMap::new();

        let x_pole: Vec<f64> = records.x_pole.into_iter().flatten().collect();
        interpolators.insert(
            EopFields::Xpole,
            Akima::new(Vec::from(&modified_julian_date[0..x_pole.len()]), x_pole)?,
        );

        let sigma_x_pole: Vec<f64> = records.sigma_x_pole.into_iter().flatten().collect();
        interpolators.insert(
            EopFields::SigmaXpole,
            Akima::new(
                Vec::from(&modified_julian_date[0..sigma_x_pole.len()]),
                sigma_x_pole,
            )?,
        );
        let y_pole: Vec<f64> = records.y_pole.into_iter().flatten().collect();
        interpolators.insert(
            EopFields::Ypole,
            Akima::new(Vec::from(&modified_julian_date[0..y_pole.len()]), y_pole)?,
        );

        let sigma_y_pole: Vec<f64> = records.sigma_y_pole.into_iter().flatten().collect();
        interpolators.insert(
            EopFields::SigmaYpole,
            Akima::new(
                Vec::from(&modified_julian_date[0..sigma_y_pole.len()]),
                sigma_y_pole,
            )?,
        );

        let delta_ut1_utc: Vec<f64> = records
            .delta_ut1_utc
            .clone()
            .into_iter()
            .flatten()
            .collect();
        interpolators.insert(
            EopFields::DeltaUt1Utc,
            Akima::new(
                Vec::from(&modified_julian_date[0..delta_ut1_utc.len()]),
                delta_ut1_utc,
            )?,
        );

        let sigma_delta_ut1_utc: Vec<f64> = records
            .sigma_delta_ut1_utc
            .clone()
            .into_iter()
            .flatten()
            .collect();
        interpolators.insert(
            EopFields::SigmaDeltaUt1Utc,
            Akima::new(
                Vec::from(&modified_julian_date[0..sigma_delta_ut1_utc.len()]),
                sigma_delta_ut1_utc,
            )?,
        );

        let lod: Vec<f64> = records.lod.clone().into_iter().flatten().collect();
        interpolators.insert(
            EopFields::Lod,
            Akima::new(Vec::from(&modified_julian_date[0..lod.len()]), lod)?,
        );

        let sigma_lod: Vec<f64> = records.sigma_lod.clone().into_iter().flatten().collect();
        interpolators.insert(
            EopFields::SigmaLod,
            Akima::new(
                Vec::from(&modified_julian_date[0..sigma_lod.len()]),
                sigma_lod,
            )?,
        );

        match convention {
            EopConvention::Iau1980 => {
                let delta_psi: Vec<f64> = records.delta_psi.clone().into_iter().flatten().collect();
                interpolators.insert(
                    EopFields::DeltaPsi,
                    Akima::new(
                        Vec::from(&modified_julian_date[0..delta_psi.len()]),
                        delta_psi,
                    )?,
                );

                let sigma_delta_psi: Vec<f64> = records
                    .sigma_delta_psi
                    .clone()
                    .into_iter()
                    .flatten()
                    .collect();
                interpolators.insert(
                    EopFields::SigmaDeltaPsi,
                    Akima::new(
                        Vec::from(&modified_julian_date[0..sigma_delta_psi.len()]),
                        sigma_delta_psi,
                    )?,
                );

                let delta_epsilon: Vec<f64> = records
                    .delta_epsilon
                    .clone()
                    .into_iter()
                    .flatten()
                    .collect();
                interpolators.insert(
                    EopFields::DeltaEpsilon,
                    Akima::new(
                        Vec::from(&modified_julian_date[0..delta_epsilon.len()]),
                        delta_epsilon,
                    )?,
                );

                let sigma_delta_epsilon: Vec<f64> = records
                    .sigma_delta_epsilon
                    .clone()
                    .into_iter()
                    .flatten()
                    .collect();
                interpolators.insert(
                    EopFields::SigmaDeltaEpsilon,
                    Akima::new(
                        Vec::from(&modified_julian_date[0..sigma_delta_epsilon.len()]),
                        sigma_delta_epsilon,
                    )?,
                );
            }
            EopConvention::Iau2000a => {
                let delta_x: Vec<f64> = records.delta_x.clone().into_iter().flatten().collect();
                interpolators.insert(
                    EopFields::DeltaX,
                    Akima::new(Vec::from(&modified_julian_date[0..delta_x.len()]), delta_x)?,
                );

                let sigma_delta_x: Vec<f64> = records
                    .sigma_delta_x
                    .clone()
                    .into_iter()
                    .flatten()
                    .collect();
                interpolators.insert(
                    EopFields::SigmaDeltaX,
                    Akima::new(
                        Vec::from(&modified_julian_date[0..sigma_delta_x.len()]),
                        sigma_delta_x,
                    )?,
                );

                let delta_y: Vec<f64> = records.delta_y.clone().into_iter().flatten().collect();
                interpolators.insert(
                    EopFields::DeltaY,
                    Akima::new(Vec::from(&modified_julian_date[0..delta_y.len()]), delta_y)?,
                );

                let sigma_delta_y: Vec<f64> = records
                    .sigma_delta_y
                    .clone()
                    .into_iter()
                    .flatten()
                    .collect();
                interpolators.insert(
                    EopFields::SigmaDeltaY,
                    Akima::new(
                        Vec::from(&modified_julian_date[0..sigma_delta_y.len()]),
                        sigma_delta_y,
                    )?,
                );
            }
        }

        Ok(Self {
            convention,
            interpolators,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::iter::zip;
    use std::path::Path;

    use float_eq::assert_float_eq;

    use super::*;

    const TEST_DATA_DIR: &str = "../../data";

    /*
     * This is the Langrangian interpolation routine from
     * ftp://hpiers.obspm.fr/iers/models/interp.f
     */
    fn lagint(x: &[i32], y: &[f64], xint: f64) -> f64 {
        let n = x.len();

        let mut yout = 0.0;
        let x0 = f64::from(x[0]);
        let xn = f64::from(x[x.len() - 1]);
        let mut k = if xint <= x0 {
            0
        } else if xint >= xn {
            n
        } else {
            x.binary_search(&(xint.trunc() as i32)).unwrap()
        };
        if k < 1 {
            k = 1
        }
        if k > n - 3 {
            k = n - 3
        }

        for m in k - 1..=k + 2 {
            let mut term = y[m];
            for j in k - 1..=k + 2 {
                if m != j {
                    let xj = f64::from(x[j]);
                    let xm = f64::from(x[m]);
                    term = term * (xint - xj) / (xm - xj);
                }
            }
            yout += term
        }

        yout
    }

    #[test]
    fn test_lagint() {
        let x = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let y = vec![0.0, 2.0, 1.0, 3.0, 2.0, 6.0, 5.5, 5.5, 2.7, 5.1, 3.0];

        let xi = vec![
            0.0, 0.5, 1., 1.5, 2.5, 3.5, 4.5, 5.1, 6.5, 7.2, 8.6, 9.9, 10.,
        ];
        let yi = vec![
            0.0,
            1.75,
            2.0,
            1.5,
            2.0,
            2.375,
            3.96875,
            6.07,
            5.64375,
            4.908,
            4.136799999999999,
            3.6889499999999975,
            3.0,
        ];

        for (xi, yi_exp) in zip(xi, yi) {
            let yi_act = lagint(&x, &y, xi);
            assert_float_eq!(yi_act, yi_exp, rel <= 1e-6);
        }
    }

    #[test]
    fn test_csv() {
        let finals1980 = Path::new(TEST_DATA_DIR).join("finals.all.csv");
        let finals2000a = Path::new(TEST_DATA_DIR).join("finals2000a.all.csv");
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
    fn test_eop() {
        let finals1980 = Path::new(TEST_DATA_DIR).join("finals.all.csv");
        let finals2000a = Path::new(TEST_DATA_DIR).join("finals2000a.all.csv");

        let eop1980 = EarthOrientation::from_path(finals1980).expect("file reading should work");
        assert_eq!(eop1980.convention(), EopConvention::Iau1980);
        assert!(eop1980.interpolators.contains_key(&EopFields::Xpole));
        assert!(eop1980.interpolators.contains_key(&EopFields::SigmaXpole));
        assert!(eop1980.interpolators.contains_key(&EopFields::Ypole));
        assert!(eop1980.interpolators.contains_key(&EopFields::SigmaYpole));
        assert!(eop1980.interpolators.contains_key(&EopFields::DeltaUt1Utc));
        assert!(eop1980
            .interpolators
            .contains_key(&EopFields::SigmaDeltaUt1Utc));
        assert!(eop1980.interpolators.contains_key(&EopFields::Lod));
        assert!(eop1980.interpolators.contains_key(&EopFields::SigmaLod));
        assert!(eop1980.interpolators.contains_key(&EopFields::DeltaPsi));
        assert!(eop1980
            .interpolators
            .contains_key(&EopFields::SigmaDeltaPsi));
        assert!(eop1980.interpolators.contains_key(&EopFields::DeltaEpsilon));
        assert!(eop1980
            .interpolators
            .contains_key(&EopFields::SigmaDeltaEpsilon));
        assert!(!eop1980.interpolators.contains_key(&EopFields::DeltaX));
        assert!(!eop1980.interpolators.contains_key(&EopFields::SigmaDeltaX));
        assert!(!eop1980.interpolators.contains_key(&EopFields::DeltaY));
        assert!(!eop1980.interpolators.contains_key(&EopFields::SigmaDeltaY));

        let eop2000a = EarthOrientation::from_path(finals2000a).expect("file reading should work");
        assert_eq!(eop2000a.convention(), EopConvention::Iau2000a);
        assert!(eop2000a.interpolators.contains_key(&EopFields::Xpole));
        assert!(eop2000a.interpolators.contains_key(&EopFields::SigmaXpole));
        assert!(eop2000a.interpolators.contains_key(&EopFields::Ypole));
        assert!(eop2000a.interpolators.contains_key(&EopFields::SigmaYpole));
        assert!(eop2000a.interpolators.contains_key(&EopFields::DeltaUt1Utc));
        assert!(eop2000a
            .interpolators
            .contains_key(&EopFields::SigmaDeltaUt1Utc));
        assert!(eop2000a.interpolators.contains_key(&EopFields::Lod));
        assert!(eop2000a.interpolators.contains_key(&EopFields::SigmaLod));
        assert!(!eop2000a.interpolators.contains_key(&EopFields::DeltaPsi));
        assert!(!eop2000a
            .interpolators
            .contains_key(&EopFields::SigmaDeltaPsi));
        assert!(!eop2000a
            .interpolators
            .contains_key(&EopFields::DeltaEpsilon));
        assert!(!eop2000a
            .interpolators
            .contains_key(&EopFields::SigmaDeltaEpsilon));
        assert!(eop2000a.interpolators.contains_key(&EopFields::DeltaX));
        assert!(eop2000a.interpolators.contains_key(&EopFields::SigmaDeltaX));
        assert!(eop2000a.interpolators.contains_key(&EopFields::DeltaY));
        assert!(eop2000a.interpolators.contains_key(&EopFields::SigmaDeltaY));

        for field in [
            EopFields::Xpole,
            EopFields::SigmaXpole,
            EopFields::Ypole,
            EopFields::SigmaYpole,
            // EopFields::DeltaUt1Utc,
            // EopFields::SigmaDeltaUt1Utc,
            EopFields::Lod,
            EopFields::SigmaLod,
            EopFields::DeltaPsi,
            EopFields::SigmaDeltaPsi,
            EopFields::DeltaEpsilon,
            EopFields::SigmaDeltaEpsilon,
            EopFields::DeltaX,
            EopFields::SigmaDeltaX,
            EopFields::DeltaY,
            EopFields::SigmaDeltaY,
        ] {
            let eop = match field {
                (EopFields::Xpole
                | EopFields::SigmaXpole
                | EopFields::Ypole
                | EopFields::SigmaYpole
                | EopFields::DeltaUt1Utc
                | EopFields::SigmaDeltaUt1Utc
                | EopFields::Lod
                | EopFields::SigmaLod
                | EopFields::DeltaPsi
                | EopFields::SigmaDeltaPsi
                | EopFields::DeltaEpsilon
                | EopFields::SigmaDeltaEpsilon) => &eop1980,
                (EopFields::DeltaX
                | EopFields::SigmaDeltaX
                | EopFields::DeltaY
                | EopFields::SigmaDeltaY) => &eop2000a,
            };
            let interpolator = &eop.interpolators[&field];
            let mjd0 = interpolator.x[0];
            let mjd1 = interpolator.x[interpolator.x.len() - 1];
            for mjd in mjd0..=mjd1 {
                let mjd = f64::from(mjd) + 0.33;
                let yi_exp = lagint(&interpolator.x, &interpolator.y, mjd);
                let yi_act = interpolator.interpolate(mjd);
                dbg!(field);
                dbg!(mjd);
                assert_float_eq!(yi_act, yi_exp, abs <= 1e-4);
            }
        }
    }
}
