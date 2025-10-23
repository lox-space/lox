// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::cio::s06::cio_locator;
use crate::cip::xy06::CipCoords;
use crate::coordinate_transformations::{
    PoleCoords, celestial_to_intermediate_frame_of_date_matrix, polar_motion_matrix,
};
use crate::eop::{EopProvider, EopProviderError};
use crate::rotation_angle::earth_rotation_angle_00;
use crate::tio::tio_locator;
use glam::{DMat3, DVec3};
use lox_bodies::{Earth, RotationalElements};
use lox_frames::dynamic::DynTransformError;
use lox_frames::transformations::TryTransform;
use lox_frames::transformations::rotations::Rotation;
use lox_frames::{Cirf, DynFrame, Icrf, transform_provider};
use lox_time::Time;
use lox_time::julian_dates::JulianDate;
use lox_time::offsets::TryOffset;
use lox_time::time_scales::{DynTimeScale, Tdb, TimeScale};
use lox_units::f64::consts::SECONDS_PER_DAY;
use thiserror::Error;

transform_provider!(EopProvider);

pub fn icrf_to_cirf(centuries: f64) -> Rotation {
    // TODO: Add IERS corrections
    let xy = CipCoords::new(centuries);
    let s = cio_locator(centuries, xy);
    let m = celestial_to_intermediate_frame_of_date_matrix(xy, s);
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
    let era = earth_rotation_angle_00(seconds / SECONDS_PER_DAY);
    let rate = Earth.rotation_rate(seconds);
    let m = DMat3::from_rotation_z(-era.to_radians());
    let v = DVec3::new(0.0, 0.0, rate);
    Rotation::new(m).with_angular_velocity(v)
}

pub fn tirf_to_itrf(centuries: f64) -> Rotation {
    // TODO: Add IERS corrections
    let pole_coords = PoleCoords::default();
    let sp = tio_locator(centuries);
    let m = polar_motion_matrix(pole_coords, sp);
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
