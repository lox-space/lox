/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO: Remove this once Lagrangian interpolation is used by lox-eop public interface.
#![allow(dead_code)]

pub mod eop;

/// The size of the window used for Lagrangian interpolation.
pub const WINDOW_SIZE: usize = 4;

/// Perform Lagrangian interpolation within a set of (x, y) pairs, returning the y-value
/// corresponding to `target_x`.
fn interpolate(x: &[f64], y: &[f64], target_x: f64) -> f64 {
    // Upstream functions should ensure these invariants.
    debug_assert!(x.len() >= 4); // there must be at least as many elements as the window size
    debug_assert_eq!(x.len(), y.len());

    let mut result = 0.0;
    let mut k = 0usize;
    for i in 0..(x.len() - 1) {
        if target_x >= x[i] && target_x < x[i + 1] {
            k = i;
        }
    }

    // The minimum value of k is 1.
    if k < 1 {
        k = 1;
    }

    // The maximum value of k is the third-to-last index.
    let max_k = x.len() as isize - 3;
    if max_k > 0 && k as isize > max_k {
        k = max_k as usize;
    }

    // Iterate over a four-element window around k.
    for m in (k - 1)..(k + 3) {
        let mut term = y[m];
        for j in (k - 1)..(k + 3) {
            if m != j {
                term *= (target_x - x[j]) / (x[m] - x[j]);
            }
        }
        result += term;
    }

    result
}
