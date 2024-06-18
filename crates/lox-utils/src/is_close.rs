/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use glam::DVec3;

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

impl IsClose for DVec3 {
    const DEFAULT_RELATIVE: f64 = 1e-8;
    const DEFAULT_ABSOLUTE: f64 = 0.0;

    fn is_close_with_tolerances(&self, rhs: &Self, rel_tol: f64, abs_tol: f64) -> bool {
        self.x.is_close_with_tolerances(&rhs.x, rel_tol, abs_tol)
            && self.y.is_close_with_tolerances(&rhs.y, rel_tol, abs_tol)
            && self.z.is_close_with_tolerances(&rhs.z, rel_tol, abs_tol)
    }
}

#[macro_export]
macro_rules! assert_close {
    ($lhs:expr, $rhs:expr) => {
        assert!($lhs.is_close(&$rhs), "{:?} ≉ {:?}", $lhs, $rhs);
    };
    ($lhs:expr, $rhs:expr, $abs_tol:expr) => {
        assert!(
            $lhs.is_close_abs(&$rhs, $abs_tol),
            "{:?} ≉ {:?}",
            $lhs,
            $rhs
        );
    };
    ($lhs:expr, $rhs:expr, $abs_tol:expr, $rel_tol:expr) => {
        assert!(
            $lhs.is_close_with_tolerances(&$rhs, $rel_tol, $abs_tol),
            "{:?} ≉ {:?}",
            $lhs,
            $rhs
        );
    };
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(1.0, 1.0 + f64::EPSILON, true)]
    #[case(0.0, 0.0 + f64::EPSILON, false)]
    fn test_is_close_f64(#[case] a: f64, #[case] b: f64, #[case] expected: bool) {
        assert_eq!(a.is_close(&b), expected);
    }

    #[rstest]
    #[case(1.0, 1.0 + f64::EPSILON, 0.0, true)]
    #[case(0.0, 0.0 + f64::EPSILON, 2.0 * f64::EPSILON, true)]
    fn test_is_close_f64_abs(
        #[case] a: f64,
        #[case] b: f64,
        #[case] abs_tol: f64,
        #[case] expected: bool,
    ) {
        assert_eq!(a.is_close_abs(&b, abs_tol), expected);
    }

    #[rstest]
    #[case(1.0, 1.0 + f64::EPSILON, 0.0, false)]
    #[case(0.0, 0.0 + f64::EPSILON, 2.0 * f64::EPSILON, false)]
    fn test_is_close_f64_rel(
        #[case] a: f64,
        #[case] b: f64,
        #[case] rel_tol: f64,
        #[case] expected: bool,
    ) {
        assert_eq!(a.is_close_rel(&b, rel_tol), expected);
    }

    #[test]
    fn test_assert_close() {
        assert_close!(1.0, 1.0 + f64::EPSILON);
        assert_close!(0.0, 0.0 + f64::EPSILON, 2.0 * f64::EPSILON);
        assert_close!(0.0, 0.0 + f64::EPSILON, 2.0 * f64::EPSILON, 0.0);
    }
}
