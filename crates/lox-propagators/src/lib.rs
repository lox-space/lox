use lox_bodies::PointMass;
use lox_coords::frames::ReferenceFrame;
use lox_coords::trajectories::Trajectory;
use lox_coords::two_body::Cartesian;
use lox_time::deltas::TimeDelta;
use lox_time::time_scales::TimeScale;
use lox_time::Time;

pub mod base;
pub mod semi_analytical;
mod stumpff;

pub trait Propagator<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: ReferenceFrame + Copy,
{
    type Error;

    // Takes a single `TimeDelta` and returns a single new state
    fn state_from_delta(
        &self,
        initial_state: Cartesian<T, O, F>,
        delta: TimeDelta,
    ) -> Result<Cartesian<T, O, F>, Self::Error>;
    // Takes a single `BaseTime` and returns a single new state
    fn state_from_time(
        &self,
        initial_state: Cartesian<T, O, F>,
        time: Time<T>,
    ) -> Result<Cartesian<T, O, F>, Self::Error>;
    // Takes a slice of `TimeDelta` and returns a `BaseTrajectory` implementation
    fn trajectory_from_deltas(
        &self,
        initial_state: Cartesian<T, O, F>,
        deltas: &[TimeDelta],
    ) -> Result<impl Trajectory<T, O, F>, Self::Error>;
    // Takes a slice `BaseTime`` and returns a `BaseTrajectory` implementation
    fn trajectory_from_times(
        &self,
        initial_state: Cartesian<T, O, F>,
        times: &[Time<T>],
    ) -> Result<impl Trajectory<T, O, F>, Self::Error>;
}
