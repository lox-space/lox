// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

/// Builder patterns for constructing orbits from orbital elements.
pub mod builders;
mod cartesian;
/// Collections of named trajectories.
pub mod ensemble;
mod keplerian;
/// Sun-synchronous orbit construction.
pub mod sso;
mod trajectory;

pub use cartesian::StateToDynGroundError;
pub use ensemble::{DynEnsemble, Ensemble};
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

/// The state representation of an orbit, either Cartesian or Keplerian.
pub enum OrbitType {
    /// Cartesian position and velocity state.
    Cartesian(Cartesian),
    /// Classical Keplerian orbital elements.
    Keplerian(Keplerian),
}

/// An orbital state parameterized by state representation, time scale, origin, and reference frame.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Orbit<S, T: TimeScale, O: Origin, R: ReferenceFrame> {
    state: S,
    time: Time<T>,
    origin: O,
    frame: R,
}

/// A dynamically-typed orbit with runtime time scale, origin, and frame.
pub type DynOrbit = Orbit<OrbitType, DynTimeScale, DynOrigin, DynFrame>;

impl<S, T, O, R> Orbit<S, T, O, R>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    /// Constructs an orbit from its state, epoch, origin, and reference frame.
    #[inline]
    pub const fn from_state(state: S, time: Time<T>, origin: O, frame: R) -> Self {
        Self {
            state,
            time,
            origin,
            frame,
        }
    }

    /// Returns the orbital state.
    #[inline]
    pub fn state(&self) -> S
    where
        S: Copy,
    {
        self.state
    }

    /// Returns the epoch of this orbit.
    #[inline]
    pub fn time(&self) -> Time<T>
    where
        T: Copy,
    {
        self.time
    }

    /// Returns the central body origin.
    #[inline]
    pub fn origin(&self) -> O
    where
        O: Copy,
    {
        self.origin
    }

    /// Returns the reference frame.
    #[inline]
    pub fn reference_frame(&self) -> R
    where
        R: Copy,
    {
        self.frame
    }

    /// Returns the gravitational parameter of the origin, or an error if undefined.
    pub fn try_gravitational_parameter(
        &self,
    ) -> Result<GravitationalParameter, UndefinedOriginPropertyError>
    where
        O: TryPointMass,
    {
        self.origin.try_gravitational_parameter()
    }

    /// Returns the gravitational parameter of the origin.
    pub fn gravitational_parameter(&self) -> GravitationalParameter
    where
        O: PointMass,
    {
        self.origin.gravitational_parameter()
    }
}

impl<S, T, O, R> Orbit<S, T, O, R>
where
    T: TimeScale + Copy + Into<DynTimeScale>,
    O: Origin + Copy + Into<DynOrigin>,
    R: ReferenceFrame + Copy + Into<DynFrame>,
{
    /// Converts this orbit into a dynamically-typed orbit.
    pub fn into_dyn(self) -> Orbit<S, DynTimeScale, DynOrigin, DynFrame> {
        Orbit::from_state(
            self.state,
            self.time.into_dyn(),
            self.origin.into(),
            self.frame.into(),
        )
    }
}

/// An orbit with Cartesian position and velocity state.
pub type CartesianOrbit<T, O, R> = Orbit<Cartesian, T, O, R>;
/// A dynamically-typed Cartesian orbit.
pub type DynCartesianOrbit = Orbit<Cartesian, DynTimeScale, DynOrigin, DynFrame>;

/// An orbit with classical Keplerian elements state.
pub type KeplerianOrbit<T, O, R> = Orbit<Keplerian, T, O, R>;
/// A dynamically-typed Keplerian orbit.
pub type DynKeplerianOrbit = Orbit<Keplerian, DynTimeScale, DynOrigin, DynFrame>;

/// Errors that can occur when constructing a trajectory.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TrajectorError {
    /// Too few states were provided to construct a trajectory.
    #[error("at least 2 states are required but only {0} were provided")]
    InsufficientStates(usize),
}
