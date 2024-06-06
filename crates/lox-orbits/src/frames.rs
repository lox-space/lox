/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::convert::Infallible;

use glam::{DMat3, DVec3};
use lox_bodies::RotationalElements;
use lox_time::transformations::OffsetProvider;
use lox_utils::types::units::Seconds;

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

pub trait TryToFrame<T, O, R: ReferenceFrame, P: FrameTransformationProvider> {
    type Output: CoordinateSystem<R>;

    fn try_to_frame(&self, frame: R, provider: &P) -> Result<Self::Output, P::Error>;
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
        let (right_ascension, declination, prime_meridian) = T::rotational_elements(seconds);
        let (right_ascension_rate, declination_rate, prime_meridian_rate) =
            T::rotational_element_rates(seconds);
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
