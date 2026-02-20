// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::f64::consts::FRAC_PI_2;

use crate::orbits::{CartesianOrbit, DynCartesianOrbit, TrajectorError, Trajectory};
use crate::propagators::Propagator;
use glam::{DMat3, DVec3};
use lox_bodies::{DynOrigin, RotationalElements, Spheroid, TrySpheroid};
use lox_core::coords::Cartesian;
use lox_core::types::units::Radians;
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::TryRotation;
use lox_frames::{DynFrame, Iau, Icrf};
use lox_time::time_scales::TimeScale;
use lox_time::{DynTime, Time};
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
    longitude: f64,
    latitude: f64,
    altitude: f64,
    body: B,
}

pub type DynGroundLocation = GroundLocation<DynOrigin>;

impl<B: Spheroid> GroundLocation<B> {
    pub fn new(longitude: f64, latitude: f64, altitude: f64, body: B) -> Self {
        GroundLocation {
            longitude,
            latitude,
            altitude,
            body,
        }
    }
}

impl DynGroundLocation {
    pub fn with_dynamic(
        longitude: f64,
        latitude: f64,
        altitude: f64,
        body: DynOrigin,
    ) -> Result<Self, &'static str> {
        if body.try_equatorial_radius().is_err() {
            return Err("no spheroid");
        }
        Ok(GroundLocation {
            longitude,
            latitude,
            altitude,
            body,
        })
    }
}

impl<B: TrySpheroid> GroundLocation<B> {
    pub fn origin(&self) -> B
    where
        B: Clone,
    {
        self.body.clone()
    }

    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    pub fn altitude(&self) -> f64 {
        self.altitude
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
        // Convert altitude from km to meters to match the equatorial radius
        let alt = self.altitude * 1e3;
        let (lon_sin, lon_cos) = self.longitude.sin_cos();
        let (lat_sin, lat_cos) = self.latitude.sin_cos();
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

    pub fn observables<T: TimeScale + Copy>(
        &self,
        state: CartesianOrbit<T, B, Iau<B>>,
    ) -> Observables
    where
        B: RotationalElements + Copy,
    {
        let rot = self.rotation_to_topocentric();
        let position = rot * (state.position() - self.body_fixed_position());
        let velocity = rot * state.velocity();
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

    pub fn observables_dyn(&self, state: DynCartesianOrbit) -> Observables {
        let rot = self.rotation_to_topocentric();
        let position = rot * (state.position() - self.body_fixed_position());
        let velocity = rot * state.velocity();
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
}

#[derive(Debug, Error)]
pub enum GroundPropagatorError {
    #[error("frame transformation error: {0}")]
    FrameTransformation(String),
    #[error(transparent)]
    Trajectory(#[from] TrajectorError),
}

pub struct GroundPropagator<B: TrySpheroid> {
    location: GroundLocation<B>,
}

pub type DynGroundPropagator = GroundPropagator<DynOrigin>;

impl<B> GroundPropagator<B>
where
    B: Spheroid,
{
    pub fn new(location: GroundLocation<B>) -> Self {
        GroundPropagator { location }
    }
}

impl DynGroundPropagator {
    pub fn with_dynamic(location: DynGroundLocation) -> Self {
        GroundPropagator { location }
    }

    pub fn propagate_dyn(&self, time: DynTime) -> Result<DynCartesianOrbit, GroundPropagatorError> {
        let body_fixed_frame = DynFrame::Iau(self.location.body);
        let rot = DefaultRotationProvider
            .try_rotation(body_fixed_frame, DynFrame::Icrf, time)
            .map_err(|err| GroundPropagatorError::FrameTransformation(err.to_string()))?;
        let (r1, v1) = rot.rotate_state(self.location.body_fixed_position(), DVec3::ZERO);
        Ok(CartesianOrbit::new(
            Cartesian::from_vecs(r1, v1),
            time,
            self.location.body,
            DynFrame::Icrf,
        ))
    }

    pub fn propagate_all_dyn(
        &self,
        times: impl IntoIterator<Item = DynTime>,
    ) -> Result<crate::orbits::DynTrajectory, GroundPropagatorError> {
        let mut states = vec![];
        for time in times {
            let state = self.propagate_dyn(time)?;
            states.push(state);
        }
        Ok(Trajectory::new(states))
    }
}

impl<T, O> Propagator<T, O, Icrf> for GroundPropagator<O>
where
    T: TimeScale + Copy,
    O: Spheroid + RotationalElements + Copy,
    DefaultRotationProvider: TryRotation<Iau<O>, Icrf, T>,
{
    type Error = GroundPropagatorError;

    fn propagate(&self, time: Time<T>) -> Result<CartesianOrbit<T, O, Icrf>, Self::Error> {
        let body_fixed_frame = Iau::new(self.location.body);
        let rot = DefaultRotationProvider
            .try_rotation(body_fixed_frame, Icrf, time)
            .map_err(|err| GroundPropagatorError::FrameTransformation(err.to_string()))?;
        let (r1, v1) = rot.rotate_state(self.location.body_fixed_position(), DVec3::ZERO);
        Ok(CartesianOrbit::new(
            Cartesian::from_vecs(r1, v1),
            time,
            self.location.body,
            Icrf,
        ))
    }
}

#[cfg(test)]
mod tests {
    use lox_bodies::Earth;
    use lox_core::coords::Cartesian;
    use lox_test_utils::assert_approx_eq;
    use lox_time::time_scales::Tdb;
    use lox_time::utc::Utc;
    use lox_time::{Time, time, utc};

    use super::*;

    #[test]
    fn test_ground_location_to_body_fixed() {
        let longitude = -4.3676f64.to_radians();
        let latitude = 40.4527f64.to_radians();
        let location = GroundLocation::new(longitude, latitude, 0.0, Earth);
        let expected = DVec3::new(4846130.017870638, -370132.8551351891, 4116364.272747229);
        assert_approx_eq!(location.body_fixed_position(), expected);
    }

    #[test]
    fn test_ground_location_rotation_to_topocentric() {
        let longitude = -4.3676f64.to_radians();
        let latitude = 40.4527f64.to_radians();
        let location = GroundLocation::new(longitude, latitude, 0.0, Earth);
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
        let longitude = -4f64.to_radians();
        let latitude = 41f64.to_radians();
        let location = GroundLocation::new(longitude, latitude, 0.0, Earth);
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
    fn test_ground_propagator() {
        let longitude = -4.3676f64.to_radians();
        let latitude = 40.4527f64.to_radians();
        let location = GroundLocation::new(longitude, latitude, 0.0, Earth);
        let propagator = GroundPropagator::new(location);
        let time = utc!(2022, 1, 31, 23).unwrap().to_time();
        let expected = DVec3::new(-1765953.5510583583, 4524585.984442561, 4120189.198495323);
        let state = propagator.propagate(time).unwrap();
        assert_approx_eq!(state.position(), expected);
    }
}
