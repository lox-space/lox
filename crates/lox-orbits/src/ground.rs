/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::f64::consts::FRAC_PI_2;

use crate::propagators::Propagator;
use crate::states::{DynState, State};
use crate::trajectories::{DynTrajectory, Trajectory, TrajectoryError};
use glam::{DMat3, DVec3};
use lox_bodies::{DynOrigin, RotationalElements, Spheroid, TrySpheroid};
use lox_frames::{DynFrame, Iau, Icrf, TryRotateTo};
use lox_time::time_scales::offsets::{DefaultOffsetProvider, TryOffset};
use lox_time::time_scales::{Tdb, TimeScale};
use lox_time::{DynTime, Time};
use lox_units::types::units::Radians;
use thiserror::Error;


pub trait Observer {
    fn compute_observables(&self, target_state: DynState) -> Observables;
    fn reference_position(&self) -> DVec3;
    fn reference_velocity(&self) -> DVec3;
}

#[derive(Clone, Debug)]
pub struct Observables {
    azimuth: Radians,
    elevation: Radians,
    range: f64,
    range_rate: f64,
    angular_velocity: f64,
}

impl Observables {
    pub fn new(azimuth: Radians, elevation: Radians, range: f64, range_rate: f64, angular_velocity: f64) -> Self {
        Observables {
            azimuth,
            elevation,
            range,
            range_rate,
            angular_velocity
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

    pub fn angular_velocity(&self) -> f64 {
        self.angular_velocity
    }
}

#[derive(Clone, Debug)]
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
    }

    fn flattening(&self) -> f64 {
        self.body
            .try_flattening()
            .expect("flattening should be available")
    }

    pub fn body_fixed_position(&self) -> DVec3 {
        let alt = self.altitude;
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

    pub fn observables<T: TimeScale + Clone>(&self, state: State<T, B, Iau<B>>) -> Observables
    where
        B: RotationalElements + Clone,
    {
        let rot = self.rotation_to_topocentric();
        let position = rot * (state.position() - self.body_fixed_position());
        let velocity = rot * state.velocity();
        let omega = position.cross(velocity) / position.length_squared();
        // Angular velocity magnitude, radians/second
        let angular_velocity = omega.length();

        let range = position.length();
        let range_rate = position.dot(velocity) / range;
        let elevation = (position.z / range).asin();
        let azimuth = position.y.atan2(-position.x);
        Observables {
            azimuth,
            elevation,
            range,
            range_rate,
            angular_velocity
        }
    }

    pub fn observables_dyn(&self, state: DynState) -> Observables {
        let rot = self.rotation_to_topocentric();
        let position = rot * (state.position() - self.body_fixed_position());
        let velocity = rot * state.velocity();
        let omega = position.cross(velocity) / position.length_squared();
        // Angular velocity magnitude, radians/second
        let angular_velocity = omega.length();
        let range = position.length();
        let range_rate = position.dot(velocity) / range;
        let elevation = (position.z / range).asin();
        let azimuth = position.y.atan2(-position.x);
        Observables {
            azimuth,
            elevation,
            range,
            range_rate,
            angular_velocity,
        }
    }
}

impl<B: TrySpheroid> Observer for GroundLocation<B> {
    fn compute_observables(&self, target_state: DynState) -> Observables {
        self.observables_dyn(target_state)
    }

    fn reference_position(&self) -> DVec3 {
        self.body_fixed_position()
    }

    fn reference_velocity(&self) -> DVec3 {
        DVec3::ZERO
    }
}

#[derive(Debug, Error)]
pub enum GroundPropagatorError {
    #[error("frame transformation error: {0}")]
    FrameTransformation(String),
    #[error(transparent)]
    Trajectory(#[from] TrajectoryError),
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

    pub fn propagate_dyn(&self, time: DynTime) -> Result<DynState, GroundPropagatorError> {
        let s = State::new(
            time,
            self.location.body_fixed_position(),
            DVec3::ZERO,
            self.location.body,
            DynFrame::Iau(self.location.body),
        );
        let rot = s
            .reference_frame()
            .try_rotation(DynFrame::Icrf, time, &DefaultOffsetProvider)
            .map_err(|err| GroundPropagatorError::FrameTransformation(err.to_string()))?;
        let (r1, v1) = rot.rotate_state(s.position(), s.velocity());
        Ok(State::new(time, r1, v1, self.location.body, DynFrame::Icrf))
    }

    pub fn propagate_all_dyn(
        &self,
        times: impl IntoIterator<Item = DynTime>,
    ) -> Result<DynTrajectory, GroundPropagatorError> {
        let mut states = vec![];
        for time in times {
            let state = self.propagate_dyn(time)?;
            states.push(state);
        }
        Ok(Trajectory::new(&states)?)
    }
}

impl<T, O> Propagator<T, O, Icrf> for GroundPropagator<O>
where
    T: TimeScale + Copy,
    O: Spheroid + RotationalElements + Clone,
    DefaultOffsetProvider: TryOffset<T, Tdb>,
{
    type Error = GroundPropagatorError;

    fn propagate(&self, time: Time<T>) -> Result<State<T, O, Icrf>, Self::Error> {
        let s = State::new(
            time,
            self.location.body_fixed_position(),
            DVec3::ZERO,
            self.location.body.clone(),
            Iau(self.location.body.clone()),
        );
        let rot = s
            .reference_frame()
            .try_rotation(Icrf, time, &DefaultOffsetProvider)
            .map_err(|err| GroundPropagatorError::FrameTransformation(err.to_string()))?;
        let (r1, v1) = rot.rotate_state(s.position(), s.velocity());
        Ok(State::new(time, r1, v1, self.location.body.clone(), Icrf))
    }
}

// this SatelliteObserver code does not belong in this ground.rs file, at all.
#[derive(Clone, Debug)]
pub struct SatelliteObserver {
    pub trajectory: DynTrajectory,
}

impl SatelliteObserver {
    pub fn new(trajectory: DynTrajectory) -> Self {
        Self { trajectory }
    }

    /// Compute observables from this satellite to a target satellite
    pub fn observables_to_satellite(&self, time: DynTime, target_state: DynState) -> Observables {
        let observer_state = self.trajectory.interpolate_at(time);

        // Relative position and velocity vectors
        let relative_position = target_state.position() - observer_state.position();
        let relative_velocity = target_state.velocity() - observer_state.velocity();

        // Angular velocity vector (orbital rate)
        let omega = relative_position.cross(relative_velocity) / relative_position.length_squared();
        // Angular velocity magnitude, radians/second
        let angular_velocity = omega.length();

        // Range and range rate
        let range = relative_position.length();
        let range_rate = relative_position.dot(relative_velocity) / range;

        // For satellite-to-satellite, we can use the observer's velocity vector
        // to define a local reference frame
        let observer_velocity_unit = observer_state.velocity().normalize();
        let observer_position_unit = observer_state.position().normalize();

        // Create a local coordinate system
        // Z-axis: radial direction (towards Earth center)
        let z_axis = -observer_position_unit;
        // Y-axis: cross-track (perpendicular to orbital plane)
        let y_axis = observer_position_unit.cross(observer_velocity_unit).normalize();
        // X-axis: along-track direction
        let x_axis = y_axis.cross(z_axis);

        // Transform relative position to local frame
        let local_position = DVec3::new(
            relative_position.dot(x_axis),
            relative_position.dot(y_axis),
            relative_position.dot(z_axis),
        );

        // Calculate elevation and azimuth in this local frame
        let elevation = (local_position.z / range).asin();
        let azimuth = local_position.y.atan2(local_position.x);

        Observables::new(azimuth, elevation, range, range_rate, angular_velocity)
    }
}

impl Observer for SatelliteObserver {
    fn compute_observables(&self, target_state: DynState) -> Observables {
        self.observables_to_satellite(target_state.time(), target_state)
    }

    fn reference_position(&self) -> DVec3 {
        // This would need the time context, so this trait might need adjustment
        // For now, return zero - this method might need rethinking
        DVec3::ZERO
    }

    fn reference_velocity(&self) -> DVec3 {
        DVec3::ZERO
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use lox_bodies::Earth;
    use lox_math::assert_close;
    use lox_math::is_close::IsClose;
    use lox_time::utc::Utc;
    use lox_time::{Time, time, utc};

    use super::*;

    #[test]
    fn test_ground_location_to_body_fixed() {
        let longitude = -4.3676f64.to_radians();
        let latitude = 40.4527f64.to_radians();
        let location = GroundLocation::new(longitude, latitude, 0.0, Earth);
        let expected = DVec3::new(4846.130017870638, -370.1328551351891, 4116.364272747229);
        assert_close!(location.body_fixed_position(), expected);
    }

    #[test]
    fn test_ground_location_rotation_to_topocentric() {
        let longitude = -4.3676f64.to_radians();
        let latitude = 40.4527f64.to_radians();
        let location = GroundLocation::new(longitude, latitude, 0.0, Earth);
        let rot = location.rotation_to_topocentric();
        let expected = DMat3::from_cols(
            DVec3::new(0.6469358921661584, 0.07615519584215287, 0.7587320591443464),
            DVec3::new(
                -0.049411020334552434,
                0.9970959763965771,
                -0.05794967578213965,
            ),
            DVec3::new(-0.7609418522440956, 0.0, 0.6488200809957448),
        );
        assert_close!(rot.x_axis, expected.x_axis);
        assert_close!(rot.y_axis, expected.y_axis);
        assert_close!(rot.z_axis, expected.z_axis);
    }

    #[test]
    fn test_ground_location_observables() {
        let longitude = -4f64.to_radians();
        let latitude = 41f64.to_radians();
        let location = GroundLocation::new(longitude, latitude, 0.0, Earth);
        let position = DVec3::new(3359.927, -2398.072, 5153.0);
        let velocity = DVec3::new(5.0657, 5.485, -0.744);
        let time = time!(Tdb, 2012, 7, 1).unwrap();
        let state = State::new(time, position, velocity, Earth, Iau(Earth));
        let observables = location.observables(state);
        let expected_range = 2707.7;
        let expected_range_rate = -7.16;
        let expected_azimuth = -53.418f64.to_radians();
        let expected_elevation = -7.077f64.to_radians();
        assert_float_eq!(observables.range, expected_range, rel <= 1e-2);
        assert_float_eq!(observables.range_rate, expected_range_rate, rel <= 1e-2);
        assert_float_eq!(observables.azimuth, expected_azimuth, rel <= 1e-2);
        assert_float_eq!(observables.elevation, expected_elevation, rel <= 1e-2);
    }

    #[test]
    fn test_ground_propagator() {
        let longitude = -4.3676f64.to_radians();
        let latitude = 40.4527f64.to_radians();
        let location = GroundLocation::new(longitude, latitude, 0.0, Earth);
        let propagator = GroundPropagator::new(location);
        let time = utc!(2022, 1, 31, 23).unwrap().to_time();
        let expected = DVec3::new(-1765.9535510583582, 4524.585984442561, 4120.189198495323);
        let state = propagator.propagate(time).unwrap();
        assert_close!(state.position(), expected);
    }
}
