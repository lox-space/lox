// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Tridiagonal matrix representation and solver.

use alloc::vec;
use alloc::vec::Vec;
use core::ops::Index;

use thiserror::Error;

/// Error returned when the diagonal dimensions of a [`Tridiagonal`] matrix are inconsistent.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("lengths of `dl` and `du` must be `d.len() - 1 = {expected}` but were {dl} and {du}")]
pub struct TridiagonalError {
    /// The required length of the off-diagonals, `d.len() - 1`.
    pub expected: usize,
    /// The actual length of the lower diagonal `dl`.
    pub dl: usize,
    /// The actual length of the upper diagonal `du`.
    pub du: usize,
}

type Idx = (usize, usize);

/// A tridiagonal matrix representation
#[derive(Clone, Debug, PartialEq)]
pub struct Tridiagonal<'a> {
    dl: &'a [f64],
    d: &'a [f64],
    du: &'a [f64],
}

impl<'a> Tridiagonal<'a> {
    /// Creates a new tridiagonal matrix from lower, main, and upper diagonals.
    pub fn new(dl: &'a [f64], d: &'a [f64], du: &'a [f64]) -> Result<Self, TridiagonalError> {
        let expected = d.len().saturating_sub(1);
        if dl.len() != expected || du.len() != expected {
            return Err(TridiagonalError {
                expected,
                dl: dl.len(),
                du: du.len(),
            });
        }
        Ok(Self { dl, d, du })
    }

    /// Returns the shape `(n, n)` of the matrix.
    pub fn shape(&self) -> (usize, usize) {
        (self.d.len(), self.d.len())
    }

    /// Solves the tridiagonal system `Ax = d`.
    ///
    /// # Panics
    ///
    /// Panics if `d.len()` differs from the number of rows of the matrix.
    pub fn solve(&self, d: &[f64]) -> Vec<f64> {
        let n = self.d.len();
        assert_eq!(
            d.len(),
            n,
            "right-hand side must have length {n} but was {}",
            d.len()
        );
        if n == 0 {
            return Vec::new();
        }
        let a = self.dl;
        let b = self.d;
        let c = self.du;

        let mut w = vec![0.0; n - 1];
        let mut g = vec![0.0; n];
        let mut p = vec![0.0; n];

        if n > 1 {
            w[0] = c[0] / b[0];
        }
        g[0] = d[0] / b[0];

        for i in 1..n - 1 {
            w[i] = c[i] / (b[i] - a[i - 1] * w[i - 1]);
        }
        for i in 1..n {
            g[i] = (d[i] - a[i - 1] * g[i - 1]) / (b[i] - a[i - 1] * w[i - 1]);
        }
        p[n - 1] = g[n - 1];
        for i in (1..n).rev() {
            p[i - 1] = g[i - 1] - w[i - 1] * p[i];
        }

        p
    }
}

impl Index<Idx> for Tridiagonal<'_> {
    type Output = f64;

    fn index(&self, (i, j): Idx) -> &Self::Output {
        let n = self.d.len();
        if i >= n {
            panic!("row index out of bounds: the number of rows is {n} but the index is {i}")
        }
        if j >= n {
            panic!("column index out of bounds: the number of columns is {n} but the index is {j}")
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
    use lox_approx::assert_approx_eq;

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
    }

    #[test]
    fn test_tridiagonal_error() {
        let du = vec![1.0, 2.0];
        let d = vec![3.0, 4.0, 5.0];
        let dl = vec![6.0];
        let tri = Tridiagonal::new(&dl, &d, &du);

        assert_eq!(
            tri,
            Err(TridiagonalError {
                expected: 2,
                dl: 1,
                du: 2,
            })
        );
    }

    #[test]
    #[should_panic(expected = "right-hand side must have length 3 but was 2")]
    fn test_tridiagonal_solve_rhs_length_mismatch() {
        let du = vec![1.0, 2.0];
        let d = vec![3.0, 4.0, 5.0];
        let dl = vec![6.0, 7.0];
        let tri = Tridiagonal::new(&dl, &d, &du).expect("should be valid");
        let _x = tri.solve(&[1.0, 2.0]);
    }

    #[test]
    fn test_tridiagonal_solve_single() {
        let tri = Tridiagonal::new(&[], &[4.0], &[]).expect("should be valid");
        assert_eq!(tri.solve(&[2.0]), vec![0.5]);
    }

    #[test]
    fn test_tridiagonal_solve_empty() {
        let tri = Tridiagonal::new(&[], &[], &[]).expect("should be valid");
        assert!(tri.solve(&[]).is_empty());
    }

    #[test]
    fn test_tridiagonal_solve() {
        let du: Vec<f64> = vec![1.0, 2.0];
        let d: Vec<f64> = vec![3.0, 4.0, 5.0];
        let dl: Vec<f64> = vec![6.0, 7.0];
        let tri = Tridiagonal::new(&dl, &d, &du).expect("should be valid");

        let b = vec![1.0, 2.0, 3.0];
        let x = tri.solve(&b);
        let exp = [-0.1666666666666666, 1.5, -1.5];

        assert_approx_eq!(x[0], exp[0], rtol <= 1e-14);
        assert_approx_eq!(x[1], exp[1], rtol <= 1e-14);
        assert_approx_eq!(x[2], exp[2], rtol <= 1e-14);
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
