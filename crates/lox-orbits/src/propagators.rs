// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::Origin;
use lox_frames::ReferenceFrame;
use lox_time::Time;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::TimeScale;

use crate::orbits::{CartesianOrbit, TrajectorError, Trajectory};

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

    /// Evaluate the state at a single time.
    fn state_at(&self, time: Time<T>) -> Result<CartesianOrbit<T, O, Self::Frame>, Self::Error>;

    /// Propagate over the given interval in the native frame.
    /// The propagator chooses the time steps.
    fn propagate(
        &self,
        interval: TimeInterval<T>,
    ) -> Result<Trajectory<T, O, Self::Frame>, Self::Error>;

    /// Propagate to an iterable of caller-chosen times.
    fn propagate_to(
        &self,
        times: impl IntoIterator<Item = Time<T>>,
    ) -> Result<Trajectory<T, O, Self::Frame>, Self::Error>
    where
        Self::Error: From<TrajectorError>,
    {
        let states: Result<Vec<_>, _> = times.into_iter().map(|t| self.state_at(t)).collect();
        Ok(Trajectory::try_new(states?)?)
    }
}
