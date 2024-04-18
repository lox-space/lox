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
use lox_utils::constants::f64::time::{DAYS_PER_JULIAN_CENTURY, MJD_J2000};
use lox_utils::math::arcsec_to_rad_two_pi;
use lox_utils::types::julian_dates::ModifiedJulianDate;
use lox_utils::types::units::{Arcseconds, Microarcseconds, Radians, Seconds};

use crate::lagrange::eop::constants::{LUNI_SOLAR_TIDAL_TERMS, OCEANIC_TIDAL_TERMS};
use crate::lagrange::WINDOW_SIZE;

mod constants;

#[derive(Clone, Copy, Debug, Error, PartialEq)]
pub enum ArgumentsError {
    #[error("lengths of `x`, `y`, `t` and `epochs` must match, but were x.len()={nx}, y.len()={ny}, t.len()={nt}, epochs.len()={nepochs}")]
    DimensionMismatch {
        nx: usize,
        ny: usize,
        nt: usize,
        nepochs: usize,
    },
    #[error(
        "at least {} datapoints are required for interpolation, but only {0} were provided",
        WINDOW_SIZE
    )]
    TooFewDataPoints(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Arguments<'a> {
    /// x polar motion.
    x: &'a [Arcseconds],
    /// y polar motion.
    y: &'a [Arcseconds],
    /// UT1-UTC.
    t: &'a [Seconds],
    /// Epochs of the data.
    epochs: &'a [ModifiedJulianDate],
    /// Epoch of the interpolated data.
    target_epoch: ModifiedJulianDate,
}

impl<'a> Arguments<'_> {
    pub fn new(
        x: &'a [Arcseconds],
        y: &'a [Arcseconds],
        t: &'a [Seconds],
        epochs: &'a [ModifiedJulianDate],
        target_epoch: ModifiedJulianDate,
    ) -> Result<Arguments<'a>, ArgumentsError> {
        if x.len() != y.len() || x.len() != t.len() || x.len() != epochs.len() {
            return Err(ArgumentsError::DimensionMismatch {
                nx: x.len(),
                ny: y.len(),
                nt: t.len(),
                nepochs: epochs.len(),
            });
        }

        if x.len() < WINDOW_SIZE {
            return Err(ArgumentsError::TooFewDataPoints(x.len()));
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
    pub x: Arcseconds,
    pub y: Arcseconds,
    pub d_ut1_utc: Seconds,
}

/// Perform Lagrangian interpolation of Earth Orientation Parameters (EOP), returning polar x- and
/// y- values and UT1-UTC at the target epoch. The result is corrected for oceanic and luni-solar
/// tidal effects.
pub fn interpolate(args: &Arguments) -> Interpolation {
    let x = crate::lagrange::interpolate(args.epochs, args.x, args.target_epoch);
    let y = crate::lagrange::interpolate(args.epochs, args.y, args.target_epoch);
    let t = crate::lagrange::interpolate(args.epochs, args.t, args.target_epoch);
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

    use float_eq::assert_float_eq;
    use rstest::{fixture, rstest};

    use crate::iers::parse_finals_csv;
    use crate::EarthOrientationParams;

    use super::*;

    #[rstest]
    #[case::valid(
        vec![0.0, 1.0, 2.0, 3.0], vec![0.0, 1.0, 2.0, 3.0], vec![0.0, 1.0, 2.0, 3.0], vec![0.0, 1.0, 2.0, 3.0], 0.0,
        Ok(Arguments {
            x: &[0.0, 1.0, 2.0, 3.0],
            y: &[0.0, 1.0, 2.0, 3.0],
            t: &[0.0, 1.0, 2.0, 3.0],
            epochs: &[0.0, 1.0, 2.0, 3.0],
            target_epoch: 0.0,
        })
    )]
    #[case::x_size_mismatch(vec![0.0], vec![], vec![], vec![], 0.0, Err(ArgumentsError::DimensionMismatch { nx: 1, ny: 0, nt: 0, nepochs: 0 }))]
    #[case::y_size_mismatch(vec![], vec![0.0], vec![], vec![], 0.0, Err(ArgumentsError::DimensionMismatch { nx: 0, ny: 1, nt: 0, nepochs: 0 }))]
    #[case::t_size_mismatch(vec![], vec![], vec![0.0], vec![], 0.0, Err(ArgumentsError::DimensionMismatch { nx: 0, ny: 0, nt: 1, nepochs: 0 }))]
    #[case::epochs_size_mismatch(vec![], vec![], vec![], vec![0.0], 0.0, Err(ArgumentsError::DimensionMismatch { nx: 0, ny: 0, nt: 0, nepochs: 1 }))]
    #[case::too_few_datapoints(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 2.0], 0.0, Err(ArgumentsError::TooFewDataPoints(3)))]
    fn test_arguments_new(
        #[case] x: Vec<Arcseconds>,
        #[case] y: Vec<Arcseconds>,
        #[case] t: Vec<Seconds>,
        #[case] epochs: Vec<ModifiedJulianDate>,
        #[case] target_epoch: ModifiedJulianDate,
        #[case] expected: Result<Arguments, ArgumentsError>,
    ) {
        let actual = Arguments::new(&x, &y, &t, &epochs, target_epoch);
        assert_eq!(expected, actual);
    }

    const FINALS2000A_PATH: &str = "../../data/finals2000A.all.csv";

    #[fixture]
    fn eop_data() -> EarthOrientationParams {
        let fixture_path = Path::new(FINALS2000A_PATH);
        parse_finals_csv(fixture_path).unwrap_or_else(|err| {
            panic!(
                "failed to parse test fixture at {}: {}",
                fixture_path.to_str().unwrap(),
                err,
            )
        })
    }

    #[rstest]
    #[case::mjd_j2000(MJD_J2000, Interpolation {
    x: 4.325128997437056e-2,
    y: 0.3779536211567663,
    d_ut1_utc: 0.35498904611828275,
    })]
    // Used to test the interpolator branch where the target date is less than two from the end of
    // the dataset.
    #[case::mjd_60615(60615.0, Interpolation {
    x: 0.2663521252106578,
    y: 0.298694318830590,
    d_ut1_utc: 4.7103969541161944e-2,
    })]
    // The following two test cases are far outside the range of IERS data, but are included to
    // establish consistency with the Bizouard F90 implementation at the extremes.
    #[case::mjd_0(0.0, Interpolation {
    x: 12072321.700398155,
    y: -24142704.67775462,
    d_ut1_utc: 778638165.7968734,
    })]
    #[case::mjd_j2100(88069.5, Interpolation {
    x: -16632958.650911978,
    y: 33267845.857896354,
    d_ut1_utc: -1072847942.5702964,
    })]
    fn test_lagrangian_interpolate(
        eop_data: EarthOrientationParams,
        #[case] target_epoch: ModifiedJulianDate,
        #[case] expected: Interpolation,
    ) -> Result<(), ArgumentsError> {
        let args = Arguments::new(
            &eop_data.x_pole,
            &eop_data.y_pole,
            &eop_data.delta_ut1_utc,
            &eop_data.mjd,
            target_epoch,
        )?;
        let result = interpolate(&args);

        assert_float_eq!(expected.x, result.x, rel <= 1e-9);
        assert_float_eq!(expected.y, result.y, rel <= 1e-9);
        assert_float_eq!(expected.d_ut1_utc, result.d_ut1_utc, rel <= 1e-9);

        Ok(())
    }
}
