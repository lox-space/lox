use lox_time::deltas::TimeDelta;
use lox_time::TimeLike;

use crate::{frames::ReferenceFrame, origins::Origin, states::State, trajectories::Trajectory};

pub mod semi_analytical;
mod stumpff;

pub trait Propagator<T, O, R>
where
    T: TimeLike + Clone,
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
        let dt = time - initial_state.time();
        self.state_from_delta(initial_state, dt)
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
        let t0 = initial_state.time();
        self.trajectory_from_deltas(
            initial_state,
            &times
                .iter()
                .map(|t| t.clone() - t0.clone())
                .collect::<Vec<_>>(),
        )
    }
}
