// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::Origin;
use lox_frames::ReferenceFrame;
use lox_frames::rotations::TryRotation;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::TimeScale;

use crate::orbits::Trajectory;

pub mod numerical;
pub mod semi_analytical;
pub mod sgp4;
mod stumpff;

pub trait Propagator<T, O>
where
    T: TimeScale + Copy,
    O: Origin + Copy,
{
    /// The propagator's native reference frame.
    type Frame: ReferenceFrame + Copy;
    type Error: std::error::Error + 'static;

    /// Propagate over the given interval in the native frame.
    fn propagate(
        &self,
        interval: TimeInterval<T>,
    ) -> Result<Trajectory<T, O, Self::Frame>, Self::Error>;

    /// Propagate and transform to a target frame.
    fn propagate_in_frame<R, P>(
        &self,
        interval: TimeInterval<T>,
        frame: R,
        provider: &P,
    ) -> Result<Trajectory<T, O, R>, Box<dyn std::error::Error>>
    where
        R: ReferenceFrame + Copy,
        P: TryRotation<Self::Frame, R, T>,
    {
        let traj = self.propagate(interval)?;
        traj.into_frame(frame, provider)
    }
}
