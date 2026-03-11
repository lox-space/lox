// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Flower constellation builder.
//!
//! Reference: Wilkins, "The Flower Constellations", Texas A&M University, 2004.

use std::f64::consts::{PI, TAU};

use lox_bodies::{Origin, TryMeanRadius, TryPointMass, TryRotationalElements};
use lox_core::elements::KeplerianBuilder;
use lox_core::units::{Angle, Distance};
use lox_frames::ReferenceFrame;
use lox_time::Time;
use lox_time::time_scales::TimeScale;

use super::{Constellation, ConstellationError, ConstellationSatellite};

/// Builder for Flower constellations (repeating ground tracks).
#[derive(Debug, Clone)]
pub struct FlowerBuilder {
    n_petals: u32,
    n_days: u32,
    nsats: usize,
    phasing_numerator: u32,
    phasing_denominator: u32,
    shape: Option<FlowerShape>,
    inclination: Angle,
    argument_of_periapsis: Angle,
    longitude_of_ascending_node: Angle,
}

#[derive(Debug, Clone)]
enum FlowerShape {
    PerigeeAltitude { altitude: Distance },
    SemiMajorAxis { sma: Distance, eccentricity: f64 },
}

impl FlowerBuilder {
    /// Creates a new Flower constellation builder.
    pub fn new(
        n_petals: u32,
        n_days: u32,
        nsats: usize,
        phasing_numerator: u32,
        phasing_denominator: u32,
    ) -> Self {
        Self {
            n_petals,
            n_days,
            nsats,
            phasing_numerator,
            phasing_denominator,
            shape: None,
            inclination: Angle::ZERO,
            argument_of_periapsis: Angle::ZERO,
            longitude_of_ascending_node: Angle::ZERO,
        }
    }

    /// Compute SMA and eccentricity from perigee altitude.
    ///
    /// The mean radius, gravitational parameter, and rotation rate are
    /// retrieved from the origin in [`build_constellation`](Self::build_constellation).
    pub fn with_perigee_altitude(mut self, altitude: Distance) -> Self {
        self.shape = Some(FlowerShape::PerigeeAltitude { altitude });
        self
    }

    /// Provide pre-computed SMA and eccentricity.
    pub fn with_semi_major_axis(mut self, sma: Distance, eccentricity: f64) -> Self {
        self.shape = Some(FlowerShape::SemiMajorAxis { sma, eccentricity });
        self
    }

    /// Sets the orbital inclination.
    pub fn with_inclination(mut self, inclination: Angle) -> Self {
        self.inclination = inclination;
        self
    }

    /// Sets the argument of periapsis.
    pub fn with_argument_of_periapsis(mut self, aop: Angle) -> Self {
        self.argument_of_periapsis = aop;
        self
    }

    /// Sets the longitude of ascending node offset applied to all satellites.
    pub fn with_longitude_of_ascending_node(mut self, longitude_of_ascending_node: Angle) -> Self {
        self.longitude_of_ascending_node = longitude_of_ascending_node;
        self
    }

    fn build_satellites(
        &self,
        sma: Distance,
        ecc: f64,
    ) -> Result<Vec<ConstellationSatellite>, ConstellationError> {
        if self.nsats == 0 {
            return Err(ConstellationError::ZeroSatellites);
        }

        // Spacing (in degrees in source, but we work in radians)
        let mut delta_raan = -TAU * self.phasing_numerator as f64 / self.phasing_denominator as f64;
        let mut delta_anom = -delta_raan * self.n_petals as f64 / self.n_days as f64;

        // Normalize to (-2*pi, 2*pi)
        if delta_raan.abs() > TAU {
            delta_raan %= TAU;
        }
        if delta_anom.abs() > TAU {
            delta_anom %= TAU;
        }

        let mut satellites = Vec::with_capacity(self.nsats);

        for i_sat in 0..self.nsats {
            let raan = Angle::radians_normalized(
                self.longitude_of_ascending_node.as_f64() + i_sat as f64 * delta_raan,
            );
            let mean_anomaly = Angle::radians_normalized(i_sat as f64 * delta_anom);

            let elements = KeplerianBuilder::new()
                .with_semi_major_axis(sma, ecc)
                .with_inclination(self.inclination)
                .with_longitude_of_ascending_node(raan)
                .with_argument_of_periapsis(self.argument_of_periapsis)
                .with_mean_anomaly(mean_anomaly)
                .build()?;

            satellites.push(ConstellationSatellite {
                plane: 0, // Flower constellations are single-plane
                index_in_plane: i_sat,
                elements,
            });
        }

        Ok(satellites)
    }

    /// Build the constellation, resolving body parameters from the origin
    /// when using perigee altitude.
    pub fn build_constellation<T, O, R>(
        &self,
        name: impl Into<String>,
        epoch: Time<T>,
        origin: O,
        frame: R,
    ) -> Result<Constellation<T, O, R>, ConstellationError>
    where
        T: TimeScale,
        O: Origin + TryMeanRadius + TryPointMass + TryRotationalElements,
        R: ReferenceFrame,
    {
        let (sma, ecc) = match &self.shape {
            Some(FlowerShape::PerigeeAltitude { altitude }) => {
                let mean_radius = origin
                    .try_mean_radius()
                    .map_err(|e| ConstellationError::UndefinedProperty(e.to_string()))?;
                let grav_param = origin
                    .try_gravitational_parameter()
                    .map_err(|e| ConstellationError::UndefinedProperty(e.to_string()))?;
                let rotation_rate = origin
                    .try_rotation_rate(0.0)
                    .map_err(|e| ConstellationError::UndefinedProperty(e.to_string()))?;

                let mu = grav_param.as_f64();
                // Orbital period: T = (2*pi / omega_body) * n_days / n_petals
                let period = (TAU / rotation_rate) * self.n_days as f64 / self.n_petals as f64;
                // Kepler's 3rd law: a = (mu * T^2 / (4*pi^2))^(1/3)
                let sma_m = (mu * period.powi(2) / (4.0 * PI * PI)).cbrt();
                let sma = Distance::new(sma_m);
                let ecc = 1.0 - (mean_radius.as_f64() + altitude.as_f64()) / sma_m;
                (sma, ecc)
            }
            Some(FlowerShape::SemiMajorAxis { sma, eccentricity }) => (*sma, *eccentricity),
            None => return Err(ConstellationError::MissingShape),
        };

        let satellites = self.build_satellites(sma, ecc)?;
        Ok(Constellation::new(name, epoch, origin, frame, satellites))
    }
}

#[cfg(test)]
mod tests {
    use lox_bodies::Earth;
    use lox_bodies::MeanRadius;
    use lox_bodies::PointMass;
    use lox_bodies::RotationalElements;
    use lox_frames::Icrf;
    use lox_test_utils::assert_approx_eq;
    use lox_time::time_scales::Tai;
    use lox_units::{AngleUnits, DistanceUnits};

    use super::*;

    #[test]
    fn test_flower_basic() {
        let epoch = Time::j2000(Tai);
        let c = FlowerBuilder::new(14, 1, 28, 1, 28)
            .with_perigee_altitude(780.0.km())
            .with_inclination(53.0.deg())
            .build_constellation("flower", epoch, Earth, Icrf)
            .unwrap();

        assert_eq!(c.len(), 28);
        // All in plane 0
        assert!(c.satellites().iter().all(|s| s.plane == 0));
    }

    #[test]
    fn test_flower_sma_eccentricity() {
        let earth_mu = PointMass::gravitational_parameter(&Earth);
        let earth_r = MeanRadius::mean_radius(&Earth);
        let omega = Earth.rotation_rate(0.0);

        let period = (TAU / omega) * 1.0 / 14.0;
        let expected_sma = (earth_mu.as_f64() * period.powi(2) / (4.0 * PI * PI)).cbrt();
        let expected_ecc = 1.0 - (earth_r.as_f64() + 780.0e3) / expected_sma;

        let epoch = Time::j2000(Tai);
        let c = FlowerBuilder::new(14, 1, 2, 1, 28)
            .with_perigee_altitude(780.0.km())
            .with_inclination(53.0.deg())
            .build_constellation("flower", epoch, Earth, Icrf)
            .unwrap();

        assert_approx_eq!(
            c.satellites()[0].elements.semi_major_axis().as_f64(),
            expected_sma,
            rtol <= 1e-10
        );
        assert_approx_eq!(
            c.satellites()[0].elements.eccentricity().as_f64(),
            expected_ecc,
            rtol <= 1e-10
        );
    }

    #[test]
    fn test_flower_with_sma() {
        let epoch = Time::j2000(Tai);
        let c = FlowerBuilder::new(14, 1, 5, 1, 28)
            .with_semi_major_axis(7000.0.km(), 0.01)
            .with_inclination(53.0.deg())
            .build_constellation("flower", epoch, Earth, Icrf)
            .unwrap();

        assert_eq!(c.len(), 5);
    }

    #[test]
    fn test_flower_missing_shape() {
        let epoch = Time::j2000(Tai);
        let result = FlowerBuilder::new(14, 1, 5, 1, 28)
            .with_inclination(53.0.deg())
            .build_constellation("flower", epoch, Earth, Icrf);

        assert!(matches!(result, Err(ConstellationError::MissingShape)));
    }

    #[test]
    fn test_flower_zero_satellites() {
        let epoch = Time::j2000(Tai);
        let result = FlowerBuilder::new(14, 1, 0, 1, 28)
            .with_perigee_altitude(780.0.km())
            .with_inclination(53.0.deg())
            .build_constellation("flower", epoch, Earth, Icrf);

        assert!(matches!(result, Err(ConstellationError::ZeroSatellites)));
    }

    #[test]
    fn test_flower_with_argument_of_periapsis() {
        let epoch = Time::j2000(Tai);
        let c = FlowerBuilder::new(14, 1, 5, 1, 28)
            .with_perigee_altitude(780.0.km())
            .with_inclination(53.0.deg())
            .with_argument_of_periapsis(45.0.deg())
            .build_constellation("flower", epoch, Earth, Icrf)
            .unwrap();

        assert_eq!(c.len(), 5);
        assert_approx_eq!(
            c.satellites()[0].elements.argument_of_periapsis().as_f64(),
            45.0_f64.to_radians(),
            atol <= 1e-10
        );
    }

    #[test]
    fn test_flower_with_longitude_of_ascending_node() {
        let epoch = Time::j2000(Tai);
        let c = FlowerBuilder::new(14, 1, 5, 1, 28)
            .with_semi_major_axis(7000.0.km(), 0.01)
            .with_inclination(53.0.deg())
            .with_longitude_of_ascending_node(90.0.deg())
            .build_constellation("flower", epoch, Earth, Icrf)
            .unwrap();

        // First satellite's RAAN should be 90 deg
        assert_approx_eq!(
            c.satellites()[0]
                .elements
                .longitude_of_ascending_node()
                .as_f64(),
            90.0_f64.to_radians(),
            atol <= 1e-10
        );
    }

    #[test]
    fn test_flower_large_phasing_normalizes() {
        // Use phasing values that trigger the modulo normalization
        // delta_raan = -TAU * 3 / 2 = -3*PI, abs > TAU so %= TAU
        let epoch = Time::j2000(Tai);
        let c = FlowerBuilder::new(14, 1, 4, 3, 2)
            .with_semi_major_axis(7000.0.km(), 0.01)
            .with_inclination(53.0.deg())
            .build_constellation("flower", epoch, Earth, Icrf)
            .unwrap();

        assert_eq!(c.len(), 4);
    }
}
