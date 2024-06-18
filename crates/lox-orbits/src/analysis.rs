/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_bodies::{RotationalElements, Spheroid};
use lox_time::julian_dates::JulianDate;
use lox_time::time_scales::Tdb;
use lox_time::transformations::TryToScale;
use lox_time::TimeLike;
use lox_utils::types::units::Radians;

use crate::frames::{BodyFixed, FrameTransformationProvider, Icrf, Topocentric};
use crate::origins::{CoordinateOrigin, Origin};
use crate::trajectories::Trajectory;

pub fn elevation<
    T: TimeLike + TryToScale<Tdb, P> + Clone,
    O: Origin + Spheroid + RotationalElements + Clone,
    P: FrameTransformationProvider,
>(
    time: T,
    frame: Topocentric<O>,
    gs: Trajectory<T, O, Icrf>,
    sc: Trajectory<T, O, Icrf>,
    provider: &P,
) -> Result<Radians, P::Error> {
    let body_fixed = BodyFixed(gs.origin());
    let gs = gs.interpolate_at(time.clone()).position();
    let sc = sc.interpolate_at(time.clone()).position();
    let r = sc - gs;
    let seconds = time.try_to_scale(Tdb, provider)?.seconds_since_j2000();
    let rot = body_fixed.rotation(seconds);
    let r_body = rot.rotate_position(r);
    let rot = frame.rotation_from_body_fixed();
    let r_sez = rot * r_body;
    Ok((r_sez.z / r.length()).asin())
}
