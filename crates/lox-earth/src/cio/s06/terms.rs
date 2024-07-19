/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_math::types::units::Radians;

pub(crate) const COEFFICIENTS: [f64; 6] = [
    94.00e-6,
    3808.65e-6,
    -122.68e-6,
    -72574.11e-6,
    27.98e-6,
    15.62e-6,
];

/// Coefficients of l, l', F, D, Ω, LVe, LE and pA.
pub(crate) type FundamentalArgCoeffs = [Radians; 8];

pub(crate) struct Term {
    pub fundamental_arg_coeffs: FundamentalArgCoeffs,
    pub sin_coeff: f64,
    pub cos_coeff: f64,
}

#[rustfmt::skip]
// @formatter:off (sometimes RustRover ignores the rustfmt skip)
pub(crate) const ZERO_ORDER: [Term; 33] = [
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  0.0,  0.0,  1.0,  0.0,  0.0,  0.0], sin_coeff: -2640.73e-6, cos_coeff:  0.39e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  0.0,  0.0,  2.0,  0.0,  0.0,  0.0], sin_coeff:   -63.53e-6, cos_coeff:  0.02e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0, -2.0,  3.0,  0.0,  0.0,  0.0], sin_coeff:   -11.75e-6, cos_coeff: -0.01e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0, -2.0,  1.0,  0.0,  0.0,  0.0], sin_coeff:   -11.21e-6, cos_coeff: -0.01e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0, -2.0,  2.0,  0.0,  0.0,  0.0], sin_coeff:     4.57e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0,  0.0,  3.0,  0.0,  0.0,  0.0], sin_coeff:    -2.02e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0,  0.0,  1.0,  0.0,  0.0,  0.0], sin_coeff:    -1.98e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  0.0,  0.0,  3.0,  0.0,  0.0,  0.0], sin_coeff:     1.72e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  1.0,  0.0,  0.0,  1.0,  0.0,  0.0,  0.0], sin_coeff:     1.41e-6, cos_coeff:  0.01e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  1.0,  0.0,  0.0, -1.0,  0.0,  0.0,  0.0], sin_coeff:     1.26e-6, cos_coeff:  0.01e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0,  0.0,  0.0, -1.0,  0.0,  0.0,  0.0], sin_coeff:     0.63e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0,  0.0,  0.0,  1.0,  0.0,  0.0,  0.0], sin_coeff:     0.63e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  1.0,  2.0, -2.0,  3.0,  0.0,  0.0,  0.0], sin_coeff:    -0.46e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  1.0,  2.0, -2.0,  1.0,  0.0,  0.0,  0.0], sin_coeff:    -0.45e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  4.0, -4.0,  4.0,  0.0,  0.0,  0.0], sin_coeff:    -0.36e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  1.0, -1.0,  1.0, -8.0, 12.0,  0.0], sin_coeff:     0.24e-6, cos_coeff:  0.12e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0,  0.0,  0.0,  0.0,  0.0,  0.0], sin_coeff:    -0.32e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0,  0.0,  2.0,  0.0,  0.0,  0.0], sin_coeff:    -0.28e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0,  2.0,  0.0,  3.0,  0.0,  0.0,  0.0], sin_coeff:    -0.27e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0,  2.0,  0.0,  1.0,  0.0,  0.0,  0.0], sin_coeff:    -0.26e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0, -2.0,  0.0,  0.0,  0.0,  0.0], sin_coeff:     0.21e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  1.0, -2.0,  2.0, -3.0,  0.0,  0.0,  0.0], sin_coeff:    -0.19e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  1.0, -2.0,  2.0, -1.0,  0.0,  0.0,  0.0], sin_coeff:    -0.18e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  0.0,  0.0,  0.0,  8.0,-13.0, -1.0], sin_coeff:     0.10e-6, cos_coeff: -0.05e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  0.0,  2.0,  0.0,  0.0,  0.0,  0.0], sin_coeff:    -0.15e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [2.0,  0.0, -2.0,  0.0, -1.0,  0.0,  0.0,  0.0], sin_coeff:     0.14e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  1.0,  2.0, -2.0,  2.0,  0.0,  0.0,  0.0], sin_coeff:     0.14e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0,  0.0, -2.0,  1.0,  0.0,  0.0,  0.0], sin_coeff:    -0.14e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0,  0.0, -2.0, -1.0,  0.0,  0.0,  0.0], sin_coeff:    -0.14e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  4.0, -2.0,  4.0,  0.0,  0.0,  0.0], sin_coeff:    -0.13e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0, -2.0,  4.0,  0.0,  0.0,  0.0], sin_coeff:     0.11e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0, -2.0,  0.0, -3.0,  0.0,  0.0,  0.0], sin_coeff:    -0.11e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0, -2.0,  0.0, -1.0,  0.0,  0.0,  0.0], sin_coeff:    -0.11e-6, cos_coeff:  0.00e-6 },
];

#[rustfmt::skip]
// @formatter:off (sometimes RustRover ignores the rustfmt skip)
pub(crate) const FIRST_ORDER: [Term; 3] = [
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  0.0,  0.0,  2.0,  0.0,  0.0,  0.0], sin_coeff: -0.07e-6, cos_coeff:  3.57e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  0.0,  0.0,  1.0,  0.0,  0.0,  0.0], sin_coeff:  1.73e-6, cos_coeff: -0.03e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0, -2.0,  3.0,  0.0,  0.0,  0.0], sin_coeff:  0.00e-6, cos_coeff:  0.48e-6 },
];

#[rustfmt::skip]
// @formatter:off (sometimes RustRover ignores the rustfmt skip)
pub(crate) const SECOND_ORDER: [Term; 25] = [
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  0.0,  0.0,  1.0,  0.0,  0.0,  0.0], sin_coeff: 743.52e-6, cos_coeff: -0.17e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0, -2.0,  2.0,  0.0,  0.0,  0.0], sin_coeff:  56.91e-6, cos_coeff:  0.06e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0,  0.0,  2.0,  0.0,  0.0,  0.0], sin_coeff:   9.84e-6, cos_coeff: -0.01e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  0.0,  0.0,  2.0,  0.0,  0.0,  0.0], sin_coeff:  -8.85e-6, cos_coeff:  0.01e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  1.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0], sin_coeff:  -6.38e-6, cos_coeff: -0.05e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0], sin_coeff:  -3.07e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  1.0,  2.0, -2.0,  2.0,  0.0,  0.0,  0.0], sin_coeff:   2.23e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0,  0.0,  1.0,  0.0,  0.0,  0.0], sin_coeff:   1.67e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0,  2.0,  0.0,  2.0,  0.0,  0.0,  0.0], sin_coeff:   1.30e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  1.0, -2.0,  2.0, -2.0,  0.0,  0.0,  0.0], sin_coeff:   0.93e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0,  0.0, -2.0,  0.0,  0.0,  0.0,  0.0], sin_coeff:   0.68e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0, -2.0,  1.0,  0.0,  0.0,  0.0], sin_coeff:  -0.55e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0, -2.0,  0.0, -2.0,  0.0,  0.0,  0.0], sin_coeff:   0.53e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  0.0,  2.0,  0.0,  0.0,  0.0,  0.0], sin_coeff:  -0.27e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0,  0.0,  0.0,  1.0,  0.0,  0.0,  0.0], sin_coeff:  -0.27e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0, -2.0, -2.0, -2.0,  0.0,  0.0,  0.0], sin_coeff:  -0.26e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0,  0.0,  0.0, -1.0,  0.0,  0.0,  0.0], sin_coeff:  -0.25e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0,  2.0,  0.0,  1.0,  0.0,  0.0,  0.0], sin_coeff:   0.22e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [2.0,  0.0,  0.0, -2.0,  0.0,  0.0,  0.0,  0.0], sin_coeff:  -0.21e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [2.0,  0.0, -2.0,  0.0, -1.0,  0.0,  0.0,  0.0], sin_coeff:   0.20e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0,  2.0,  2.0,  0.0,  0.0,  0.0], sin_coeff:   0.17e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [2.0,  0.0,  2.0,  0.0,  2.0,  0.0,  0.0,  0.0], sin_coeff:   0.13e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [2.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0], sin_coeff:  -0.13e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [1.0,  0.0,  2.0, -2.0,  2.0,  0.0,  0.0,  0.0], sin_coeff:  -0.12e-6, cos_coeff:  0.00e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0,  0.0,  0.0,  0.0,  0.0,  0.0], sin_coeff:  -0.11e-6, cos_coeff:  0.00e-6 },
];

#[rustfmt::skip]
// @formatter:off (sometimes RustRover ignores the rustfmt skip)
pub(crate) const THIRD_ORDER: [Term; 4] = [
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  0.0,  0.0,  1.0,  0.0,  0.0,  0.0], sin_coeff:  0.30e-6, cos_coeff: -23.42e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0, -2.0,  2.0,  0.0,  0.0,  0.0], sin_coeff: -0.03e-6, cos_coeff:  -1.46e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  2.0,  0.0,  2.0,  0.0,  0.0,  0.0], sin_coeff: -0.01e-6, cos_coeff:  -0.25e-6 },
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  0.0,  0.0,  2.0,  0.0,  0.0,  0.0], sin_coeff:  0.00e-6, cos_coeff:   0.23e-6 },
];

#[rustfmt::skip]
// @formatter:off (sometimes RustRover ignores the rustfmt skip)
pub(crate) const FOURTH_ORDER: [Term; 1] = [
    Term{ fundamental_arg_coeffs: [0.0,  0.0,  0.0,  0.0,  1.0,  0.0,  0.0,  0.0], sin_coeff: -0.26e-6, cos_coeff: -0.01e-6 }
];
