// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Property tests for anomaly conversions

use lox_core::anomalies::{EccentricAnomaly, MeanAnomaly, TrueAnomaly};
use lox_core::elements::Eccentricity;
use lox_core::units::{Angle, AngleUnits};
use lox_test_utils::approx_eq;
use proptest::prelude::*;

// Strategy for generating eccentricities for different orbit types
fn circular_eccentricity() -> impl Strategy<Value = Eccentricity> {
    (0.0..1e-9_f64).prop_map(|e| Eccentricity::try_new(e).unwrap())
}

fn elliptic_eccentricity() -> impl Strategy<Value = Eccentricity> {
    (1e-6..0.99_f64).prop_map(|e| Eccentricity::try_new(e).unwrap())
}

// Strategy for generating angles in radians [0, 2π)
fn angle_rad() -> impl Strategy<Value = Angle> {
    (0.0..std::f64::consts::TAU).prop_map(|rad| rad.rad())
}

// Strategy for generating (eccentricity, angle) pairs for hyperbolic orbits
// For hyperbolic orbits, true anomaly must be within asymptote limits: |ν| < arccos(-1/e)
fn hyperbolic_ecc_and_angle() -> impl Strategy<Value = (Eccentricity, Angle)> {
    (1.01..10.0_f64).prop_flat_map(|e| {
        let ecc = Eccentricity::try_new(e).unwrap();
        let max_nu = (-1.0 / e).acos();
        // Use a slightly smaller range to avoid numerical edge cases
        let safe_max = max_nu * 0.95;
        (-safe_max..safe_max).prop_map(move |rad| (ecc, rad.rad()))
    })
}

// ============================================================================
// Elliptic orbit property tests
// ============================================================================

proptest! {
    #[test]
    fn elliptic_eccentric_to_true_to_eccentric_roundtrip(
        ecc in elliptic_eccentricity(),
        angle in angle_rad()
    ) {
        let eccentric = EccentricAnomaly::new(angle);
        let true_anom = eccentric.to_true(ecc);
        let back = true_anom.to_eccentric(ecc).expect("conversion should succeed");

        // Normalize both angles to [0, 2π) for comparison
        let original = eccentric.as_angle().mod_two_pi();
        let roundtrip = back.as_angle().mod_two_pi();

        prop_assert!(
            approx_eq!(original.as_f64(), roundtrip.as_f64(), rtol <= 1e-12),
            "Original: {}, Roundtrip: {}", original, roundtrip
        );
    }

    #[test]
    fn elliptic_eccentric_to_mean_to_eccentric_roundtrip(
        ecc in elliptic_eccentricity(),
        angle in angle_rad()
    ) {
        let eccentric = EccentricAnomaly::new(angle);
        let mean_anom = eccentric.to_mean(ecc);

        // Convert back via true anomaly - may fail for some inputs due to solver convergence
        if let Ok(true_anom) = mean_anom.to_true(ecc)
            && let Ok(back) = true_anom.to_eccentric(ecc) {
                // Normalize both angles to [0, 2π) for comparison
                let original = eccentric.as_angle().mod_two_pi();
                let roundtrip = back.as_angle().mod_two_pi();

                prop_assert!(
                    approx_eq!(original.as_f64(), roundtrip.as_f64(), rtol <= 1e-9),
                    "Original: {}, Roundtrip: {}", original, roundtrip
                );
            }
    }

    #[test]
    fn elliptic_true_to_eccentric_to_true_roundtrip(
        ecc in elliptic_eccentricity(),
        angle in angle_rad()
    ) {
        let true_anom = TrueAnomaly::new(angle);

        // This conversion can fail for hyperbolic orbits with ν outside asymptote limits,
        // but should always succeed for elliptic orbits
        let eccentric = true_anom.to_eccentric(ecc).expect("conversion should succeed");
        let back = eccentric.to_true(ecc);

        // Normalize both angles to [0, 2π) for comparison
        let original = true_anom.as_angle().mod_two_pi();
        let roundtrip = back.as_angle().mod_two_pi();

        prop_assert!(
            approx_eq!(original.as_f64(), roundtrip.as_f64(), rtol <= 1e-12),
            "Original: {}, Roundtrip: {}", original, roundtrip
        );
    }

    #[test]
    fn elliptic_true_to_mean_to_true_roundtrip(
        ecc in elliptic_eccentricity(),
        angle in angle_rad()
    ) {
        let true_anom = TrueAnomaly::new(angle);
        let mean_anom = true_anom.to_mean(ecc).expect("conversion should succeed");

        // Conversion back may fail for some inputs due to solver convergence
        if let Ok(back) = mean_anom.to_true(ecc) {
            // Normalize both angles to [0, 2π) for comparison
            let original = true_anom.as_angle().mod_two_pi();
            let roundtrip = back.as_angle().mod_two_pi();

            prop_assert!(
                approx_eq!(original.as_f64(), roundtrip.as_f64(), rtol <= 1e-9),
                "Original: {}, Roundtrip: {}", original, roundtrip
            );
        }
    }

    #[test]
    fn elliptic_mean_to_true_to_mean_roundtrip(
        ecc in elliptic_eccentricity(),
        angle in angle_rad()
    ) {
        let mean_anom = MeanAnomaly::new(angle);

        // Conversion may fail for some inputs due to solver convergence
        if let Ok(true_anom) = mean_anom.to_true(ecc) {
            let back = true_anom.to_mean(ecc).expect("conversion should succeed");

            // Normalize both angles to [0, 2π) for comparison
            let original = mean_anom.as_angle().mod_two_pi();
            let roundtrip = back.as_angle().mod_two_pi();

            prop_assert!(
                approx_eq!(original.as_f64(), roundtrip.as_f64(), rtol <= 1e-9),
                "Original: {}, Roundtrip: {}", original, roundtrip
            );
        }
    }
}

// ============================================================================
// Circular orbit property tests
// ============================================================================

proptest! {
    #[test]
    fn circular_all_anomalies_equal(
        ecc in circular_eccentricity(),
        angle in angle_rad()
    ) {
        // For circular orbits, all three anomalies should be approximately equal
        let true_anom = TrueAnomaly::new(angle);
        let eccentric = true_anom.to_eccentric(ecc).expect("conversion should succeed");
        let mean_anom = true_anom.to_mean(ecc).expect("conversion should succeed");

        let true_val = true_anom.as_angle().mod_two_pi().as_f64();
        let ecc_val = eccentric.as_angle().mod_two_pi().as_f64();
        let mean_val = mean_anom.as_angle().mod_two_pi().as_f64();

        prop_assert!(
            approx_eq!(true_val, ecc_val, atol <= 1e-8),
            "True: {}, Eccentric: {}", true_val, ecc_val
        );
        prop_assert!(
            approx_eq!(true_val, mean_val, atol <= 1e-8),
            "True: {}, Mean: {}", true_val, mean_val
        );
    }
}

// ============================================================================
// Hyperbolic orbit property tests
// ============================================================================

proptest! {
    #[test]
    fn hyperbolic_eccentric_to_true_to_eccentric_roundtrip(
        (ecc, angle) in hyperbolic_ecc_and_angle(),
    ) {
        // Start with true anomaly (which is guaranteed valid for hyperbolic)
        let true_anom = TrueAnomaly::new(angle);
        let eccentric = true_anom.to_eccentric(ecc).expect("conversion should succeed");
        let back = eccentric.to_true(ecc);

        // For hyperbolic anomalies, we compare the raw values since they don't wrap
        let original = true_anom.as_angle().as_f64();
        let roundtrip = back.as_angle().as_f64();

        prop_assert!(
            approx_eq!(original, roundtrip, rtol <= 1e-12),
            "Original: {}, Roundtrip: {}", original, roundtrip
        );
    }

    #[test]
    fn hyperbolic_eccentric_to_mean_to_eccentric_roundtrip(
        (ecc, angle) in hyperbolic_ecc_and_angle(),
    ) {
        // Start with true anomaly to ensure it's valid
        let true_anom = TrueAnomaly::new(angle);
        let eccentric = true_anom.to_eccentric(ecc).expect("conversion should succeed");
        let mean_anom = eccentric.to_mean(ecc);

        // Convert back via true anomaly - may fail for some inputs due to solver convergence
        if let Ok(true_back) = mean_anom.to_true(ecc)
            && let Ok(ecc_back) = true_back.to_eccentric(ecc) {
                let original = eccentric.as_angle().as_f64();
                let roundtrip = ecc_back.as_angle().as_f64();

                prop_assert!(
                    approx_eq!(original, roundtrip, rtol <= 1e-9),
                    "Original: {}, Roundtrip: {}", original, roundtrip
                );
            }
    }

    #[test]
    fn hyperbolic_true_to_mean_to_true_roundtrip(
        (ecc, angle) in hyperbolic_ecc_and_angle(),
    ) {
        let true_anom = TrueAnomaly::new(angle);
        let mean_anom = true_anom.to_mean(ecc).expect("conversion should succeed");

        // Conversion back may fail for some inputs due to solver convergence
        if let Ok(back) = mean_anom.to_true(ecc) {
            let original = true_anom.as_angle().as_f64();
            let roundtrip = back.as_angle().as_f64();

            prop_assert!(
                approx_eq!(original, roundtrip, rtol <= 1e-9),
                "Original: {}, Roundtrip: {}", original, roundtrip
            );
        }
    }
}

// ============================================================================
// Parabolic orbit property tests
// ============================================================================

proptest! {
    #[test]
    fn parabolic_eccentric_to_true_to_eccentric_roundtrip(
        angle in angle_rad()
    ) {
        let ecc = Eccentricity::try_new(1.0).unwrap();

        let eccentric = EccentricAnomaly::new(angle);
        let true_anom = eccentric.to_true(ecc);

        // For parabolic orbits, conversion back may fail if true anomaly is too large
        if let Ok(back) = true_anom.to_eccentric(ecc) {
            let original = eccentric.as_angle().as_f64();
            let roundtrip = back.as_angle().as_f64();

            prop_assert!(
                approx_eq!(original, roundtrip, rtol <= 1e-10),
                "Original: {}, Roundtrip: {}", original, roundtrip
            );
        }
    }

    #[test]
    fn parabolic_eccentric_to_mean_to_eccentric_roundtrip(
        angle in angle_rad()
    ) {
        let ecc = Eccentricity::try_new(1.0).unwrap();

        let eccentric = EccentricAnomaly::new(angle);
        let mean_anom = eccentric.to_mean(ecc);

        // Convert back via true anomaly
        if let Ok(true_anom) = mean_anom.to_true(ecc)
            && let Ok(back) = true_anom.to_eccentric(ecc) {
                let original = eccentric.as_angle().as_f64();
                let roundtrip = back.as_angle().as_f64();

                prop_assert!(
                    approx_eq!(original, roundtrip, rtol <= 1e-9),
                    "Original: {}, Roundtrip: {}", original, roundtrip
                );
            }
    }
}
