// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

pub mod sso;

use std::{
    f64::consts::{PI, TAU},
    iter::zip,
    num::ParseFloatError,
    ops::Sub,
};

use glam::{DMat3, DVec3};
use itertools::Itertools;
use lox_bodies::{
    DynOrigin, Origin, PointMass, RotationalElements, Spheroid, TryPointMass, TrySpheroid,
    UndefinedOriginPropertyError,
};
use lox_core::units::{AngleUnits, Distance};
use lox_core::{
    anomalies::{EccentricAnomaly, TrueAnomaly},
    coords::{Cartesian, CartesianTrajectory, TimeStampedCartesian},
    elements::{
        ArgumentOfPeriapsis, Eccentricity, GravitationalParameter, Inclination, Keplerian,
        LongitudeOfAscendingNode,
    },
    utils::Linspace,
};
use lox_ephem::{Ephemeris, path_from_ids};
use lox_frames::{
    DynFrame, Iau, Icrf, NonQuasiInertialFrameError, QuasiInertial, ReferenceFrame, TryBodyFixed,
    TryQuasiInertial, rotations::TryRotation, traits::frame_id,
};
use lox_math::roots::{BoxedError, Brent, FindRoot, RootFinderError, Secant};
use lox_time::{
    DynTime, Time,
    deltas::TimeDelta,
    julian_dates::JulianDate,
    time_scales::{DynTimeScale, Tai, TimeScale},
    utc::Utc,
};
use thiserror::Error;

use crate::events::{Event, FindEventError, Window, find_events, find_windows};
use crate::ground::{DynGroundLocation, GroundLocation};

pub enum OrbitType {
    Cartesian(Cartesian),
    Keplerian(Keplerian),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orbit<S, T: TimeScale, O: Origin, R: ReferenceFrame> {
    state: S,
    time: Time<T>,
    origin: O,
    frame: R,
}

pub type DynOrbit = Orbit<OrbitType, DynTimeScale, DynOrigin, DynFrame>;

impl<S, T, O, R> Orbit<S, T, O, R>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    pub const fn from_state(state: S, time: Time<T>, origin: O, frame: R) -> Self {
        Self {
            state,
            time,
            origin,
            frame,
        }
    }

    pub fn state(&self) -> S
    where
        S: Copy,
    {
        self.state
    }

    pub fn time(&self) -> Time<T>
    where
        T: Copy,
    {
        self.time
    }

    pub fn origin(&self) -> O
    where
        O: Copy,
    {
        self.origin
    }

    pub fn reference_frame(&self) -> R
    where
        R: Copy,
    {
        self.frame
    }

    pub fn try_gravitational_parameter(
        &self,
    ) -> Result<GravitationalParameter, UndefinedOriginPropertyError>
    where
        O: TryPointMass,
    {
        self.origin
            .try_gravitational_parameter()
            .map(GravitationalParameter::km3_per_s2)
    }

    pub fn gravitational_parameter(&self) -> GravitationalParameter
    where
        O: PointMass,
    {
        GravitationalParameter::km3_per_s2(self.origin.gravitational_parameter())
    }
}

pub type CartesianOrbit<T, O, R> = Orbit<Cartesian, T, O, R>;

impl<T, O, R> CartesianOrbit<T, O, R>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    pub const fn new(cartesian: Cartesian, time: Time<T>, origin: O, frame: R) -> Self {
        Self {
            state: cartesian,
            time,
            origin,
            frame,
        }
    }

    pub fn position(&self) -> DVec3 {
        self.state.position()
    }

    pub fn velocity(&self) -> DVec3 {
        self.state.velocity()
    }

    pub fn to_keplerian(&self) -> KeplerianOrbit<T, O, R>
    where
        T: Copy,
        O: Copy + PointMass,
        R: Copy,
    {
        Orbit {
            state: self.state.to_keplerian(self.gravitational_parameter()),
            time: self.time,
            origin: self.origin,
            frame: self.frame,
        }
    }

    pub fn try_to_keplerian(&self) -> Result<KeplerianOrbit<T, O, R>, UndefinedOriginPropertyError>
    where
        T: Copy,
        O: Copy + TryPointMass,
        R: Copy,
    {
        Ok(Orbit {
            state: self.state.to_keplerian(self.try_gravitational_parameter()?),
            time: self.time,
            origin: self.origin,
            frame: self.frame,
        })
    }

    pub fn try_to_frame<R1, P>(
        &self,
        frame: R1,
        provider: &P,
    ) -> Result<CartesianOrbit<T, O, R1>, P::Error>
    where
        R: Copy,
        P: TryRotation<R, R1, T>,
        R1: ReferenceFrame + Copy,
        O: Copy,
        T: Copy,
    {
        let rot = provider.try_rotation(self.frame, frame, self.time)?;
        let (r1, v1) = rot.rotate_state(self.state.position(), self.state.velocity());
        Ok(CartesianOrbit::new(
            Cartesian::from_vecs(r1, v1),
            self.time,
            self.origin,
            frame,
        ))
    }
}

// 1.3: Ground location conversion helper
type LonLatAlt = (f64, f64, f64);

fn rv_to_lla(r: DVec3, r_eq: f64, f: f64) -> Result<LonLatAlt, RootFinderError> {
    let rm = r.length();
    let r_delta = (r.x.powi(2) + r.y.powi(2)).sqrt();
    let mut lon = r.y.atan2(r.x);

    if lon.abs() >= PI {
        if lon < 0.0 {
            lon += TAU;
        } else {
            lon -= TAU;
        }
    }

    let delta = (r.z / rm).asin();

    let root_finder = Secant::default();

    let lat = root_finder.find(
        |lat: f64| {
            let e = (2.0 * f - f.powi(2)).sqrt();
            let c = r_eq / (1.0 - e.powi(2) * lat.sin().powi(2)).sqrt();
            Ok((r.z + c * e.powi(2) * lat.sin()) / r_delta - lat.tan())
        },
        delta,
    )?;

    let e = (2.0 * f - f.powi(2)).sqrt();
    let c = r_eq / (1.0 - e.powi(2) * lat.sin().powi(2)).sqrt();

    let alt = r_delta / lat.cos() - c;

    Ok((lon, lat, alt))
}

// 1.3: Ground location for body-fixed frames
impl<T, O> CartesianOrbit<T, O, Iau<O>>
where
    T: TimeScale,
    O: Origin + RotationalElements + Spheroid + Copy,
{
    pub fn to_ground_location(&self) -> Result<GroundLocation<O>, RootFinderError> {
        let r = self.position();
        let r_eq = self.origin.equatorial_radius();
        let f = self.origin.flattening();
        let (lon, lat, alt) = rv_to_lla(r, r_eq, f)?;
        Ok(GroundLocation::new(lon, lat, alt, self.origin))
    }
}

// 1.3: Dynamic ground location error
#[derive(Debug, Error)]
pub enum StateToDynGroundError {
    #[error("equatorial radius and flattening factor are not available for origin `{}`", .0.name())]
    UndefinedSpheroid(DynOrigin),
    #[error(transparent)]
    RootFinderError(#[from] RootFinderError),
    #[error("not a body-fixed frame {0}")]
    NonBodyFixedFrame(String),
}

// 1.4: LVLH rotation helper
fn rotation_lvlh(position: DVec3, velocity: DVec3) -> DMat3 {
    let r = position.normalize();
    let v = velocity.normalize();
    let z = -r;
    let y = -r.cross(v);
    let x = y.cross(z);
    DMat3::from_cols(x, y, z)
}

// 1.4: LVLH rotation for ICRF orbits
impl<T, O> CartesianOrbit<T, O, Icrf>
where
    T: TimeScale,
    O: Origin,
{
    pub fn rotation_lvlh(&self) -> DMat3 {
        rotation_lvlh(self.position(), self.velocity())
    }
}

// 1.2: Origin change for ICRF orbits
impl<T, O> CartesianOrbit<T, O, Icrf>
where
    T: TimeScale + Copy,
    O: Origin + Copy,
{
    pub fn to_origin<O1: Origin + Copy, E: Ephemeris>(
        &self,
        target: O1,
        ephemeris: &E,
    ) -> Result<CartesianOrbit<T, O1, Icrf>, E::Error> {
        let epoch = self.time().seconds_since_j2000();
        let mut pos = self.position();
        let mut vel = self.velocity();
        let mut pos_eph = DVec3::ZERO;
        let mut vel_eph = DVec3::ZERO;
        let origin_id = self.origin.id();
        let target_id = target.id();
        let path = path_from_ids(origin_id.0, target_id.0);
        for (origin, target) in path.into_iter().tuple_windows() {
            let (p, v) = ephemeris.state(epoch, origin, target)?;
            let p: DVec3 = p.into();
            let v: DVec3 = v.into();
            pos_eph += p;
            vel_eph += v;
        }
        pos -= pos_eph;
        vel -= vel_eph;
        Ok(CartesianOrbit::new(
            Cartesian::from_vecs(pos, vel),
            self.time,
            target,
            Icrf,
        ))
    }
}

pub type DynCartesianOrbit = Orbit<Cartesian, DynTimeScale, DynOrigin, DynFrame>;

// 1.2, 1.3, 1.4: Dynamic orbit methods
impl DynCartesianOrbit {
    pub fn to_origin_dynamic<E: Ephemeris>(
        &self,
        target: DynOrigin,
        ephemeris: &E,
    ) -> Result<DynCartesianOrbit, E::Error> {
        let epoch = self.time().seconds_since_j2000();
        let mut pos = self.position();
        let mut vel = self.velocity();
        let mut pos_eph = DVec3::ZERO;
        let mut vel_eph = DVec3::ZERO;
        let origin_id = self.origin.id();
        let target_id = target.id();
        let path = path_from_ids(origin_id.0, target_id.0);
        for (origin, target) in path.into_iter().tuple_windows() {
            let (p, v) = ephemeris.state(epoch, origin, target)?;
            let p: DVec3 = p.into();
            let v: DVec3 = v.into();
            pos_eph += p;
            vel_eph += v;
        }
        pos -= pos_eph;
        vel -= vel_eph;
        Ok(CartesianOrbit::new(
            Cartesian::from_vecs(pos, vel),
            self.time,
            target,
            DynFrame::Icrf,
        ))
    }

    pub fn to_dyn_ground_location(&self) -> Result<DynGroundLocation, StateToDynGroundError> {
        if self.frame.try_body_fixed().is_err() {
            return Err(StateToDynGroundError::NonBodyFixedFrame(
                self.frame.name().to_string(),
            ));
        }
        let r = self.position();
        let (Ok(r_eq), Ok(f)) = (
            self.origin.try_equatorial_radius(),
            self.origin.try_flattening(),
        ) else {
            return Err(StateToDynGroundError::UndefinedSpheroid(self.origin));
        };

        let (lon, lat, alt) = rv_to_lla(r, r_eq, f)?;

        Ok(DynGroundLocation::with_dynamic(lon, lat, alt, self.origin).unwrap())
    }

    pub fn try_rotation_lvlh(&self) -> Result<DMat3, &'static str> {
        if self.frame != DynFrame::Icrf {
            return Err("only valid for ICRF");
        }
        Ok(rotation_lvlh(self.position(), self.velocity()))
    }
}

// 1.5: Sub operator
impl<T, O, R> Sub for CartesianOrbit<T, O, R>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let state = Cartesian::from_vecs(
            self.position() - rhs.position(),
            self.velocity() - rhs.velocity(),
        );
        CartesianOrbit::new(state, self.time, self.origin, self.frame)
    }
}

pub type KeplerianOrbit<T, O, R> = Orbit<Keplerian, T, O, R>;

impl<T, O, R> KeplerianOrbit<T, O, R>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    pub const fn new(keplerian: Keplerian, time: Time<T>, origin: O, frame: R) -> Self
    where
        R: QuasiInertial,
    {
        Orbit {
            state: keplerian,
            time,
            origin,
            frame,
        }
    }

    pub fn try_from_keplerian(
        keplerian: Keplerian,
        time: Time<T>,
        origin: O,
        frame: R,
    ) -> Result<Self, NonQuasiInertialFrameError>
    where
        R: TryQuasiInertial,
    {
        frame.try_quasi_inertial()?;
        Ok(Orbit {
            state: keplerian,
            time,
            origin,
            frame,
        })
    }

    pub fn semi_major_axis(&self) -> Distance {
        self.state.semi_major_axis()
    }

    pub fn eccentricity(&self) -> Eccentricity {
        self.state.eccentricity()
    }

    pub fn inclination(&self) -> Inclination {
        self.state.inclination()
    }

    pub fn longitude_of_ascending_node(&self) -> LongitudeOfAscendingNode {
        self.state.longitude_of_ascending_node()
    }

    pub fn argument_of_periapsis(&self) -> ArgumentOfPeriapsis {
        self.state.argument_of_periapsis()
    }

    pub fn true_anomaly(&self) -> TrueAnomaly {
        self.state.true_anomaly()
    }

    pub fn to_cartesian(&self) -> CartesianOrbit<T, O, R>
    where
        T: Copy,
        O: Copy + PointMass,
        R: Copy,
    {
        Orbit {
            state: self.state.to_cartesian(self.gravitational_parameter()),
            time: self.time,
            origin: self.origin,
            frame: self.frame,
        }
    }

    pub fn try_to_cartesian(&self) -> Result<CartesianOrbit<T, O, R>, UndefinedOriginPropertyError>
    where
        T: Copy,
        O: Copy + TryPointMass,
        R: Copy,
    {
        Ok(Orbit {
            state: self.state.to_cartesian(self.try_gravitational_parameter()?),
            time: self.time,
            origin: self.origin,
            frame: self.frame,
        })
    }

    pub fn orbital_period(&self) -> Option<TimeDelta>
    where
        O: TryPointMass,
    {
        self.state
            .orbital_period(self.try_gravitational_parameter().ok()?)
    }

    pub fn trace(&self, n: usize) -> Option<Trajectory<T, O, R>>
    where
        T: Copy,
        O: TryPointMass + Copy,
        R: Copy,
    {
        let period = self.orbital_period()?;
        let mean_motion = TAU / period.to_seconds().to_f64();
        let mean_anomaly_at_epoch = self.true_anomaly().to_mean(self.eccentricity()).ok()?;

        let state_iter = self
            .state
            .iter_trace(self.try_gravitational_parameter().ok()?, n);

        let data: CartesianTrajectory = zip(Linspace::new(-PI, PI, n), state_iter)
            .map(|(ecc, state)| {
                let mean_anomaly = EccentricAnomaly::new(ecc.rad()).to_mean(self.eccentricity());
                let time_of_flight = (mean_anomaly - mean_anomaly_at_epoch).as_f64() / mean_motion;
                TimeStampedCartesian {
                    time: TimeDelta::from_seconds_f64(time_of_flight),
                    state,
                }
            })
            .collect();

        Some(Trajectory {
            epoch: self.time,
            origin: self.origin,
            frame: self.frame,
            data,
        })
    }
}

pub type DynKeplerianOrbit = Orbit<Keplerian, DynTimeScale, DynOrigin, DynFrame>;

#[derive(Debug, Error)]
pub enum TrajectorError {
    #[error("at least 2 states are required but only {0} were provided")]
    InsufficientStates(usize),
}

#[derive(Debug, Clone)]
pub struct Trajectory<T: TimeScale, O: Origin, R: ReferenceFrame> {
    epoch: Time<T>,
    origin: O,
    frame: R,
    data: CartesianTrajectory,
}

impl<T, O, R> Trajectory<T, O, R>
where
    T: TimeScale + Copy,
    O: Origin + Copy,
    R: ReferenceFrame + Copy,
{
    pub fn new(states: impl IntoIterator<Item = CartesianOrbit<T, O, R>>) -> Self {
        let mut states = states.into_iter().peekable();
        let first = states.peek().unwrap();
        let epoch = first.time();
        let origin = first.origin();
        let frame = first.reference_frame();
        let data = states
            .map(|orb| {
                let time = orb.time() - epoch;
                TimeStampedCartesian {
                    time,
                    state: orb.state(),
                }
            })
            .collect();
        Self {
            epoch,
            origin,
            frame,
            data,
        }
    }

    pub fn try_new(
        states: impl IntoIterator<Item = CartesianOrbit<T, O, R>>,
    ) -> Result<Self, TrajectorError> {
        let mut states = states.into_iter().peekable();
        for i in 0..2 {
            states.peek().ok_or(TrajectorError::InsufficientStates(i))?;
        }
        Ok(Self::new(states))
    }

    pub fn at(&self, time: Time<T>) -> CartesianOrbit<T, O, R> {
        let t = (time - self.epoch).to_seconds().to_f64();
        let state = self.data.at(t);
        Orbit {
            state,
            time,
            origin: self.origin,
            frame: self.frame,
        }
    }

    pub fn into_frame<R1, P>(
        self,
        frame: R1,
        provider: P,
    ) -> Result<Trajectory<T, O, R1>, Box<dyn std::error::Error>>
    where
        R1: ReferenceFrame + Copy,
        P: TryRotation<R, R1, T>,
    {
        if frame_id(&self.frame) == frame_id(&frame) {
            return Ok(Trajectory {
                epoch: self.epoch,
                origin: self.origin,
                frame,
                data: self.data,
            });
        }

        let data: Result<CartesianTrajectory, P::Error> = self
            .data
            .into_iter()
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

        Ok(Trajectory {
            epoch: self.epoch,
            origin: self.origin,
            frame,
            data: data?,
        })
    }

    pub fn epoch(&self) -> Time<T> {
        self.epoch
    }

    pub fn origin(&self) -> O {
        self.origin
    }

    pub fn reference_frame(&self) -> R {
        self.frame
    }

    pub fn start_time(&self) -> Time<T> {
        self.epoch
    }

    pub fn end_time(&self) -> Time<T> {
        let time_steps = self.data.time_steps();
        let last = time_steps.last().copied().unwrap_or(0.0);
        self.epoch + TimeDelta::from_seconds_f64(last)
    }

    pub fn times(&self) -> Vec<Time<T>> {
        let time_steps = self.data.time_steps();
        time_steps
            .iter()
            .map(|&t| self.epoch + TimeDelta::from_seconds_f64(t))
            .collect()
    }

    pub fn states(&self) -> Vec<CartesianOrbit<T, O, R>> {
        let time_steps = self.data.time_steps();
        time_steps
            .iter()
            .map(|&t| {
                let state = self.data.at(t);
                let time = self.epoch + TimeDelta::from_seconds_f64(t);
                Orbit {
                    state,
                    time,
                    origin: self.origin,
                    frame: self.frame,
                }
            })
            .collect()
    }

    pub fn to_vec(&self) -> Vec<Vec<f64>> {
        let time_steps = self.data.time_steps();
        time_steps
            .iter()
            .map(|&t| {
                let state = self.data.at(t);
                vec![
                    t,
                    state.position().x,
                    state.position().y,
                    state.position().z,
                    state.velocity().x,
                    state.velocity().y,
                    state.velocity().z,
                ]
            })
            .collect()
    }

    pub fn interpolate(&self, dt: TimeDelta) -> CartesianOrbit<T, O, R> {
        let t = dt.to_seconds().to_f64();
        let state = self.data.at(t);
        Orbit {
            state,
            time: self.epoch + dt,
            origin: self.origin,
            frame: self.frame,
        }
    }

    pub fn interpolate_at(&self, time: Time<T>) -> CartesianOrbit<T, O, R> {
        self.interpolate(time - self.epoch)
    }

    pub fn position(&self, t: f64) -> DVec3 {
        self.data.position(t)
    }

    pub fn velocity(&self, t: f64) -> DVec3 {
        self.data.velocity(t)
    }

    pub fn find_events<F, E>(&self, func: F) -> Result<Vec<Event<T>>, FindEventError>
    where
        F: Fn(CartesianOrbit<T, O, R>) -> Result<f64, E> + Copy,
        E: Into<BoxedError>,
    {
        let root_finder = Brent::default();
        let time_steps = self.data.time_steps();
        find_events(
            |t| {
                let state = self.data.at(t);
                func(Orbit {
                    state,
                    time: self.epoch + TimeDelta::from_seconds_f64(t),
                    origin: self.origin,
                    frame: self.frame,
                })
                .map_err(Into::into)
            },
            self.epoch,
            &time_steps,
            root_finder,
        )
    }

    pub fn find_windows<F, E>(&self, func: F) -> Result<Vec<Window<T>>, RootFinderError>
    where
        F: Fn(CartesianOrbit<T, O, R>) -> Result<f64, E> + Copy,
        E: Into<BoxedError>,
    {
        let root_finder = Brent::default();
        let time_steps = self.data.time_steps();
        find_windows(
            |t| {
                let state = self.data.at(t);
                func(Orbit {
                    state,
                    time: self.epoch + TimeDelta::from_seconds_f64(t),
                    origin: self.origin,
                    frame: self.frame,
                })
                .map_err(Into::into)
            },
            self.epoch,
            self.end_time(),
            &time_steps,
            root_finder,
        )
    }
}

// Trajectory error types for CSV import and origin transformation
#[derive(Clone, Debug, Error, PartialEq)]
pub enum TrajectoryError {
    #[error("`states` must have at least 2 elements but had {0}")]
    InsufficientStates(usize),
    #[error("CSV error: {0}")]
    CsvError(String),
}

impl From<csv::Error> for TrajectoryError {
    fn from(err: csv::Error) -> Self {
        TrajectoryError::CsvError(err.to_string())
    }
}

#[derive(Debug, Error)]
pub enum TrajectoryTransformationError {
    #[error(transparent)]
    TrajectoryError(#[from] TrajectorError),
    #[error("state transformation failed: {0}")]
    StateTransformationError(String),
}

// Origin transformation for ICRF trajectories
impl<T, O> Trajectory<T, O, Icrf>
where
    T: TimeScale + Copy,
    O: Origin + Copy,
{
    pub fn to_origin<O1: Origin + Copy, E: Ephemeris>(
        &self,
        target: O1,
        ephemeris: &E,
    ) -> Result<Trajectory<T, O1, Icrf>, TrajectoryTransformationError> {
        let states: Result<Vec<_>, _> = self
            .states()
            .into_iter()
            .map(|state| {
                state.to_origin(target, ephemeris).map_err(|e| {
                    TrajectoryTransformationError::StateTransformationError(e.to_string())
                })
            })
            .collect();
        Ok(Trajectory::new(states?))
    }
}

// CSV import for Tai trajectories
impl<O, R> Trajectory<Tai, O, R>
where
    O: Origin + Copy,
    R: ReferenceFrame + Copy,
{
    pub fn from_csv(csv: &str, origin: O, frame: R) -> Result<Self, TrajectoryError> {
        let states = parse_csv_states(csv, origin, frame)?;
        if states.len() < 2 {
            return Err(TrajectoryError::InsufficientStates(states.len()));
        }
        Ok(Trajectory::new(states))
    }
}

// CSV import for DynTrajectory
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
            let position = parse_csv_vec3(&record, 1, 2, 3)?;
            let velocity = parse_csv_vec3(&record, 4, 5, 6)?;
            let state = Cartesian::from_vecs(position, velocity);
            states.push(CartesianOrbit::new(state, time, origin, frame));
        }
        if states.len() < 2 {
            return Err(TrajectoryError::InsufficientStates(states.len()));
        }
        Ok(DynTrajectory::new(states))
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

fn parse_csv_states<O: Origin + Copy, R: ReferenceFrame + Copy>(
    csv: &str,
    origin: O,
    frame: R,
) -> Result<Vec<CartesianOrbit<Tai, O, R>>, TrajectoryError> {
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
        let position = parse_csv_vec3(&record, 1, 2, 3)?;
        let velocity = parse_csv_vec3(&record, 4, 5, 6)?;
        let state = Cartesian::from_vecs(position, velocity);
        states.push(CartesianOrbit::new(state, time, origin, frame));
    }
    Ok(states)
}

pub type DynTrajectory = Trajectory<DynTimeScale, DynOrigin, DynFrame>;

impl<T, O, R> FromIterator<CartesianOrbit<T, O, R>> for Trajectory<T, O, R>
where
    T: TimeScale + Copy,
    O: Origin + Copy,
    R: ReferenceFrame + Copy,
{
    fn from_iter<U: IntoIterator<Item = CartesianOrbit<T, O, R>>>(iter: U) -> Self {
        Self::new(iter)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use lox_bodies::{Earth, Jupiter, Venus};
    use lox_core::coords::Cartesian;
    use lox_ephem::spk::parser::{Spk, parse_daf_spk};
    use lox_frames::providers::DefaultRotationProvider;
    use lox_test_utils::{assert_approx_eq, data_file};
    use lox_time::{Time, time, time_scales::Tdb, utc::Utc};

    use super::*;

    #[test]
    fn test_bodyfixed() {
        let iau_jupiter = Iau::new(Jupiter);

        let r0 = DVec3::new(6068.27927, -1692.84394, -2516.61918);
        let v0 = DVec3::new(-0.660415582, 5.495938726, -5.303093233);
        let r1 = DVec3::new(3922.220687351738, 5289.381014412637, -1631.4837924820245);
        let v1 = DVec3::new(-1.852284168309543, -0.8227941105651749, -7.14175174489828);

        let tdb = time!(Tdb, 2000, 1, 1, 12).unwrap();
        let s = CartesianOrbit::new(Cartesian::from_vecs(r0, v0), tdb, Jupiter, Icrf);
        let s1 = s
            .try_to_frame(iau_jupiter, &DefaultRotationProvider)
            .unwrap();
        let s0 = s1.try_to_frame(Icrf, &DefaultRotationProvider).unwrap();

        assert_approx_eq!(s0.position(), r0, rtol <= 1e-8);
        assert_approx_eq!(s0.velocity(), v0, rtol <= 1e-8);

        assert_approx_eq!(s1.position(), r1, rtol <= 1e-8);
        assert_approx_eq!(s1.velocity(), v1, rtol <= 1e-8);
    }

    #[test]
    fn test_cartesian_to_keplerian_roundtrip() {
        let time = time!(Tdb, 2023, 3, 25, 21, 8, 0.0).expect("time should be valid");
        let pos = DVec3::new(
            -0.107622532467967e7,
            -0.676589636432773e7,
            -0.332308783350379e6,
        );
        let vel = DVec3::new(
            0.935685775154103e4,
            -0.331234775037644e4,
            -0.118801577532701e4,
        );

        let cartesian = CartesianOrbit::new(Cartesian::from_vecs(pos, vel), time, Earth, Icrf);
        let cartesian1 = cartesian.to_keplerian().to_cartesian();

        assert_eq!(cartesian1.time(), time);
        assert_eq!(cartesian1.origin(), Earth);
        assert_eq!(cartesian1.reference_frame(), Icrf);

        assert_approx_eq!(cartesian.position(), cartesian1.position(), rtol <= 1e-8);
        assert_approx_eq!(cartesian.velocity(), cartesian1.velocity(), rtol <= 1e-6);
    }

    #[test]
    fn test_to_ground_location() {
        let lat_exp = 51.484f64.to_radians();
        let lon_exp = -35.516f64.to_radians();
        let alt_exp = 237.434; // km

        let position = DVec3::new(3359.927, -2398.072, 5153.0);
        let velocity = DVec3::new(5.0657, 5.485, -0.744);
        let time = time!(Tdb, 2012, 7, 1).unwrap();
        let state = CartesianOrbit::new(
            Cartesian::from_vecs(position, velocity),
            time,
            Earth,
            Iau::new(Earth),
        );
        let ground = state.to_ground_location().unwrap();
        assert_approx_eq!(ground.latitude(), lat_exp, rtol <= 1e-4);
        assert_approx_eq!(ground.longitude(), lon_exp, rtol <= 1e-4);
        assert_approx_eq!(ground.altitude(), alt_exp, rtol <= 1e-4);
    }

    #[test]
    fn test_to_origin() {
        let r_venus = DVec3::new(
            1.001977553295792e8,
            2.200234656010247e8,
            9.391473630346918e7,
        );
        let v_venus = DVec3::new(-59.08617935009049, 22.682387107225292, 12.05029567478702);
        let r = DVec3::new(6068279.27, -1692843.94, -2516619.18) / 1e3;
        let v = DVec3::new(-660.415582, 5495.938726, -5303.093233) / 1e3;

        let r_exp = r - r_venus;
        let v_exp = v - v_venus;

        let utc = Utc::from_iso("2016-05-30T12:00:00.000").unwrap();
        let tai = utc.to_time();

        let s_earth = CartesianOrbit::new(Cartesian::from_vecs(r, v), tai, Earth, Icrf);
        let s_venus = s_earth.to_origin(Venus, ephemeris()).unwrap();

        assert_approx_eq!(s_venus.position(), r_exp);
        assert_approx_eq!(s_venus.velocity(), v_exp);
    }

    #[test]
    fn test_rotation_lvlh() {
        let time = time!(Tdb, 2023, 3, 25, 21, 8, 0.0).unwrap();
        let pos = DVec3::new(6678.0, 0.0, 0.0);
        let vel = DVec3::new(0.0, 7.73, 0.0);

        let state = CartesianOrbit::new(Cartesian::from_vecs(pos, vel), time, Earth, Icrf);
        let rot = state.rotation_lvlh();

        // For a prograde equatorial orbit at x-axis, LVLH z should point to -x (nadir),
        // y should point to -z (cross-track), x should point to +y (velocity direction)
        let z = rot.col(2);
        let x = rot.col(0);
        assert_approx_eq!(z, -DVec3::X, atol <= 1e-10);
        assert_approx_eq!(x, DVec3::Y, atol <= 1e-10);
    }

    #[test]
    fn test_sub_operator() {
        let time = time!(Tdb, 2023, 3, 25, 21, 8, 0.0).unwrap();
        let s1 = CartesianOrbit::new(
            Cartesian::from_vecs(DVec3::new(10.0, 20.0, 30.0), DVec3::new(1.0, 2.0, 3.0)),
            time,
            Earth,
            Icrf,
        );
        let s2 = CartesianOrbit::new(
            Cartesian::from_vecs(DVec3::new(3.0, 5.0, 7.0), DVec3::new(0.5, 1.0, 1.5)),
            time,
            Earth,
            Icrf,
        );
        let diff = s1 - s2;
        assert_approx_eq!(diff.position(), DVec3::new(7.0, 15.0, 23.0));
        assert_approx_eq!(diff.velocity(), DVec3::new(0.5, 1.0, 1.5));
    }

    fn ephemeris() -> &'static Spk {
        let contents = std::fs::read(data_file("spice/de440s.bsp")).unwrap();
        static EPHEMERIS: OnceLock<Spk> = OnceLock::new();
        EPHEMERIS.get_or_init(|| parse_daf_spk(&contents).unwrap())
    }
}
