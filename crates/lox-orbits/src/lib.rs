/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub use glam::DVec3;

use lox_bodies::PointMass;
use lox_time::TimeSystem;

use crate::frames::ReferenceFrame;

pub mod anomalies;
pub mod base;
pub mod events;
pub mod frames;
pub mod propagators;
pub mod trajectories;
pub mod two_body;

pub trait CoordinateSystem: TimeSystem {
    type Origin: PointMass;
    type Frame: ReferenceFrame;

    fn origin(&self) -> Self::Origin;
    fn reference_frame(&self) -> Self::Frame;
}
