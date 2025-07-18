/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::rotations::Rotation;
use glam::{DMat3, DVec3};
use lox_bodies::RotationalElements;
use lox_bodies::{TryRotationalElements, UndefinedOriginPropertyError};
use lox_time::Time;
use lox_time::time_scales::Tdb;
use lox_time::time_scales::TryToScale;
use lox_time::{julian_dates::JulianDate, time_scales::TimeScale};
use std::f64::consts::{FRAC_PI_2, TAU};
use thiserror::Error;

use super::Iau;
use super::Icrf;
use super::TryRotateTo;

#[derive(Debug, Error)]
pub enum IauFrameTransformationError {
    #[error(transparent)]
    UndefinedRotationalElements(#[from] UndefinedOriginPropertyError),
    #[error("TDB transformation error: {0}")]
    Tdb(String),
}

pub(crate) fn icrf_to_iau<T, O, P>(
    time: Time<T>,
    body: O,
    provider: Option<&P>,
) -> Result<Rotation, IauFrameTransformationError>
where
    T: TimeScale + TryToScale<Tdb, P>,
    O: TryRotationalElements,
{
    let seconds = time
        .try_to_scale(Tdb, provider)
        .map_err(|err| IauFrameTransformationError::Tdb(err.to_string()))?
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

impl<T, O, P> TryRotateTo<T, Iau<O>, P> for Icrf
where
    T: TimeScale + TryToScale<Tdb, P>,
    O: RotationalElements,
{
    type Error = IauFrameTransformationError;

    fn try_rotation(
        &self,
        frame: Iau<O>,
        time: Time<T>,
        provider: Option<&P>,
    ) -> Result<Rotation, Self::Error> {
        icrf_to_iau(time, frame.0, provider)
    }
}

impl<T, O, P> TryRotateTo<T, Icrf, P> for Iau<O>
where
    T: TimeScale + TryToScale<Tdb, P>,
    O: RotationalElements + Clone,
{
    type Error = IauFrameTransformationError;

    fn try_rotation(
        &self,
        _frame: Icrf,
        time: Time<T>,
        provider: Option<&P>,
    ) -> Result<Rotation, Self::Error> {
        Ok(icrf_to_iau(time, self.0.clone(), provider)?.transpose())
    }
}
