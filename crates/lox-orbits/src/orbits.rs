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

pub use cartesian::StateToGroundError;
pub use ensemble::Ensemble;
pub use trajectory::{Trajectory, TrajectoryError, TrajectoryTransformationError};

use lox_bodies::{CoordinateOrigin, Origin, PointMass, TryPointMass, UndefinedOriginPropertyError};
use lox_core::{
    coords::Cartesian,
    elements::{GravitationalParameter, Keplerian},
};
use lox_frames::{Frame, ReferenceFrame};
use lox_time::{
    Time,
    time_scales::{ContinuousTimeScale, TimeScale},
};

/// The state representation of an orbit, either Cartesian or Keplerian.
pub enum OrbitType {
    /// Cartesian position and velocity state.
    Cartesian(Cartesian),
    /// Classical Keplerian orbital elements.
    Keplerian(Keplerian),
}

/// An orbital state parameterized by state representation, time scale, origin, and reference frame.
///
/// The time scale, origin, and frame default to the runtime-determined
/// [`TimeScale`], [`Origin`], and [`Frame`]. Name them explicitly —
/// `Orbit<Cartesian, Tai, Earth, Icrf>` — to have the compiler track them.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Orbit<
    S,
    T: ContinuousTimeScale = TimeScale,
    O: CoordinateOrigin = Origin,
    R: ReferenceFrame = Frame,
> {
    state: S,
    time: Time<T>,
    origin: O,
    frame: R,
}

impl<S, T, O, R> Orbit<S, T, O, R>
where
    T: ContinuousTimeScale,
    O: CoordinateOrigin,
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
    T: ContinuousTimeScale + Copy + Into<TimeScale>,
    O: CoordinateOrigin + Copy + Into<Origin>,
    R: ReferenceFrame + Copy + Into<Frame>,
{
    /// Converts this orbit into a dynamically-typed orbit.
    pub fn into_dynamic(self) -> Orbit<S, TimeScale, Origin, Frame> {
        Orbit::from_state(
            self.state,
            self.time.into_dynamic(),
            self.origin.into(),
            self.frame.into(),
        )
    }
}

/// An orbit with Cartesian position and velocity state.
pub type CartesianOrbit<T = TimeScale, O = Origin, R = Frame> = Orbit<Cartesian, T, O, R>;

/// An orbit with classical Keplerian elements state.
pub type KeplerianOrbit<T = TimeScale, O = Origin, R = Frame> = Orbit<Keplerian, T, O, R>;
