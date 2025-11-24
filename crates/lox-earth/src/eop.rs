// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use csv::{ByteRecord, ReaderBuilder};
use lox_core::f64::consts::{SECONDS_BETWEEN_MJD_AND_J2000, SECONDS_PER_DAY};
use lox_io::spice::lsk::LeapSecondsKernel;
use lox_math::series::{Series, SeriesError};
use lox_time::{
    deltas::TimeDelta,
    julian_dates::JulianDate,
    utc::{
        Utc, UtcError,
        leap_seconds::{DefaultLeapSecondsProvider, LeapSecondsProvider},
        transformations::TryToUtc,
    },
};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
struct EopRecord {
    #[serde(rename = "MJD")]
    modified_julian_date: f64,
    #[serde(rename = "Year")]
    year: i64,
    #[serde(rename = "Month")]
    month: u8,
    #[serde(rename = "Day")]
    day: u8,
    x_pole: Option<f64>,
    y_pole: Option<f64>,
    #[serde(rename = "UT1-UTC")]
    delta_ut1_utc: Option<f64>,
    #[serde(rename = "dPsi")]
    dpsi: Option<f64>,
    #[serde(rename = "dEpsilon")]
    deps: Option<f64>,
    #[serde(rename = "dX")]
    dx: Option<f64>,
    #[serde(rename = "dY")]
    dy: Option<f64>,
}

impl EopRecord {
    fn merge(mut self, other: &Self) -> Self {
        self.dpsi = self.dpsi.or(other.dpsi);
        self.deps = self.deps.or(other.deps);
        self.dx = self.dx.or(other.dx);
        self.dy = self.dy.or(other.dy);
        self
    }
}

#[derive(Debug, Error)]
pub enum EopParserError {
    #[error("{0}")]
    Csv(String),
    #[error("either a 'finals.all.csv' or a 'finals2000A.all.csv' file needs to be provided")]
    NoFiles,
    #[error("could not find corresponding leap second for {0}")]
    LeapSecond(Utc),
    #[error("mismatched dimensions for columns '{0} (n={1})' and '{2} (n={3})'")]
    DimensionMismatch(String, usize, String, usize),
    #[error(transparent)]
    Utc(#[from] UtcError),
    #[error(transparent)]
    Series(#[from] SeriesError),
}

impl From<csv::Error> for EopParserError {
    fn from(err: csv::Error) -> Self {
        EopParserError::Csv(err.to_string())
    }
}

#[derive(Default)]
pub struct EopParser {
    paths: (Option<PathBuf>, Option<PathBuf>),
    lsk: Option<LeapSecondsKernel>,
}

impl EopParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_path(mut self, path: impl AsRef<Path>) -> Self {
        self.paths = (Some(path.as_ref().to_owned()), None);
        self
    }

    pub fn from_paths(mut self, path1: impl AsRef<Path>, path2: impl AsRef<Path>) -> Self {
        self.paths = (
            Some(path1.as_ref().to_owned()),
            Some(path2.as_ref().to_owned()),
        );
        self
    }

    pub fn with_leap_seconds_kernel(mut self, lsp: LeapSecondsKernel) -> Self {
        self.lsk = Some(lsp);
        self
    }

    pub fn parse(self) -> Result<EopProvider, EopParserError> {
        let n = if let Some(iau1980) = self.paths.0.as_ref() {
            let mut reader = ReaderBuilder::new().delimiter(b';').from_path(iau1980)?;
            reader.records().count()
        } else {
            return Err(EopParserError::NoFiles);
        };

        let mut j2000: Vec<f64> = Vec::with_capacity(n);
        let mut delta_ut1_tai: Vec<f64> = Vec::with_capacity(n);
        let mut x_pole: Vec<f64> = Vec::with_capacity(n);
        let mut y_pole: Vec<f64> = Vec::with_capacity(n);
        let mut dpsi: Vec<f64> = Vec::with_capacity(n);
        let mut deps: Vec<f64> = Vec::with_capacity(n);
        let mut dx: Vec<f64> = Vec::with_capacity(n);
        let mut dy: Vec<f64> = Vec::with_capacity(n);

        // The EOP from the IERS are split into two different files based on the two IAU conventions for some reason.
        // Both files have the same header and identical data except that the columns for the IAU1980 parameters are
        // only populated in `finals.all.csv` and the columns for the IAU2000 parameters are only populated in
        // `finals2000A.all.csv`.
        // We do not want to force the user to provide both files if they do not need them. At the same time, we want
        // to parse both files in one pass if they are present to avoid looping over tens of thousands of lines
        // multiple time.
        // I have not been able to fulfil both requirements and make the compiler happy without pretending that there
        // are always two files. In case the user provided two files, we parse both files and merge the records.
        // If we have only one file, we still parse it twice and record merging is a no-op.
        let mut rdr1 = ReaderBuilder::new()
            .delimiter(b';')
            .from_path(self.paths.0.as_ref().unwrap())?;
        let mut rdr2 = ReaderBuilder::new()
            .delimiter(b';')
            .from_path(self.paths.1.as_ref().or(self.paths.0.as_ref()).unwrap())?;

        let mut raw1 = ByteRecord::new();
        let mut raw2 = ByteRecord::new();
        let headers = rdr1.byte_headers()?.clone();

        while rdr1.read_byte_record(&mut raw1)? && rdr2.read_byte_record(&mut raw2)? {
            let r1: EopRecord = raw1.deserialize(Some(&headers))?;
            let r2: EopRecord = raw2.deserialize(Some(&headers))?;
            let r = r1.merge(&r2);

            j2000.push(r.modified_julian_date * SECONDS_PER_DAY - SECONDS_BETWEEN_MJD_AND_J2000);

            if let Some(delta_ut1_utc) = r.delta_ut1_utc {
                let utc = Utc::builder().with_ymd(r.year, r.month, r.day).build()?;
                let delta_tai_utc = self
                    .lsk
                    .as_ref()
                    .map(|lsp| lsp.delta_utc_tai(utc))
                    .or_else(|| Some(DefaultLeapSecondsProvider.delta_utc_tai(utc)))
                    .flatten()
                    .ok_or(EopParserError::LeapSecond(utc))?;
                delta_ut1_tai.push(delta_ut1_utc + delta_tai_utc.as_seconds_f64())
            }

            if let (Some(xp), Some(yp)) = (r.x_pole, r.y_pole) {
                x_pole.push(xp);
                y_pole.push(yp);
            }

            if let (Some(d_psi), Some(d_eps)) = (r.dpsi, r.deps) {
                dpsi.push(d_psi);
                deps.push(d_eps);
            }

            if let (Some(d_x), Some(d_y)) = (r.dx, r.dy) {
                dx.push(d_x);
                dy.push(d_y);
            }
        }

        let n = x_pole.len();
        let npn = dpsi.len().max(dx.len());

        let index = Index(Arc::new(j2000[0..n].to_vec()));
        let np_index = Index(Arc::new(j2000[0..npn].to_vec()));

        Ok(EopProvider {
            polar_motion: (
                Series::try_cubic_spline(index.clone(), x_pole)?,
                Series::try_cubic_spline(index.clone(), y_pole)?,
            ),
            delta_ut1_tai: Series::try_cubic_spline(index.clone(), delta_ut1_tai)?,
            nut_prec: NutPrecCorrections {
                iau1980: if !dpsi.is_empty() {
                    Some((
                        Series::try_cubic_spline(np_index.clone(), dpsi)?,
                        Series::try_cubic_spline(np_index.clone(), deps)?,
                    ))
                } else {
                    None
                },
                iau2000: if !dx.is_empty() {
                    Some((
                        Series::try_cubic_spline(np_index.clone(), dx)?,
                        Series::try_cubic_spline(np_index.clone(), dy)?,
                    ))
                } else {
                    None
                },
            },
            lsk: self.lsk.clone(),
        })
    }
}

#[derive(Debug, Default, Error)]
pub enum EopProviderError {
    #[error(transparent)]
    Utc(#[from] UtcError),
    #[error("value was extrapolated")]
    ExtrapolatedValue(f64),
    #[error("values were extrapolated")]
    ExtrapolatedValues(f64, f64),
    #[error("no 'finals.all.csv' file was loaded")]
    MissingIau1980,
    #[error("no 'finals2000A.all.csv' file was loaded")]
    MissingIau2000,
    #[default]
    #[error("unreachable")]
    Never,
}

#[derive(Clone, Debug)]
struct Index(Arc<Vec<f64>>);

impl AsRef<[f64]> for Index {
    fn as_ref(&self) -> &[f64] {
        self.0.as_ref()
    }
}
type TSeries = Series<Index, Vec<f64>>;

#[derive(Debug, Clone)]
struct NutPrecCorrections {
    iau1980: Option<(TSeries, TSeries)>,
    iau2000: Option<(TSeries, TSeries)>,
}

#[derive(Debug, Clone)]
pub struct EopProvider {
    polar_motion: (TSeries, TSeries),
    delta_ut1_tai: TSeries,
    nut_prec: NutPrecCorrections,
    lsk: Option<LeapSecondsKernel>,
}

impl EopProvider {
    pub fn polar_motion<T: TryToUtc>(&self, t: T) -> Result<(f64, f64), EopProviderError> {
        let t = t.try_to_utc()?.seconds_since_j2000();
        let px = self.polar_motion.0.interpolate(t);
        let py = self.polar_motion.1.interpolate(t);
        let (min, _) = self.polar_motion.0.first();
        let (max, _) = self.polar_motion.0.last();
        if t < min || t > max {
            return Err(EopProviderError::ExtrapolatedValues(px, py));
        }
        Ok((px, py))
    }

    pub fn nutation_precession_iau1980<T: TryToUtc>(
        &self,
        t: T,
    ) -> Result<(f64, f64), EopProviderError> {
        let Some(nut_prec) = &self.nut_prec.iau1980 else {
            return Err(EopProviderError::MissingIau1980);
        };
        let t = t.try_to_utc()?.seconds_since_j2000();
        let dpsi = nut_prec.0.interpolate(t);
        let deps = nut_prec.1.interpolate(t);
        let (min, _) = nut_prec.0.first();
        let (max, _) = nut_prec.0.last();
        if t < min || t > max {
            return Err(EopProviderError::ExtrapolatedValues(dpsi, deps));
        }
        Ok((dpsi, deps))
    }

    pub fn nutation_precession_iau2000<T: TryToUtc>(
        &self,
        t: T,
    ) -> Result<(f64, f64), EopProviderError> {
        let Some(nut_prec) = &self.nut_prec.iau2000 else {
            return Err(EopProviderError::MissingIau2000);
        };
        let t = t.try_to_utc()?.seconds_since_j2000();
        let dx = nut_prec.0.interpolate(t);
        let dy = nut_prec.1.interpolate(t);
        let (min, _) = nut_prec.0.first();
        let (max, _) = nut_prec.0.last();
        if t < min || t > max {
            return Err(EopProviderError::ExtrapolatedValues(dx, dy));
        }
        Ok((dx, dy))
    }

    pub fn delta_ut1_tai(&self, tai: TimeDelta) -> Result<TimeDelta, EopProviderError> {
        let seconds = tai.seconds_since_j2000();
        let (t0, _) = self.delta_ut1_tai.first();
        let (tn, _) = self.delta_ut1_tai.last();
        let val = self.delta_ut1_tai.interpolate(seconds);
        if seconds < t0 || seconds > tn {
            return Err(EopProviderError::ExtrapolatedValue(val));
        }
        Ok(TimeDelta::from_seconds_f64(val))
    }

    pub fn delta_tai_ut1(&self, ut1: TimeDelta) -> Result<TimeDelta, EopProviderError> {
        let seconds = ut1.seconds_since_j2000();
        let (t0, _) = self.delta_ut1_tai.first();
        let (tn, _) = self.delta_ut1_tai.last();
        // Use the UT1 offset as an initial guess even though the table is based on TAI
        let mut val = self.delta_ut1_tai.interpolate(seconds);
        // Interpolate again with the adjusted offsets
        for _ in 0..2 {
            val = self.delta_ut1_tai.interpolate(seconds - val);
        }
        if seconds < t0 || seconds > tn {
            return Err(EopProviderError::ExtrapolatedValue(val));
        }
        Ok(-TimeDelta::from_seconds_f64(val))
    }

    pub fn get_lsk(&self) -> Option<&LeapSecondsKernel> {
        self.lsk.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::data_file;
    use lox_time::{Time, time_scales::Tai};

    use super::*;

    #[test]
    #[should_panic(expected = "NoFiles")]
    fn test_eop_parser_no_files() {
        EopParser::new().parse().unwrap();
    }

    #[test]
    #[should_panic(expected = "MissingIau2000")]
    fn test_eop_provider_missing_iau2000() {
        let t: Time<Tai> = Time::default();
        let eop = EopParser::new()
            .from_path(data_file("iers/finals.all.csv"))
            .parse()
            .unwrap();
        eop.nutation_precession_iau2000(t).unwrap();
    }

    #[test]
    #[should_panic(expected = "MissingIau1980")]
    fn test_eop_provider_missing_iau1980() {
        let t: Time<Tai> = Time::default();
        let eop = EopParser::new()
            .from_path(data_file("iers/finals2000A.all.csv"))
            .parse()
            .unwrap();
        eop.nutation_precession_iau1980(t).unwrap();
    }
}
