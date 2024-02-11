/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::bodies::PointMass;
use crate::coords::two_body::Cartesian;
use crate::frames::ReferenceFrame;
use crate::time::continuous::TimeScale;

pub mod base;

pub struct Trajectory<T, O, R>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    R: ReferenceFrame + Copy,
{
    pub states: Vec<Cartesian<T, O, R>>,
}
