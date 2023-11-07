/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::f64;
use std::iter::zip;
use std::ops::Sub;

use thiserror::Error;

fn diff<T: Sub<T, Output = T> + Copy>(x: &Vec<T>) -> Vec<T::Output> {
    let n = x.len();
    let mut dx = vec![];
    for (i1, i2) in zip(0..=n - 2, 1..=n - 1) {
        let dxi = x[i2] - x[i1];
        dx.push(dxi);
    }
    dx
}

#[derive(Error, Debug)]
pub enum AkimaError {
    #[error("size of `x` and `y` must match")]
    SizeMismatch,
}

pub struct Akima {
    n: usize,
    pub x: Vec<i32>,
    pub y: Vec<f64>,
    b: Vec<f64>,
    c: Vec<f64>,
    d: Vec<f64>,
}

impl Akima {
    pub fn new(x: Vec<i32>, y: Vec<f64>) -> Result<Self, AkimaError> {
        let n = x.len();
        if n != y.len() {
            return Err(AkimaError::SizeMismatch);
        }

        let mut b = vec![];
        let mut c = vec![];
        let mut d = vec![];

        let dx = diff(&x);
        let dy = diff(&y);
        let mut m = vec![];
        for (dxi, dyi) in zip(dx.clone(), dy) {
            m.push(dyi / f64::from(dxi));
        }
        m.insert(0, 2.0 * m[0] - m[1]);
        m.insert(0, 2.0 * m[0] - m[1]);
        m.push(2.0 * m[m.len() - 1] - m[m.len() - 2]);
        m.push(2.0 * m[m.len() - 1] - m[m.len() - 2]);

        for (i1, i2) in zip(3..m.len(), 0..m.len() - 3) {
            b.push(0.5 * (m[i1] + m[i2]));
        }

        let dm: Vec<f64> = diff(&m).iter().map(|x| x.abs()).collect();
        let f1 = &dm[2..n + 2];
        let f2 = &dm[0..n];
        let f12: Vec<f64> = zip(f1, f2).map(|(x1, x2)| x1 + x2).collect();
        let f12_max = f12.iter().cloned().fold(-1. / 0. /* inf */, f64::max);
        let ind: Vec<usize> = f12
            .iter()
            .enumerate()
            .filter(|(_, x)| **x > 1e-9 * f12_max)
            .map(|(idx, _)| idx)
            .collect();
        for i in ind {
            b[i] = (f1[i] * m[i + 1] + f2[i] * m[i + 2]) / f12[i]
        }

        for i in 0..n - 1 {
            c.push((3.0 * m[i + 2] - 2.0 * b[i] - b[i + 1]) / f64::from(dx[i]))
        }

        for i in 0..n - 1 {
            d.push((b[i] + b[i + 1] - 2.0 * m[i + 2]) / f64::from(dx[i].pow(2)))
        }
        Ok(Self {
            n: x.len(),
            x,
            y,
            b,
            c,
            d,
        })
    }

    pub fn interpolate(&self, xi: f64) -> f64 {
        let x = &self.x;
        let x0 = f64::from(x[0]);
        let xn = f64::from(x[x.len() - 1]);
        if xi <= x0 {
            return self.y[0];
        }
        if xi >= xn {
            return self.y[self.n - 1];
        }

        let idx = x.binary_search(&(xi.trunc() as i32)).unwrap();

        let wj = xi - f64::from(self.x[idx]);
        let y = self.y[idx];
        let b = self.b[idx];
        let c = self.c[idx];
        let d = self.d[idx];
        d.mul_add(wj, c).mul_add(wj, b).mul_add(wj, y)
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use super::*;

    #[test]
    fn test_diff() {
        let x = vec![
            -0.00561583609169947,
            0.29513581230551944,
            -0.944145132062222,
            0.8539804645085572,
            -0.6630410427468136,
            -0.33045519762661285,
            -0.5237166946868412,
            -1.1435794359757951,
            -0.5221715292393267,
            0.4762176135879527,
        ];

        let dx_exp = vec![
            0.3007516483972189,
            -1.2392809443677413,
            1.7981255965707792,
            -1.5170215072553708,
            0.3325858451202008,
            -0.19326149706022838,
            -0.6198627412889539,
            0.6214079067364684,
            0.9983891428272795,
        ];

        let dx_act = diff(&x);
        assert_eq!(dx_act, dx_exp);
    }

    #[test]
    fn test() {
        let x = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let y = vec![0.0, 2.0, 1.0, 3.0, 2.0, 6.0, 5.5, 5.5, 2.7, 5.1, 3.0];

        let b = vec![
            3.5,
            0.5,
            0.5,
            0.875,
            1.0,
            -0.09090909090909091,
            -0.1917808219178082,
            -0.2456140350877193,
            -0.8054794520547944,
            -0.01237113402061866,
            -4.349999999999999,
        ];

        let c = vec![
            -1.5,
            -4.5,
            4.125,
            -5.75,
            10.090909090909092,
            -1.1264009962640098,
            0.6291756789233357,
            -7.103292477769766,
            8.823330038130205,
            -1.9252577319587632,
        ];

        let d = vec![
            0.0,
            3.0,
            -2.625,
            3.875,
            -7.090909090909091,
            0.7173100871731009,
            -0.4373948570055275,
            4.548906512857486,
            -5.617850586075412,
            -0.16237113402061798,
        ];

        let xi = vec![
            0.0, 0.5, 1., 1.5, 2.5, 3.5, 4.5, 5.1, 6.5, 7.2, 8.6, 9.9, 10.,
        ];
        let yi = vec![
            0.,
            1.375,
            2.,
            1.5,
            1.953125,
            2.484375,
            4.136_363_636_363_637,
            5.980_362_391_033_624,
            5.506_729_151_646_239,
            5.203_136_745_974_525,
            4.179_655_415_901_708,
            3.411_038_659_793_813,
            3.0,
        ];

        let akima = Akima::new(x, y).expect("vectors lengths should match");
        assert_eq!(akima.b, b);
        assert_eq!(akima.c, c);
        assert_eq!(akima.d, d);

        for (xi, yi_exp) in zip(xi, yi) {
            let yi_act = akima.interpolate(xi);
            assert_float_eq!(yi_act, yi_exp, rel <= 1e-8);
        }
    }
}
