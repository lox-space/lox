// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_time::{Time, offsets::OffsetProvider, time_scales::TimeScale};

pub use rotations::Rotation;

use crate::ReferenceFrame;

pub mod iau;
pub mod rotations;

pub trait RotationProvider: OffsetProvider {}

pub trait TryRotation<Origin, Target, T>: RotationProvider
where
    Origin: ReferenceFrame,
    Target: ReferenceFrame,
    T: TimeScale,
{
    type Error: std::error::Error + Send + Sync + 'static;

    fn try_rotation(
        &self,
        origin: Origin,
        target: Target,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error>;
}
