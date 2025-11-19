// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_time::{Time, time_scales::TimeScale};

pub use rotations::Rotation;

use crate::ReferenceFrame;

pub mod iau;
pub mod rotations;

pub trait TransformProvider {}

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
