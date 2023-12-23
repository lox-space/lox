/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::MAX_POWER_OF_T;

pub(crate) struct PolynomialCoefficients {
    pub x: [f64; MAX_POWER_OF_T + 1],
    pub y: [f64; MAX_POWER_OF_T + 1],
}

pub(crate) const COEFFICIENTS: PolynomialCoefficients = PolynomialCoefficients {
    x: [
        -0.016617,
        2004.191898,
        -0.4297829,
        -0.19861834,
        0.000007578,
        0.0000059285,
    ],
    y: [
        -0.006951,
        -0.025896,
        -22.4072747,
        0.00190059,
        0.001112526,
        0.0000001358,
    ],
};
