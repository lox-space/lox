/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::f64::consts::FRAC_PI_2;

use crate::frames::{DynFrame, Iau, Icrf, TryRotateTo};
use crate::propagators::Propagator;
use crate::states::{DynState, State};
use crate::trajectories::{DynTrajectory, Trajectory, TrajectoryError};
use glam::{DMat3, DVec3};
use lox_bodies::{DynOrigin, RotationalElements, Spheroid, TrySpheroid};
use lox_math::types::units::Radians;
use lox_time::time_scales::TryToScale;
use lox_time::time_scales::{Tdb, TimeScale};
use lox_time::ut1::DeltaUt1TaiProvider;
use lox_time::{DynTime, Time};
use thiserror::Error;

#[derive(Clone, Debug)]
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

    pub fn observables_dyn(&self, state: DynState) -> Observables {
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
    Trajectory(#[from] TrajectoryError),
}

pub struct GroundPropagator<B: TrySpheroid, P> {
    location: GroundLocation<B>,
    // FIXME: We should not take ownership of the provider here
    provider: Option<P>,
}

pub type DynGroundPropagator<P> = GroundPropagator<DynOrigin, P>;

impl<B, P> GroundPropagator<B, P>
where
    B: Spheroid,
{
    pub fn new(location: GroundLocation<B>, provider: Option<P>) -> Self {
        GroundPropagator { location, provider }
    }
}

impl<P: DeltaUt1TaiProvider> DynGroundPropagator<P> {
    pub fn with_dynamic(location: DynGroundLocation, provider: Option<P>) -> Self {
        GroundPropagator { location, provider }
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
            .try_rotation(DynFrame::Icrf, time, self.provider.as_ref())
            .map_err(|err| GroundPropagatorError::FrameTransformation(err.to_string()))?;
        let (r1, v1) = rot.rotate_state(s.position(), s.velocity());
        Ok(State::new(time, r1, v1, self.location.body, DynFrame::Icrf))
    }

    pub(crate) fn propagate_all_dyn(
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

impl<T, O, P> Propagator<T, O, Icrf> for GroundPropagator<O, P>
where
    T: TimeScale + TryToScale<Tdb, P> + Clone,
    O: Spheroid + RotationalElements + Clone,
{
    type Error = GroundPropagatorError;

    fn propagate(&self, time: Time<T>) -> Result<State<T, O, Icrf>, Self::Error> {
        let s = State::new(
            time.clone(),
            self.location.body_fixed_position(),
            DVec3::ZERO,
            self.location.body.clone(),
            Iau(self.location.body.clone()),
        );
        let rot = s
            .reference_frame()
            .try_rotation(Icrf, time.clone(), self.provider.as_ref())
            .map_err(|err| GroundPropagatorError::FrameTransformation(err.to_string()))?;
        let (r1, v1) = rot.rotate_state(s.position(), s.velocity());
        Ok(State::new(time, r1, v1, self.location.body.clone(), Icrf))
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
        let propagator = GroundPropagator::new(location, None::<()>);
        let time = utc!(2022, 1, 31, 23).unwrap().to_time();
        let expected = DVec3::new(-1765.9535510583582, 4524.585984442561, 4120.189198495323);
        let state = propagator.propagate(time).unwrap();
        assert_close!(state.position(), expected);
    }
}
