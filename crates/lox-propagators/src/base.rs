/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_coords::base::{BaseState, BaseTwoBody};
use lox_coords::trajectories::base::BaseTrajectory;
use lox_time::base_time::BaseTime;
use lox_time::deltas::TimeDelta;

pub trait BasePropagator {
    type Error;
    type Output: BaseTrajectory;

    // Takes a single `TimeDelta` and returns a single new state
    fn state_from_delta(
        &self,
        initial_state: (BaseTime, impl BaseTwoBody),
        delta: TimeDelta,
    ) -> Result<BaseState, Self::Error>;
    // Takes a single `BaseTime` and returns a single new state
    fn state_from_time(
        &self,
        initial_state: (BaseTime, impl BaseTwoBody),
        time: BaseTime,
    ) -> Result<BaseState, Self::Error>;
    // Takes a slice of `TimeDelta` and returns a `BaseTrajectory` implementation
    fn trajectory_from_deltas(
        &self,
        initial_state: (BaseTime, impl BaseTwoBody),
        deltas: &[TimeDelta],
    ) -> Result<Self::Output, Self::Error>;
    // Takes a slice `BaseTime`` and returns a `BaseTrajectory` implementation
    fn trajectory_from_times(
        &self,
        initial_state: (BaseTime, impl BaseTwoBody),
        times: &[BaseTime],
    ) -> Result<Self::Output, Self::Error>;
}
