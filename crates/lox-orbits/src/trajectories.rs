use std::num::ParseFloatError;

use csv::Error;
use glam::DVec3;
use lox_ephem::Ephemeris;
use lox_time::DynTime;
use lox_time::time_scales::{DynTimeScale, TimeScale};
use thiserror::Error;

use lox_bodies::{DynOrigin, Origin};
use lox_math::roots::Brent;
use lox_math::series::{Series, SeriesError};
use lox_time::time_scales::Tai;
use lox_time::utc::Utc;
use lox_time::{Time, deltas::TimeDelta};

use crate::events::{Event, Window, find_events, find_windows};
use crate::frames::ReferenceFrame;
use crate::frames::{DynFrame, Icrf};
use crate::states::State;

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
pub struct Trajectory<T: TimeScale, O: Origin, R: ReferenceFrame> {
    states: Vec<State<T, O, R>>,
    t: Vec<f64>,
    x: Series<Vec<f64>, Vec<f64>>,
    y: Series<Vec<f64>, Vec<f64>>,
    z: Series<Vec<f64>, Vec<f64>>,
    vy: Series<Vec<f64>, Vec<f64>>,
    vx: Series<Vec<f64>, Vec<f64>>,
    vz: Series<Vec<f64>, Vec<f64>>,
}

pub type DynTrajectory = Trajectory<DynTimeScale, DynOrigin, DynFrame>;

impl<T, O, R> Trajectory<T, O, R>
where
    T: TimeScale + Clone,
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
        // let t = ArcVecF64(Arc::new(t));
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

    pub fn origin(&self) -> O {
        self.states.first().unwrap().origin()
    }

    pub fn reference_frame(&self) -> R {
        self.states.first().unwrap().reference_frame()
    }

    pub fn start_time(&self) -> Time<T> {
        self.states[0].time()
    }

    pub fn end_time(&self) -> Time<T> {
        self.states.last().unwrap().time()
    }

    pub fn times(&self) -> Vec<Time<T>> {
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
        let times = self.t.clone();
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

    pub fn interpolate_at(&self, time: Time<T>) -> State<T, O, R> {
        self.interpolate(time - self.start_time())
    }

    pub fn find_events<F: Fn(State<T, O, R>) -> f64>(&self, func: F) -> Vec<Event<T>> {
        let root_finder = Brent::default();
        find_events(
            |t| {
                func(State::new(
                    self.start_time() + TimeDelta::try_from_decimal_seconds(t).unwrap(),
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
                    self.start_time() + TimeDelta::try_from_decimal_seconds(t).unwrap(),
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
    T: TimeScale + Clone,
    O: Origin + Clone,
{
    pub fn to_origin<O1: Origin + Clone, E: Ephemeris>(
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

impl<O, R> Trajectory<Tai, O, R>
where
    O: Origin + Clone,
    R: ReferenceFrame + Clone,
{
    pub fn from_csv(
        csv: &str,
        origin: O,
        frame: R,
    ) -> Result<Trajectory<Tai, O, R>, TrajectoryError> {
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
                .to_time();
            let x = record
                .get(1)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid x coordinate: {e}"))
                })?;
            let y = record
                .get(2)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid y coordinate: {e}"))
                })?;
            let z = record
                .get(3)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid z coordinate: {e}"))
                })?;
            let position = DVec3::new(x, y, z);
            let vx = record
                .get(4)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid x velocity: {e}"))
                })?;
            let vy = record
                .get(5)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid y velocity: {e}"))
                })?;
            let vz = record
                .get(6)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid z velocity: {e}"))
                })?;
            let velocity = DVec3::new(vx, vy, vz);
            let state = State::new(time, position, velocity, origin.clone(), frame.clone());
            states.push(state);
        }
        Trajectory::new(&states)
    }
}

impl DynTrajectory {
    pub fn from_csv_dyn(
        csv: &str,
        origin: DynOrigin,
        frame: DynFrame,
    ) -> Result<DynTrajectory, TrajectoryError> {
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let mut states = Vec::new();
        for result in reader.records() {
            let record = result?;
            if record.len() != 7 {
                return Err(TrajectoryError::CsvError(
                    "invalid record length".to_string(),
                ));
            }
            let time: DynTime = Utc::from_iso(record.get(0).unwrap())
                .map_err(|e| TrajectoryError::CsvError(e.to_string()))?
                .to_dyn_time();
            let x = record
                .get(1)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid x coordinate: {e}"))
                })?;
            let y = record
                .get(2)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid y coordinate: {e}"))
                })?;
            let z = record
                .get(3)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid z coordinate: {e}"))
                })?;
            let position = DVec3::new(x, y, z);
            let vx = record
                .get(4)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid x velocity: {e}"))
                })?;
            let vy = record
                .get(5)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid y velocity: {e}"))
                })?;
            let vz = record
                .get(6)
                .unwrap()
                .parse()
                .map_err(|e: ParseFloatError| {
                    TrajectoryError::CsvError(format!("invalid z velocity: {e}"))
                })?;
            let velocity = DVec3::new(vx, vy, vz);
            let state = State::new(time, position, velocity, origin, frame);
            states.push(state);
        }
        Trajectory::new(&states)
    }
}

#[derive(Error, Debug)]
pub enum TrajectoryTransformationError {
    #[error(transparent)]
    TrajectoryError(#[from] TrajectoryError),
    #[error("state transformation failed: {0}")]
    StateTransformationError(String),
}
