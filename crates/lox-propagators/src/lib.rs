use lox_core::bodies::PointMass;
use lox_core::coords::trajectories::Trajectory;
use lox_core::coords::two_body::Cartesian;
use lox_core::frames::ReferenceFrame;

pub mod semi_analytical;
mod stumpff;

pub trait Propagator<T: PointMass + Copy, S: ReferenceFrame + Copy> {
    type Error;
    // Takes a single `TimeDelta` and returns a single new state
    fn state_from_delta(
        &self,
        initial_state: Cartesian<T, S>,
        delta: f64,
    ) -> Result<Cartesian<T, S>, Self::Error>;
    // Takes a single `Time` and returns a single new state
    // fn state_from_time(&self, initial_state: Cartesian<T, S>, time: Time) -> Cartesian<T, S>;
    // Takes a `Vec<TimeDelta>` and returns a `Trajectory`
    fn trajectory_from_deltas(
        &self,
        initial_state: Cartesian<T, S>,
        deltas: &[f64],
    ) -> Result<Trajectory<T, S>, Self::Error> {
        let mut state = initial_state;
        let mut states: Vec<Cartesian<T, S>> = vec![];
        for delta in deltas {
            state = self.state_from_delta(state, *delta)?;
            states.push(state)
        }
        Ok(Trajectory { states })
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
