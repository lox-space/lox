/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */
use crate::rotations::Rotation;
use glam::{DMat3, DVec3};
use lox_bodies::{TryRotationalElements, UndefinedOriginPropertyError};
use lox_time::julian_dates::JulianDate;
use lox_time::time_scales::Tdb;
use lox_time::transformations::{OffsetProvider, TryToScale};
use lox_time::TimeLike;
use std::f64::consts::{FRAC_PI_2, TAU};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum IcrfToBodyFixedError {
    #[error(transparent)]
    UndefinedRotationalElements(#[from] UndefinedOriginPropertyError),
    #[error("time error: {0}")]
    TimeError(String),
}

pub(crate) fn icrf_to_bodyfixed<
    P: OffsetProvider,
    T: TimeLike + TryToScale<Tdb, P>,
    O: TryRotationalElements,
>(
    time: T,
    body: &O,
    provider: &P,
) -> Result<Rotation, IcrfToBodyFixedError> {
    let seconds = time
        .try_to_scale(Tdb, provider)
        .map_err(|err| IcrfToBodyFixedError::TimeError(err.to_string()))?
        .seconds_since_j2000();
    let (right_ascension, declination, rotation_angle) = body.try_rotational_elements(seconds)?;
    let (right_ascension_rate, declination_rate, rotation_rate) =
        body.try_rotational_element_rates(seconds)?;

    let m1 = DMat3::from_rotation_z(-(right_ascension + FRAC_PI_2));
    let m2 = DMat3::from_rotation_x(-(FRAC_PI_2 - declination));
    let m3 = DMat3::from_rotation_z(-(rotation_angle % TAU));
    let m = m3 * m2 * m1;
    let v = DVec3::new(right_ascension_rate, -declination_rate, rotation_rate);
    Ok(Rotation::new(m).with_angular_velocity(v))
}
//
// impl<O: RotationalElements, P: FrameTransformationProvider> TryRotateTo<BodyFixed<O>, P> for Icrf {
//     type Error = ();
//
//     fn try_rotation<T: TimeLike>(
//         &self,
//         frame: &BodyFixed<O>,
//         time: T,
//         _provider: &P,
//     ) -> Result<Rotation, Self::Error> {
//         // FIXME
//         let seconds = time.seconds_since_j2000();
//         Ok(icrf_to_bodyfixed(&frame.0, seconds).unwrap())
//     }
// }
