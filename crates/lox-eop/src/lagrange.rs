/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_bodies::fundamental::iers03::mean_moon_sun_elongation_iers03;
use lox_bodies::{Moon, Sun};
use lox_time::constants::f64::DAYS_PER_JULIAN_CENTURY; // todo: Circular dependency. Need to hoist constants.
use lox_time::intervals::TDBJulianCenturiesSinceJ2000;
use lox_utils::types::{Arcsec, Radians, Seconds};
use std::f64::consts::{FRAC_PI_2, PI, TAU};
use thiserror::Error;

type MJD = f64;

#[derive(Debug, Error)]
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
        let tidal_correction = ray(self.target_epoch);
        Interpolation {
            x: x + tidal_correction.x,
            y: y + tidal_correction.y,
            t: t + tidal_correction.t,
        }
    }
}

/// Perform Lagrangian interpolation within a set of (x, y) pairs, returning the y-value
/// corresponding to `target_x`
fn interpolate(x: &[f64], y: &[f64], target_x: f64) -> f64 {
    let n = x.len();
    let mut result = 0.0;
    let mut k = 0usize;
    for i in 0..(n - 1) {
        if target_x >= x[i] && target_x < x[i + 1] {
            k = i;
            break; // todo: break or continue?
        }
    }

    if k < 2 {
        k = 2;
    }
    if k > (n - 3) {
        k = n - 3
    }

    for m in k - 1..k + 2 {
        let mut term = y[m];
        for j in k - 1..k + 2 {
            if m != j {
                term *= (target_x - x[j]) / (x[m] - x[j]);
            }
        }
        result += term
    }

    return result;
}

struct TidalCorrection {
    x: Arcsec,
    y: Arcsec,
    t: Seconds,
}

/// Implements the Ray model for diurnal/subdiurnal tides.
fn ray(target_epoch: MJD) -> TidalCorrection {
    let t = (target_epoch - 51544.0) / DAYS_PER_JULIAN_CENTURY;

    // Fundamental arguments.
    // TODO: Fortran implementation gives these in Arcseconds, but the lox implementations are
    // converted to radians. We'd need implementations returning Arcseconds to use the same
    // approach for calculating args1-8.
    let l = Moon.mean_anomaly_iers03(t);
    let lp = Sun.mean_anomaly_iers03(t); // todo: LPRIME is defined but not used by Fortran implementation?
    let f = Moon.mean_longitude_minus_ascending_node_mean_longitude_iers03(t);
    let d = mean_moon_sun_elongation_iers03(t);
    let omega = Moon.ascending_node_mean_longitude_iers03(t);
    let theta = theta(t);

    // todo: this implementation can't be copied directly, since Lox funadamental args are already
    // in radians.
    const RADIANS_PER_ARCSECOND: Radians = PI / 648000.0;
    let arg7 = ((-l - 2.0 * f - 2.0 * omega + theta) * RADIANS_PER_ARCSECOND) % TAU - FRAC_PI_2; // todo: proper names for consts and pre-calculate where possible
    let arg1 = ((-2.0 * f - 2.0 * omega + theta) * RADIANS_PER_ARCSECOND) % TAU - FRAC_PI_2;
    let arg2 =
        ((-2.0 * f + 2.0 * d - 2.0 * omega + theta) * RADIANS_PER_ARCSECOND) % TAU + FRAC_PI_2;
    let arg3 = (theta * RADIANS_PER_ARCSECOND) % TAU + FRAC_PI_2;
    let arg4 = ((-L - 2.0 * f - 2.0 * omega + 2.0 * theta) * RADIANS_PER_ARCSECOND) % TAU;
    let arg5 = ((-2.0 * f - 2.0 * omega + 2.0 * theta) * RADIANS_PER_ARCSECOND) % TAU;
    let arg6 = ((-2.0 * f + 2.0 * d - 2.0 * omega + 2.0 * theta) * RADIANS_PER_ARCSECOND) % TAU;
    let arg8 = (-2.0 * theta * PI / 648000) % TAU;

    let (sin_arg7, cos_arg7) = arg7.sin_cos();
    let (sin_arg1, cos_arg1) = arg1.sin_cos();
    let (sin_arg2, cos_arg2) = arg2.sin_cos();
    let (sin_arg3, cos_arg3) = arg3.sin_cos();
    let (sin_arg4, cos_arg4) = arg4.sin_cos();
    let (sin_arg5, cos_arg5) = arg5.sin_cos();
    let (sin_arg6, cos_arg6) = arg6.sin_cos();
    let (sin_arg8, cos_arg8) = arg8.sin_cos();

    let x = -0.026 * sin_arg7 + 0.006 * cos_arg7 - 0.133 * sin_arg1 + 0.049 * cos_arg1
        - 0.050 * sin_arg2
        + 0.025 * cos_arg2
        - 0.152 * sin_arg3
        + 0.078 * cos_arg3
        - 0.057 * sin_arg4
        - 0.013 * cos_arg4
        - 0.330 * sin_arg5
        - 0.028 * cos_arg5
        - 0.145 * sin_arg6
        + 0.064 * cos_arg6
        - 0.036 * sin_arg8
        + 0.017 * cos_arg8;

    let y = -0.006 * sin_arg7
        - 0.026 * cos_arg7
        - 0.049 * sin_arg1
        - 0.133 * cos_arg1
        - 0.025 * sin_arg2
        - 0.050 * cos_arg2
        - 0.078 * sin_arg3
        - 0.152 * cos_arg3
        + 0.011 * sin_arg4
        + 0.033 * cos_arg4
        + 0.037 * sin_arg5
        + 0.196 * cos_arg5
        + 0.059 * sin_arg6
        + 0.087 * cos_arg6
        + 0.018 * sin_arg8
        + 0.022 * cos_arg8;

    let t = 0.0245 * sin_arg7
        + 0.0503 * cos_arg7
        + 0.1210 * sin_arg1
        + 0.1605 * cos_arg1
        + 0.0286 * sin_arg2
        + 0.0516 * cos_arg2
        + 0.0864 * sin_arg3
        + 0.1771 * cos_arg3
        - 0.0380 * sin_arg4
        - 0.0154 * cos_arg4
        - 0.1617 * sin_arg5
        - 0.0720 * cos_arg5
        - 0.0759 * sin_arg6
        - 0.0004 * cos_arg6
        - 0.0196 * sin_arg8
        - 0.0038 * cos_arg8;

    TidalCorrection {
        x: x * 1e-3,
        y: y * 1e-3,
        t: t * 1e-4,
    }
}

// todo: figure out where this should live
fn theta(t: TDBJulianCenturiesSinceJ2000) -> Radians {
    fast_polynomial::poly_array(
        t,
        &[
            67310.54841,
            (876600.0 * 3600.0 + 8640184.812866),
            0.093104,
            -6.2e-6,
        ],
    ) * 15.0
        + 648000.0
}
