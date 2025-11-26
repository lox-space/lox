// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use fast_polynomial::poly_array;
use thiserror::Error;

use crate::math::roots::RootFinderError;
use crate::math::slices::Monotonic;

use super::linear_algebra::tridiagonal::Tridiagonal;
use super::slices::Diff;

const MIN_POINTS_LINEAR: usize = 2;
const MIN_POINTS_SPLINE: usize = 4;

#[derive(Clone, Debug, Error, PartialEq)]
pub enum SeriesError {
    #[error("`x` and `y` must have the same length but were {0} and {1}")]
    DimensionMismatch(usize, usize),
    #[error("length of `x` and `y` must at least 2 but was {0}")]
    InsufficientPoints(usize),
    #[error("x-axis must be strictly monotonic")]
    NonMonotonic,
    #[error(transparent)]
    RootFinder(#[from] RootFinderError),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Interpolation {
    Linear,
    CubicSpline(Arc<[f64]>, Arc<[f64]>, Arc<[f64]>, Arc<[f64]>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Series {
    x: Arc<[f64]>,
    y: Arc<[f64]>,
    interpolation: Interpolation,
}

pub enum InterpolationType {
    Linear,
    CubicSpline,
}

impl Series {
    pub fn try_new(
        x: impl Into<Arc<[f64]>>,
        y: impl Into<Arc<[f64]>>,
        interpolation: InterpolationType,
    ) -> Result<Self, SeriesError> {
        let x: Arc<[f64]> = x.into();
        let y: Arc<[f64]> = y.into();

        Self::check(&x, &y)?;

        Ok(Self::new(x, y, interpolation))
    }

    pub fn new(
        x: impl Into<Arc<[f64]>>,
        y: impl Into<Arc<[f64]>>,
        interpolation: InterpolationType,
    ) -> Self {
        let x: Arc<[f64]> = x.into();
        let y: Arc<[f64]> = y.into();

        Self::assert(&x, &y);

        match interpolation {
            InterpolationType::Linear => Self::linear(x, y),
            InterpolationType::CubicSpline => {
                let n = x.len();
                if n < MIN_POINTS_SPLINE {
                    Self::linear(x, y)
                } else {
                    Self::cubic_spline(x, y)
                }
            }
        }
    }

    fn linear(x: Arc<[f64]>, y: Arc<[f64]>) -> Self {
        Self {
            x,
            y,
            interpolation: Interpolation::Linear,
        }
    }

    fn cubic_spline(x: Arc<[f64]>, y: Arc<[f64]>) -> Self {
        let n = x.len();

        let dx = x.diff();
        let nd = dx.len();
        let slope: Vec<f64> = y
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
        du.insert(0, x[2] - x[0]);
        let delta = x[2] - x[0];
        b.insert(
            0,
            ((dx[0] + 2.0 * delta) * dx[1] * slope[0] + dx[0].powi(2) * slope[1]) / delta,
        );
        d.push(dx[nd - 2]);
        let delta = x[n - 1] - x[n - 3];
        dl.push(delta);
        b.push(
            (dx[nd - 1].powi(2) * slope[nd - 2]
                + (2.0 * delta + dx[nd - 1]) * dx[nd - 2] * slope[nd - 1])
                / delta,
        );

        let tri = Tridiagonal::new(&dl, &d, &du).unwrap_or_else(|err| {
            unreachable!(
                "dimensions should be correct for tridiagonal system: {}",
                err
            )
        });
        let s = tri.solve(&b);
        let t: Vec<f64> = s[0..n - 1]
            .iter()
            .enumerate()
            .map(|(idx, si)| (si + s[idx + 1] - 2.0 * slope[idx]) / dx[idx])
            .collect();

        let c1 = y[0..n - 1].to_vec();
        let c2 = s[0..n - 1].to_vec();
        let c3: Vec<f64> = slope
            .iter()
            .enumerate()
            .map(|(idx, si)| (si - s[idx]) / dx[idx] - t[idx])
            .collect();
        let c4: Vec<f64> = t.iter().enumerate().map(|(idx, ti)| ti / dx[idx]).collect();

        Self {
            x,
            y,
            interpolation: Interpolation::CubicSpline(c1.into(), c2.into(), c3.into(), c4.into()),
        }
    }

    pub fn interpolate(&self, xp: f64) -> f64 {
        let x = self.x.as_ref();
        let y = self.y.as_ref();
        let x0 = *x.first().unwrap();
        let xn = *x.last().unwrap();
        let idx = if xp <= x0 {
            0
        } else if xp >= xn {
            x.len() - 2
        } else {
            x.partition_point(|&val| xp > val) - 1
        };
        match &self.interpolation {
            Interpolation::Linear => {
                let x0 = x[idx];
                let x1 = x[idx + 1];
                let y0 = y[idx];
                let y1 = y[idx + 1];
                y0 + (y1 - y0) * (xp - x0) / (x1 - x0)
            }
            Interpolation::CubicSpline(c1, c2, c3, c4) => {
                poly_array(xp - x[idx], &[c1[idx], c2[idx], c3[idx], c4[idx]])
            }
        }
    }

    pub fn x(&self) -> &[f64] {
        self.x.as_ref()
    }

    pub fn y(&self) -> &[f64] {
        self.y.as_ref()
    }

    pub fn first(&self) -> (f64, f64) {
        (*self.x().first().unwrap(), *self.y().first().unwrap())
    }

    pub fn last(&self) -> (f64, f64) {
        (*self.x().last().unwrap(), *self.y().last().unwrap())
    }

    fn check(x: &[f64], y: &[f64]) -> Result<(), SeriesError> {
        if !x.is_strictly_increasing() {
            return Err(SeriesError::NonMonotonic);
        }

        let n = x.len();

        if y.len() != n {
            return Err(SeriesError::DimensionMismatch(n, y.len()));
        }

        if n < MIN_POINTS_LINEAR {
            return Err(SeriesError::InsufficientPoints(n));
        }
        Ok(())
    }

    fn assert(x: &[f64], y: &[f64]) {
        assert!(x.is_strictly_increasing());

        let n = x.len();
        assert!(y.len() == n);
        assert!(n >= MIN_POINTS_LINEAR);
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[rstest]
    #[case(0.5, 0.5)]
    #[case(1.0, 1.0)]
    #[case(1.5, 1.5)]
    #[case(2.5, 2.5)]
    #[case(5.5, 5.5)]
    fn test_series_linear(#[case] xp: f64, #[case] expected: f64) {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let s = Series::try_new(x, y, InterpolationType::Linear).unwrap();
        let actual = s.interpolate(xp);
        assert_eq!(actual, expected);
    }

    // Reference values from AstroBase.jl
    #[rstest]
    #[case(0.0, -14.303290471048534)]
    #[case(0.1, -12.036932976759344)]
    #[case(0.2, -9.978070560771739)]
    #[case(0.3, -8.117883404355377)]
    #[case(0.4, -6.447551688779917)]
    #[case(0.5, -4.958255595315013)]
    #[case(0.6, -3.6411753052303184)]
    #[case(0.7, -2.487490999795493)]
    #[case(0.8, -1.4883828602801898)]
    #[case(0.9, -0.6350310679540686)]
    #[case(1.0, 0.08138419591321655)]
    #[case(1.1, 0.6696827500520098)]
    #[case(1.2, 1.1386844131926532)]
    #[case(1.3, 1.4972090040654928)]
    #[case(1.4, 1.754076341400871)]
    #[case(1.5, 1.9181062439291328)]
    #[case(1.6, 1.9981185303806206)]
    #[case(1.7, 2.002933019485679)]
    #[case(1.8, 1.9413695299746523)]
    #[case(1.9, 1.8222478805778837)]
    #[case(2.0, 1.6543878900257172)]
    #[case(2.1, 1.4466093770484965)]
    #[case(2.2, 1.2077321603765656)]
    #[case(2.3, 0.9465760587402696)]
    #[case(2.4, 0.6719608908699499)]
    #[case(2.5, 0.3927064754959517)]
    #[case(2.6, 0.11763263134861876)]
    #[case(2.7, -0.14444082284170534)]
    #[case(2.8, -0.384694068344675)]
    #[case(2.9, -0.5943072864299493)]
    #[case(3.0, -0.7644606583671828)]
    #[case(3.1, -0.8886377407066958)]
    #[case(3.2, -0.9695355911214641)]
    #[case(3.3, -1.012154642565128)]
    #[case(3.4, -1.021495327991328)]
    #[case(3.5, -1.0025580803537035)]
    #[case(3.6, -0.960343332605895)]
    #[case(3.7, -0.8998515177015425)]
    #[case(3.8, -0.8260830685942864)]
    #[case(3.9, -0.744038418237766)]
    #[case(4.0, -0.6587179995856219)]
    #[case(4.1, -0.5751222455914945)]
    #[case(4.2, -0.4982515892090227)]
    #[case(4.3, -0.433106463391848)]
    #[case(4.4, -0.38468730109360944)]
    #[case(4.5, -0.3579945352679478)]
    #[case(4.6, -0.3580285988685027)]
    #[case(4.7, -0.3897899248489146)]
    #[case(4.8, -0.458278946162823)]
    #[case(4.9, -0.5684960957638693)]
    #[case(5.0, -0.7254418066056914)]
    #[case(5.1, -0.9341165116419302)]
    #[case(5.2, -1.1995206438262285)]
    #[case(5.3, -1.5266546361122217)]
    #[case(5.4, -1.9205189214535554)]
    #[case(5.5, -2.3861139328038625)]
    #[case(5.6, -2.9284401031167873)]
    #[case(5.7, -3.5524978653459742)]
    #[case(5.8, -4.263287652445054)]
    #[case(5.9, -5.065809897367678)]
    #[case(6.0, -5.965065033067472)]
    fn test_series_spline(#[case] xp: f64, #[case] expected: f64) {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![
            0.08138419591321655,
            1.6543878900257172,
            -0.7644606583671828,
            -0.6587179995856219,
            -0.7254418066056914,
        ];

        let s = Series::try_new(x, y, InterpolationType::CubicSpline).unwrap();
        let actual = s.interpolate(xp);
        assert_approx_eq!(actual, expected, rtol <= 1e-12);
    }

    #[rstest]
    #[case(Series::try_new(vec![1.0], vec![1.0], InterpolationType::Linear), Err(SeriesError::InsufficientPoints(1)))]
    #[case(Series::try_new(vec![1.0], vec![1.0], InterpolationType::CubicSpline), Err(SeriesError::InsufficientPoints(1)))]
    #[case(Series::try_new(vec![1.0, 2.0], vec![1.0], InterpolationType::Linear), Err(SeriesError::DimensionMismatch(2, 1)))]
    #[case(Series::try_new(vec![1.0, 2.0], vec![1.0], InterpolationType::CubicSpline), Err(SeriesError::DimensionMismatch(2, 1)))]
    fn test_series_errors(
        #[case] actual: Result<Series, SeriesError>,
        #[case] expected: Result<Series, SeriesError>,
    ) {
        assert_eq!(actual, expected);
    }
}
