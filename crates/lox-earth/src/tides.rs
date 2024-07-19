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
use lox_math::constants::f64::time::{DAYS_PER_JULIAN_CENTURY, MJD_J2000};
use lox_math::math::arcsec_to_rad_two_pi;
use lox_math::types::julian_dates::ModifiedJulianDate;
use lox_math::types::units::{Arcseconds, Microarcseconds, Radians, Seconds};

use crate::tides::constants::{LUNI_SOLAR_TIDAL_TERMS, OCEANIC_TIDAL_TERMS};

mod constants;

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
    x: Vec<Arcseconds>,
    /// y polar motion.
    y: Vec<Arcseconds>,
    /// UT1-UTC.
    t: Vec<Seconds>,
    /// Epochs of the data.
    epochs: Vec<ModifiedJulianDate>,
    /// Epoch of the interpolated data.
    target_epoch: ModifiedJulianDate,
}

impl Arguments {
    pub fn new(
        x: Vec<Arcseconds>,
        y: Vec<Arcseconds>,
        t: Vec<Seconds>,
        epochs: Vec<ModifiedJulianDate>,
        target_epoch: ModifiedJulianDate,
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
    x: Arcseconds,
    y: Arcseconds,
    d_ut1_utc: ModifiedJulianDate,
}

/// χ (GMST + π) followed by Delaunay arguments l, l', F, D, Ω.
type TidalArgs = [Arcseconds; 6];

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
    x: Arcseconds,
    y: Arcseconds,
    t: Seconds,
}

struct OceanicTidalTerm {
    /// Coefficients of fundamental arguments χ (GMST + π), l, l', F, D, Ω
    coefficients: [i8; 6],
    x_sin: Microarcseconds,
    x_cos: Microarcseconds,
    y_sin: Microarcseconds,
    y_cos: Microarcseconds,
    t_sin: Microarcseconds,
    t_cos: Microarcseconds,
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
            let coeff = term.coefficients[i] as f64;
            agg = arg.mul_add(coeff, agg);
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
    x: Arcseconds,
    y: Arcseconds,
}

struct LuniSolarTidalTerm {
    /// Coefficients of fundamental arguments χ (GMST + π), l, l', F, D, Ω
    coefficients: [i8; 6],
    x_sin: Microarcseconds,
    x_cos: Microarcseconds,
    y_sin: Microarcseconds,
    y_cos: Microarcseconds,
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

fn julian_centuries_since_j2000(mjd: ModifiedJulianDate) -> f64 {
    (mjd - MJD_J2000) / DAYS_PER_JULIAN_CENTURY
}

/// GMST + π.
fn chi(julian_centuries_since_j2000: f64) -> Radians {
    let mut arcsec = fast_polynomial::poly_array(
        julian_centuries_since_j2000,
        &[
            67310.54841,
            876600.0f64.mul_add(3600.0, 8640184.812866),
            0.093104,
            -6.2e-6,
        ],
    );
    arcsec = arcsec.mul_add(15.0, 648000.0);
    arcsec_to_rad_two_pi(arcsec)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use rstest::{fixture, rstest};

    use lox_io::iers::EarthOrientationParams;

    use super::*;

    #[rstest]
    #[case::vec_sizes_match(vec![], vec![], vec![], vec![], 0.0, Ok(Arguments::default()))]
    #[case::x_size_mismatch(vec![0.0], vec![], vec![], vec![], 0.0, Err(ArgumentSizeMismatchError { nx: 1, ny: 0, nt: 0, nepochs: 0 }))]
    #[case::y_size_mismatch(vec![], vec![0.0], vec![], vec![], 0.0, Err(ArgumentSizeMismatchError { nx: 0, ny: 1, nt: 0, nepochs: 0 }))]
    #[case::t_size_mismatch(vec![], vec![], vec![0.0], vec![], 0.0, Err(ArgumentSizeMismatchError { nx: 0, ny: 0, nt: 1, nepochs: 0 }))]
    #[case::epochs_size_mismatch(vec![], vec![], vec![], vec![0.0], 0.0, Err(ArgumentSizeMismatchError { nx: 0, ny: 0, nt: 0, nepochs: 1 }))]
    fn test_arguments_new(
        #[case] x: Vec<Arcseconds>,
        #[case] y: Vec<Arcseconds>,
        #[case] t: Vec<Seconds>,
        #[case] epochs: Vec<ModifiedJulianDate>,
        #[case] target_epoch: ModifiedJulianDate,
        #[case] expected: Result<Arguments, ArgumentSizeMismatchError>,
    ) {
        let actual = Arguments::new(x, y, t, epochs, target_epoch);
        assert_eq!(expected, actual);
    }

    const FINALS2000A_PATH: &str = "../../data/finals2000A.all.csv";

    #[fixture]
    fn eop_data() -> EarthOrientationParams {
        let fixture_path = Path::new(FINALS2000A_PATH);
        EarthOrientationParams::parse_finals_csv(fixture_path).unwrap_or_else(|err| {
            panic!(
                "failed to parse test fixture at {}: {}",
                fixture_path.to_str().unwrap(),
                err,
            )
        })
    }
}
