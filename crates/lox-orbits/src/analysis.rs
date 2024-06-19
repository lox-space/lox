/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::events::{find_windows, Window};
use lox_bodies::{RotationalElements, Spheroid};
use lox_time::deltas::TimeDelta;
use lox_time::julian_dates::JulianDate;
use lox_time::time_scales::Tdb;
use lox_time::transformations::TryToScale;
use lox_time::TimeLike;
use lox_utils::roots::Brent;
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
    frame: &Topocentric<O>,
    gs: &Trajectory<T, O, Icrf>,
    sc: &Trajectory<T, O, Icrf>,
    provider: &P,
) -> Radians {
    let body_fixed = BodyFixed(gs.origin());
    let gs = gs.interpolate_at(time.clone()).position();
    let sc = sc.interpolate_at(time.clone()).position();
    let r = sc - gs;
    let Ok(tdb) = time.try_to_scale(Tdb, provider) else {
        // FIXME
        return f64::NAN;
    };
    let seconds = tdb.seconds_since_j2000();
    let rot = body_fixed.rotation(seconds);
    let r_body = rot.rotate_position(r);
    let rot = frame.rotation_from_body_fixed();
    let r_sez = rot * r_body;
    (r_sez.z / r.length()).asin()
}

pub fn visibility<
    T: TimeLike + TryToScale<Tdb, P> + Clone,
    O: Origin + Spheroid + RotationalElements + Clone,
    P: FrameTransformationProvider,
>(
    times: &[T],
    frame: &Topocentric<O>,
    min_elevation: Radians,
    gs: &Trajectory<T, O, Icrf>,
    sc: &Trajectory<T, O, Icrf>,
    provider: &P,
) -> Vec<Window<T>> {
    if times.len() < 2 {
        return vec![];
    }
    let start = times.first().unwrap().clone();
    let end = times.last().unwrap().clone();
    let times: Vec<f64> = times
        .iter()
        .map(|t| (t.clone() - start.clone()).to_decimal_seconds())
        .collect();
    let root_finder = Brent::default();
    find_windows(
        |t| {
            elevation(
                start.clone() + TimeDelta::from_decimal_seconds(t).unwrap(),
                &frame,
                &gs,
                &sc,
                provider,
            ) - min_elevation
        },
        start.clone(),
        end.clone(),
        &times,
        root_finder,
    )
}
