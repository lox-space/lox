// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::orbits::{CartesianOrbit, TrajectorError, Trajectory};
use crate::propagators::Propagator;
use glam::{DMat3, DVec3};
use lox_bodies::{DynOrigin, RotationalElements, Spheroid, TrySpheroid};
use lox_core::coords::{Cartesian, LonLatAlt};
use lox_core::types::units::Radians;
use lox_core::units::Distance;
use lox_frames::{DynFrame, Iau, ReferenceFrame};
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::TimeScale;
use thiserror::Error;

/// Topocentric observation of a satellite from a ground location.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Observables {
    azimuth: Radians,
    elevation: Radians,
    range: f64,
    range_rate: f64,
}

impl Observables {
    /// Creates a new set of observables.
    pub fn new(azimuth: Radians, elevation: Radians, range: f64, range_rate: f64) -> Self {
        Observables {
            azimuth,
            elevation,
            range,
            range_rate,
        }
    }
    /// Returns the azimuth angle in radians.
    pub fn azimuth(&self) -> Radians {
        self.azimuth
    }

    /// Returns the elevation angle in radians.
    pub fn elevation(&self) -> Radians {
        self.elevation
    }

    /// Returns the slant range in meters.
    pub fn range(&self) -> f64 {
        self.range
    }

    /// Returns the range rate in meters per second.
    pub fn range_rate(&self) -> f64 {
        self.range_rate
    }
}

/// A location on the surface of a celestial body.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroundLocation<B: TrySpheroid> {
    coordinates: LonLatAlt,
    body: B,
}

/// Type alias for a ground location with a dynamic origin.
pub type DynGroundLocation = GroundLocation<DynOrigin>;

/// Infallible constructor — requires compile-time `Spheroid` guarantee.
impl<B: Spheroid> GroundLocation<B> {
    /// Creates a new ground location on a body that is guaranteed to be a spheroid.
    pub fn new(coordinates: LonLatAlt, body: B) -> Self {
        GroundLocation { coordinates, body }
    }
}

/// Fallible constructor — for `DynOrigin` and other `TrySpheroid` types.
impl<B: TrySpheroid> GroundLocation<B> {
    /// Creates a new ground location, returning an error if the body has no spheroid.
    pub fn try_new(coordinates: LonLatAlt, body: B) -> Result<Self, &'static str> {
        if body.try_equatorial_radius().is_err() {
            return Err("no spheroid");
        }
        Ok(GroundLocation { coordinates, body })
    }
}

impl<B: TrySpheroid + Into<DynOrigin>> GroundLocation<B> {
    /// Converts the ground location into a dynamic representation.
    pub fn into_dyn(self) -> DynGroundLocation {
        GroundLocation {
            coordinates: self.coordinates,
            body: self.body.into(),
        }
    }
}

impl<B: TrySpheroid> GroundLocation<B> {
    /// Returns the central body.
    pub fn origin(&self) -> B
    where
        B: Clone,
    {
        self.body.clone()
    }

    /// Returns the geodetic coordinates.
    pub fn coordinates(&self) -> LonLatAlt {
        self.coordinates
    }

    /// Returns the longitude in radians.
    pub fn longitude(&self) -> f64 {
        self.coordinates.lon().to_radians()
    }

    /// Returns the latitude in radians.
    pub fn latitude(&self) -> f64 {
        self.coordinates.lat().to_radians()
    }

    /// Returns altitude in km (for backward compatibility).
    pub fn altitude(&self) -> f64 {
        self.coordinates.alt().to_kilometers()
    }

    fn equatorial_radius(&self) -> Distance {
        self.body
            .try_equatorial_radius()
            .expect("equatorial radius should be available")
    }

    fn flattening(&self) -> f64 {
        self.body
            .try_flattening()
            .expect("flattening should be available")
    }

    /// Returns the body-fixed Cartesian position in meters.
    pub fn body_fixed_position(&self) -> DVec3 {
        self.coordinates
            .to_body_fixed(self.equatorial_radius(), self.flattening())
    }

    /// Returns the rotation matrix from body-fixed to topocentric (SEZ) frame.
    pub fn rotation_to_topocentric(&self) -> DMat3 {
        self.coordinates.rotation_to_topocentric()
    }

    /// Computes topocentric observables from raw body-fixed position and velocity vectors.
    pub fn compute_observables(&self, state_position: DVec3, state_velocity: DVec3) -> Observables {
        let rot = self.rotation_to_topocentric();
        let position = rot * (state_position - self.body_fixed_position());
        let velocity = rot * state_velocity;
        let range = position.length();
        let range_rate = position.dot(velocity) / range;
        let elevation = (position.z / range).asin();
        let azimuth = position.y.atan2(-position.x);
        Observables {
            azimuth,
            elevation,
            range,
            range_rate,
        }
    }

    /// Computes topocentric observables from a Cartesian orbit in the body-fixed frame.
    pub fn observables<T: TimeScale + Copy>(
        &self,
        state: CartesianOrbit<T, B, Iau<B>>,
    ) -> Observables
    where
        B: RotationalElements + Copy,
    {
        self.compute_observables(state.position(), state.velocity())
    }

    /// Computes topocentric observables from a dynamic Cartesian orbit.
    pub fn observables_dyn(&self, state: crate::orbits::DynCartesianOrbit) -> Observables {
        self.compute_observables(state.position(), state.velocity())
    }
}

/// Errors that can occur during ground propagation.
#[derive(Debug, Error)]
pub enum GroundPropagatorError {
    /// A frame transformation failed.
    #[error("frame transformation error: {0}")]
    FrameTransformation(String),
    /// A trajectory construction error occurred.
    #[error(transparent)]
    Trajectory(#[from] TrajectorError),
}

/// Propagator that produces a stationary body-fixed trajectory for a ground location.
pub struct GroundPropagator<B: TrySpheroid, R: ReferenceFrame> {
    location: GroundLocation<B>,
    frame: R,
    step: Option<TimeDelta>,
}

/// Type alias for a ground propagator with dynamic origin and frame.
pub type DynGroundPropagator = GroundPropagator<DynOrigin, DynFrame>;

/// Typed constructor -- for static bodies with `Spheroid + RotationalElements`.
impl<B: Spheroid + RotationalElements> GroundPropagator<B, Iau<B>> {
    /// Creates a new ground propagator in the body's IAU frame.
    pub fn new(location: GroundLocation<B>) -> Self
    where
        B: Copy,
    {
        let frame = Iau::new(location.body);
        GroundPropagator {
            location,
            frame,
            step: None,
        }
    }
}

/// Fallible constructor for `DynOrigin`.
impl GroundPropagator<DynOrigin, DynFrame> {
    /// Creates a new ground propagator, returning an error if the body has no spheroid.
    pub fn try_new(location: GroundLocation<DynOrigin>) -> Result<Self, &'static str> {
        if location.body.try_equatorial_radius().is_err() {
            return Err("no spheroid");
        }
        let frame = DynFrame::Iau(location.body);
        Ok(GroundPropagator {
            location,
            frame,
            step: None,
        })
    }
}

impl<B: TrySpheroid, R: ReferenceFrame> GroundPropagator<B, R> {
    /// Sets the propagation time step.
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = Some(step);
        self
    }

    /// Returns a reference to the underlying ground location.
    pub fn location(&self) -> &GroundLocation<B> {
        &self.location
    }

    /// Compute the body-fixed state at a single time.
    pub fn state_at<T: TimeScale + Copy>(&self, time: Time<T>) -> CartesianOrbit<T, B, R>
    where
        B: Copy,
        R: Copy,
    {
        let pos = self.location.body_fixed_position();
        CartesianOrbit::new(
            Cartesian::from_vecs(pos, DVec3::ZERO),
            time,
            self.location.body,
            self.frame,
        )
    }
}

/// Single `Propagator` impl covers both typed and Dyn paths.
impl<T, B, R> Propagator<T, B> for GroundPropagator<B, R>
where
    T: TimeScale + Copy + PartialOrd,
    B: TrySpheroid + lox_bodies::Origin + Copy,
    R: ReferenceFrame + Copy,
{
    type Frame = R;
    type Error = GroundPropagatorError;

    fn state_at(&self, time: Time<T>) -> Result<CartesianOrbit<T, B, R>, GroundPropagatorError> {
        Ok(self.state_at(time))
    }

    fn propagate(&self, interval: TimeInterval<T>) -> Result<Trajectory<T, B, R>, Self::Error> {
        let pos = self.location.body_fixed_position();
        let step = self.step.unwrap_or(TimeDelta::from_seconds(60));
        let states: Vec<_> = interval
            .step_by(step)
            .map(|t| {
                CartesianOrbit::new(
                    Cartesian::from_vecs(pos, DVec3::ZERO),
                    t,
                    self.location.body,
                    self.frame,
                )
            })
            .collect();
        Trajectory::try_new(states).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use lox_bodies::Earth;
    use lox_core::coords::Cartesian;
    use lox_frames::Icrf;
    use lox_frames::providers::DefaultRotationProvider;
    use lox_test_utils::assert_approx_eq;
    use lox_time::intervals::Interval;
    use lox_time::time_scales::Tdb;
    use lox_time::utc::Utc;
    use lox_time::{Time, time, utc};

    use super::*;

    #[test]
    fn test_ground_location_to_body_fixed() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let location = GroundLocation::new(coords, Earth);
        let expected = DVec3::new(4846130.017870638, -370132.8551351891, 4116364.272747229);
        assert_approx_eq!(location.body_fixed_position(), expected);
    }

    #[test]
    fn test_ground_location_rotation_to_topocentric() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let location = GroundLocation::new(coords, Earth);
        let act = location.rotation_to_topocentric();
        let exp = DMat3::from_cols(
            DVec3::new(0.6469358921661584, 0.07615519584215287, 0.7587320591443464),
            DVec3::new(
                -0.049411020334552434,
                0.9970959763965771,
                -0.05794967578213965,
            ),
            DVec3::new(-0.7609418522440956, 0.0, 0.6488200809957448),
        );
        assert_approx_eq!(exp, act);
    }

    #[test]
    fn test_ground_location_observables() {
        let coords = LonLatAlt::from_degrees(-4.0, 41.0, 0.0).unwrap();
        let location = GroundLocation::new(coords, Earth);
        let position = DVec3::new(3359927.0, -2398072.0, 5153000.0);
        let velocity = DVec3::new(5065.7, 5485.0, -744.0);
        let time = time!(Tdb, 2012, 7, 1).unwrap();
        let state = CartesianOrbit::new(
            Cartesian::from_vecs(position, velocity),
            time,
            Earth,
            Iau::new(Earth),
        );
        let observables = location.observables(state);
        let expected_range = 2707700.0;
        let expected_range_rate = -7160.0;
        let expected_azimuth = -53.418f64.to_radians();
        let expected_elevation = -7.077f64.to_radians();
        assert_approx_eq!(observables.range, expected_range, rtol <= 1e-2);
        assert_approx_eq!(observables.range_rate, expected_range_rate, rtol <= 1e-2);
        assert_approx_eq!(observables.azimuth, expected_azimuth, rtol <= 1e-2);
        assert_approx_eq!(observables.elevation, expected_elevation, rtol <= 1e-2);
    }

    #[test]
    fn test_ground_propagator_body_fixed() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let location = GroundLocation::new(coords, Earth);
        let propagator = GroundPropagator::new(location.clone());
        let time = utc!(2022, 1, 31, 23).unwrap().to_time();
        let t1 = time + TimeDelta::from_minutes(5);
        let interval = Interval::new(time, t1);
        let traj = propagator.propagate(interval).unwrap();
        // All states should have the same body-fixed position
        let expected = location.body_fixed_position();
        for state in traj.states() {
            assert_approx_eq!(state.position(), expected);
            assert_approx_eq!(state.velocity(), DVec3::ZERO);
        }
    }

    #[test]
    fn test_ground_propagator_in_icrf() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let location = GroundLocation::new(coords, Earth);
        let propagator = GroundPropagator::new(location);
        let time = utc!(2022, 1, 31, 23).unwrap().to_time();
        let t1 = time + TimeDelta::from_minutes(5);
        let interval = Interval::new(time, t1);
        let traj = propagator
            .propagate(interval)
            .unwrap()
            .into_frame(Icrf, &DefaultRotationProvider)
            .unwrap();
        let state = traj.states()[0];
        let expected = DVec3::new(-1765953.5510583583, 4524585.984442561, 4120189.198495323);
        assert_approx_eq!(state.position(), expected);
    }

    #[test]
    fn test_try_new_with_static_body() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let location = GroundLocation::try_new(coords, Earth).unwrap();
        assert_approx_eq!(location.longitude(), -4.3676f64.to_radians());
        assert_approx_eq!(location.latitude(), 40.4527f64.to_radians());
        assert_approx_eq!(location.altitude(), 0.0);
    }

    #[test]
    fn test_try_new_with_dyn_origin() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let location = GroundLocation::try_new(coords, DynOrigin::Earth).unwrap();
        assert_eq!(location.origin(), DynOrigin::Earth);
    }

    #[test]
    fn test_try_new_rejects_non_spheroid() {
        let coords = LonLatAlt::from_degrees(0.0, 0.0, 0.0).unwrap();
        let result = GroundLocation::try_new(coords, DynOrigin::SolarSystemBarycenter);
        assert!(result.is_err());
    }

    #[test]
    fn test_into_dyn_ground_location() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let location = GroundLocation::new(coords, Earth);
        let dyn_location = location.into_dyn();
        assert_eq!(dyn_location.origin(), DynOrigin::Earth);
        assert_approx_eq!(dyn_location.longitude(), -4.3676f64.to_radians());
        assert_approx_eq!(dyn_location.latitude(), 40.4527f64.to_radians());
    }

    #[test]
    fn test_ground_propagator_try_new_with_dyn_origin() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let location = GroundLocation::try_new(coords, DynOrigin::Earth).unwrap();
        let propagator = GroundPropagator::try_new(location).unwrap();
        let time = utc!(2022, 1, 31, 23).unwrap().to_time();
        let t1 = time + TimeDelta::from_minutes(5);
        let interval = Interval::new(time, t1);
        let traj = propagator
            .propagate(interval)
            .unwrap()
            .into_frame(DynFrame::Icrf, &DefaultRotationProvider)
            .unwrap();
        let state = traj.states()[0];
        // Same result as the static version
        let expected = DVec3::new(-1765953.5510583583, 4524585.984442561, 4120189.198495323);
        assert_approx_eq!(state.position(), expected);
    }
}
