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

pub struct Trajectory<T, S>
where
    T: PointMass + Copy,
    S: ReferenceFrame + Copy,
{
    pub states: Vec<Cartesian<T, S>>,
}
