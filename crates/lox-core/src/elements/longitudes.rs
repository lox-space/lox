// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Conversions between the true, eccentric, and mean longitude of elliptic orbits.
//!
//! The longitudes are the anomalies offset by the longitude of periapsis ϖ, and the eccentricity
//! enters only through the components h = e·sin(ϖ) and k = e·cos(ϖ) of the eccentricity vector.
//! Unlike the anomalies, the longitudes are therefore well-defined for circular orbits.
//!
//! As in the [`anomalies`](crate::anomalies) module, the conversions do not normalise: the
//! result stays in the same revolution as the input.
//!
//! All functions require h² + k² < 1.

use crate::anomalies::{AnomalyError, solve_kepler};
use crate::math::float::{atan, sqrt};
use crate::units::Angle;

fn beta(h: f64, k: f64) -> f64 {
    1.0 / (1.0 + sqrt(1.0 - h * h - k * k))
}

/// Converts the true longitude L to the eccentric longitude F.
pub(crate) fn true_to_eccentric_longitude(true_longitude: Angle, h: f64, k: f64) -> Angle {
    let b = beta(h, k);
    let (sin_l, cos_l) = true_longitude.sin_cos();
    let delta = 2.0 * atan(b * (k * sin_l - h * cos_l) / (1.0 + b * (k * cos_l + h * sin_l)));
    Angle::new(true_longitude.as_f64() - delta)
}

/// Converts the eccentric longitude F to the true longitude L.
pub(crate) fn eccentric_to_true_longitude(eccentric_longitude: Angle, h: f64, k: f64) -> Angle {
    let b = beta(h, k);
    let (sin_f, cos_f) = eccentric_longitude.sin_cos();
    let delta = 2.0 * atan(b * (k * sin_f - h * cos_f) / (1.0 - b * (k * cos_f + h * sin_f)));
    Angle::new(eccentric_longitude.as_f64() + delta)
}

/// Converts the eccentric longitude F to the mean longitude λ = F - k·sin(F) + h·cos(F).
pub(crate) fn eccentric_to_mean_longitude(eccentric_longitude: Angle, h: f64, k: f64) -> Angle {
    let (sin_f, cos_f) = eccentric_longitude.sin_cos();
    Angle::new(eccentric_longitude.as_f64() - k * sin_f + h * cos_f)
}

/// Converts the mean longitude λ to the eccentric longitude F.
///
/// # Errors
///
/// Returns [`AnomalyError::ConvergenceFailure`] if Kepler's equation cannot be solved.
pub(crate) fn mean_to_eccentric_longitude(
    mean_longitude: Angle,
    h: f64,
    k: f64,
) -> Result<Angle, AnomalyError> {
    Ok(Angle::new(solve_kepler(mean_longitude.as_f64(), h, k)?))
}

/// Converts the mean longitude λ to the true longitude L.
///
/// # Errors
///
/// Returns [`AnomalyError::ConvergenceFailure`] if Kepler's equation cannot be solved.
pub(crate) fn mean_to_true_longitude(
    mean_longitude: Angle,
    h: f64,
    k: f64,
) -> Result<Angle, AnomalyError> {
    Ok(eccentric_to_true_longitude(
        mean_to_eccentric_longitude(mean_longitude, h, k)?,
        h,
        k,
    ))
}

/// Converts the true longitude L to the mean longitude λ.
pub(crate) fn true_to_mean_longitude(true_longitude: Angle, h: f64, k: f64) -> Angle {
    eccentric_to_mean_longitude(true_to_eccentric_longitude(true_longitude, h, k), h, k)
}

#[cfg(test)]
mod tests {
    use core::f64::consts::{PI, TAU};

    use lox_approx::assert_approx_eq;
    use proptest::prelude::*;
    use rstest::rstest;

    use crate::anomalies::TrueAnomaly;
    use crate::elements::Eccentricity;
    use crate::math::float::{abs, cos, sin};
    use crate::units::AngleUnits;

    use super::*;

    #[rstest]
    #[case(0.0, 0.0, 1.0)]
    #[case(1e-6, 0.3, -2.5)]
    #[case(0.1, 1.2, 0.4)]
    #[case(0.5, 4.0, 3.0)]
    #[case(0.9, 2.0, -1.0)]
    #[case(0.999, 5.5, 0.01)]
    fn test_longitudes_match_anomalies(#[case] e: f64, #[case] pomega: f64, #[case] nu: f64) {
        let h = e * sin(pomega);
        let k = e * cos(pomega);
        let ecc = Eccentricity::try_new(e).unwrap();
        let true_anomaly = TrueAnomaly::new(nu.rad());
        let eccentric_anomaly = true_anomaly.to_eccentric(ecc).unwrap().as_f64();
        let mean_anomaly = true_anomaly.to_mean(ecc).unwrap().as_f64();

        let true_longitude = Angle::new(nu + pomega);
        let eccentric_longitude = true_to_eccentric_longitude(true_longitude, h, k);
        let mean_longitude = true_to_mean_longitude(true_longitude, h, k);

        assert_approx_eq!(
            Angle::new(eccentric_longitude.as_f64() - pomega - eccentric_anomaly)
                .mod_two_pi_signed(),
            Angle::ZERO,
            atol <= 1e-12
        );
        assert_approx_eq!(
            Angle::new(mean_longitude.as_f64() - pomega - mean_anomaly).mod_two_pi_signed(),
            Angle::ZERO,
            atol <= 1e-12
        );
        assert_approx_eq!(
            eccentric_to_true_longitude(eccentric_longitude, h, k),
            true_longitude,
            atol <= 1e-12
        );
        assert_approx_eq!(
            mean_to_true_longitude(mean_longitude, h, k).unwrap(),
            true_longitude,
            atol <= 1e-10
        );
    }

    #[test]
    fn test_circular_longitudes_coincide() {
        let l = 2.0.rad();
        assert_eq!(true_to_eccentric_longitude(l, 0.0, 0.0), l);
        assert_eq!(true_to_mean_longitude(l, 0.0, 0.0), l);
        assert_eq!(mean_to_true_longitude(l, 0.0, 0.0).unwrap(), l);
    }

    #[test]
    fn test_longitudes_stay_in_revolution() {
        let (h, k) = (0.3, -0.4);
        for revolutions in [-3.0, 0.0, 5.0] {
            let l = Angle::new(1.0 + revolutions * TAU);
            let lambda = true_to_mean_longitude(l, h, k);
            assert!(abs(lambda.as_f64() - l.as_f64()) < PI);
            assert_approx_eq!(
                mean_to_true_longitude(lambda, h, k).unwrap(),
                l,
                atol <= 1e-10
            );
        }
    }

    #[test]
    fn test_mean_to_eccentric_longitude_converges() {
        for e in [0.0, 1e-12, 0.1, 0.5, 0.9, 0.99, 0.9999, 0.999999] {
            for step in 0..360 {
                let pomega = f64::from(step) * 0.0175 + 0.3;
                let (h, k) = (e * sin(pomega), e * cos(pomega));
                for revolution in [-2.0, 0.0, 3.0] {
                    let lambda = f64::from(step) * 0.0175 + revolution * TAU;
                    let f = mean_to_eccentric_longitude(Angle::new(lambda), h, k)
                        .unwrap()
                        .as_f64();
                    let residual = f - k * sin(f) + h * cos(f) - lambda;
                    let scale = if lambda.abs() > 1.0 {
                        lambda.abs()
                    } else {
                        1.0
                    };
                    assert!(
                        residual.abs() <= 8.0 * f64::EPSILON * scale,
                        "e = {e}, ϖ = {pomega}, λ = {lambda}: residual {residual}"
                    );
                }
            }
        }
    }

    /// Eccentricities of elliptic orbits, with extra weight on the near-circular and
    /// near-parabolic ends. Unlike the anomalies, which treat eccentricities within 1e-8 of 1 as
    /// parabolic, the longitudes accept any eccentricity below 1, so the upper bound is closer.
    fn elliptic_eccentricity() -> impl Strategy<Value = f64> {
        prop_oneof![
            Just(0.0),
            0.0..1e-8,
            1e-8..0.9,
            0.9..0.9999,
            0.9999..1.0 - 1e-12,
        ]
    }

    proptest! {
        #[test]
        fn prop_mean_to_eccentric_longitude_solves_keplers_equation(
            e in elliptic_eccentricity(),
            pomega in 0.0..TAU,
            lambda in -50.0..50.0f64,
        ) {
            let (h, k) = (e * sin(pomega), e * cos(pomega));
            let f = mean_to_eccentric_longitude(Angle::new(lambda), h, k)
                .unwrap()
                .as_f64();
            let tolerance = 8.0 * f64::EPSILON * abs(lambda).max(1.0);
            // The solution stays within e of the mean longitude, so in the same revolution.
            prop_assert!(abs(f - lambda) <= e + tolerance);
            let residual = eccentric_to_mean_longitude(Angle::new(f), h, k).as_f64() - lambda;
            prop_assert!(abs(residual) <= tolerance, "residual: {residual}");
        }

        // The true longitude changes slowly with the mean longitude near apoapsis, which amplifies
        // rounding errors in the true longitude by up to (1 + e)² / (1 - e²)^(3/2) on the way back,
        // so the round trip is limited to moderately eccentric orbits.
        #[test]
        fn prop_mean_to_true_longitude_round_trip(
            e in 0.0..0.99,
            pomega in 0.0..TAU,
            lambda in -50.0..50.0f64,
        ) {
            let (h, k) = (e * sin(pomega), e * cos(pomega));
            let true_longitude = mean_to_true_longitude(Angle::new(lambda), h, k).unwrap();
            prop_assert!(abs(true_longitude.as_f64() - lambda) < PI);
            let mean_longitude = true_to_mean_longitude(true_longitude, h, k).as_f64();
            prop_assert!(abs(mean_longitude - lambda) <= 1e-10, "error: {}", mean_longitude - lambda);
        }
    }
}
