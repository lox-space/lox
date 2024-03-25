/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fast_polynomial::poly_array;
use thiserror::Error;

use crate::linear_algebra::tridiagonal::Tridiagonal;
use crate::vector_traits::Diff;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum LoxCubicSplineError {
    #[error("`x` and `y` must have the same length but were {0} and {1}")]
    DimensionMismatch(usize, usize),
    #[error("length of `x` and `y` must at least 4 but was {0}")]
    InsufficientPoints(usize),
    #[error("linear system could not be solved")]
    Unsolvable,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CubicSpline<TX: AsRef<[f64]>, TY: AsRef<[f64]>> {
    n: usize,
    x: TX,
    y: TY,
    c1: Vec<f64>,
    c2: Vec<f64>,
    c3: Vec<f64>,
    c4: Vec<f64>,
}

impl<TX: AsRef<[f64]>, TY: AsRef<[f64]>> CubicSpline<TX, TY> {
    pub fn new(x: TX, y: TY) -> Result<Self, LoxCubicSplineError> {
        let xr = x.as_ref();
        let yr = y.as_ref();
        let n = xr.len();

        if yr.len() != n {
            return Err(LoxCubicSplineError::DimensionMismatch(n, yr.len()));
        }

        if n < 4 {
            return Err(LoxCubicSplineError::InsufficientPoints(n));
        }

        let dx = xr.diff();
        let nd = dx.len();
        let slope: Vec<f64> = yr
            .diff()
            .iter()
            .enumerate()
            .map(|(idx, y)| y / dx[idx])
            .collect();

        let mut d: Vec<f64> = dx[0..nd - 1]
            .iter()
            .enumerate()
            .map(|(idx, dxi)| 2.0 * (dxi + dx[idx + 1]))
            .collect();
        let mut du: Vec<f64> = dx[0..nd - 1].to_vec();
        let mut dl: Vec<f64> = dx[1..].to_vec();
        let mut b: Vec<f64> = dx[0..nd - 1]
            .iter()
            .enumerate()
            .map(|(idx, dxi)| 3.0 * (dx[idx + 1] * slope[idx] + dxi * slope[idx + 1]))
            .collect();

        // Not-a-knot boundary condition
        d.insert(0, dx[1]);
        du.insert(0, xr[2] - xr[0]);
        let delta = xr[2] - xr[0];
        b.insert(
            0,
            ((dx[0] + 2.0 * delta) * dx[1] * slope[0] + dx[0].powi(2) * slope[1]) / delta,
        );
        d.push(dx[nd - 2]);
        let delta = xr[n - 1] - xr[n - 3];
        dl.push(delta);
        b.push(
            (dx[nd - 1].powi(2) * slope[nd - 2]
                + (2.0 * delta + dx[nd - 1]) * dx[nd - 2] * slope[nd - 1])
                / delta,
        );

        let tri = Tridiagonal::new(&dl, &d, &du).expect("should be valid");
        let s = tri.solve(&b).ok_or(LoxCubicSplineError::Unsolvable)?;
        let t: Vec<f64> = s[0..n - 1]
            .iter()
            .enumerate()
            .map(|(idx, si)| (si + s[idx + 1] - 2.0 * slope[idx]) / dx[idx])
            .collect();

        let c1 = yr[0..n - 1].to_vec();
        let c2 = s[0..n - 1].to_vec();
        let c3: Vec<f64> = slope
            .iter()
            .enumerate()
            .map(|(idx, si)| (si - s[idx]) / dx[idx] - t[idx])
            .collect();
        let c4: Vec<f64> = t.iter().enumerate().map(|(idx, ti)| ti / dx[idx]).collect();

        Ok(Self {
            n,
            x,
            y,
            c1,
            c2,
            c3,
            c4,
        })
    }

    pub fn interpolate(&self, x0: f64) -> f64 {
        let x = self.x.as_ref();
        let y = self.y.as_ref();
        let mut idx = x.partition_point(|val| x0 >= *val);
        if idx == 0 {
            return *y.first().unwrap();
        }
        if idx == self.n {
            return *y.last().unwrap();
        }
        idx -= 1;
        let x = x0 - x[idx];
        poly_array(x, &[self.c1[idx], self.c2[idx], self.c3[idx], self.c4[idx]])
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use super::*;

    #[test]
    fn test_cubic_spline() {
        let x = vec![
            -1.45971551,
            -1.27401241,
            -1.24858571,
            -0.98854859,
            -0.85262672,
            -0.36269993,
            0.12312452,
            0.20519359,
            0.82698527,
            1.18579896,
        ];
        let y = vec![
            -1.12016875,
            0.53793553,
            -0.32205336,
            -0.73225522,
            0.53240318,
            -0.42591753,
            0.96449187,
            0.2450982,
            -0.68313154,
            0.52273895,
        ];

        let spl = CubicSpline::new(&x, &y).expect("should be valid");

        assert_float_eq!(spl.interpolate(1.0), -0.07321713407025687, rel <= 1e-8);
        assert_eq!(spl.interpolate(-5.0), y[0]);
        assert_eq!(spl.interpolate(5.0), y[9]);
    }

    #[test]
    fn test_cubic_spline_errors() {
        let x = vec![1.0, 2.0, 3.0, 4.0];
        let y = vec![1.0, 2.0, 3.0, 4.0];

        let spl = CubicSpline::new(&x, &y);
        assert!(spl.is_ok());

        let spl = CubicSpline::new(&x, &y[0..3]);
        assert_eq!(spl, Err(LoxCubicSplineError::DimensionMismatch(4, 3)));

        let spl = CubicSpline::new(&x[0..3], &y[0..3]);
        assert_eq!(spl, Err(LoxCubicSplineError::InsufficientPoints(3)));
    }
}
