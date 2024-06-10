use std::sync::Arc;

use crate::{
    frames::{CoordinateSystem, ReferenceFrame},
    origins::{CoordinateOrigin, Origin},
    states::State,
};
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

pub struct Trajectory<T, O: Origin, R: ReferenceFrame> {
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
    R: ReferenceFrame,
    O: Origin + Clone,
{
    fn origin(&self) -> O {
        self.origin.clone()
    }
}

impl<T, O, R> CoordinateSystem<R> for Trajectory<T, O, R>
where
    O: Origin,
    R: ReferenceFrame + Clone,
{
    fn reference_frame(&self) -> R {
        self.frame.clone()
    }
}
