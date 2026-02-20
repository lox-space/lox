// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::Origin;
use lox_time::Time;
use lox_time::time_scales::TimeScale;

use crate::orbits::{CartesianOrbit, TrajectorError, Trajectory};
use lox_frames::ReferenceFrame;

pub mod numerical;
pub mod semi_analytical;
pub mod sgp4;
mod stumpff;

pub trait Propagator<T, O, R>
where
    T: TimeScale + Copy,
    O: Origin + Copy,
    R: ReferenceFrame + Copy,
{
    type Error: From<TrajectorError>;

    fn propagate(&self, time: Time<T>) -> Result<CartesianOrbit<T, O, R>, Self::Error>;

    fn propagate_all(
        &self,
        times: impl IntoIterator<Item = Time<T>>,
    ) -> Result<Trajectory<T, O, R>, Self::Error> {
        let states: Vec<_> = times
            .into_iter()
            .map(|time| self.propagate(time))
            .collect::<Result<_, _>>()?;
        Ok(Trajectory::try_new(states)?)
    }
}
