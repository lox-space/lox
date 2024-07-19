/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::f64::consts::FRAC_PI_2;

use glam::{DMat3, DVec3};
use thiserror::Error;

use lox_bodies::{RotationalElements, Spheroid};
use lox_math::types::units::Radians;
use lox_time::prelude::Tdb;
use lox_time::transformations::TryToScale;
use lox_time::TimeLike;

use crate::frames::{BodyFixed, CoordinateSystem, FrameTransformationProvider, Icrf, TryToFrame};
use crate::origins::CoordinateOrigin;
use crate::propagators::Propagator;
use crate::states::State;
use crate::trajectories::TrajectoryError;

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
pub struct GroundLocation<B: Spheroid> {
    longitude: f64,
    latitude: f64,
    altitude: f64,
    body: B,
}

impl<B: Spheroid> GroundLocation<B> {
    pub fn new(longitude: f64, latitude: f64, altitude: f64, body: B) -> Self {
        GroundLocation {
            longitude,
            latitude,
            altitude,
            body,
        }
    }

    pub fn with_body<T: Spheroid>(self, body: T) -> GroundLocation<T> {
        GroundLocation {
            longitude: self.longitude,
            latitude: self.latitude,
            altitude: self.altitude,
            body,
        }
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

    pub fn body_fixed_position(&self) -> DVec3 {
        let alt = self.altitude;
        let (lon_sin, lon_cos) = self.longitude.sin_cos();
        let (lat_sin, lat_cos) = self.latitude.sin_cos();
        let f = self.body.flattening();
        let r_eq = self.body.equatorial_radius();
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

    pub fn observables<T: TimeLike + Clone>(&self, state: State<T, B, BodyFixed<B>>) -> Observables
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
}

impl<O: Spheroid + Clone> CoordinateOrigin<O> for GroundLocation<O> {
    fn origin(&self) -> O {
        self.body.clone()
    }
}

#[derive(Debug, Error)]
pub enum GroundPropagatorError {
    #[error("frame transformation error: {0}")]
    FrameTransformationError(String),
    #[error(transparent)]
    TrajectoryError(#[from] TrajectoryError),
}

pub struct GroundPropagator<B: Spheroid, P: FrameTransformationProvider> {
    location: GroundLocation<B>,
    // FIXME: We should not take ownership of the provider here
    provider: P,
}

impl<B, P> GroundPropagator<B, P>
where
    B: Spheroid,
    P: FrameTransformationProvider,
{
    pub fn new(location: GroundLocation<B>, provider: P) -> Self {
        GroundPropagator { location, provider }
    }
}

impl<O, P> CoordinateOrigin<O> for GroundPropagator<O, P>
where
    O: Spheroid + Clone,
    P: FrameTransformationProvider,
{
    fn origin(&self) -> O {
        self.location.body.clone()
    }
}

impl<O, P> CoordinateSystem<Icrf> for GroundPropagator<O, P>
where
    O: Spheroid,
    P: FrameTransformationProvider,
{
    fn reference_frame(&self) -> Icrf {
        Icrf
    }
}

impl<T, O, P> Propagator<T, O, Icrf> for GroundPropagator<O, P>
where
    T: TimeLike + TryToScale<Tdb, P> + Clone,
    O: Spheroid + RotationalElements + Clone,
    P: FrameTransformationProvider,
{
    type Error = GroundPropagatorError;

    fn propagate(&self, time: T) -> Result<State<T, O, Icrf>, Self::Error> {
        State::new(
            time,
            self.location.body_fixed_position(),
            DVec3::ZERO,
            self.location.body.clone(),
            BodyFixed(self.location.body.clone()),
        )
        .try_to_frame(Icrf, &self.provider)
        .map_err(|err| GroundPropagatorError::FrameTransformationError(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use lox_bodies::Earth;
    use lox_math::assert_close;
    use lox_math::is_close::IsClose;
    use lox_time::transformations::ToTai;
    use lox_time::utc::Utc;
    use lox_time::{time, utc, Time};

    use crate::frames::NoOpFrameTransformationProvider;

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
        let state = State::new(time, position, velocity, Earth, BodyFixed(Earth));
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
        let provider = NoOpFrameTransformationProvider;
        let propagator = GroundPropagator::new(location, provider);
        let time = utc!(2022, 1, 31, 23).unwrap().to_tai();
        let expected = DVec3::new(-1765.9535510583582, 4524.585984442561, 4120.189198495323);
        let state = propagator.propagate(time).unwrap();
        assert_close!(state.position(), expected);
    }
}
