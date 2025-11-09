// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::Origin;
use lox_core::coords::TimeStampedCartesian;
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::time_scales::TimeScale;

use crate::orbits::Orbit;
use crate::trajectories::TrajectoryError;
use crate::{states::State, trajectories::Trajectory};
use lox_frames::ReferenceFrame;

pub mod numerical;
pub mod semi_analytical;
pub mod sgp4;
mod stumpff;

pub trait Propagator<T, O, R>
where
    T: TimeScale + Clone,
    O: Origin + Clone,
    R: ReferenceFrame + Clone,
{
    type Error: From<TrajectoryError>;

    fn propagate(&self, time: Time<T>) -> Result<State<T, O, R>, Self::Error>;

    fn propagate_all(
        &self,
        times: impl IntoIterator<Item = Time<T>>,
    ) -> Result<Trajectory<T, O, R>, Self::Error> {
        let mut states = vec![];
        for time in times {
            let state = self.propagate(time)?;
            states.push(state);
        }
        Ok(Trajectory::new(&states)?)
    }
}

pub trait StatePropagator: Send + Sync + 'static {
    type State;
    type Error: std::error::Error + Send + Sync + 'static;

    fn propagate(
        &self,
        initial_state: Self::State,
        step: TimeDelta,
    ) -> Result<Self::State, Self::Error>;
}

pub trait DynPropagator: Send + Sync + 'static {
    fn propagate(
        &self,
        initial_state: TimeStampedCartesian,
        step: TimeDelta,
    ) -> Result<TimeStampedCartesian, Box<dyn std::error::Error>>;
}

impl<T: StatePropagator<State = TimeStampedCartesian>> DynPropagator for T {
    fn propagate(
        &self,
        initial_state: TimeStampedCartesian,
        step: TimeDelta,
    ) -> Result<TimeStampedCartesian, Box<dyn std::error::Error>> {
        Ok(<Self as StatePropagator>::propagate(
            self,
            initial_state,
            step,
        )?)
    }
}
