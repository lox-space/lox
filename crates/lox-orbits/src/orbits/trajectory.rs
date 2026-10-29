// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::num::ParseFloatError;

use lox_bodies::{CoordinateOrigin, Origin};
use lox_core::coords::{
    Cartesian, CartesianTrajectory, CartesianTrajectoryError, TimeStampedCartesian,
};
use lox_core::glam::DVec3;
use lox_ephem::Ephemeris;
use lox_frames::{Frame, Icrf, ReferenceFrame, rotations::TryRotation, traits::frame_key};
use lox_time::{Time, deltas::TimeDelta, time_scales::TimeScale, utc::Utc};
use thiserror::Error;

use lox_time::intervals::TimeInterval;

use crate::propagators::Propagator;

use super::{CartesianOrbit, Orbit};

/// A time-ordered sequence of Cartesian orbital states with Hermite cubic interpolation.
///
/// Position is interpolated using Hermite cubic splines that incorporate both
/// position and velocity at each knot point. Velocity is derived as the
/// analytical derivative of the position spline.
///
/// Queries outside of the trajectory are extrapolated up to the
/// [extrapolation limit](Self::extrapolation_limit) before the start and after
/// the end time, which defaults to
/// [`DEFAULT_EXTRAPOLATION_LIMIT`](lox_core::coords::DEFAULT_EXTRAPOLATION_LIMIT).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trajectory<O: CoordinateOrigin = Origin, R: ReferenceFrame = Frame> {
    epoch: Time,
    origin: O,
    frame: R,
    data: CartesianTrajectory,
}

impl<O, R> Trajectory<O, R>
where
    O: CoordinateOrigin + Copy,
    R: ReferenceFrame + Copy,
{
    /// Constructs a trajectory from its constituent parts.
    pub fn from_parts(epoch: Time, origin: O, frame: R, data: CartesianTrajectory) -> Self {
        Self {
            epoch,
            origin,
            frame,
            data,
        }
    }

    /// Decomposes this trajectory into its constituent parts.
    pub fn into_parts(self) -> (Time, O, R, CartesianTrajectory) {
        (self.epoch, self.origin, self.frame, self.data)
    }

    /// Constructs a trajectory from an iterator of Cartesian orbital states.
    ///
    /// The origin and reference frame are taken from the first state.
    ///
    /// # Errors
    ///
    /// Returns [`TrajectoryError::InsufficientStates`] if fewer than 2 states are
    /// given, and [`TrajectoryError::InvalidStates`] if the epochs are not strictly
    /// increasing or a state contains a non-finite value.
    pub fn try_new(
        states: impl IntoIterator<Item = CartesianOrbit<O, R>>,
    ) -> Result<Self, TrajectoryError> {
        let mut states = states.into_iter().peekable();
        let first = states
            .peek()
            .ok_or(TrajectoryError::InsufficientStates(0))?;
        let epoch = first.time();
        let origin = first.origin();
        let frame = first.reference_frame();
        let data = CartesianTrajectory::try_from_states(states.map(|orb| TimeStampedCartesian {
            time: orb.time() - epoch,
            state: orb.state(),
        }))?;
        Ok(Self::from_parts(epoch, origin, frame, data))
    }

    /// Interpolates the trajectory at the given absolute time.
    ///
    /// Returns `None` if `time` lies beyond the
    /// [extrapolation limit](Self::extrapolation_limit).
    #[inline]
    pub fn at(&self, time: Time) -> Option<CartesianOrbit<O, R>> {
        self.at_delta(time - self.epoch)
    }

    /// Interpolates the trajectory at the given time delta from the trajectory start.
    ///
    /// Returns `None` if `dt` lies beyond the
    /// [extrapolation limit](Self::extrapolation_limit).
    #[inline]
    pub fn at_delta(&self, dt: TimeDelta) -> Option<CartesianOrbit<O, R>> {
        let t = dt.as_seconds_f64();
        let state = self.data.at(t)?;
        Some(Orbit::from_state(
            state,
            self.epoch + dt,
            self.origin,
            self.frame,
        ))
    }

    /// Returns how far the trajectory is extrapolated before the start and after
    /// the end time.
    pub fn extrapolation_limit(&self) -> TimeDelta {
        self.data.extrapolation_limit()
    }

    /// Sets how far the trajectory is extrapolated before the start and after
    /// the end time. A negative limit is treated as zero.
    ///
    /// Extrapolation continues the cubic of the first or last interval, so its
    /// accuracy degrades quickly beyond a fraction of the step size.
    pub fn with_extrapolation_limit(mut self, limit: TimeDelta) -> Self {
        self.data = self.data.with_extrapolation_limit(limit);
        self
    }

    /// Transforms the entire trajectory into a different reference frame.
    pub fn into_frame<R1, P>(self, frame: R1, provider: &P) -> Result<Trajectory<O, R1>, P::Error>
    where
        R1: ReferenceFrame + Copy,
        P: TryRotation<R, R1, TimeScale>,
    {
        if frame_key(&self.frame) == frame_key(&frame) {
            return Ok(Trajectory::from_parts(
                self.epoch,
                self.origin,
                frame,
                self.data,
            ));
        }

        let states: Result<Vec<TimeStampedCartesian>, P::Error> = self
            .data
            .iter()
            .map(|TimeStampedCartesian { time, state }| {
                let t = self.epoch + time;
                provider
                    .try_rotation(self.frame, frame, t)
                    .map(|rot| TimeStampedCartesian {
                        time,
                        state: rot * state,
                    })
            })
            .collect();

        Ok(Trajectory::from_parts(
            self.epoch,
            self.origin,
            frame,
            CartesianTrajectory::from_states(states?)
                .with_extrapolation_limit(self.extrapolation_limit()),
        ))
    }

    /// Returns the central body origin.
    pub fn origin(&self) -> O {
        self.origin
    }

    /// Returns the reference frame.
    pub fn reference_frame(&self) -> R {
        self.frame
    }

    /// Returns the start time of the trajectory.
    pub fn start_time(&self) -> Time {
        self.epoch
    }

    /// Returns the end time of the trajectory.
    pub fn end_time(&self) -> Time {
        let last = self.data.time_steps().last().copied().unwrap_or(0.0);
        self.epoch + TimeDelta::from_seconds_f64(last)
    }

    /// Returns the absolute times of all data points in the trajectory.
    pub fn times(&self) -> Vec<Time> {
        self.data
            .time_steps()
            .iter()
            .map(|&t| self.epoch + TimeDelta::from_seconds_f64(t))
            .collect()
    }

    /// Returns all orbital states at the original data points.
    pub fn states(&self) -> Vec<CartesianOrbit<O, R>> {
        self.data
            .iter()
            .map(|TimeStampedCartesian { time, state }| {
                Orbit::from_state(state, self.epoch + time, self.origin, self.frame)
            })
            .collect()
    }

    /// Returns the trajectory as a vector of `[t, x, y, z, vx, vy, vz]` rows.
    pub fn to_vec(&self) -> Vec<Vec<f64>> {
        self.data
            .iter()
            .map(|TimeStampedCartesian { time, state }| {
                let (p, v) = (state.position(), state.velocity());
                vec![time.as_seconds_f64(), p.x, p.y, p.z, v.x, v.y, v.z]
            })
            .collect()
    }
}

impl<O, R> Trajectory<O, R>
where
    O: CoordinateOrigin + Copy + Into<Origin>,
    R: ReferenceFrame + Copy + Into<Frame>,
{
    /// Converts this trajectory into a dynamically-typed trajectory.
    pub fn into_dynamic(self) -> Trajectory {
        Trajectory::from_parts(
            self.epoch.into_dynamic(),
            self.origin.into(),
            self.frame.into(),
            self.data,
        )
    }
}

impl<O, R> Propagator<O> for Trajectory<O, R>
where
    O: CoordinateOrigin + Copy,
    R: ReferenceFrame + Copy,
{
    type Frame = R;
    type Error = TrajectoryError;

    #[inline]
    fn state_at(&self, time: Time) -> Result<CartesianOrbit<O, R>, TrajectoryError> {
        match self.at(time) {
            Some(state) => Ok(state),
            None => Err(TrajectoryError::OutOfRange(time)),
        }
    }

    fn propagate(&self, interval: TimeInterval) -> Result<Trajectory<O, R>, Self::Error> {
        let mut states = Vec::new();
        states.push(self.state_at(interval.start())?);
        for s in self.states() {
            if s.time() > interval.start() && s.time() < interval.end() {
                states.push(s);
            }
        }
        states.push(self.state_at(interval.end())?);
        Ok(Trajectory::try_new(states)?.with_extrapolation_limit(self.extrapolation_limit()))
    }

    fn propagate_to(
        &self,
        times: impl IntoIterator<Item = Time>,
    ) -> Result<Trajectory<O, R>, Self::Error> {
        let states: Result<Vec<_>, _> = times.into_iter().map(|t| self.state_at(t)).collect();
        Ok(Trajectory::try_new(states?)?.with_extrapolation_limit(self.extrapolation_limit()))
    }
}

/// Errors that can occur when constructing or parsing a trajectory.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum TrajectoryError {
    /// Too few states were provided.
    #[error("`states` must have at least 2 elements but had {0}")]
    InsufficientStates(usize),
    /// The states are not strictly increasing in time or contain non-finite values.
    #[error("invalid states: {0}")]
    InvalidStates(CartesianTrajectoryError),
    /// An error occurred while parsing CSV data.
    #[error("CSV error: {0}")]
    CsvError(String),
    /// The requested time lies beyond the trajectory's extrapolation limit.
    #[error("{0} lies outside of the trajectory")]
    OutOfRange(Time),
}

impl From<CartesianTrajectoryError> for TrajectoryError {
    fn from(err: CartesianTrajectoryError) -> Self {
        match err {
            CartesianTrajectoryError::InsufficientStates(n) => {
                TrajectoryError::InsufficientStates(n)
            }
            err => TrajectoryError::InvalidStates(err),
        }
    }
}

impl From<csv::Error> for TrajectoryError {
    fn from(err: csv::Error) -> Self {
        TrajectoryError::CsvError(err.to_string())
    }
}

/// Errors that can occur when transforming a trajectory to a different origin.
#[derive(Debug, Error)]
pub enum TrajectoryTransformationError {
    /// The underlying trajectory construction failed.
    #[error(transparent)]
    TrajectoryError(#[from] TrajectoryError),
    /// A state transformation (e.g. origin change) failed.
    #[error("state transformation failed: {0}")]
    StateTransformationError(String),
}

impl<O> Trajectory<O, Icrf>
where
    O: CoordinateOrigin + Copy,
{
    /// Transforms the entire trajectory to a different central body origin using an ephemeris.
    pub fn to_origin<O1: CoordinateOrigin + Copy, E: Ephemeris>(
        &self,
        target: O1,
        ephemeris: &E,
    ) -> Result<Trajectory<O1, Icrf>, TrajectoryTransformationError> {
        if self.origin().id() == target.id() {
            return Ok(Trajectory::from_parts(
                self.epoch,
                target,
                Icrf,
                self.data.clone(),
            ));
        }
        let states: Result<Vec<_>, _> = self
            .states()
            .into_iter()
            .map(|state| {
                state.to_origin(target, ephemeris).map_err(|e| {
                    TrajectoryTransformationError::StateTransformationError(e.to_string())
                })
            })
            .collect();
        Ok(Trajectory::try_new(states?)?.with_extrapolation_limit(self.extrapolation_limit()))
    }
}

impl<O, R> Trajectory<O, R>
where
    O: CoordinateOrigin + Copy,
    R: ReferenceFrame + Copy,
{
    /// Parses a trajectory from CSV data with columns `[time, x, y, z, vx, vy, vz]` in km and km/s.
    pub fn from_csv(csv: &str, origin: O, frame: R) -> Result<Self, TrajectoryError> {
        Trajectory::try_new(parse_csv_states(csv, origin, frame)?)
    }
}

impl Trajectory {
    /// Parses a dynamically-typed trajectory from CSV data.
    pub fn from_csv_dynamic(
        csv: &str,
        origin: Origin,
        frame: Frame,
    ) -> Result<Trajectory, TrajectoryError> {
        Ok(Trajectory::from_csv(csv, origin, frame)?.into_dynamic())
    }
}

fn parse_csv_f64(record: &csv::StringRecord, idx: usize) -> Result<f64, TrajectoryError> {
    record
        .get(idx)
        .unwrap()
        .parse()
        .map_err(|e: ParseFloatError| TrajectoryError::CsvError(format!("invalid value: {e}")))
}

fn parse_csv_vec3(
    record: &csv::StringRecord,
    i0: usize,
    i1: usize,
    i2: usize,
) -> Result<DVec3, TrajectoryError> {
    Ok(DVec3::new(
        parse_csv_f64(record, i0)?,
        parse_csv_f64(record, i1)?,
        parse_csv_f64(record, i2)?,
    ))
}

fn parse_csv_states<O: CoordinateOrigin + Copy, R: ReferenceFrame + Copy>(
    csv: &str,
    origin: O,
    frame: R,
) -> Result<Vec<CartesianOrbit<O, R>>, TrajectoryError> {
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    let mut states = Vec::new();
    for result in reader.records() {
        let record = result?;
        if record.len() != 7 {
            return Err(TrajectoryError::CsvError(
                "invalid record length".to_string(),
            ));
        }
        let time = Utc::from_iso(record.get(0).unwrap())
            .map_err(|e| TrajectoryError::CsvError(e.to_string()))?
            .to_dynamic_time();
        // CSV data is in km and km/s, convert to m and m/s
        let position = parse_csv_vec3(&record, 1, 2, 3)? * 1e3;
        let velocity = parse_csv_vec3(&record, 4, 5, 6)? * 1e3;
        let state = Cartesian::from_vecs(position, velocity);
        states.push(CartesianOrbit::new(state, time, origin, frame));
    }
    Ok(states)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_bodies::{Earth, Origin};
    use lox_core::glam::DVec3;
    use lox_frames::providers::DefaultRotationProvider;
    use lox_frames::{Frame, Icrf};
    use lox_time::time_scales::TimeScale;
    use lox_time::{time, time_scales::Tdb};

    fn sample_trajectory() -> Trajectory<Earth, Icrf> {
        let t0 = time!(Tdb, 2023, 1, 1, 12).unwrap();
        let t1 = t0 + lox_time::deltas::TimeDelta::from_seconds(60);
        let t2 = t0 + lox_time::deltas::TimeDelta::from_seconds(120);
        let s0 = CartesianOrbit::new(
            Cartesian::from_vecs(DVec3::new(7000e3, 0.0, 0.0), DVec3::new(0.0, 7500.0, 0.0)),
            t0.into_dynamic(),
            Earth,
            Icrf,
        );
        let s1 = CartesianOrbit::new(
            Cartesian::from_vecs(
                DVec3::new(6999e3, 100e3, 0.0),
                DVec3::new(-10.0, 7499.0, 0.0),
            ),
            t1.into_dynamic(),
            Earth,
            Icrf,
        );
        let s2 = CartesianOrbit::new(
            Cartesian::from_vecs(
                DVec3::new(6996e3, 200e3, 0.0),
                DVec3::new(-20.0, 7498.0, 0.0),
            ),
            t2.into_dynamic(),
            Earth,
            Icrf,
        );
        Trajectory::try_new(vec![s0, s1, s2]).unwrap()
    }

    #[test]
    fn test_trajectory_from_csv_rejects_non_finite_state() {
        let csv = "time,x,y,z,vx,vy,vz\n\
                   2023-01-01T12:00:00,7000,0,0,0,7.5,0\n\
                   2023-01-01T12:01:00,NaN,0,0,0,7.5,0\n";
        let err = Trajectory::from_csv(csv, Earth, Icrf).unwrap_err();
        assert_eq!(
            err,
            TrajectoryError::InvalidStates(CartesianTrajectoryError::NonFinite(1))
        );
    }

    #[test]
    fn test_trajectory_into_dynamic() {
        let traj = sample_trajectory();
        let first_pos = traj.states().first().unwrap().position();
        let dynamic_traj = traj.into_dynamic();

        assert_eq!(dynamic_traj.origin(), Origin::Earth);
        assert_eq!(dynamic_traj.reference_frame(), Frame::Icrf);
        assert_eq!(
            dynamic_traj.states().first().unwrap().time().scale(),
            TimeScale::Tdb
        );
        assert_eq!(dynamic_traj.states().first().unwrap().position(), first_pos);
    }

    #[test]
    fn test_trajectory_at_extrapolation_limit() {
        let traj = sample_trajectory();
        let (start, end) = (traj.start_time(), traj.end_time());
        let inside = TimeDelta::from_microseconds(999);
        let outside = TimeDelta::from_microseconds(1001);

        assert!(traj.at(start).is_some());
        assert!(traj.at(end).is_some());
        assert!(traj.at(start - inside).is_some());
        assert!(traj.at(end + inside).is_some());
        assert!(traj.at(start - outside).is_none());
        assert!(traj.at(end + outside).is_none());
        assert!(traj.at_delta(TimeDelta::from_seconds(-1)).is_none());

        let err = traj.state_at(end + TimeDelta::from_seconds(1)).unwrap_err();
        assert_eq!(
            err,
            TrajectoryError::OutOfRange(end + TimeDelta::from_seconds(1))
        );

        let traj = traj.with_extrapolation_limit(TimeDelta::from_seconds(10));
        assert_eq!(traj.extrapolation_limit(), TimeDelta::from_seconds(10));
        assert!(traj.at(start - TimeDelta::from_seconds(10)).is_some());
        assert!(traj.at(end + TimeDelta::from_seconds(10)).is_some());
        assert!(traj.at(end + TimeDelta::from_seconds(11)).is_none());
        assert_eq!(traj.end_time(), end);
    }

    #[test]
    fn test_trajectory_transformations_keep_extrapolation_limit() {
        let limit = TimeDelta::from_seconds(10);
        let traj = sample_trajectory().with_extrapolation_limit(limit);

        let interval = TimeInterval::new(
            traj.start_time() + TimeDelta::from_seconds(30),
            traj.end_time(),
        );
        assert_eq!(
            traj.propagate(interval).unwrap().extrapolation_limit(),
            limit
        );
        assert_eq!(
            traj.clone()
                .into_frame(Icrf, &DefaultRotationProvider)
                .unwrap()
                .extrapolation_limit(),
            limit
        );
        assert_eq!(
            traj.clone()
                .into_dynamic()
                .into_frame(Frame::J2000, &DefaultRotationProvider)
                .unwrap()
                .extrapolation_limit(),
            limit
        );
        assert_eq!(traj.into_dynamic().extrapolation_limit(), limit);
    }

    #[test]
    fn test_trajectory_propagate_to_keeps_extrapolation_limit() {
        let traj = sample_trajectory().with_extrapolation_limit(TimeDelta::ZERO);
        let (start, end) = (traj.start_time(), traj.end_time());

        let resampled = traj
            .propagate_to([start, start + TimeDelta::from_seconds(60), end])
            .unwrap();
        assert_eq!(resampled.extrapolation_limit(), TimeDelta::ZERO);
        assert!(resampled.at(end).is_some());
        assert!(
            resampled
                .at(end + TimeDelta::from_microseconds(500))
                .is_none()
        );
    }

    #[test]
    fn test_trajectory_into_parts() {
        let traj = sample_trajectory();
        let epoch_before = traj.start_time();
        let (epoch, origin, frame, _data) = traj.into_parts();

        assert_eq!(origin, Earth);
        assert_eq!(frame, Icrf);
        assert_eq!(epoch, epoch_before);
    }
}
