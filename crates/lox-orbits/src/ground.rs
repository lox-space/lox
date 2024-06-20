/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use glam::DVec3;
use thiserror::Error;

use lox_bodies::{RotationalElements, Spheroid};
use lox_time::prelude::Tdb;
use lox_time::transformations::TryToScale;
use lox_time::TimeLike;

use crate::frames::{BodyFixed, CoordinateSystem, FrameTransformationProvider, Icrf, TryToFrame};
use crate::origins::CoordinateOrigin;
use crate::propagators::Propagator;
use crate::states::State;
use crate::trajectories::TrajectoryError;

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

    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    pub fn altitude(&self) -> f64 {
        self.altitude
    }

    pub fn to_body_fixed(&self) -> DVec3 {
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
            self.location.to_body_fixed(),
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
    use super::*;
    use crate::frames::NoOpFrameTransformationProvider;
    use lox_bodies::Earth;
    use lox_time::transformations::ToTai;
    use lox_time::utc;
    use lox_time::utc::Utc;
    use lox_utils::assert_close;
    use lox_utils::is_close::IsClose;

    #[test]
    fn test_ground_location_to_body_fixed() {
        let longitude = -4.3676f64.to_radians();
        let latitude = 40.4527f64.to_radians();
        let location = GroundLocation::new(longitude, latitude, 0.0, Earth);
        let expected = DVec3::new(4846.130017870638, -370.1328551351891, 4116.364272747229);
        assert_close!(location.to_body_fixed(), expected);
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
