/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::f64::consts::TAU;

use thiserror::Error;

use lox_bodies::fundamental::iers03::mean_moon_sun_elongation_iers03;
use lox_bodies::{Moon, Sun};
use lox_time::constants::f64::DAYS_PER_JULIAN_CENTURY;
// todo: Impending circular dependency. Need to hoist constants.
use lox_time::constants::julian_dates::MJD_J2000;
use lox_time::intervals::TDBJulianCenturiesSinceJ2000;
use lox_utils::math::arcsec_to_rad_two_pi;
use lox_utils::types::{Arcsec, Radians, Seconds};

use crate::lagrange::constants::{LUNI_SOLAR_TIDAL_TERMS, OCEANIC_TIDAL_TERMS};

mod constants;

type MJD = f64;

/// A polynomial function which may be executed repeatedly for arbitrary values of `x`.
type Polynomial1D = fn(x: f64) -> f64;

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
    epochs: Vec<MJD>,
    /// Epoch of the interpolated data.
    target_epoch: MJD,
}

impl Arguments {
    pub fn new(
        x: Vec<Arcsec>,
        y: Vec<Arcsec>,
        t: Vec<Seconds>,
        epochs: Vec<MJD>,
        target_epoch: MJD,
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

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Interpolation {
    x: Arcsec,
    y: Arcsec,
    t: MJD,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lagrange {
    n: usize,
    /// x polar motion.
    x: Vec<Arcsec>,
    /// y polar motion.
    y: Vec<Arcsec>,
    /// UT1-UTC.
    t: Vec<Seconds>,
    /// Epochs of the data.
    epochs: Vec<MJD>,
    /// Epoch for the interpolated value.
    target_epoch: MJD,
}

impl Lagrange {
    pub fn new(args: Arguments) -> Self {
        Self {
            n: args.x.len(),
            x: args.x,
            y: args.y,
            t: args.t,
            epochs: args.epochs,
            target_epoch: args.target_epoch,
        }
    }

    /// Perform Lagrangian interpolation for `target_epoch`, using the Ray model for diurnal and
    /// subdiurnal tidal variations.
    pub fn interpolate(&self) -> Interpolation {
        let x = interpolate(&self.epochs, &self.x, self.target_epoch);
        let y = interpolate(&self.epochs, &self.y, self.target_epoch);
        let t = interpolate(&self.epochs, &self.t, self.target_epoch);
        let centuries = julian_centuries_since_j2000(self.target_epoch);
        let tidal_args = tidal_args(julian_centuries_since_j2000(self.target_epoch));
        let tidal_correction = oceanic_tidal_correction(centuries, &tidal_args);
        let lunisolar_correction = luni_solar_tidal_correction(&tidal_args);
        Interpolation {
            x: x + tidal_correction.x + lunisolar_correction.x,
            y: y + tidal_correction.y + lunisolar_correction.y,
            t: t + tidal_correction.t,
        }
    }
}

/// Perform Lagrangian interpolation within a set of (x, y) pairs, returning the y-value
/// corresponding to `target_x`
fn interpolate(x: &[f64], y: &[f64], target_x: f64) -> f64 {
    let mut result = 0.0;
    let mut k = 0usize;
    for i in 0..(x.len() - 1) {
        if target_x >= x[i] && target_x < x[i + 1] {
            k = i;
            break;
        }
    }

    if k < 1 {
        k = 1;
    }
    if k > x.len() - 3 {
        k = x.len() - 3;
    }

    for m in (k - 1)..(k + 3) {
        let mut term = y[m];
        for j in (k - 1)..(k + 3) {
            if m != j {
                term *= (target_x - x[j]) / (x[m] - x[j]);
            }
        }
        result += term;
    }

    result
}

/// χ (GMST + π) followed by Delaunay arguments l, l', F, D, Ω.
type TidalArgs = [Arcsec; 6];

fn tidal_args(t: TDBJulianCenturiesSinceJ2000) -> TidalArgs {
    [
        chi(t),
        Moon.mean_anomaly_iers03(t),
        Sun.mean_anomaly_iers03(t),
        Moon.mean_longitude_minus_ascending_node_mean_longitude_iers03(t),
        mean_moon_sun_elongation_iers03(t),
        Moon.ascending_node_mean_longitude_iers03(t),
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
fn oceanic_tidal_correction(
    t: TDBJulianCenturiesSinceJ2000,
    tidal_args: &TidalArgs,
) -> OceanicTidalCorrection {
    // χ (GMST + π), l, l', F, D, Ω
    let tidal_args_dt: [RadiansPerDay; 6] =
        [chi_dt(t), l_dt(t), lp_dt(t), f_dt(t), d_dt(t), omega_dt(t)];

    let mut x = 0.0;
    let mut y = 0.0;
    let mut t = 0.0;

    for term in OCEANIC_TIDAL_TERMS {
        let mut agg = 0.0;
        let mut dt_agg = 0.0;
        for i in 0..6 {
            let coeff = term.coefficients[i] as f64;
            agg += coeff * tidal_args[i];
            dt_agg += coeff * tidal_args_dt[i];
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

fn julian_centuries_since_j2000(mjd: MJD) -> TDBJulianCenturiesSinceJ2000 {
    (mjd - MJD_J2000) / DAYS_PER_JULIAN_CENTURY
}

/// GMST + π.
fn chi(t: TDBJulianCenturiesSinceJ2000) -> Radians {
    let arcsec = fast_polynomial::poly_array(
        t,
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

fn chi_dt(t: TDBJulianCenturiesSinceJ2000) -> RadiansPerDay {
    let arcsec = fast_polynomial::poly_array(
        t,
        &[
            876600.0 * 3600.0 + 8640184.812866,
            2.0 * 0.093104,
            -3.0 * 6.2e-6,
        ],
    ) * 15.0;
    arcsec_to_radians_per_day(arcsec)
}

fn l_dt(t: TDBJulianCenturiesSinceJ2000) -> RadiansPerDay {
    let arcsec = fast_polynomial::poly_array(
        t,
        &[
            1717915923.2178,
            2.0 * 31.8792,
            3.0 * 0.051635,
            -4.0 * 0.00024470,
        ],
    );
    arcsec_to_radians_per_day(arcsec)
}

fn lp_dt(t: TDBJulianCenturiesSinceJ2000) -> RadiansPerDay {
    let arcsec = fast_polynomial::poly_array(
        t,
        &[
            129596581.0481,
            -2.0 * 0.5532,
            -3.0 * 0.000136,
            -4.0 * 0.00001149,
        ],
    );
    arcsec_to_radians_per_day(arcsec)
}

fn f_dt(t: TDBJulianCenturiesSinceJ2000) -> RadiansPerDay {
    let arcsec = fast_polynomial::poly_array(
        t,
        &[
            1739527262.8478,
            -2.0 * 12.7512,
            -3.0 * 0.001037,
            4.0 * 0.00000417,
        ],
    );
    arcsec_to_radians_per_day(arcsec)
}

fn d_dt(t: TDBJulianCenturiesSinceJ2000) -> RadiansPerDay {
    let arcsec = fast_polynomial::poly_array(
        t,
        &[
            1602961601.2090,
            -2.0 * 6.3706,
            3.0 * 0.006593,
            -4.0 * 0.00003169,
        ],
    );
    arcsec_to_radians_per_day(arcsec)
}

fn omega_dt(t: TDBJulianCenturiesSinceJ2000) -> RadiansPerDay {
    let arcsec = fast_polynomial::poly_array(
        t,
        &[
            -6962890.2665,
            2.0 * 7.4722,
            3.0 * 0.007702,
            -4.0 * 0.00005939,
        ],
    );
    arcsec_to_radians_per_day(arcsec)
}

#[inline]
fn arcsec_to_radians_per_day(arcsec: Arcsec) -> RadiansPerDay {
    arcsec_to_rad_two_pi(arcsec) / DAYS_PER_JULIAN_CENTURY
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use std::path::Path;

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
        #[case] epochs: Vec<MJD>,
        #[case] target_epoch: MJD,
        #[case] expected: Result<Arguments, ArgumentSizeMismatchError>,
    ) {
        let actual = Arguments::new(x, y, t, epochs, target_epoch);
        assert_eq!(expected, actual);
    }

    const FINALS2000A_PATH: &str = "tests/fixtures/finals2000A.all.csv";

    struct UnwrappedEOPData {
        x_pole: Vec<Arcsec>,
        y_pole: Vec<Arcsec>,
        delta_ut1_utc: Vec<Seconds>,
        mjd: Vec<MJD>,
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
        t: 0.35498904611828275,
    })]
    #[case::mjd_0(0.0, Interpolation {
        x: 12072321.700398155,
        y: -24142704.67775462,
        t: 778638165.7968734,
    })]
    fn test_lagrangian_interpolate(
        eop_data: UnwrappedEOPData,
        #[case] target_epoch: MJD,
        #[case] expected: Interpolation,
    ) -> Result<(), ArgumentSizeMismatchError> {
        let args = Arguments::new(
            eop_data.x_pole,
            eop_data.y_pole,
            eop_data.delta_ut1_utc,
            eop_data.mjd,
            target_epoch,
        )?;
        let lagrange = Lagrange::new(args);
        let interpolation = lagrange.interpolate();

        assert_float_eq!(expected.x, interpolation.x, rel <= 1e-9);
        assert_float_eq!(expected.y, interpolation.y, rel <= 1e-9);
        assert_float_eq!(expected.t, interpolation.t, rel <= 1e-9);

        Ok(())
    }
}
