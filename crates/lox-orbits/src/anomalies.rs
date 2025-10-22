// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

pub fn hyperbolic_to_true(hyperbolic_anomaly: f64, eccentricity: f64) -> f64 {
    2.0 * (((1.0 + eccentricity) / (eccentricity - 1.0)).sqrt() * (hyperbolic_anomaly / 2.0).tanh())
        .atan()
}

pub fn eccentric_to_true(eccentric_anomaly: f64, eccentricity: f64) -> f64 {
    2.0 * (((1.0 + eccentricity) / (1.0 - eccentricity)).sqrt() * (eccentric_anomaly / 2.0).tan())
        .atan()
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_hyperbolic() {
        assert_approx_eq!(
            hyperbolic_to_true(PI / 2.0, 1.2),
            2.2797028138935547,
            rtol <= 1e-8
        );
    }

    #[test]
    fn test_eccentric() {
        assert_approx_eq!(
            eccentric_to_true(PI / 2.0, 0.2),
            1.7721542475852272,
            rtol <= 1e-8
        );
    }
}
