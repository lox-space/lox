/*
 * Copyright (c) 2025. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_time::{Time, offsets::OffsetProvider, time_scales::TimeScale};

pub use rotations::Rotation;

use crate::ReferenceFrame;

pub mod iau;
pub mod rotations;

pub trait TransformProvider: OffsetProvider {}

pub trait TryTransform<Origin, Target, T>: TransformProvider
where
    Origin: ReferenceFrame,
    Target: ReferenceFrame,
    T: TimeScale,
{
    type Error: std::error::Error + Send + Sync + 'static;

    fn try_transform(
        &self,
        origin: Origin,
        target: Target,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error>;
}
