/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Mod `slices` provides utility functions for working with slices.

/// Returns true if the slice contents are sorted in ascending order, and false otherwise.
pub fn is_sorted_asc<T: Ord>(slice: &[T]) -> bool {
    slice.windows(2).all(|x| x[0] <= x[1])
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(&[1, 2, 3, 4, 5], true)]
    #[case(&[1, 2, 3, 5, 4], false)]
    #[case(&[1, 1, 1, 1, 1], true)]
    #[case(&[5, 4, 3, 2, 1], false)]
    fn test_is_sorted(#[case] slice: &[i32], #[case] expected: bool) {
        assert_eq!(is_sorted_asc(slice), expected);
    }
}
