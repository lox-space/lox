use std::convert::Infallible;
use std::sync::Arc;

use crate::frames::{BodyFixed, FrameTransformationProvider, Icrf, TryToFrame};
use crate::{
    frames::{CoordinateSystem, ReferenceFrame},
    origins::{CoordinateOrigin, Origin},
    states::State,
};
use lox_bodies::{Body, RotationalElements};
use lox_time::time_scales::Tdb;
use lox_time::transformations::TryToScale;
use lox_time::{deltas::TimeDelta, TimeLike};
use lox_utils::series::{Series, SeriesError};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
struct ArcVecF64(Arc<Vec<f64>>);

impl AsRef<[f64]> for ArcVecF64 {
    fn as_ref(&self) -> &[f64] {
        self.0.as_ref()
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TrajectoryError {
    #[error("`states` must have at least 2 elements but had {0}")]
    InsufficientStates(usize),
    #[error(transparent)]
    SeriesError(#[from] SeriesError),
}

pub struct Trajectory<T: TimeLike, O: Origin, R: ReferenceFrame> {
    states: Vec<State<T, O, R>>,
    origin: O,
    frame: R,
    t0: T,
    t1: T,
    dt_max: TimeDelta,
    t: ArcVecF64,
    x: Series<ArcVecF64, Vec<f64>>,
    y: Series<ArcVecF64, Vec<f64>>,
    z: Series<ArcVecF64, Vec<f64>>,
    vy: Series<ArcVecF64, Vec<f64>>,
    vx: Series<ArcVecF64, Vec<f64>>,
    vz: Series<ArcVecF64, Vec<f64>>,
}

impl<T, O, R> Trajectory<T, O, R>
where
    T: TimeLike + Clone,
    O: Origin + Clone,
    R: ReferenceFrame + Clone,
{
    pub fn new(states: &[State<T, O, R>]) -> Result<Self, TrajectoryError> {
        if states.len() < 2 {
            return Err(TrajectoryError::InsufficientStates(states.len()));
        }
        let origin = states[0].origin();
        let frame = states[0].reference_frame();
        let t0 = states[0].time();
        let t1 = states[states.len() - 1].time();
        let dt_max = t1.clone() - t0.clone();
        let t: Vec<f64> = states
            .iter()
            .map(|s| (s.time() - t0.clone()).to_decimal_seconds())
            .collect();
        let t = ArcVecF64(Arc::new(t));
        let x: Vec<f64> = states.iter().map(|s| s.position().x).collect();
        let y: Vec<f64> = states.iter().map(|s| s.position().y).collect();
        let z: Vec<f64> = states.iter().map(|s| s.position().z).collect();
        let vx: Vec<f64> = states.iter().map(|s| s.velocity().x).collect();
        let vy: Vec<f64> = states.iter().map(|s| s.velocity().y).collect();
        let vz: Vec<f64> = states.iter().map(|s| s.velocity().z).collect();
        let x = Series::with_cubic_spline(t.clone(), x)?;
        let y = Series::with_cubic_spline(t.clone(), y)?;
        let z = Series::with_cubic_spline(t.clone(), z)?;
        let vx = Series::with_cubic_spline(t.clone(), vx)?;
        let vy = Series::with_cubic_spline(t.clone(), vy)?;
        let vz = Series::with_cubic_spline(t.clone(), vz)?;
        Ok(Self {
            states: states.to_vec(),
            origin,
            frame,
            t0,
            t1,
            dt_max,
            t,
            x,
            y,
            z,
            vx,
            vy,
            vz,
        })
    }
}

impl<T, O, R> CoordinateOrigin<O> for Trajectory<T, O, R>
where
    T: TimeLike,
    O: Origin + Clone,
    R: ReferenceFrame,
{
    fn origin(&self) -> O {
        self.origin.clone()
    }
}

impl<T, O, R> CoordinateSystem<R> for Trajectory<T, O, R>
where
    T: TimeLike,
    O: Origin,
    R: ReferenceFrame + Clone,
{
    fn reference_frame(&self) -> R {
        self.frame.clone()
    }
}

#[derive(Error, Debug)]
pub enum TrajectoryTransformationError {
    #[error(transparent)]
    TrajectoryError(#[from] TrajectoryError),
    #[error(transparent)]
    Infallible(#[from] Infallible),
}

impl<T, O, R, P> TryToFrame<BodyFixed<R>, P> for Trajectory<T, O, Icrf>
where
    T: TryToScale<Tdb, P> + TimeLike + Clone,
    O: Body + Clone,
    R: RotationalElements + Clone,
    P: FrameTransformationProvider,
{
    type Output = Trajectory<T, O, BodyFixed<R>>;
    type Error = TrajectoryTransformationError;

    fn try_to_frame(&self, frame: BodyFixed<R>, provider: &P) -> Result<Self::Output, Self::Error> {
        let mut states: Vec<State<T, O, BodyFixed<R>>> = Vec::with_capacity(self.states.len());
        for state in &self.states {
            let state = state.try_to_frame(frame.clone(), provider).unwrap();
            states.push(state)
        }
        Ok(Trajectory::new(&states)?)
    }
}
