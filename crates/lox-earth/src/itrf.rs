/*
 * Copyright (c) 2025. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */
use crate::cio::s06::s;
use crate::cip::xy06::cip_coords;
use crate::coordinate_transformations::{
    celestial_to_intermediate_frame_of_date_matrix, polar_motion_matrix,
};
use crate::eop::{EopProvider, EopProviderError};
use crate::rotation_angle::RotationAngle;
use crate::tio::sp_00;
use glam::{DMat3, DVec2, DVec3};
use lox_bodies::{Earth, RotationalElements};
use lox_frames::dynamic::DynTransformError;
use lox_frames::transformations::TryTransform;
use lox_frames::transformations::rotations::Rotation;
use lox_frames::{Cirf, DynFrame, Icrf, transform_provider};
use lox_time::Time;
use lox_time::julian_dates::JulianDate;
use lox_time::offsets::TryOffset;
use lox_time::time_scales::{DynTimeScale, Tdb, TimeScale};
use lox_units::constants::f64::time::SECONDS_PER_DAY;
use thiserror::Error;

transform_provider!(EopProvider);

pub fn icrf_to_cirf(centuries: f64) -> Rotation {
    // TODO: Add IERS corrections
    let xy = cip_coords(centuries);
    let cio_locator = s(centuries, xy);
    let m = celestial_to_intermediate_frame_of_date_matrix(xy, cio_locator);
    Rotation::new(m)
}

impl<T> TryTransform<Icrf, Cirf, T> for EopProvider
where
    T: TimeScale + Copy,
    Self: TryOffset<T, Tdb, Error = EopProviderError>,
{
    type Error = EopProviderError;

    fn try_transform(
        &self,
        _origin: Icrf,
        _target: Cirf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        let _centuries_tdb = time.try_to_scale(Tdb, self)?.centuries_since_j2000();
        todo!()
    }
}

pub fn cirf_to_tirf(seconds: f64) -> Rotation {
    let era = Earth::rotation_angle_00(seconds / SECONDS_PER_DAY);
    let rate = Earth.rotation_rate(seconds);
    let m = DMat3::from_rotation_z(-era);
    let v = DVec3::new(0.0, 0.0, rate);
    Rotation::new(m).with_angular_velocity(v)
}

pub fn tirf_to_itrf(centuries: f64) -> Rotation {
    // TODO: Add IERS corrections
    let pole_coords: DVec2 = (0.0, 0.0).into();
    let tio_locator = sp_00(centuries);
    let m = polar_motion_matrix(pole_coords, tio_locator);
    Rotation::new(m)
}

#[derive(Debug, Error)]
pub enum DynTransformEopError {
    #[error(transparent)]
    DynTransform(#[from] DynTransformError),
    #[error(transparent)]
    Eop(#[from] EopProviderError),
}

impl TryTransform<DynFrame, DynFrame, DynTimeScale> for EopProvider {
    type Error = DynTransformEopError;

    fn try_transform(
        &self,
        _origin: DynFrame,
        _target: DynFrame,
        _time: Time<DynTimeScale>,
    ) -> Result<Rotation, Self::Error> {
        todo!()
    }
}
