// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

pub mod sso;

use glam::DVec3;
use lox_bodies::{DynOrigin, Origin, PointMass, TryPointMass, UndefinedOriginPropertyError};
use lox_core::{
    anomalies::TrueAnomaly,
    coords::{Cartesian, CartesianTrajectory, TimeStampedCartesian},
    elements::{
        ArgumentOfPeriapsis, Eccentricity, GravitationalParameter, Inclination, Keplerian,
        LongitudeOfAscendingNode,
    },
};
use lox_frames::{
    DynFrame, NonQuasiInertialFrameError, QuasiInertial, ReferenceFrame, TryQuasiInertial,
    traits::frame_id, transformations::TryTransform,
};
use lox_time::{
    Time,
    deltas::TimeDelta,
    time_scales::{DynTimeScale, TimeScale},
};
use lox_units::Distance;

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
        let grav_param = GravitationalParameter::km3_per_s2(self.origin.gravitational_parameter());
        let keplerian = self.state.to_keplerian(grav_param);
        Orbit {
            state: keplerian,
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
        let grav_param =
            GravitationalParameter::km3_per_s2(self.origin.try_gravitational_parameter()?);
        let keplerian = self.state.to_keplerian(grav_param);
        Ok(Orbit {
            state: keplerian,
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
        let grav_param = GravitationalParameter::km3_per_s2(self.origin.gravitational_parameter());
        let cartesian = self.state.to_cartesian(grav_param);
        Orbit {
            state: cartesian,
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
        let grav_param =
            GravitationalParameter::km3_per_s2(self.origin.try_gravitational_parameter()?);
        let cartesian = self.state.to_cartesian(grav_param);
        Ok(Orbit {
            state: cartesian,
            time: self.time,
            origin: self.origin,
            frame: self.frame,
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
        let t = (time - self.epoch).as_seconds_f64();
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
        P: TryTransform<R, R1, T>,
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
                    .try_transform(self.frame, frame, t)
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
