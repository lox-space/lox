/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */
use crate::rotations::Rotation;
use glam::{DMat3, DVec2, DVec3};
use lox_bodies::{Earth, RotationalElements};
use lox_earth::cio::s06::s;
use lox_earth::cip::xy06::xy;
use lox_earth::coordinate_transformations::{
    celestial_to_intermediate_frame_of_date_matrix, polar_motion_matrix,
};
use lox_earth::rotation_angle::RotationAngle;
use lox_earth::tio::sp_00;
use lox_math::constants::f64::time::SECONDS_PER_DAY;

pub fn icrf_to_cirf(centuries: f64) -> Rotation {
    // TODO: Add IERS corrections
    let cip_coords = xy(centuries);
    let cio_locator = s(centuries, cip_coords);
    let m = celestial_to_intermediate_frame_of_date_matrix(cip_coords, cio_locator);
    Rotation::new(m)
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
