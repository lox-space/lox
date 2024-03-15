/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::lagrange::eop::constants::{LUNI_SOLAR_TIDAL_TERMS, MJD_J2000, OCEANIC_TIDAL_TERMS};
use lox_bodies::fundamental::iers03::mean_moon_sun_elongation_iers03;
use lox_bodies::{Moon, Sun};
use lox_time::constants::f64::DAYS_PER_JULIAN_CENTURY;
use lox_utils::math::arcsec_to_rad_two_pi;
use lox_utils::types::{Arcsec, Radians, Seconds};
use std::f64::consts::TAU;
use thiserror::Error;

mod constants;

type Mjd = f64;

#[derive(Clone, Copy, Debug, Error, PartialEq)]
#[error("sizes of `x`, `y`, `t` and `epochs` must match, but were x: {nx}, y: {ny}, t: {nt}, epochs: {nepochs}")]
pub struct ArgumentSizeMismatchError {
    nx: usize,
    ny: usize,
    nt: usize,
    nepochs: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Arguments {
    /// x polar motion.
    x: Vec<Arcsec>,
    /// y polar motion.
    y: Vec<Arcsec>,
    /// UT1-UTC.
    t: Vec<Seconds>,
    /// Epochs of the data.
    epochs: Vec<Mjd>,
    /// Epoch of the interpolated data.
    target_epoch: Mjd,
}

impl Arguments {
    pub fn new(
        x: Vec<Arcsec>,
        y: Vec<Arcsec>,
        t: Vec<Seconds>,
        epochs: Vec<Mjd>,
        target_epoch: Mjd,
    ) -> Result<Arguments, ArgumentSizeMismatchError> {
        if x.len() != y.len() || x.len() != t.len() || x.len() != epochs.len() {
            return Err(ArgumentSizeMismatchError {
                nx: x.len(),
                ny: y.len(),
                nt: t.len(),
                nepochs: epochs.len(),
            });
        }

        Ok(Arguments {
            x,
            y,
            t,
            epochs,
            target_epoch,
        })
    }
}

/// The result of the Lagrangian interpolation of polar motion and UT1-UTC.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Interpolation {
    x: Arcsec,
    y: Arcsec,
    d_ut1_utc: Mjd,
}

/// Perform Lagrangian interpolation of Earth Orientation Parameters (EOP), returning polar x- and
/// y- values and UT1-UTC at the target epoch. The result is corrected for oceanic and luni-solar
/// tidal effects.
pub fn interpolate(args: Arguments) -> Interpolation {
    let x = crate::lagrange::interpolate(&args.epochs, &args.x, args.target_epoch);
    let y = crate::lagrange::interpolate(&args.epochs, &args.y, args.target_epoch);
    let t = crate::lagrange::interpolate(&args.epochs, &args.t, args.target_epoch);
    let tidal_args = tidal_args(julian_centuries_since_j2000(args.target_epoch));
    let tidal_correction = oceanic_tidal_correction(&tidal_args);
    let lunisolar_correction = luni_solar_tidal_correction(&tidal_args);
    Interpolation {
        x: x + tidal_correction.x + lunisolar_correction.x,
        y: y + tidal_correction.y + lunisolar_correction.y,
        d_ut1_utc: t + tidal_correction.t,
    }
}

/// χ (GMST + π) followed by Delaunay arguments l, l', F, D, Ω.
type TidalArgs = [Arcsec; 6];

fn tidal_args(julian_centuries_since_j2000: f64) -> TidalArgs {
    [
        chi(julian_centuries_since_j2000),
        Moon.mean_anomaly_iers03(julian_centuries_since_j2000),
        Sun.mean_anomaly_iers03(julian_centuries_since_j2000),
        Moon.mean_longitude_minus_ascending_node_mean_longitude_iers03(
            julian_centuries_since_j2000,
        ),
        mean_moon_sun_elongation_iers03(julian_centuries_since_j2000),
        Moon.ascending_node_mean_longitude_iers03(julian_centuries_since_j2000),
    ]
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct OceanicTidalCorrection {
    x: Arcsec,
    y: Arcsec,
    t: Seconds,
}

type MicroArcsec = f64;

struct OceanicTidalTerm {
    /// Coefficients of fundamental arguments χ (GMST + π), l, l', F, D, Ω
    coefficients: [i8; 6],
    x_sin: MicroArcsec,
    x_cos: MicroArcsec,
    y_sin: MicroArcsec,
    y_cos: MicroArcsec,
    t_sin: MicroArcsec,
    t_cos: MicroArcsec,
}

type RadiansPerDay = f64;

/// Returns the diurnal/subdiurnal oceanic tidal effects on polar motion and UT1-UTC. Based on
/// Bizouard (2002), Gambis (1997) and Eanes (1997).
fn oceanic_tidal_correction(tidal_args: &TidalArgs) -> OceanicTidalCorrection {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut t = 0.0;

    for term in OCEANIC_TIDAL_TERMS {
        let mut agg = 0.0;
        for (i, arg) in tidal_args.iter().enumerate() {
            agg += term.coefficients[i] as f64 * arg;
        }
        agg %= TAU;

        let (sin_agg, cos_agg) = agg.sin_cos();
        x += term.x_sin * sin_agg + term.x_cos * cos_agg;
        y += term.y_sin * sin_agg + term.y_cos * cos_agg;
        t += term.t_sin * sin_agg + term.t_cos * cos_agg;
    }

    OceanicTidalCorrection {
        x: x * 1e-6,
        y: y * 1e-6,
        t: t * 1e-6,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct LuniSolarTidalCorrection {
    x: Arcsec,
    y: Arcsec,
}

struct LuniSolarTidalTerm {
    /// Coefficients of fundamental arguments χ (GMST + π), l, l', F, D, Ω
    coefficients: [i8; 6],
    x_sin: MicroArcsec,
    x_cos: MicroArcsec,
    y_sin: MicroArcsec,
    y_cos: MicroArcsec,
}

/// Returns the luni-solar correction to polar motion.
fn luni_solar_tidal_correction(tidal_args: &TidalArgs) -> LuniSolarTidalCorrection {
    let mut x = 0.0;
    let mut y = 0.0;

    for term in LUNI_SOLAR_TIDAL_TERMS {
        let mut agg = 0.0;
        for (i, arg) in tidal_args.iter().enumerate() {
            agg += term.coefficients[i] as f64 * arg;
        }
        agg %= TAU;

        let (sin_agg, cos_agg) = agg.sin_cos();
        x += term.x_sin * sin_agg + term.x_cos * cos_agg;
        y += term.y_sin * sin_agg + term.y_cos * cos_agg;
    }

    LuniSolarTidalCorrection {
        x: x * 1e-6,
        y: y * 1e-6,
    }
}

fn julian_centuries_since_j2000(mjd: Mjd) -> f64 {
    (mjd - MJD_J2000) / DAYS_PER_JULIAN_CENTURY
}

/// GMST + π.
fn chi(julian_centuries_since_j2000: f64) -> Radians {
    let arcsec = fast_polynomial::poly_array(
        julian_centuries_since_j2000,
        &[
            67310.54841,
            876600.0 * 3600.0 + 8640184.812866,
            0.093104,
            -6.2e-6,
        ],
    ) * 15.0
        + 648000.0;
    arcsec_to_rad_two_pi(arcsec)
}

#[inline]
fn arcsec_to_radians_per_day(arcsec: Arcsec) -> RadiansPerDay {
    arcsec_to_rad_two_pi(arcsec) / DAYS_PER_JULIAN_CENTURY
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use float_eq::assert_float_eq;
    use num_traits::ToPrimitive;
    use rstest::{fixture, rstest};

    use crate::{read_records, Records};

    use super::*;

    #[rstest]
    #[case::vec_sizes_match(vec![], vec![], vec![], vec![], 0.0, Ok(Arguments::default()))]
    #[case::x_size_mismatch(vec![0.0], vec![], vec![], vec![], 0.0, Err(ArgumentSizeMismatchError { nx: 1, ny: 0, nt: 0, nepochs: 0 }))]
    #[case::y_size_mismatch(vec![], vec![0.0], vec![], vec![], 0.0, Err(ArgumentSizeMismatchError { nx: 0, ny: 1, nt: 0, nepochs: 0 }))]
    #[case::t_size_mismatch(vec![], vec![], vec![0.0], vec![], 0.0, Err(ArgumentSizeMismatchError { nx: 0, ny: 0, nt: 1, nepochs: 0 }))]
    #[case::epochs_size_mismatch(vec![], vec![], vec![], vec![0.0], 0.0, Err(ArgumentSizeMismatchError { nx: 0, ny: 0, nt: 0, nepochs: 1 }))]
    fn test_arguments_new(
        #[case] x: Vec<Arcsec>,
        #[case] y: Vec<Arcsec>,
        #[case] t: Vec<Seconds>,
        #[case] epochs: Vec<Mjd>,
        #[case] target_epoch: Mjd,
        #[case] expected: Result<Arguments, ArgumentSizeMismatchError>,
    ) {
        let actual = Arguments::new(x, y, t, epochs, target_epoch);
        assert_eq!(expected, actual);
    }

    const FINALS2000A_PATH: &str = "../../data/finals2000A.all.csv";

    struct UnwrappedEOPData {
        x_pole: Vec<Arcsec>,
        y_pole: Vec<Arcsec>,
        delta_ut1_utc: Vec<Seconds>,
        mjd: Vec<Mjd>,
    }

    #[fixture]
    fn eop_data() -> UnwrappedEOPData {
        let fixture_path = Path::new(FINALS2000A_PATH);

        let records: Records = read_records(FINALS2000A_PATH)
            .unwrap_or_else(|err| {
                panic!(
                    "failed to read test fixture at {}: {}",
                    fixture_path.to_str().unwrap(),
                    err,
                )
            })
            .into();

        let x_pole: Vec<f64> = records
            .x_pole
            .iter()
            .map(|opt| opt.expect("x_pole value should not be None"))
            .collect();
        let y_pole = records
            .y_pole
            .iter()
            .map(|opt| opt.expect("y_pole value should not be None"))
            .collect();
        let delta_ut1_utc = records
            .delta_ut1_utc
            .iter()
            .map(|opt| opt.expect("delta_ut1_utc value should not be None"))
            .collect();
        let mjd = records
            .modified_julian_date
            .iter()
            .map(|date| {
                date.to_f64().unwrap_or_else(|| {
                    panic!("fixture MJD `{}` could not be represented as an f64", date)
                })
            })
            .collect();

        UnwrappedEOPData {
            x_pole,
            y_pole,
            delta_ut1_utc,
            mjd,
        }
    }

    #[rstest]
    #[case::mjd_j2000(MJD_J2000, Interpolation {
    x: 4.325128997437056e-2,
    y: 0.3779536211567663,
    d_ut1_utc: 0.35498904611828275,
    })]
    #[case::mjd_0(0.0, Interpolation {
    x: 12072321.700398155,
    y: -24142704.67775462,
    d_ut1_utc: 778638165.7968734,
    })]
    fn test_lagrangian_interpolate(
        eop_data: UnwrappedEOPData,
        #[case] target_epoch: Mjd,
        #[case] expected: Interpolation,
    ) -> Result<(), ArgumentSizeMismatchError> {
        let args = Arguments::new(
            eop_data.x_pole,
            eop_data.y_pole,
            eop_data.delta_ut1_utc,
            eop_data.mjd,
            target_epoch,
        )?;
        let result = interpolate(args);

        assert_float_eq!(expected.x, result.x, rel <= 1e-9);
        assert_float_eq!(expected.y, result.y, rel <= 1e-9);
        assert_float_eq!(expected.d_ut1_utc, result.d_ut1_utc, rel <= 1e-9);

        Ok(())
    }
}
