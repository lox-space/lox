/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use nalgebra::{DVector, RealField};

pub trait Diff<T: RealField> {
    fn diff(&self) -> DVector<T>;
}

impl<T: RealField> Diff<T> for DVector<T> {
    fn diff(&self) -> DVector<T> {
        let n = self.len();
        let x1 = self.rows(1, n - 1);
        let x0 = self.rows(0, n - 1);
        x1 - x0
    }
}

pub trait SetSlice<T: RealField> {
    fn set_slice(&mut self, start: usize, s: &DVector<T>);
}

impl<T: RealField> SetSlice<T> for DVector<T> {
    fn set_slice(&mut self, start: usize, s: &DVector<T>) {
        let end = s.len();
        for (j, i) in (start..end).enumerate() {
            self[(i, 0)] = s[(j, 0)].clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::DVector;

    use super::*;

    #[test]
    fn test_diff() {
        let x: DVector<f64> = vec![1.0, 2.0, 3.0].into();
        let exp: DVector<f64> = vec![1.0, 1.0].into();

        assert_eq!(x.diff(), exp);
    }

    #[test]
    fn test_set_slice() {
        let mut v: DVector<f64> = DVector::zeros(5);
        let s: DVector<f64> = vec![1.0, 2.0, 3.0].into();
        let exp: DVector<f64> = vec![0.0, 1.0, 2.0, 3.0, 0.0].into();
        v.set_slice(1, &s);
        assert_eq!(v, exp);
    }
}
