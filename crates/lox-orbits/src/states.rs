/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::frames::{Bodyfixed, Icrf, ReferenceFrame, TryToFrame};
use glam::DVec3;
use lox_bodies::{PointMass, RotationalElements};
use lox_time::transformations::ToTdb;
use std::convert::Infallible;

pub struct State<T, O: PointMass, R: ReferenceFrame> {
    time: T,
    origin: O,
    frame: R,
    position: DVec3,
    velocity: DVec3,
}

impl<T, O, R> State<T, O, R>
where
    O: PointMass,
    R: ReferenceFrame,
{
    pub fn new(time: T, center: O, frame: R, position: DVec3, velocity: DVec3) -> Self {
        Self {
            time,
            origin: center,
            frame,
            position,
            velocity,
        }
    }
}

impl<T: ToTdb, O: PointMass + RotationalElements> TryToFrame<Bodyfixed<O>> for State<T, O, Icrf> {
    type Error = Infallible;

    fn try_to_frame(&self, frame: Bodyfixed<O>) -> Result<Bodyfixed<O>, Self::Error> {
        todo!()
    }
}
