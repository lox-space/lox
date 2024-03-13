/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::ops::Index;

use nalgebra::{DMatrix, DVector};
use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("lengths of `dl` and `du` must be `d.len() - 1 = {0}` but was {1} and {2}")]
pub struct LoxTridiagonalError(usize, usize, usize);

type Idx = (usize, usize);

/// A tridiagonal matrix representation
#[derive(Clone, Debug, PartialEq)]
pub struct Tridiagonal<'a> {
    dl: &'a [f64],
    d: &'a [f64],
    du: &'a [f64],
}

impl<'a> Tridiagonal<'a> {
    pub fn new(dl: &'a [f64], d: &'a [f64], du: &'a [f64]) -> Result<Self, LoxTridiagonalError> {
        let n = d.len();
        if (dl.len() != n - 1 || du.len() != n - 1)
            && !(d.is_empty() && dl.is_empty() && du.is_empty())
        {
            return Err(LoxTridiagonalError(n - 1, dl.len(), du.len()));
        }
        Ok(Self { dl, d, du })
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

impl<'a> From<Tridiagonal<'a>> for DMatrix<f64> {
    fn from(value: Tridiagonal) -> Self {
        let n = value.d.len();
        DMatrix::from_fn(n, n, |i, j| value[(i, j)])
    }
}

impl<'a> Index<Idx> for Tridiagonal<'a> {
    type Output = f64;

    fn index(&self, (i, j): Idx) -> &Self::Output {
        let n = self.d.len();
        if i >= n {
            panic!(
                "row index out of bounds: the number of rows is {} but the index is {}",
                n, i
            )
        }
        if j >= n {
            panic!(
                "column index out of bounds: the number of columns is {} but the index is {}",
                n, j
            )
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
    use float_eq::assert_float_eq;

    use super::*;

    #[test]
    fn test_tridiagonal() {
        let du = vec![1.0, 2.0];
        let d = vec![3.0, 4.0, 5.0];
        let dl = vec![6.0, 7.0];
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

    #[test]
    fn test_tridiagonal_error() {
        let du = vec![1.0, 2.0];
        let d = vec![3.0, 4.0, 5.0];
        let dl = vec![6.0];
        let tri = Tridiagonal::new(&dl, &d, &du);

        assert_eq!(tri, Err(LoxTridiagonalError(2, 1, 2)));
    }

    #[test]
    fn test_tridiagonal_solve() {
        let du: Vec<f64> = vec![1.0, 2.0];
        let d: Vec<f64> = vec![3.0, 4.0, 5.0];
        let dl: Vec<f64> = vec![6.0, 7.0];
        let tri = Tridiagonal::new(&dl, &d, &du).expect("should be valid");

        let b = vec![1.0, 2.0, 3.0];
        let x = tri.solve(&b).expect("should be solvable");
        let exp = [-0.1666666666666666, 1.4999999999999996, -1.4999999999999993];

        assert_float_eq!(x[0], exp[0], rel <= 1e-8);
        assert_float_eq!(x[1], exp[1], rel <= 1e-8);
        assert_float_eq!(x[2], exp[2], rel <= 1e-8);
    }

    #[test]
    #[should_panic(expected = "row index out of bounds")]
    fn test_tridiagonal_invalid_row() {
        let du = vec![1.0, 2.0];
        let d = vec![3.0, 4.0, 5.0];
        let dl = vec![6.0, 7.0];
        let tri = Tridiagonal::new(&dl, &d, &du).expect("should be valid");
        let _x = tri[(3, 0)];
    }

    #[test]
    #[should_panic(expected = "column index out of bounds")]
    fn test_tridiagonal_invalid_column() {
        let du = vec![1.0, 2.0];
        let d = vec![3.0, 4.0, 5.0];
        let dl = vec![6.0, 7.0];
        let tri = Tridiagonal::new(&dl, &d, &du).expect("should be valid");
        let _x = tri[(0, 3)];
    }
}
