/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_core::coords::base::{BaseCartesian, BaseTwoBody};
use lox_core::coords::trajectories::base::BaseTrajectory;

pub trait BasePropagator {
    type Error;
    // Takes a single `TimeDelta` and returns a single new state
    fn state_from_delta(
        &self,
        initial_state: impl BaseTwoBody,
        delta: f64,
    ) -> Result<BaseCartesian, Self::Error>;
    // Takes a single `Time` and returns a single new state
    // fn state_from_time(&self, initial_state: Cartesian<T, S>, time: Time) -> Cartesian<T, S>;
    // Takes a `Vec<TimeDelta>` and returns a `Trajectory`
    fn trajectory_from_deltas(
        &self,
        initial_state: impl BaseTwoBody,
        deltas: &[f64],
    ) -> Result<BaseTrajectory, Self::Error> {
        // FIXME: Get the proper `grav_param`
        let mut state = initial_state.to_cartesian(0.0);
        let mut states: Vec<BaseCartesian> = vec![];
        for delta in deltas {
            state = self.state_from_delta(state, *delta)?;
            states.push(state)
        }
        Ok(BaseTrajectory { states })
    }
    // Takes a `Vec<Time>` and returns a `Trajectory`
    // fn trajectory_from_times(
    //     &self,
    //     initial_state: Cartesian<T, S>,
    //     times: &[Time],
    // ) -> Trajectory<T, S> {
    //     if times.is_empty() {
    //         return Trajectory { states: vec![] };
    //     }
    //     let t0 = times.first().unwrap();
    //     let deltas: Vec<TimeDelta> = times.iter().map(|t| t - t0).collect();
    //     self.trajectory_from_deltas(initial_state, &deltas)
    // }
}
