// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::{Earth, Origin, TryMeanRadius, TrySpheroid};
use lox_core::elements::{KeplerianBuilder, KeplerianError};
use lox_core::units::{Angle, Distance};
use lox_frames::Icrf;
use lox_time::Time;
use lox_time::time_scales::{Tai, TimeScale};
use thiserror::Error;

use crate::orbits::KeplerianOrbit;
use crate::orbits::keplerian::KeplerianOrbitError;

/// Errors that can occur when building an orbit.
#[derive(Debug, Clone, Error)]
pub enum OrbitBuilderError {
    /// Invalid Keplerian elements.
    #[error(transparent)]
    Keplerian(#[from] KeplerianError),
    /// Keplerian orbit validation failed.
    #[error(transparent)]
    Orbit(#[from] KeplerianOrbitError),
    /// The origin body does not define a mean radius.
    #[error("the origin does not have a mean radius")]
    MissingMeanRadius,
    /// No orbital shape (semi-major axis, radii, or altitudes) was specified.
    #[error("no orbital shape was specified")]
    MissingShape,
    /// Both true anomaly and mean anomaly were provided.
    #[error("both true anomaly and mean anomaly were specified")]
    AmbiguousAnomaly,
}

// ---------------------------------------------------------------------------
// KeplerianOrbitBuilder
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Shape {
    SemiMajorAxis(Distance, f64),
    Radii(Distance, Distance),
    Altitudes(Distance, Distance),
}

/// Builder for constructing Keplerian orbits from orbital elements.
#[derive(Debug, Clone)]
pub struct KeplerianOrbitBuilder<T: TimeScale, O: Origin> {
    time: Time<T>,
    origin: O,
    shape: Option<Shape>,
    inclination: Angle,
    longitude_of_ascending_node: Angle,
    argument_of_periapsis: Angle,
    true_anomaly: Option<Angle>,
    mean_anomaly: Option<Angle>,
}

impl Default for KeplerianOrbitBuilder<Tai, Earth> {
    fn default() -> Self {
        Self::new()
    }
}

impl KeplerianOrbitBuilder<Tai, Earth> {
    /// Creates a new builder with default TAI time scale and Earth origin.
    pub fn new() -> Self {
        Self {
            time: Time::default(),
            origin: Earth,
            shape: None,
            inclination: Angle::ZERO,
            longitude_of_ascending_node: Angle::ZERO,
            argument_of_periapsis: Angle::ZERO,
            true_anomaly: None,
            mean_anomaly: None,
        }
    }
}

// Typestate: change time scale
impl<S: TimeScale, O: Origin> KeplerianOrbitBuilder<S, O> {
    /// Sets the epoch and changes the time scale.
    pub fn with_time<T: TimeScale>(self, time: Time<T>) -> KeplerianOrbitBuilder<T, O> {
        KeplerianOrbitBuilder {
            time,
            origin: self.origin,
            shape: self.shape,
            inclination: self.inclination,
            longitude_of_ascending_node: self.longitude_of_ascending_node,
            argument_of_periapsis: self.argument_of_periapsis,
            true_anomaly: self.true_anomaly,
            mean_anomaly: self.mean_anomaly,
        }
    }

    /// Sets the central body origin.
    pub fn with_origin<N: Origin>(self, origin: N) -> KeplerianOrbitBuilder<S, N> {
        KeplerianOrbitBuilder {
            time: self.time,
            origin,
            shape: self.shape,
            inclination: self.inclination,
            longitude_of_ascending_node: self.longitude_of_ascending_node,
            argument_of_periapsis: self.argument_of_periapsis,
            true_anomaly: self.true_anomaly,
            mean_anomaly: self.mean_anomaly,
        }
    }
}

impl<T: TimeScale, O: Origin> KeplerianOrbitBuilder<T, O> {
    /// Sets the orbital shape via semi-major axis and eccentricity.
    pub fn with_semi_major_axis(mut self, semi_major_axis: Distance, eccentricity: f64) -> Self {
        self.shape = Some(Shape::SemiMajorAxis(semi_major_axis, eccentricity));
        self
    }

    /// Sets the orbital shape via periapsis and apoapsis radii.
    pub fn with_radii(mut self, periapsis_radius: Distance, apoapsis_radius: Distance) -> Self {
        self.shape = Some(Shape::Radii(periapsis_radius, apoapsis_radius));
        self
    }

    /// Sets the orbital shape via periapsis and apoapsis altitudes above the mean radius.
    pub fn with_altitudes(
        mut self,
        periapsis_altitude: Distance,
        apoapsis_altitude: Distance,
    ) -> Self {
        self.shape = Some(Shape::Altitudes(periapsis_altitude, apoapsis_altitude));
        self
    }

    /// Sets the orbital inclination.
    pub fn with_inclination(mut self, inclination: Angle) -> Self {
        self.inclination = inclination;
        self
    }

    /// Sets the longitude of the ascending node.
    pub fn with_longitude_of_ascending_node(mut self, longitude_of_ascending_node: Angle) -> Self {
        self.longitude_of_ascending_node = longitude_of_ascending_node;
        self
    }

    /// Sets the argument of periapsis.
    pub fn with_argument_of_periapsis(mut self, argument_of_periapsis: Angle) -> Self {
        self.argument_of_periapsis = argument_of_periapsis;
        self
    }

    /// Sets the true anomaly.
    pub fn with_true_anomaly(mut self, true_anomaly: Angle) -> Self {
        self.true_anomaly = Some(true_anomaly);
        self
    }

    /// Sets the mean anomaly.
    pub fn with_mean_anomaly(mut self, mean_anomaly: Angle) -> Self {
        self.mean_anomaly = Some(mean_anomaly);
        self
    }

    /// Builds the Keplerian orbit in the ICRF frame.
    pub fn build(self) -> Result<KeplerianOrbit<T, O, Icrf>, OrbitBuilderError>
    where
        O: TryMeanRadius + TrySpheroid + Copy,
        T: Copy,
    {
        if self.true_anomaly.is_some() && self.mean_anomaly.is_some() {
            return Err(OrbitBuilderError::AmbiguousAnomaly);
        }

        let shape = self.shape.ok_or(OrbitBuilderError::MissingShape)?;

        let mut builder = KeplerianBuilder::new();

        builder = match shape {
            Shape::SemiMajorAxis(sma, ecc) => builder.with_semi_major_axis(sma, ecc),
            Shape::Radii(rp, ra) => builder.with_radii(rp, ra),
            Shape::Altitudes(alt_p, alt_a) => {
                let mean_radius = self
                    .origin
                    .try_mean_radius()
                    .map_err(|_| OrbitBuilderError::MissingMeanRadius)?;
                builder.with_altitudes(alt_p, alt_a, mean_radius)
            }
        };

        builder = builder
            .with_inclination(self.inclination)
            .with_longitude_of_ascending_node(self.longitude_of_ascending_node)
            .with_argument_of_periapsis(self.argument_of_periapsis);

        if let Some(ta) = self.true_anomaly {
            builder = builder.with_true_anomaly(ta);
        }
        if let Some(ma) = self.mean_anomaly {
            builder = builder.with_mean_anomaly(ma);
        }

        let keplerian = builder.build()?;

        let orbit = KeplerianOrbit::try_from_keplerian(keplerian, self.time, self.origin, Icrf)?;

        Ok(orbit)
    }
}

// ---------------------------------------------------------------------------
// CircularBuilder
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum CircularSize {
    SemiMajorAxis(Distance),
    Altitude(Distance),
}

/// Builder for constructing circular orbits (eccentricity = 0).
#[derive(Debug, Clone)]
pub struct CircularBuilder<T: TimeScale, O: Origin> {
    time: Time<T>,
    origin: O,
    size: Option<CircularSize>,
    inclination: Angle,
    longitude_of_ascending_node: Angle,
    true_anomaly: Angle,
}

impl Default for CircularBuilder<Tai, Earth> {
    fn default() -> Self {
        Self::new()
    }
}

impl CircularBuilder<Tai, Earth> {
    /// Creates a new circular orbit builder with default TAI time scale and Earth origin.
    pub fn new() -> Self {
        Self {
            time: Time::default(),
            origin: Earth,
            size: None,
            inclination: Angle::ZERO,
            longitude_of_ascending_node: Angle::ZERO,
            true_anomaly: Angle::ZERO,
        }
    }
}

// Typestate: change time scale
impl<S: TimeScale, O: Origin> CircularBuilder<S, O> {
    /// Sets the epoch and changes the time scale.
    pub fn with_time<T: TimeScale>(self, time: Time<T>) -> CircularBuilder<T, O> {
        CircularBuilder {
            time,
            origin: self.origin,
            size: self.size,
            inclination: self.inclination,
            longitude_of_ascending_node: self.longitude_of_ascending_node,
            true_anomaly: self.true_anomaly,
        }
    }

    /// Sets the central body origin.
    pub fn with_origin<N: Origin>(self, origin: N) -> CircularBuilder<S, N> {
        CircularBuilder {
            time: self.time,
            origin,
            size: self.size,
            inclination: self.inclination,
            longitude_of_ascending_node: self.longitude_of_ascending_node,
            true_anomaly: self.true_anomaly,
        }
    }
}

impl<T: TimeScale, O: Origin> CircularBuilder<T, O> {
    /// Sets the orbit size via semi-major axis.
    pub fn with_semi_major_axis(mut self, semi_major_axis: Distance) -> Self {
        self.size = Some(CircularSize::SemiMajorAxis(semi_major_axis));
        self
    }

    /// Sets the orbit size via altitude above the mean radius.
    pub fn with_altitude(mut self, altitude: Distance) -> Self {
        self.size = Some(CircularSize::Altitude(altitude));
        self
    }

    /// Sets the orbital inclination.
    pub fn with_inclination(mut self, inclination: Angle) -> Self {
        self.inclination = inclination;
        self
    }

    /// Sets the longitude of the ascending node.
    pub fn with_longitude_of_ascending_node(mut self, longitude_of_ascending_node: Angle) -> Self {
        self.longitude_of_ascending_node = longitude_of_ascending_node;
        self
    }

    /// Sets the true anomaly.
    pub fn with_true_anomaly(mut self, true_anomaly: Angle) -> Self {
        self.true_anomaly = true_anomaly;
        self
    }

    /// Builds the circular Keplerian orbit in the ICRF frame.
    pub fn build(self) -> Result<KeplerianOrbit<T, O, Icrf>, OrbitBuilderError>
    where
        O: TryMeanRadius + TrySpheroid + Copy,
        T: Copy,
    {
        let size = self.size.ok_or(OrbitBuilderError::MissingShape)?;

        let sma = match size {
            CircularSize::SemiMajorAxis(sma) => sma,
            CircularSize::Altitude(alt) => {
                let mean_radius = self
                    .origin
                    .try_mean_radius()
                    .map_err(|_| OrbitBuilderError::MissingMeanRadius)?;
                Distance::new(alt.as_f64() + mean_radius.as_f64())
            }
        };

        let keplerian = KeplerianBuilder::new()
            .with_semi_major_axis(sma, 0.0)
            .with_inclination(self.inclination)
            .with_longitude_of_ascending_node(self.longitude_of_ascending_node)
            .with_true_anomaly(self.true_anomaly)
            .build()?;

        let orbit = KeplerianOrbit::try_from_keplerian(keplerian, self.time, self.origin, Icrf)?;

        Ok(orbit)
    }
}

#[cfg(test)]
mod tests {
    use lox_bodies::{Earth, MeanRadius};
    use lox_core::units::{AngleUnits, DistanceUnits};
    use lox_test_utils::assert_approx_eq;
    use lox_time::time_scales::Tdb;
    use lox_time::{Time, deltas::TimeDelta, utc::Utc};

    use super::*;

    const JD1: f64 = 2458849.5;
    const JD2: f64 = 49.78099017 - 1.0;

    fn epoch() -> Time<Tdb> {
        Utc::from_delta(TimeDelta::from_two_part_julian_date(JD1, JD2))
            .unwrap()
            .to_time()
            .to_scale(Tdb)
    }

    #[test]
    fn test_keplerian_builder_sma_ecc() {
        let orbit = KeplerianOrbitBuilder::new()
            .with_time(epoch())
            .with_semi_major_axis(7178.1363.km(), 0.001)
            .with_inclination(97.0.deg())
            .build()
            .unwrap();

        assert_approx_eq!(orbit.semi_major_axis(), 7178.1363.km(), rtol <= 1e-10);
        assert_approx_eq!(orbit.eccentricity().as_f64(), 0.001, atol <= 1e-15);
        assert_approx_eq!(
            orbit.inclination().as_f64(),
            97.0.deg().as_f64(),
            rtol <= 1e-10
        );
    }

    #[test]
    fn test_keplerian_builder_radii() {
        let rp = 7000.0.km();
        let ra = 7400.0.km();
        let orbit = KeplerianOrbitBuilder::new()
            .with_time(epoch())
            .with_radii(rp, ra)
            .build()
            .unwrap();

        let exp_sma = 7200.0.km();
        let exp_ecc = (7400.0 - 7000.0) / (7400.0 + 7000.0);

        assert_approx_eq!(orbit.semi_major_axis(), exp_sma, rtol <= 1e-10);
        assert_approx_eq!(orbit.eccentricity().as_f64(), exp_ecc, rtol <= 1e-10);
    }

    #[test]
    fn test_keplerian_builder_altitudes() {
        let alt_p = 600.0.km();
        let alt_a = 1000.0.km();
        let mean_radius = MeanRadius::mean_radius(&Earth);
        let orbit = KeplerianOrbitBuilder::new()
            .with_time(epoch())
            .with_altitudes(alt_p, alt_a)
            .build()
            .unwrap();

        let rp = alt_p.as_f64() + mean_radius.as_f64();
        let ra = alt_a.as_f64() + mean_radius.as_f64();
        let exp_sma = Distance::new((rp + ra) / 2.0);
        let exp_ecc = (ra - rp) / (ra + rp);

        assert_approx_eq!(orbit.semi_major_axis(), exp_sma, rtol <= 1e-10);
        assert_approx_eq!(orbit.eccentricity().as_f64(), exp_ecc, rtol <= 1e-10);
    }

    #[test]
    fn test_keplerian_builder_missing_shape() {
        let result = KeplerianOrbitBuilder::new()
            .with_time(epoch())
            .with_inclination(97.0.deg())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_keplerian_builder_ambiguous_anomaly() {
        let result = KeplerianOrbitBuilder::new()
            .with_time(epoch())
            .with_semi_major_axis(7178.0.km(), 0.0)
            .with_true_anomaly(0.0.deg())
            .with_mean_anomaly(0.0.deg())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_circular_builder_sma() {
        let orbit = CircularBuilder::new()
            .with_time(epoch())
            .with_semi_major_axis(7178.0.km())
            .with_inclination(97.0.deg())
            .build()
            .unwrap();

        assert_approx_eq!(orbit.semi_major_axis(), 7178.0.km(), rtol <= 1e-10);
        assert_approx_eq!(orbit.eccentricity().as_f64(), 0.0, atol <= 1e-15);
        assert_approx_eq!(
            orbit.inclination().as_f64(),
            97.0.deg().as_f64(),
            rtol <= 1e-10
        );
    }

    #[test]
    fn test_circular_builder_altitude() {
        let altitude = 800.0.km();
        let mean_radius = MeanRadius::mean_radius(&Earth);
        let orbit = CircularBuilder::new()
            .with_time(epoch())
            .with_altitude(altitude)
            .build()
            .unwrap();

        let exp_sma = Distance::new(altitude.as_f64() + mean_radius.as_f64());
        assert_approx_eq!(orbit.semi_major_axis(), exp_sma, rtol <= 1e-10);
        assert_approx_eq!(orbit.eccentricity().as_f64(), 0.0, atol <= 1e-15);
    }

    #[test]
    fn test_circular_builder_missing_size() {
        let result = CircularBuilder::new().with_time(epoch()).build();

        assert!(result.is_err());
    }

    #[test]
    fn test_circular_builder_defaults() {
        let orbit = CircularBuilder::new()
            .with_time(epoch())
            .with_semi_major_axis(7178.0.km())
            .build()
            .unwrap();

        assert_approx_eq!(orbit.inclination().as_f64(), 0.0, atol <= 1e-15);
        assert_approx_eq!(
            orbit.longitude_of_ascending_node().as_f64(),
            0.0,
            atol <= 1e-15
        );
        assert_approx_eq!(orbit.argument_of_periapsis().as_f64(), 0.0, atol <= 1e-15);
        assert_approx_eq!(orbit.true_anomaly().as_f64(), 0.0, atol <= 1e-15);
    }
}
