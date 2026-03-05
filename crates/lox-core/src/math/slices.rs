// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Utility traits for working with slices of numbers.

use std::ops::Sub;

/// Computes consecutive differences of a slice.
pub trait Diff<T> {
    /// Returns a vector of differences between consecutive elements.
    fn diff(&self) -> Vec<T>;
}

impl<T> Diff<T> for [T]
where
    T: Copy + Sub<Output = T>,
{
    fn diff(&self) -> Vec<T> {
        let n = self.len();
        self.iter()
            .take(n - 1)
            .enumerate()
            .map(|(idx, x)| self[idx + 1] - *x)
            .collect()
    }
}

/// Tests whether a slice is monotonically ordered.
pub trait Monotonic {
    /// Returns `true` if elements are non-decreasing.
    fn is_increasing(&self) -> bool;
    /// Returns `true` if elements are non-increasing.
    fn is_decreasing(&self) -> bool;
    /// Returns `true` if each element is strictly greater than the previous.
    fn is_strictly_increasing(&self) -> bool;
    /// Returns `true` if each element is strictly less than the previous.
    fn is_strictly_decreasing(&self) -> bool;
}

impl<T> Monotonic for [T]
where
    T: PartialOrd,
{
    fn is_increasing(&self) -> bool {
        self.as_ref().iter().is_sorted()
    }

    fn is_decreasing(&self) -> bool {
        self.as_ref().iter().is_sorted_by(|a, b| a >= b)
    }

    fn is_strictly_increasing(&self) -> bool {
        self.as_ref().iter().is_sorted_by(|a, b| a < b)
    }

    fn is_strictly_decreasing(&self) -> bool {
        self.as_ref().iter().is_sorted_by(|a, b| a > b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff() {
        let x: Vec<f64> = vec![1.0, 2.0, 3.0];
        let exp: Vec<f64> = vec![1.0, 1.0];
        assert_eq!(x.diff(), exp);
    }

    #[test]
    fn test_monotonic() {
        let x1: Vec<f64> = vec![1.0, 1.0, 3.0];
        let x2: Vec<f64> = vec![1.0, 2.0, 3.0];
        let x3: Vec<f64> = vec![3.0, 2.0, 2.0];
        let x4: Vec<f64> = vec![3.0, 2.0, 1.0];
        assert!(x1.is_increasing());
        assert!(!x1.is_strictly_increasing());
        assert!(x2.is_increasing());
        assert!(x2.is_strictly_increasing());
        assert!(x3.is_decreasing());
        assert!(!x3.is_strictly_decreasing());
        assert!(x4.is_decreasing());
        assert!(x4.is_strictly_decreasing());
    }
}
