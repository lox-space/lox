/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fast_polynomial::poly_array;
use nalgebra::DVector;

use crate::linear_algebra::tridiagonal::Tridiagonal;
use crate::vector_traits::{Diff, SetSlice};

pub struct CubicSpline {
    n: usize,
    x: DVector<f64>,
    y: DVector<f64>,
    c1: DVector<f64>,
    c2: DVector<f64>,
    c3: DVector<f64>,
    c4: DVector<f64>,
}

impl CubicSpline {
    pub fn new(x: &DVector<f64>, y: &DVector<f64>) -> Result<Self, &'static str> {
        let n = x.len();
        let mut dl: DVector<f64> = DVector::zeros(n - 1);
        let mut d: DVector<f64> = DVector::zeros(n);
        let mut du: DVector<f64> = DVector::zeros(n - 1);
        let mut b: DVector<f64> = DVector::zeros(n);

        let dx = x.diff();
        let nd = dx.len();
        let slope = y.diff().component_div(&dx);

        let dx1 = dx.rows(0, nd - 1).clone_owned();
        let dx2 = dx.rows(1, nd - 1).clone_owned();
        let slope1 = slope.rows(0, nd - 1).clone_owned();
        let slope2 = slope.rows(1, nd - 1).clone_owned();

        d.set_slice(1, &(&dx1 + &dx2).scale(2.0));
        du.set_slice(1, &dx1);
        dl.set_slice(0, &dx2);
        b.set_slice(
            1,
            &(&dx2.component_mul(&slope1) + &dx1.component_mul(&slope2)).scale(3.0),
        );

        // Not-a-knot boundary condition
        d[0] = dx[1];
        du[0] = x[2] - x[0];
        let delta = x[2] - x[0];
        b[0] = ((dx[0] + 2.0 * delta) * dx[1] * slope[0] + dx[0].powi(2) * slope[1]) / delta;
        d[n - 1] = dx[nd - 1];
        let delta = x[n - 1] - x[n - 3];
        dl[nd - 1] = delta;
        b[n - 1] = (dx[nd - 1].powi(2) * slope[nd - 2]
            + (2.0 * delta + dx[nd - 1]) * dx[nd - 2] * slope[nd - 1])
            / delta;

        println!("{:?}", dx);
        println!("{:?}", d);
        println!("{:?}", b);

        let tri = Tridiagonal::new(dl, d, du)?;
        let s = tri.solve(&b).ok_or("could not be solved")?;
        let s1 = s.rows(0, n - 1).clone_owned();
        let s2 = s.rows(1, n - 1).clone_owned();
        let t = (&s1 + &s2 - &slope.scale(2.0)).component_div(&dx);

        let c1 = y.rows(0, n - 1).clone_owned();
        let c2 = s1.clone();
        let c3 = (&slope - &s1).component_div(&(&dx - &t));
        let c4 = t.component_div(&dx);

        Ok(Self {
            n,
            x: x.clone(),
            y: y.clone(),
            c1,
            c2,
            c3,
            c4,
        })
    }

    pub fn interpolate(&self, x0: f64) -> f64 {
        println!("{:?}", self.c2);
        let x: Vec<&f64> = self.x.iter().collect();
        let idx = x.binary_search_by(|val| val.partial_cmp(&&x0).unwrap());
        match idx {
            Ok(idx) => {
                let x = x0 - self.x[idx];
                poly_array(x, &[self.c1[idx], self.c2[idx], self.c3[idx], self.c4[idx]])
            }
            Err(idx) => match idx {
                0 => self.y[0],
                _ => self.y[self.n - 1],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use nalgebra::DVector;

    use super::*;

    #[test]
    fn test_cubic_spline() {
        let x: DVector<f64> = vec![
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
        ]
        .into();
        let y: DVector<f64> = vec![
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
        ]
        .into();

        let spl = CubicSpline::new(&x, &y).expect("should be valid");

        assert_float_eq!(spl.interpolate(1.0), -0.07321713407025687, rel <= 1e-8);
    }
}
