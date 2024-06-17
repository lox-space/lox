use lox_time::deltas::TimeDelta;
use lox_time::TimeLike;

use crate::{frames::ReferenceFrame, origins::Origin, states::State, trajectories::Trajectory};

pub mod semi_analytical;
mod stumpff;

pub trait Propagator<T, O, R>
where
    T: TimeLike + Clone,
    O: Origin + Clone,
    R: ReferenceFrame + Clone,
{
    type Error;

    fn propagate(
        &self,
        initial_state: &State<T, O, R>,
        delta: TimeDelta,
    ) -> Result<State<T, O, R>, Self::Error>;

    fn propagate_all(
        &self,
        initial_state: &State<T, O, R>,
        deltas: impl IntoIterator<Item = TimeDelta>,
    ) -> Result<Trajectory<T, O, R>, Self::Error>;
}
