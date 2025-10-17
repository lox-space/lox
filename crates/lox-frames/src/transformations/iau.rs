/*
 * Copyright (c) 2025. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::transformations::rotations::Rotation;
use crate::transformations::{TransformProvider, TryTransform};
use glam::{DMat3, DVec3};
use lox_bodies::TryRotationalElements;
use lox_time::Time;
use lox_time::offsets::TryOffset;
use lox_time::time_scales::Tdb;
use lox_time::{julian_dates::JulianDate, time_scales::TimeScale};
use std::f64::consts::{FRAC_PI_2, TAU};

use crate::frames::{Iau, Icrf};

pub fn icrf_to_iau(
    right_ascension: f64,
    declination: f64,
    rotation_angle: f64,
    right_ascension_rate: f64,
    declination_rate: f64,
    rotation_rate: f64,
) -> Rotation {
    let m1 = DMat3::from_rotation_z(-(right_ascension + FRAC_PI_2));
    let m2 = DMat3::from_rotation_x(-(FRAC_PI_2 - declination));
    let m3 = DMat3::from_rotation_z(-(rotation_angle % TAU));
    let m = m3 * m2 * m1;
    let v = DVec3::new(right_ascension_rate, -declination_rate, rotation_rate);
    Rotation::new(m).with_angular_velocity(v)
}

impl<T, Scale, Origin> TryTransform<Icrf, Iau<Origin>, Scale> for T
where
    Scale: TimeScale + Copy,
    Origin: TryRotationalElements + Copy,
    T: TransformProvider + TryOffset<Scale, Tdb>,
{
    type Error = <Self as TryOffset<Scale, Tdb>>::Error;

    fn try_transform(
        &self,
        _origin: Icrf,
        target: Iau<Origin>,
        time: Time<Scale>,
    ) -> Result<Rotation, Self::Error> {
        let seconds = time.try_to_scale(Tdb, self)?.seconds_since_j2000();

        let (right_ascension, declination, rotation_angle) = target.rotational_elements(seconds);
        let (right_ascension_rate, declination_rate, rotation_rate) =
            target.rotational_element_rates(seconds);
        Ok(icrf_to_iau(
            right_ascension,
            declination,
            rotation_angle,
            right_ascension_rate,
            declination_rate,
            rotation_rate,
        ))
    }
}

impl<T, Scale, Origin> TryTransform<Iau<Origin>, Icrf, Scale> for T
where
    Scale: TimeScale,
    Origin: TryRotationalElements,
    T: TryTransform<Icrf, Iau<Origin>, Scale> + TryOffset<Scale, Tdb>,
{
    type Error = <Self as TryTransform<Icrf, Iau<Origin>, Scale>>::Error;

    fn try_transform(
        &self,
        origin: Iau<Origin>,
        target: Icrf,
        time: Time<Scale>,
    ) -> Result<Rotation, Self::Error> {
        Ok(self.try_transform(target, origin, time)?.transpose())
    }
}
