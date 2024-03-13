/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::ops::Index;

use nalgebra::{DMatrix, DVector};

type Idx = (usize, usize);

/// A tridiagonal matrix representation
#[derive(Clone, Debug)]
pub struct Tridiagonal {
    dl: Vec<f64>,
    d: Vec<f64>,
    du: Vec<f64>,
}

impl Tridiagonal {
    pub fn new(dl: &[f64], d: &[f64], du: &[f64]) -> Result<Self, &'static str> {
        let n = d.len();
        if (dl.len() != n - 1 || du.len() != n - 1)
            && !(d.is_empty() && dl.is_empty() && du.is_empty())
        {
            return Err("wrong");
        }
        Ok(Self {
            dl: dl.to_vec(),
            d: d.to_vec(),
            du: du.to_vec(),
        })
    }

    pub fn shape(&self) -> (usize, usize) {
        (self.d.len(), self.d.len())
    }

    pub fn solve(&self, b: &[f64]) -> Option<Vec<f64>> {
        let b: DVector<f64> = b.to_vec().into();
        let m: DMatrix<f64> = self.clone().into();
        m.lu().solve(&b).map(|x| x.data.into())
    }
}

impl From<Tridiagonal> for DMatrix<f64> {
    fn from(value: Tridiagonal) -> Self {
        let n = value.d.len();
        DMatrix::from_fn(n, n, |i, j| value[(i, j)])
    }
}

impl Index<Idx> for Tridiagonal {
    type Output = f64;

    fn index(&self, (i, j): Idx) -> &Self::Output {
        let n = self.d.len();
        if i >= n || j >= n {
            panic!("Index out of bounds")
        }
        if i == j {
            &self.d[i]
        } else if i == j + 1 {
            &self.dl[j]
        } else if i + 1 == j {
            &self.du[i]
        } else {
            &0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tridiagonal() {
        let du: Vec<f64> = vec![1.0, 2.0];
        let d: Vec<f64> = vec![3.0, 4.0, 5.0];
        let dl: Vec<f64> = vec![6.0, 7.0];
        let tri = Tridiagonal::new(&dl, &d, &du).expect("should be valid");

        assert_eq!(tri.shape(), (3, 3));

        assert_eq!(&tri[(0, 0)], &3.0);
        assert_eq!(&tri[(1, 0)], &6.0);
        assert_eq!(&tri[(2, 0)], &0.0);
        assert_eq!(&tri[(0, 1)], &1.0);
        assert_eq!(&tri[(1, 1)], &4.0);
        assert_eq!(&tri[(2, 1)], &7.0);
        assert_eq!(&tri[(0, 2)], &0.0);
        assert_eq!(&tri[(1, 2)], &2.0);
        assert_eq!(&tri[(2, 2)], &5.0);

        let m: DMatrix<f64> = tri.into();
        let exp = DMatrix::from_row_slice(3, 3, &[3.0, 1.0, 0.0, 6.0, 4.0, 2.0, 0.0, 7.0, 5.0]);
        assert_eq!(m, exp);
    }
}
