// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Street-of-Coverage constellation builder.
//!
//! Reference: Huang, Colombo, Bernelli-Zazzera, Acta Astronautica 188:151-170, 2021.

use std::f64::consts::PI;

use lox_bodies::Origin;
use lox_core::elements::KeplerianBuilder;
use lox_core::math::optim::{BrentMinimizer, FindBracketedMinimum};
use lox_core::math::roots::BoxedError;
use lox_core::units::{Angle, Distance};
use lox_frames::ReferenceFrame;
use lox_time::Time;
use lox_time::time_scales::TimeScale;

use super::{Constellation, ConstellationError, ConstellationSatellite};

/// Compute the minimum half-width of street-of-coverage for j-fold coverage.
/// Ref: Eq. 3 of Huang et al. 2021.
fn c_j(nu: f64, s: f64, j: usize) -> f64 {
    (nu.cos() / (j as f64 * PI / s).cos()).acos()
}

/// Objective function for the SoC optimizer (Eq. 37 of Huang et al. 2021).
fn nu_to_minimize(nu: f64, p: f64, s: f64, j: usize, inc: f64) -> f64 {
    let c_j_nu = c_j(nu, s, j);
    let c_1_nu = c_j(nu, s, 1);
    let sin_i = inc.sin();
    ((p - 1.0) * ((nu + c_j_nu) / 2.0).sin().asin().abs() / sin_i.abs()
        - ((PI - c_1_nu - c_j_nu) / 2.0).sin().asin().abs() / sin_i.abs())
    .abs()
}

/// Builder for Street-of-Coverage constellations.
#[derive(Debug, Clone)]
pub struct StreetOfCoverageBuilder {
    nsats: usize,
    nplanes: usize,
    semi_major_axis: Option<(Distance, f64)>,
    inclination: Angle,
    coverage_fold: usize,
    argument_of_periapsis: Angle,
    longitude_of_ascending_node: Angle,
}

impl StreetOfCoverageBuilder {
    /// Creates a new Street-of-Coverage builder with the given satellite and plane counts.
    pub fn new(nsats: usize, nplanes: usize) -> Self {
        Self {
            nsats,
            nplanes,
            semi_major_axis: None,
            inclination: Angle::ZERO,
            coverage_fold: 1,
            argument_of_periapsis: Angle::ZERO,
            longitude_of_ascending_node: Angle::ZERO,
        }
    }

    /// Sets the semi-major axis and eccentricity.
    pub fn with_semi_major_axis(mut self, sma: Distance, eccentricity: f64) -> Self {
        self.semi_major_axis = Some((sma, eccentricity));
        self
    }

    /// Sets the orbital inclination.
    pub fn with_inclination(mut self, inclination: Angle) -> Self {
        self.inclination = inclination;
        self
    }

    /// Sets the coverage fold (number of simultaneous covering satellites).
    pub fn with_coverage_fold(mut self, j: usize) -> Self {
        self.coverage_fold = j;
        self
    }

    /// Sets the argument of periapsis.
    pub fn with_argument_of_periapsis(mut self, aop: Angle) -> Self {
        self.argument_of_periapsis = aop;
        self
    }

    /// Sets the longitude of ascending node offset applied to all orbital planes.
    pub fn with_longitude_of_ascending_node(mut self, longitude_of_ascending_node: Angle) -> Self {
        self.longitude_of_ascending_node = longitude_of_ascending_node;
        self
    }

    /// Builds the satellite list without wrapping in a [`Constellation`].
    pub fn build(&self) -> Result<Vec<ConstellationSatellite>, ConstellationError> {
        if self.nplanes == 0 {
            return Err(ConstellationError::ZeroPlanes);
        }
        if self.nsats == 0 {
            return Err(ConstellationError::ZeroSatellites);
        }
        if !self.nsats.is_multiple_of(self.nplanes) {
            return Err(ConstellationError::SatellitePlaneMismatch {
                nsats: self.nsats,
                nplanes: self.nplanes,
            });
        }

        let (sma, ecc) = self
            .semi_major_axis
            .ok_or(ConstellationError::MissingShape)?;

        let sats_per_plane = self.nsats / self.nplanes;
        let j = self.coverage_fold;
        let p = self.nplanes;
        let s = sats_per_plane;
        let inc = self.inclination.as_f64();

        // Eq. 11: constraint check
        if j * p * (p - 1) > self.nsats {
            return Err(ConstellationError::SocConstraint);
        }

        // Near-equatorial orbits are not allowed
        if inc.sin().abs() < 1e-9 {
            return Err(ConstellationError::SocEquatorialInclination);
        }

        // Optimize nu (central angle of coverage)
        let nu_min = j as f64 * PI / s as f64; // Eq. 7
        let nu_max = PI / 2.0;

        let pf = p as f64;
        let sf = s as f64;

        let nu_opt = BrentMinimizer::default()
            .find_minimum_in_bracket(
                |nu: f64| -> Result<f64, BoxedError> { Ok(nu_to_minimize(nu, pf, sf, j, inc)) },
                (nu_min, nu_max),
            )
            .map_err(|_| ConstellationError::SocNotConverged)?;

        // RAAN spacing (co-rotating), Eq. 2
        let c_j_opt = c_j(nu_opt, sf, j);
        let delta_raan = 2.0 * ((nu_opt + c_j_opt) / 2.0).sin().asin().abs() / inc.sin().abs();

        // Intra-plane anomaly spacing, Eq. 5
        let delta_anom_intra = 2.0 * PI / sf;

        // Inter-plane anomaly spacing, Eq. 6
        let delta_anom_inter = j as f64 * PI / sf
            - 2.0 * ((delta_raan / 2.0).cos() / ((nu_opt + c_j_opt) / 2.0).cos()).acos();

        let mut satellites = Vec::with_capacity(self.nsats);

        for plane in 0..p {
            let raan = Angle::radians_normalized(
                self.longitude_of_ascending_node.as_f64() + plane as f64 * delta_raan,
            );

            for sat in 0..s {
                let mean_anomaly = Angle::radians_normalized(
                    sat as f64 * delta_anom_intra + plane as f64 * delta_anom_inter,
                );

                let elements = KeplerianBuilder::new()
                    .with_semi_major_axis(sma, ecc)
                    .with_inclination(self.inclination)
                    .with_longitude_of_ascending_node(raan)
                    .with_argument_of_periapsis(self.argument_of_periapsis)
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

    use lox_units::{AngleUnits, DistanceUnits};

    use super::*;

    #[test]
    fn test_soc_basic() {
        let sats = StreetOfCoverageBuilder::new(24, 4)
            .with_semi_major_axis(7159.0.km(), 0.0)
            .with_inclination(53.0.deg())
            .with_coverage_fold(1)
            .build()
            .unwrap();

        assert_eq!(sats.len(), 24);
        let planes: HashSet<usize> = sats.iter().map(|s| s.plane).collect();
        assert_eq!(planes.len(), 4);
        for plane in 0..4 {
            assert_eq!(sats.iter().filter(|s| s.plane == plane).count(), 6);
        }
    }

    #[test]
    fn test_soc_constraint_error() {
        // j * p * (p - 1) = 1 * 4 * 3 = 12 > 8 = nsats
        let result = StreetOfCoverageBuilder::new(8, 4)
            .with_semi_major_axis(7159.0.km(), 0.0)
            .with_inclination(53.0.deg())
            .with_coverage_fold(1)
            .build();

        assert!(matches!(result, Err(ConstellationError::SocConstraint)));
    }

    #[test]
    fn test_soc_equatorial_error() {
        let result = StreetOfCoverageBuilder::new(24, 4)
            .with_semi_major_axis(7159.0.km(), 0.0)
            .with_inclination(0.0.deg())
            .with_coverage_fold(1)
            .build();

        assert!(matches!(
            result,
            Err(ConstellationError::SocEquatorialInclination)
        ));
    }

    #[test]
    fn test_soc_zero_planes() {
        let result = StreetOfCoverageBuilder::new(24, 0)
            .with_semi_major_axis(7159.0.km(), 0.0)
            .with_inclination(53.0.deg())
            .build();

        assert!(matches!(result, Err(ConstellationError::ZeroPlanes)));
    }

    #[test]
    fn test_soc_zero_satellites() {
        let result = StreetOfCoverageBuilder::new(0, 4)
            .with_semi_major_axis(7159.0.km(), 0.0)
            .with_inclination(53.0.deg())
            .build();

        assert!(matches!(result, Err(ConstellationError::ZeroSatellites)));
    }

    #[test]
    fn test_soc_satellite_plane_mismatch() {
        let result = StreetOfCoverageBuilder::new(25, 4)
            .with_semi_major_axis(7159.0.km(), 0.0)
            .with_inclination(53.0.deg())
            .build();

        assert!(matches!(
            result,
            Err(ConstellationError::SatellitePlaneMismatch { .. })
        ));
    }

    #[test]
    fn test_soc_missing_shape() {
        let result = StreetOfCoverageBuilder::new(24, 4)
            .with_inclination(53.0.deg())
            .build();

        assert!(matches!(result, Err(ConstellationError::MissingShape)));
    }

    #[test]
    fn test_soc_with_argument_of_periapsis() {
        let sats = StreetOfCoverageBuilder::new(24, 4)
            .with_semi_major_axis(7159.0.km(), 0.0)
            .with_inclination(53.0.deg())
            .with_argument_of_periapsis(30.0.deg())
            .build()
            .unwrap();

        assert_eq!(sats.len(), 24);
        for sat in &sats {
            assert!(
                (sat.elements.argument_of_periapsis().as_f64() - 30.0_f64.to_radians()).abs()
                    < 1e-10
            );
        }
    }

    #[test]
    fn test_soc_with_longitude_of_ascending_node() {
        let sats_no_raan = StreetOfCoverageBuilder::new(24, 4)
            .with_semi_major_axis(7159.0.km(), 0.0)
            .with_inclination(53.0.deg())
            .build()
            .unwrap();

        let sats_with_raan = StreetOfCoverageBuilder::new(24, 4)
            .with_semi_major_axis(7159.0.km(), 0.0)
            .with_inclination(53.0.deg())
            .with_longitude_of_ascending_node(60.0.deg())
            .build()
            .unwrap();

        // First plane RAAN should differ by 60 deg
        let diff = sats_with_raan[0]
            .elements
            .longitude_of_ascending_node()
            .as_f64()
            - sats_no_raan[0]
                .elements
                .longitude_of_ascending_node()
                .as_f64();
        assert!((diff - 60.0_f64.to_radians()).abs() < 1e-10);
    }

    #[test]
    fn test_soc_build_constellation() {
        use lox_bodies::Earth;
        use lox_frames::Icrf;
        use lox_time::Time;
        use lox_time::time_scales::Tai;

        let epoch = Time::j2000(Tai);
        let c = StreetOfCoverageBuilder::new(24, 4)
            .with_semi_major_axis(7159.0.km(), 0.0)
            .with_inclination(53.0.deg())
            .build_constellation("soc", epoch, Earth, Icrf)
            .unwrap();

        assert_eq!(c.name(), "soc");
        assert_eq!(c.len(), 24);
    }
}
