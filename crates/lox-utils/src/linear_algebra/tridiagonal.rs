/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::ops::{Index, IndexMut};

use nalgebra::{DMatrix, DVector, RealField};

type Idx = (usize, usize);

/// A tridiagonal matrix representation
#[derive(Clone, Debug)]
pub struct Tridiagonal<T: RealField> {
    zero: T,
    dl: DVector<T>,
    d: DVector<T>,
    du: DVector<T>,
}

impl<T: RealField> Tridiagonal<T> {
    pub fn new(dl: DVector<T>, d: DVector<T>, du: DVector<T>) -> Result<Self, &'static str> {
        let n = d.len();
        if (dl.len() != n - 1 || du.len() != n - 1)
            && !(d.is_empty() && dl.is_empty() && du.is_empty())
        {
            return Err("wrong");
        }
        Ok(Self {
            zero: T::zero(),
            dl,
            d,
            du,
        })
    }

    pub fn shape(&self) -> (usize, usize) {
        (self.d.len(), self.d.len())
    }

    pub fn solve(&self, b: &DVector<T>) -> Option<DVector<T>> {
        let m: DMatrix<T> = self.clone().into();
        m.lu().solve(b)
    }
}

impl<T: RealField> From<Tridiagonal<T>> for DMatrix<T> {
    fn from(value: Tridiagonal<T>) -> Self {
        let n = value.d.len();
        DMatrix::from_fn(n, n, |i, j| value[(i, j)].clone())
    }
}

impl<T: RealField> Index<Idx> for Tridiagonal<T> {
    type Output = T;

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
            &self.zero
        }
    }
}

impl<T: RealField> IndexMut<Idx> for Tridiagonal<T> {
    fn index_mut(&mut self, (i, j): Idx) -> &mut T {
        let n = self.d.len();
        if i >= n || j >= n {
            panic!("Index out of bounds")
        }
        if i == j {
            &mut self.d[i]
        } else if i == j + 1 {
            &mut self.dl[j]
        } else if i + 1 == j {
            &mut self.du[i]
        } else {
            &mut self.zero
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tridiagonal() {
        let du: DVector<f64> = vec![1.0, 2.0].into();
        let d: DVector<f64> = vec![3.0, 4.0, 5.0].into();
        let dl: DVector<f64> = vec![6.0, 7.0].into();
        let tri = Tridiagonal::new(dl, d, du).expect("should be valid");

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
