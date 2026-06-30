// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Power Flux Density (PFD) calculations and ITU regulatory masks.

use std::f64::consts::PI;

use lox_core::units::{Angle, Decibel, Distance, Frequency};
use thiserror::Error;

/// Computes the power flux density in dBW/m²/ref_bw.
///
/// PFD = 10·log₁₀(EIRP_linear / (4π·d²) · (ref_bw / occupied_bw))
pub fn power_flux_density(
    eirp: Decibel,
    distance: Distance,
    occupied_bw: Frequency,
    reference_bw: Frequency,
) -> Decibel {
    let eirp_w = eirp.to_linear();
    let d_m = distance.to_meters();
    let pfd_linear =
        eirp_w / (4.0 * PI * d_m * d_m) * (reference_bw.to_hertz() / occupied_bw.to_hertz());
    Decibel::from_linear(pfd_linear)
}

/// A piecewise-linear PFD mask over elevation in dBW/m²/ref_bw.
///
/// ITU Radio Regulations Article 21 specifies PFD limits as piecewise-linear
/// functions of the arrival angle, with band-dependent breakpoints and slopes.
/// The mask is linear in elevation between consecutive breakpoints and constant
/// below the first and above the last.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "PfdMaskRepr", into = "PfdMaskRepr")
)]
pub struct PfdMask {
    /// `(elevation breakpoint, mask value)` pairs with strictly ascending elevations.
    nodes: Vec<(Angle, Decibel)>,
}

/// Serde wire format for [`PfdMask`]: forces deserialization through the
/// validated constructor so the ascending-breakpoint invariant holds.
#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
struct PfdMaskRepr {
    nodes: Vec<(Angle, Decibel)>,
}

#[cfg(feature = "serde")]
impl From<PfdMask> for PfdMaskRepr {
    fn from(mask: PfdMask) -> Self {
        Self { nodes: mask.nodes }
    }
}

#[cfg(feature = "serde")]
impl TryFrom<PfdMaskRepr> for PfdMask {
    type Error = PfdMaskError;

    fn try_from(repr: PfdMaskRepr) -> Result<Self, Self::Error> {
        PfdMask::new(repr.nodes)
    }
}

impl PfdMask {
    /// Creates a mask from `(elevation breakpoint, mask value)` nodes.
    ///
    /// Requires at least two nodes with strictly ascending, finite elevations
    /// and finite mask values.
    pub fn new(nodes: Vec<(Angle, Decibel)>) -> Result<Self, PfdMaskError> {
        if nodes.len() < 2 {
            return Err(PfdMaskError::TooFewNodes(nodes.len()));
        }
        for (elevation, value) in &nodes {
            if !elevation.to_radians().is_finite() || !value.as_f64().is_finite() {
                return Err(PfdMaskError::NonFiniteNode(*elevation, *value));
            }
        }
        for pair in nodes.windows(2) {
            if pair[1].0.to_radians() <= pair[0].0.to_radians() {
                return Err(PfdMaskError::UnsortedBreakpoints(pair[0].0, pair[1].0));
            }
        }
        Ok(Self { nodes })
    }

    /// The ITU RR Article 21.16 mask shape for a given low-elevation limit.
    ///
    /// - θ < 5°: `start`
    /// - 5° ≤ θ < 25°: `start + 0.5 · (θ − 5)`
    /// - θ ≥ 25°: `start + 10 dB`
    pub fn art_21_16(start: Decibel) -> Self {
        Self::new(vec![
            (Angle::degrees(5.0), start),
            (Angle::degrees(25.0), start + Decibel::new(10.0)),
        ])
        .expect("two ascending breakpoints are always valid")
    }

    /// Returns the mask value at the given elevation angle.
    pub fn value_at(&self, elevation: Angle) -> Decibel {
        let el = elevation.to_radians();
        let (first, last) = (self.nodes[0], self.nodes[self.nodes.len() - 1]);
        if el <= first.0.to_radians() {
            return first.1;
        }
        if el >= last.0.to_radians() {
            return last.1;
        }
        for pair in self.nodes.windows(2) {
            let (el_lo, val_lo) = pair[0];
            let (el_hi, val_hi) = pair[1];
            if el < el_hi.to_radians() {
                let t = (el - el_lo.to_radians()) / (el_hi.to_radians() - el_lo.to_radians());
                return Decibel::new(val_lo.as_f64() + t * (val_hi.as_f64() - val_lo.as_f64()));
            }
        }
        last.1
    }

    /// Returns the mask nodes.
    pub fn nodes(&self) -> &[(Angle, Decibel)] {
        &self.nodes
    }
}

/// Errors produced while constructing a [`PfdMask`].
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum PfdMaskError {
    /// A mask needs at least two breakpoints.
    #[error("PFD mask requires at least 2 nodes, got {0}")]
    TooFewNodes(usize),
    /// A breakpoint or mask value is not finite.
    #[error("PFD mask node ({} deg, {} dB) must be finite", .0.to_degrees(), .1.as_f64())]
    NonFiniteNode(Angle, Decibel),
    /// Breakpoint elevations must be strictly ascending.
    #[error(
        "PFD mask breakpoints must be strictly ascending, got {} deg before {} deg",
        .0.to_degrees(),
        .1.to_degrees()
    )]
    UnsortedBreakpoints(Angle, Angle),
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;
    use lox_core::units::{DecibelUnits, FrequencyUnits};

    use super::*;

    fn art_21_16_mask() -> PfdMask {
        PfdMask::art_21_16((-154.0).db())
    }

    #[test]
    fn test_pfd_isotropic() {
        // 0 dBW EIRP, 1000 km, equal bandwidths
        // PFD = 1 / (4*pi*(1e6)^2) = 7.958e-14 → -131.0 dBW/m²
        let pfd = power_flux_density(0.0.db(), Distance::kilometers(1000.0), 1.0.mhz(), 1.0.mhz());
        let expected = 10.0 * (1.0 / (4.0 * std::f64::consts::PI * 1e12)).log10();
        assert_approx_eq!(pfd.as_f64(), expected, atol <= 0.01);
    }

    #[test]
    fn test_pfd_bandwidth_scaling() {
        // ref_bw < occupied_bw should reduce PFD
        let pfd_wide = power_flux_density(
            10.0.db(),
            Distance::kilometers(500.0),
            10.0.mhz(),
            1.0.mhz(),
        );
        let pfd_equal =
            power_flux_density(10.0.db(), Distance::kilometers(500.0), 1.0.mhz(), 1.0.mhz());
        // 10 dB difference (10 MHz vs 1 MHz)
        assert_approx_eq!((pfd_equal - pfd_wide).as_f64(), 10.0, atol <= 0.01);
    }

    #[test]
    fn test_pfd_mask_low_elevation() {
        let mask = art_21_16_mask();
        assert_approx_eq!(
            mask.value_at(Angle::degrees(0.0)).as_f64(),
            -154.0,
            atol <= 1e-10
        );
        assert_approx_eq!(
            mask.value_at(Angle::degrees(4.9)).as_f64(),
            -154.0,
            atol <= 1e-10
        );
    }

    #[test]
    fn test_pfd_mask_transition() {
        // At 15°: -154 + 0.5*(15-5) = -154 + 5 = -149
        let mask = art_21_16_mask();
        assert_approx_eq!(
            mask.value_at(Angle::degrees(15.0)).as_f64(),
            -149.0,
            atol <= 1e-10
        );
    }

    #[test]
    fn test_pfd_mask_high_elevation() {
        let mask = art_21_16_mask();
        assert_approx_eq!(
            mask.value_at(Angle::degrees(25.0)).as_f64(),
            -144.0,
            atol <= 1e-10
        );
        assert_approx_eq!(
            mask.value_at(Angle::degrees(90.0)).as_f64(),
            -144.0,
            atol <= 1e-10
        );
    }

    #[test]
    fn test_pfd_mask_boundary_at_5_degrees() {
        // Exactly at 5°: start of the transition zone
        let mask = art_21_16_mask();
        assert_approx_eq!(
            mask.value_at(Angle::degrees(5.0)).as_f64(),
            -154.0,
            atol <= 1e-10
        );
    }

    #[test]
    fn test_pfd_mask_three_segments() {
        // Mask with two different slopes: 0.5 dB/deg then 0.25 dB/deg
        let mask = PfdMask::new(vec![
            (Angle::degrees(5.0), (-150.0).db()),
            (Angle::degrees(15.0), (-145.0).db()),
            (Angle::degrees(25.0), (-142.5).db()),
        ])
        .unwrap();
        assert_approx_eq!(
            mask.value_at(Angle::degrees(0.0)).as_f64(),
            -150.0,
            atol <= 1e-10
        );
        assert_approx_eq!(
            mask.value_at(Angle::degrees(10.0)).as_f64(),
            -147.5,
            atol <= 1e-10
        );
        assert_approx_eq!(
            mask.value_at(Angle::degrees(20.0)).as_f64(),
            -143.75,
            atol <= 1e-10
        );
        assert_approx_eq!(
            mask.value_at(Angle::degrees(60.0)).as_f64(),
            -142.5,
            atol <= 1e-10
        );
    }

    #[test]
    fn test_pfd_mask_nodes_accessor() {
        let mask = art_21_16_mask();
        assert_eq!(mask.nodes().len(), 2);
        assert_approx_eq!(mask.nodes()[0].0.to_degrees(), 5.0, atol <= 1e-12);
        assert_approx_eq!(mask.nodes()[1].1.as_f64(), -144.0, atol <= 1e-12);
    }

    #[test]
    fn test_pfd_mask_rejects_too_few_nodes() {
        let err = PfdMask::new(vec![(Angle::degrees(5.0), (-150.0).db())]).unwrap_err();
        assert_eq!(err, PfdMaskError::TooFewNodes(1));
        assert!(err.to_string().contains("at least 2"));
    }

    #[test]
    fn test_pfd_mask_rejects_unsorted_breakpoints() {
        let err = PfdMask::new(vec![
            (Angle::degrees(25.0), (-144.0).db()),
            (Angle::degrees(5.0), (-154.0).db()),
        ])
        .unwrap_err();
        assert!(matches!(err, PfdMaskError::UnsortedBreakpoints(_, _)));
        assert!(err.to_string().contains("ascending"));
    }

    #[test]
    fn test_pfd_mask_rejects_duplicate_breakpoints() {
        let err = PfdMask::new(vec![
            (Angle::degrees(5.0), (-154.0).db()),
            (Angle::degrees(5.0), (-144.0).db()),
        ])
        .unwrap_err();
        assert!(matches!(err, PfdMaskError::UnsortedBreakpoints(_, _)));
    }

    #[test]
    fn test_pfd_mask_rejects_non_finite_value() {
        let err = PfdMask::new(vec![
            (Angle::degrees(5.0), Decibel::new(f64::NAN)),
            (Angle::degrees(25.0), (-144.0).db()),
        ])
        .unwrap_err();
        assert!(matches!(err, PfdMaskError::NonFiniteNode(_, _)));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_pfd_mask_serde_round_trip() {
        let mask = art_21_16_mask();
        let json = serde_json::to_string(&mask).unwrap();
        let round_trip: PfdMask = serde_json::from_str(&json).unwrap();
        assert_eq!(mask, round_trip);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_pfd_mask_serde_rejects_invalid_payload() {
        // Unsorted breakpoints must be rejected at deserialization time.
        let bad = r#"{"nodes":[[0.4363,-144.0],[0.0873,-154.0]]}"#;
        assert!(serde_json::from_str::<PfdMask>(bad).is_err());
    }

    #[test]
    fn test_pfd_with_high_eirp() {
        // 30 dBW EIRP, 500 km, 10 MHz occupied, 4 kHz ref
        let pfd_val = power_flux_density(
            30.0.db(),
            Distance::kilometers(500.0),
            10.0.mhz(),
            Frequency::hertz(4e3),
        );
        let eirp_w = 10.0_f64.powf(3.0); // 30 dBW = 1000 W
        let d_m = 500e3;
        let expected =
            10.0 * (eirp_w / (4.0 * std::f64::consts::PI * d_m * d_m) * (4e3 / 10e6)).log10();
        assert_approx_eq!(pfd_val.as_f64(), expected, atol <= 0.01);
    }
}
