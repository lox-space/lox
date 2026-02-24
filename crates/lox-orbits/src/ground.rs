// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::f64::consts::FRAC_PI_2;

use crate::orbits::{CartesianOrbit, TrajectorError, Trajectory};
use crate::propagators::Propagator;
use glam::{DMat3, DVec3};
use lox_bodies::{DynOrigin, RotationalElements, Spheroid, TrySpheroid};
use lox_core::coords::{Cartesian, LonLatAlt};
use lox_core::types::units::Radians;
use lox_frames::{DynFrame, Iau, ReferenceFrame};
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::TimeScale;
use thiserror::Error;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Observables {
    azimuth: Radians,
    elevation: Radians,
    range: f64,
    range_rate: f64,
}

impl Observables {
    pub fn new(azimuth: Radians, elevation: Radians, range: f64, range_rate: f64) -> Self {
        Observables {
            azimuth,
            elevation,
            range,
            range_rate,
        }
    }
    pub fn azimuth(&self) -> Radians {
        self.azimuth
    }

    pub fn elevation(&self) -> Radians {
        self.elevation
    }

    pub fn range(&self) -> f64 {
        self.range
    }

    pub fn range_rate(&self) -> f64 {
        self.range_rate
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroundLocation<B: TrySpheroid> {
    coordinates: LonLatAlt,
    body: B,
}

pub type DynGroundLocation = GroundLocation<DynOrigin>;

/// Infallible constructor — requires compile-time `Spheroid` guarantee.
impl<B: Spheroid> GroundLocation<B> {
    pub fn new(coordinates: LonLatAlt, body: B) -> Self {
        GroundLocation { coordinates, body }
    }
}

/// Fallible constructor — for `DynOrigin` and other `TrySpheroid` types.
impl<B: TrySpheroid> GroundLocation<B> {
    pub fn try_new(coordinates: LonLatAlt, body: B) -> Result<Self, &'static str> {
        if body.try_equatorial_radius().is_err() {
            return Err("no spheroid");
        }
        Ok(GroundLocation { coordinates, body })
    }
}

impl<B: TrySpheroid + Into<DynOrigin>> GroundLocation<B> {
    pub fn into_dyn(self) -> DynGroundLocation {
        GroundLocation {
            coordinates: self.coordinates,
            body: self.body.into(),
        }
    }
}

impl<B: TrySpheroid> GroundLocation<B> {
    pub fn origin(&self) -> B
    where
        B: Clone,
    {
        self.body.clone()
    }

    pub fn coordinates(&self) -> LonLatAlt {
        self.coordinates
    }

    pub fn longitude(&self) -> f64 {
        self.coordinates.lon().to_radians()
    }

    pub fn latitude(&self) -> f64 {
        self.coordinates.lat().to_radians()
    }

    /// Returns altitude in km (for backward compatibility).
    pub fn altitude(&self) -> f64 {
        self.coordinates.alt().to_kilometers()
    }

    fn equatorial_radius(&self) -> f64 {
        self.body
            .try_equatorial_radius()
            .expect("equatorial radius should be available")
            .to_meters()
    }

    fn flattening(&self) -> f64 {
        self.body
            .try_flattening()
            .expect("flattening should be available")
    }

    pub fn body_fixed_position(&self) -> DVec3 {
        let alt = self.coordinates.alt().to_meters();
        let (lon_sin, lon_cos) = self.coordinates.lon().sin_cos();
        let (lat_sin, lat_cos) = self.coordinates.lat().sin_cos();
        let f = self.flattening();
        let r_eq = self.equatorial_radius();
        let e = (2.0 * f - f.powi(2)).sqrt();
        let c = r_eq / (1.0 - e.powi(2) * lat_sin.powi(2)).sqrt();
        let s = c * (1.0 - e.powi(2));
        let r_delta = (c + alt) * lat_cos;
        let r_kappa = (s + alt) * lat_sin;
        DVec3::new(r_delta * lon_cos, r_delta * lon_sin, r_kappa)
    }

    pub fn rotation_to_topocentric(&self) -> DMat3 {
        let rot1 = DMat3::from_rotation_z(self.longitude()).transpose();
        let rot2 = DMat3::from_rotation_y(FRAC_PI_2 - self.latitude()).transpose();
        rot2 * rot1
    }

    fn compute_observables(&self, state_position: DVec3, state_velocity: DVec3) -> Observables {
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

    pub fn observables<T: TimeScale + Copy>(
        &self,
        state: CartesianOrbit<T, B, Iau<B>>,
    ) -> Observables
    where
        B: RotationalElements + Copy,
    {
        self.compute_observables(state.position(), state.velocity())
    }

    pub fn observables_dyn(&self, state: crate::orbits::DynCartesianOrbit) -> Observables {
        self.compute_observables(state.position(), state.velocity())
    }
}

#[derive(Debug, Error)]
pub enum GroundPropagatorError {
    #[error("frame transformation error: {0}")]
    FrameTransformation(String),
    #[error(transparent)]
    Trajectory(#[from] TrajectorError),
}

pub struct GroundPropagator<B: TrySpheroid, R: ReferenceFrame> {
    location: GroundLocation<B>,
    frame: R,
    step: Option<TimeDelta>,
}

pub type DynGroundPropagator = GroundPropagator<DynOrigin, DynFrame>;

/// Typed constructor — for static bodies with `Spheroid + RotationalElements`.
impl<B: Spheroid + RotationalElements> GroundPropagator<B, Iau<B>> {
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
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = Some(step);
        self
    }

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
    use lox_core::units::{Angle, Distance};
    use lox_frames::Icrf;
    use lox_frames::providers::DefaultRotationProvider;
    use lox_test_utils::assert_approx_eq;
    use lox_time::intervals::Interval;
    use lox_time::time_scales::Tdb;
    use lox_time::utc::Utc;
    use lox_time::{Time, time, utc};

    use super::*;

    fn lla(lon_deg: f64, lat_deg: f64, alt_m: f64) -> LonLatAlt {
        LonLatAlt::builder()
            .longitude(Angle::degrees(lon_deg))
            .latitude(Angle::degrees(lat_deg))
            .altitude(Distance::meters(alt_m))
            .build()
            .unwrap()
    }

    #[test]
    fn test_ground_location_to_body_fixed() {
        let coords = lla(-4.3676, 40.4527, 0.0);
        let location = GroundLocation::new(coords, Earth);
        let expected = DVec3::new(4846130.017870638, -370132.8551351891, 4116364.272747229);
        assert_approx_eq!(location.body_fixed_position(), expected);
    }

    #[test]
    fn test_ground_location_rotation_to_topocentric() {
        let coords = lla(-4.3676, 40.4527, 0.0);
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
        let coords = lla(-4.0, 41.0, 0.0);
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
        let coords = lla(-4.3676, 40.4527, 0.0);
        let location = GroundLocation::new(coords, Earth);
        let propagator = GroundPropagator::new(location.clone());
        let time = utc!(2022, 1, 31, 23).unwrap().to_time();
        let t1 = time + TimeDelta::from_minutes(5.0);
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
        let coords = lla(-4.3676, 40.4527, 0.0);
        let location = GroundLocation::new(coords, Earth);
        let propagator = GroundPropagator::new(location);
        let time = utc!(2022, 1, 31, 23).unwrap().to_time();
        let t1 = time + TimeDelta::from_minutes(5.0);
        let interval = Interval::new(time, t1);
        let traj = propagator
            .propagate_in_frame(interval, Icrf, &DefaultRotationProvider)
            .unwrap();
        let state = traj.states()[0].clone();
        let expected = DVec3::new(-1765953.5510583583, 4524585.984442561, 4120189.198495323);
        assert_approx_eq!(state.position(), expected);
    }

    #[test]
    fn test_try_new_with_static_body() {
        let coords = lla(-4.3676, 40.4527, 0.0);
        let location = GroundLocation::try_new(coords, Earth).unwrap();
        assert_approx_eq!(location.longitude(), -4.3676f64.to_radians());
        assert_approx_eq!(location.latitude(), 40.4527f64.to_radians());
        assert_approx_eq!(location.altitude(), 0.0);
    }

    #[test]
    fn test_try_new_with_dyn_origin() {
        let coords = lla(-4.3676, 40.4527, 0.0);
        let location = GroundLocation::try_new(coords, DynOrigin::Earth).unwrap();
        assert_eq!(location.origin(), DynOrigin::Earth);
    }

    #[test]
    fn test_try_new_rejects_non_spheroid() {
        let coords = lla(0.0, 0.0, 0.0);
        let result = GroundLocation::try_new(coords, DynOrigin::SolarSystemBarycenter);
        assert!(result.is_err());
    }

    #[test]
    fn test_into_dyn_ground_location() {
        let coords = lla(-4.3676, 40.4527, 0.0);
        let location = GroundLocation::new(coords, Earth);
        let dyn_location = location.into_dyn();
        assert_eq!(dyn_location.origin(), DynOrigin::Earth);
        assert_approx_eq!(dyn_location.longitude(), -4.3676f64.to_radians());
        assert_approx_eq!(dyn_location.latitude(), 40.4527f64.to_radians());
    }

    #[test]
    fn test_ground_propagator_try_new_with_dyn_origin() {
        let coords = lla(-4.3676, 40.4527, 0.0);
        let location = GroundLocation::try_new(coords, DynOrigin::Earth).unwrap();
        let propagator = GroundPropagator::try_new(location).unwrap();
        let time = utc!(2022, 1, 31, 23).unwrap().to_time();
        let t1 = time + TimeDelta::from_minutes(5.0);
        let interval = Interval::new(time, t1);
        let traj = propagator
            .propagate_in_frame(interval, DynFrame::Icrf, &DefaultRotationProvider)
            .unwrap();
        let state = traj.states()[0].clone();
        // Same result as the static version
        let expected = DVec3::new(-1765953.5510583583, 4524585.984442561, 4120189.198495323);
        assert_approx_eq!(state.position(), expected);
    }
}
