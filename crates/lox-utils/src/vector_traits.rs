/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub trait Diff {
    fn diff(&self) -> Vec<f64>;
}

impl Diff for [f64] {
    fn diff(&self) -> Vec<f64> {
        let n = self.len();
        self[0..n - 1]
            .iter()
            .enumerate()
            .map(|(idx, x)| self[idx + 1] - x)
            .collect()
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
}
