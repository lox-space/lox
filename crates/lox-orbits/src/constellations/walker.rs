// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Walker Star and Walker Delta constellation builders.

use std::f64::consts::{PI, TAU};

use lox_bodies::Origin;
use lox_core::elements::KeplerianBuilder;
use lox_core::units::{Angle, Distance};
use lox_frames::ReferenceFrame;
use lox_time::Time;
use lox_time::time_scales::TimeScale;

use super::{Constellation, ConstellationError, ConstellationSatellite};

/// Walker variant distinguishing the RAAN spread.
#[derive(Debug, Clone, Copy)]
enum WalkerVariant {
    /// RAAN spread = 360° (TAU).
    Delta,
    /// RAAN spread = 180° (PI).
    Star,
}

impl WalkerVariant {
    fn raan_spread(self) -> f64 {
        match self {
            WalkerVariant::Delta => TAU,
            WalkerVariant::Star => PI,
        }
    }
}

/// Shared fields for Walker builders.
#[derive(Debug, Clone)]
struct WalkerParams {
    nsats: usize,
    nplanes: usize,
    semi_major_axis: Option<(Distance, f64)>,
    inclination: Angle,
    phasing: usize,
    argument_of_periapsis: Angle,
}

impl WalkerParams {
    fn new(nsats: usize, nplanes: usize) -> Self {
        Self {
            nsats,
            nplanes,
            semi_major_axis: None,
            inclination: Angle::ZERO,
            phasing: 0,
            argument_of_periapsis: Angle::ZERO,
        }
    }
}

fn walker_build(
    params: &WalkerParams,
    variant: WalkerVariant,
) -> Result<Vec<ConstellationSatellite>, ConstellationError> {
    if params.nplanes == 0 {
        return Err(ConstellationError::ZeroPlanes);
    }
    if params.nsats == 0 {
        return Err(ConstellationError::ZeroSatellites);
    }
    if !params.nsats.is_multiple_of(params.nplanes) {
        return Err(ConstellationError::SatellitePlaneMismatch {
            nsats: params.nsats,
            nplanes: params.nplanes,
        });
    }

    let (sma, ecc) = params
        .semi_major_axis
        .ok_or(ConstellationError::MissingShape)?;

    let sats_per_plane = params.nsats / params.nplanes;
    let raan_spacing = variant.raan_spread() / params.nplanes as f64;
    let anomaly_spacing = TAU / sats_per_plane as f64;
    let phase_offset = params.phasing as f64 * variant.raan_spread() / params.nsats as f64;

    let mut satellites = Vec::with_capacity(params.nsats);

    for plane in 0..params.nplanes {
        let raan = Angle::radians_normalized(plane as f64 * raan_spacing);

        for sat in 0..sats_per_plane {
            let mean_anomaly = Angle::radians_normalized(
                sat as f64 * anomaly_spacing + plane as f64 * phase_offset,
            );

            let elements = KeplerianBuilder::new()
                .with_semi_major_axis(sma, ecc)
                .with_inclination(params.inclination)
                .with_longitude_of_ascending_node(raan)
                .with_argument_of_periapsis(params.argument_of_periapsis)
                .with_mean_anomaly(mean_anomaly)
                .build()?;

            satellites.push(ConstellationSatellite {
                plane,
                index_in_plane: sat,
                elements,
            });
        }
    }

    Ok(satellites)
}

/// Builder for Walker Delta constellations (RAAN spread = 360°).
#[derive(Debug, Clone)]
pub struct WalkerDeltaBuilder {
    params: WalkerParams,
}

impl WalkerDeltaBuilder {
    /// Creates a new Walker Delta builder with the given satellite and plane counts.
    pub fn new(nsats: usize, nplanes: usize) -> Self {
        Self {
            params: WalkerParams::new(nsats, nplanes),
        }
    }

    /// Sets the semi-major axis and eccentricity.
    pub fn with_semi_major_axis(mut self, sma: Distance, eccentricity: f64) -> Self {
        self.params.semi_major_axis = Some((sma, eccentricity));
        self
    }

    /// Sets the orbital inclination.
    pub fn with_inclination(mut self, inclination: Angle) -> Self {
        self.params.inclination = inclination;
        self
    }

    /// Sets the inter-plane phasing parameter.
    pub fn with_phasing(mut self, phasing: usize) -> Self {
        self.params.phasing = phasing;
        self
    }

    /// Sets the argument of periapsis.
    pub fn with_argument_of_periapsis(mut self, aop: Angle) -> Self {
        self.params.argument_of_periapsis = aop;
        self
    }

    /// Builds the satellite list without wrapping in a [`Constellation`].
    pub fn build(&self) -> Result<Vec<ConstellationSatellite>, ConstellationError> {
        walker_build(&self.params, WalkerVariant::Delta)
    }

    /// Builds a full [`Constellation`] with the given metadata.
    pub fn build_constellation<T: TimeScale, O: Origin, R: ReferenceFrame>(
        &self,
        name: impl Into<String>,
        epoch: Time<T>,
        origin: O,
        frame: R,
    ) -> Result<Constellation<T, O, R>, ConstellationError> {
        let satellites = self.build()?;
        Ok(Constellation::new(name, epoch, origin, frame, satellites))
    }
}

/// Builder for Walker Star constellations (RAAN spread = 180°).
#[derive(Debug, Clone)]
pub struct WalkerStarBuilder {
    params: WalkerParams,
}

impl WalkerStarBuilder {
    /// Creates a new Walker Star builder with the given satellite and plane counts.
    pub fn new(nsats: usize, nplanes: usize) -> Self {
        Self {
            params: WalkerParams::new(nsats, nplanes),
        }
    }

    /// Sets the semi-major axis and eccentricity.
    pub fn with_semi_major_axis(mut self, sma: Distance, eccentricity: f64) -> Self {
        self.params.semi_major_axis = Some((sma, eccentricity));
        self
    }

    /// Sets the orbital inclination.
    pub fn with_inclination(mut self, inclination: Angle) -> Self {
        self.params.inclination = inclination;
        self
    }

    /// Sets the inter-plane phasing parameter.
    pub fn with_phasing(mut self, phasing: usize) -> Self {
        self.params.phasing = phasing;
        self
    }

    /// Sets the argument of periapsis.
    pub fn with_argument_of_periapsis(mut self, aop: Angle) -> Self {
        self.params.argument_of_periapsis = aop;
        self
    }

    /// Builds the satellite list without wrapping in a [`Constellation`].
    pub fn build(&self) -> Result<Vec<ConstellationSatellite>, ConstellationError> {
        walker_build(&self.params, WalkerVariant::Star)
    }

    /// Builds a full [`Constellation`] with the given metadata.
    pub fn build_constellation<T: TimeScale, O: Origin, R: ReferenceFrame>(
        &self,
        name: impl Into<String>,
        epoch: Time<T>,
        origin: O,
        frame: R,
    ) -> Result<Constellation<T, O, R>, ConstellationError> {
        let satellites = self.build()?;
        Ok(Constellation::new(name, epoch, origin, frame, satellites))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use lox_test_utils::assert_approx_eq;
    use lox_units::{AngleUnits, DistanceUnits};

    use super::*;

    #[test]
    fn test_walker_delta_iridium() {
        let sats = WalkerDeltaBuilder::new(66, 6)
            .with_semi_major_axis(7159.0.km(), 0.0)
            .with_inclination(53.0.deg())
            .with_phasing(1)
            .build()
            .unwrap();

        assert_eq!(sats.len(), 66);
        // 11 sats per plane
        let planes: HashSet<usize> = sats.iter().map(|s| s.plane).collect();
        assert_eq!(planes.len(), 6);
        for plane in 0..6 {
            let count = sats.iter().filter(|s| s.plane == plane).count();
            assert_eq!(count, 11);
        }

        // Check RAAN spacing: 360/6 = 60 deg
        let raan0 = sats[0].elements.longitude_of_ascending_node().as_f64();
        let raan1 = sats[11].elements.longitude_of_ascending_node().as_f64();
        assert_approx_eq!(raan1 - raan0, 60.0_f64.to_radians(), atol <= 1e-10);
    }

    #[test]
    fn test_walker_star_8_4() {
        let sats = WalkerStarBuilder::new(8, 4)
            .with_semi_major_axis(7000.0.km(), 0.0)
            .with_inclination(90.0.deg())
            .build()
            .unwrap();

        assert_eq!(sats.len(), 8);
        // RAAN spacing: 180/4 = 45 deg
        let raan0 = sats[0].elements.longitude_of_ascending_node().as_f64();
        let raan1 = sats[2].elements.longitude_of_ascending_node().as_f64();
        assert_approx_eq!(raan1 - raan0, 45.0_f64.to_radians(), atol <= 1e-10);
    }

    #[test]
    fn test_walker_delta_raan_spacing() {
        // Walker Delta 8/4 should have RAAN spacing = 360/4 = 90 deg
        let sats = WalkerDeltaBuilder::new(8, 4)
            .with_semi_major_axis(7000.0.km(), 0.0)
            .with_inclination(90.0.deg())
            .build()
            .unwrap();

        let raan0 = sats[0].elements.longitude_of_ascending_node().as_f64();
        let raan1 = sats[2].elements.longitude_of_ascending_node().as_f64();
        assert_approx_eq!(raan1 - raan0, 90.0_f64.to_radians(), atol <= 1e-10);
    }

    #[test]
    fn test_walker_validation_mismatch() {
        let result = WalkerDeltaBuilder::new(7, 3)
            .with_semi_major_axis(7000.0.km(), 0.0)
            .with_inclination(53.0.deg())
            .build();

        assert!(matches!(
            result,
            Err(ConstellationError::SatellitePlaneMismatch {
                nsats: 7,
                nplanes: 3
            })
        ));
    }

    #[test]
    fn test_walker_validation_zero_planes() {
        let result = WalkerDeltaBuilder::new(6, 0)
            .with_semi_major_axis(7000.0.km(), 0.0)
            .build();

        assert!(matches!(result, Err(ConstellationError::ZeroPlanes)));
    }

    #[test]
    fn test_walker_validation_zero_sats() {
        let result = WalkerDeltaBuilder::new(0, 3)
            .with_semi_major_axis(7000.0.km(), 0.0)
            .build();

        assert!(matches!(result, Err(ConstellationError::ZeroSatellites)));
    }

    #[test]
    fn test_walker_validation_missing_shape() {
        let result = WalkerDeltaBuilder::new(6, 3)
            .with_inclination(53.0.deg())
            .build();

        assert!(matches!(result, Err(ConstellationError::MissingShape)));
    }

    #[test]
    fn test_walker_build_constellation() {
        use lox_bodies::Earth;
        use lox_frames::Icrf;
        use lox_time::time_scales::Tai;

        let epoch = Time::j2000(Tai);
        let c = WalkerDeltaBuilder::new(6, 3)
            .with_semi_major_axis(7000.0.km(), 0.0)
            .with_inclination(53.0.deg())
            .build_constellation("test", epoch, Earth, Icrf)
            .unwrap();

        assert_eq!(c.name(), "test");
        assert_eq!(c.len(), 6);
    }

    #[test]
    fn test_walker_star_with_phasing() {
        let sats = WalkerStarBuilder::new(8, 4)
            .with_semi_major_axis(7000.0.km(), 0.0)
            .with_inclination(90.0.deg())
            .with_phasing(2)
            .build()
            .unwrap();

        assert_eq!(sats.len(), 8);
        // With circular orbits (e=0), true anomaly == mean anomaly
        // Phase offset = 2 * PI / 8 = PI/4
        // Plane 1 sat 0 should have true_anomaly = PI/4
        let plane1_sat0 = &sats[2]; // 2 sats per plane
        let expected_phase = std::f64::consts::FRAC_PI_4;
        assert_approx_eq!(
            plane1_sat0.elements.true_anomaly().as_f64(),
            expected_phase,
            atol <= 1e-10
        );
    }

    #[test]
    fn test_walker_delta_with_argument_of_periapsis() {
        let sats = WalkerDeltaBuilder::new(6, 3)
            .with_semi_major_axis(7000.0.km(), 0.0)
            .with_inclination(53.0.deg())
            .with_argument_of_periapsis(90.0.deg())
            .build()
            .unwrap();

        for sat in &sats {
            assert_approx_eq!(
                sat.elements.argument_of_periapsis().as_f64(),
                90.0_f64.to_radians(),
                atol <= 1e-10
            );
        }
    }

    #[test]
    fn test_walker_star_build_constellation() {
        use lox_bodies::Earth;
        use lox_frames::Icrf;
        use lox_time::time_scales::Tai;

        let epoch = Time::j2000(Tai);
        let c = WalkerStarBuilder::new(8, 4)
            .with_semi_major_axis(7000.0.km(), 0.0)
            .with_inclination(90.0.deg())
            .build_constellation("star", epoch, Earth, Icrf)
            .unwrap();

        assert_eq!(c.name(), "star");
        assert_eq!(c.len(), 8);
    }

    #[test]
    fn test_walker_star_validation_errors() {
        // Zero planes
        assert!(matches!(
            WalkerStarBuilder::new(6, 0)
                .with_semi_major_axis(7000.0.km(), 0.0)
                .build(),
            Err(ConstellationError::ZeroPlanes)
        ));
        // Zero sats
        assert!(matches!(
            WalkerStarBuilder::new(0, 3)
                .with_semi_major_axis(7000.0.km(), 0.0)
                .build(),
            Err(ConstellationError::ZeroSatellites)
        ));
        // Missing shape
        assert!(matches!(
            WalkerStarBuilder::new(6, 3).build(),
            Err(ConstellationError::MissingShape)
        ));
    }
}
