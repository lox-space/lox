use lox_bodies::Origin;
use lox_time::Time;
use lox_time::time_scales::TimeScale;

use crate::trajectories::TrajectoryError;
use crate::{frames::ReferenceFrame, states::State, trajectories::Trajectory};

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
