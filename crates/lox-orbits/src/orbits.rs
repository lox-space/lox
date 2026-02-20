// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

mod cartesian;
mod keplerian;
pub mod sso;
mod trajectory;

pub use cartesian::StateToDynGroundError;
pub use trajectory::{DynTrajectory, Trajectory, TrajectoryError, TrajectoryTransformationError};

use lox_bodies::{DynOrigin, Origin, PointMass, TryPointMass, UndefinedOriginPropertyError};
use lox_core::{
    coords::Cartesian,
    elements::{GravitationalParameter, Keplerian},
};
use lox_frames::{DynFrame, ReferenceFrame};
use lox_time::{
    Time,
    time_scales::{DynTimeScale, TimeScale},
};
use thiserror::Error;

pub enum OrbitType {
    Cartesian(Cartesian),
    Keplerian(Keplerian),
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        self.origin.try_gravitational_parameter()
    }

    pub fn gravitational_parameter(&self) -> GravitationalParameter
    where
        O: PointMass,
    {
        self.origin.gravitational_parameter()
    }
}

pub type CartesianOrbit<T, O, R> = Orbit<Cartesian, T, O, R>;
pub type DynCartesianOrbit = Orbit<Cartesian, DynTimeScale, DynOrigin, DynFrame>;

pub type KeplerianOrbit<T, O, R> = Orbit<Keplerian, T, O, R>;
pub type DynKeplerianOrbit = Orbit<Keplerian, DynTimeScale, DynOrigin, DynFrame>;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum TrajectorError {
    #[error("at least 2 states are required but only {0} were provided")]
    InsufficientStates(usize),
}
