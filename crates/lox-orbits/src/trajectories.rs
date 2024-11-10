use std::num::ParseFloatError;
use std::sync::Arc;

use csv::Error;
use glam::DVec3;
use lox_ephem::Ephemeris;
use thiserror::Error;

use lox_bodies::{Body, RotationalElements};
use lox_math::roots::Brent;
use lox_math::series::{Series, SeriesError};
use lox_time::time_scales::{Tai, Tdb};
use lox_time::transformations::TryToScale;
use lox_time::utc::leap_seconds::BuiltinLeapSeconds;
use lox_time::utc::Utc;
use lox_time::{deltas::TimeDelta, Time, TimeLike};

use crate::events::{find_events, find_windows, Event, Window};
use crate::frames::{BodyFixed, FrameTransformationProvider, Icrf, TryToFrame};
use crate::{
    frames::{CoordinateSystem, ReferenceFrame},
    origins::{CoordinateOrigin, Origin},
    states::State,
};

#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
struct ArcVecF64(Arc<Vec<f64>>);

impl AsRef<[f64]> for ArcVecF64 {
    fn as_ref(&self) -> &[f64] {
        self.0.as_ref()
    }
}

impl From<csv::Error> for TrajectoryError {
    fn from(err: Error) -> Self {
        TrajectoryError::CsvError(err.to_string())
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TrajectoryError {
    #[error("`states` must have at least 2 elements but had {0}")]
    InsufficientStates(usize),
    #[error(transparent)]
    SeriesError(#[from] SeriesError),
    #[error("invalid time scale: {0}")]
    CsvError(String),
}

#[derive(Clone, Debug)]
pub struct Trajectory<T: TimeLike, O: Origin, R: ReferenceFrame> {
    states: Vec<State<T, O, R>>,
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
        let start_time = states[0].time();
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
            t,
            x,
            y,
            z,
            vx,
            vy,
            vz,
        })
    }

    pub fn with_frame<R1: ReferenceFrame + Clone>(self, frame: R1) -> Trajectory<T, O, R1> {
        let states: Vec<State<T, O, R1>> = self
            .states
            .into_iter()
            .map(|s| s.with_frame(frame.clone()))
            .collect();
        Trajectory::new(&states).unwrap()
    }

    pub fn with_origin<O1: Origin + Clone>(&self, origin: O1) -> Trajectory<T, O1, R> {
        let states: Vec<State<T, O1, R>> = self
            .states
            .iter()
            .map(|s| s.with_origin(origin.clone()))
            .collect();
        Trajectory::new(&states).unwrap()
    }

    pub fn with_origin_and_frame<O1: Origin + Clone, R1: ReferenceFrame + Clone>(
        &self,
        origin: O1,
        frame: R1,
    ) -> Trajectory<T, O1, R1> {
        let states: Vec<State<T, O1, R1>> = self
            .states
            .iter()
            .map(|s| s.with_origin_and_frame(origin.clone(), frame.clone()))
            .collect();
        Trajectory::new(&states).unwrap()
    }

    pub fn start_time(&self) -> T {
        self.states[0].time()
    }

    pub fn end_time(&self) -> T {
        self.states.last().unwrap().time()
    }

    pub fn times(&self) -> Vec<T> {
        self.states.iter().map(|s| s.time()).collect()
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

    pub fn to_vec(&self) -> Vec<Vec<f64>> {
        let times = self.t.clone().0;
        let mut vec = Vec::with_capacity(times.len());
        for (i, state) in self.states.iter().enumerate() {
            vec.push(vec![
                times[i],
                state.position().x,
                state.position().y,
                state.position().z,
                state.velocity().x,
                state.velocity().y,
                state.velocity().z,
            ]);
        }
        vec
    }

    pub fn interpolate(&self, dt: TimeDelta) -> State<T, O, R> {
        let t = dt.to_decimal_seconds();
        State::new(
            self.start_time() + dt,
            self.position(t),
            self.velocity(t),
            self.origin(),
            self.reference_frame(),
        )
    }

    pub fn interpolate_at(&self, time: T) -> State<T, O, R> {
        self.interpolate(time - self.start_time())
    }

    pub fn find_events<F: Fn(State<T, O, R>) -> f64>(&self, func: F) -> Vec<Event<T>> {
        let root_finder = Brent::default();
        find_events(
            |t| {
                func(State::new(
                    self.start_time() + TimeDelta::from_decimal_seconds(t).unwrap(),
                    self.position(t),
                    self.velocity(t),
                    self.origin(),
                    self.reference_frame(),
                ))
            },
            self.start_time(),
            self.t.as_ref(),
            root_finder,
        )
        .unwrap_or_default()
    }

    pub fn find_windows<F: Fn(State<T, O, R>) -> f64>(&self, func: F) -> Vec<Window<T>> {
        let root_finder = Brent::default();
        find_windows(
            |t| {
                func(State::new(
                    self.start_time() + TimeDelta::from_decimal_seconds(t).unwrap(),
                    self.position(t),
                    self.velocity(t),
                    self.origin(),
                    self.reference_frame(),
                ))
            },
            self.start_time(),
            self.end_time(),
            self.t.as_ref(),
            root_finder,
        )
    }
}

impl<T, O> Trajectory<T, O, Icrf>
where
    T: TimeLike + Clone,
    O: Origin + Body + Clone,
{
    pub fn to_origin<O1: Origin + Body + Clone, E: Ephemeris>(
        &self,
        target: O1,
        ephemeris: &E,
    ) -> Result<Trajectory<T, O1, Icrf>, TrajectoryTransformationError> {
        let mut states: Vec<State<T, O1, Icrf>> = Vec::with_capacity(self.states.len());
        for state in &self.states {
            let state = state.to_origin(target.clone(), ephemeris).map_err(|e| {
                TrajectoryTransformationError::StateTransformationError(e.to_string())
            })?;
            states.push(state);
        }
        Ok(Trajectory::new(&states)?)
    }
}

impl<O, R> Trajectory<Time<Tai>, O, R>
where
    O: Origin + Clone,
    R: ReferenceFrame + Clone,
{
    pub fn from_csv(
        csv: &str,
        origin: O,
        frame: R,
    ) -> Result<Trajectory<Time<Tai>, O, R>, TrajectoryError> {
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let mut states = Vec::new();
        for result in reader.records() {
            let record = result?;
            if record.len() != 7 {
                return Err(TrajectoryError::CsvError(
                    "invalid record length".to_string(),
                ));
            }
            let time: Time<Tai> = Utc::from_iso(record.get(0).unwrap())
                .map_err(|e| TrajectoryError::CsvError(e.to_string()))?
                .try_to_scale(Tai, &BuiltinLeapSeconds)
                .map_err(|e| TrajectoryError::CsvError(e.to_string()))?;
            let x = record
                .get(1)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid x coordinate: {}", e))
                })?;
            let y = record
                .get(2)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid y coordinate: {}", e))
                })?;
            let z = record
                .get(3)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid z coordinate: {}", e))
                })?;
            let position = DVec3::new(x, y, z);
            let vx = record
                .get(4)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid x velocity: {}", e))
                })?;
            let vy = record
                .get(5)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid y velocity: {}", e))
                })?;
            let vz = record
                .get(6)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid z velocity: {}", e))
                })?;
            let velocity = DVec3::new(vx, vy, vz);
            let state = State::new(time, position, velocity, origin.clone(), frame.clone());
            states.push(state);
        }
        Trajectory::new(&states)
    }
}

impl<T, O, R> CoordinateOrigin<O> for Trajectory<T, O, R>
where
    T: TimeLike,
    O: Origin + Clone,
    R: ReferenceFrame,
{
    fn origin(&self) -> O {
        self.states[0].origin()
    }
}

impl<T, O, R> CoordinateSystem<R> for Trajectory<T, O, R>
where
    T: TimeLike,
    O: Origin,
    R: ReferenceFrame + Clone,
{
    fn reference_frame(&self) -> R {
        self.states[0].reference_frame()
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
