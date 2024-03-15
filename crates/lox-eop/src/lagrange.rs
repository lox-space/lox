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

/// Perform Lagrangian interpolation within a set of (x, y) pairs, returning the y-value
/// corresponding to `target_x`.
fn interpolate(x: &[f64], y: &[f64], target_x: f64) -> f64 {
    debug_assert_eq!(x.len(), y.len()); // upstream functions should ensure this invariant

    let mut result = 0.0;
    let mut k = 0usize;
    for i in 0..(x.len() - 1) {
        if target_x >= x[i] && target_x < x[i + 1] {
            k = i;
            break;
        }
    }

    if k < 1 {
        k = 1;
    }
    if k > x.len() - 3 {
        k = x.len() - 3;
    }

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
