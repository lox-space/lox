// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::orbits::{CartesianOrbit, Trajectory, TrajectoryError};
use crate::propagators::Propagator;
use lox_bodies::CoordinateOrigin;
use lox_core::coords::{Cartesian, Ellipsoid, LonLatAlt};
use lox_core::glam::{DMat3, DVec3};
use lox_core::units::{Angle, Distance, Velocity};
use lox_frames::traits::{
    ReferenceEllipsoid, TryReferenceEllipsoid, UndefinedReferenceEllipsoidError, frame_key,
};
use lox_frames::{BodyFixed, Frame, NonBodyFixedFrameError, TryBodyFixed};
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use thiserror::Error;

/// Topocentric observation of a satellite from a ground location.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Observables {
    azimuth: Angle,
    elevation: Angle,
    range: Distance,
    range_rate: Velocity,
}

impl Observables {
    /// Creates a new set of observables.
    pub fn new(azimuth: Angle, elevation: Angle, range: Distance, range_rate: Velocity) -> Self {
        Observables {
            azimuth,
            elevation,
            range,
            range_rate,
        }
    }
    /// Returns the azimuth angle.
    pub fn azimuth(&self) -> Angle {
        self.azimuth
    }

    /// Returns the elevation angle.
    pub fn elevation(&self) -> Angle {
        self.elevation
    }

    /// Returns the slant range.
    pub fn range(&self) -> Distance {
        self.range
    }

    /// Returns the range rate.
    pub fn range_rate(&self) -> Velocity {
        self.range_rate
    }
}

/// A location on the surface of a celestial body.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(
        try_from = "EllipsoidLocationDe<R>",
        bound(deserialize = "R: serde::Deserialize<'de> + TryBodyFixed")
    )
)]
pub struct EllipsoidLocation<R = Frame> {
    coordinates: LonLatAlt,
    ellipsoid: Ellipsoid,
    frame: R,
}

/// Deserialization shadow for [`EllipsoidLocation`].
///
/// Deserializing into the public type directly would bypass the body-fixed
/// check that [`EllipsoidLocation::origin`] relies on.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
#[serde(rename = "EllipsoidLocation")]
struct EllipsoidLocationDe<R> {
    coordinates: LonLatAlt,
    ellipsoid: Ellipsoid,
    frame: R,
}

#[cfg(feature = "serde")]
impl<R: TryBodyFixed> TryFrom<EllipsoidLocationDe<R>> for EllipsoidLocation<R> {
    type Error = NonBodyFixedFrameError;

    fn try_from(de: EllipsoidLocationDe<R>) -> Result<Self, Self::Error> {
        de.frame.try_body_fixed()?;
        Ok(EllipsoidLocation {
            coordinates: de.coordinates,
            ellipsoid: de.ellipsoid,
            frame: de.frame,
        })
    }
}

impl<R> EllipsoidLocation<R> {
    /// Creates a location on the frame's conventional reference ellipsoid.
    pub fn new(coordinates: LonLatAlt, frame: R) -> Self
    where
        R: ReferenceEllipsoid,
    {
        EllipsoidLocation {
            coordinates,
            ellipsoid: frame.reference_ellipsoid(),
            frame,
        }
    }

    /// Creates a location on an explicitly chosen reference ellipsoid.
    ///
    /// Use this to reference coordinates to a different datum, such as a historical
    /// ellipsoid or one whose values differ from the body's.
    pub fn with_ellipsoid(coordinates: LonLatAlt, ellipsoid: Ellipsoid, frame: R) -> Self
    where
        R: BodyFixed,
    {
        EllipsoidLocation {
            coordinates,
            ellipsoid,
            frame,
        }
    }
}

impl<R> EllipsoidLocation<R>
where
    R: TryReferenceEllipsoid,
{
    /// Creates a location on the frame's conventional reference ellipsoid,
    /// returning an error if the frame has none.
    pub fn try_new(
        coordinates: LonLatAlt,
        frame: R,
    ) -> Result<Self, UndefinedReferenceEllipsoidError> {
        let ellipsoid = frame.try_reference_ellipsoid()?;
        Ok(EllipsoidLocation {
            coordinates,
            ellipsoid,
            frame,
        })
    }
}

impl<R> EllipsoidLocation<R>
where
    R: TryBodyFixed,
{
    /// Creates a location on an explicitly chosen reference ellipsoid,
    /// returning an error if the frame is not body-fixed.
    ///
    /// See [`EllipsoidLocation::with_ellipsoid`] for when to override the
    /// frame's conventional ellipsoid.
    pub fn try_with_ellipsoid(
        coordinates: LonLatAlt,
        ellipsoid: Ellipsoid,
        frame: R,
    ) -> Result<Self, NonBodyFixedFrameError> {
        frame.try_body_fixed()?;
        Ok(EllipsoidLocation {
            coordinates,
            ellipsoid,
            frame,
        })
    }
}

impl<R> EllipsoidLocation<R>
where
    R: Into<Frame>,
{
    /// Converts the ground location into a dynamic representation.
    pub fn into_dynamic(self) -> EllipsoidLocation {
        EllipsoidLocation {
            coordinates: self.coordinates,
            ellipsoid: self.ellipsoid,
            frame: self.frame.into(),
        }
    }
}

impl<R> EllipsoidLocation<R> {
    /// Returns the reference frame for this location.
    pub fn frame(&self) -> R
    where
        R: Clone,
    {
        self.frame.clone()
    }

    /// Returns the central body of the location's frame.
    pub fn origin(&self) -> R::Origin
    where
        R: TryBodyFixed,
    {
        self.frame
            .try_origin()
            .expect("validated at EllipsoidLocation construction")
    }

    /// Returns the geodetic coordinates.
    pub fn coordinates(&self) -> LonLatAlt {
        self.coordinates
    }

    /// Returns the geodetic longitude.
    pub fn longitude(&self) -> Angle {
        self.coordinates.lon()
    }

    /// Returns the geodetic latitude.
    pub fn latitude(&self) -> Angle {
        self.coordinates.lat()
    }

    /// Returns the altitude above the reference ellipsoid.
    pub fn altitude(&self) -> Distance {
        self.coordinates.alt()
    }

    /// Returns the ellipsoid for this location.
    pub fn ellipsoid(&self) -> Ellipsoid {
        self.ellipsoid
    }

    /// Returns the body-fixed Cartesian position in meters.
    pub fn body_fixed_position(&self) -> DVec3 {
        self.coordinates.to_body_fixed(&self.ellipsoid())
    }

    /// Returns the rotation matrix from body-fixed to topocentric (SEZ) frame.
    pub fn rotation_to_topocentric(&self) -> DMat3 {
        self.coordinates.rotation_to_topocentric()
    }

    /// Computes topocentric observables from a Cartesian state.
    ///
    /// The state must be centred on this location's body and expressed in its
    /// frame. Neither is enforced by the signature — a mismatch yields silently
    /// wrong observables rather than an error — so both are asserted in debug
    /// builds.
    pub fn observables<O>(&self, state: CartesianOrbit<O, R>) -> Observables
    where
        O: CoordinateOrigin + Copy,
        R: TryBodyFixed + Copy,
    {
        debug_assert_eq!(
            state.origin().id(),
            self.origin().id(),
            "state origin `{}` is not the location's body `{}`",
            state.origin().name(),
            self.origin().name(),
        );
        debug_assert_eq!(
            frame_key(&state.reference_frame()),
            frame_key(&self.frame),
            "state frame `{}` is not the location's frame `{}`",
            state.reference_frame().abbreviation(),
            self.frame.abbreviation(),
        );
        let rot = self.rotation_to_topocentric();
        let position = rot * (state.position() - self.body_fixed_position());
        let velocity = rot * state.velocity();
        let range = position.length();
        let range_rate = position.dot(velocity) / range;
        let elevation = (position.z / range).asin();
        let azimuth = position.y.atan2(-position.x);
        Observables {
            azimuth: Angle::radians(azimuth),
            elevation: Angle::radians(elevation),
            range: Distance::meters(range),
            range_rate: Velocity::meters_per_second(range_rate),
        }
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
    Trajectory(#[from] TrajectoryError),
}

/// Propagator that produces a stationary body-fixed trajectory for a ground location.
pub struct GroundPropagator<R = Frame> {
    location: EllipsoidLocation<R>,
    step: Option<TimeDelta>,
}

impl<R> GroundPropagator<R> {
    /// Creates a new ground propagator.
    pub fn new(location: EllipsoidLocation<R>) -> Self {
        GroundPropagator {
            location,
            step: None,
        }
    }

    /// Sets the propagation time step.
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = Some(step);
        self
    }

    /// Returns a reference to the underlying ground location.
    pub fn location(&self) -> &EllipsoidLocation<R> {
        &self.location
    }

    /// Compute the body-fixed state at a single time.
    pub fn state_at(&self, time: Time) -> CartesianOrbit<R::Origin, R>
    where
        R: TryBodyFixed + Copy,
    {
        let pos = self.location.body_fixed_position();
        CartesianOrbit::new(
            Cartesian::from_vecs(pos, DVec3::ZERO),
            time,
            self.location.origin(),
            self.location.frame,
        )
    }
}

/// Single `Propagator` impl covers both typed and Dyn paths.
impl<R> Propagator<R::Origin> for GroundPropagator<R>
where
    R: TryBodyFixed + Copy,
{
    type Frame = R;
    type Error = GroundPropagatorError;

    fn state_at(&self, time: Time) -> Result<CartesianOrbit<R::Origin, R>, GroundPropagatorError> {
        Ok(self.state_at(time))
    }

    fn propagate(&self, interval: TimeInterval) -> Result<Trajectory<R::Origin, R>, Self::Error> {
        let pos = self.location.body_fixed_position();
        let step = self.step.unwrap_or(TimeDelta::from_seconds(60));
        let states: Vec<_> = interval
            .step_by(step)
            .map(|t| {
                CartesianOrbit::new(
                    Cartesian::from_vecs(pos, DVec3::ZERO),
                    t,
                    self.location.origin(),
                    self.location.frame,
                )
            })
            .collect();
        Trajectory::try_new(states).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;
    use lox_bodies::{Earth, Origin};
    use lox_core::coords::Cartesian;
    use lox_frames::providers::DefaultRotationProvider;
    use lox_frames::{Iau, Icrf};
    use lox_time::intervals::Interval;
    use lox_time::time_scales::Tdb;
    use lox_time::{time, utc};

    use super::*;

    #[test]
    fn test_ground_location_to_body_fixed() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let location = EllipsoidLocation::new(coords, Iau::new(Earth));
        let expected = DVec3::new(4846130.017870638, -370132.8551351891, 4116364.272747229);
        assert_approx_eq!(location.body_fixed_position(), expected);
    }

    #[test]
    fn test_ground_location_rotation_to_topocentric() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let location = EllipsoidLocation::new(coords, Iau::new(Earth));
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
        let location = EllipsoidLocation::new(coords, Iau::new(Earth));
        let position = DVec3::new(3359927.0, -2398072.0, 5153000.0);
        let velocity = DVec3::new(5065.7, 5485.0, -744.0);
        let time = time!(Tdb, 2012, 7, 1).unwrap();
        let state = CartesianOrbit::new(
            Cartesian::from_vecs(position, velocity),
            time.into_dynamic(),
            Earth,
            Iau::new(Earth),
        );
        let observables = location.observables(state);
        let expected_range = Distance::kilometers(2707.7);
        let expected_range_rate = Velocity::kilometers_per_second(-7.16);
        let expected_azimuth = Angle::degrees(-53.418);
        let expected_elevation = Angle::degrees(-7.077);
        assert_approx_eq!(observables.range, expected_range, rtol <= 1e-2);
        assert_approx_eq!(observables.range_rate, expected_range_rate, rtol <= 1e-2);
        assert_approx_eq!(observables.azimuth, expected_azimuth, rtol <= 1e-2);
        assert_approx_eq!(observables.elevation, expected_elevation, rtol <= 1e-2);
    }

    /// `observables` is generic over the state's origin, so a state centred on
    /// the wrong body type-checks. The debug assertion is the only thing
    /// standing between that and silently offset observables.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "is not the location's body")]
    fn test_observables_rejects_mismatched_origin() {
        let coords = LonLatAlt::from_degrees(-4.0, 41.0, 0.0).unwrap();
        let location = EllipsoidLocation::try_new(coords, Frame::Iau(Origin::Earth)).unwrap();
        let state = CartesianOrbit::new(
            Cartesian::from_vecs(DVec3::new(3359927.0, -2398072.0, 5153000.0), DVec3::ZERO),
            time!(Tdb, 2012, 7, 1).unwrap().into_dynamic(),
            Origin::Moon,
            Frame::Iau(Origin::Earth),
        );
        location.observables(state);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "is not the location's frame")]
    fn test_observables_rejects_mismatched_frame() {
        let coords = LonLatAlt::from_degrees(-4.0, 41.0, 0.0).unwrap();
        let location = EllipsoidLocation::try_new(coords, Frame::Iau(Origin::Earth)).unwrap();
        let state = CartesianOrbit::new(
            Cartesian::from_vecs(DVec3::new(3359927.0, -2398072.0, 5153000.0), DVec3::ZERO),
            time!(Tdb, 2012, 7, 1).unwrap().into_dynamic(),
            Origin::Earth,
            Frame::Icrf,
        );
        location.observables(state);
    }

    #[test]
    fn test_ground_propagator_body_fixed() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let location = EllipsoidLocation::new(coords, Iau::new(Earth));
        let propagator = GroundPropagator::new(location.clone());
        let time = utc!(2022, 1, 31, 23).unwrap().to_dynamic_time();
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
        let location = EllipsoidLocation::new(coords, Iau::new(Earth));
        let propagator = GroundPropagator::new(location);
        let time = utc!(2022, 1, 31, 23).unwrap().to_dynamic_time();
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
        let location = EllipsoidLocation::try_new(coords, Iau::new(Earth)).unwrap();
        assert_approx_eq!(location.longitude(), Angle::degrees(-4.3676));
        assert_approx_eq!(location.latitude(), Angle::degrees(40.4527));
        assert_approx_eq!(location.altitude(), Distance::meters(0.0));
    }

    #[test]
    fn test_accessors_are_in_si_units() {
        // `LonLatAlt::from_degrees` takes metres, and the accessors return the
        // same: radians and metres, matching `body_fixed_position`.
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 450.0).unwrap();
        let location = EllipsoidLocation::try_new(coords, Iau::new(Earth)).unwrap();
        assert_approx_eq!(location.altitude(), Distance::meters(450.0));
        assert_approx_eq!(
            location.body_fixed_position().length(),
            location
                .coordinates()
                .to_body_fixed(&location.ellipsoid())
                .length()
        );
    }

    #[test]
    fn test_try_new_with_dynamic_origin() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let location = EllipsoidLocation::try_new(coords, Frame::Iau(Origin::Earth)).unwrap();
        assert_eq!(location.origin(), Origin::Earth);
    }

    #[test]
    fn test_try_new_rejects_non_spheroid() {
        let coords = LonLatAlt::from_degrees(0.0, 0.0, 0.0).unwrap();
        let result = EllipsoidLocation::try_new(coords, Frame::Iau(Origin::Phobos));
        assert!(result.is_err());
    }

    #[test]
    fn test_with_ellipsoid_overrides_the_frame_default() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let default = EllipsoidLocation::new(coords, Iau::new(Earth));
        let overridden =
            EllipsoidLocation::with_ellipsoid(coords, Ellipsoid::WGS84, Iau::new(Earth));
        assert_eq!(overridden.ellipsoid(), Ellipsoid::WGS84);
        assert_ne!(overridden.ellipsoid(), default.ellipsoid());
    }

    #[test]
    fn test_try_with_ellipsoid_rejects_non_body_fixed_frame() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let result = EllipsoidLocation::try_with_ellipsoid(coords, Ellipsoid::WGS84, Frame::Icrf);
        assert!(result.is_err());
    }

    #[test]
    fn test_into_dynamic_ground_location() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let location = EllipsoidLocation::new(coords, Iau::new(Earth));
        let dynamic_location = location.into_dynamic();
        assert_eq!(dynamic_location.origin(), Origin::Earth);
        assert_approx_eq!(dynamic_location.longitude(), Angle::degrees(-4.3676));
        assert_approx_eq!(dynamic_location.latitude(), Angle::degrees(40.4527));
    }

    #[test]
    fn test_ground_propagator_try_new_with_dynamic_origin() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let location = EllipsoidLocation::try_new(coords, Frame::Iau(Origin::Earth)).unwrap();
        let propagator = GroundPropagator::new(location);
        let time = utc!(2022, 1, 31, 23).unwrap().to_dynamic_time();
        let t1 = time + TimeDelta::from_minutes(5);
        let interval = Interval::new(time, t1);
        let traj = propagator
            .propagate(interval)
            .unwrap()
            .into_frame(Frame::Icrf, &DefaultRotationProvider)
            .unwrap();
        let state = traj.states()[0];
        // Same result as the static version
        let expected = DVec3::new(-1765953.5510583583, 4524585.984442561, 4120189.198495323);
        assert_approx_eq!(state.position(), expected);
    }
}
