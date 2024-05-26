/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub trait IsClose {
    const DEFAULT_RELATIVE: f64;
    const DEFAULT_ABSOLUTE: f64;

    fn is_close_with_tolerances(&self, rhs: &Self, rel_tol: f64, abs_tol: f64) -> bool;

    fn is_close_abs(&self, rhs: &Self, abs_tol: f64) -> bool {
        self.is_close_with_tolerances(rhs, Self::DEFAULT_RELATIVE, abs_tol)
    }

    fn is_close_rel(&self, rhs: &Self, rel_tol: f64) -> bool {
        self.is_close_with_tolerances(rhs, rel_tol, Self::DEFAULT_ABSOLUTE)
    }

    fn is_close(&self, rhs: &Self) -> bool {
        self.is_close_with_tolerances(rhs, Self::DEFAULT_RELATIVE, Self::DEFAULT_ABSOLUTE)
    }
}

impl IsClose for f64 {
    const DEFAULT_RELATIVE: f64 = 1e-8;

    const DEFAULT_ABSOLUTE: f64 = 0.0;

    fn is_close_with_tolerances(&self, rhs: &Self, rel_tol: f64, abs_tol: f64) -> bool {
        (self - rhs).abs() <= f64::max(rel_tol * f64::max(self.abs(), rhs.abs()), abs_tol)
    }
}

#[macro_export]
macro_rules! assert_close {
    ($lhs:expr, $rhs:expr) => {
        assert!($lhs.is_close(&$rhs), "{:?} ≉ {:?}", $lhs, $rhs);
    };
    ($lhs:expr, $rhs:expr, $rel_tol:expr) => {
        assert!(
            $lhs.is_close_rel(&$rhs, $rel_tol),
            "{:?} ≉ {:?}",
            $lhs,
            $rhs
        );
    };
    ($lhs:expr, $rhs:expr, $rel_tol:expr, $abs_tol:expr) => {
        assert!(
            $lhs.is_close_with_tolerances(&$rhs, $rel_tol, $abs_tol),
            "{:?} ≉ {:?}",
            $lhs,
            $rhs
        );
    };
}
