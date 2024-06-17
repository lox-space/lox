use glam::DVec3;
use std::sync::Arc;

use crate::events::{find_events, find_windows, Event, Window};
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
use lox_utils::roots::Brent;
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

#[derive(Clone, Debug)]
pub struct Trajectory<T: TimeLike, O: Origin, R: ReferenceFrame> {
    states: Vec<State<T, O, R>>,
    origin: O,
    frame: R,
    start_time: T,
    end_time: T,
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
        let start_time = states[0].time();
        let end_time = states[states.len() - 1].time();
        let t: Vec<f64> = states
            .iter()
            .map(|s| (s.time() - start_time.clone()).to_decimal_seconds())
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
            start_time,
            end_time,
            t,
            x,
            y,
            z,
            vx,
            vy,
            vz,
        })
    }

    pub fn times(&self) -> Vec<f64> {
        self.t.as_ref().to_vec()
    }

    pub fn states(&self) -> Vec<State<T, O, R>> {
        self.states.clone()
    }

    pub fn position(&self, t: f64) -> DVec3 {
        let x = self.x.interpolate(t);
        let y = self.y.interpolate(t);
        let z = self.z.interpolate(t);
        DVec3::new(x, y, z)
    }

    pub fn velocity(&self, t: f64) -> DVec3 {
        let vx = self.vx.interpolate(t);
        let vy = self.vy.interpolate(t);
        let vz = self.vz.interpolate(t);
        DVec3::new(vx, vy, vz)
    }

    pub fn interpolate(&self, dt: TimeDelta) -> State<T, O, R> {
        let t = dt.to_decimal_seconds();
        dbg!(t);
        State::new(
            self.start_time.clone() + dt,
            self.origin.clone(),
            self.frame.clone(),
            self.position(t),
            self.velocity(t),
        )
    }

    pub fn interpolate_at(&self, time: T) -> State<T, O, R> {
        self.interpolate(time - self.start_time.clone())
    }

    pub fn find_events<F: Fn(T, DVec3, DVec3) -> f64>(&self, func: F) -> Vec<Event<T>> {
        let root_finder = Brent::default();
        find_events(
            |t| {
                func(
                    self.start_time.clone() + TimeDelta::from_decimal_seconds(t).unwrap(),
                    self.position(t),
                    self.velocity(t),
                )
            },
            self.start_time.clone(),
            self.t.as_ref(),
            root_finder,
        )
        .unwrap_or_default()
    }

    pub fn find_windows<F: Fn(T, DVec3, DVec3) -> f64>(&self, func: F) -> Vec<Window<T>> {
        let root_finder = Brent::default();
        find_windows(
            |t| {
                func(
                    self.start_time.clone() + TimeDelta::from_decimal_seconds(t).unwrap(),
                    self.position(t),
                    self.velocity(t),
                )
            },
            self.start_time.clone(),
            self.end_time.clone(),
            self.t.as_ref(),
            root_finder,
        )
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
    #[error("state transformation failed: {0}")]
    StateTransformationError(String),
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
            let state = state.try_to_frame(frame.clone(), provider).map_err(|e| {
                TrajectoryTransformationError::StateTransformationError(e.to_string())
            })?;
            states.push(state)
        }
        Ok(Trajectory::new(&states)?)
    }
}
