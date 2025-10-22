// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

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
