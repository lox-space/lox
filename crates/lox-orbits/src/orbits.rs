// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

pub mod sso;

use std::{
    f64::consts::{PI, TAU},
    iter::zip,
};

use glam::DVec3;
use lox_bodies::{DynOrigin, Origin, PointMass, TryPointMass, UndefinedOriginPropertyError};
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
use lox_frames::{
    DynFrame, NonQuasiInertialFrameError, QuasiInertial, ReferenceFrame, TryQuasiInertial,
    rotations::TryRotation, traits::frame_id,
};
use lox_time::{
    Time,
    deltas::TimeDelta,
    time_scales::{DynTimeScale, TimeScale},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orbit<S, T: TimeScale, O: Origin, R: ReferenceFrame> {
    state: S,
    time: Time<T>,
    origin: O,
    frame: R,
}

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
}

pub type DynCartesianOrbit = Orbit<Cartesian, DynTimeScale, DynOrigin, DynFrame>;

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
                    time: time_of_flight,
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

#[derive(Debug, Clone)]
pub struct Trajectory<T: TimeScale, O: Origin, R: ReferenceFrame> {
    epoch: Time<T>,
    origin: O,
    frame: R,
    data: CartesianTrajectory,
}

impl<T, O, R> Trajectory<T, O, R>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    pub fn at(&self, time: Time<T>) -> CartesianOrbit<T, O, R>
    where
        T: Copy,
        O: Copy,
        R: Copy,
    {
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
        T: Copy,
        R: Copy,
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
                let dt: TimeDelta = time.into();
                let t = self.epoch + dt;
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
}

pub type DynTrajectory = Trajectory<DynTimeScale, DynOrigin, DynFrame>;
