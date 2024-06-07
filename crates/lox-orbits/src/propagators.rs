use lox_time::deltas::TimeDelta;
use lox_time::Datetime;

use crate::{frames::ReferenceFrame, origins::Origin, states::State, trajectories::Trajectory};

pub mod semi_analytical;
mod stumpff;

pub trait Propagator<T, O, R>
where
    T: Datetime,
    O: Origin,
    R: ReferenceFrame,
{
    type Error;

    // Takes a single `TimeDelta` and returns a single new state
    fn state_from_delta(
        &self,
        initial_state: State<T, O, R>,
        delta: TimeDelta,
    ) -> Result<State<T, O, R>, Self::Error>;
    // Takes a single `BaseTime` and returns a single new state
    fn state_from_time(
        &self,
        initial_state: State<T, O, R>,
        time: T,
    ) -> Result<State<T, O, R>, Self::Error> {
        self.state_from_delta(initial_state, time.to_delta())
    }
    // Takes a slice of `TimeDelta` and returns a `BaseTrajectory` implementation
    fn trajectory_from_deltas(
        &self,
        initial_state: State<T, O, R>,
        deltas: &[TimeDelta],
    ) -> Result<Trajectory<T, O, R>, Self::Error>;
    // Takes a slice `BaseTime` and returns a `BaseTrajectory` implementation
    fn trajectory_from_times(
        &self,
        initial_state: State<T, O, R>,
        times: &[T],
    ) -> Result<Trajectory<T, O, R>, Self::Error> {
        self.trajectory_from_deltas(
            initial_state,
            &times.iter().map(|t| t.to_delta()).collect::<Vec<_>>(),
        )
    }
}
