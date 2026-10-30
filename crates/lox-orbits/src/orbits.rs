// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

/// Builder patterns for constructing orbits from orbital elements.
pub mod builders;
mod cartesian;
/// Collections of named trajectories.
pub mod ensemble;
mod equinoctial;
mod keplerian;
mod modified_equinoctial;
/// Sun-synchronous orbit construction.
pub mod sso;
mod trajectory;

pub use cartesian::{StateToGroundError, ToKeplerianError};
pub use ensemble::Ensemble;
pub use equinoctial::{FromEquinoctialError, ToEquinoctialError};
pub use modified_equinoctial::ToModifiedEquinoctialError;
pub use trajectory::{Trajectory, TrajectoryError, TrajectoryTransformationError};

use lox_bodies::{CoordinateOrigin, Origin, PointMass, TryPointMass, UndefinedOriginPropertyError};
use lox_core::{
    coords::Cartesian,
    elements::{Equinoctial, GravitationalParameter, Keplerian, ModifiedEquinoctial},
};
use lox_frames::{Frame, NonQuasiInertialFrameError, ReferenceFrame, TryQuasiInertial};
use lox_time::Time;

/// An orbital state parameterized by state representation, origin, and reference frame.
///
/// The epoch carries its time scale at runtime as a [`Time`]; the orbit layer does
/// not track it in the type system. The origin and frame default to the
/// runtime-determined [`Origin`] and [`Frame`] — name them explicitly,
/// `Orbit<Cartesian, Earth, Icrf>`, to have the compiler track them.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(
        try_from = "OrbitDe<S, O, R>",
        bound(deserialize = "S: serde::Deserialize<'de> + CheckFrame<R>, \
                             O: serde::Deserialize<'de> + CoordinateOrigin, \
                             R: serde::Deserialize<'de> + ReferenceFrame")
    )
)]
pub struct Orbit<S, O: CoordinateOrigin = Origin, R: ReferenceFrame = Frame> {
    state: S,
    time: Time,
    origin: O,
    frame: R,
}

/// Checks that a reference frame suits a type of orbital state.
///
/// This is the single statement of the frame requirement of each state type: Cartesian states
/// are valid in any frame, while orbital elements require a quasi-inertial frame. The fallible
/// constructors, the conversions to orbital elements, and the deserialization of an [`Orbit`]
/// enforce it through this trait. [`Orbit::from_state`] does not check it, so callers using it
/// are responsible for a suitable frame.
pub trait CheckFrame<R> {
    /// Checks that `frame` suits this type of state.
    ///
    /// # Errors
    ///
    /// Returns a [`NonQuasiInertialFrameError`] if the state requires a quasi-inertial frame and
    /// `frame` is not.
    fn check_frame(frame: &R) -> Result<(), NonQuasiInertialFrameError>;
}

impl<R> CheckFrame<R> for Cartesian {
    fn check_frame(_frame: &R) -> Result<(), NonQuasiInertialFrameError> {
        Ok(())
    }
}

macro_rules! quasi_inertial_state {
    ($($state:ty),*) => {
        $(
            impl<R: TryQuasiInertial> CheckFrame<R> for $state {
                fn check_frame(frame: &R) -> Result<(), NonQuasiInertialFrameError> {
                    frame.try_quasi_inertial()
                }
            }
        )*
    };
}

quasi_inertial_state!(Keplerian, Equinoctial, ModifiedEquinoctial);

/// Deserialization shadow for [`Orbit`].
///
/// Deserializing into the public type directly would bypass the frame check of
/// [`CheckFrame`].
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
#[serde(rename = "Orbit")]
struct OrbitDe<S, O, R> {
    state: S,
    time: Time,
    origin: O,
    frame: R,
}

#[cfg(feature = "serde")]
impl<S, O, R> TryFrom<OrbitDe<S, O, R>> for Orbit<S, O, R>
where
    S: CheckFrame<R>,
    O: CoordinateOrigin,
    R: ReferenceFrame,
{
    type Error = NonQuasiInertialFrameError;

    fn try_from(de: OrbitDe<S, O, R>) -> Result<Self, Self::Error> {
        S::check_frame(&de.frame)?;
        Ok(Orbit::from_state(de.state, de.time, de.origin, de.frame))
    }
}

impl<S, O, R> Orbit<S, O, R>
where
    O: CoordinateOrigin,
    R: ReferenceFrame,
{
    /// Constructs an orbit from its state, epoch, origin, and reference frame.
    #[inline]
    pub const fn from_state(state: S, time: Time, origin: O, frame: R) -> Self {
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
    pub fn time(&self) -> Time {
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

    /// Returns an orbit with the same epoch, origin, and frame but a different state.
    pub(crate) fn with_state<S1>(&self, state: S1) -> Orbit<S1, O, R>
    where
        O: Copy,
        R: Copy,
    {
        Orbit::from_state(state, self.time, self.origin, self.frame)
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

impl<S, O, R> Orbit<S, O, R>
where
    O: CoordinateOrigin + Copy + Into<Origin>,
    R: ReferenceFrame + Copy + Into<Frame>,
{
    /// Converts this orbit into one with a runtime-determined origin and frame.
    ///
    /// The epoch is unaffected: the orbit layer already carries its time scale
    /// at runtime.
    pub fn into_dynamic(self) -> Orbit<S, Origin, Frame> {
        Orbit::from_state(self.state, self.time, self.origin.into(), self.frame.into())
    }
}

/// An orbit with Cartesian position and velocity state.
pub type CartesianOrbit<O = Origin, R = Frame> = Orbit<Cartesian, O, R>;

/// An orbit with classical Keplerian elements state.
pub type KeplerianOrbit<O = Origin, R = Frame> = Orbit<Keplerian, O, R>;

/// An orbit with equinoctial elements state.
pub type EquinoctialOrbit<O = Origin, R = Frame> = Orbit<Equinoctial, O, R>;

/// An orbit with modified equinoctial elements state.
pub type ModifiedEquinoctialOrbit<O = Origin, R = Frame> = Orbit<ModifiedEquinoctial, O, R>;
