use lox_time::TimeLike;

use crate::trajectories::TrajectoryError;
use crate::{frames::ReferenceFrame, origins::Origin, states::State, trajectories::Trajectory};

pub mod semi_analytical;
pub mod sgp4;
mod stumpff;

pub trait Propagator<T, O, R>
where
    T: TimeLike + Clone,
    O: Origin + Clone,
    R: ReferenceFrame + Clone,
{
    type Error: From<TrajectoryError>;

    fn propagate(&self, time: T) -> Result<State<T, O, R>, Self::Error>;

    fn propagate_all(
        &self,
        times: impl IntoIterator<Item = T>,
    ) -> Result<Trajectory<T, O, R>, Self::Error> {
        let mut states = vec![];
        for time in times {
            let state = self.propagate(time)?;
            states.push(state);
        }
        Ok(Trajectory::new(&states)?)
    }
}
