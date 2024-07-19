/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::convert::Infallible;
use std::f64::consts::FRAC_PI_2;

use crate::ground::GroundLocation;
use glam::{DMat3, DVec3};
use lox_bodies::{RotationalElements, Spheroid};
use lox_math::types::units::Seconds;
use lox_time::transformations::OffsetProvider;

use crate::rotations::Rotation;

pub trait ReferenceFrame {
    fn name(&self) -> String;
    fn abbreviation(&self) -> String;
}

pub trait CoordinateSystem<T: ReferenceFrame> {
    fn reference_frame(&self) -> T;
}

pub trait FrameTransformationProvider: OffsetProvider {}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct NoOpFrameTransformationProvider;

impl OffsetProvider for NoOpFrameTransformationProvider {
    type Error = Infallible;
}
impl FrameTransformationProvider for NoOpFrameTransformationProvider {}

pub trait TryToFrame<R: ReferenceFrame, P: FrameTransformationProvider> {
    type Output: CoordinateSystem<R>;
    type Error;

    fn try_to_frame(&self, frame: R, provider: &P) -> Result<Self::Output, Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct Icrf;

impl ReferenceFrame for Icrf {
    fn name(&self) -> String {
        "International Celestial Reference Frame".to_string()
    }

    fn abbreviation(&self) -> String {
        "ICRF".to_string()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct BodyFixed<T: RotationalElements>(pub T);

impl<T: RotationalElements> BodyFixed<T> {
    pub fn rotation(&self, seconds: Seconds) -> Rotation {
        let (right_ascension, declination, prime_meridian) = self.0.rotational_elements(seconds);
        let (right_ascension_rate, declination_rate, prime_meridian_rate) =
            self.0.rotational_element_rates(seconds);
        let m1 = DMat3::from_rotation_z(-right_ascension);
        let m2 = DMat3::from_rotation_x(-declination);
        let m3 = DMat3::from_rotation_z(-prime_meridian);
        let m = m3 * m2 * m1;
        let v = DVec3::new(right_ascension_rate, declination_rate, prime_meridian_rate);
        Rotation::new(m).with_angular_velocity(v)
    }
}

impl<T: RotationalElements> ReferenceFrame for BodyFixed<T> {
    fn name(&self) -> String {
        let body = self.0.name();
        match body {
            "Sun" | "Moon" => format!("IAU Body-Fixed Reference Frame for the {}", body),
            _ => format!("IAU Body-Fixed Reference Frame for {}", body),
        }
    }

    fn abbreviation(&self) -> String {
        let body = self.0.name().replace([' ', '-'], "_").to_uppercase();
        format!("IAU_{}", body)
    }
}

#[derive(Clone, Debug)]
pub struct Topocentric<B: Spheroid>(GroundLocation<B>);

impl<B: Spheroid> Topocentric<B> {
    pub fn new(location: GroundLocation<B>) -> Self {
        Topocentric(location)
    }

    pub fn from_coords(longitude: f64, latitude: f64, altitude: f64, body: B) -> Self {
        let location = GroundLocation::new(longitude, latitude, altitude, body);
        Topocentric(location)
    }

    pub fn rotation_from_body_fixed(&self) -> DMat3 {
        let r1 = DMat3::from_rotation_z(self.0.longitude()).transpose();
        let r2 = DMat3::from_rotation_y(FRAC_PI_2 - self.0.latitude()).transpose();
        r2 * r1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_bodies::Earth;
    use lox_math::assert_close;
    use lox_math::is_close::IsClose;

    #[test]
    fn test_topocentric() {
        let topo = Topocentric::from_coords(8.0, 50.0, 0.0, Earth);
        let r = topo.rotation_from_body_fixed();
        let x_axis = DVec3::new(
            0.038175550084451906,
            -0.9893582466233818,
            -0.14040258976976597,
        );
        let y_axis = DVec3::new(
            -0.25958272521858694,
            -0.14550003380861354,
            0.9546970980000851,
        );
        let z_axis = DVec3::new(-0.9649660284921128, 0.0, -0.2623748537039304);
        assert_close!(r.x_axis, x_axis);
        assert_close!(r.y_axis, y_axis);
        assert_close!(r.z_axis, z_axis);
    }
}
