/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

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
    x: Vec<Arcseconds>,
    /// y polar motion.
    y: Vec<Arceconds>,
    /// UT1-UTC.
    t: Vec<Seconds>,
    /// Epochs of the data.
    epochs: Vec<MJD>,
    /// Epoch of the interpolated data.
    target_epoch: MJD,
}

impl Arguments {
    fn new(
        x: Vec<Arcseconds>,
        y: Vec<Arcseconds>,
        t: Vec<Seconds>,
        epochs: Vec<MJD>,
        target_epoch: MJD,
    ) -> Result<Arguments, LagrangeError> {
        if x.size != y.size || x.size != t.size || x.size != epochs.size {
            Err(LagrangeError::SizeMismatch {
                nx: x.size,
                ny: y.size,
                nt: t.size,
                nepochs: epochs.size,
            })
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
    x: Arcseconds,
    y: Arcseconds,
    t: Seconds,
}

// todo: PartialEq?
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lagrange {
    n: usize,
    /// x polar motion.
    x: Vec<Arcseconds>,
    /// y polar motion.
    y: Vec<Arceconds>,
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
            n: args.x.size,
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
        let tidal_correction = ray(&self.target_epoch);
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
    let n = x.size();
    let result = 0.0;
    let mut k = 0isize; // isize needed?
    for i in 0..(n - 1) {
        if target_x >= x[i] && target_x < x[i + 1] {
            k = i as isize;
            break; // break or continue?
        }
    }

    if k < 2 {
        k = 2;
    }

    if k > (n - 3) {
        k = (n - 3)
    }

    for m in (k - 1..k + 2) {
        let term = y[m];
        for j in (k - 1..k + 2) {
            if m != j {
                term *= (target_x - x[j]) / (x[m] - x[j]);
            }
        }
        result += term
    }

    return result;
}

struct TidalCorrection {
    x: Arcseconds,
    y: Arcseconds,
    t: Seconds,
}

/// Implements the Ray model for diurnal/subdiurnal tides.
fn ray(target_epoch: MJD) -> TidalCorrection {
    let t = (MJD - 51544.0) / DAYS_PER_JULIAN_CENTURY;

    // Fundamental arguments.
    let l = Moon::mean_anomaly_iers03(t);
    let lp = Sun::mean_anomaly_iers03(t);
    let f = Moon::mean_longitude_minus_ascending_node_mean_longitude(t);
    let d = mean_moon_sun_elongation_iers03(t);
    let omega = Moon::ascending_node_mean_longitude(t);
    let theta = theta(t);

    let arg7 = ((-l - 2.0 * f - 2.0 * omega + theta) * PI / 648000.0) % TAU - PI / 2; // todo: proper names for consts and pre-calculate where possible
    let arg1 = ((-2.0 * f - 2.0 * omega + theta) * PI / 648000.0) % TAU - PI / 2;
    let arg2 = ((-2.0 * f + 2.0 * d - 2.0 * omega + theta) * PI / 648000.0) % TAU + PI / 2;
    let arg3 = (theta * PI / 648000.0) % TAU + PI / 2;
    let arg4 = ((-L - 2.0 * f - 2.0 * omega + 2.0 * theta) * PI / 648000.0) % TAU;
    let arg5 = ((-2.0 * f - 2.0 * omega + 2.0 * theta) * PI / 648000.0) % TAU;
    let arg6 = ((-2.0 * f + 2.0 * d - 2.0 * omega + 2.0 * theta) * PI / 648000.0) % TAU;
    let arg8 = (-2.0 * theta * PI / 648000) % TAU;

    let (sinArg7, cosArg7) = sincos(arg7);
    let (sinArg1, cosArg1) = sincos(arg1);
    let (sinArg2, cosArg2) = sincos(arg2);
    let (sinArg3, cosArg3) = sincos(arg3);
    let (sinArg4, cosArg4) = sincos(arg4);
    let (sinArg5, cosArg5) = sincos(arg5);
    let (sinArg6, cosArg6) = sincos(arg6);
    let (sinArg8, cosArg8) = sincos(arg8);

    let x = -0.026 * sinArg7 + 0.006 * cosArg7 - 0.133 * sinArg1 + 0.049 * cosArg1
        - 0.050 * sinArg2
        + 0.025 * cosArg2
        - 0.152 * sinArg3
        + 0.078 * cosArg3
        - 0.057 * sinArg4
        - 0.013 * cosArg4
        - 0.330 * sinArg5
        - 0.028 * cosArg5
        - 0.145 * sinArg6
        + 0.064 * cosArg6
        - 0.036 * sinArg8
        + 0.017 * cosArg8;

    let y = -0.006 * sinArg7
        - 0.026 * cosArg7
        - 0.049 * sinArg1
        - 0.133 * cosArg1
        - 0.025 * sinArg2
        - 0.050 * cosArg2
        - 0.078 * sinArg3
        - 0.152 * cosArg3
        + 0.011 * sinArg4
        + 0.033 * cosArg4
        + 0.037 * sinArg5
        + 0.196 * cosArg5
        + 0.059 * sinArg6
        + 0.087 * cosArg6
        + 0.018 * sinArg8
        + 0.022 * cosArg8;

    let t = 0.0245 * sinArg7
        + 0.0503 * cosArg7
        + 0.1210 * sinArg1
        + 0.1605 * cosArg1
        + 0.0286 * sinArg2
        + 0.0516 * cosArg2
        + 0.0864 * sinArg3
        + 0.1771 * cosArg3
        - 0.0380 * sinArg4
        - 0.0154 * cosArg4
        - 0.1617 * sinArg5
        - 0.0720 * cosArg5
        - 0.0759 * sinArg6
        - 0.0004 * cosArg6
        - 0.0196 * sinArg8
        - 0.0038 * cosArg8;

    TidalCorrection {
        x: x * 1e-3,
        y: y * 1e-3,
        t: t * 1e-4
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
